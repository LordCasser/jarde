# Conditional-switch private patch: adversarial static audit

Status: source-only review of `/private/tmp/jarde-proved-conditional-switch-fallthrough-implementation-v2/implementation.patch`. No patch was applied and no parser, compiler, formatter, test, Git, or other toolchain was run. The findings below are risks to verify, not claims about runtime behavior.

## Scope and real fixture

The OpenSpec contract is in `openspec/changes/recover-proved-conditional-switch-fallthrough/design.md` and `specs/java8-recovery/spec.md`. It requires a finite complete-path proof, canonical edge/path identity and ownership, one adjacent fallthrough destination, and a path-specific switch exit; it also requires full rejection/cancellation tests and actual Builder scope validation.

The accepted real-IR audit reports the complete `test(IZZ)Ljava/lang/String;` method with 11 canonical blocks and 17 Normal edges: dispatch BCI 7 to entries 32/117/146/149, join 171; case 32 reaches both 117 and 171, while 117/146/149 reach 171. All node paths are empty. It did not exercise exceptional edges, clone paths, external predecessors, cycles, or return/throw exits (`results/real-ir-audit-luna-v3/facts-and-debt.md`). The patch's only added permanent test is the positive real-class test; `tests-draft.md` lists negative and Stop tests but they are not in the patch.

## Findings

### 1. Join-break handling is gated on having a proved adjacent fallthrough target

Patch `implementation.patch:130-132` only creates a transfer `SwitchBreak` when `frame.switch_fallthrough_target.is_some()`. The direct conditional-join path repeats that gate at `:164-190`, and the `ArmsDoNotMeet` exception repeats it at `:208-218`. `switch_arm` sets that option from `fallthroughs.get(&target)` (`:232-243` and `:253-280`), so a certified `Some(empty)` map yields no target for any arm.

A concrete counter-shape is an ordinary switch case with no edge to a later case, containing `if (p) goto switch_join;` followed by a side-effecting block reachable only on the other branch. The case's ordinary tail `break` cannot represent the early branch: emitting the join arm as empty lets generated Java continue into the next case before reaching the side effect, or emitting the break after the whole `if` changes the side-effect path. The stated requirement “条件路径都结束在公共出口” also expects branched no-fallthrough cases to remain structurally representable.

Minimal direction: make the switch-join transfer certificate available for every arm with a proved switch join, independent of whether the fallthrough map contains an adjacent target. Keep the adjacent-target map solely for case ordering/stop boundaries. Add a no-fallthrough two-path fixture with an observable sibling-side-effect path and assert its exact source/runtime behavior.

### 2. The SwitchBreak context guard can downgrade a tree to fallback after the Region has already claimed its source block

Transfer leaves are appended after `Straight { blocks: prefix }` at `:149-154`; for a conditional join, the `If` already owns the branch block and one arm contains the leaf at `:167-203`. Builder then validates the active owner at `:964-980`, but on mismatch calls `fallback(vec![source_bci], ...)` rather than rejecting the enclosing switch/Region before emission.

A concrete malformed/cross-scope Region shape is a `SwitchBreak` in a parent arm whose source block is already represented by its parent `Straight` or `If`, while an inner switch or loop is active at visit time. `fallback` may claim the same physical source block a second time or publish a partial outer structure with a local quoted leaf. This is precisely the nested-switch / parent-break-through-inner-loop distinction in the OpenSpec; static code does not show that `fallback` rolls back the enclosing rendered statements or block-owner set.

Minimal direction: reject the enclosing switch/arm transaction before visiting/emitting any partial arm if any contained leaf has the wrong nearest owner; alternatively prove and test that the surrounding checkpoint rolls back every statement, source origin, and physical owner before fallback. The test should assert unique block ownership and no partially published Switch/If when the mismatch is encountered.

### 3. The physical branch BCI is reused as the break leaf's source origin

`switch_break_branch` writes `source_bci: branch_bci` (`:406-409`), and Builder emits a direct origin for that same BCI (`:982-985`). The parent `Region::If` also necessarily reports the conditional branch instruction. This can legitimately map one instruction to a line containing `if (...) break;`, but the patch/test draft does not state whether that BCI is expected to have one or two source-map segments/owners. The transfer version avoids this exact overlap because its physical transfer is represented in the preceding `Straight` block and the leaf owns no block.

Concrete check: for real conditional branch@89 and branch@114, inspect the complete source-map segment list and region ownership. Ensure each physical block remains owned once, both `break;` statements have the correct branch source, and no global “each BCI exactly once” invariant is accidentally violated. If the renderer cannot distinguish branch-condition origin from taken-edge origin, keep the same BCI but explicitly test/document that relationship; do not invent a synthetic edge BCI.

Minimal direction: extend the real-class test to assert the exact origin/segment relationship for branch@89/@114 and one-owner physical block inventory, rather than only counting two textual breaks and checking each BCI appears in some break segment (`after/p3_conditional_switch_fallthrough.rs:125-156`).

