# EM23 nested field update: corrected JADX path

This supplements and corrects the attribution in `architecture-audit-luna-v1.md`. It is a read-only source audit: no implementation, product defect finding, or EM23 completion is claimed.

## Actual JADX field-to-compound rewrite

Atlas was opened on `/Users/lordcasser/workspace/testzone/jadx` and used for scoped symbol searches in `SimplifyVisitor.java`, `RegisterArg.java`, and `InsnWrapArg.java`. Those searches located `convertFieldArith`, `simplifyArith`, `RegisterArg.sameCodeVar`, and `InsnWrapArg.equals`; the behavior below was then verified directly in those source ranges.

The field rewrite is `jadx-core/src/main/java/jadx/core/dex/visitors/SimplifyVisitor.java:64-77,80-107,110-150,609-665`, not `PrepareForCodeGen.modifyArith`:

1. `visit` walks method basic blocks; `simplifyBlock` recursively simplifies wrapped arguments before dispatching an instruction. `IPUT`/`SPUT` are sent to `convertFieldArith` (`:110-150`).
2. `convertFieldArith` only proceeds when the store value is an `InsnWrapArg` around `ARITH` (or the separate string-concat shape), whose left operand wraps `IGET`/`SGET` (`:609-625`).
3. It requires exact `FieldInfo.equals` between the store and read (`:626-629`). `FieldInfo.equals` compares declaring class, name, and type (`jadx-core/src/main/java/jadx/core/dex/info/FieldInfo.java:86-97`). For instance `IGET`→`IPUT`, it additionally requires the read receiver argument to equal the store receiver argument (`SimplifyVisitor.java:631-638`).
4. `InsnArg.equals` on registers compares register number and SSA variable (`RegisterArg.java:206-216`). For wrapped instructions, `InsnWrapArg.equals` recursively checks `InsnNode.isSame` and each wrapped argument (`InsnWrapArg.java:66-86`); `InsnNode.isSame` checks instruction type/arity and wrapped children (`InsnNode.java:368-395`), while `IndexInsnNode.isSame` also compares its index (`IndexInsnNode.java:37-46`). This is structural identity of these decompiler operands, not a general Java proof that arbitrary repeated expressions are interchangeable.
5. After the guards pass, it duplicates the wrapped field-read as the compound target, unbinds the old read and store-value use, then builds `ArithNode.oneArgOp` with the original arithmetic operator and RHS (`SimplifyVisitor.java:639-649`; `ArithNode.java:68-77`). `InsnGen.makeArithOneArg` prints that node as `<target> <op>= <rhs>` (`jadx-core/src/main/java/jadx/core/codegen/InsnGen.java:1216-1236`).

The method's explicit `field.equals(innerField)` and receiver-argument equality are the core guards. `PrepareForCodeGen.modifyArith` (`PrepareForCodeGen.java:229-249`) is a later register-result normalization, and `ModVisitor.fixFieldUsage` (`ModVisitor.java:175-210`) handles receiver casts/visibility. Neither is the field compound-assignment conversion. The v1 cited them as context but gave them too much explanatory weight.

## What the test establishes—and what it does not

`TestFieldIncrement2.java:17-23` indeed supplies two distinct source forms: explicit `this.a.f = this.a.f + n` and compound `this.a.f *= n`. The test asserts rendered `this.a.f += n;` and `this.a.f *= n;` at `:26-32`.

The source spelling of `test1` alone does not tell us whether its compiled input has two distinct receiver values or one shared receiver value. The JADX conversion above only folds the `IPUT` form when its decoded `IGET` and `IPUT` receiver operands compare equal. The test's output assertion is evidence that its fixture reaches the asserted rendering, but it does not publish the raw instruction graph. Therefore do not label `test1` a guaranteed negative control for receiver identity: its compiled representation may already share the evaluated receiver. Conversely, do not infer that same field name means the receivers match. The baseline must inspect the actual classfile inputs/decoded evidence.

For Jarde, the conservative boundary is its own proof: `crates/jarde-java/src/build.rs:13936-14118` accepts only instance-int `putfield` with `iadd`/`isub`, and requires the read and write receiver SSA values to be the two outputs of one `dup` (`:14004-14030`), with single-use and order checks (`:14032-14058`). It has no `imul` branch, while `crates/jarde-java/src/ast.rs:728-745` and `build.rs:27206-27263` only represent/emit `=`, `+=`, and `-=`. Thus `*=` is a real unimplemented operator within this narrow proof path; this audit does not establish a broader nested-field defect.

## Bounded next-baseline guidance

Keep both JADX methods in a complete-class baseline, but classify them by the original compiled receiver evidence, not by source spelling:

- `test2` / `this.a.f *= n`: primary candidate. Confirm one target receiver value is duplicated for nested `getfield f` and `putfield f`; this is the exact location invariant Jarde already proves for add/sub.
- `test1` / explicit `this.a.f = this.a.f + n`: comparison case. Record whether original bytecode uses one shared receiver or distinct `this.a` reads. If shared, it exercises the same receiver identity shape despite two source occurrences. If distinct, retain the explicit read and write behavior and do not synthesize `+=` from field owner/name/descriptor equality alone.

This is a conservative implementation boundary based on current Jarde SSA evidence, not a claim that every Java Memory Model implementation must reject all optimizations of repeated nonvolatile reads. Repeated reads can have language/runtime subtleties; this audit makes no absolute optimization claim. It only says the existing Jarde proof does not establish equality for two distinct SSA receiver values. Keep the slice to nonvolatile `int this.a.f`; do not generalize to other operators, volatile fields, or arbitrary side-effecting receiver expressions here.
