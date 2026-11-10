//! A4.1 pins: the per-candidate [`RegionSummary`] reconciles with the canonical CFG/SSA and the
//! walk's own accounts on the shapes the recovery really produces — a loop with proved breaks
//! (`Loop`/`ForHeader`), a do-while whose body owns its header, a guard statement's boundary, and
//! a `try`/`catch` whose clauses bind in exception-table order.

use std::collections::BTreeSet;

use super::finite_loop_catch_tests::with_frozen_method_ir;
use super::summary::{
    RegionSummary, RegionSummaryMismatch, RegionTerminalKind, RegionTransferKind,
    reconcile_method_summaries,
};
use super::*;

use jarde_reader::budget::{Budget, Limits};

const LOOP4_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-break-branch-bodies/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop4$TestCls.class"
);
const LOOP4_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-break-branch-bodies/original/no-debug/classes/jadx/tests/integration/loops/TestBreakInLoop4$TestCls.class"
);
const DO2_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-post-test-loop-return-leaves/original/debug/classes/jadx/tests/integration/loops/TestDoWhileBreak2$TestCls.class"
);
const DO2_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-post-test-loop-return-leaves/original/no-debug/classes/jadx/tests/integration/loops/TestDoWhileBreak2$TestCls.class"
);
const RESOURCE_GUARD: &[u8] = include_bytes!(
    "../../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/near/P5TwoResources.class"
);
const RETRY_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-branching-named-catch-retry-loops/original/debug/TestBreakInLoop2$TestCls.class"
);
const RETRY_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-branching-named-catch-retry-loops/original/no-debug/TestBreakInLoop2$TestCls.class"
);
const SN: &[u8] =
    include_bytes!("../../../../tests/fixtures/p3-switch-nonlocal-exits/javac8/SN.class");

fn ample() -> Budget {
    Budget::new(Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 100,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 32,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    })
}

/// The region of the tree whose variant matches, by a caller-provided predicate.
fn find_region<'a>(region: &'a Region, keep: &impl Fn(&Region) -> bool) -> Option<&'a Region> {
    if keep(region) {
        return Some(region);
    }
    match region {
        Region::Sequence { regions } | Region::Loop { body: regions, .. } => regions
            .iter()
            .find_map(|child| find_region(child, keep)),
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => find_region(then_arm, keep).or_else(|| find_region(else_arm, keep)),
        Region::ShortCircuitValue { tail, .. } => {
            tail.iter().find_map(|child| find_region(child, keep))
        }
        Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => groups
            .iter()
            .find_map(|group| find_region(&group.arm, keep)),
        Region::Try { body, catches, .. } => find_region(body, keep).or_else(|| {
            catches
                .iter()
                .find_map(|clause| find_region(clause.body(), keep))
        }),
        Region::Guard {
            body, finally_body, ..
        } => body
            .as_deref()
            .and_then(|body| find_region(body, keep))
            .or_else(|| {
                finally_body
                    .as_deref()
                    .and_then(|cleanup| find_region(cleanup, keep))
            }),
        Region::Straight { .. }
        | Region::TwoExitReturn { .. }
        | Region::SharedTailEarlyReturn { .. }
        | Region::EntrySplitLoop { .. }
        | Region::LoopBreak { .. }
        | Region::LoopContinue { .. }
        | Region::SwitchBreak { .. }
        | Region::Fallback { .. } => None,
    }
}

#[test]
fn summary_reconciles_the_named_catch_boundaries_and_retry_transfers() {
    for bytes in [RETRY_DEBUG, RETRY_NO_DEBUG] {
        with_frozen_method_ir(bytes, b"test", b"(Ljava/util/List;)V", |canonical, view, ssa, operations, code, recovered| {
            let _ = (view, code);
            assert!(recovered.fallbacks().is_empty());
            let mut budget = ample();
            let mismatches = reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                .expect("the reconciliation completes within an ample budget");
            assert!(
                mismatches.is_empty(),
                "the derived view matches the graph and the walk: {mismatches:?}"
            );

            let tree = Region::Sequence {
                regions: recovered.regions.clone(),
            };
            let try_region = find_region(&tree, &|region| matches!(region, Region::Try { .. }))
                .expect("the retry loop's body is the recovered try/catch");
            let summary = RegionSummary::derive(try_region, ssa, operations, &mut budget)
                .expect("the try candidate derives within an ample budget");
            // The clause, in exception-table order: one named catch whose handler entry and
            // parameter the table states.
            assert_eq!(summary.exceptions.len(), 1);
            let boundary = &summary.exceptions[0];
            assert_eq!(boundary.handler.bci(), 9);
            assert_eq!(boundary.parameter, 2);
            assert_eq!(
                boundary.types,
                crate::guard::CatchTypes::Named(vec![13])
            );
            // The protected range is the candidate's own ownership: the row's exception edges
            // leave blocks the try owns.
            let row = canonical
                .handler_rows()
                .iter()
                .find(|row| row.handler() == Some(&boundary.handler))
                .expect("the canonical row binds the clause handler");
            assert!(
                row.protected()
                    .iter()
                    .all(|block| summary.owned.contains(block)),
                "the row protects only owned blocks"
            );
            // The subtree's transfers: the try's normal exit is one break, the catch's else arm
            // continues the retry loop.
            let breaks: BTreeSet<u32> = summary
                .transfers
                .iter()
                .filter(|transfer| matches!(transfer.kind, RegionTransferKind::Break { .. }))
                .map(|transfer| transfer.source_bci)
                .collect();
            assert_eq!(breaks, BTreeSet::from([6]));
            let continues: BTreeSet<u32> = summary
                .transfers
                .iter()
                .filter(|transfer| matches!(transfer.kind, RegionTransferKind::Continue { .. }))
                .map(|transfer| transfer.source_bci)
                .collect();
            assert_eq!(continues, BTreeSet::from([31]));
        });
    }
}

#[test]
fn summary_reconciles_the_switch_arms_boundary() {
    with_frozen_method_ir(SN, b"retInSwitchNoTail", b"([I)I", |canonical, view, ssa, operations, code, recovered| {
        let _ = (view, code);
        assert!(recovered.fallbacks().is_empty());
        let mut budget = ample();
        let mismatches = reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
            .expect("the reconciliation completes within an ample budget");
        assert!(
            mismatches.is_empty(),
            "the derived view matches the graph and the walk: {mismatches:?}"
        );

        let tree = Region::Sequence {
            regions: recovered.regions.clone(),
        };
        let switch_region = find_region(&tree, &|region| matches!(region, Region::Switch { .. }))
            .expect("the loop body recovers the dispatch as a Switch");
        let Region::Switch {
            branch, groups, ..
        } = switch_region
        else {
            unreachable!("found by the match above")
        };
        let summary = RegionSummary::derive(switch_region, ssa, operations, &mut budget)
            .expect("the switch candidate derives within an ample budget");
        // The dispatch is the statement's entry; both arms leave it without a shared join.
        assert_eq!(summary.entry.as_ref().map(CanonicalBlockId::bci), Some(branch.bci()));
        assert_eq!(branch.bci(), 16);
        assert_eq!(summary.exits, Vec::new());
        assert_eq!(groups.len(), 2);
        // The arm's nonlocal completion is the loop's continue, collected with its destination.
        let continues: BTreeSet<u32> = summary
            .transfers
            .iter()
            .filter(|transfer| matches!(transfer.kind, RegionTransferKind::Continue { .. }))
            .map(|transfer| transfer.source_bci)
            .collect();
        assert_eq!(continues, BTreeSet::from([57]));
        // The loop's own boundary is the one stated exit of the method's structure.
        let loop_region = find_region(&tree, &|region| matches!(region, Region::Loop { .. }))
            .expect("the dispatch lies in one recovered loop");
        let loop_summary = RegionSummary::derive(loop_region, ssa, operations, &mut budget)
            .expect("the loop candidate derives within an ample budget");
        assert_eq!(
            loop_summary
                .exits
                .iter()
                .map(CanonicalBlockId::bci)
                .collect::<Vec<_>>(),
            vec![60]
        );
    });
}

