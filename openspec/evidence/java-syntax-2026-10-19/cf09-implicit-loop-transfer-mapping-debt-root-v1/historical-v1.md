# ComplexIf implicit tail-goto mapping note

Scope: read-only analysis of the already-recorded `TestBreakInComplexIf.test(Map,int)` BCI 69 source-map gap. This is an architecture-debt note only; it does not propose a product/test/fixture change in the current leaf and does not claim a probe or new implementation was run.

## Frozen evidence

- Candidate execution receipt: `/private/tmp/jarde-testbreakincomplexif-final-candidate-root-v1-20261019/execution.json`, SHA-256 `f3e5138308a8362860b2c3acf1f584d867b185a837c1a9f44efff91ab20259c9`.
- Candidate API cell: `jarde/debug/child/all/target/report.json`, SHA-256 `8905119da49aed069dec7d217f7340ddf66f379756ba897e35b8dcddf6986dbe`; whole class output `whole-source.java`, SHA-256 `d38616752273ec973680f76e9d253ea723c160f40227dafc51d22f8716cceefe`.
- Original target class: `/private/tmp/jarde-testbreakincomplexif-baseline-root-v2-20261019/debug/classes/jadx/tests/integration/loops/TestBreakInComplexIf$TestCls.class`, SHA-256 `bb77f87c9a621de2deb8c439e9270a76813445aa77564e547001fb3237ab3a0f`.
- Live source pins used for static inspection: `crates/jarde-java/src/region.rs` SHA-256 `5da4d9f40e0892ac134bb42fb1508fffad090b92338a86b9403efd47b7977510`; `crates/jarde-java/src/emit.rs` SHA-256 `80169b0bc68693734d10589831f20cd12695045eb7348fc659a55e25f9fad1d0`; `crates/jarde-java/src/source_map.rs` SHA-256 `f5803ffa204b2af9621fc057ee628046bae1c61c83881f8682950e0d7fe73d8b`.

## What the evidence says

The original physical `test(Ljava/util/Map;I)I` has a 74-byte Code array. BCI 69 is a two-byte `goto` whose signed branch offset is -62, so its destination is BCI 7 (`69 - 62`). BCI 72 loads the result local and BCI 73 returns it. The candidate map audit reports exactly BCI 69 missing for this method; the other physical BCIs are mapped. This is an implicit loop-tail transfer: the Java source has no corresponding explicit statement at that position.

The recovered whole-class source still contains the enclosing loop and its explicit body statements. In the selected candidate report the test method is `structured`, `java`, `contains_statements`, with 3 regions, 13 statements, and 41 source-map segments. BCI 63 and 66 (the `length` and `x` increments in the loop body) have direct segments; BCI 72/73 have return segments. BCI 69 has no source-map segment/origin. This is therefore a narrow provenance-coverage gap, not a missing loop reconstruction or a missing source statement.

The emitter architecture explains the shape of the gap. Atlas inspection of `emit.rs` found `emit_source_map` replaying the already-built AST against the artifact text, and `Emitter::stmts` wrapping each AST statement in `node(&stmt.origin, ...)`. `node` records a segment only for the emitted byte span and uses that node's existing `OriginSet`. `stmt_with_indent` sets its write anchor from `stmt.origin.primary().bci()` and emits a `While` from its label/condition/body/closing brace; there is no separate emitted AST node for a JVM back-edge instruction. `put` in replay mode checks exact text agreement; normal emission charges `OutputBytes`, and `node` checks phase continuation before publishing the segment. Thus the map does not infer every control-flow edge from Code; it reports provenance carried by the nodes whose text is emitted.

Region Atlas metadata lists `Walker::implicit_tail_latch_origin` in `region.rs` (line 13686), with callers `header_tested_loop` (13847) and `loop_arm_join_source` (5908). That establishes an existing loop/latch-origin path, but not that this particular BCI 69 is captured or forwarded into the output AST. Atlas's source excerpt for those `region.rs` symbols was inconsistent: the reported signature/range was paired with source belonging to another routine. I therefore treat the symbol names, locations, and call-graph edges as evidence only; I do not claim to have confirmed the helper's exact implementation or this site's candidate value from that excerpt.

## Assessment and bounded direction

**Verified:** BCI 69 is an actual back-edge to loop header BCI 7; it is absent from the candidate map; existing mapped spans do carry derived origins (for example BCI 11 is derived on a loop-header-owned span); and source-map publication is AST replay rather than a sweep over physical Code.

**Not yet verified:** whether `implicit_tail_latch_origin` returns BCI 69 for this concrete loop, whether that value survives Region-to-AST construction, and whether the omission is an explicit contract choice. No new probe was run here. The current evidence cannot distinguish “deliberately unmapped because an edge emits no text” from “the proven latch origin was lost before replay.”

The smallest follow-up, if complete physical-BCI mapping is a required contract, is to trace this one latch from canonical edge/Region proof through the AST `OriginSet` into the replayed loop node. Reuse the existing proof and existing origin representation; do not add a new CFG recognizer or globally mark a loop body with every edge BCI. A narrowly assigned derived origin on the loop-closing span is a possible design only if the existing proof identifies that exact back-edge, owner, and loop unambiguously. If the contract is statement-only, keep BCI 69 unmapped and state that explicitly instead of pretending the Code map is exhaustive.

Any implementation must keep primary statement origins unchanged, preserve exact text replay and physical method identity, add derived provenance only after the same exact loop/latch proof succeeds, and make the new origin charge/publish through the existing replay path. A cancellation or budget stop must still return only the charged prefix; a map must not be post-hoc completed after a stop or failed text gate. Refusal or ambiguous/exceptional/subroutine edges must retain the current conservative behavior. This note does not authorize those changes in the active leaf.

## Atlas source references

- `crates/jarde-java/src/region.rs`: `Walker::implicit_tail_latch_origin` signature/range at 13686; call graph reports `header_tested_loop`, `loop_arm_join_source`, and one real-IR test as callers.
- `crates/jarde-java/src/emit.rs`: `emit_source_map` (line 323), `Emitter::stmts` (652), `Emitter::node` (606), `Emitter::stmt_with_indent` (775), `Emitter::put` (1579).
- `crates/jarde-java/src/source_map.rs`: `Segment { start, end, origin }` (258) and `SourceMap { segments }` (319).

The Atlas source extraction caveat above is material: line/symbol references should be revalidated before using the `region.rs` helper as an implementation contract.
