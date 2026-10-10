# Conditional switch fallthrough: CF12 real-IR audit v3

This is a private, read-only audit that records the completed real-IR diagnostic and tightens the v2 proposal with observed facts. It does not change repository files or claim that the proposed product change was implemented or accepted. The diagnostic raw files remain at `/private/tmp/jarde-cf12-real-ir-diagnostics-root-v2/`.

## Diagnostic provenance and limits

`execution.json` records two focused `cargo test` commands with exit code 0 and all 52 source pins restored. `1.stdout.raw` records `region::tests::diagnose_cf12_switch_fallthrough_from_original_class_ir` as 1 passed, 0 failed, 0 ignored. The complete graph and recovery dump are in `1.stderr.raw`. These facts establish that the temporary diagnostic ran on the frozen original class and emitted the observations below. They do not establish a product implementation, source rendering, runtime parity, or rejection behavior for edge kinds absent from this fixture.

The diagnostic input is the complete original 1563-byte `TestSwitchWithFallThroughCase$TestCls.class`; the requested method is `test(IZZ)Ljava/lang/String;`. It was not reduced to the target method or altered. The test reports all canonical node paths as `[]`, `canonical_unreachable=[]`, and a NormalFlow projection with 17 kept edges, 0 canonical Return edges, and 0 excluded Exception/Call edges.

## Observed canonical graph

Decoded switch at BCI 7 has cases `1 -> 32`, `2 -> 117`, `3 -> 146`, with default `149`. The common join is BCI 171. All 17 canonical edge rows are `Normal`:

| From | To | Role from the decoded switch / operations |
|---:|---:|---|
| 0 | 32 | dispatch to case 1 |
| 0 | 117 | dispatch to case 2 |
| 0 | 146 | dispatch to case 3 |
| 0 | 149 | dispatch to default |
| 32 | 59 | case 1 conditional continuation |
| 32 | 117 | case 1 route to case 2 |
| 59 | 63 | case 1 conditional continuation |
| 59 | 117 | case 1 route to case 2 |
| 63 | 67 | one branch to the common join |
| 63 | 92 | other branch to the common join |
| 67 | 171 | common join |
| 92 | 171 | common join |
| 117 | 121 | case 2 body |
| 117 | 171 | case 2 route to the common join |
| 121 | 171 | common join |
| 146 | 171 | case 3 route to the common join |
| 149 | 171 | default route to the common join |

The exact incoming rows that matter for ownership are:

- Entry 32: only dispatch predecessor 0.
- Interior 59: only predecessor 32.
- Interior 63: only predecessor 59.
- Interior 67 and 92: only predecessor 63.
- Next-case entry 117: predecessors 0 (dispatch), 32, and 59. It must be treated as a boundary for case 1, not owned by that arm.
- Case 2 interior 121: only predecessor 117.
- Join 171: predecessors 67, 92, 117, 121, 146, and 149. It stays outside each case body.

There are no other canonical incoming or outgoing rows in this diagnostic. No case body ends in a source-level return or throw. The physical `Return` operation is at BCI 195, after the common join block beginning at 171; it is not a per-case terminal outcome. Therefore the actual positive path needs only two classified outcomes: next case entry 117 and proved join 171. The terminal-return/throw support in the general proposal remains unexercised here.

## What current recovery reports

`recovered_regions` currently contains:

- a `Fallback` for block 117 with `Loop { block_bci: 117 }`;
- a `Fallback` with `SwitchArmsOverlap { block_bci: 0 }` claiming blocks 0, 32, 59, 63, 67, 92, and 121;
- an `UncoveredBlocks` fallback for 146, 149, and 171.

This is the observed output, not proof of the exact internal cause of every fallback. It is consistent with the missing case-to-case certificate allowing overlapping arm walks. The audit does not infer a source-level loop: the printed canonical graph contains no cycle, and the existing `Loop` fallback label must not be read as evidence of a real loop.

The original frozen JADX source confirms the intended case flow: case 1 has conditional work, case 2 follows at the next label for some routes, and the remaining routes break to the shared continuation. Case 2 and case 3/default reach the common continuation without falling through to another case. Source text is corroboration; the proof target remains the observed bytecode graph and ownership rows above.

## Minimal proof change indicated by this fixture

Keep the proposed change local to the existing `switch_fallthroughs` certificate and existing `Frame::switch_arm` / renderer contract. On this graph, the certificate should classify:

- source entry 32: outcomes `{117, 171}`; record exactly `32 -> 117` because 117 is the unique immediately following grouped case entry, while 171 is the already-proved join;
- source entry 117: outcomes `{171}`; complete no-fallthrough result;
- source entries 146 and 149: outcomes `{171}`; complete no-fallthrough result.

The probe for 32 must stop at the 117 boundary. Its interior ownership check is exact: 59 is entered only from 32, 63 only from 59, and 67/92 only from 63. The 117 boundary may have dispatch predecessor 0 and arm predecessors 32/59; it is not consumed as case-1 body. Likewise, join 171 is a terminal boundary, not part of any case body. With the existing switch frame boundary behavior, 117 should then be owned exactly once by the case-2 group, while the earlier arm can express the conditional route to that group and the conditional routes to the join.

The actual fixture confirms the v2 classifier's bounded-DAG direction and target ordering. It does not validate all v2 guard cases. Preserve exact full-canonical-edge checks and incoming ownership, because the fixture has no Exception, Call, canonical Return, cycle, external interior predecessor, or clone-path edge to exercise those refusals. Retain separate focused adversarial tests for each. Do not add a new region type, flag, general graph framework, or case-body copy.

## Small validation matrix update

For the positive frozen class, assert the exact switch targets, complete 17-row edge multiset, all-normal edge kinds, paths `[]`, incoming sets above, and case outcomes `{117,171}`, `{171}`, `{171}`, `{171}`. Then assert that recovered regions own 117 once under the case-2 arm, keep 171 outside all case bodies, and contain no Loop region for this switch. Independently retain refusal tests for hidden Exception/Call/canonical Return edges, source-level Return/Throw leaves, an external predecessor, cycles, multiple/later case targets, and path changes; none of those are proven by the present diagnostic.

After implementation, acceptance still requires the unchanged complete class to compile and run against the oracle, raw runtime/source equality, default/all equality, and exact source-map/physical ownership. This diagnostic alone is not acceptance.
