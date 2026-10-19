use super::finite_loop_catch_tests::with_frozen_method_ir;
use super::*;

const DO2_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-post-test-loop-return-leaves/original/debug/classes/jadx/tests/integration/loops/TestDoWhileBreak2$TestCls.class"
);
const DO2_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-post-test-loop-return-leaves/original/no-debug/classes/jadx/tests/integration/loops/TestDoWhileBreak2$TestCls.class"
);

fn do2_test_walker<'a>(
    canonical: &'a CanonicalCfg,
    view: &'a NormalFlowView,
    ssa: &'a SsaTable,
    operations: &'a Operations,
    code: &'a MethodCodeFacts,
    chains: &'a crate::concat::Plan,
    sites: &'a crate::init::Sites,
    budget: &'a mut Budget,
) -> Walker<'a> {
    Walker {
        canonical,
        view,
        ssa,
        operations,
        pool: &[],
        chains,
        sites,
        code,
        method_synchronized: Some(false),
        has_reachable_explicit_monitor: false,
        return_is_boolean: false,
        handlers: &code.exception_handlers,
        profile: &crate::pass::JAVA_8,
        budget,
        catch_joins: BTreeSet::new(),
        loop_catch_terminal_owners: Vec::new(),
        fragmented: None,
        excluded_edge_nodes: BTreeSet::new(),
        visited: BTreeSet::new(),
        unclosed_tail_at: None,
        own_monitor: None,
        depth: 0,
    }
}

fn do2_test_limits(analysis_steps: u64) -> jarde_reader::budget::Limits {
    jarde_reader::budget::Limits {
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
        analysis_steps,
        normalization_clones: 1 << 20,
        nested_depth: 32,
        dependency_depth: 32,
        elapsed_millis: u64::MAX,
    }
}

fn find_do2_loop(regions: &[Region], header_bci: u32) -> Option<&Region> {
    regions
        .iter()
        .find(|region| matches!(region, Region::Loop { header, .. } if header.bci() == header_bci))
}

