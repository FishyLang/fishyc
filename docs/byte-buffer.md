# ByteBuffer

ByteBuffer is available without imports from the canonical `src/stdlib.fih`
prelude. It is an owned mutable value for arbitrary bytes,
including zero and non-UTF-8 bytes. Its [String](string.md) snapshots are independent.

`let b=a` moves ownership and invalidates `a`. Mutation requires `let mut` and
an exclusive receiver borrow. `copy()` creates independent mutable storage;
`snapshot()` creates an independent immutable String. No buffer free/drop is needed.

## API

| Static method | Contract |
|---|---|
| `new()->ByteBuffer` | Empty buffer with length/capacity zero |
| `max_capacity()->u64` | 1,073,741,824 bytes (1 GiB) |

| Instance method | Contract |
|---|---|
| `len()->u64`, `capacity()->u64` | Logical byte count and addressable capacity |
| `byte_at(index:u64)->u8` | Read an existing byte; invalid index panics |
| `set(index:u64,value:u8)->void` | Write an existing byte; invalid index panics |
| `reserve(required:u64)->void` | Absolute capacity request; over-limit request panics |
| `try_reserve(required:u64)->Result<(),string>` | Same growth; over-limit request is Err |
| `push(value:u8)->void` | Append one byte; over-limit growth panics |
| `try_push(value:u8)->Result<(),string>` | Same append; over-limit growth is Err |
| `append_bytes(other:&ByteBuffer)->void` | Append the source's original logical prefix; over-limit result panics |
| `try_append_bytes(other:&ByteBuffer)->Result<(),string>` | Same append; over-limit result is Err |
| `append_c_string(raw:string)->void` | Borrow/copy through the first C NUL; over-limit result panics |
| `try_append_c_string(raw:string)->Result<(),string>` | Same append; over-limit result is Err |
| `copy()->ByteBuffer` | Independent mutable copy, preserving capacity |
| `snapshot()->String` | Independent immutable snapshot; same copy policy as String.from_bytes |

Reserve is absolute, not additional space. Capacity grows by doubling from
128 bytes and never exceeds 1 GiB. Reservation is lazy addressable tree space:
it does not allocate every block or promise that later allocations will succeed.
Growth preserves existing bytes without copying their prefix. Empty reserve
does not change length. Copy preserves the capacity metadata.

Appending requires independent source and destination borrows. For self-append,
first make an explicit `copy()` and append that copy. Size checks occur before growth/addition;
recoverable size failure leaves the destination unchanged. There is no implicit
freeze, detach, consume, refcount inspection or copy-on-write.

## Build, alias, copy and snapshot

```fishy
fn main()->i32 {
    let mut a=ByteBuffer.new();a.append_c_string("ab");
    let mut b=a.copy();
    let text=a.snapshot();
    let mut independent=b.copy();
    b.set(0,120);independent.push(99);
    let raw=text.to_c_string().unwrap();
    printf("%s %lu %lu\n",raw,a.byte_at(0),independent.len());
    free(raw);
    return 0;
}
```

The snapshot remains `ab`; `a` and its copies have independent mutable storage. Even empty snapshots keep no mutable builder
root. C inputs are borrowed and never freed by append; owned raw exports from
String still require caller free.

## Fallible size handling

```fishy
fn main()->i32 {
    let mut bytes=ByteBuffer.new();
    match bytes.try_reserve(ByteBuffer.max_capacity()+1) {
        Ok(done)->{printf("reserved\n"); ();}
        Err(reason)->{printf("capacity: %s\n",reason); ();}
    };
    let left=String.try_from_c_string("fish").unwrap();
    match left.try_concat(String.from_c_string("y")) {
        Ok(text)->{
            let raw=text.to_c_string().unwrap();
            printf("%s\n",raw);free(raw); ();
        }
        Err(reason)->{printf("concat: %s\n",reason); ();}
    };
    return 0;
}
```

The try methods recover from the **representation size limit**, not OOM.
Success uses storable unit `()` for buffer operations. Err messages are borrowed
literals and must not be freed. Null C pointers, invalid indexes, managed/native
allocation failure and impossible invariants panic. Panic exits without a scope
unwinding promise. A nonnull live, readable, NUL-terminated input remains the
caller's native precondition.

Storage is ARC-managed and acyclic. Indexed operations cost O(tree height);
bulk import, append, equality and export walk blocks. See the
[representation rationale](managed-string-prototype.md) for measured overhead.

`is_empty()` checks logical length. `clear()` releases all backing nodes and resets
length and capacity to zero for this owner. `append_string(value:&String)` copies
all managed bytes, including embedded NUL, and preserves the String. It currently
uses one temporary counted native copy internally; that allocation is released
before returning. The ordinary infallible capacity/OOM contract applies.