#[test]
fn summary_reconciles_the_loop_break_body_recovery() {
    for bytes in [LOOP4_DEBUG, LOOP4_NO_DEBUG] {
        with_frozen_method_ir(bytes, b"test", b"(C)D", |canonical, view, ssa, operations, code, recovered| {
            let _ = (view, code);
            assert!(recovered.fallbacks().is_empty());
            let mut budget = ample();
            let mismatches = reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                .expect("the reconciliation completes within an ample budget");
            assert!(
                mismatches.is_empty(),
                "the derived view matches the graph and the walk: {mismatches:?}"
            );

            let tree = Region::Sequence {
                regions: recovered.regions.clone(),
            };
            let loop_region = find_region(&tree, &|region| {
                matches!(region, Region::Loop { header, .. } if header.bci() == 5)
            })
            .expect("header 5 is recovered as the real loop");
            let summary = RegionSummary::derive(loop_region, ssa, operations, &mut budget)
                .expect("the loop candidate derives within an ample budget");
            assert_eq!(
                summary.entry.as_ref().map(CanonicalBlockId::bci),
                Some(5),
                "the loop is entered at its header"
            );
            assert_eq!(
                summary
                    .exits
                    .iter()
                    .map(CanonicalBlockId::bci)
                    .collect::<Vec<_>>(),
                vec![42],
                "the stated normal exit is the loop's proved exit"
            );
            let breaks: BTreeSet<u32> = summary
                .transfers
                .iter()
                .filter(|transfer| matches!(transfer.kind, RegionTransferKind::Break { .. }))
                .map(|transfer| transfer.source_bci)
                .collect();
            assert_eq!(breaks, BTreeSet::from([26, 33]));
            let owned: BTreeSet<u32> = summary.owned.iter().map(CanonicalBlockId::bci).collect();
            for bci in [5, 11, 17, 23, 29, 36] {
                assert!(owned.contains(&bci), "the loop owns physical block {bci}");
            }
        });
    }
}

#[test]
fn summary_reconciles_the_do_while_body_ownership() {
    for bytes in [DO2_DEBUG, DO2_NO_DEBUG] {
        with_frozen_method_ir(bytes, b"test", b"()Ljava/lang/Object;", |canonical, view, ssa, operations, code, recovered| {
            let _ = (view, code);
            assert!(recovered.fallbacks().is_empty());
            let mut budget = ample();
            let mismatches = reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                .expect("the reconciliation completes within an ample budget");
            assert!(
                mismatches.is_empty(),
                "the derived view matches the graph and the walk: {mismatches:?}"
            );

            let tree = Region::Sequence {
                regions: recovered.regions.clone(),
            };
            let loop_region = find_region(&tree, &|region| {
                matches!(region, Region::Loop { form: LoopForm::DoWhile, .. })
            })
            .expect("the do-while is recovered as a Loop");
            let summary = RegionSummary::derive(loop_region, ssa, operations, &mut budget)
                .expect("the do-while candidate derives within an ample budget");
            // A do-while's body owns its header: `Region::blocks` folds the alias, so the set
            // names the header exactly once and the entry is still where control enters.
            let owned: Vec<u32> = summary.owned.iter().map(CanonicalBlockId::bci).collect();
            let unique: BTreeSet<u32> = owned.iter().copied().collect();
            assert_eq!(owned.len(), unique.len(), "no duplicated physical owner");
            assert!(unique.contains(&0), "the do-while header at bci 0 is owned");
            assert_eq!(summary.entry.as_ref().map(CanonicalBlockId::bci), Some(0));
            // The terminal return the loop proved stays a terminal of the candidate.
            let returns: BTreeSet<u32> = summary
                .terminals
                .iter()
                .filter(|terminal| terminal.kind == RegionTerminalKind::Return)
                .map(|terminal| terminal.block.bci())
                .collect();
            assert!(returns.contains(&17), "the in-body return leaf 17 is a terminal: {returns:?}");
        });
    }
}

#[test]
fn summary_reconciles_the_guard_statement_boundary() {
    with_frozen_method_ir(RESOURCE_GUARD, b"two", b"()Ljava/lang/String;", |canonical, view, ssa, operations, code, recovered| {
        let _ = (view, code);
        let guard = recovered
            .regions
            .iter()
            .find(|region| matches!(region, Region::Guard { .. }))
            .unwrap_or_else(|| panic!("the real TWR method is recognized as a Guard"));
        let Region::Guard { prefix, plan, .. } = guard else {
            unreachable!("found by the match above")
        };
        let summary = RegionSummary::derive(guard, ssa, operations, &mut ample())
            .expect("the guard candidate derives within an ample budget");
        // The statement owns every block: the prefix the walk already claimed plus the plan's.
        let mut expected: Vec<CanonicalBlockId> = prefix.clone();
        expected.extend(plan.owned().iter().cloned());
        assert_eq!(summary.owned, expected, "owned is the plan's own claim, in method order");
        // The closes the header reconstructs are the statement's own implicit effects.
        assert!(
            !summary.effects.elided_cleanup_bcis.is_empty(),
            "a resources statement elides its physical cleanup"
        );
        // The boundary: the whole-construct catch the plan presents, when its table states one.
        assert_eq!(
            summary.exceptions.len(),
            usize::from(plan.enclosing().is_some()),
            "only an enclosing clause binds a boundary"
        );
        if let Some(enclosing) = plan.enclosing() {
            let boundary = &summary.exceptions[0];
            assert_eq!(&boundary.handler, enclosing.handler());
            assert_eq!(boundary.parameter, enclosing.parameter());
        }

        let mut budget = ample();
        let mismatches = reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
            .expect("the reconciliation completes within an ample budget");
        assert!(
            mismatches.is_empty(),
            "the derived view matches the graph and the walk: {mismatches:?}"
        );
    });
}



/// The `try`/`catch` retry loop: a `while (true)` body whose normal exit is one break, whose retry
/// arm is one continue, and whose post-`try` tail is a second break — all of the same loop.
const RETRY_LOOP_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-prefixed-catch-retry-loops/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop3$TestCls.class"
);
const RETRY_LOOP_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-prefixed-catch-retry-loops/original/no-debug/classes/jadx/tests/integration/loops/TestBreakInLoop3$TestCls.class"
);

/// The named-catch loop whose `return` sits **inside** the protected range and whose handler returns
/// as well: two nested break arms, an in-range return and a handler return.
const CATCH_RETURN_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-named-catch-loop-return-tails/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop5$TestCls.class"
);
const CATCH_RETURN_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-named-catch-loop-return-tails/original/no-debug/classes/jadx/tests/integration/loops/TestBreakInLoop5$TestCls.class"
);

/// Every transfer this subtree states, read through the composition owner the walk itself reads.
fn subtree_transfers(region: &Region, out: &mut Vec<(u32, u32, bool)>) {
    if let Some(transfer) = region.transfer_leaf() {
        out.push((
            transfer.source_bci,
            transfer.kind.loop_header().map(CanonicalBlockId::bci).unwrap_or(0),
            transfer.kind.is_continue(),
        ));
    }
    match region {
        Region::Sequence { regions } | Region::Loop { body: regions, .. } => {
            for child in regions {
                subtree_transfers(child, out);
            }
        }
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => {
            subtree_transfers(then_arm, out);
            subtree_transfers(else_arm, out);
        }
        Region::ShortCircuitValue { tail, .. } => {
            for child in tail {
                subtree_transfers(child, out);
            }
        }
        Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
            for group in groups {
                subtree_transfers(&group.arm, out);
            }
        }
        Region::Try { body, catches, .. } => {
            subtree_transfers(body, out);
            for clause in catches {
                subtree_transfers(clause.body(), out);
            }
        }
        Region::Guard {
            body, finally_body, ..
        } => {
            if let Some(body) = body {
                subtree_transfers(body, out);
            }
            if let Some(cleanup) = finally_body {
                subtree_transfers(cleanup, out);
            }
        }
        _ => {}
    }
}