### 4. The complete-edge certificate is carefully charged, but the budget accounting needs a targeted adversarial boundary test

The proof scans and records all canonical edges once (`:454-470`), then traverses each case DAG with a work stack, validating path identity and canonical/view outgoing multisets (`:505-649`). It separately bills successor pushes (`:628-635`), the owner-map pass (`:676-692`), case-entry incoming rows (`:693-735`), and every owned-node incoming row (`:737-789`). Duplicate parallel rows are preserved in `Vec`s and rejected against the deduplicated view (`:448-450`, `:580-584`, `:708-714`, `:756-762`). This is a sound direction for the 11-block/17-edge fixture and does not simply trust `NormalFlowView`.

The adversarial concern is that `outgoing.get(...).cloned()` / `incoming.get(...).cloned()` allocate/copy the full incidence row before the later `charge(rows.len())` at `:698-704` and `:746-752`; the second `switch_break_*` helpers also rescan all canonical edges per leaf (`:322-333`, `:387-401`). The initial full-edge charge bounds these copies by already-accounted input edge count, so this is not an unbounded traversal, but Stop can arrive only after an incidence clone was allocated and helper scans multiply work. The current tests draft asks for budgets/cancellation but does not pin these actual call sites or guarantee no partial map/Region/report publication.

Minimal direction: add tight-budget tests stopped during (a) initial canonical indexing, (b) a high-degree incident-row inspection, (c) each switch-break recheck, and (d) owner/incoming validation. Assert the returned `StopReason` dimension/BCI and that no partial map or arm is published. If the copy-before-charge shape is unacceptable under the reader's resource contract, charge the row length before cloning (or iterate the borrowed slice after charging).

### 5. Scope-stack checkpointing is present, but push/pop balance and actual masking are untested

`FrameBuilder` adds both `switch_depth` and `switch_break_targets` to checkpoint state and restores them (`:870-898`). Both Switch and SwitchExpr render paths push a target containing branch/join/loop/switch depths, then pop after iterating arms (`:912-950`). `Region::SwitchBreak` only emits an unlabelled break when the last target matches and neither loop nor switch depth changed (`:964-985`). This matches the intended nearest unlabelled-break rule in the normal path.

Still unproved: whether all early return/error paths between push and pop are impossible (the shown arm loop appears non-fallible, but surrounding helper calls and output publication need checking); whether a checkpoint can restore a cloned target stack while another nested switch mutates it; and whether the `fallback` mismatch path described above leaves the enclosing frame usable. Existing `after` test only asserts two break strings, case2 once, and no loop report; it does not cover nested switch, parent switch across inner switch, or parent switch across inner loop.

Minimal direction: add three Builder-level source-tree tests: nearest switch consumes; parent target under nested switch refuses; parent target under inner loop refuses. For the two refusals, assert exact refusal classification, unchanged checkpoint depth/target stack, and no partial emitted switch. Also assert stack length/target identity after a failed nested arm before a sibling arm is visited.

### 6. Real-path integration is intentionally narrow; ordinary/one-arm gate needs two contrasts

The transfer gate at `:129-155` validates a terminal transfer through `switch_break_transfer`, and the branch gate at `:164-190` validates the direct branch edge with `switch_break_branch`. Both helpers require the selected switch join and same canonical path; transfer requires exactly one full canonical Normal edge to the join, while branch requires exactly two full canonical Normal edges including the join (`:291-410`). The normal conditional acceptance gate adds `then_switch_breaks`/`else_switch_breaks` only for a trailing matching `SwitchBreak` (`:204-228`), which is appropriately narrower than treating an arbitrary empty `Straight` as a break.

A remaining control question is whether `Frame::switch_arm`'s `fallthrough_target` remains the correct arm-local certificate while nested frames are created. The patch copies it in `arm`, `nested_loop`, and `scoped` (`:67-125`) and only replaces it on a frame with a join (`:102-117`). This propagates the parent break context through nested constructs; Builder depth checks then reject a leaf crossing an inner construct. It appears conservative, but no test proves the ordinary If gate still rejects an unproved empty arm, while accepting the exact CF12 mixed arm and keeping the straight shared-label cases unchanged.

Minimal direction: preserve three contrasts in permanent tests: exact CF12 mixed branch accepted; branch with an unrelated empty arm/no canonical direct edge to join rejected; all paths to join with no adjacent case remains an ordinary no-fallthrough If. Include straight/shared-label and real loop/switch regression runs from the design.

## Static conclusion

The core path certificate is substantially stronger than the earlier map-only approach: it retains complete canonical edge rows, path identity, duplicate edges, per-arm ownership and earlier-arm proof for incoming case entries. The most concrete correctness gap is that break generation depends on a nonempty adjacent fallthrough map, although a proved conditional switch exit can matter even when no adjacent case is reached. The Builder's mismatch-to-local-fallback behavior and the missing negative/budget/scope tests also need resolution before this patch can be treated as acceptable. This is a static audit only; it does not establish compilation or implementation behavior.
