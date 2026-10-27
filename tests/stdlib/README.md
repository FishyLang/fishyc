# Stdlib compatibility tests (FAST)

These manually authored programs protect the **current** prelude/resource
contracts documented in [docs/stdlib.md](../../docs/stdlib.md). They introduce no
new stdlib APIs. Recursive-generic ownership regressions live in `tests/ownership/`, separate
from this FAST library compatibility suite.

```sh
cargo build
python3 tests/stdlib/run.py --binary target/debug/fishyc
cargo test --test stdlib_validation
```

`cargo test` also runs this integration target. Rust helper sources live in
`tests/support/` and are included by the compiler's unit-test modules; they are
not independent integration crates. Top-level `*_validation.rs` files are the
actual integration targets. No tests are removed or duplicated by that layout.

Run a subset, retain artifacts, or explicitly enable custom optimization:

```sh
python3 tests/stdlib/run.py --binary target/debug/fishyc --case read_line_compatibility
python3 tests/stdlib/run.py --binary target/debug/fishyc --custom-opt --output /tmp/fishyc-stdlib
```

`FISHYC_STDLIB_TEST_OUTPUT=/tmp/fishyc-stdlib cargo test --test stdlib_validation`
retains Cargo-run artifacts. Each invocation creates a unique run directory,
with source copies, per-stage stdout/stderr, objects and result.json. A summary
results.json is written at the output root. Default artifacts use a temporary
directory and are removed after the run. Do not put generated outputs in tests.

## Manifest

`cases.json` is a map from case name to the following fields. Each case runs
once, with the requested configuration; there is no automatic Cartesian matrix.

| Field | Meaning/default |
|---|---|
| `source` | Manually authored Fishy source text |
| `source_file` | Alternative source file relative to the manifest; the example case references the real ../../test.fih rather than a duplicate |
| `compile` | Expect compilation success, default true; false requires a clean exit 1, diagnostic match and no generated object, and never executes native code |
| `diagnostic` | Required stderr substring for expected compile failure |
| `stdout`, `stderr`, `exit` | Exact runtime expectations; stdout required for success, stderr defaults empty, exit defaults 0 |
| `stdin`, `argv` | Runtime input and argument list; default empty; stdin closes after supplied text so EOF is deterministic |
| `files` | Relative fixture path -> text contents; restored before **each** native/O2/guard execution |
| `expected_files` | Relative output path -> expected text after each execution |
| `custom_opt` | Enable compiler `-O` for this case, default false; CLI --custom-opt enables it for every selected case |
| `verify_llvm` | Explicit independent LLVM verification, default false |
| `llvm_o2` | Also optimize LLVM, verify the optimized module, lower and execute it; implies original independent verification; default false |
| `heap_live` | Enable existing allocation guard and require this live-allocation count, normally zero; also applied to requested O2 execution |

The compiler's mandatory internal LLVM verification always remains enabled.
Extra opt/llc invocations happen only for marked cases. The default manifest
marks one focused raw-text case for O2 and one managed Vec case for custom -O.
Other FAST cases do not run every optimization combination.

Requirements: Python 3, the built compiler and a C linker (`cc -lm`). Marked
verification/O2 cases additionally need LLVM 17 opt/llc; `LLVM_BIN` overrides
`/usr/lib/llvm-17/bin`. Selected accounting cases reuse the unchanged
`tests/ownership/heap_guard.c` via linker malloc/free/realloc wrapping. The guard
observes wrapped object allocations, not every private libc allocation; output,
status and live values remain part of every runtime assertion. It is not an
ownership proof. The runner reports missing tools/timeouts as failures.

## Initial coverage

- Option Some/None and Result Ok/Err; Result.unwrap_or eager fallback effects.
- Primitive Vec push/get/set/pop/clear/clone/drop, and managed-child sharing and
  survival after clearing/dropping another buffer.
- Borrowed literal/bool strings; owned substring/concat (including empty result),
  content equality versus distinct allocation pointer identity.
- Nonempty split's separately owned pieces, empty/trailing pieces, and empty
  split's borrowed source pointer/new Vec allocation.
- read_line ordinary/blank/bounded lines, draining truncation, following line and
  EOF, with explicit freeing of every result.
- Existing native FILE cleanup, argv, fixture input/output and nonzero status.
- A representative wrong-element-type compile rejection.
- The real repository example with apple-banana input and zero live allocations.

## Test tiers

FAST is this suite. NORMAL adds relevant core semantic/custom-IR smoke cases and
selected custom off/on or O2/accounting runs for changed allocation/generic/FFI
paths. FULL includes check/build/Rust tests, all four focused compiler corpora and
the established retained compiler/Phase 2/parser/semantic-execution runners.
Use FULL for representation/native/ownership changes; do not run the full matrix
for every small library helper. Keep new cases deterministic and minimal.
