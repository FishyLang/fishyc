# Current ownership / ARC contracts

Thirty-eight individually authored programs exercise Fishy's existing acyclic ARC model. They cover local, argument, return, temporary, aggregate, pattern, capture, reference, replacement and global owner edges, plus the existing explicit Vec and C-string/raw-pointer contracts.

For each case, custom optimization off/on runs independent LLVM verification and ordinary/LLVM-O2 execution. Both objects are also linked against `heap_guard.c`, copied unchanged from the repository's existing `internal/tests/completeness/heap_guard.c`. Guarded runs require the same stdout/status plus zero remaining allocations. This is existing test accounting, not a new ownership model or instrumentation design.

The corpus has 76 compiler configurations, 152 explicit LLVM verification commands, 152 ordinary/O2 executions and 152 guarded ordinary/O2 executions. The Rust tests in `src/ownership_tests.rs` independently inspect retain/release counts, pins and ordering so a zero final allocation count cannot conceal duplicated cleanup or premature destruction.

```sh
cargo test
cargo test ir_builder::ownership_tests
python3 tests/ownership/run.py --binary target/debug/fishyc --output /tmp/fishy-ownership-results
```

Requires Python 3, `cc`, GNU-compatible linker wrapping and LLVM 17 `opt`/`llc`. `LLVM_BIN` overrides `/usr/lib/llvm-17/bin`. `FISHYC_OWNERSHIP_TEST_OUTPUT` preserves Cargo integration artifacts; the runner's `--output` preserves source, AST, custom IR, LLVM before/after O2, objects, executable and JSON/log results. Missing tools fail rather than count as passes.

Managed handles are released automatically. Vec's backing allocation still requires `drop`; raw malloc pointers and allocated C strings still require their existing explicit frees. No dangling raw references or cyclic ownership graphs are used as tests.
