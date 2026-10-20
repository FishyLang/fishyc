# LLVM representation and ABI regressions

Twenty-four individually authored source programs protect the valid typed custom-IR to LLVM boundary. The runner compiles each program with custom optimization off/on, independently verifies LLVM, executes the ordinary object, runs LLVM `default<O2>`, verifies the optimized module, emits another object and executes it. Both executions must match the stated stdout/status; custom modes must agree too.

This gives 48 compiler configurations, 96 explicit LLVM verifications (48 original + 48 optimized modules), and 96 native executions. The external ABI case links its small, fixed C helper to both objects. No allocation instrumentation or source generation is used.

```sh
cargo test
cargo test backend::abi_tests
python3 tests/llvm_abi/run.py --binary target/debug/fishyc --output /tmp/fishy-llvm-results
```

Requires Python 3, `cc`, and LLVM 17 `opt`/`llc`. `LLVM_BIN` overrides `/usr/lib/llvm-17/bin`; missing tools fail the test. Set `FISHYC_LLVM_TEST_OUTPUT` when running Cargo to retain source, AST, custom IR, LLVM before/after O2, objects, executables, logs and JSON results. Otherwise temporary artifacts are removed after testing.

Direct Rust tests in `src/llvm_abi_tests.rs` assert actual LLVM types, target layout, function signatures, enum capacity, word-register boundaries, resolved callable identities and block emission order. The equality regression also executes its valid module before/after LLVM O2 using LLVM's JIT. These tests do not change source semantics or ARC policy.
