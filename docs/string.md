# String

String is an immutable managed byte value in the canonical `src/stdlib.fih`
prelude, available without imports. Lowercase `string` is the raw C-pointer type.

Assignment moves a String and invalidates the source binding. `clone()` creates
another logical String with explicit immutable backing sharing. Queries borrow;
concat returns a new String. Owned values clean up deterministically through ARC. There is no String free/drop operation.
Use `equals` for content; `==` compares handle identity.

## Construction and lookup

| Static method | Contract |
|---|---|
| `new()->String` | Fresh empty String wrapper |
| `max_capacity()->u64` | 1,073,741,824 bytes (1 GiB) |
| `from_c_string(raw:string)->String` | Copy borrowed readable C bytes through the first NUL; never free the input |
| `try_from_c_string(raw:string)->Result<String,string>` | Same copy; Err if the input exceeds the supported size |
| `from_bytes(bytes:&ByteBuffer)->String` | Independent snapshot/copy of a [ByteBuffer](byte-buffer.md), including NUL bytes |

| Instance method | Contract |
|---|---|
| `len()->u64`, `is_empty()->bool` | Byte length and emptiness |
| `byte_at(index:u64)->Option<u8>` | Some byte, including zero; None outside the logical length |
| `equals(other:&String)->bool` | Full byte content equality, including bytes after embedded NUL |
| `concat(other:&String)->String` | New String with both byte sequences; panic if the result exceeds the limit |
| `try_concat(other:&String)->Result<String,string>` | Same result; recoverable size-limit failure |
| `to_c_string()->Result<string,string>` | Independent owned C allocation on Ok; Err for embedded NUL |

`from_bytes` never freezes, consumes or aliases the builder's mutable storage.
Later builder writes/appends cannot change the String. Ordinary assignments
transfer ownership. Use `clone()` when both variables must remain usable.

## C interoperability

```fishy
fn main()->i32 {
    let text=String.from_c_string("fish");
    let raw=text.to_c_string().unwrap();
    printf("%s %lu\n",raw,text.len());
    free(raw);
    return 0;
}
```

Raw input is borrowed only for the import call. Owned inputs remain the caller's
responsibility after copying. Null input panics; a nonnull readable,
NUL-terminated pointer is a native precondition.

C export rejects any embedded zero before allocating. On Ok, the caller owns a
new length+1 allocation with a terminator, including for an empty String. It
survives the String and must be freed once. Err messages are borrowed literals;
do not free them. There is no borrowed contiguous C view or implicit conversion.

