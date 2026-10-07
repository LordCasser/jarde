# Task 1.1 — where the val$/super order is decided (read + transcript)

Date: 2026-10-07T06:17:01Z (UTC)

## The answer: `crates/jarde-java/src/ctor_order.rs`, not `member_inner.rs`/`facade.rs`

```
$ grep -rn "present_prologue_first" crates src
crates/jarde-java/src/build.rs:8082:    crate::ctor_order::present_prologue_first(
crates/jarde-java/src/ctor_order.rs:77:pub(crate) fn present_prologue_first(

$ sed -n '8074,8096p' crates/jarde-java/src/build.rs   # the one call site
            }
        }
    }
    // A constructor whose pre-super statement prefix is the compiler's certified synthetic
    // capture group (`this$0`, `val$x`) is presented prologue-first — `super(…)` as the first
    // statement, the group after it — because the byte order the layer above presented is legal
    // JVM but not legal Java source. Everything that prefix is not (a user field's write, a
    // computed value, an interleaved effect) is left exactly where the bytes put it.
    crate::ctor_order::present_prologue_first(
        &mut builder.stmts,
        builder.ssa,
        builder.operations,
        builder.fields,
        inputs.class_fields,
        inputs.declaring_class,
        inputs.prologues,
        inputs.parameters,
        inputs.has_receiver,
        builder.budget,
    )?;
    Ok(Program {
        field_increments,
        statements: builder.statements,
```

The order is decided by `ctor_order::present_prologue_first` (`crates/jarde-java/src/ctor_order.rs`), called once per built body from `build.rs::build` (the `Program` assembly tail).

- `member_inner.rs`/`facade.rs` (root crate) own the *anonymous projection* at the allocation site (which is refused for `DB$2`: superclass `java.util.ArrayList` is not a same-package spellable source type -> `anonymous_super_source_type_unproved`) and the *companion instantiation* text `new DB$2(arg0)`; neither writes the companion constructor's statement order.
- The order gate itself (guard, since `recover-ctor-reorder-dispatch-guard`): `operations.get(prologue.bci)` -> `Operation::Invoke(target)` -> `owner == "java/lang/Object" && name == "<init>" && descriptor == "()V"`, else verbatim.

## The class-header declaration view the gate already has

```
$ sed -n '374,382p' crates/jarde-java/src/build.rs   # Inputs: the class header facts
    pub(crate) direct_super_class: Option<&'a jarde_reader::model::JvmBytes>,
    /// The direct interfaces from the same class header as the decoded body.
    pub(crate) direct_interfaces: &'a [jarde_reader::model::JvmString],
    /// The current class's member headers from that same class header, when available.
    pub(crate) class_methods: Option<&'a [jarde_reader::classfile::MemberHeader]>,
    /// The current class's field headers from that same class header, when available.
    pub(crate) class_fields: Option<&'a [jarde_reader::classfile::MemberHeader]>,
    /// The verdict of the `bridge@1` rule for this very body, when the member is declared a bridge
    /// or its body is the forward a bridge is written as.
$ grep -n "class_methods" crates/jarde-java/src/build.rs | tail -3
8758:    class_methods: Option<&'a [jarde_reader::classfile::MemberHeader]>,
24592:            self.class_methods.map(|methods| {
24747:        if let Some(methods) = self.class_methods {
```
