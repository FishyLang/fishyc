#!/usr/bin/env python3
"""Text-foundation fixtures, using the existing stdlib/ownership runner."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--case', action='append')
    parser.add_argument('--mode', choices=('plain', 'custom-O'))
    args = parser.parse_args()
    here = Path(__file__).resolve().parent
    root = here.parents[1]
    cases = json.loads((here / 'cases.json').read_text())
    if args.case:
        if set(args.case) - cases.keys():
            parser.error('unknown case selected')
        cases = {name: case for name, case in cases.items() if name in args.case}
    for case in cases.values():
        if 'source_file' in case:
            case['source'] = (here / case.pop('source_file')).read_text()
    with tempfile.TemporaryDirectory(prefix='fishyc-text-foundation-') as temporary:
        output = (args.output or Path(temporary)).resolve()
        output.mkdir(parents=True, exist_ok=True)
        manifest = output / 'effective-cases.json'
        manifest.write_text(json.dumps(cases, indent=2) + '\n')
        modes = (args.mode,) if args.mode else ('plain', 'custom-O')
        results = {}
        for mode in modes:
            command = [sys.executable, str(root / 'tests/stdlib/run.py'),
                       '--binary', str(args.binary.resolve()), '--manifest', str(manifest),
                       '--output', str(output / mode)]
            if mode == 'custom-O':
                command.append('--custom-opt')
            completed = subprocess.run(command)
            if completed.returncode:
                raise SystemExit(completed.returncode)
            observed = json.loads((output / mode / 'results.json').read_text())
            results.update({f'{name}/{mode}': result for name, result in observed.items()})
        (output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
        passed = sum(result['pass'] for result in results.values())
        print(f'{passed}/{len(results)} text-foundation configurations passed')
        raise SystemExit(0 if passed == len(cases) * len(modes) else 1)


if __name__ == '__main__':
    main()
