# Declared call binding controls R1

Frozen Java 8 declarations and consumer call sites for `declared_call_binding`. `CallBox<T>` contains the erased generic overload pair `set(T)` / `set(String)`, a unique `get()`, `fixed()` plus a two-argument `fixed(String,String)` overload, and generic `infer(U)`. Separate physical classes provide a fixed/varargs overload pair, a compiler-generated bridge/synthetic override, and an ordinary same-descriptor inherited override.

`javap` receipts establish each descriptor and the `ACC_VARARGS`, `ACC_BRIDGE`, and `ACC_SYNTHETIC` flags. No method is executed.

The initial consumer source tried to compile a direct `CallBox<String>.set(String)` invocation. Corretto 8 rejects that source as ambiguous because the two substituted formals coincide; the rejected source and compiler output are preserved in `failed-attempt-r1/` and `javac-initial.log`. The final consumer instead has one actual raw-receiver call and a separate `CallBox<String>` parameter Signature. The Rust control combines the resolved physical `set(String)` member from the actual call with that independently real, selected `CallBox<String>` application to test the overload collision directly. It does not claim an emitted Java call exists at the ambiguous application.
