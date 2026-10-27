#!/usr/bin/env python3
"""Report allocation sanity measurements; no benchmark or fixed count assertions."""
import argparse
import os
from pathlib import Path
import subprocess
import tempfile
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--source', type=Path, default=Path(__file__).with_name('profile.fih'))
    parser.add_argument('--custom-opt', action='store_true')
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='fishyc-string-profile-') as temporary:
        directory = (args.output or Path(temporary)).resolve()
        directory.mkdir(parents=True, exist_ok=True)
        (directory / 'source.fih').write_text(args.source.read_text())
        commands = [
            [str(args.binary.resolve()), 'source.fih', '--emit-llvm', '--dump-ir', '-o', 'program.o'],
            ['cc', '-c', str(Path(__file__).with_name('profile_guard.c').resolve()), '-o', 'guard.o'],
            ['cc', 'program.o', 'guard.o', '-Wl,--wrap=malloc', '-Wl,--wrap=free',
             '-Wl,--wrap=realloc', '-lm', '-o', 'profile'],
        ]
        if args.custom_opt:
            commands[0].append('-O')
        for index, command in enumerate(commands):
            observed = subprocess.run(command, cwd=directory, capture_output=True, text=True, timeout=30)
            (directory / f'stage-{index}.stdout').write_text(observed.stdout)
            (directory / f'stage-{index}.stderr').write_text(observed.stderr)
            if observed.returncode:
                raise SystemExit(observed.stderr)
        start = time.monotonic()
        observed = subprocess.run([str(directory / 'profile')], cwd=directory, capture_output=True,
                                  text=True, env=dict(os.environ, FISHYC_EXPECT_LIVE='0'), timeout=120)
        elapsed = time.monotonic() - start
        (directory / 'profile.stdout').write_text(observed.stdout)
        (directory / 'profile.stderr').write_text(observed.stderr)
        print('measure rows: size operation allocation_requests live_allocations live_requested_bytes'
              if args.source.name == 'representation.fih' else
              'label cumulative_allocations live_allocations live_requested_bytes')
        print(observed.stdout, end='')
        print(f'native_seconds {elapsed:.6f}')
        if observed.returncode:
            raise SystemExit(observed.stderr)


if __name__ == '__main__':
    main()