#[test]
fn do_while_break2_reader_claims_the_proved_return_leaf_and_latch_once() {
    for class_bytes in [DO2_DEBUG, DO2_NO_DEBUG] {
        with_frozen_method_ir(
            class_bytes,
            b"test",
            b"()Ljava/lang/Object;",
            |canonical, view, ssa, operations, _code, recovered| {
                assert!(recovered.fallbacks().is_empty(), "{:#?}", recovered.regions);
                assert_eq!(recovered.claimed.len(), canonical.blocks().len());
                let bcis: BTreeSet<_> = canonical
                    .blocks()
                    .iter()
                    .map(|block| block.id().bci())
                    .collect();
                assert_eq!(bcis, BTreeSet::from([0, 17, 19, 31]));

                let header = canonical
                    .blocks()
                    .iter()
                    .find(|block| block.id().bci() == 0)
                    .expect("real loop header BCI 0")
                    .id();
                let header_node = view.index_of(header).expect("header node");
                assert_eq!(
                    ssa.block(header)
                        .expect("entry body SSA")
                        .instructions()
                        .iter()
                        .map(|instruction| instruction.bci())
                        .collect::<Vec<_>>(),
                    vec![0, 1, 4, 9, 12, 13, 14],
                    "next, cast, local store, and return test stay in bytecode order"
                );
                let natural = view
                    .loop_entered_at(header_node)
                    .expect("actual post-test natural loop");
                assert_eq!(
                    natural
                        .blocks()
                        .iter()
                        .filter_map(|node| view.id_of(*node).map(CanonicalBlockId::bci))
                        .collect::<BTreeSet<_>>(),
                    BTreeSet::from([0, 19])
                );
                assert_eq!(
                    natural
                        .latches()
                        .iter()
                        .filter_map(|node| view.id_of(*node).map(CanonicalBlockId::bci))
                        .collect::<Vec<_>>(),
                    vec![19]
                );
                let latch = canonical
                    .blocks()
                    .iter()
                    .find(|block| block.id().bci() == 19)
                    .expect("post-test latch BCI 19")
                    .id();
                assert_eq!(
                    ssa.block(latch)
                        .expect("latch SSA")
                        .instructions()
                        .iter()
                        .map(|instruction| instruction.bci())
                        .collect::<Vec<_>>(),
                    vec![19, 20, 23, 28],
                    "hasNext remains after the body and before the loop back edge"
                );

                let return_block = canonical
                    .blocks()
                    .iter()
                    .find(|block| block.id().bci() == 17)
                    .expect("early-return leaf BCI 17")
                    .id();
                let return_node = view.index_of(return_block).expect("return leaf node");
                let incoming: Vec<_> = canonical
                    .edges()
                    .iter()
                    .filter(|edge| edge.to() == return_block)
                    .collect();
                assert_eq!(incoming.len(), 1);
                assert_eq!(incoming[0].from(), header);
                assert_eq!(incoming[0].kind(), CanonicalEdgeKind::Normal);
                assert!(
                    canonical
                        .edges()
                        .iter()
                        .all(|edge| edge.from() != return_block)
                );
                assert!(view.successors(return_node).is_empty());
                let return_ir = ssa.block(return_block).expect("return leaf SSA");
                assert!(matches!(
                    return_ir
                        .instructions()
                        .last()
                        .and_then(|instruction| { operations.get(instruction.bci()) }),
                    Some(Operation::Return)
                ));

                let Region::Loop {
                    header: loop_header,
                    tests,
                    form,
                    for_header,
                    body,
                    exit,
                    ..
                } = find_do2_loop(&recovered.regions, 0).expect("loop at BCI 0")
                else {
                    panic!("reader must publish the proved post-test loop");
                };
                assert_eq!(loop_header.bci(), 0);
                assert_eq!(*form, LoopForm::DoWhile);
                assert!(for_header.is_none());
                assert_eq!(tests.len(), 1);
                assert_eq!(tests[0].0.bci(), 19);
                assert_eq!(exit.as_ref().map(CanonicalBlockId::bci), Some(31));
                let body_bcis: Vec<_> = body
                    .iter()
                    .flat_map(Region::blocks)
                    .map(|block| block.bci())
                    .collect();
                assert!(body_bcis.contains(&0));
                assert!(body_bcis.contains(&17));
                assert!(
                    !body_bcis.contains(&19),
                    "the latch is owned by Loop.tests, not duplicated in body"
                );

                let mut owner_counts = BTreeMap::new();
                for region in &recovered.regions {
                    for block in region.blocks() {
                        *owner_counts.entry(block.bci()).or_insert(0usize) += 1;
                    }
                }
                assert_eq!(owner_counts.keys().copied().collect::<BTreeSet<_>>(), bcis);
                assert!(owner_counts.values().all(|count| *count == 1));
            },
        );
    }
}

