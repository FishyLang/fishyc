# Fishy synchronous core baseline

This is the active architecture reference after the deliberate native runtime
purge. The starting working tree contained S0–S12 complete and revalidated,
including the runtime-boundary correction; S13–S18 were unimplemented. That
hybrid roadmap is now retired, including its implementation stages. This reset
is deletion and simplification, with no replacement async, concurrency or native
runtime in Rust.

The before-state inventory and source/hash snapshot were saved before deletion
under `/tmp/fishy-sync-reset/{before,before-sha256.json,dependency-audit.md}`.
The snapshot, rather than Git HEAD, defines this report's delta: the working tree
already contained extensive user changes and untracked implementation files.
`test.fih`, pre-existing unrelated changes and prior Git deletions were preserved.
No reset, clean, staging, commit or history rewrite was performed.

[The machine-readable audit](SYNCHRONOUS_CORE_RESET_AUDIT.json) contains the exact
file changes, per-file diff counts, all 252 former native function definitions
with source/group/disposition, the 54 former compiler/runtime imports, final
external declarations, resource tests and artifact evidence. The old
[RUNTIME_BOUNDARY_INVENTORY.json](RUNTIME_BOUNDARY_INVENTORY.json) is explicitly
historical. Its classifications describe the removed source, not a migration plan.

## Active architecture

```text
Fishy source + monolithic src/stdlib.fih
             |
      lexer / parser / AST
             |
  name / type / ownership analysis
             |
          custom IR
             |
          optimizer
             |
            LLVM
             |
       native object / executable
             |
 narrow direct libc/libm/system ABI where needed
```

There is no Fishy-owned C runtime or platform implementation. The runtime and
platform production directories and their ABI headers have been deleted. Generated
allocation, ARC, typed destructors and ordered global cleanup remain compiler
mechanisms; their code is emitted into the program's LLVM module. They are not
an FFI runtime or a new Rust runtime. Normal Rust compiler implementation remains.

The public Fishy stdlib remains the single `src/stdlib.fih`. Ordinary algorithms
stay in Fishy: immutable String/List trees, mutable ByteBuffer, Vec operations,
Option/Result, text search/slicing/case conversion, integer conversion and output
retry/newline policy. Compiler primitives cover representation, generics, ordinary
captures, typed ownership and direct code generation. The remaining external ABI
is explicit, narrow and host-dependent; it is not a hidden portability layer.

## Dependency inventory and deletion decisions

Every owned C definition was classified before deletion. The complete per-symbol
inventory is in the audit JSON; this table records the dependency trees and their
Fishy/compiler/IR/backend consequences. All retired APIs disappear outright.

