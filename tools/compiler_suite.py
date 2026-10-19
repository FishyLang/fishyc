#!/usr/bin/env python3
"""Reproducible source/custom-IR/LLVM/runtime differential compiler suite."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]

def run(command, directory, stem, timeout=15):
    stdout = directory / (stem + '.stdout')
    stderr = directory / (stem + '.stderr')
    with stdout.open('w') as out, stderr.open('w') as err:
        try:
            result = subprocess.run(command, cwd=directory, stdout=out, stderr=err,
                                    stdin=subprocess.DEVNULL, timeout=timeout)
            code = result.returncode
        except subprocess.TimeoutExpired:
            code = 'timeout'
    return {'exit': code, 'stdout': stdout.read_text(errors='replace')[:65536],
            'stderr': stderr.read_text(errors='replace')[:65536]}

def suite(binary, output, names=None):
    manifest = json.loads((ROOT / 'tests/fixtures/manifest.json').read_text())
    llvm_bin = Path(os.environ.get('LLVM_BIN', '/usr/lib/llvm-17/bin'))
    results = {}
    for name, expected in manifest.items():
        if names and name not in names:
            continue
        for optimized in (False, True):
            key = name + ('/custom-O' if optimized else '/plain')
            directory = output / key
            directory.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / 'tests/fixtures' / (name + '.fih'), directory / 'source.fih')
            command = [str(binary), 'source.fih', '--dump-ir', '--emit-llvm', '-o', 'program.o']
            if optimized:
                command.append('-O')
            entry = {'expected': expected, 'compile': run(command, directory, 'compiler')}
            for original, renamed in [('source.ir', 'custom-ir.txt'), ('source.ll', 'llvm-unoptimized.ll')]:
                if (directory / original).exists():
                    (directory / original).rename(directory / renamed)
            llvm = directory / 'llvm-unoptimized.ll'
            if llvm.exists():
                entry['verify'] = run([str(llvm_bin/'opt'), '-passes=verify', '-disable-output', str(llvm)], directory, 'verify')
                if entry['verify']['exit'] == 0:
                    entry['llvm_opt'] = run([str(llvm_bin/'opt'), '-passes=default<O2>', '-S', str(llvm), '-o', 'llvm-optimized.ll'], directory, 'llvm-opt')
            if entry['compile']['exit'] == 0:
                entry['link'] = run(['cc', 'program.o', '-o', 'program'], directory, 'link')
                if entry['link']['exit'] == 0:
                    entry['runtime'] = run([str(directory/'program')], directory, 'runtime', 2)
                if entry.get('llvm_opt', {}).get('exit') == 0:
                    llc = run([str(llvm_bin/'llc'), '-filetype=obj', '-relocation-model=pic', 'llvm-optimized.ll', '-o', 'llvm-O2.o'], directory, 'llc')
                    if llc['exit'] == 0:
                        link = run(['cc', 'llvm-O2.o', '-o', 'llvm-O2'], directory, 'llvm-link')
                        if link['exit'] == 0:
                            entry['llvm_runtime'] = run([str(directory/'llvm-O2')], directory, 'llvm-runtime', 2)
            if expected['error']:
                entry['pass'] = entry['compile']['exit'] == 1 and not any(marker in entry['compile']['stderr'] for marker in ('panicked','Internal Compiler Error','Internal IR validation'))
            else:
                entry['pass'] = entry.get('verify', {}).get('exit') == 0 and all(
                    entry.get(kind, {}).get('exit') == expected['exit'] and
                    entry.get(kind, {}).get('stdout') == expected['stdout']
                    for kind in ('runtime', 'llvm_runtime'))
            (directory/'result.json').write_text(json.dumps(entry, indent=2)+'\n')
            results[key] = entry
            print(f"{key}: {'PASS' if entry['pass'] else 'FAIL'}", flush=True)
    output.mkdir(parents=True, exist_ok=True)
    (output/'results.json').write_text(json.dumps(results, indent=2)+'\n')
    return results

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--assert-results', action='store_true')
    parser.add_argument('--cases', nargs='*')
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='fishyc-suite-') as tmp:
        results = suite(args.binary.resolve(), (args.output or Path(tmp)).resolve(), args.cases)
        failures = [name for name, result in results.items() if not result['pass']]
        print(f"{len(results)-len(failures)}/{len(results)} passed")
        if args.assert_results and failures:
            raise SystemExit(1)

if __name__ == '__main__':
    main()