/// A4.1 — nested `break`/`continue` keep their target across a metadata layout change.
///
/// The retry loop's body holds a `try` whose handler arm continues the loop and a tail that breaks
/// it. The two compiled layouts (with and without a `LocalVariableTable`) move no instruction, so
/// the recovery must state the *same* targets from both: each transfer names its own loop, the
/// loop's stated exit is the block the breaks reach, and the clause's handler/parameter are the
/// table's.
#[test]
fn a41_nested_break_and_continue_keep_their_target_across_the_metadata_layout() {
    let mut readings = Vec::new();
    for bytes in [RETRY_LOOP_DEBUG, RETRY_LOOP_NO_DEBUG] {
        with_frozen_method_ir(
            bytes,
            b"writeMore",
            b"(Ljava/lang/String;)V",
            |canonical, view, ssa, operations, code, recovered| {
                let _ = (view, code);
                assert!(recovered.fallbacks().is_empty());
                let mut budget = ample();
                let mismatches =
                    reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                        .expect("the reconciliation completes within an ample budget");
                assert!(
                    mismatches.is_empty(),
                    "the composition facts match the graph: {mismatches:?}"
                );

                let loop_region = recovered
                    .regions
                    .iter()
                    .find(|region| matches!(region, Region::Loop { .. }))
                    .expect("the retry body is recovered as the method's loop");
                let summary =
                    RegionSummary::derive(loop_region, ssa, operations, &mut budget)
                        .expect("the loop candidate derives within an ample budget");
                let loop_bci = summary
                    .entry
                    .as_ref()
                    .expect("a loop is entered at its header")
                    .bci();

                // Every transfer names this one loop: the normal exit's break, the retry arm's
                // continue, and the post-`try` tail's break.
                let mut transfers: Vec<(u32, u32, bool)> = summary
                    .transfers
                    .iter()
                    .map(|transfer| {
                        (
                            transfer.source_bci,
                            transfer
                                .kind
                                .loop_header()
                                .map(CanonicalBlockId::bci)
                                .unwrap_or(0),
                            transfer.kind.is_continue(),
                        )
                    })
                    .collect();
                transfers.sort_unstable();
                assert_eq!(
                    transfers,
                    vec![(28, loop_bci, false), (45, loop_bci, true), (79, loop_bci, false)],
                    "break, continue and the tail break all keep their proved target"
                );
                // The loop states one normal exit: the block the two breaks reach.
                let exits: Vec<u32> = summary.exits.iter().map(CanonicalBlockId::bci).collect();
                assert_eq!(exits, vec![82], "the loop's stated exit is the break destination");
                // The clause binds the table's handler and parameter.
                assert_eq!(summary.exceptions.len(), 1);
                assert_eq!(summary.exceptions[0].handler.bci(), 31);
                assert_eq!(summary.exceptions[0].parameter, 4);

                // The composition owner the walk reads states exactly what the summary collected
                // for this subtree: one fact, two consumers, no second derivation.
                let mut from_composition = Vec::new();
                subtree_transfers(loop_region, &mut from_composition);
                from_composition.sort_unstable();
                assert_eq!(
                    from_composition, transfers,
                    "the walk's own transfer reading is the summary's transfer fact"
                );

                readings.push((transfers, exits, summary.exceptions.len()));
            },
        );
    }
    assert_eq!(readings[0], readings[1], "the layout change moves no composition fact");
}

/// A4.1 — nested `break` arms and the `return` inside a protected range keep their targets and
/// their prior effects.
///
/// The named-catch loop has two nested break arms (`m`/`M` and `g`/`G`), a `return` **inside** the
/// protected range whose block still takes the handler's exception edge from an earlier instruction,
/// and a handler `return`. Every break names the one loop; both returns are terminals; and the
/// in-range return's exception edge is not a normal continuation.
#[test]
fn a41_nested_break_arms_and_the_in_range_return_keep_their_targets() {
    let mut readings = Vec::new();
    for bytes in [CATCH_RETURN_DEBUG, CATCH_RETURN_NO_DEBUG] {
        with_frozen_method_ir(
            bytes,
            b"test",
            b"(Ljava/lang/String;)J",
            |canonical, view, ssa, operations, code, recovered| {
                let _ = (view, code);
                assert!(recovered.fallbacks().is_empty());
                let mut budget = ample();
                let mismatches =
                    reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                        .expect("the reconciliation completes within an ample budget");
                assert!(
                    mismatches.is_empty(),
                    "the in-range return's exception edge is not a contradiction: {mismatches:?}"
                );

                let region = recovered
                    .regions
                    .first()
                    .expect("the method recovers one region");
                let summary =
                    RegionSummary::derive(region, ssa, operations, &mut budget)
                        .expect("the candidate derives within an ample budget");

                // Three break arms, each naming the loop they leave — the two nested `if` arms and
                // the arm that ends the loop after the second comparison.
                let mut breaks: Vec<u32> = summary
                    .transfers
                    .iter()
                    .filter(|transfer| transfer.kind.is_break())
                    .map(|transfer| transfer.source_bci)
                    .collect();
                breaks.sort_unstable();
                assert_eq!(breaks, vec![64, 78, 85], "each nested break keeps its own source");
                let loop_headers: BTreeSet<u32> = summary
                    .transfers
                    .iter()
                    .filter_map(|transfer| transfer.kind.loop_header())
                    .map(CanonicalBlockId::bci)
                    .collect();
                assert_eq!(loop_headers.len(), 1, "all three arms break the one loop");

                // Both returns are terminals: the in-range one and the handler's.
                let returns: BTreeSet<u32> = summary
                    .terminals
                    .iter()
                    .filter(|terminal| terminal.kind == RegionTerminalKind::Return)
                    .map(|terminal| terminal.block.bci())
                    .collect();
                assert_eq!(returns, BTreeSet::from([106, 124]));

                // The in-range return's block takes exactly one edge and it is the handler's: the
                // exception of an earlier instruction of the same contiguous block, not a normal
                // continuation of the `return`.
                let in_range = summary
                    .terminals
                    .iter()
                    .find(|terminal| terminal.block.bci() == 106)
                    .expect("the in-range return is a terminal");
                let leaving: Vec<CanonicalEdgeKind> = canonical
                    .edges()
                    .iter()
                    .filter(|edge| edge.from() == &in_range.block)
                    .map(|edge| edge.kind().clone())
                    .collect();
                assert_eq!(leaving.len(), 1, "one edge leaves the in-range return block");
                assert!(
                    matches!(leaving[0], CanonicalEdgeKind::Exception { .. }),
                    "and it is the protected range's exception edge, not a continuation"
                );

                // The clause binds the handler the in-range return's edge reaches.
                assert_eq!(summary.exceptions.len(), 1);
                assert_eq!(summary.exceptions[0].handler.bci(), 124);
                assert_eq!(summary.exceptions[0].parameter, 2);

                readings.push((breaks, returns, summary.exceptions[0].handler.bci()));
            },
        );
    }
    assert_eq!(readings[0], readings[1], "the layout change moves no composition fact");
}





/// Family 1 — a loop used as a branch arm: the two javac8 layouts of the same source.
const ARM_LOOP_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/A/javac8/debug/classes/NestedReturnA$TestCls.class"
);
const ARM_LOOP_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/A/javac8/no-debug/classes/NestedReturnA$TestCls.class"
);

/// Family 2 — the switch whose join stays inside a loop body, in both registered javac targets.
const SWITCH_JOIN_JAVAC8: &[u8] =
    include_bytes!("../../../../tests/fixtures/p3-switch-nonlocal-exits/javac8/SM.class");