| Subsystem | Former production implementation / ABI | Dependencies and former consumers | Decision |
|---|---|---|---|
| Async syntax and frames | `async_analysis.rs`, `async_ir.rs`, `async_lowering.rs`; task/frame routines in `stdlib_native.c` | parser/AST/checker, state machine IR, LLVM resume/drop adapters; async and concurrency suites/docs | Delete syntax, types, analysis, lowering, descriptors, imports and tests |
| Task/completion/Waker | `completion_runtime.h/.inc`, task control in native translation unit; `fishy_v2_completion_*`, task/waker routes | Awaitable, CompletionSource, outcome/subscriptions, typed result storage, weak endpoint identities | Delete entire completion model and generated runtime boundary |
| Executors/workers | `runtime/executor.h/.inc`, `runtime/workers.inc`, `platform/native_jobs.inc`; executor/task admission symbols | LocalExecutor, ThreadPoolExecutor, submit/compute, registry/TLS/M, FIFO scheduling, S9/S10 tests | Delete services, pools, dispatch, home/routing and lifecycle |
| Reactor and timers | `runtime/reactor.inc`, `platform/linux_runtime.inc`, `platform/linux_http_reactor.inc` | poll/pipe/native notification, monotonic deadlines, async sleep, native envelopes | Delete service and backend mechanisms; no synchronous timer replacement |
| Native Thread | thread/descriptors/startup/join code in `stdlib_native.c`, concurrency ABI | mandatory-join Thread API, attachment/identity/publication, pthread lifecycle | Delete dedicated thread implementation and all compiler contracts |
| Synchronization | `stdlib_native.c`, `concurrency_abi.h`, `concurrency_builtins.rs` | Atomic/MemoryOrder, Mutex/RwLock/CV/guards, scoped effects, LLVM atomics and C locks | Delete APIs, special types, effect tracking, intrinsics and ABI |
| Channel | `runtime/channel.h/.inc`, concurrency builtin registry | Sender/Receiver, ring/FIFO/handoff/close policy, async/try/blocking, completions and typed ownership adapters | Delete complete channel tree; no synchronous Channel |
| Mobility/publication | `concurrency_analysis.rs`, concurrency registry and checker branches | Mobile, ConcurrentFn, ScopedFn, cross-thread/frame provenance | Delete concurrency model; retain ordinary lexical capture identities |
| HTTP/libcurl | `stdlib_native.c`, HTTP reactor adapter; `fishy_native_http_*` | curl easy/multi callbacks, dynamic loading, TLS, HTTP policy in Fishy; HTTP/JSON fixtures | Delete sync/async HTTP, all curl interop, servers, examples and current docs |
| JSON | Json/reader/writer/helpers in `stdlib.fih` | parsing/serialization/errors, HTTP helpers, expansion tests | Delete even though much of it was Fishy, as explicitly requested |
| Managed float conversion | locale/strtod/format bridge in `stdlib_native.c` | String.parse_f64/from_f64 and their tests | Delete bridge and methods; retain float arithmetic and compiler interpolation |
| Output bridge | `platform/stdio_output.inc`, `fishy_native_output_*` | managed print/eprint Fishy retry helpers | Delete bridge; keep Fishy algorithm over direct system `write` |
| Runtime lifecycle / build | phase/M/TLS/service state in `stdlib_native.c`, all ABI headers, `native_boundary.rs`, `build.rs` | boot/attach/quiesce/finalize, bundled native object, -pthread / loader / curl | Delete; retain only synchronous generated global initialization/cleanup |
| Managed allocation / ARC | LLVM backend and typed IR ownership edges | all surviving managed values, String/collections/closures | Retain compiler mechanism; replace atomic count protocol with checked ordinary loads/stores |
| Raw external ABI | prelude libc/libm externs, normal compiler FFI lowering | allocation, raw strings, formatting, panic, raw stdio | Retain direct system ABI; no owned implementation or shim |
| Allocation guards | four C files under tests/internal tooling, listed below | bounds/canary/quarantine/live accounting and String profiles | Retain test-only instrumentation, never bundled into production |

The former runtime inventory's A/B/C/D responsibilities all have explicit
outcomes: concurrency-specific compiler primitives (A), runtime/library policy
(B), OS mechanisms (C) and HTTP/library interop (D) are deleted together with their
consumers. The surviving ordinary compiler primitives and direct external ABI
are independent. No algorithm was translated from C to Rust to keep a retired API.

## Compiler, custom IR and backend deletion

`async` and `await` are ordinary identifiers. Their tokens, parser branches, AST
nodes, function flags, diagnostics, result type transformations, legality passes,
frame persistence and conversion protocol are gone. `async fn` and `await value`
have no special grammar. A small parser regression proves both identifier use and
absence of the old grammar.

Task, Awaitable, CompletionSource, Waker, executor, channel and synchronization
builtins have no intrinsic type or method resolution. There are no reserved
compatibility types: user-defined structs may freely use names such as Task or
Mutex. Unknown old stdlib names are unresolved normally. There are no unsupported
stubs, hidden feature flags, aliases or commented implementations.

IR operations removed: `Atomic`, `AwaitResult`, `AwaitSuspend`, `Channel`,
`Completion`, `Condition`, `Executor`, `Mutex`, `QualifyCallable`, `Task`, `Thread`.
IR types removed: `Task`, `Callable` qualifications, `Sync`, `TryGuard`.
Optimizer/remapping/ownership/verifier/backend cases, provisional async
verification and ModuleIr async metadata were removed with them. The normal
SSA, dominance, type and ownership checks remain. Direct Task/Awaitable awaits
and generic `into_awaitable` lowering both disappear.