#[test]
fn do_while_break2_terminal_return_gate_keeps_scope_and_goto_proofs_narrow() {
    with_frozen_method_ir(
        DO2_DEBUG,
        b"test",
        b"()Ljava/lang/Object;",
        |canonical, view, ssa, operations, code, _recovered| {
            let header = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 0)
                .expect("loop header")
                .id();
            let header_node = view.index_of(header).expect("header node");
            let natural = view.loop_entered_at(header_node).expect("natural loop");
            let blocks = natural.blocks().clone();
            let exit = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 31)
                .expect("ordinary exit")
                .id();
            let exit_node = view.index_of(exit).expect("exit node");
            let return_block = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 17)
                .expect("return leaf")
                .id();
            let return_node = view.index_of(return_block).expect("return node");
            let latch_node = *natural.latches().first().expect("one real latch");
            let chains = crate::concat::Plan::empty();
            let sites = crate::init::Sites::empty();
            let mut budget = Budget::new(do2_test_limits(1 << 20));
            let mut walker = do2_test_walker(
                canonical,
                view,
                ssa,
                operations,
                code,
                &chains,
                &sites,
                &mut budget,
            );
            let proved = walker
                .loop_terminal_returns(&blocks, Some(exit_node), &Frame::default())
                .expect("original return certificate completes");
            assert_eq!(
                proved
                    .iter()
                    .filter_map(|node| view.id_of(*node).map(CanonicalBlockId::bci))
                    .collect::<Vec<_>>(),
                vec![17]
            );
            let wrong_exit = walker
                .loop_terminal_returns(&blocks, Some(return_node), &Frame::default())
                .expect("wrong normal exit is a bounded refusal");
            assert!(
                !wrong_exit.contains(&return_node),
                "asking for the return leaf as the normal exit must not certify that same leaf"
            );
            assert!(
                wrong_exit
                    .iter()
                    .any(|node| view.id_of(*node).is_some_and(|id| id.bci() == 31)),
                "the other terminal return can still be reported relative to the wrong exit"
            );
            let parent_scope = Frame {
                scope: Some(blocks.clone()),
                ..Frame::default()
            };
            let out_of_scope = walker
                .loop_terminal_returns(&blocks, Some(exit_node), &parent_scope)
                .expect("parent scope check completes");
            assert!(out_of_scope.is_empty());
            assert!(!walker.loop_terminal_return_edge_candidate(
                header_node,
                return_node,
                exit_node,
                header_node,
                latch_node,
                &blocks,
                &parent_scope,
            ));
            assert!(!walker.loop_terminal_return_edge_candidate(
                header_node,
                exit_node,
                exit_node,
                header_node,
                latch_node,
                &blocks,
                &Frame::default(),
            ));
            assert!(!exact_normal_predecessors(
                &[
                    (CanonicalEdgeKind::Normal, header.clone()),
                    (
                        CanonicalEdgeKind::Normal,
                        view.id_of(latch_node).unwrap().clone()
                    ),
                ],
                &[header.clone()],
            ));
            assert!(!exact_normal_predecessors(
                &[(
                    CanonicalEdgeKind::Exception { handler_ordinal: 0 },
                    header.clone(),
                )],
                &[header.clone()],
            ));
            assert!(
                !walker
                    .post_test_break_prefix_proved(
                        header_node,
                        return_node,
                        exit_node,
                        header_node,
                        latch_node,
                        &blocks,
                        &Frame::default(),
                    )
                    .expect("the original goto-prefix gate completes")
            );
        },
    );
}

