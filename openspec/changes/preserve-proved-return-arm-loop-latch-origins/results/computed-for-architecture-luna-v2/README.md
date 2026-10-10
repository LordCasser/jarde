# CF07 computed-init `for` architecture audit

Private, read-only architecture notes for the next computed-initializer slice. No repository files were changed. No CLI, JDK, Cargo, Git, or formatter was run. The current return-origin product and its evidence remain frozen.

## What the source establishes

`Walker::for_header_candidate` (`crates/jarde-java/src/region.rs`) has two independent rejection points relevant to CF07:

1. The preheader must end in a producer decoded as `Operation::Push(ConstantValue::Int(_))` followed by the induction-slot store. This excludes a computed initializer such as `iload; iconst_1; isub; istore`.
2. After constructing the candidate, it scans every SSA block. Any induction-slot read or write outside the natural-loop `blocks` and the preheader rejects the candidate. The check applies even if the outside block is a proved terminal return leaf.

In the header-tested path, `prove_for_header(header, header_node, blocks)` is called before `loop_terminal_returns(blocks, exit_node, frame)`. Therefore widening only the producer gate cannot admit a return expression that reads the induction slot: the SSA access scan still rejects it before the existing return certificate is available.

The frozen CF07 class evidence at `preserve-proved-return-arm-loop-latch-origins/results/cf07-candidate-root-v1/manifest.json` records `lastIndexOf([IIII)I` as `iload_3; iconst_1; isub; istore 4` at BCIs 0–3; the loop header is BCI 5; its direct return leaf has `iload 4` at BCI 19 and `ireturn` at 21; the natural-loop blocks are reported as BCIs 5, 11, and 22. This makes both candidate gates relevant: the initializer is computed, and BCI 19 reads local 4 outside the natural-loop block set. The instruction/raw manifest is frozen evidence; it does not establish that a changed implementation has run or passed.

`loop_terminal_returns` already provides a narrow method-exit certificate: a candidate is outside the loop and normal exit, has no normal-view successors, has exactly the comparison block as its canonical normal predecessor, has no canonical outgoing edges, and ends in a decoded JVM return. It is computed later and then passed to `Frame::loop_body`, which widens the body scope so that the return arm can be represented and counted exactly once. This is a suitable existing fact to reuse for an early, local read exception; it is not a general exemption for arbitrary out-of-loop code.

A conservative next design can compute the existing terminal-return set before `prove_for_header` and pass that set into the candidate scan. Keep rejecting induction-slot writes outside the loop and preheader; allow only reads from certified return-leaf blocks. Do not widen the loop's natural blocks or change the ownership proof. The precise certificate and its frame-scope constraints must remain the same as the later body walk. This reuses existing proof data and does not require storing a new `Region`/IR entity.

## Separate Builder/origin boundary

`build.rs` consumes `ForHeader` in a separate way. `for_init_index` locates the exact `init_bci` statement and permits only empty declarations between it and the loop. Builder marks the proved update instruction as settled while walking the body, then independently renders one update assignment and checks that its local name matches the initializer before emitting `StmtKind::For`. A terminal return `If` remains part of the loop body; the helper does not move its return arm into the header clauses.

There is a separate region-origin limitation in `region.rs::implicit_tail_latch_origin`: it handles a final no-join return/latch `If` only when `for_header.is_none()`. With a `ForHeader`, this region helper currently accepts only a final `Straight` latch and validates that the block is the proved update block with the exact update and transfer BCIs. Thus admitting the computed initializer and certified return read alone would not prove that the region preserves the latch transfer origin for a `for`. This restriction belongs to region-origin production, not Builder consumption; keep it distinct from the Builder checks below.

If the next slice needs both behaviors, the narrow region-layer extension is to let `implicit_tail_latch_origin` recognize a `ForHeader` latch arm only when that arm is exactly `proof.update_block`, ends in `proof.update_bci` plus the unique transfer back to the proved header, and passes the existing canonical edge/latch checks. The opposite arm must be the same exact terminal-return leaf certificate used by the loop body. Keep the current straight-latch path unchanged. Builder remains responsible for consuming the resulting `ForHeader` and rendering its initializer/update clauses; do not infer a `for` from source spelling or from a return-arm shape alone.

## Facts versus work still required

Established by static source and the frozen baseline:

- The producer check and the out-of-loop induction-slot scan are distinct gates.
- The second scan executes as part of `prove_for_header`, before `loop_terminal_returns` is computed in the header-tested path.
- CF07's original initializer is `iload_3; iconst_1; isub; istore 4`; BCI 19 reads local 4 before `ireturn`; BCI 19 is outside the reported natural loop blocks.
- The existing certificate can identify terminal return leaves and later widen body scope.
- Builder has a distinct final-If origin restriction for `ForHeader` loops.

Not established, and must not be reported as successful:

- That `for_header_candidate` reaches either gate for this precise candidate once the producer gate is changed.
- That an earlier `loop_terminal_returns` call produces BCI 19 in the exact candidate context and frame scope.
- That read-only exemptions preserve all ForHeader SSA/phi invariants on this method.
- That the Builder produces a `for` with the return/latch body and preserves the latch-origin source map.
- Any changed CLI, source map, compile, runtime, or dual-JDK result.

## Minimum next real-IR diagnostic

Use the already frozen CF07 class and the existing real-IR unit-test setup in `region.rs` (the test that opens the frozen `LoopCases.class`, analyzes `lastIndexOf([IIII)I`, runs `recover`, and constructs a `Walker`). Do not create a synthetic CFG or infer the candidate from Java source.

Add a temporary, non-product diagnostic around the actual header-tested candidate that records, in order: natural-loop node/BCI set; the current preheader tail and decoded producer; the first `for_header_candidate` rejection reason; the terminal-return set from the same `blocks`, `exit_node`, and `frame`; and the induction-slot read/write sites outside `blocks`/preheader, annotated with whether each node is in that return set. Then exercise the existing candidate with the smallest proposed read-only terminal-return exception and report whether the next refusal moves to another proof gate. If the local API cannot expose a structured reason without changing production code, add only test-local tracing or a focused helper-level test; do not make a “producer-only” patch and declare it sufficient.

For the separate Builder question, after a region-level proof is available, use a focused real-class rendering check to assert the full generated `for` body, exact return expression, init/update clause, source-map span, and latch transfer origin. Preserve baseline-vs-candidate source/map comparison and fresh whole-class validation for any actual product attempt. Keep the current frozen return-origin CLI/results unchanged.