Ordinary closure capture resolution is preserved in `src/captures.rs`: exact
lexical binding identity, transitive free captures and specialized types. It has
no mobility, provenance or executor state. Ordinary references, caller-place
checks, generics/constraints, structs/enums/unions/tuples, arrays/slices, traits,
patterns, destructuring, lambdas, defaults, operators and casts remain subject to
their existing synchronous language rules.

## Ownership and lifecycle

Universal atomic ARC was only necessary for the deleted cross-thread model.
The baseline uses checked **single-threaded** ARC emitted directly by LLVM:

- Null retain/release is a no-op. Valid pointers carry the existing three-word
  header `[typed destructor, count, length]`; payload-relative offsets stay intact.
- Retain rejects zero (resurrection) and counts at or above the signed count
  limit before incrementing. Release rejects zero before decrementing. Invalid
  counts terminate through `llvm.trap`; they cannot wrap into live ownership.
- Final release stores zero, invokes the generated exact typed destructor, then
  frees the allocation through external `free`. Recursive managed children use
  their generated typed ownership edges. No arbitrary generic layout is guessed.
- Incoming owners are retained/pinned before outgoing owners are released on
  replacement; normal early-return, branch, loop, capture and global rules remain.
- No atomic operations, CAS loops or fences remain in generated ARC. One Rust
  `AtomicUsize` remains in `token.rs` solely for compiler token identities across
  Rust test/compiler threads; it is not a Fishy type or generated runtime feature.

Acyclic ARC still does not collect cycles. Raw pointers/resources retain their
manual contracts. Vec's raw backing allocation still requires `drop()`; immutable
List and managed String/ByteBuffer are automatic. Raw `string` and explicit
String C exports remain manual where documented. No exception/unwind cleanup
promise was introduced; fatal panic/trap paths terminate the process.

Only generated ordered global initialization and reverse clear-before-release
cleanup remain. `__fishy_program_exit` calls synchronous global cleanup. Cold /
Initializing / Running / Stopping / Quiescent / Destroyed, attachments, TLS,
service registry, native joins and shutdown coordination are gone. There is no
metadata mutex or runtime lock hierarchy in the synchronous baseline.

## Stdlib and external ABI

Surviving facilities include Option/Result callbacks, Vec, persistent List,
String/ByteBuffer and ASCII/text operations, raw strings, numeric bounds and
integer/bool conversion, managed output, basic raw stdio, formatting/interpolation
and existing mathematical operations. `String.from_raw_bytes` was preserved from
its former HTTP-adjacent location because it is an independent counted byte-copy
facility. No filesystem/process/network expansion was added.

The 20 prelude external declarations are:

```text
pow
malloc free realloc memcpy memset
printf snprintf fopen fclose fputs fprintf exit getchar
strlen strcat strstr strcmp strncpy
write
```

These are direct external ABI declarations, with no Fishy-owned implementation.
`malloc/free` support both generated managed objects and explicit raw storage;
`llvm.trap` is emitted directly. This is the narrow fundamental mechanism
exception, not a native compatibility layer. Managed output retains its Fishy
complete-write/newline/error logic over `write(i32,^u8,u64)->i64`; stream IDs 1/2
and this ABI are currently POSIX/64-bit assumptions. Short writes retry; negative,
zero-progress or impossible counts fail with Result.Err. Mixing counted output
with buffered FILE output requires respecting external buffering contracts.
Managed float bridge methods were removed rather than replaced. No locale bridge
remains. Compiler float interpolation still uses direct `snprintf`.

The host LLVM target/data layout is used where already implemented; the stdlib
and managed header still contain 64-bit/LP64 assumptions. Windows support and a
new portability layer were not implemented or claimed. Existing external FFI is
an explicit low-level contract, not permission to publish Fishy managed values to
foreign threads.

## Build and binary audit

`build.rs` was deleted. There is no C compile command, native object include,
OUT_DIR runtime blob, object-bundling linker pass, native archive or C sanitizer
build path. The backend verifies/emits a normal LLVM object. Linking remains:

```sh
cargo build
./target/debug/fishyc source.fih -o program.o
cc program.o -lm -o program
./program
```

