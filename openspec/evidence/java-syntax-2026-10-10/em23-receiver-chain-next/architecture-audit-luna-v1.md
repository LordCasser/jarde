# EM23 receiver-chain field update: bounded architecture audit

Status: read-only analysis; no implementation or EM23 completion is claimed. Scope is nonvolatile `int` field `this.a.f` and the two shapes in JADX's `TestFieldIncrement2`.

## Source evidence

- JADX fixture: `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arith/TestFieldIncrement2.java:10-24`. `TestCls.A.f` is an `int`; `TestCls.a` is an `A`. `test1(int n)` spells `this.a.f` on both sides (`:17-19`); `test2(int n)` spells `this.a.f *= n` (`:21-23`). Its integration assertion checks both rendered forms at `:26-32`.
- JADX transformation: `jadx-core/src/main/java/jadx/core/dex/visitors/PrepareForCodeGen.java:81-90,229-249` runs `modifyArith` per basic block. That routine converts an arithmetic instruction to `ARITH_ONEARG` when its result equals/is the same code variable as the left register. `InsnGen.java:1196-1236` then emits ordinary binary arithmetic or the operator's compound spelling (`+=`, `-=`, `*=`, etc.). This is a register/arithmetic-shape simplification; the inspected guard does not compare nested field receiver evaluations.
- JADX field adjustment: `ModVisitor.java:159-161,175-210` applies `fixFieldUsage` to `IGET`/`IPUT`; it adds a cast when receiver static type/visibility requires one. It does not establish that two separate `this.a` reads yield the same receiver object.
- Jarde member evidence: `crates/jarde-java/src/field.rs:512-533,709-814` records each access's BCI, access kind, staticness, owner, name, descriptor, and SSA receiver/value. `verify` checks the receiver's stated type against the member owner. This establishes which member each instruction names, not equivalence between receivers from distinct instructions.
- Jarde compound proof: `crates/jarde-java/src/build.rs:13936-14118` currently accepts only nonstatic descriptor-`I` `putfield`, with `iadd`/`isub`. It requires the read and write to name the same owner/name/descriptor, then separately proves both receiver values are the two outputs of one JVM `dup` (`:14004-14030`), checks single consumers and instruction order (`:14032-14058`), and bounds the receiver/RHS expression dependencies (`:14060-14105`).
- Jarde presentation/provenance: `build.rs:27206-27263` turns a proved field update into `FieldAssign` with `Add` or `Subtract`; `ast.rs:728-745,770-784` exposes only `=`, `+=`, and `-=`; `emit.rs:848-865` prints the assignment. `field.rs:318-344` records the primary write and, for compound add/sub, derived read evidence. `build.rs:15182-15188` anchors a proved update at the store and adds duplicate/read/arithmetic BCIs as derived origins.

Atlas was opened on this repository and used for narrow symbol locations (`prove_field_update`, `FieldAssign`, `field::Plan::claim`, `Builder::field_write`). The conclusions above were checked against the source ranges cited; no whole-repository index or toolchain was run.

## Shape distinction and reusable proof

The explicit assignment in `test1` is not interchangeable with a compound target. Java evaluates the left-side receiver and then evaluates the right-side expression. The two textual `this.a` evaluations are distinct reads and can select different `A` objects if `a` changes between them. Equal `A.f` owner/name/descriptor therefore does not prove equal field locations. Keep the ordinary read and write receivers distinct; do not fold this shape merely because both accesses name `A.f` or happen to render as `this.a.f`.

The compound assignment in `test2` evaluates the target receiver once. The existing Jarde proof's strongest reusable part is precisely this location proof: the same `dup` produces the receiver for `getfield f` and `putfield f`, both have one intended consumer, and the read/arithmetic/store order is constrained. Its member-pool identity, receiver typing, expression-boundary, consumer, and `OriginSet` checks are reusable for a narrowly bounded multiply case. The current proof cannot yet produce `*=`: it excludes `imul`/`Multiply`, and the AST/emitter has no `Multiply` assignment operator. Adding only a multiply spelling without the same receiver-identity proof would be unsound.

## Recommended next baseline

Priority: one complete-class baseline derived from this exact fixture shape, with both methods retained as paired controls:

1. `compoundMultiply(int n)`: `this.a.f *= n` — the positive candidate for proving single evaluation of the nested receiver and later presenting `*=`.
2. `explicitAdd(int n)`: `this.a.f = this.a.f + n` — a control that must preserve two independently evaluated receiver chains and must not be converted to a compound update by member-name equality.

Keep `A.f` and `TestCls.a` nonvolatile, preserve a simple initializer/constructor, and use a fixed runner that invokes each method and observes `a.f`. Compare the original compiled class with the full generated class; retain raw runtime output and verify source-map origins. The baseline should answer whether current ordinary field reads/writes already preserve the explicit control and whether the multiply case reaches a bounded fallback. It is not evidence that either behavior is already implemented.

Preconditions/invariants for any later implementation are limited to this slice: target is a nonstatic `int` field; the member identity is exact; the compound read and write consume the same SSA receiver produced by the target-evaluation copy; no extra consumer or reordered read/arithmetic/store is admitted; and the explicit two-chain control remains unmerged. Preserve write/read/arithmetic/receiver BCIs in provenance. Volatile fields, other operators, arbitrary receiver chains, and broader update families are outside this audit.
