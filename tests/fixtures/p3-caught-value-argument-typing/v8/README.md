# p3-caught-value-argument-typing/v8/S1.class

Committed sample for the catch-parameter slot-reuse presentation check: javac 23.0.1,
`--release 8 -g` (the default debug table the compiler writes), compiled from the sibling
`S1.source.java` with

```
javac --release 8 S1.source.java
```

The class is the fixed patrol fixture of
`openspec/evidence/java-syntax-2026-10-01/catch-param-slot-reuse/` (the same 1221 bytes):
`twrHelper` declares `try (S1 r = new S1())` and a `catch (IllegalStateException e)` whose
parameter **reuses the resource's slot 0** (the binding `astore_0` at BCI 38 overwrites the
resource's store at BCI 7), and its handler passes the parameter to `tag`: `return tag(e);`.
`plainHelper` is the no-reuse control: its catch parameter owns slot 0 alone.

SHA-256 of `S1.class`:
`e372f219aa6b556e6717b10648d99ff3d5f4bdaca5a4b4f68233d2f157b3c765`
