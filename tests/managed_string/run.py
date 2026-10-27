#!/usr/bin/env python3
"""Acceptance tests for stable String/ByteBuffer and prototype compatibility."""
import argparse
import json
import re
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--case', action='append')
    parser.add_argument('--mode', choices=('plain', 'custom-O'), help='default: both')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[2]
    cases = json.loads(Path(__file__).with_name('cases.json').read_text())
    if args.case:
        if set(args.case) - cases.keys():
            parser.error('unknown case selected')
        cases = {name: case for name, case in cases.items() if name in args.case}
    for case in cases.values():
        if 'doc_source' in case:
            document = (Path(__file__).parent / case.pop('doc_source')).read_text()
            examples = re.findall(r'```fishy\n(.*?)```', document, re.S)
            case['source'] = examples[case.pop('doc_example')]
    with tempfile.TemporaryDirectory(prefix='fishyc-managed-string-') as temporary:
        output = (args.output or Path(temporary)).resolve()
        output.mkdir(parents=True, exist_ok=True)
        manifest = output / 'effective-cases.json'
        manifest.write_text(json.dumps(cases, indent=2) + '\n')
        results = {}
        modes = (args.mode,) if args.mode else ('plain', 'custom-O')
        for mode in modes:
            command = [sys.executable, str(root / 'tests' / 'stdlib' / 'run.py'),
                       '--binary', str(args.binary.resolve()), '--manifest', str(manifest),
                       '--output', str(output / mode)]
            if mode == 'custom-O':
                command.append('--custom-opt')
            completed = subprocess.run(command)
            result_file = output / mode / 'results.json'
            # Never accept an older summary after a child runner failure.
            if completed.returncode != 0:
                raise SystemExit(completed.returncode)
            if not result_file.exists():
                raise SystemExit('stdlib runner produced no results')
            observed = json.loads(result_file.read_text())
            results.update({f'{name}/{mode}': result for name, result in observed.items()})
        (output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
        passed = sum(result['pass'] for result in results.values())
        print(f'{passed}/{len(results)} managed String configurations passed')
        raise SystemExit(0 if passed == len(cases) * len(modes) else 1)


if __name__ == '__main__':
    main()
