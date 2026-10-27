# Ownership and physical ARC contracts

The corpus contains 43 native programs and 33 rejection programs. It covers
moves, calls, returns, temporaries, aggregates, patterns, captures, references,
replacement, global cleanup, recursive generics and the real Vec/raw-resource APIs.
Legacy unsafe examples remain as rejection cases; valid counterparts use explicit
moves, borrowing, independent construction or the available concrete clone methods.

Every case runs with custom optimization off/on. Native programs additionally run
independent LLVM verification, LLVM O2, ordinary and guarded native execution.
The heap guard requires zero remaining allocations and unchanged values/status.
Rejection cases require a specific diagnostic and reject compiler/IR failures as
invalid test results. The guard and accepted-program cleanup expectations were
not relaxed for the ownership migration.

```sh
cargo test
cargo test ir_builder::ownership_tests
python3 tests/ownership/run.py --binary target/debug/fishyc --output /tmp/fishy-ownership-results
```

Requires Python 3, `cc`, GNU-compatible linker wrapping and LLVM 17 `opt`/`llc`.
`LLVM_BIN` changes the LLVM tool directory; `FISHYC_OWNERSHIP_TEST_OUTPUT` preserves
Cargo integration artifacts. `tests/memory_model` adds source safety, destruction
order and focused ASan/LSan checks. Sanitizers require actual successful execution.

Managed owners and Vec backing allocations are destroyed automatically. Vec's
compatibility `drop()` method clears/frees early and resets its resource state;
repeated calls and later automatic cleanup remain safe. Raw allocated C strings,
malloc addresses and explicit raw-parts exports still require manual cleanup.
Raw addresses do not manufacture trusted safe references.

Recursive generic cases preserve concrete Node/Option/array/scalar destruction.
Impossible direct recursive values formerly constructed from uninitialized null
sentinels now reject; concrete finite Option-terminated graphs still execute with
zero live allocations. See the authoritative implementation report in
`internal/MEMORY_MODEL_ARC_IMPLEMENTATION.md` for the source contract and limits.
