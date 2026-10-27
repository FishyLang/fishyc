# Consumer needs, recorded before stdlib implementation

The first consumer analyzes an existing managed String containing a small
multi-line note. It counts bytes, LF-delimited lines (including a final
unterminated line), ASCII-whitespace-delimited words and whitespace bytes. It
checks the trimmed note's header/trailer, locates its first LF and extracts the
first line. The deterministic entry point supplies data, not a file or stdin.

The initial executable is retained as `consumer-before.fih` in the run's /tmp
evidence directory. Its local copy_range, matches_at, first, trimmed and
whitespace helpers expose the missing pieces: byte_at creates an Option and
walks the tree per byte; slice copying additionally calls builder.push per byte;
callers must repeat range arithmetic; prefix/suffix comparisons and trim scans
are otherwise reimplemented locally. Word/line statistics remain a natural small
consumer loop, rather than becoming five application-specific String methods.

Stabilize only try_slice(start,end), starts_with, ends_with, find and trim_ascii,
plus is_ascii_whitespace in stdlib/ascii.fih, shared by trim and the consumer.
Use Result for checked slice positions, a half-open independent copy, Option for
first search position, byte-exact matching (including NUL), and ASCII bytes
9 through 13 plus 32 for whitespace. Unchanged trim shares the same immutable
String; partial trim copies; an all-whitespace nonempty value becomes empty.

Defer contains (find().is_some composes), panicking slice, separate byte search,
digit/alpha helpers, split/line collections and public traversal. None is
necessary to finish this consumer. Search starts with a simple naive candidate
scan, without per-candidate heap scratch or per-byte Option allocation.
