# P3 narrow B/C/S field stores

This is the minimized `narrow-field-stores/core-bcs` input for
`recover-narrow-field-stores`. The Java sources declare the six target fields as `int`; the
committed class is produced by a deterministic class-file patch that changes only each target
`field_info` descriptor and its matching `Fieldref` `NameAndType` descriptor:

```text
javac --release 8 -g:none -d v8 NarrowFieldStoreEffects.java NarrowFieldStores.java NarrowFieldStoresRunner.java
python3 patch_field_stores.py NarrowFieldStores.source.class NarrowFieldStores.patched.class
```

The patch does not alter any method `Code` bytes. The source and patched classes are preserved in
the sibling evidence directory, together with `patch_field_stores.py`, its JSON report, complete
`javap` output, and the original/patched JVM logs.

| property | source class | committed patched class |
| --- | ---: | ---: |
| bytes | 1502 | 1526 |
| SHA-256 | `94cbe660367772ce2bf2debc860a9c7ad4607e9b940f0176b6d8a24e8471f8cb` | `d4797140eed55121de091518e7db4d9dcb5052d0c9a451ef6ded07e1589af8ef` |
| class-file version | 52.0 | 52.0 |
| methods / `Code` attributes | 19 | 19 |
| method Code bytes | unchanged baseline | byte-identical to source |

The six patched descriptors are `byteField:B`, `charField:C`, `shortField:S`, and their
`static*` counterparts. Their writers cover instance `putfield`, static `putstatic`, one
effectful producer per type, and a receiver helper whose producer is evaluated before a null
receiver reaches `putfield`. The three naturally narrow `ordinary*Field` methods are controls for
legal source-typed field stores.

The source-only runner uses 21 int boundary values, producer success/failure, previous field
values, null receiver with producer failure and success, static stores, and ordinary controls. The
original and patched classes each verify and execute 267 rows under `java -Xverify:all`; their
runtime text intentionally differs where B/C/S fields truncate the int value. Runtime output
SHA-256 is `00e009d8fda381ec5540a0fc745b8a42f04edbb7f98caa70f84d1996caeb7d6f` for the source
class and `eb57ef6493d6713fbaaaf26b87e55bf5708ca8acfb671270f6cc9b6c175cc10c` for the patched
class.

The fixed CLI `/tmp/jarde-cli-deferred-final-ecab` currently emits 18 `@bytecode` references at
the six narrowed field stores and their producer/consumer positions. Its complete Java text
still compiles as a no-op-shaped class, so the RED test checks field values, producer calls, and
exception order against the patched JVM oracle rather than accepting compilation alone. The
CLI SHA is `ecab8244d1709765fa0f2b0effcebe49b9f066eea911843d11d34abec5837330` before and after
the evidence run.