#[test]
fn do_while_break2_real_extra_normal_entry_refuses_terminal_return_leaf() {
    // The frozen no-debug class has one `ifne -28` at BCI 28. Redirecting that real latch edge
    // to BCI 17 adds a second normal predecessor to the existing return leaf without changing
    // the leaf's opcode or its frame. This hostile real-input graph must not certify the leaf.
    let mut bytes = DO2_NO_DEBUG.to_vec();
    let transfer = [0x9a, 0xff, 0xe4];
    let positions: Vec<_> = bytes
        .windows(transfer.len())
        .enumerate()
        .filter_map(|(index, window)| (window == transfer).then_some(index))
        .collect();
    assert_eq!(positions.len(), 1, "one frozen BCI 29 ifne transfer");
    bytes[positions[0] + 1] = 0xff;
    bytes[positions[0] + 2] = 0xf5; // BCI 28 + signed offset -11 = BCI 17.

    with_frozen_method_ir(
        &bytes,
        b"test",
        b"()Ljava/lang/Object;",
        |canonical, view, ssa, operations, code, _recovered| {
            let header = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 0)
                .expect("header")
                .id();
            let latch = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 19)
                .expect("latch")
                .id();
            let leaf = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 17)
                .expect("return leaf")
                .id();
            let incoming: Vec<_> = canonical
                .edges()
                .iter()
                .filter(|edge| edge.to() == leaf)
                .collect();
            assert_eq!(
                incoming.len(),
                2,
                "the mutated CFG has two real incoming edges"
            );
            assert!(
                incoming
                    .iter()
                    .all(|edge| edge.kind() == CanonicalEdgeKind::Normal)
            );
            assert!(incoming.iter().any(|edge| edge.from() == header));
            assert!(incoming.iter().any(|edge| edge.from() == latch));
            assert!(matches!(
                ssa.block(leaf)
                    .and_then(|block| block.instructions().last())
                    .and_then(|instruction| operations.get(instruction.bci())),
                Some(Operation::Return)
            ));

            let header_node = view.index_of(header).expect("header node");
            let latch_node = view.index_of(latch).expect("latch node");
            let leaf_node = view.index_of(leaf).expect("leaf node");
            let exit_node = view
                .index_of(
                    canonical
                        .blocks()
                        .iter()
                        .find(|block| block.id().bci() == 31)
                        .expect("ordinary exit")
                        .id(),
                )
                .expect("exit node");
            let chains = crate::concat::Plan::empty();
            let sites = crate::init::Sites::empty();
            let mut budget = Budget::new(do2_test_limits(1 << 20));
            let mut walker = do2_test_walker(
                canonical,
                view,
                ssa,
                operations,
                code,
                &chains,
                &sites,
                &mut budget,
            );
            let scope = BTreeSet::from([header_node, latch_node]);
            let proved = walker
                .loop_terminal_returns(&scope, Some(exit_node), &Frame::default())
                .expect("hostile actual certificate scan completes");
            assert!(
                !proved.contains(&leaf_node),
                "foreign incoming normal edge rejects leaf: {proved:?}"
            );
        },
    );
}

