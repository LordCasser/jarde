# Task 1.1 — the refusal chain at HEAD, and the gating matrix

Every render here is produced by `render-set.sh` (which asserts the self-header of every text it
accepts) against the worktree's own CLI. The four directories are the four states of the build:

| directory | state |
| --- | --- |
| `baseline/` | HEAD (no change of this slice) |
| `continuation-only/` | the guard certificate's continuation completion alone (commit A) |
| `copy-family-only/` | the loop-test position's admission alone (commit B) |
| `both/` | A + B — the delivered state |

## The refusal chain `IO.readAll` had at HEAD (instrumented, then removed)

1. `guard::prove_resource_guard_finally` reads the normal path's completion. `readAll`'s normal
   path ends in `41: goto 53` — the body's completion carried past the close into the statement's
   own continuation — and the `Transfer` arm of the completion match returned `Ok(None)`:
   *"The transfer form … is not this certificate's. `readAll`'s interrupted read and the fixed
   CF-16 void loop both end here"*. **The resource guard is never claimed.**
2. With no guard claimed, the region walk reaches the loop's header block (BCI 17) with no
   accounted handler edge and quotes it —
   `jre_region_exception_edge`: *"block at BCI 17 leaves through exception handler 0: a handler's
   shape is not part of the recoverable subset"* — and blocks 27/37/44 stay uncovered
   (`jre_region_uncovered_blocks`). The loop is never a `Region::Loop`, so the copy family's
   purity criterion (`region::test_is_pure`) is **never asked**.
3. `plan_declarations` then finds the resource local's (slot 1) uses at paths `[0]`, `[1]`, `[2]`
   — the prologue, and the two quoted gaps — so it is `Incomplete` with the crossing diagnostic,
   and the whole member is quoted with that one line:
   `// local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented
   as one lexically bound definition-use slice` at `// @bytecode 0 17 27 37 44 53`.

So at HEAD the **emitted** diagnostic is the *crossing* cascade, and its cause is the guard
certificate's completion narrowing — not the copy family's purity criterion the io slice's boundary
note names. The purity criterion is what refuses the shape *once the guard is claimed* (step 1 of
the `continuation-only` column below), which is why the slice needs both changes.

## The matrix

Every render of the set is stored under `gating/<state>/`, so the table below can be re-read from
the evidence. "identical" means byte-identical to the `baseline/` render of the same file.

| anchor / negative | A alone | B alone | A + B |
| --- | --- | --- | --- |
| `IO.readAll` (3 legs: patrol jar, v8, v8-javac8) | **identical** | **identical** | **presented whole** (0 quotes in the class) |
| `countLines`, `main`, `IOMidRead`, `IONegatives`, `NestedDepth` | identical | identical | identical |
| `LK` (3 legs), `LockGuardNegatives`, `LockGuardProbe` | identical | identical | identical |
| `DS`, `REF`, `NEG.shortChain`, `ExtraCopy`, `WrongType` | identical | identical | identical |
| `LoopTestValues` (the postfix family's `storeTest` negative included) | identical | identical | identical |
| `NEG.liveLine` | identical | **presented** (the registered same-form loop outside a guard body) | presented |
| `cf06.NegativeAssignments.loopCondition` | identical | **presented** | presented |
| this slice's `Probe.readAll` (no guard) | identical (the copy family's own position) | **presented** | presented |
| this slice's `Probe.guardPlain` (the guard body's own control) | refused (the guard is claimed, its body cannot complete: the loop's dance is not admitted yet) | refused (the guard is not claimed at all) | **presented** |
| this slice's `ProbeControls` (both refusals) | refused, the guard body's own quote | refused, the crossing cascade | refused (the refusals this slice freezes) |
| this slice's `MultiCopy` (the patched control) | refused | refused | refused |

The A-only column is byte-identical on **every** pre-existing render — the certificate's claim
changes the region tree (the guard becomes a `Region::Guard`) but the members' texts and the
crossing cascades are the same, so the change is invisible until the copy family admits the loop's
test. Its one visible effect is on this slice's own guard-body control (`guardPlain`), whose quote
moves from the crossing cascade to the guard's own body refusal: the certificate now claims the
statement and stops at the loop inside it. The B-only column flips the loop-test position where the
copy family owns it — `NEG.liveLine`, CF-06's `loopCondition` and the guard-free probe — and
**nothing else**: `IO.readAll` and `guardPlain` still refuse (their guard is not claimed), the
parameter-target form (`LoopTestValues.storeTest`, `ProbeControls.parameterTarget`) keeps its
region-layer refusal, the multi-consumer patch keeps its refusal, and every other
copy/dup-store/postfix/guard anchor is byte-identical.

## What the continuation completion is (commit A)

`prove_resource_guard_finally`'s `Operation::Transfer` arm now proves the statement's own
continuation and stores it in the plan:

* the transfer's block has one successor, reached from nowhere else → `Continuation::Block`, and
  the plan's `join` is that block;
* the transfer's target was **fused** into the transfer's own block (the canonical graph fuses a
  single-successor/single-predecessor chain, which is what `readAll`'s `goto 53` is: the target
  block's only predecessor is the goto, and the transfer's own block carries no exception edge) →
  `Continuation::Tail { span }`, the block's own tail after the transfer, and the plan's `join` is
  `None` (the run ends where the block does).

The claim accepts a fused tail only where every instruction but the last is a straight one and the
last completes the run (`Return`/`Throw`), so the tail is a statement sequence the Java writer can
state; the builder writes it after the `try` statement (`range(span)`), which is where the
bytecode runs it. `Continuation::Block` is the form a cleanup block with an exception edge of its
own would take (the LK family's shape) and needs no builder-side tail: the walk continues at the
join block.

## What the loop-test admission is (commit B)

* `region::test_expression_instruction` admits the `dup; store` pair whose stored value the test
  consumes (`store_dance_part`): the store writes one local slot **the body declares** and takes
  one of the copy's two values, and the test reads the other. A parameter target, a copy whose
  values anything else consumes, and a store no test reads keep the `StatementFree` refusal at
  their own BCI.
* `build::prove_local_assignments` presents a **local** target at a loop's own test as the
  in-place assignment expression (`LocalAssignmentPresentation::Expression`), and the
  `observed && enters_handler` rule now refuses only presentations that **move** the assignment:
  the eliminated form drops the store and the expression writes it where the bytecode ran it, so
  a protected body's own loop test is admitted; the split form (written in front of the structure)
  keeps the rule. A parameter target at a loop test keeps the refusal it has (its split form
  cannot be written in front of a re-evaluated condition, and the expression is the copy family's
  local form).