`cc` here is the system linker driver; it is not compiling owned C. Cargo.toml and
Cargo.lock were already free of direct owned-runtime `cc` dependencies, so they
remain unchanged. A transitive `cc` build dependency remains in **external
llvm-sys**, which builds its vendor `targetwrappers` object/archive for LLVM's
compiler API. Removing it would damage the independent LLVM compiler backend.
The fresh build's Fishy rustc invocation has no owned runtime link input.

A fresh offline target-directory build passed. Its only C wrapper object/archive
belongs to llvm-sys outside this repository, not Fishy. `nm`/`ldd` checks on the
final compiler, generated smoke object and generated executable find no retired
`fishy_v2_*`, async/task/thread/native runtime symbols or libcurl dependency.
The smoke executable prints 42, uses libc and needs no Fishy object/archive.
The compiler's normal LLVM/Rust dependencies (including libffi, compression,
terminal and C++ support libraries) remain; their own system ABI use is unrelated
to the deleted Fishy runtime. There is no Fishy curl loader left to bypass a
simple dynamic-link audit.

Evidence: `/tmp/fishy-sync-reset/fresh-build.log`, `fresh-native-artifacts.txt`,
`final-cc-dependency.txt`, and `final-binary-audit/` (compile/link/run/nm/ldd logs).
The three obsolete root LLVM dumps `condition.ll`, `rwlock.ll`, `source.ll` and
one cached `target/.../stdlib_native.o` were archived byte-for-byte outside the
repository under `retired-generated-artifacts/`, then removed. Unrelated older
synchronous test dumps and ignored user build outputs were preserved; they are
not source inputs or fresh build dependencies.

## Final repository C/header/include audit

A whole-repository find, including ignored build directories, returns exactly:

| Remaining file | Purpose / classification |
|---|---|
| `tests/ownership/heap_guard.c` | Test-only linker allocation wrapper: quarantine, canaries, invalid/double free and zero-live accounting; obsolete pthread locking removed for synchronous execution |
| `tests/managed_string/profile_guard.c` | Test-only allocation/work counters, includes the ownership guard; never a production dependency |
| `internal/tests/completeness/heap_guard.c` | Test-only completeness/Phase 2/parser/execution accounting |
| `internal/tools/heap_guard.c` | Existing test-only compiler tooling guard |

No `.h` or `.inc` remains. There is **zero Fishy-owned production C LOC** and no
production C compilation. Scalar FFI tests may generate tiny C `helper.c` files in
temporary directories to validate an external ABI; they do not implement any
Fishy runtime and are not bundled into programs normally. C test guards are not
an alternative runtime.

Repository term searches now find retired names in historical records, audit
metadata, the small retirement tests and Python test orchestration only. Python
`json` parses test manifests; Python `ThreadPoolExecutor` runs independent compiler
fixtures in parallel. Neither provides a Fishy API. Public docs describe only the
synchronous baseline. All C/platform/runtime source directories are gone, not
empty shells or disabled implementations.

## Validation and test retirement

The async/concurrency/runtime-boundary integration targets and their fixtures,
C harnesses, sanitizer drivers, HTTP servers/TLS fixtures and examples were
deleted. JSON/HTTP expansion cases and current docs were removed. Completeness's
single async decision case was removed, reducing 368 to 366 configurations.
Historical totals are not targets to preserve.

One ownership fixture originally used Json solely to test closure-parameter
shadowing. It now uses managed String with the same independent ownership intent,
expected output and zero-live contract; that unrelated regression was retained.
The float bridge assertions alone were removed from numeric tests. General
ownership, semantic, custom IR, LLVM ABI, String and text manifests were retained.
A small retirement parser/type test and one focused synchronous ARC gate were
added; no mass conversion of retired tests into rejection fixtures was performed.

| Retained gate | Final result |
|---|---:|
| cargo check / cargo build | PASS, warning-free |
| Fresh offline cargo build | PASS |
| Full cargo test: compiler units | 99/99 |
| Full cargo test: integration functions | 11/11 across 9 targets |
| Semantic types | 165/165, including 101 intentional rejection cases |
| Custom IR | 50/50 |
| LLVM / ABI | 48/48 |
| Ownership / recursive generic managed graphs | 102/102 |
| Managed String | 80/80 plus canonical prelude allocation profiles |
| Text foundation | 16/16 plus bounded allocation/work profiles |
| Stdlib FAST | 11/11 |
| Synchronous expansion (core/numbers/output) | 6/6 |
| New checked synchronous ARC | 2/2 custom modes |
| Independent completeness | 366/366 |
| Independent Phase 2 | 234/234 |
| Independent parser regressions | 68/68 |
| Independent semantic execution | 18/18 |
| Generated LLVM ASan + LeakSanitizer | PASS in both custom modes |

