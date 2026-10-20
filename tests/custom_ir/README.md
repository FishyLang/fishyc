# Checked frontend / custom IR contracts

`cases.json` contains 25 individually authored deterministic Fishy programs.
`run.py` compiles each with custom optimization disabled and enabled, preserves
checked AST/custom IR dumps, links, and compares native stdout/status against
both the expectation and the other mode. No program generation or mutation is
performed. LLVM lowering is used only to execute the resulting custom IR.

```sh
cargo test
cargo test ir_builder::contract_tests
cargo test --test custom_ir_validation
python3 tests/custom_ir/run.py --binary target/debug/fishyc --output /tmp/fishy-ir-results
```

`FISHYC_IR_TEST_OUTPUT=/tmp/fishy-ir-results cargo test --test custom_ir_validation`
retains the integration artifacts. Python 3 and a C linker are required in
addition to the compiler's existing LLVM build dependencies. The new runner
does not run LLVM optimization or add allocation instrumentation.

Eighteen direct Rust tests in `src/custom_ir_tests.rs` check materialized
merge types, exact call targets, full default arity, scalar storage, generic
SizeOf operands, optimizer register/entry identities, side effects, and precise
rejection of hand-written invalid IR. The malformed IR cases stop at the custom
verifier and never enter LLVM.

The semantic corpus in `tests/semantic/` remains separate and unchanged. The
retained compiler, Phase 2, parser, and semantic execution manifests remain at
the paths recorded in `internal/SEMANTIC_TYPE_VALIDATION.md`.
