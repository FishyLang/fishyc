# Current Fishy standard library

The canonical, monolithic standard library is `src/stdlib.fih`. Its public APIs,
including String, ByteBuffer, List, Option and Result, are available without imports.
The former `stdlib/*.fih` and prototype module paths are retired: remove their
`using` declarations. The validated native target is 64-bit Linux.
See [the synchronous baseline](../internal/SYNCHRONOUS_CORE_BASELINE.md) for
the active architecture; there is no Fishy-owned native runtime.

## Resources at a glance

| Resource | Owner and cleanup |
|---|---|
| String literal, `bool.to_string()` result | Borrowed/static C-string bytes; **do not free** |
| `substring`, `concat`, `read_line` result | Caller-owned raw C string; call `free` once, including empty results |
| String, ByteBuffer | ARC-managed; no manual free/drop; explicit String C exports are caller-owned |
| Struct, enum, nonempty tuple, array/slice, closure, trait object | ARC-managed; managed children are released automatically |
| `Vec<T>` | Unique owner; automatic initialized-element and backing-allocation cleanup; optional early `drop()` |
| `malloc` allocation, `fopen` FILE handle, other native resource | Manual contract: use `free`, `fclose`, or the resource's native cleanup |

A raw pointer field does not become automatically freed because its enclosing
struct is managed. ARC manages acyclic graphs; it does not collect cycles or
extend nonmanaged stack-reference lifetimes. `()` is a storable unit value and
requires no ARC; `void` means no value.

Ordinary non-Copy bindings move on assignment and by-value calls. `&T` parameters
borrow shared; `&mut T` parameters borrow exclusively and require mutable caller
storage. Calls can auto-borrow places according to the signature. Mutation requires
`let mut`; safe borrows never retain their referent. Explicit `String.clone()`,
`List.clone()` and `ByteBuffer.copy()` have concrete type-specific contracts.

## Raw strings

`string` is a C-string pointer, not an ARC-owned text object. Length and text
operations count bytes before the first NUL; indexing/slicing do not interpret
Unicode characters. Keep borrowed storage alive while using its pointers.

Source string **`==` compares pointer identity**. Use **`a.equals(b)`** for content
comparison. Two separately allocated equal strings need not have equal pointers.

| API | Result/failure contract |
|---|---|
| `"literal"`, `bool.to_string()` | Borrowed/static pointer; never free |
| `s.len()`, `s.is_empty()` | Byte length/emptiness before the first NUL |
| `s.contains(sub)` | Byte search; an empty needle currently returns false |
| `s.starts_with(prefix)`, `s.ends_with(suffix)` | Byte prefix/suffix; an empty argument returns true |
| `s.substring(start,end)` | New owned copy of `[start,min(end,s.len()))`; start >= safe end produces a new owned empty string |
| `s.concat(other)`, source string `+` | Newly allocated C string; caller frees the result |
| `read_line(buf_size)` | New owned C string; requires positive capacity; details below |
| `s.split(nonempty_delimiter)` | New Vec with independently allocated owned token strings, including empty/trailing tokens |
| `s.split("")` | New Vec containing one **borrowed pointer equal to s**; it does not copy/split the bytes |

Allocation failure and several invalid operations call `panic`, which prints a
fatal diagnostic and exits. There is no exception unwinding cleanup guarantee.
Interpolation uses manually allocated C strings too: known internal formatting
buffers are cleaned up by the compiler, but the final assigned/returned string
remains manual. Do not assume every string-returning function owns its result.

```fishy
fn main()->i32 {
    let borrowed = "fish";
    let owned = borrowed.substring(0,2);
    printf("%s %s\n", borrowed, owned);
    free(owned); // Do not free borrowed.
    return 0;
}
```

For a **nonempty** delimiter, release each token before dropping the Vec:

```fishy
fn main()->i32 {
    let mut parts = "apple-banana".split("-");
    let mut i:u64 = 0;
    while i < parts.length() {
        let token = ^parts.get(i);
        printf("%s\n", token);
        free(token);
        i = i + 1;
    }
    parts.drop();
    return 0;
}
```

For an **empty** delimiter, keep the input alive and do not free the token
independently:

```fishy
fn main()->i32 {
    let input = "a".concat("b");
    let mut parts = input.split("");
    printf("%s\n", ^parts.get(0)); // The same pointer as input.
    parts.drop();
    free(input); // Exactly one cleanup for the owned input.
    return 0;
}
```

`read_line(n)` stores at most `n-1` bytes, strips LF, preserves CR, and **consumes
the rest of an overlong line**. It does not return the remaining bytes on the next
call. Both a blank line and EOF produce separately allocated empty strings, so
this API cannot distinguish those conditions. Free every result. A zero buffer
size panics; truncation/read errors are not reported as Result values.

## Vec

`Vec.new()` needs a concrete type, for example `let mut v:Vec<u8> = Vec.new();`.
Mutators (`push`, `set`, `insert`, `reserve`, `clear`, `drop`, `remove`,
`swap_remove`, `pop`) borrow exclusively. Queries borrow shared. `get(index)`
returns `&T`; `try_get`, `first` and `last` return `Option<&T>`.
`contains_by(value, equal)` uses `fn(&T,&T)->bool`, keeping content equality explicit.

- Assignment moves the Vec; it does not share mutable backing.
- `reserve(new_cap)` requests absolute capacity. Invalid indexes panic.
- `pop` returns `Result<T,string>`; remove/swap_remove return owned T. Extraction
  transfers initialized elements, including arbitrary non-Copy T.
- `clear()` destroys initialized elements in reverse order and keeps capacity.
- Normal destruction frees initialized elements and raw backing. Optional `drop()`
  performs early cleanup and resets the resource state; repeating it is safe.