The new focused gate exercises nested recursive generic String graphs, arrays,
tuples, Option/Result, persistent List, closure captures, manual C exports and
identifier reuse. LLVM-only test fault injection independently verifies retain
overflow, release underflow and resurrection trap, plus null no-op behavior.
Normal and LLVM-O2 executions require zero live tracked allocations in both
custom optimizer modes. ARC IR is checked for ordinary operations and the absence
of atomic/CAS/fence count machinery. External import checks reject retired native
symbols. A structural source/build assertion rejects production C/header/inc
files and reintroduced owned build.rs compilation.

Existing ownership suites retain replacement, aliasing, loops/branches/returns,
recursive specializations, enum/union payloads, trait objects, captures and global
cleanup coverage. Applicable runners preserve custom IR verification, independent
LLVM verification, LLVM O2/native execution and canary/live allocation guards.
No optimizer, verifier, ownership test or resource guard was disabled.

ASan instruments generated Fishy LLVM, not a replacement native runtime. The
first sandboxed LeakSanitizer execution failed under ptrace; the authorized run
outside the sandbox passed with leak detection enabled, no suppressions. There
is no production native C left for native UBSan coverage; source-language UBSan
instrumentation is not implemented by this LLVM-only compiler. The removed C
sanitizer harnesses were not retained as empty machinery. TSan is not a meaningful
validation gate for this single-threaded Fishy baseline; no clean TSan result is
claimed and no ARC ordering was adjusted for a sanitizer.

Full logs: `/tmp/fishy-sync-reset/final-cargo.log`,
`final-{completeness,phase2,parser,execution}.log`; focused ASan:
`focused-core-authorized.log` and `focused-core/`; final check/build/units after
comment/format/dead-no-op cleanup: `final-*-confirmed.log`. Functional suites were
not repeatedly rerun after documentation/format-only changes. The final binary
was separately compiled, linked and executed for the symbol/dependency audit.

## Quantified deletion

Physical lines include comments and blanks. Production means `src/**` plus the
root build script. Header LOC is separated from C implementation LOC; runtime /
platform directory LOC is a subset, not an additional total. Diff counts compare
the preserved dirty starting tree, never Git HEAD.

