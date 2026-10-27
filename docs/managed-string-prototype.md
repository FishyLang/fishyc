# Managed String representation history

The prototype and former facade files are retired. The one production implementation
of String, ByteBuffer, byte storage and text operations lives in `src/stdlib.fih`.
Use the [String](string.md) and [ByteBuffer](byte-buffer.md) references for current APIs;
remove old module `using` declarations. ManagedString/MutableBytes/PrototypeBytes
were historical names, and are no longer production declarations.

The historical accepted representation used a managed binary tree with 128-byte leaves,
a shared immutable empty root, logarithmic indexed access, linear bulk walks,
independent builder copy/snapshot publication and a 1 GiB bound. Every edge is
ordinary acyclic ARC storage; no raw backing allocation or finalizer is assumed.

The earlier engineering reports retain the original experiments and measurements.
There is no second source implementation or compatibility module graph to maintain.
