#!/usr/bin/env python3
"""Execute hand-authored custom-IR contracts with custom optimization off/on."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
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
        command = [str(binary), 'source.fih', '--dump-ast', '--dump-ir', '-o', 'program.o']
        if mode == 'custom-O':
            command.append('-O')
        entry = {'compile': run(command, directory, 'compiler')}
        if entry['compile']['exit'] == 0:
            entry['link'] = run(['cc', 'program.o', '-lm', '-o', 'program'], directory, 'link')
            if entry['link']['exit'] == 0:
                entry['runtime'] = run([str(directory / 'program')], directory, 'runtime', 3)
        entry['pass'] = (entry['compile']['exit'] == 0
                         and entry.get('runtime', {}).get('exit') == expected['exit']
                         and entry.get('runtime', {}).get('stdout') == expected['stdout'])
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
    with tempfile.TemporaryDirectory(prefix='fishyc-custom-ir-') as temporary:
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
