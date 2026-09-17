# Method-signature grammar fixtures

These four class files are real `javac` output used to pin the `MethodTypeSignature`
grammar (JVMS 4.7.9.1) of the metadata consumer: a method signature's `Result` is a
`JavaTypeSignature | V`, and `{ThrowsSignature}` follows it. The samples are the
counterexamples from the 2.2 review round: before the fix, every one of them made the
signature scan fail with `query_signature_malformed` even though the input is legal, and
they are checked in so the regression tests run against compiler output instead of only
against hand-built structure.

## Provenance

- **Compiler:** `javac` 23.0.1 (`/usr/bin/javac`, reported by `javac -version` as
  `javac 23.0.1`).
- **Sources:** the four `.java` files in this directory, verbatim as compiled.
- The compiler is a generation-only input. It is not checked into this repository and is
  not a runtime or test-time dependency: the tests read the checked-in `.class` bytes.

Exact command (run from this directory):

```text
javac --release 17 -g:none -d v17 GenericReturn.java TypeVarResult.java ThrowsVar.java ThrowsMixed.java
```

## Outputs

| file | bytes | SHA-256 | method signature the tests read |
| --- | ---: | --- | --- |
| `v17/GenericReturn.class` | 242 | `97e4287cf9b3d175338d0d3dfab7f231c9101f9835679fc2a239063d5656e82b` | `()Ljava/util/List<Ljava/lang/String;>;` |
| `v17/TypeVarResult.class` | 290 | `9094fc4199da6562358a1cee3221f657cde2534eecb31e15a51907bca471cfa0` | `(TT;)TT;` |
| `v17/ThrowsVar.class` | 296 | `de96c79a7d824bcf4a375ba9a244f845c8e2a980e53588e7b6ca5a22f73e3685` | `()V^TE;` |
| `v17/ThrowsMixed.class` | 341 | `ca0919badf235b7fc72603b568acabac6b836b7cef61b332272fc2b281b0b1a2` | `()V^TE;^Ljava/io/IOException;` |

## What each sample covers

- `GenericReturn` — a generic **return** type: `List<String>` is a `JavaTypeSignature` with
  type arguments, which the descriptor production cannot read. `java/lang/String` exists
  nowhere else in the file (no `CONSTANT_Class` and no other `Utf8`), so a hit can only
  come from inside the signature.
- `TypeVarResult` — a type variable as the return type: `(TT;)TT;` names no class type at
  all, so the sample must scan `Complete` and produce **no** item for `T`.
- `ThrowsVar` — `throws` a type variable: `()V^TE;` must not fail and must produce no item
  for `E`.
- `ThrowsMixed` — `throws E, java.io.IOException`: a `ThrowsSignature` list with both a
  type variable and a class type. `java/io/IOException` must produce a
  `GenericSignature` item from the method signature (`java/io/IOException` has a
  `CONSTANT_Class` entry too, because the `Exceptions` attribute lists it, but a
  `Signature`-only request reads only the signature).

`TypeVarResult`, `ThrowsVar` and `ThrowsMixed` also carry a **class** signature
(`<T:Ljava/lang/Object;>Ljava/lang/Object;` and `<E:Ljava/lang/Exception;>Ljava/lang/Object;`
respectively), which already parsed before the fix. The tests use it as the control that
the class was really scanned: it is the one signature item a `java/lang/Object` or
`java/lang/Exception` query finds in those files.

Note recorded while generating these fixtures: a method whose `throws` clause holds only
concrete classes gets **no** `Signature` attribute at all (`javac` emits one only when the
generic signature differs from the descriptor), which is why the concrete-throws case is
covered by `ThrowsMixed` rather than by a sample of its own.