Managed content may include arbitrary bytes and embedded NUL. Length and byte
lookup do not interpret Unicode characters or graphemes; no UTF-8 validation is
performed. Raw `string` remains the C-compatible pointer type: literal syntax,
raw equality, interpolation, bool conversion and raw method ownership are
unchanged. See [raw resource contracts](stdlib.md#raw-strings).

## Immutable aliases and concat

```fishy
fn main()->i32 {
    let base=String.from_c_string("fish");
    let shared=base.clone();
    let joined=shared.concat(String.from_c_string("y"));
    printf("%ld %lu %lu\n",base==shared,base.len(),joined.len());
    return 0;
}
```

Neither operand changes. A concat with an empty operand may share private
immutable storage while still returning a fresh public wrapper. For repeated
construction, accumulate in ByteBuffer and snapshot once; repeated immutable
concat can copy cumulative prefixes.

## Byte-oriented text operations

These operations compare **bytes**, including embedded NUL and non-UTF-8 data.
They do not decode Unicode or validate UTF-8. Positions count bytes from zero.
Every returned String is managed and requires no manual cleanup.

| Instance method | Contract |
|---|---|
| `try_slice(start:u64,end:u64)->Result<String,string>` | Independent immutable copy of the half-open range `[start,end)` |
| `starts_with(prefix:&String)->bool` | Content prefix comparison; empty prefix is true |
| `ends_with(suffix:&String)->bool` | Content suffix comparison; empty suffix is true |
| `find(needle:&String)->Option<u64>` | First byte position of a complete match; None if absent |
| `trim_ascii()->String` | Remove only leading/trailing ASCII whitespace |

Slicing accepts `0 <= start <= end <= len()`. Empty ranges, including
`[len(),len())`, succeed with an empty String. End equal to length is valid;
start greater than end or end greater than length returns Err with the borrowed
literal `"String slice range out of bounds"`. Bounds are checked before range
arithmetic, including for the largest u64 positions. There is no clamping or
panicking slice convenience method. A full slice also copies: the source may
leave scope before the result, without borrowed views or lifetime obligations.
Allocation failure still follows the existing panic policy.

Prefix/suffix checks compare content, independently of identity or reserved tree
height. A longer prefix/suffix returns false; both checks allocate nothing.
Search returns Some(0) for an empty needle, even in an empty String; a longer
needle returns None. Exact, prefix, suffix and overlapping matches follow the
same rule: return the smallest starting byte position. NUL does not terminate a
match. `contains(needle)` expresses containment. Search allocates its one managed Option result, without
per-candidate heap scratch or per-byte Option wrappers.

ASCII whitespace is exactly space (32), horizontal tab (9), LF (10), vertical
tab (11), form feed (12) and CR (13). Trim preserves interior whitespace and all
other bytes, including NUL, 0x85, 0xA0 and high bytes. An already trimmed value
(including empty) returns the **same immutable backing through an explicit String clone**, allocating
a fresh logical String wrapper. A partial trim returns an independent copied String; a nonempty
all-whitespace input returns a fresh empty wrapper sharing only the ordinary
canonical empty storage. No result borrows mutable storage.

`is_ascii_whitespace(byte:u8)->bool` is available in the prelude. It classifies that same six-byte set in constant time,
without allocation, locale behavior or Unicode interpretation. It is shared by
trim and consumers; no general character framework is implied.

With byte count `k`, touched leaves `L` and tree height `h`, slicing and
prefix/suffix comparison cost O(k + L*h). Slicing allocates its independent
storage and bounded traversal scratch; comparison uses O(h) stack space and no
heap scratch. Trim scans boundary whitespace in O(w + L*h), for `w` inspected
bytes, then copies a retained partial range if needed. Unchanged trim costs
O(h + 1) to inspect its edges. Heights include lazy reserved capacity, not just byte
length. Leaf width is fixed at 128 bytes.

Search is deliberately naive: for haystack length `n`, nonempty needle length
`m`, and maximum input height `h`, worst-case time is O(n * (m + h)); stack space
is O(h). Candidate starts are scanned in block order. Their first chunks are
compared in already loaded leaves; only candidates continuing across a leaf
boundary need a root range comparison. A one-byte needle takes O(n + L*h).
No KMP/Boyer-Moore or linear-time arbitrary-pattern guarantee is promised.

The deterministic [text-analysis consumer](../tests/text_foundation/consumer.fih)
counts bytes, lines, whitespace-delimited words and ASCII whitespace; trims a
note, checks its header/trailer, finds LF and copies its first line. Run it via
[the text-foundation driver](../tests/text_foundation/README.md); that driver
copies the program into an isolated source root using the canonical prelude.

## Size and errors

The maximum logical length is **1 GiB** on the validated 64-bit Linux target.
The bound is a deliberate library policy. `max_capacity()` reports it.
Use try construction/concat for sizes influenced by external data. Their Result
errors report size-limit failure, not allocation failure. Infallible variants
panic on size-limit violation. Null raw pointers, allocation failure and broken
internal invariants use the existing panic/exit behavior, without a scope
unwinding guarantee. Optional indexing and embedded-NUL export rejection remain
recoverable Option/Result operations.

Use factories and public methods to preserve invariants. Safe references enforce shared/exclusive access and inferred lexical lifetimes.
Raw native memory operations retain their manual obligations.

See [ByteBuffer](byte-buffer.md) for snapshot construction and fallible capacity/
concat examples, and [representation rationale](managed-string-prototype.md)
for allocation costs and implementation constraints.

## Practical text additions

| Method | Behavior |
|---|---|
| `contains(needle)` | Byte substring presence; empty needle matches |
| `find_from(needle,start)` | First position at or after start; start > len returns None; empty needle matches start |
| `trim()`, `trim_start()`, `trim_end()` | Six-byte ASCII whitespace policy; unchanged values clone immutable backing |
| `to_lower_ascii()`, `to_upper_ascii()` | Copy and map only ASCII letters; preserve every other byte |
| `split(delimiter)->List<String>` | Non-overlapping matches; preserves adjacent/trailing empty fields; empty delimiter gives one immutable source value |
| `lines()->List<String>` | LF or CRLF; final LF adds no phantom line; empty input gives an empty List; lone CR remains content |
| `replace(needle,replacement)->Result<String,string>` | Non-overlapping replacement; empty needle and size overflow are Err |
| `repeat(times)->Result<String,string>` | Checked repetition; zero times or empty input gives empty String |

Split results and lines clean up automatically. Byte positions can divide a UTF-8
sequence: these methods intentionally operate on bytes. Find-from uses naive
range comparisons, O(n*(m+h)) worst case. Split/replace combine that search with
independent range copies and persistent-list appends; no linear arbitrary-pattern
search guarantee is made. Case conversion and repetition copy their output.

`is_ascii_digit`, `is_ascii_alpha`, `is_ascii_alphanumeric`, `to_ascii_lower` and
`to_ascii_upper` classify/map individual bytes independently of locale.
Numeric parsing and managed formatting are documented in [stdlib](stdlib.md#conversion-and-output).

## Counted native interoperability

`String.from_raw_bytes(pointer,count)->Result<String,string>` copies arbitrary
bytes, including embedded NUL; it rejects a null pointer for nonzero count and
sizes above the library limit. A nonzero pointer must identify count readable
bytes. It never frees the input. `to_raw_bytes()->^u8` returns an independent raw
allocation containing len bytes plus a convenience zero terminator; **the caller
must free it once**. The count remains `String.len()`: embedded zeros do not
truncate it. Ordinary managed printing needs neither operation.