#[test]
fn do_while_break2_late_budget_stop_restores_new_return_candidate_state() {
    with_frozen_method_ir(
        DO2_DEBUG,
        b"test",
        b"()Ljava/lang/Object;",
        |canonical, view, ssa, operations, code, _recovered| {
            let header = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 0)
                .expect("loop header")
                .id()
                .clone();
            let header_node = view.index_of(&header).expect("header node");
            let chains = crate::concat::Plan::empty();
            let sites = crate::init::Sites::empty();
            let mut full_budget = Budget::new(do2_test_limits(1 << 20));
            let mut full = do2_test_walker(
                canonical,
                view,
                ssa,
                operations,
                code,
                &chains,
                &sites,
                &mut full_budget,
            );
            let full_exit_node = view
                .index_of(
                    canonical
                        .blocks()
                        .iter()
                        .find(|block| block.id().bci() == 31)
                        .expect("ordinary exit")
                        .id(),
                )
                .expect("exit node");
            full.visited.insert(full_exit_node);
            full.unclosed_tail_at = Some(777);
            let full_run = full
                .loop_region(&header, header_node, &Frame::default())
                .expect("complete candidate finishes");
            assert!(matches!(full_run.0.as_slice(), [Region::Loop { .. }]));
            let full_cost = full.budget.usage().analysis_steps;
            assert!(full_cost > 1);
            let expected_visited: BTreeSet<_> = canonical
                .blocks()
                .iter()
                .filter_map(|block| view.index_of(block.id()))
                .collect();
            assert_eq!(
                full.visited, expected_visited,
                "successful candidate publishes every owner"
            );
            assert_eq!(full.unclosed_tail_at, Some(777));
            drop(full);

            let mut proof_budget = Budget::new(do2_test_limits(1 << 20));
            let mut proof = do2_test_walker(
                canonical,
                view,
                ssa,
                operations,
                code,
                &chains,
                &sites,
                &mut proof_budget,
            );
            let header_node_proof = view.index_of(&header).expect("header node");
            let natural = view
                .loop_entered_at(header_node_proof)
                .expect("natural loop")
                .blocks()
                .clone();
            let exit_node_proof = view
                .index_of(
                    canonical
                        .blocks()
                        .iter()
                        .find(|block| block.id().bci() == 31)
                        .expect("ordinary exit")
                        .id(),
                )
                .expect("exit node");
            let proved = proof
                .loop_terminal_returns(&natural, Some(exit_node_proof), &Frame::default())
                .expect("same loop return proof completes");
            assert_eq!(proved.len(), 1);
            let proof_cost = proof.budget.usage().analysis_steps;
            assert!(
                full_cost - 1 > proof_cost,
                "late stop follows leaf certification"
            );
            drop(proof);

            let limit = full_cost - 1;
            let token = jarde_reader::budget::CancellationToken::new();
            token.cancel();
            let mut cancelled_budget =
                Budget::with_cancellation_token(do2_test_limits(1 << 20), token);
            let mut cancelled = do2_test_walker(
                canonical,
                view,
                ssa,
                operations,
                code,
                &chains,
                &sites,
                &mut cancelled_budget,
            );
            let cancel_stop = cancelled
                .loop_region(&header, header_node, &Frame::default())
                .expect_err("pre-cancelled return candidate stops before publication");
            assert!(matches!(cancel_stop, StopReason::Cancelled { .. }));
            assert!(cancelled.visited.is_empty());
            assert_eq!(cancelled.unclosed_tail_at, None);
            assert!(cancelled.catch_joins.is_empty());
            assert!(cancelled.loop_catch_terminal_owners.is_empty());
            assert!(cancelled.fragmented.is_none());
            assert!(cancelled.excluded_edge_nodes.is_empty());
            assert!(cancelled.own_monitor.is_none());
            assert_eq!(cancelled.depth, 0);
            drop(cancelled);

            let mut limited_budget = Budget::new(do2_test_limits(limit));
            let mut limited = do2_test_walker(
                canonical,
                view,
                ssa,
                operations,
                code,
                &chains,
                &sites,
                &mut limited_budget,
            );
            let exit_node = view
                .index_of(
                    canonical
                        .blocks()
                        .iter()
                        .find(|block| block.id().bci() == 31)
                        .expect("ordinary exit")
                        .id(),
                )
                .expect("exit node");
            limited.visited.insert(exit_node);
            limited.unclosed_tail_at = Some(777);
            let before_visited = limited.visited.clone();
            let before_unclosed = limited.unclosed_tail_at;
            let before_catches = limited.catch_joins.clone();
            let before_terminal_owners = limited.loop_catch_terminal_owners.clone();
            let before_fragmented = limited.fragmented.clone();
            let before_excluded = limited.excluded_edge_nodes.clone();
            let before_monitor = limited.own_monitor.clone();
            let before_depth = limited.depth;
            let stop = limited
                .loop_region(&header, header_node, &Frame::default())
                .expect_err("one step below full candidate stops");
            assert!(matches!(
                &stop,
                StopReason::Budget {
                    dimension: CountedBudgetDimension::AnalysisSteps,
                    ..
                }
            ));
            assert!(
                stop.at().is_none_or(|at| at >= 17),
                "the same-state one-under stop must occur after terminal leaf work: {stop:?}"
            );
            assert_eq!(limited.budget.usage().analysis_steps, limit);
            assert_eq!(limited.visited, before_visited);
            assert_eq!(limited.unclosed_tail_at, before_unclosed);
            assert_eq!(limited.catch_joins, before_catches);
            assert_eq!(
                limited.loop_catch_terminal_owners.len(),
                before_terminal_owners.len()
            );
            assert_eq!(limited.fragmented.is_some(), before_fragmented.is_some());
            assert_eq!(limited.excluded_edge_nodes, before_excluded);
            assert_eq!(
                limited
                    .own_monitor
                    .as_ref()
                    .map(|monitor| monitor.owned.clone()),
                before_monitor.as_ref().map(|monitor| monitor.owned.clone())
            );
            assert_eq!(limited.depth, before_depth);
        },
    );
}
