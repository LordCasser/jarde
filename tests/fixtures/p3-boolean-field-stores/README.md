# P3 boolean field stores

`ZFieldStores.java` is compiled with `javac --release 8 -g:none`; the committed
`v8/ZFieldStores.class` is the frozen source class after changing only `instanceFlag` and
`staticFlag`, plus their matching `Fieldref` descriptors, from `I` to `Z`. The descriptor-only patch
keeps all 11 Code arrays byte-identical. `patch_field_stores.py` records the exact input-side patch;
the recovered output is never edited.

The source-only runner covers seven positive and negative integer values through direct and
effectful instance/static writes, including producer failure, null receivers and ordinary boolean
controls. The patched class is 914 bytes, class-file version 52.0, SHA-256
`671ee4999373f8d95e989a480ded9a648b9733c44ddb5c12f408b24e3b4b1e96`. The source class SHA-256 is
`f1089b9ed4bf8802697c7c88c7bdd79f79af93ec4d6f04135fefb764e020c4fd`. `code-sha256.json` pins all
11 source/patched Code SHA-256 pairs. The pre-fix jarde output is preserved in
`pre-implementation-runtime.stdout`; it differs from the patched JVM on 26 of 40 rows.

`tests/p3_boolean_field_stores.rs` validates the real descriptors and Code hashes, source mapping,
the frozen pre-fix red result, and (with a JDK) recompiles and executes the full recovered class
against the patched JVM runner.
