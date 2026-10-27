#!/usr/bin/env python3
"""Verify the synchronous ARC contract without a Fishy-owned native runtime."""
import argparse
import json
import os
from pathlib import Path
import re
import resource
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
LLVM = Path(os.environ.get('LLVM_BIN', '/usr/lib/llvm-17/bin'))

# Test-only LLVM fault injection. The payload's existing header is compiler-owned;
# no C helper or production ABI is introduced by this probe.
PROBE = '''
define i32 @arc_probe(ptr %value, i32 %mode) {
entry:
  call void @__fishy_retain(ptr null)
  call void @__fishy_release(ptr null)
  %count = getelementptr i64, ptr %value, i64 -2
  switch i32 %mode, label %done [i32 2, label %overflow
                               i32 3, label %underflow
                               i32 4, label %resurrection]
overflow:
  store i64 9223372036854775807, ptr %count, align 8
  call void @__fishy_retain(ptr %value)
  unreachable
underflow:
  store i64 0, ptr %count, align 8
  call void @__fishy_release(ptr %value)
  unreachable
resurrection:
  store i64 0, ptr %count, align 8
  call void @__fishy_retain(ptr %value)
  unreachable
done:
  call void @__fishy_release(ptr %value)
  ret i32 0
}
'''


def run(command, directory, stem, expected=0, env=None):
    result = subprocess.run([str(x) for x in command], cwd=directory,
                            capture_output=True, text=True, timeout=60, env=env)
    (directory / (stem + '.stdout')).write_text(result.stdout)
    (directory / (stem + '.stderr')).write_text(result.stderr)
    assert result.returncode == expected, (stem, result.returncode, result.stderr)
    return result


def validate(binary, output, mode, sanitizers, prepare_only):
    directory = output / mode
    directory.mkdir(parents=True, exist_ok=True)
    (directory / 'source.fih').write_text(Path(__file__).with_name('ownership.fih').read_text())
    run([binary, 'source.fih', '--emit-llvm', '-o', 'program.o', *(['-O'] if mode == 'custom-O' else [])], directory, 'compile')
    source = (directory / 'source.ll').read_text()
    for name in ('__fishy_retain', '__fishy_release'):
        body = re.search(r'define internal void @' + name + r'\([^\n]*\).*?^}', source, re.M | re.S)
        assert body, name
        assert not re.search(r'\b(atomic|atomicrmw|cmpxchg|fence)\b', body[0]), name
        assert 'llvm.trap' in body[0], name
    # External dependencies are inspected from the emitted module, not inferred
    # from obsolete source registries or matching method names.
    imports = re.findall(r'^declare [^\n]*?@([^ (]+)', source, re.M)
    assert not any(n.startswith(('fishy_', '__fishy_')) for n in imports), imports
    assert not any(n.startswith(('pthread_', 'curl_', 'epoll_', 'poll')) for n in imports), imports
    harness = re.sub(r'^declare i32 @arc_probe\([^\n]*\)\n', '', source, flags=re.M) + PROBE
    (directory / 'harness.ll').write_text(harness)
    run([LLVM / 'opt', '-passes=verify', '-disable-output', 'harness.ll'], directory, 'verify')
    run(['cc', '-c', ROOT / 'tests/ownership/heap_guard.c', '-o', 'guard.o'], directory, 'guard')
    for suffix, module in [('plain', 'harness.ll'), ('O2', 'optimized.ll')]:
        if suffix == 'O2':
            run([LLVM / 'opt', '-passes=default<O2>', '-S', 'harness.ll', '-o', module], directory, 'optimize')
            run([LLVM / 'opt', '-passes=verify', '-disable-output', module], directory, 'verify-O2')
        run([LLVM / 'llc', '-filetype=obj', '-relocation-model=pic', module, '-o', suffix + '.o'], directory, 'llc-' + suffix)
        run(['cc', suffix + '.o', 'guard.o', '-Wl,--wrap=malloc', '-Wl,--wrap=free', '-Wl,--wrap=realloc', '-lm', '-o', suffix], directory, 'link-' + suffix)
        result = run([directory / suffix], directory, 'run-' + suffix, env=dict(os.environ, FISHYC_EXPECT_LIVE='0'))
        assert result.stdout == 'owned 42\n', result.stdout
        for count, label in [(1, 'overflow'), (2, 'underflow'), (3, 'resurrection')]:
            run([directory / suffix, *(['fault'] * count)], directory, suffix + '-' + label, expected=-4)
    if sanitizers:
        instrumented = re.sub(r'^(define[^\n]*?)\s*\{', r'\1 sanitize_address {', harness, flags=re.M)
        (directory / 'asan.ll').write_text(instrumented)
        run([LLVM / 'opt', '-passes=asan', '-S', 'asan.ll', '-o', 'asan-instrumented.ll'], directory, 'asan-instrument')
        run([LLVM / 'opt', '-passes=verify', '-disable-output', 'asan-instrumented.ll'], directory, 'asan-verify')
        run([LLVM / 'llc', '-filetype=obj', '-relocation-model=pic', 'asan-instrumented.ll', '-o', 'asan.o'], directory, 'asan-llc')
        run(['cc', '-fsanitize=address', 'asan.o', '-lm', '-o', 'asan'], directory, 'asan-link')
        if not prepare_only:
            assert run([directory / 'asan'], directory, 'asan-run').stdout == 'owned 42\n'
    print(mode + ': PASS (LLVM verify/O2, native, zero live allocations, checked ARC)', flush=True)
    return dict(pass_=True, imports=imports, sanitizer='prepared' if prepare_only else 'passed' if sanitizers else 'not requested')


def main():
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--sanitizers', action='store_true')
    parser.add_argument('--prepare-only', action='store_true')
    args = parser.parse_args()
    assert not (ROOT / 'build.rs').exists(), 'retired native build script returned'
    assert not [p for p in (ROOT / 'src').rglob('*') if p.suffix in ('.c', '.h', '.inc')], 'production C machinery returned'
    with tempfile.TemporaryDirectory(prefix='fishyc-sync-') as temporary:
        output = (args.output or Path(temporary)).resolve()
        results = {mode: validate(args.binary.resolve(), output, mode, args.sanitizers, args.prepare_only)
                   for mode in ('plain', 'custom-O')}
        (output / 'results.json').write_text(json.dumps(results, indent=2) + '\n')


if __name__ == '__main__':
    main()
