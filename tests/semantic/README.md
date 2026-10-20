# Semantic/type regressions

`cases.json` contains manually authored deterministic Fishy programs. Categories
label coverage; they are not source-generation instructions. Acceptance cases
use explicit annotations to check the types of intermediate values and bindings.
Rejection cases require exit 1 and a stable diagnostic fragment, and exclude
panics/internal errors. Every case runs with `--check`; no object or LLVM output
may be produced.

Run with `cargo test --test semantic_validation`, or retain diagnostics with:

```sh
cargo build
python3 tests/semantic/run.py --binary target/debug/fishyc --output /tmp/fishy-semantic-results
```

Five Rust unit tests in `src/semantic_tests.rs` inspect materialized types,
specialization identities/signatures, remainder types, and full-arity default
plans directly. Run them with `cargo test type_checker::tests`.

`execution.json` contains nine small programs for the corrected interactions.
The retained integration runner checks both custom optimization modes, verifies
LLVM, and compares native and LLVM-O2 execution:

```sh
python3 internal/tests/completeness/run.py --binary target/debug/fishyc --manifest tests/semantic/execution.json --assert-results
```

No test constructs arbitrary programs or mutates source inputs. Unsupported
tuple match patterns, untyped lambda parameters, polymorphic callable values,
object remainder capture, and `break`/`continue` have rejection expectations;
tuple destructuring and typed direct generic calls have acceptance expectations.