const SWITCH_JOIN_JAVAC23: &[u8] =
    include_bytes!("../../../../tests/fixtures/p3-switch-nonlocal-exits/javac23/SM.class");

/// Family 3 — the continue ladder: a `continue` and a `break` of one loop in nested arms.
const LADDER_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-continue-ladder-joins/original/debug/classes/jadx/tests/integration/loops/TestContinueInLoop$TestCls.class"
);
const LADDER_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-continue-ladder-joins/original/no-debug/classes/jadx/tests/integration/loops/TestContinueInLoop$TestCls.class"
);

/// The first region for which `keep` holds, walking the tree the way composition does.
fn first_region<'a>(region: &'a Region, keep: &impl Fn(&Region) -> bool) -> Option<&'a Region> {
    if keep(region) {
        return Some(region);
    }
    let mut found = None;
    match region {
        Region::Sequence { regions } | Region::Loop { body: regions, .. } => {
            for child in regions {
                found = first_region(child, keep);
                if found.is_some() {
                    return found;
                }
            }
        }
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => {
            found = first_region(then_arm, keep)
                .or_else(|| first_region(else_arm, keep));
        }
        Region::ShortCircuitValue { tail, .. } => {
            for child in tail {
                found = first_region(child, keep);
                if found.is_some() {
                    return found;
                }
            }
        }
        Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
            for group in groups {
                found = first_region(&group.arm, keep);
                if found.is_some() {
                    return found;
                }
            }
        }
        Region::Try { body, catches, .. } => {
            found = first_region(body, keep).or_else(|| {
                catches
                    .iter()
                    .find_map(|clause| first_region(clause.body(), keep))
            });
        }
        Region::Guard {
            body, finally_body, ..
        } => {
            found = body
                .as_deref()
                .and_then(|body| first_region(body, keep))
                .or_else(|| {
                    finally_body
                        .as_deref()
                        .and_then(|cleanup| first_region(cleanup, keep))
                });
        }
        _ => {}
    }
    found
}

/// A4.1 — a loop arm whose stated exit **is** the enclosing branch's join.
///
/// This is the exit-vs-join reading the walk makes when one arm of a two-edge body is itself a
/// loop: the arm leaves exactly where its sibling joins, so the loop's own stated exit and the
/// `If`'s join are the same block, and the sibling arm is empty. The reading comes from the loop's
/// variant ([`Region::loop_exits_at`]), never from a destination borrowed off the join.
#[test]
fn a41_loop_arm_exit_is_the_enclosing_join_in_both_layouts() {
    let mut readings = Vec::new();
    for bytes in [ARM_LOOP_DEBUG, ARM_LOOP_NO_DEBUG] {
        with_frozen_method_ir(
            bytes,
            b"run",
            b"(IIIII)I",
            |canonical, view, ssa, operations, code, recovered| {
                let _ = (view, code);
                assert!(recovered.fallbacks().is_empty());
                let mut budget = ample();
                let mismatches =
                    reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                        .expect("the reconciliation completes within an ample budget");
                assert!(mismatches.is_empty(), "the composition facts match: {mismatches:?}");

                let tree = Region::Sequence {
                    regions: recovered.regions.clone(),
                };
                // The branch arm that is a loop ending where its sibling joins. The arm may state
                // the loop directly or behind a straight prefix, so the loop is looked up inside it.
                let arm = first_region(&tree, &|region| {
                    let Region::If {
                        then_arm,
                        else_arm,
                        join: Some(join),
                        ..
                    } = region
                    else {
                        return false;
                    };
                    let sibling_is_empty = |arm: &Region| {
                        matches!(arm, Region::Straight { blocks } if blocks.is_empty())
                    };
                    let loop_meets = |arm: &Region| {
                        first_region(arm, &|inner| inner.loop_exits_at(join)).is_some()
                    };
                    (sibling_is_empty(then_arm) && loop_meets(else_arm))
                        || (sibling_is_empty(else_arm) && loop_meets(then_arm))
                })
                .expect("the nested loop is a branch arm whose exit is the join");
                let Region::If {
                    then_arm,
                    else_arm,
                    join: Some(join),
                    ..
                } = arm
                else {
                    unreachable!("found by the predicate above")
                };
                let loop_in_then = first_region(then_arm, &|inner| inner.loop_exit().is_some()).is_some();
                let (arm_loop, sibling) = if loop_in_then {
                    (
                        first_region(then_arm, &|inner| inner.loop_exit().is_some()),
                        else_arm.as_ref(),
                    )
                } else {
                    (
                        first_region(else_arm, &|inner| inner.loop_exit().is_some()),
                        then_arm.as_ref(),
                    )
                };
                let arm_loop = arm_loop.expect("the arm that holds the loop");
                assert_eq!(
                    arm_loop.loop_exit(),
                    Some(join),
                    "the loop arm's stated exit is the enclosing join"
                );
                assert!(
                    matches!(sibling, Region::Straight { blocks } if blocks.is_empty()),
                    "the sibling arm is empty: its own edge reaches the join"
                );
                readings.push((arm_loop.loop_exit().map(CanonicalBlockId::bci), join.bci()));
            },
        );
    }
    assert_eq!(readings[0], readings[1], "the layout change moves no exit-vs-join fact");
    assert_eq!(
        readings[0],
        (Some(99), 99),
        "the nested loop's exit is the join at BCI 99"
    );
}

/// A4.1 — a switch's join stays the switch's own destination inside a loop body.
///
/// The switch arm that falls through to the next case reaches the switch's join, not the loop's
/// exit, so the join is stated on the switch and the block after the switch (which the loop body
/// owns) is that same join. The switch-arm reading is the only one that may claim it.
#[test]
fn a41_switch_join_stays_the_switch_destination_in_both_compilers() {
    let mut readings = Vec::new();
    for bytes in [SWITCH_JOIN_JAVAC8, SWITCH_JOIN_JAVAC23] {
        with_frozen_method_ir(
            bytes,
            b"onlyBreak",
            b"([I)I",
            |canonical, view, ssa, operations, code, recovered| {
                let _ = (view, code);
                assert!(recovered.fallbacks().is_empty());
                let mut budget = ample();
                let mismatches =
                    reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                        .expect("the reconciliation completes within an ample budget");
                assert!(mismatches.is_empty(), "the composition facts match: {mismatches:?}");

                let tree = Region::Sequence {
                    regions: recovered.regions.clone(),
                };
                let loop_region = first_region(&tree, &|region| {
                    matches!(region, Region::Loop { .. })
                })
                .expect("the method is one loop over the array");
                let Region::Loop { exit, body, .. } = loop_region else {
                    unreachable!("found by the predicate above")
                };
                let switch_region = first_region(loop_region, &|region| {
                    matches!(region, Region::Switch { .. })
                })
                .expect("the loop body dispatches on the element");
                let Region::Switch { join, .. } = switch_region else {
                    unreachable!("found by the predicate above")
                };
                let switch_join = join
                    .as_ref()
                    .expect("the switch states the join its fallthrough breaks to");
                // The switch's join is not the loop's exit: two different destinations.
                assert!(
                    exit.as_ref() != Some(switch_join),
                    "the switch's join is its own destination, not the loop's exit"
                );
                // The block the loop body walks after the switch is that same join, so the join is
                // owned by the body and reached by the arm that falls through.
                let after_switch = body
                    .iter()
                    .skip_while(|region| !std::ptr::eq(*region, switch_region))
                    .nth(1)
                    .expect("the loop body continues after the switch");
                assert_eq!(
                    run_single_block(std::slice::from_ref(after_switch)),
                    Some(switch_join),
                    "the loop body resumes at the switch's own join"
                );
                readings.push((
                    switch_join.bci(),
                    exit.as_ref().map(CanonicalBlockId::bci),
                ));
            },
        );
    }
    assert_eq!(readings[0], readings[1], "the compiler target moves no switch-join fact");
    assert_eq!(readings[0], (52, Some(61)), "join 52 stays inside the loop that exits at 61");
}

