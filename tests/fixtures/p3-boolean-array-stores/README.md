# Boolean array low-bit store fixtures

These are the permanent Java 8 (`--release 8 -g:none`, classfile version 52) inputs for
`tests/p3_boolean_array_stores.rs`. `RawBool.java` and `Order.java` are ordinary `int[]` sources;
the committed classes under `v8/` are the verifier-valid, narrowly patched `[Z` variants.
`RawBoolRunner.java` and `OrderRunner.java` are source-only reflection runners. Their expected
stdout is captured in `expected/` from the frozen patched JVM executions.

`patches/` records exact source and result hashes, method descriptors, Code hex, and opcode offsets.
The originals are SHA-256 `e38d0e46a055d5183001d2e51ed329c2a0f121409f27ead257d34f6ab76317e3`
(RawBool, 172 bytes) and `63dc18e93763ffbf9d6ce73a299a7d140023eb278dd2d430f6e47d208bb55ef0`
(Order, 532 bytes). The committed patched classes are SHA-256
`32286d138ace6a328a8c5ba7d0a0d66fadb4bf9dbbab79385e6dc3a01dcd6c2b` (RawBool, 172 bytes) and
`31d8fc3394c39cfe9a9e62da435af9245d2f26cd24e62e077e7310e1bf97d414` (Order, 532 bytes).

The reproduction and pre-change tool-stage records are in
`openspec/changes/recover-boolean-array-stores/verification-1.2.md`. No compiled runner classes are
stored in this fixture; tests compile runners into private temporary directories.
