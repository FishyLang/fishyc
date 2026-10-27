#!/usr/bin/env python3
"""Small, deterministic compatibility tests for the current Fishy prelude."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile


class CheckFailed(Exception):
    pass


def command(argv, directory, stem, *, stdin="", env=None, timeout=30):
    try:
        process = subprocess.run(argv, cwd=directory, input=stdin, capture_output=True,
                                 text=True, timeout=timeout, env=env)
        result = dict(exit=process.returncode, stdout=process.stdout, stderr=process.stderr)
    except subprocess.TimeoutExpired:
        result = dict(exit="timeout", stdout="", stderr=f"{stem} timed out")
    except OSError as error:
        result = dict(exit="tool-error", stdout="", stderr=str(error))
    for stream in ("stdout", "stderr"):
        (directory / f"{stem}.{stream}").write_text(result[stream])
    return dict(command=[str(arg) for arg in argv], **result)


def check_case(name, case, args, output):
    directory = output / name
    directory.mkdir(parents=True, exist_ok=True)
    stages = {}
    result = dict(pass_=False, stages=stages)

    def run(stem, argv, **kwargs):
        observed = command(argv, directory, stem, **kwargs)
        stages[stem] = observed
        return observed

    def succeed(stem, argv):
        observed = run(stem, argv)
        if observed['exit'] != 0:
            raise CheckFailed(f"{stem}: exit {observed['exit']}\n{observed['stderr']}")

    def fixtures():
        # Every execution gets the same fixture contents, including O2/guard runs.
        for relative, contents in case.get('files', {}).items():
            target = directory / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(contents)

    def execute(obj, stem, guard=False):
        executable = directory / stem
        link = ['cc', obj]
        if guard:
            link += ['guard.o', '-Wl,--wrap=malloc', '-Wl,--wrap=free', '-Wl,--wrap=realloc']
        succeed(stem + '-link', [*link, '-lm', '-o', str(executable)])
        fixtures()
        environment = dict(os.environ)
        environment.update(case.get('env', {}))
        if guard:
            environment['FISHYC_EXPECT_LIVE'] = str(case['heap_live'])
        observed = run(stem, [str(executable), *case.get('argv', [])],
                       stdin=case.get('stdin', ''), env=environment, timeout=5)
        expected = (case.get('exit', 0), case['stdout'], case.get('stderr', ''))
        actual = (observed['exit'], observed['stdout'], observed['stderr'])
        if actual != expected:
            raise CheckFailed(f"{stem}: expected {expected!r}; observed {actual!r}")
        for relative, contents in case.get('expected_files', {}).items():
            path = directory / relative
            if not path.is_file() or path.read_text() != contents:
                raise CheckFailed(f"{stem}: unexpected output file {relative}")

    try:
        source = (args.manifest.parent / case['source_file']).read_text() if 'source_file' in case else case['source']
        (directory / 'source.fih').write_text(source)
        fixtures()
        llvm_o2 = case.get('llvm_o2', False)
        verify = case.get('verify_llvm', False) or llvm_o2
        compile_command = [str(args.binary), 'source.fih', '-o', 'program.o']
        if case.get('custom_opt', False) or args.custom_opt:
            compile_command.append('-O')
        if verify:
            compile_command.append('--emit-llvm')
        observed = run('compile', compile_command, timeout=case.get('compile_timeout', 30))
        if not case.get('compile', True):
            diagnostic = case['diagnostic']
            if (observed['exit'] != 1 or diagnostic not in observed['stderr']
                    or any(marker in observed['stderr'] for marker in
                           ('panicked at', 'Internal Compiler Error', 'Internal IR validation'))
                    or (directory / 'program.o').exists()):
                raise CheckFailed(f"expected clean rejection containing {diagnostic!r}: {observed!r}")
        else:
            if observed['exit'] != 0:
                raise CheckFailed(f"compile: exit {observed['exit']}\n{observed['stderr']}")
            llvm_bin = Path(os.environ.get('LLVM_BIN', '/usr/lib/llvm-17/bin'))
            if verify:
                succeed('verify', [str(llvm_bin / 'opt'), '-passes=verify', '-disable-output', 'source.ll'])
            guard = 'heap_live' in case
            if guard:
                guard_source = Path(__file__).resolve().parent.parent / 'ownership' / 'heap_guard.c'
                succeed('guard-build', ['cc', '-c', str(guard_source), '-o', 'guard.o'])
            execute('program.o', 'native')
            if guard:
                execute('program.o', 'guarded', True)
            if llvm_o2:
                succeed('llvm-opt', [str(llvm_bin / 'opt'), '-passes=default<O2>', '-S',
                                     'source.ll', '-o', 'optimized.ll'])
                succeed('verify-o2', [str(llvm_bin / 'opt'), '-passes=verify',
                                      '-disable-output', 'optimized.ll'])
                succeed('llc', [str(llvm_bin / 'llc'), '-filetype=obj', '-relocation-model=pic',
                               'optimized.ll', '-o', 'optimized.o'])
                execute('optimized.o', 'native-o2')
                if guard:
                    execute('optimized.o', 'guarded-o2', True)
        result['pass_'] = True
    except (CheckFailed, KeyError, OSError, ValueError) as error:
        result['failure'] = str(error)
    result['pass'] = result.pop('pass_')
    (directory / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
    print(f"{name}: {'PASS' if result['pass'] else 'FAIL'}", flush=True)
    if not result['pass']:
        print(result['failure'], flush=True)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, default=Path(__file__).with_name('cases.json'))
    parser.add_argument('--output', type=Path, help='keep sources, objects and per-stage results here')
    parser.add_argument('--case', action='append', help='run one named case; may be repeated')
    parser.add_argument('--custom-opt', action='store_true', help='enable custom optimization for selected cases')
    args = parser.parse_args()
    args.binary = args.binary.resolve()
    args.manifest = args.manifest.resolve()
    cases = json.loads(args.manifest.read_text())
    if args.case:
        unknown = set(args.case) - cases.keys()
        if unknown:
            parser.error(f"unknown cases: {', '.join(sorted(unknown))}")
        cases = {name: case for name, case in cases.items() if name in args.case}
    if not cases:
        parser.error('no cases selected')
    with tempfile.TemporaryDirectory(prefix='fishyc-stdlib-') as temporary:
        output = (args.output or Path(temporary)).resolve()
        output.mkdir(parents=True, exist_ok=True)
        # A unique run directory prevents old objects from affecting repeated runs.
        run_output = Path(tempfile.mkdtemp(prefix='run-', dir=output))
        results = {name: check_case(name, case, args, run_output) for name, case in cases.items()}
        (output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')
        passed = sum(result['pass'] for result in results.values())
        print(f'{passed}/{len(results)} stdlib cases passed')
        if args.output:
            print(f'Artifacts: {run_output}')
        raise SystemExit(0 if passed == len(results) else 1)


if __name__ == '__main__':
    main()