/// A4.1 — the continue ladder keeps the continue's and the break's loop across layouts.
///
/// One loop body holds a nested `if` whose arm continues it and a later `if` whose arm breaks it.
/// The ladder's elected join is the arm's continuation, and each transfer names the loop it
/// leaves — the reading the election consumes (`run_single_block` for the bridge arm, and the
/// transfer's own loop identity).
#[test]
fn a41_continue_ladder_keeps_its_targets_in_both_layouts() {
    let mut readings = Vec::new();
    for bytes in [LADDER_DEBUG, LADDER_NO_DEBUG] {
        with_frozen_method_ir(
            bytes,
            b"test",
            b"([II)V",
            |canonical, view, ssa, operations, code, recovered| {
                let _ = (view, code);
                assert!(recovered.fallbacks().is_empty());
                let mut budget = ample();
                let mismatches =
                    reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                        .expect("the reconciliation completes within an ample budget");
                assert!(mismatches.is_empty(), "the composition facts match: {mismatches:?}");

                let tree = Region::Sequence {
                    regions: recovered.regions.clone(),
                };
                let loop_region = first_region(&tree, &|region| {
                    matches!(region, Region::Loop { .. })
                })
                .expect("the method is one loop");
                let loop_bci = loop_region
                    .loop_exit()
                    .is_some()
                    .then(|| {
                        first_region(&tree, &|region| matches!(region, Region::Loop { .. }))
                            .and_then(|region| match region {
                                Region::Loop { header, .. } => Some(header.bci()),
                                _ => None,
                            })
                    })
                    .flatten()
                    .expect("the loop is entered at its header");

                // Both transfers the body holds name this one loop.
                let mut transfers = Vec::new();
                subtree_transfers(loop_region, &mut transfers);
                transfers.sort_unstable();
                assert_eq!(
                    transfers,
                    vec![(32, loop_bci, true), (47, loop_bci, false)],
                    "the continue and the break both name this loop"
                );

                // The ladder's elected join is a stated `If` join inside the body, and the jumping
                // arm is exactly one bridge block — the single-block run the re-election consumes.
                let ladder_if = first_region(loop_region, &|region| {
                    matches!(
                        region,
                        Region::If {
                            join: Some(_),
                            then_arm,
                            else_arm,
                            ..
                        } if matches!(run_single_block(std::slice::from_ref(then_arm.as_ref())), Some(_))
                            != matches!(run_single_block(std::slice::from_ref(else_arm.as_ref())), Some(_))
                    )
                });
                let ladder_join = ladder_if.and_then(|region| match region {
                    Region::If { join, .. } => join.as_ref().map(CanonicalBlockId::bci),
                    _ => None,
                });
                readings.push((transfers, ladder_join, loop_bci));
            },
        );
    }
    assert_eq!(readings[0], readings[1], "the layout change moves no ladder fact");
}

/// A4.2 family 1 — a named `catch` **inside** a loop body: the loop owns the iteration, the `try`
/// owns the range its one row protects, and every throw site in that range is the clause's.
const LOOP_TRY_JAVAC8: &[u8] =
    include_bytes!("../../../../tests/fixtures/p3-loop-body-catch/DV-none-javac8.class");
const LOOP_TRY_JAVAC23: &[u8] =
    include_bytes!("../../../../tests/fixtures/p3-loop-body-catch/DV-none-javac23.class");

/// A4.2 family 2 — a named `catch` **around** a loop: the `try` owns the prefix, the loop and the
/// clause, and the loop's own exit is the range end.
const TRY_LOOP: &[u8] = include_bytes!(
    "../../../../tests/fixtures/cf11-ordinary-try-list-generic/UpstreamNestedLoops.class"
);

/// A4.2 order fixture — one `try` with two clauses reaching two different handlers.
const TWO_CLAUSES: &[u8] =
    include_bytes!("../../../../tests/fixtures/p3-typed-catch/v8/TypedCatch.class");

/// The one `try` candidate a recovered method holds, wherever the tree states it — a `catch` is a
/// statement like any other, so it may sit at the top level (family 2) or inside a loop body
/// (family 1).
fn find_try(recovered: &Recovered) -> &Region {
    recovered
        .regions
        .iter()
        .find_map(|region| first_region(region, &|inner| matches!(inner, Region::Try { .. })))
        .expect("the fixture recovers one try")
}

