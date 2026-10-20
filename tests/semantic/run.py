#!/usr/bin/env python3
"""Check hand-authored Fishy regressions without entering IR or code generation."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile


def suite(binary, manifest, output):
    cases = json.loads(manifest.read_text())
    results = {}
    for name, case in cases.items():
        directory = output / name
        directory.mkdir(parents=True, exist_ok=True)
        (directory / 'source.fih').write_text(case['source'])
        for path, source in case.get('files', {}).items():
            target = directory / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(source)
        try:
            process = subprocess.run([str(binary), 'source.fih', '--check'],
                                     cwd=directory, capture_output=True, text=True,
                                     stdin=subprocess.DEVNULL, timeout=15)
            status, stdout, stderr = process.returncode, process.stdout, process.stderr
        except subprocess.TimeoutExpired:
            status, stdout, stderr = 'timeout', '', 'semantic check timed out'
        expected_status = 1 if case['error'] else 0
        passed = (status == expected_status
                  and case.get('diagnostic', '') in stderr
                  and not any(marker in stderr for marker in
                              ('panicked', 'Internal Compiler Error', 'Internal IR validation'))
                  and (case['error'] or 'Checking successful.' in stdout)
                  and not any(directory.glob('*.o'))
                  and not any(directory.glob('*.ll')))
        results[name] = {'pass': passed, 'exit': status, 'stdout': stdout, 'stderr': stderr}
        print(f"{name}: {'PASS' if passed else 'FAIL'}", flush=True)
        if not passed:
            print(f"expected exit {expected_status}, got {status}\n{stderr}", flush=True)
    (output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
    failed = [name for name, result in results.items() if not result['pass']]
    print(f'{len(results) - len(failed)}/{len(results)} passed; {sum(c["error"] for c in cases.values())} rejection cases')
    return not failed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, default=Path(__file__).with_name('cases.json'))
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='fishyc-semantics-') as temporary:
        success = suite(args.binary.resolve(), args.manifest.resolve(), args.output or Path(temporary))
    raise SystemExit(0 if success else 1)


if __name__ == '__main__':
    main()
