# 1.1 — the refusal at HEAD, the criterion, and which reading this slice writes

Worktree = `6ca9cfdf` (the filing). Every measurement is the same checkout built twice: the
**baseline** binary with `crates/jarde-java/src/guard.rs` and `region.rs` at their HEAD state, the
**admission** binary with this slice's changes in place. The anchor bytes are the nested-lock
slice's own fixture (in-library), both legs.

## The refusal at HEAD (re-verified, both legs, byte-identical)

`MLProbe.nestedLocksBranching(Z)V` refused whole at HEAD with the text the nested-lock slice
registered (`tests/recover_nested_lock_finally_bodies.rs` pinned it):

```
// @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25
// BCI 55: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
// @bytecode 28 31 32 34 37 38 39 42 45 46 49 52 55 56 57 60 63 64 67 70 71 72
// 3 live block(s) are reachable only through edges the normal-flow view leaves out: [28, 38, 55]
```

Instrumented `guard::prove_lock_guard_finally` (temporary `eprintln!`, removed after measuring; the
transcript is `trace-admission.txt` beside this file, first section):

```
TRACE LG-TRANSFER transfer=52 block=38 successors=[] instructions=[38, 39, 42, 45, 46, 49, 52, 72] row=(14, 38, 55) handler=55
TRACE LG-TRANSFER refuse at successor-less transfer block
```

So the certificate's own structural checks pass — the row, the lead, the two three-instruction
release groups, the reverse pairing, the copies' coverage — and the refusal is one line: the
**void completion's transfer has no successor block to state**.

## The criterion, located

| where | what it states |
| --- | --- |
| `guard.rs::prove_lock_guard_finally`, the `Some(&Operation::Transfer)` arm of the completion match | **the void completion**: `facts.view.successor_ids(normal_block).first()` — the transfer's one successor must be the method's own value-less `return`, a block of its own (`in_block(..).len() == 1`, `Operation::Return`, no stack operands, no successors, `predecessors == [the transfer's block]`). The branching shape has **no** successor here, so the arm returned `Ok(None)` at its first line |
| `guard.rs::prove_resource_guard_finally`, the same arm's `[]` branch (the `recover-loop-test-copy-store` slice) | **the tail-span reading**: where the transfer's target was *fused* into the transfer's own block, the continuation is that block's tail after the transfer — every instruction but the last straight, the last completing the run (`Return`/`Throw`) — stored as `Continuation::Tail { span }` and written after the `try` by the builder |

The two are different questions. The resource guard's reading answers *"which statements does the
run continue with"* (its `Continues` completion: the code after the `finally`, the value the method
answers built there). The lock guard's completion answers *"what does the method complete with"*,
and the certificate proves exactly one value-less `return`.

## Which reading: the **sibling** (and why)

`guard.rs::fused_void_return` — this slice's reading — is the **sibling** of the resource guard's
tail-span reading: the same fusion, read on the same block, with the void completion's own
criterion instead of "the statements the run continues with". The tail must be **exactly** the
method's own value-less `Return` (`[returns]` — one instruction, `Operation::Return`, nothing on the
stack), and the run ends where the block does.

The reason is measured, not stylistic. The certificate's admitted set may not depend on which
**layout** javac happened to leave, and the separate-block layout refuses every completion but the
value-less return. The controls (both legs, `Scratch`/`Scratch2` scratch sources, `gating.out`'s
`/tmp` siblings; the texts are in `trace-admission.txt`):

| the same completion, straight body (release copy in the body's block, return a block of its own) | HEAD | admission |
| --- | --- | --- |
| `void m() { lock; try { count++; } finally { unlock } }` — the value-less return | presents | presents |
| `void m() { …; throw this.stored; }` — the tail is `aload_0; getfield; athrow`: **straight**, ends in a `throw` | refused | refused |
| `void m() { …; throw new IllegalStateException("after"); }` — the tail holds an allocation | refused | refused |
| `void m() { …; count++; }` / `… while (flag) count--;` / `… if (after) count--;` | refused | refused |

With the general tail-span reading (straight instructions + `Return`/`Throw` last) the second row
would **present** in the fused layout while its separate-block control refuses: the same completion
would be admitted or refused according to whether the body branches, which is the layout dependence
this slice exists to remove. The sibling keeps the certificate's criterion and moves only the
reading of *where* the completion sits.

Two more consequences of the same criterion, both measured:

* the `Void` form's own arm is **byte-identical** to HEAD — the fused case is a new branch beside
  it, and the separate case's checks are the same statements in the same order (the diff moves them
  into a `_` arm; `gating.out` shows LK/IO/nested-lock/loop-test-copy/copy all identical);
* a value-carrying or multi-instruction tail stays refused, so the slice cannot admit a
  value-returning method through the void completion (`tailAssign`-shaped probes stay refused).

## The region-reader finding: the body needed a boundary, not a new shape

Admitting the completion alone was **not** enough: with the fused tail read, the certificate claims
the shape and the **body walk** then refused it —

```
TRACE BSFB start=0 span=(14, 38) expected=[0, 28] regions=1 next=None supported=false
TRACE BSFB body Fallback { blocks: [block 0], reason: LoopLeavesEarly { block_bci: 0 } }
```

`region::Walker::bounded_shared_finally_body` walks a protected body with `frame.scope` = the
blocks the body's instructions occupy and `frame.boundary = None`. The branch at BCI 25 has two
successors: the throwing arm (BCI 28, inside the scope) and the fall-through to the release copy
(BCI 38, the range's end — **outside** the scope and no boundary), so the successor filter dropped
that edge and the walk quoted the branch (`FallbackReason::LoopLeavesEarly`).

The body's own end is the walk's boundary — the code after the range is not the body's to claim —
and `frame.boundary` is the existing mechanism for exactly that (`try_level` sets it to the
statement's join, `loop_finally`'s update walk to the exit block). The fix is one statement: the
boundary is the block that begins where the protected range stops (`span.1`), when the canonical
graph starts one there. **No new region shape, no new reader arm, no new frame field**: the branch
itself presents through the walk's existing `if` shapes, exactly as the filing predicted ("the body
by the existing region shapes").

Measured effect of the boundary, corpus-wide (`04-corpus-and-oracle.md`): **10 of 968** renders
move — the branching anchor on both legs and this slice's own fixture; every other class in
`tests/fixtures/**` is byte-identical, and the five families' 37 anchor renders are byte-identical
(`02-gating.md`).

## The measured line, stated

| shape | layout (measured) | before | after |
| --- | --- | --- | --- |
| `MLProbe.nestedLocksBranching` (branching body, two locks) | release copy in a block of its own at the range's end; no exception edge on it; the trailing `return` **fused** (`block 38 = [38, 39, 42, 45, 46, 49, 52, 72]`, `successors=[]`) | refused (BCI 55) | **presents** |
| `MLOrder.nestedLocksThrowing` (straight body) | release copy in the protected call's block, which carries the range's exception edge; the return stays its own block (`successors=[58]`) | presents | presents |
| `LK.put`/`take` (loop body) | the release block still holds protected instructions, so it carries the row's exception edge; the return stays its own block (`transfer=53 block=27 successors=[66] preds=[[27]]`) | presents | presents |