/// A4.2 — both problem families recover, and the handler order and every throw site's protection
/// attribution are unchanged from what the tree states.
///
/// *Family 1* is a `catch` inside a loop body; *family 2* is a loop inside a `catch`'s range. Both
/// are the closed ordinary named catch the combination check is about. For each, the clause list is
/// the exception table's own order, the clause's handler is the block its row reaches, the row's
/// protected blocks are the candidate's own, and — the fact this task exists for — **every throw
/// site of an owned block takes an edge whose ordinal is one of the clause's rows and whose target
/// is that clause's handler**. A throw site attributed to any other row would be an exception the
/// statement does not write.
#[test]
fn a42_both_named_catch_families_keep_their_handler_order_and_throw_site_ownership() {
    // Family 1, both compiler targets: the same source, a different layout.
    let mut family_one = Vec::new();
    for bytes in [LOOP_TRY_JAVAC8, LOOP_TRY_JAVAC23] {
        with_frozen_method_ir(
            bytes,
            b"plainFor",
            b"([I)I",
            |canonical, view, ssa, operations, code, recovered| {
                let _ = (view, code);
                assert!(recovered.fallbacks().is_empty());
                let mut budget = ample();
                let mismatches =
                    reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                        .expect("the reconciliation completes within an ample budget");
                assert!(mismatches.is_empty(), "the view matches the graph: {mismatches:?}");

                let try_region = find_try(recovered);
                let summary = RegionSummary::derive(try_region, ssa, operations, &mut budget)
                    .expect("the try candidate derives within an ample budget");

                // One clause, bound where the table says: the handler entry and the parameter.
                assert_eq!(summary.exceptions.len(), 1);
                let clause = &summary.exceptions[0];
                assert_eq!(clause.handler.bci(), 35);
                assert_eq!(clause.parameter, 3);

                // The composition owner reads the same clause the derived view records.
                let from_owner = try_region.exception_clauses();
                assert_eq!(from_owner.len(), 1);
                assert_eq!(from_owner[0].handler, clause.handler);
                assert_eq!(from_owner[0].types, clause.types);
                assert_eq!(from_owner[0].parameter, clause.parameter);

                // Every throw site in a block this clause's own row **protects** is under that row,
                // and the row's exception edge lands on the clause's handler. This is the
                // attribution the task keeps: a site under any other row would be an exception the
                // statement does not write. The clause *body*'s own blocks are owned by this
                // candidate too, and they are correctly not under this row — a `catch` body runs
                // outside the range it catches for.
                let row = canonical
                    .handler_rows()
                    .iter()
                    .find(|row| row.handler() == Some(&clause.handler))
                    .expect("the canonical row binds the clause handler");
                let protected_sites: Vec<u32> = canonical
                    .throw_sites()
                    .iter()
                    .filter(|site| row.protected().contains(site.block()))
                    .map(|site| site.bci())
                    .collect();
                assert!(
                    !protected_sites.is_empty(),
                    "the protected range holds throwing instructions"
                );
                for site in canonical
                    .throw_sites()
                    .iter()
                    .filter(|site| row.protected().contains(site.block()))
                {
                    // The site's own handler list is the declaration order the JVM dispatches in.
                    assert!(
                        site.handlers().contains(&row.ordinal()),
                        "throw site {} is under this try's row: {:?}",
                        site.bci(),
                        site.handlers()
                    );
                    let edge = canonical
                        .edges()
                        .iter()
                        .find(|edge| {
                            edge.from() == site.block()
                                && edge.kind()
                                    == CanonicalEdgeKind::Exception {
                                        handler_ordinal: row.ordinal(),
                                    }
                        })
                        .expect("the row's edge leaves the block holding the site");
                    assert_eq!(
                        edge.to(),
                        &clause.handler,
                        "the row's edge reaches the clause's own handler"
                    );
                }
                // The row protects only blocks the candidate owns.
                assert!(
                    row.protected()
                        .iter()
                        .all(|block| summary.owned.contains(block))
                );

                // Family 1 puts the `try` **inside** the loop body: the loop states the exit its break
                // arms reach, and the `try` is one of its body regions.
                let loop_region = recovered
                    .regions
                    .iter()
                    .find_map(|region| first_region(region, &|inner| {
                        matches!(inner, Region::Loop { .. })
                    }))
                    .expect("the method is one loop");
                let loop_exit: u32 = loop_region
                    .loop_exit()
                    .expect("the loop states its exit")
                    .bci();
                assert!(
                    first_region(loop_region, &|inner| matches!(inner, Region::Try { .. }))
                        .is_some(),
                    "the named catch is a statement of the loop body"
                );
                let handler_bci = clause.handler.bci();
                let parameter = clause.parameter;
                let transfers = summary.transfers.len();
                family_one.push((handler_bci, parameter, loop_exit, transfers));
            },
        );
    }
    assert_eq!(
        family_one[0], family_one[1],
        "the compiler target moves no handler-order or attribution fact"
    );
    assert_eq!(
        family_one[0],
        (35, 3, 45, 0),
        "handler 35 sits in slot 3 inside the loop that exits at 45"
    );

    // Family 2: the loop is inside the `try`'s own range.
    with_frozen_method_ir(
        TRY_LOOP,
        b"handled",
        b"(Ljava/util/List;Ljava/util/List;)V",
        |canonical, view, ssa, operations, code, recovered| {
            let _ = (view, code);
            assert!(recovered.fallbacks().is_empty());
            let mut budget = ample();
            let mismatches =
                reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                    .expect("the reconciliation completes within an ample budget");
            assert!(mismatches.is_empty(), "the view matches the graph: {mismatches:?}");

            let try_region = find_try(recovered);
            let summary = RegionSummary::derive(try_region, ssa, operations, &mut budget)
                .expect("the try candidate derives within an ample budget");
            assert_eq!(summary.exceptions.len(), 1);
            let clause = &summary.exceptions[0];
            assert_eq!(clause.handler.bci(), 50);
            assert_eq!(clause.parameter, 3);

            // The protected range owns the loop: the loop's header and body are this candidate's
            // blocks, and the loop's own exit is the range end the row states.
            let loop_region = first_region(try_region, &|region| matches!(region, Region::Loop { .. }))
                .expect("the try's protected range holds the loop");
            let Region::Loop { header, exit, .. } = loop_region else {
                unreachable!("found by the predicate above")
            };
            assert_eq!(header.bci(), 7);
            assert_eq!(exit.as_ref().map(CanonicalBlockId::bci), Some(47));
            assert!(
                summary.owned.contains(header),
                "the loop header is inside the protected range"
            );

            let row = canonical
                .handler_rows()
                .iter()
                .find(|row| row.handler() == Some(&clause.handler))
                .expect("the canonical row binds the clause handler");
            assert_eq!((row.handler_bci(), row.ordinal()), (50, 0));
            assert!(
                !row.protected().is_empty(),
                "the row protects the range it was proved for"
            );
            for site in canonical
                .throw_sites()
                .iter()
                .filter(|site| row.protected().contains(site.block()))
            {
                assert!(
                    site.handlers().contains(&row.ordinal()),
                    "throw site {} is under the try's row: {:?}",
                    site.bci(),
                    site.handlers()
                );
            }
            assert!(
                row.protected()
                    .iter()
                    .all(|block| summary.owned.contains(block))
            );
        },
    );
}

/// A4.2 — the clauses are written in **exception-table order**, and a reordered view is refused.
///
/// The JVM dispatches clauses in table order, so that order is a fact of the statement and not a
/// presentation choice. Two clauses reaching two different handlers still match their rows one for
/// one when swapped, which is why the row agreement cannot catch a reordering — the order is proved
/// on its own ([`RegionSummaryMismatch::ExceptionClauseOrderMismatch`]), and this test pins both the
/// real order and the refusal of a swapped one.
#[test]
fn a42_clause_order_is_the_exception_table_order_and_reordering_is_refused() {
    with_frozen_method_ir(
        TWO_CLAUSES,
        b"twoCatches",
        b"(I)I",
        |canonical, view, ssa, operations, code, recovered| {
            let _ = (view, code);
            assert!(recovered.fallbacks().is_empty());
            let mut budget = ample();
            let mismatches =
                reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                    .expect("the reconciliation completes within an ample budget");
            assert!(mismatches.is_empty(), "the view matches the graph: {mismatches:?}");

            let try_region = find_try(recovered);
            let clauses = try_region.exception_clauses();
            let handlers: Vec<u32> = clauses.iter().map(|clause| clause.handler.bci()).collect();
            assert_eq!(
                handlers,
                vec![28, 31],
                "the clauses are written in the table's own order"
            );
            // The table states the same order: walking the clauses in the order the tree writes them
            // visits strictly increasing row ordinals. A swap would visit them out of order.
            let mut previous: Option<u32> = None;
            for clause in &clauses {
                let row = canonical
                    .handler_rows()
                    .iter()
                    .find(|row| row.handler() == Some(&clause.handler))
                    .expect("the row binds this clause");
                if let Some(previous) = previous {
                    assert!(
                        row.ordinal() > previous,
                        "handler {} follows the clause before it in the table",
                        clause.handler.bci()
                    );
                }
                previous = Some(row.ordinal());
            }
            assert_eq!(previous, Some(1), "the last clause is the table's last row");

            // A swapped clause list still matches every row one for one — and is refused, because
            // the order is the statement's.
            let mut swapped = recovered.clone();
            let position = swapped
                .regions
                .iter()
                .position(|region| matches!(region, Region::Try { .. }))
                .expect("the recovered tree holds the try");
            let Region::Try { catches, .. } = &mut swapped.regions[position] else {
                unreachable!("found by the position above");
            };
            assert_eq!(catches.len(), 2);
            catches.swap(0, 1);

            let mut budget = ample();
            let mismatches =
                reconcile_method_summaries(&swapped, canonical, ssa, operations, &mut budget)
                    .expect("the reconciliation of the reordered view completes");
            let refused: Vec<u32> = mismatches
                .iter()
                .filter_map(|mismatch| match mismatch {
                    RegionSummaryMismatch::ExceptionClauseOrderMismatch { handler } => {
                        Some(handler.bci())
                    }
                    _ => None,
                })
                .collect();
            assert!(
                !refused.is_empty(),
                "a reordered clause list is refused: {mismatches:?}"
            );
            // The swapped list writes handler 31 first and handler 28 second; the row ordinals run
            // 1 then 0, so the second clause is the one that fails to advance and is named.
            assert_eq!(
                refused,
                vec![28],
                "the clause that no longer follows its row is the one named"
            );
        },
    );
}

/// The same decoded body with a replaced exception table: the instruction stream and its operands
/// are unchanged, which is what makes a table-only mutation a fact about the table.
fn rebuilt_code(
    code: &jarde_reader::classfile::MethodCodeFacts,
    exception_handlers: Vec<jarde_reader::classfile::ExceptionHandlerFact>,
    exception_handler_count: u32,
) -> jarde_reader::classfile::MethodCodeFacts {
    jarde_reader::classfile::MethodCodeFacts::from_parts(
        code.max_stack,
        code.max_locals,
        code.code_span.clone(),
        code.instructions
            .iter()
            .cloned()
            .zip(code.operands().iter().cloned())
            .collect(),
        exception_handlers,
        exception_handler_count,
        code.execution.clone(),
        code.stopped_at.clone(),
        code.debug().clone(),
    )
}

