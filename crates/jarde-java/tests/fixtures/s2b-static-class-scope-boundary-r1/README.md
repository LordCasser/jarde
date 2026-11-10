# Static class-variable scope boundary

`GenericScopeBoundary.java` was compiled with Corretto 8 (`javac 1.8.0_432`, `-g -source 8 -target 8`). `GenericScopeBoundary<T>` has an ordinary static `Object pass(Object)`; `GenericScopeShadow<T>` supplies the legal positive control `static <T> T pass(T)`, where the method's `T` shadows the class's `T`.

`GenericScopeBoundary-invalid-static-signature.class` is a deliberate classfile mutation, not legal javac source. It adds one `CONSTANT_Utf8` `(TT;)TT;` and a two-byte method `Signature` attribute to `GenericScopeBoundary.pass`. The JVM descriptor and `Code` bytes are unchanged. `javap -v` output and SHA-256 pins are in `original.javap.txt`, `mutant.javap.txt`, and `pins.json`.

The independent Rust integration test binds actual `MethodIr` and selected physical headers through `declared_value_types`. Its negative case checks that the copied local value and method declaration cannot publish the enclosing class `T` from a static method; its positive case checks that the method-scoped shadow `T` remains spellable. The test is prepared but was not run because Cargo is occupied by the parent task.
