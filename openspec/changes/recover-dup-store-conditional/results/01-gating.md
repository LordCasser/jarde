# Task 1.1 — the dup-store dance's refusal points, located and gated apart

The proposal's premise is that the anchors refuse at the copy family's gate
("the copy at BCI N has no proved local assignment") and that the reference/null form may land
elsewhere. Both were measured on the merge-state build (`HEAD` of this worktree, which already
carries `recover-postfix-old-value-snapshot` Phase A), with the anchors rendered by
`results/render.sh` (self-header asserted) and with temporary `eprintln!` instrumentation at the
refusal sites. The transcripts are `results/renders/*.txt` (HEAD) and the sections below.

## The two anchors, verbatim at HEAD

`OP2.condAssign` (`static boolean condAssign(int x){ return (x = x + 1) > 0; }`,
`iload_0; iconst_1; iadd; dup@3; istore_0; ifle`) — whole method quoted:

```
// @bytecode 3
// the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
// @bytecode 4
// the copy at BCI 3 has no proved local assignment
// @bytecode 0 8 12 5
// the copy at BCI 3 has no proved local assignment
// @bytecode 13
// the value at BCI 13 is the entry state of stack depth 0, which no instruction produced
```

`AC.ioLoop` (`while((line = read()) != null)`, `invokestatic read; dup@5; astore_1; ifnull`) —
whole method quoted, and the *reported* line is a region cascade, not the copy family:

```
// @bytecode 0 2 10 16
// local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
```

The cascade's own cause is in the method's report (`jre_region_unmet_precondition`):

```
the loop@1 rule did not claim the block at BCI 2: it requires a test block whose every instruction
is part of a value expression, and the instruction at BCI 5 is not part of one, so presenting the
structure would have moved that effect out of the shape it decides
```

BCI 5 is the `dup`. The two forms therefore land at **two different gates**, as the matrix allowed.

## Gate A — the int form: the copy family's own proof and render arm

Instrumentation of `prove_local_assignments` (the walk that proves `dup; store; test`) and of
`render_value`'s `Operation::Duplicate` arm shows, for `OP2.condAssign`:

```
INSTRUMENT local-assignment parameter slot 0 (name arg0) at BCI 4
```

The physical shape **is** proved — `local_assignment_at` answers a shape for `dup@3`, `istore_0@4`
and the test `ifle@5` — and the walk then discards it because the store target is a *parameter*
slot (`shape.slot < parameters`). No entry reaches `local_assignments`, so the render arm refuses
with the copy family's own text. The gate is one place: the proof's admission rule plus the arm
that reads the map it fills.

`OP2.condAssignOld` refuses at a **third** gate, and its bytecode is not the dance at all: javac
compiles `(x += 1)` on a slot as `iinc 0,1; iload_0; ifle` (both legs, verified by `javap`), so the
short-circuit chain proof is what runs, and its statement build refuses with

```
INSTRUMENT short-circuit statement refusal at BCI 16: the short-circuit local at BCI 16 has no closed Boolean declaration decision
```

i.e. `proves_boolean_local_store` admits a read of the chain's boolean local only in a
`putstatic Z` / `boolean` `ireturn` / `append(Z)` position, and `b` is read by the ternary's `ifeq`.
That is a different mechanism from this change's decision 1 (recorded here, not improvised).

## Gate B — the reference form: the region layer's `StatementFree` precondition

`AC.ioLoop`'s dance sits in a **loop test block**. Two facts follow:

1. `region.rs::test_is_pure` (the `loop@1` rule's declared `Precondition::StatementFree`) admits only
   `Push`/`Load`/`Arithmetic`/`Negate`/`NumericComparison` and the test's own call/field/array-length
   producers. The `dup` (BCI 5) and the `astore_1` (BCI 6) are neither, so the loop region is never
   claimed and the method is quoted through the local-crossing cascade above.
2. `prove_local_assignments` collects only `Region::If` and `Region::ShortCircuitValue` tests
   (`Region::Loop { .. } => {}` — "this proof never enters a loop"), so even with the region claimed
   the loop test's dance would have no proof.

Both landing points are therefore needed for the reference form; the int form needs only Gate A.

## The four copy-family members at HEAD (the same-gate question)

| member | shape | state at HEAD |
| --- | --- | --- |
| array dance | `if((v = bs[i]) == true)` (`aaload; dup@13; istore_1; iconst_1; if_icmpeq`) | refused — `the copy at BCI 13 has no proved local assignment` |
| postfix old value | `iload i; iinc i,1` | **recovered** on main (`int local1 = arg0++;`) — `recover-postfix-old-value-snapshot` Phase A |
| putfield chain | `FAM.a = FAM.b = FAM.c = 5` (`iconst_5; dup@1; putfield c; …`) | refused — `the copy at BCI 1 has no proved local assignment` |
| **dup-store** | `iadd; dup@3; istore_0; ifle` | refused — copy family (Gate A); in a loop test, Gate B first |

Three of the four still refuse at the same emitted text (the copy family); the postfix member was
implemented by its own change and no longer refuses. The dup-store member is the only one whose
*reference* form additionally refuses earlier, at the region layer.

## The two-reader criterion, checked against the negatives

* the anchor shapes are exactly two readers: the copy the store takes and the copy the test reads
  (`local_assignment_at` requires each copy's sole use to be one of the two);
* the multi-reader negative (`ExtraCopy.class`, `iconst_5` patched to `dup`: the first copy feeds
  another copy rather than the test) fails the same identity check and stays refused;
* `CF-06`'s `NegativeAssignments` controls (interleaved effect, live handler, loop condition) stay
  refused: the first two fail the physical proof, the third is a **loop** test whose target has a
  later reader, which this slice deliberately does not present (the split form is not sound in a
  re-evaluated condition — see `02-implementation.md`).
