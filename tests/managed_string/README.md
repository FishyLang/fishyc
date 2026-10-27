# Managed String regression

The canonical production implementation is embedded from `src/stdlib.fih`.
All fixtures use prelude String and ByteBuffer without imports; no stdlib modules
are staged. Former production facade/prototype paths are retired.

```sh
python3 tests/managed_string/run.py --binary target/debug/fishyc
python3 tests/managed_string/profile.py --binary target/debug/fishyc --source tests/managed_string/prelude_profile.fih
python3 tests/managed_string/profile.py --binary target/debug/fishyc --source tests/managed_string/representation.fih
```

The 40 cases preserve byte content, independent snapshots, mutable aliases,
C interop ownership, arbitrary bytes/NUL, range/size errors, generic nesting,
function/control-flow/global lifetimes, private fields/methods and allocation
cleanup. Retired-module import rejections became direct private-prelude rejection
checks. The former mandatory-import rejection cases now assert availability.
The identity case passes String/ByteBuffer through ordinary user-module aliases.
Four examples are extracted from the actual String/ByteBuffer documentation.

Both custom modes run; marked cases additionally verify and execute LLVM-O2.
Successful guarded executions require zero final tracked allocations. The
prelude profile retains the exact canonical-empty allocation assertions.
The text profile retains zero-allocation comparisons/unchanged trim and exact
single-result allocation assertions. Profiling now measures the actual embedded
implementation, not instrumented copies of retired module sources; historical
method-entry instrumentation and alternate-block-size experiments are retired.
Allocation request/live byte counts and profile cleanup checks are retained.
