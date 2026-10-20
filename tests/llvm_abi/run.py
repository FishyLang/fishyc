#!/usr/bin/env python3
"""Verify hand-authored LLVM ABI cases and compare native/LLVM-O2 results."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import subprocess
import tempfile


def run(command, directory, stem, timeout=15):
    try:
        result = subprocess.run(command, cwd=directory, stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, timeout=timeout)
        value = dict(exit=result.returncode, stdout=result.stdout, stderr=result.stderr)
    except subprocess.TimeoutExpired:
        value = dict(exit='timeout', stdout='', stderr=f'{stem} timed out')
    (directory / (stem + '.stdout')).write_text(value['stdout'])
    (directory / (stem + '.stderr')).write_text(value['stderr'])
    return value


def check_case(task):
    binary, output, name, expected = task
    modes = {}
    for mode in ('plain', 'custom-O'):
        directory = output / name / mode
        directory.mkdir(parents=True, exist_ok=True)
        (directory / 'source.fih').write_text(expected['source'])
        for path, source in expected.get('files', {}).items():
            target = directory / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(source)
        command = [str(binary), 'source.fih', '--dump-ast', '--dump-ir', '--emit-llvm', '-o', 'program.o']
        if mode == 'custom-O':
            command.append('-O')
        entry = {'compile': run(command, directory, 'compiler')}
        llvm_bin = Path(os.environ.get('LLVM_BIN', '/usr/lib/llvm-17/bin'))
        if entry['compile']['exit'] == 0:
            entry['verify'] = run([str(llvm_bin / 'opt'), '-passes=verify',
                                   '-disable-output', 'source.ll'], directory, 'verify')
            helpers = []
            if (directory / 'helper.c').exists():
                entry['helper'] = run(['cc', '-c', 'helper.c', '-o', 'helper.o'], directory, 'helper')
                helpers = ['helper.o']
            entry['link'] = run(['cc', 'program.o', *helpers, '-lm', '-o', 'program'], directory, 'link')
            if entry['link']['exit'] == 0:
                entry['runtime'] = run([str(directory / 'program')], directory, 'runtime', 3)
            entry['llvm_opt'] = run([str(llvm_bin / 'opt'), '-passes=default<O2>',
                                     '-S', 'source.ll', '-o', 'optimized.ll'], directory, 'llvm-opt')
            if entry['llvm_opt']['exit'] == 0:
                entry['verify_O2'] = run([str(llvm_bin / 'opt'), '-passes=verify',
                                          '-disable-output', 'optimized.ll'], directory, 'verify-O2')
                entry['llc'] = run([str(llvm_bin / 'llc'), '-filetype=obj', '-relocation-model=pic',
                                    'optimized.ll', '-o', 'optimized.o'], directory, 'llc')
                if entry['llc']['exit'] == 0:
                    entry['link_O2'] = run(['cc', 'optimized.o', *helpers, '-lm', '-o', 'optimized'], directory, 'link-O2')
                    if entry['link_O2']['exit'] == 0:
                        entry['runtime_O2'] = run([str(directory / 'optimized')], directory, 'runtime-O2', 3)
        entry['pass'] = (entry['compile']['exit'] == 0
                         and all(entry.get(k, {}).get('exit') == 0 for k in ('verify', 'llvm_opt', 'verify_O2'))
                         and all(entry.get(k, {}).get('exit') == expected['exit']
                                 and entry.get(k, {}).get('stdout') == expected['stdout']
                                 for k in ('runtime', 'runtime_O2')))
        modes[mode] = entry
    observed = [modes[m].get('runtime', {}) for m in ('plain', 'custom-O')]
    equivalent = ((observed[0].get('exit'), observed[0].get('stdout'))
                  == (observed[1].get('exit'), observed[1].get('stdout')))
    for mode, entry in modes.items():
        entry['pass'] = entry['pass'] and equivalent
        print(f"{name}/{mode}: {'PASS' if entry['pass'] else 'FAIL'}", flush=True)
        if not entry['pass']:
            print(json.dumps(entry, indent=2), flush=True)
        (output / name / mode / 'result.json').write_text(json.dumps(entry, indent=2) + '\n')
    return {name + '/' + mode: entry for mode, entry in modes.items()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--manifest', type=Path, default=Path(__file__).with_name('cases.json'))
    args = parser.parse_args()
    cases = json.loads(args.manifest.read_text())
    with tempfile.TemporaryDirectory(prefix='fishyc-llvm-abi-') as temporary:
        output = (args.output or Path(temporary)).resolve()
        tasks = [(args.binary.resolve(), output, name, case) for name, case in cases.items()]
        with ThreadPoolExecutor(max_workers=4) as pool:
            results = {}
            for result in pool.map(check_case, tasks):
                results.update(result)
        (output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
        passed = sum(result['pass'] for result in results.values())
        print(f'{passed}/{len(results)} configurations passed')
        raise SystemExit(0 if passed == len(results) else 1)


if __name__ == '__main__':
    main()
