# Text foundation regression

```sh
python3 tests/text_foundation/run.py --binary target/debug/fishyc
python3 tests/managed_string/profile.py --binary target/debug/fishyc --source tests/text_foundation/profile.fih
```

Eight fixtures use the canonical monolithic prelude without stdlib imports.
They preserve the established slice, prefix/suffix, search, trim, consumer,
independent oracle and managed ownership assertions. Both custom modes verify
LLVM; successful cases additionally execute LLVM-O2 and the unchanged heap guard,
requiring zero final tracked allocations. The allocation profile protects exact
query-allocation and canonical-empty invariants in both custom modes. Historical
copied-module method-entry profiling is retired; production storage is unchanged.