| Measure | Before | After |
|---|---:|---:|
| Total production LOC | 27,713 | 19,828 |
| Rust compiler LOC | 22,836 | 18,372 |
| Rust build-script LOC | 18 | 0 |
| Fishy stdlib LOC | 2,245 | 1,456 |
| Owned C implementation LOC (.c + implementation .inc) | 2,361 | 0 |
| Owned ABI header LOC | 253 | 0 |
| Runtime/platform directory LOC | 1,626 | 0 |
| Production .c files | 1 | 0 |
| Production C implementation units (.c + .inc) | 10 | 0 |
| Prelude direct extern declarations | 39 | 20 |
| Custom IR operation variants | 49 | 38 |
| Compiler unit tests | 117 | 99 |
| Integration test functions | 29 | 11 |
| Integration targets | 11 | 9 |
| Owned native function definitions | 252 | 0 |
| Compiler/runtime native imports | 54 | 0 |
| Stdlib named manifest cases (FAST + String + text + expansion) | 70 | 62 |
| Stdlib executed configurations (including expansion's former HTTP extras) | 133 | 113 |
| Logical runtime external dependency families | libc/libm/pthread/dlfcn/libcurl (5) | libc/libm (2) |

The 54 compiler/runtime import entries were a typed registry, distinct from the
39 former textual prelude extern declarations. The current 20 direct externs
contain no owned runtime imports. Logical family counts describe runtime
requirements, not ELF DSO counts on glibc where several libraries are merged.

Production deletion is **7,885 lines net**. Across implementation,
tests, current docs and historical retirement notices: **627 added,
13,671 deleted, 399 paired modified lines**, net **-13,044**.
Replacements contribute to both added/deleted; “modified” is the matched count
within replacement regions. The new baseline and machine-readable audit are
excluded from these code/deletion diff numbers to avoid self-referential counts;
their documentation LOC can be read directly. Generated LLVM/object artifacts
are separately recorded, not counted as source deletion.

There are **6 added, 101 deleted and 42 modified source/test/doc files**
against the before snapshot. Additionally three obsolete generated LLVM dumps
and one cached native object were archived and removed. No Rust runtime was added;
the only new production Rust file is the small ordinary capture-plan data model.

## Exact changed-file manifest

### Added

```text
src/captures.rs
tests/synchronous_core/ownership.fih
tests/synchronous_core/run.py
tests/synchronous_core_validation.rs
internal/SYNCHRONOUS_CORE_BASELINE.md
internal/SYNCHRONOUS_CORE_RESET_AUDIT.json
```

### Deleted

```text
build.rs
docs/async.md
docs/concurrency.md
docs/http.md
docs/json.md
docs/runtime-boundary.md
docs/runtime-portability.md
examples/async_timers.fih
examples/http_concurrent.fih
examples/http_get_json.fih
examples/http_get_text.fih
examples/http_post_json.fih
examples/json_build.fih
examples/json_read.fih
src/async_analysis.rs
src/async_ir.rs
src/async_lowering.rs
src/completion_runtime.h
src/completion_runtime.inc
src/concurrency_abi.h
src/concurrency_analysis.rs
src/concurrency_builtins.rs
src/native_boundary.rs
src/platform/linux_http_reactor.inc
src/platform/linux_runtime.inc
src/platform/native_jobs.inc
src/platform/stdio_output.inc
src/runtime/channel.h
src/runtime/channel.inc
src/runtime/executor.h
src/runtime/executor.inc
src/runtime/reactor.inc
src/runtime/workers.inc
src/runtime_platform.h
src/stdlib_native.c
tests/async/README.md
tests/async/cases.json
tests/async/http.fih
tests/async/http_concurrent.fih
tests/async/native_pool.c
tests/async/run.py
tests/async_validation.rs
tests/concurrency/README.md
tests/concurrency/arc.c
tests/concurrency/arc.fih
tests/concurrency/arc.py
tests/concurrency/atomic.c
tests/concurrency/atomic.fih
tests/concurrency/atomic.py
tests/concurrency/await_adapter.c
tests/concurrency/await_adapter.fih
tests/concurrency/await_adapter.py
tests/concurrency/captures.fih
tests/concurrency/captures.py
tests/concurrency/channel.c
tests/concurrency/channel.fih
tests/concurrency/channel.py
tests/concurrency/channel_allocation.c
tests/concurrency/completion.c
tests/concurrency/completion.fih
tests/concurrency/completion.py
tests/concurrency/condition.c
tests/concurrency/condition.fih
tests/concurrency/condition.py
tests/concurrency/executor.c
tests/concurrency/executor.fih
tests/concurrency/executor.py
tests/concurrency/lifecycle.c
tests/concurrency/lifecycle.py
tests/concurrency/mutex.c
tests/concurrency/mutex.fih
tests/concurrency/mutex.py
tests/concurrency/pool.c
tests/concurrency/pool.fih
tests/concurrency/pool.py
tests/concurrency/pool_http.py
tests/concurrency/publication.c
tests/concurrency/publication.fih
tests/concurrency/publication.py
tests/concurrency/rwlock.c
tests/concurrency/rwlock.fih
tests/concurrency/rwlock.py
tests/concurrency/sanitizers.py
tests/concurrency/sync6.py
tests/concurrency/task_control.c
tests/concurrency/task_control.fih
tests/concurrency/task_control.py
tests/concurrency/thread.c
tests/concurrency/thread.fih
tests/concurrency/thread.py
tests/concurrency/tsan_fence_probe.c
tests/concurrency_validation.rs
tests/runtime_boundary/inventory.py
tests/runtime_boundary/policy.c
tests/runtime_boundary/policy.fih
tests/runtime_boundary/run.py
tests/runtime_boundary_validation.rs
tests/stdlib_expansion/http.fih
tests/stdlib_expansion/json.fih
tests/stdlib_expansion/malformed.fih
tests/stdlib_expansion/utf8.fih
```

### Modified

```text
.gitignore
docs/stdlib.md
docs/string.md
internal/ASYNC_AWAIT_ARCHITECTURE.md
internal/ASYNC_AWAIT_IMPLEMENTATION.md
internal/HYBRID_ASYNC_CONCURRENCY_ARCHITECTURE.md
internal/HYBRID_ASYNC_CONCURRENCY_IMPLEMENTATION.md
internal/RUNTIME_BOUNDARY_INVENTORY.json
internal/STDLIB_EXPANSION.md
internal/tests/completeness/cases.json
internal/tests/completeness/run.py
src/ast.rs
src/ast_visit.rs
src/backend.rs
src/ir.rs
src/ir_builder.rs
src/ir_optimizer.rs
src/ir_verify.rs
src/main.rs
src/module_loader.rs
src/module_names.rs
src/parser.rs
src/scanner.rs
src/stdlib.fih
src/token.rs
src/type_checker.rs
tests/llvm_abi/run.py
tests/managed_string/profile.py
tests/ownership/cases.json
tests/ownership/heap_guard.c
tests/ownership/run.py
tests/stdlib/run.py
tests/stdlib_expansion/README.md
tests/stdlib_expansion/cases.json
tests/stdlib_expansion/numbers.fih
tests/stdlib_expansion/run.py
tests/stdlib_expansion_validation.rs
tests/support/custom_ir_tests.rs
tests/support/llvm_abi_tests.rs
tests/support/ownership_tests.rs
tests/support/parser_tests.rs
tests/support/semantic_tests.rs
```

## Remaining limitations and next work

The synchronous baseline intentionally has no concurrency, async, networking or
JSON. Its host ABI is not yet a portable platform abstraction. Fatal errors do
not unwind. ARC does not collect cycles and raw-resource/Vec cleanup remains
explicit. The old roadmap imposes no compatibility or future feature obligation.

Based on the surviving source, useful next directions (recommendations only) are:

1. **Contextual generic aggregate inference.** The focused recursive graph needs
   an intermediate typed `Option<Node<String>>` binding before constructing a Node;
   nested `Option.None()` in the aggregate is not inferred. Improve that ordinary
   compiler inference path and its diagnostics without reviving runtime features.
2. **Total lowering diagnostics.** `ir_builder.rs` still has `unimplemented!` fallbacks
   for unsupported unary/call/expression/statement paths. Check unsupported forms
   before lowering and report normal source diagnostics instead of compiler panics.
3. **Raw resource and ownership contracts.** Make the existing Vec backing-buffer,
   raw string export and external handle cleanup rules easier to verify and use;
   preserve the strong managed graph/recursive generic regression corpus.
4. **Target/ABI consistency.** Audit the remaining 64-bit header, integer/pointer
   casts, raw libc sizes and counted output ABI against target data layout before
   adding another target. Do not grow a new owned native portability runtime.
5. **Retained test orchestration.** Make the independent completeness/Phase 2/parser
   pipelines easier to discover alongside Cargo without losing either optimization
   mode, LLVM-O2 execution or allocation checks.

None of these directions was implemented during the reset. S13–S18 remain
unimplemented and retired; there is no pending cancellation/scope/async-lock lane
or replacement executor architecture to continue.

## Final status

Fishy synchronous core baseline established.

The former hybrid async/concurrency architecture is retired.

Removed:
- async/await
- Task/Awaitable/completion/Waker infrastructure
- executors/schedulers/workers/reactor
- native threading and synchronization
- Channel
- concurrency-only mobility/publication machinery
- HTTP/libcurl
- JSON
- Fishy-owned C runtime/platform implementation

Fishy-owned production C LOC: **0**.

The surviving compiler, ownership model, custom IR, LLVM backend and synchronous
standard-library functionality are revalidated.

No replacement async/concurrency/native runtime was implemented.

Future development starts from `internal/SYNCHRONOUS_CORE_BASELINE.md`.
