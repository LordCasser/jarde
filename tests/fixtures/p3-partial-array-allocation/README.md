# P3 partial array allocation fixture

This is the minimized `core-no-overload` input for `recover-partial-array-allocations`.
It was compiled with javac 23.0.1 using:

```text
javac --release 8 -g:none -d v8 PartialArrayEffects.java PartialArrays.java
javac --release 8 -g:none -cp v8 -d v8 PartialArraysRunner.java
```

The committed class is the verified `PartialArrays.class`; the source compilation is the
reproducible origin of its bytecode, while the class bytes are the fixture under test.

| property | value |
| --- | --- |
| class | `PartialArrays` |
| class-file version | 52.0 (Java 8) |
| bytes | 777 |
| SHA-256 | `487389b770dbfdd226ae9ae3ff87c9d6be9df4e7df01781505f5d2a55e10e8f2` |
| methods / `Code` attributes | 10 |
| debug attributes | none (`-g:none`) |
| test | `tests/p3_partial_array_allocation.rs` |

The class contains exactly one field and these ten methods: the constructor, `primitive(I)[[I`,
`reference(I)[[Ljava/lang/String;`, `prefix(II)[[[I`,
`referencePrefix(II)[[[Ljava/lang/Object;`, `local(I)[[I`, `field(I)V`, `length(I)I`,
`effects(II)[[[I`, and `complete(II)[[I`. The bytecode shapes are the intended boundaries:

* `primitive` and `reference` use `anewarray` with an array component, leaving a partial outer
  array;
* `prefix` and `referencePrefix` use `multianewarray` with count 2 against a three-dimensional
  descriptor, leaving the tail dimension unallocated;
* `local`, `field`, and `length` consume the same partial allocation through a local store, a
  static field store, and `arraylength` respectively;
* `effects` evaluates two dimension calls before `multianewarray`, preserving the first/second
  failure and trace order; `complete` is the full-dimension control case.

The source-only runner has 69 independent rows over negative, zero, and positive dimensions.
The original frozen class and the complete JADX output produce identical 69-line output under
`java -Xverify:all`. The current jarde output has 18 `@bytecode` references and the complete
recovered source fails `javac --release 8`; the RED test keeps this gap visible. The fixed result
must compile and run the same 69 lines as the frozen original. The full evidence and replay logs
are preserved under
`openspec/evidence/java-syntax-2026-09-22/numeric-conversions/partial-array-allocation/core-no-overload/`.