- There is no general Vec.clone without an element Clone contract. Explicitly
  construct independent concrete elements when another mutable Vec is needed.
- A `Vec<string>` owns address values, not the pointed-to raw C buffers. Follow
  the string producer's manual cleanup contract.
- `into_array()` returns **`^T`, not a managed Fishy array**. It exports raw parts
  and clears the Vec's backing/element obligations. Save length/capacity first;
  the recipient must destroy initialized managed elements and free the allocation.
  This is an explicit manual-resource boundary.

## Option and Result

`Option<T>` has `Some(T)` and `None`; methods are `is_some`, `is_none`, `unwrap`,
`unwrap_or`, `map(fn(T)->U)` and `and_then(fn(T)->Option<U>)`.
`Result<T,E>` has `Ok(T)` and `Err(E)`; methods are `is_ok()`, `is_err()`,
`unwrap`, `unwrap_or(default_val)`, `map(fn(T)->U)`, `map_err(fn(E)->F)` and
`and_then(fn(T)->Result<U,E>)`.

Unwrap on None/Err panics. **Both unwrap_or methods evaluate fallback arguments
eagerly**, even when unused. Use `match` when fallback work should be conditional.
Managed payloads follow ARC; a raw string payload still follows its producer's
manual contract. Nullary variants use constructor call syntax, for example
`let absent:Option<u8> = Option.None();`.

Inspectors borrow the variant. `unwrap`, `unwrap_or`, maps and `and_then` consume
it; calling them moves a non-Copy variant. Payloads transfer once. An unused owned
fallback is destroyed normally. Borrow a variant for repeated inspection. Owned callback parameters are mutable
bindings internally so potentially stateful closures can be invoked. A returned
callback with an unknown environment contract similarly needs `let mut callback`.
Direct read-only lambda bindings do not need `mut`.

## Native handles and formatting

`malloc`/`realloc`/`free` and `fopen`/`fclose` are low-level native interfaces.
Check native failure results and pair each successful resource acquisition with
its cleanup on ordinary return paths. A FILE pointer is opaque: close it with
`fclose`, not `free`. There is no automatic File destructor API.

```fishy
fn main()->i32 {
    let file = fopen("message.txt", "w");
    if file == null { return 1; }
    let written = fputs("fish\n", file);
    let closed = fclose(file);
    if written < 0 { return 1; }
    if closed != 0 { return 1; }
    return 0;
}
```

`printf`/`snprintf`/`fprintf` use native format/type contracts; they do not own
argument strings. `fputs(text,file)` and `fprintf(file,format,...)` have their C
argument order. Use `print`, `println`, `eprint` or `eprintln` with managed String for ordinary
output. Each returns `Result<(),string>` and supports embedded NUL. String also
has `print()` and `println()` methods. Existing FILE handles still need close.

The migrated example in [`tests/stdlib/demo.fih`](../tests/stdlib/demo.fih) follows these contracts. Run
[`tests/stdlib`](../tests/stdlib/README.md) for compatibility/resource regressions.

## Managed text and bytes

[String](string.md) is immutable arbitrary-byte text with checked slices, search,
contains/find_from, split/lines, replace/repeat, ASCII trim and ASCII case conversion.
[ByteBuffer](byte-buffer.md) uniquely owns mutable byte storage; copy and snapshot are
independent. Both have a 1 GiB maximum. Neither needs manual cleanup. Lowercase
`string` retains its raw C-pointer contract and literal/interpolation behavior.

## Managed collections

`List<T>` is an immutable persistent sequence with `new`, `len`, `is_empty`,
`get`, `first`, `last`, `push`, `set` and concrete `clone`. Getters borrow and
return `Option<&T>`; set returns `Result<List<T>,string>`. Push/set consume the
inserted T and return a new List while explicitly sharing immutable nodes.
Assignment moves a List; `clone()` explicitly creates another logical list sharing
that backing. No arbitrary generic element copy is hidden in extraction.
Lookup and updates take O(log n), with O(log n) new update nodes and automatic
cleanup. The limit is 1,073,741,824 items; over-limit push and OOM panic. Vec supplies
unique contiguous mutable storage with automatic backing cleanup.

## Conversion and output

`String.parse_i64` and `parse_u64` return Result with a
`ParseNumberError { offset, message }`. Input must be nonempty and fully consumed.
No whitespace is skipped; use `.trim()` explicitly. Integer parsers accept an
optional `+`; only signed parsing accepts `-`. Integers are decimal and checked
before every accumulation step; overflow is Err. Leading zeroes are permitted.

`String.from_i64`, `from_u64`, `from_bool` return managed String.
Floating point arithmetic and compiler interpolation remain supported; the former
native-bridge managed floating point conversion methods have been removed.

```fishy
fn main()->i32 {
    let count=String.from_c_string("42").parse_i64().unwrap();
    println(String.from_c_string("count: ").concat(String.from_i64(count))).unwrap();
    return 0;
}
```

## Direct external ABI

The prelude's `extern fn` declarations call libc/libm directly for allocation,
raw strings, stdio, formatting and mathematical power. Managed counted output
calls the host's `write` ABI with stream identifiers 1 and 2; it does not use a
Fishy-owned bridge. Its signature currently targets a 64-bit POSIX host.
Do not mix buffered FILE output and counted output without explicitly managing
the external stream buffering contract.

Compile an object with `fishyc source.fih -o program.o`, then link with
`cc program.o -lm -o program`. The system linker driver does not compile any
Fishy-owned C source. Independently lowered LLVM objects use the same command.
There is no native archive to bundle and no additional Fishy runtime to link.

JSON, HTTP, async execution and native concurrency are intentionally absent.
The old architecture is historical; there are no compatibility aliases or stubs.
