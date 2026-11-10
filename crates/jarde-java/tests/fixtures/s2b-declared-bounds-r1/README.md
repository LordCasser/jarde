# Declared generic bounds control

These three classfiles were compiled from the frozen Java 8 sources and flags recorded at `input-manifest.json` in the change evidence. `Bounded` declares `T extends String` and a method-local `T` shadow; `SelfBound`/`SelfChild` encode an ordinary F-bound and its valid recursive argument. The new control reads classfile signatures only. It executes no target class.

The existing physical `Foo`, `List`, `Object`, and `String` headers are reused byte-for-byte from `s2b-bound-type-identities-r1`; their source path, length, and SHA-256 are pinned in the input manifest. Compiled outputs and `javap` signature confirmation are pinned in `compile-receipt.json` under the same evidence directory.
