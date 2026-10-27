# Synchronous standard-library checks

`python3 tests/stdlib_expansion/run.py --binary target/debug/fishyc` checks the
monolithic prelude in both custom optimization modes. Use `--case core`,
`--case numbers` or `--case output` to focus, and `--output PATH` to retain results.
Cargo discovers the same suite through `stdlib_expansion_validation`.

The three fixtures cover Option/Result callbacks, Vec conveniences, persistent
List/String ownership, ASCII/text operations, checked integer conversions,
managed integer/bool formatting and counted stdout/stderr (including embedded
NUL). Six configurations run custom-IR verification, independent LLVM verification,
LLVM O2, native execution and zero-live allocator accounting.

Requirements: Python 3, LLVM 17 opt/llc and a system linker driver. No HTTP server,
TLS fixture, curl dependency, thread runtime or Fishy-owned C runtime is used.
The old JSON/HTTP/async/concurrency fixtures were deliberately deleted.