/// A4.2 — the closed ordinary named catch's combination check **refuses** its four failure shapes.
///
/// The positive certificate is [`ordinary_try_list_scope`]: the combination of one named clause, its
/// protected prefix and the `foreach` loop the range holds. Each of the four shapes below is a
/// mutation of that real candidate, and each mutation is made in the fact the corresponding
/// condition reads — not in the tree's presentation:
///
/// * **a foreign entry** — the clause's handler stops being reachable only through its own row,
///   because the body that proves it is no longer the clause's;
/// * **overlapping protection** — a second physical row covers part of the same range, so the JVM's
///   dispatch order no longer says which clause owns the code;
/// * **an unresolved exit** — the range's exclusive end is not where the last owned block ends, so
///   the statement's continuation is not the one its row states;
/// * **duplicate ownership** — one physical block is claimed twice, so the candidate is not a
///   partition of the blocks it presents.
///
/// Every one of them must answer `Ok(None)`: a refusal, never a stop and never a certificate.
#[test]
fn a42_the_closed_named_catch_check_refuses_foreign_entry_overlap_exit_and_duplicate_ownership() {
    with_frozen_method_ir(
        TRY_LOOP,
        b"handled",
        b"(Ljava/util/List;Ljava/util/List;)V",
        |canonical, _view, _ssa, _operations, code, recovered| {
            // The positive baseline: this candidate is the certified closed named catch.
            let mut budget = ample();
            assert!(
                super::ordinary_try_list_scope(recovered, canonical, code, &mut budget)
                    .expect("the positive certificate completes")
                    .is_some(),
                "the fixture is the certified shape"
            );

            // 1. A foreign entry: the clause body is moved out of the clause, so the handler the
            //    candidate owns is no longer the one its row reaches through the clause alone.
            let mut detached = recovered.clone();
            assert!(
                super::move_ordinary_try_catch_body_to_sibling_for_test(&mut detached),
                "the fixture's ordinary try has a clause body to detach"
            );
            assert!(
                super::ordinary_try_list_scope(&detached, canonical, code, &mut ample())
                    .expect("a detached clause body is a refusal, not a stop")
                    .is_none(),
                "a handler reachable outside its clause is refused"
            );

            // 2. Overlapping protection: a second row covers the same range.
            let row = code.exception_handlers[0].clone();
            let mut overlapping_rows = code.exception_handlers.clone();
            let mut competitor = row.clone();
            competitor.ordinal = row.ordinal + 1;
            competitor.handler_bci = canonical
                .blocks()
                .iter()
                .map(|block| block.id().bci())
                .find(|bci| *bci != row.handler_bci)
                .expect("the method has another block");
            overlapping_rows.push(competitor);
            let overlapping = rebuilt_code(code, overlapping_rows, code.exception_handler_count);
            assert!(
                super::ordinary_try_list_scope(recovered, canonical, &overlapping, &mut ample())
                    .expect("an overlapping row is a refusal, not a stop")
                    .is_none(),
                "two rows over one range are refused"
            );

            // 3. An unresolved exit: the range no longer ends where the last owned block does.
            let mut shrunk_rows = code.exception_handlers.clone();
            shrunk_rows[0].end_bci -= 1;
            let shrunk = rebuilt_code(code, shrunk_rows, code.exception_handler_count);
            assert!(
                super::ordinary_try_list_scope(recovered, canonical, &shrunk, &mut ample())
                    .expect("a range end that is not the block end is a refusal")
                    .is_none(),
                "a range whose end is not the continuation is refused"
            );

            // 4. Duplicate ownership: one candidate is presented twice.
            let mut doubled = recovered.clone();
            doubled.regions.push(doubled.regions[0].clone());
            assert!(
                super::ordinary_try_list_scope(&doubled, canonical, code, &mut ample())
                    .expect("a duplicate owner is a refusal, not a stop")
                    .is_none(),
                "a block owned twice is refused"
            );

            // And the unmutated candidate still certifies: the four refusals above are the
            // mutations' doing, not the check's own state.
            assert!(
                super::ordinary_try_list_scope(recovered, canonical, code, &mut ample())
                    .expect("the positive certificate still completes")
                    .is_some()
            );
        },
    );
}

// ---------------------------------------------------------------------------------------------
// A4.3 — the un-migrated finally / TWR / monitor shapes keep their facts after 5.1–5.4.
//
// These pin *structure*, not text: the artifact-level suites (`p3_shared_join_finally`,
// `twr_direct_finally`, `nested_monitor_regions`, …) already pin each shape's presentation and
// refusal boundaries, and they pass unchanged in this worktree. What was NOT pinned anywhere
// before is what the migrated Region contract states about those same trees — which blocks each
// candidate owns, which exits it names, where it leaves the method, and which cleanup it elides.
// Those are the facts the 5.1–5.4 migration could silently have moved while every text assertion
// kept passing, so they are what 5.5 checks here.
// ---------------------------------------------------------------------------------------------

/// The `finally`-copy shape the S4 regression was found on: a loop with two returns, cleanup in a
/// guard. This is the exact method whose presentation broke and was repaired.
const MULTI_RETURN_FINALLY: &[u8] = include_bytes!(
    "../../../../openspec/evidence/java-syntax-2026-09-28/cf16-test5-multi-return/TestTryCatchFinally5$TestCls.class"
);
const MULTI_RETURN_FINALLY_DESCRIPTOR: &[u8] = b"(Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$A;Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$B;)Ljava/util/List;";

/// The committed single-exit `synchronized` statement (`recover-twr-direct-finally`'s control).
const SINGLE_EXIT_MONITOR: &[u8] =
    include_bytes!("../../../../tests/fixtures/p3-sync-return/v8/Locked.class");

fn terminals_of(summary: &RegionSummary) -> Vec<(u32, RegionTerminalKind)> {
    summary
        .terminals
        .iter()
        .map(|terminal| (terminal.block.bci(), terminal.kind))
        .collect()
}

#[test]
fn a43_the_multi_return_finally_keeps_every_exit_and_its_one_cleanup_guard() {
    with_frozen_method_ir(
        MULTI_RETURN_FINALLY,
        b"test",
        MULTI_RETURN_FINALLY_DESCRIPTOR,
        |canonical, _view, ssa, operations, _code, recovered| {
            // The shape is presented, not quoted: the regression this pins was a whole-method
            // refusal of exactly this method.
            assert!(
                recovered.fallbacks().is_empty(),
                "the multi-return finally presents: {:?}",
                recovered.fallbacks()
            );
            let mut budget = ample();
            let mismatches =
                reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                    .expect("the reconciliation completes within an ample budget");
            assert!(
                mismatches.is_empty(),
                "the derived view still matches the graph and the walk: {mismatches:?}"
            );

            let tree = Region::Sequence {
                regions: recovered.regions.clone(),
            };
            // Return/throw coverage: every exit the shape has is recorded, and the cleanup's
            // rethrow is one of them. Three returns and the handler's `athrow` — no exit the
            // source wrote may be missing, and none may be invented.
            // The production-derived view, one entry per top-level candidate. Their owned sets are
            // disjoint, so summing their terminals counts every exit exactly once — which is the
            // property being checked: no exit the source wrote is dropped, and none is invented.
            let mut terminals: Vec<(u32, RegionTerminalKind)> = recovered
                .summaries
                .iter()
                .flat_map(terminals_of)
                .collect();
            terminals.sort_by_key(|(bci, kind)| (*bci, matches!(kind, RegionTerminalKind::Throw)));
            assert_eq!(
                terminals,
                vec![
                    (10, RegionTerminalKind::Return),
                    (31, RegionTerminalKind::Return),
                    (79, RegionTerminalKind::Return),
                    (93, RegionTerminalKind::Throw),
                ],
                "every return and the cleanup's rethrow stay accounted for"
            );

            // The cleanup itself is one guard, and it owns a bounded tree of its own: the
            // regression was in reading that tree, so its presence is asserted structurally.
            let guard = find_region(&tree, &|region| matches!(region, Region::Guard { .. }))
                .expect("the finally cleanup is one guard statement");
            assert!(
                !guard.composed_children().is_empty(),
                "the guard holds the body its cleanup was lowered into"
            );
        },
    );
}

