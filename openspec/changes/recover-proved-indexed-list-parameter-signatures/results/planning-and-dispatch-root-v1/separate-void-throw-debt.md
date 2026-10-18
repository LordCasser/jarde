# Read-only analysis: explicit throw arm in a generic list loop

This memo records the actual v1 controls baseline and a bounded design reading. It does not identify the internal first refusing function: the captured API used default evidence (`region_details` was not requested), so the report establishes the uncovered-region boundary, not the full private Region/CFG probe.

## Observed baseline

The complete source control `NestedLoops2GenericControls.voidGetCastString(List<String>)` is lines 12–16 of the pinned source below. The pinned JADX full-source output preserves the same null-check/throw behavior (lines 13–19), merely spelling the local as `String` without the explicit cast and formatting the throw arm as a block.

For the debug child/default API cell in the v1 receipt, the target is the exact standalone class snapshot (3067 bytes, digest `e61d2805b3839a76fd78cc05ff6e8f9d5830177626bc8923b93a0119cf2a1fde`). The method outcome is `recovered` but `quality=fallback`, `content=explanation_only`, with the sole fallback `jre_region_uncovered_blocks`. Its diagnostic is “1 live block(s) are reachable only through edges the normal-flow view leaves out: [27]”. Recovery reports 4 regions, 1 statement, 1 segment over 6 canonical blocks. Default evidence omitted region details, so those counts must not be mistaken for a complete ownership certificate.

The whole-method quote annotates bytecodes `0 2 12 27 37 43`: the quoted block at BCI 27 ends in a control-flow exit at BCI 36, is reachable from presented structure, and dropping it would still compile while changing behavior. The artifact then uses `jarde_refused_body()` and states that generic Signature projection is unsupported because there is no complete source proof. This is a safety-preserving fallback; the diagnostic is not evidence that BCI 27 is physically unreachable.

Physical Code facts from the same class: `voidGetCastString(Ljava/util/List;)V`, Code length 44, no exception-table rows. BCI 24 is `ifnonnull` with one successor at 37 and null fallthrough at 27. BCI 27 starts `new IllegalStateException`; BCI 30 dup, 31 message constant, 33 constructor call, 36 `athrow`. BCI 37 increments the loop index and BCI 40 jumps to header 2; BCI 43 returns. Thus the throw is a direct terminal arm of an ordinary conditional inside the loop, not a handler/finally completion.

## Relation to existing throw/finally work

The already-supported low-level pieces appear reusable: `build.rs` maps a proved `Operation::Throw` whose single stack read renders successfully into `StmtKind::Throw` (around 26935), and `quoted_control_flow_exit` already treats a quoted terminal `Operation::Throw` as behavior-changing when a normal edge enters it (`region.rs` around 2820–2842). The baseline also reports that the one construction candidate is presented under `new@1`; the visible gap is ownership/structured presentation of the arm, not a demonstrated missing exception-construction rule.

The earlier `finThrow` and `nestedFin` leaves solve different proofs. `finThrow` has an exact catch-all protected interval and cleanup-copy/save-return relationship. `nestedFin` proves nested cleanup copies, exact handler rows and exception/return SSA chains. Neither should authorize this class: its method has no exception handlers and the null arm exits the method directly. `prove_terminal_throw_subgraph` in `region.rs` is scoped to an exact declared handler row and loop-catch shape; it checks throw leaves stay in that protected interval. Generalizing it to this unhandled throw would erase the key boundary rather than reuse the intended proof.

## Smallest future leaf and adversarial checks

First capture same-invocation IR/Region details for this exact method to locate where BCI 27 stops being owned. If the existing ordinary `If` walk can already own a terminal throw arm, fix only the loop/if ownership handoff; otherwise allow the narrow `If` shape with one arm ending in the existing `Throw` statement and the other reaching the loop continuation. Keep the natural-loop header/latch/exit and exact SSA values for `value`, its null comparison, the constructed exception, and the throw operand. Do not add a catch/finally certificate, new public AST/Region variant, or generic exceptional-flow solver absent evidence that the existing statement builder cannot express this shape.

Keep the initial scope to a single terminal explicit throw arm inside one loop body. Counterexamples should include: an unrelated incoming edge into the throw construction, shared throw tail, throw arm leaving a nested loop/switch, a method-call exception (implicit exceptional edge, not explicit `athrow`), finally/catch-table wrapping, both branches terminal, a quoted arm whose omission changes behavior, and a value/constructor proof failure. These distinguish safe lexical ownership from merely seeing `athrow`; they should continue to fall back unless independently proved. `quoted_control_flow_exit` should remain a safeguard, not be repurposed as an ownership proof.

## Inspection provenance and pins

Atlas was opened on the repository and used for scoped searches/exploration of `crates/jarde-java/src/region.rs`; it identifies `prove_terminal_throw_subgraph` as called by the loop-catch proof, not as a general throw proof. OpenSpec documents were read directly because they are not indexed Atlas source. No shared file was written and no JDK, CLI, Cargo, JADX, Git mutation, or network command was run.

- `/private/tmp/jarde-testnestedloops2-controls-baseline-root-v1-20261019/execution.json` — 11029462 bytes, SHA-256 `af878134ce2644435d4abfab675670e81090557df27f58a33cc93285d3deabe6`
- `/private/tmp/jarde-testnestedloops2-controls-prep-luna-v1-20261019/src/jadx/tests/integration/loops/NestedLoops2GenericControls.java` — 2325 bytes, SHA-256 `b39f610887e560eb8c7631727e9ea2cf2510518e182f608f9b1caab9a54a42b1`
- `/private/tmp/jarde-testnestedloops2-controls-baseline-root-v1-20261019/jadx/debug/default/sources/jadx/tests/integration/loops/NestedLoops2GenericControls.java` — 2454 bytes, SHA-256 `e43a9ef2774cd7b51722a8dae930e898fe05e8c1ad6f4e0e10f936b7296c78df`
- `/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/region.rs` — 1172294 bytes, SHA-256 `9e27c9bee263a9fa2d09c1860e6cfcb45605b16b92e524c92009d156f18b59ea`
- `/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs` — 2024019 bytes, SHA-256 `d22ee8f1c40d0a98a174e759904a6a97313b85728bb3056d4aeba18ca8704df9`
- `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-branch-throw-return-finally/design.md` — 6964 bytes, SHA-256 `19873277e7ce6d9e5eae7cbe11d398bc656e585de51c19b82c1bc2a81c1b25c0`
- `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-return-through-nested-finally-cleanup/design.md` — 9907 bytes, SHA-256 `a825c1511c456964ce452ec32e22dce67218e05f6c2abd1a1908e70067c71076`
