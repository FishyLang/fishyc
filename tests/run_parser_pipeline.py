#!/usr/bin/env python3
"""Parser regressions through source, both IR optimizers, and execution."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]

def command(argv, directory, stem, timeout=15):
    with (directory / (stem + '.stdout')).open('w') as stdout, (directory / (stem + '.stderr')).open('w') as stderr:
        try:
            result = subprocess.run(argv, cwd=directory, stdout=stdout, stderr=stderr,
                                    stdin=subprocess.DEVNULL, timeout=timeout)
            status = result.returncode
        except subprocess.TimeoutExpired:
            status = 'timeout'
    return {'exit': status,
            'stdout': (directory / (stem + '.stdout')).read_text(errors='replace'),
            'stderr': (directory / (stem + '.stderr')).read_text(errors='replace')}

def suite(binary, output):
    cases = json.loads((ROOT / 'tests/parser_cases.json').read_text())
    llvm = Path(os.environ.get('LLVM_BIN', '/usr/lib/llvm-17/bin'))
    results = {}
    for name, expected in cases.items():
        for optimized in (False, True):
            key = name + ('/custom-O' if optimized else '/plain')
            directory = output / key
            directory.mkdir(parents=True, exist_ok=True)
            (directory / 'source.fih').write_text(expected['source'])
            args = [str(binary), 'source.fih', '--dump-ast', '--dump-ir', '--emit-llvm', '-o', 'program.o']
            if optimized:
                args.append('-O')
            result = {'compile': command(args, directory, 'compile')}
            if expected['error']:
                result['pass'] = result['compile']['exit'] == 1 and not any(
                    s in result['compile']['stderr'] for s in ('panicked', 'Internal Compiler Error', 'Internal IR validation'))
            else:
                if result['compile']['exit'] == 0:
                    result['verify'] = command([str(llvm / 'opt'), '-passes=verify', '-disable-output', 'source.ll'], directory, 'verify')
                    result['link'] = command(['cc', 'program.o', '-o', 'program'], directory, 'link')
                    if result['link']['exit'] == 0:
                        result['runtime'] = command([str(directory / 'program')], directory, 'runtime', 3)
                    result['opt'] = command([str(llvm / 'opt'), '-passes=default<O2>', '-S', 'source.ll', '-o', 'llvm-O2.ll'], directory, 'opt')
                    if result['opt']['exit'] == 0:
                        result['llc'] = command([str(llvm / 'llc'), '-filetype=obj', '-relocation-model=pic', 'llvm-O2.ll', '-o', 'llvm-O2.o'], directory, 'llc')
                        if result['llc']['exit'] == 0:
                            link = command(['cc', 'llvm-O2.o', '-o', 'llvm-O2'], directory, 'link-O2')
                            if link['exit'] == 0:
                                result['runtime-O2'] = command([str(directory / 'llvm-O2')], directory, 'runtime-O2', 3)
                result['pass'] = result.get('verify', {}).get('exit') == 0 and all(
                    result.get(kind, {}).get('exit') == expected['exit'] and
                    result.get(kind, {}).get('stdout') == expected['stdout']
                    for kind in ('runtime', 'runtime-O2'))
            (directory / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
            results[key] = result
            print(f"{key}: {'PASS' if result['pass'] else 'FAIL'}", flush=True)
    (output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
    return all(r['pass'] for r in results.values())

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--assert-results', action='store_true')
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='fishyc-parser-suite-') as temporary:
        passed = suite(args.binary.resolve(), (args.output or Path(temporary)).resolve())
        if args.assert_results and not passed:
            raise SystemExit(1)

if __name__ == '__main__':
    main()