#[test]
fn a43_the_monitor_statement_pairs_one_enter_with_one_exit_and_invents_no_clause() {
    with_frozen_method_ir(
        SINGLE_EXIT_MONITOR,
        b"locked",
        b"()I",
        |canonical, _view, ssa, operations, _code, recovered| {
            assert!(recovered.fallbacks().is_empty(), "{:?}", recovered.fallbacks());
            let mut budget = ample();
            let mismatches =
                reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                    .expect("the reconciliation completes within an ample budget");
            assert!(mismatches.is_empty(), "{mismatches:?}");

            let tree = Region::Sequence {
                regions: recovered.regions.clone(),
            };
            let guard = find_region(&tree, &|region| matches!(region, Region::Guard { .. }))
                .expect("the synchronized statement is one guard");
            let Region::Guard { plan, .. } = guard else {
                unreachable!("found by the match above")
            };
            let crate::guard::Shape::Monitor {
                enter_bci,
                normal_exit_bci,
                ..
            } = plan.shape()
            else {
                panic!("the statement is the monitor shape");
            };
            // Pairing, at the instruction level: the enter the header reads the lock from is a
            // `monitorenter`, and the normal exit the statement writes is its `monitorexit`.
            assert!(
                matches!(
                    operations.get(*enter_bci),
                    Some(Operation::Monitor { enter: true })
                ),
                "BCI {enter_bci} is the monitor's enter"
            );
            assert!(
                matches!(
                    operations.get(*normal_exit_bci),
                    Some(Operation::Monitor { enter: false })
                ),
                "BCI {normal_exit_bci} is the statement's normal exit"
            );

            // A `synchronized` statement binds no catch clause. An enclosing clause would make the
            // combination decisions treat it as a protected range it never was.
            assert!(
                guard.exception_clauses().is_empty(),
                "a monitor statement states no exception clause of its own"
            );

            // Return coverage: the body's `return` is a terminal, and the handler's rethrow is the
            // other. The monitor's own cleanup writes no statement, so neither may be dropped.
            let summary = RegionSummary::derive(guard, ssa, operations, &mut budget)
                .expect("the guard candidate derives within an ample budget");
            assert_eq!(
                terminals_of(&summary),
                vec![(0, RegionTerminalKind::Return), (11, RegionTerminalKind::Throw)],
                "the return inside the braces and the handler's rethrow both stay recorded"
            );
        },
    );
}

#[test]
fn a43_the_direct_twr_clause_elides_its_cleanup_and_owns_every_elided_instruction() {
    with_frozen_method_ir(
        RESOURCE_GUARD,
        b"two",
        b"()Ljava/lang/String;",
        |canonical, _view, ssa, operations, _code, recovered| {
            assert!(recovered.fallbacks().is_empty(), "{:?}", recovered.fallbacks());
            let mut budget = ample();
            let mismatches =
                reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                    .expect("the reconciliation completes within an ample budget");
            assert!(mismatches.is_empty(), "{mismatches:?}");

            let tree = Region::Sequence {
                regions: recovered.regions.clone(),
            };
            let guard = find_region(&tree, &|region| matches!(region, Region::Guard { .. }))
                .expect("the try-with-resources statement is one guard");
            let summary = RegionSummary::derive(guard, ssa, operations, &mut budget)
                .expect("the guard candidate derives within an ample budget");

            // Cleanup: the header reconstructs the closes the source never wrote. They are effects
            // performed implicitly by being what the statement is, so they must be stated — and
            // every one of them must be an instruction the statement actually owns, or the
            // reconciliation above would have said so.
            assert!(
                !summary.effects.elided_cleanup_bcis.is_empty(),
                "a resources statement elides its physical cleanup"
            );
            // Cleanup order: the effect list is the physical order the closes run in. Reversing it
            // would present a close sequence the class file does not have.
            let mut ascending = summary.effects.elided_cleanup_bcis.clone();
            ascending.sort_unstable();
            assert_eq!(
                summary.effects.elided_cleanup_bcis, ascending,
                "the elided cleanup is stated in the order it runs"
            );

            // Return/throw coverage: the body's return is a terminal, and the two cleanup handlers
            // rethrow. Losing either rethrow would silently claim an exceptional path terminates
            // nowhere.
            let kinds: Vec<RegionTerminalKind> = terminals_of(&summary)
                .into_iter()
                .map(|(_, kind)| kind)
                .collect();
            assert_eq!(
                kinds,
                vec![
                    RegionTerminalKind::Return,
                    RegionTerminalKind::Throw,
                    RegionTerminalKind::Throw,
                ],
                "the body's return and both cleanup rethrows stay recorded"
            );
        },
    );
}

/// The shared catch-all `finally` the S4 regression surfaced on (`p3_shared_join_finally`'s fixed
/// fixture): one guard whose protected body is an ordinary `try`, with the join's `return` in a
/// sibling candidate after it.
const SHARED_JOIN_FINALLY: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoin.class"
);

#[test]
fn a43_the_shared_join_finally_keeps_its_boundary_and_reads_one_clause() {
    with_frozen_method_ir(
        SHARED_JOIN_FINALLY,
        b"test",
        b"(Ljava/lang/Object;)Z",
        |canonical, _view, ssa, operations, _code, recovered| {
            assert!(recovered.fallbacks().is_empty(), "{:?}", recovered.fallbacks());
            let mut budget = ample();
            let mismatches =
                reconcile_method_summaries(recovered, canonical, ssa, operations, &mut budget)
                    .expect("the reconciliation completes within an ample budget");
            assert!(mismatches.is_empty(), "{mismatches:?}");

            // Two candidates: the statement, then the run that follows it at its stated exit.
            assert_eq!(recovered.summaries.len(), 2, "the shape walks out as two candidates");
            let statement = &recovered.summaries[0];
            let tail = &recovered.summaries[1];
            assert_eq!(
                statement.owned.iter().map(|b| b.bci()).collect::<Vec<_>>(),
                vec![0, 18, 31],
                "the statement owns its header, the clause handler and the cleanup"
            );
            assert_eq!(
                statement.exits.iter().map(|b| b.bci()).collect::<Vec<_>>(),
                vec![39],
                "the statement's normal continuation is the join it proves"
            );
            // The boundary is stated once, from the clause the tree really binds. `Guard` states no
            // clause of its own here — the clause is the child `Try`'s — so a reader that stopped at
            // the guard would lose it. This pins that the two readings stay connected.
            assert!(
                find_region(
                    &Region::Sequence { regions: recovered.regions.clone() },
                    &|region| matches!(region, Region::Try { .. })
                )
                .is_some_and(|try_region| !try_region.exception_clauses().is_empty()),
                "the protected body's clause is stated by the try that binds it"
            );
            assert_eq!(statement.exceptions.len(), 1, "one clause reaches the boundary");
            assert_eq!(statement.exceptions[0].handler.bci(), 18);
            assert_eq!(statement.exceptions[0].parameter, 2);

            // The join's own `return` stays a terminal of the run after the statement, and the
            // cleanup's rethrow stays the statement's. Neither moved.
            assert_eq!(
                terminals_of(&tail),
                vec![(39, RegionTerminalKind::Return)],
                "the join return is the tail candidate's terminal"
            );
            assert_eq!(
                terminals_of(statement),
                vec![(31, RegionTerminalKind::Throw)],
                "the cleanup's rethrow is the statement's terminal"
            );
        },
    );
}
