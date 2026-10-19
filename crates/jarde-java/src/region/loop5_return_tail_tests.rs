use super::*;

const LOOP5_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-named-catch-loop-return-tails/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop5$TestCls.class",
);
const LOOP5_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-named-catch-loop-return-tails/original/no-debug/classes/jadx/tests/integration/loops/TestBreakInLoop5$TestCls.class",
);

fn loop5_frozen_parts(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
    code: &MethodCodeFacts,
    budget: &mut Budget,
) -> (
    CanonicalBlockId,
    crate::guard::Catches,
    Vec<CanonicalBlockId>,
    Vec<CanonicalBlockId>,
    CanonicalBlockId,
    Vec<CanonicalBlockId>,
    BTreeSet<usize>,
) {
    let start = canonical
        .blocks()
        .iter()
        .find(|block| block.id().bci() == 0)
        .expect("Loop5 try entry exists")
        .id()
        .clone();
    let shape = crate::guard::catches(
        canonical,
        view,
        ssa,
        operations,
        &code.exception_handlers,
        None,
        None,
        &crate::pass::JAVA_8,
        &start,
        budget,
    )
    .expect("Loop5 named catch facts complete")
    .expect("Loop5 has a named catch");
    let handler = shape.sites[0].handler.clone();
    let tail = canonical
        .blocks()
        .iter()
        .find(|block| block.id().bci() == 106)
        .expect("Loop5 returned expression block exists")
        .id()
        .clone();
    let prefix: Vec<_> = canonical
        .blocks()
        .iter()
        .filter(|block| block.id().bci() == 0)
        .map(|block| block.id().clone())
        .collect();
    let loop_blocks: Vec<_> = canonical
        .blocks()
        .iter()
        .filter(|block| block.id().bci() != 0 && block.id() != &tail && block.id() != &handler)
        .map(|block| block.id().clone())
        .collect();
    let mut owners = prefix.clone();
    owners.extend(loop_blocks.iter().cloned());
    owners.push(tail.clone());
    let visited = owners
        .iter()
        .map(|block| {
            view.index_of(block)
                .expect("candidate owner is in normal flow")
        })
        .collect();
    (start, shape, prefix, loop_blocks, tail, owners, visited)
}

fn run_loop5_return_tail_certificate(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
    code: &MethodCodeFacts,
    start: &CanonicalBlockId,
    shape: &crate::guard::Catches,
    prefix: &[CanonicalBlockId],
    loop_blocks: &[CanonicalBlockId],
    tail: &CanonicalBlockId,
    owners: &[CanonicalBlockId],
    visited: &BTreeSet<usize>,
    budget: &mut Budget,
) -> Result<bool, StopReason> {
    let chains = crate::concat::Plan::empty();
    let sites = crate::init::Sites::empty();
    let start_node = view.index_of(start).expect("try entry is mapped");
    let mut walker = Walker {
        canonical,
        view,
        ssa,
        operations,
        pool: &[],
        chains: &chains,
        sites: &sites,
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
        visited: visited.clone(),
        unclosed_tail_at: Some(777),
        own_monitor: None,
        depth: 7,
    };
    let frame = Frame {
        own_try: Some(start_node),
        named_catch_rows: Some(std::sync::Arc::new(BTreeSet::from([0]))),
        ..Frame::default()
    };
    let before_visited = walker.visited.clone();
    let saved_visited: BTreeSet<_> = prefix
        .iter()
        .map(|block| view.index_of(block).expect("saved prefix is mapped"))
        .collect();
    let before_tail = walker.unclosed_tail_at;
    let before_joins = walker.catch_joins.clone();
    let before_loop_owners = format!("{:?}", walker.loop_catch_terminal_owners);
    let before_fragmented = format!("{:?}", walker.fragmented);
    let before_monitor = format!("{:?}", walker.own_monitor);
    let before_depth = walker.depth;
    let result = walker.certified_try_return_tail(
        tail,
        &canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() == 14)
            .expect("Loop5 loop header exists")
            .id()
            .clone(),
        shape,
        &frame,
        prefix,
        loop_blocks,
        owners,
        &saved_visited,
    );
    assert_eq!(walker.visited, before_visited);
    assert_eq!(walker.unclosed_tail_at, before_tail);
    assert_eq!(walker.catch_joins, before_joins);
    assert_eq!(
        format!("{:?}", walker.loop_catch_terminal_owners),
        before_loop_owners
    );
    assert_eq!(format!("{:?}", walker.fragmented), before_fragmented);
    assert_eq!(format!("{:?}", walker.own_monitor), before_monitor);
    assert_eq!(walker.depth, before_depth);
    result
}

#[derive(Debug, PartialEq, Eq)]
struct Loop5WalkSnapshot {
    visited: BTreeSet<usize>,
    unclosed_tail_at: Option<u32>,
    catch_joins: BTreeSet<(usize, usize)>,
    loop_owners: String,
    fragmented: String,
    own_monitor: String,
    depth: usize,
}

fn loop5_return_tail_body(region: &Region) -> bool {
    fn one_loop_header(region: &Region) -> Option<u32> {
        match region {
            Region::Loop { header, .. } => Some(header.bci()),
            Region::Sequence { regions } if regions.len() == 1 => one_loop_header(&regions[0]),
            _ => None,
        }
    }
    let Region::Sequence { regions } = region else {
        return false;
    };
    if regions.len() != 3
        || !matches!(&regions[0], Region::Straight { blocks } if blocks.iter().map(CanonicalBlockId::bci).collect::<Vec<_>>() == vec![0])
        || one_loop_header(&regions[1]) != Some(14)
        || !matches!(&regions[2], Region::Straight { blocks } if blocks.iter().map(CanonicalBlockId::bci).collect::<Vec<_>>() == vec![106])
    {
        return false;
    }
    true
}

fn run_loop5_try_level(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
    code: &MethodCodeFacts,
    start: &CanonicalBlockId,
    shape: &crate::guard::Catches,
    budget: &mut Budget,
) -> (Result<(), StopReason>, Loop5WalkSnapshot, bool) {
    let chains = crate::concat::Plan::empty();
    let sites = crate::init::Sites::empty();
    let node = view.index_of(start).expect("Loop5 try entry is mapped");
    let mut walker = Walker {
        canonical,
        view,
        ssa,
        operations,
        pool: &[],
        chains: &chains,
        sites: &sites,
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
        depth: 0,
        own_monitor: None,
        unclosed_tail_at: None,
    };
    let result = walker.try_level(start, node, &Frame::default(), shape, None);
    let published = result
        .as_ref()
        .is_ok_and(|(body, _, _, _, _)| loop5_return_tail_body(body));
    let snapshot = Loop5WalkSnapshot {
        visited: walker.visited.clone(),
        unclosed_tail_at: walker.unclosed_tail_at,
        catch_joins: walker.catch_joins.clone(),
        loop_owners: format!("{:?}", walker.loop_catch_terminal_owners),
        fragmented: format!("{:?}", walker.fragmented),
        own_monitor: format!("{:?}", walker.own_monitor),
        depth: walker.depth,
    };
    (result.map(|_| ()), snapshot, published)
}

#[test]
fn loop5_named_break_body_proof_binds_one_row_and_rejects_scope_drift() {
    with_frozen_method_ir(
        LOOP5_DEBUG,
        b"test",
        b"(Ljava/lang/String;)J",
        |canonical, view, ssa, operations, code, _| {
            let start = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 0)
                .unwrap()
                .id()
                .clone();
            let header = view
                .index_of(
                    &canonical
                        .blocks()
                        .iter()
                        .find(|block| block.id().bci() == 14)
                        .unwrap()
                        .id()
                        .clone(),
                )
                .unwrap();
            let blocks = view.loop_entered_at(header).unwrap().blocks().clone();
            let exits = {
                let chains = crate::concat::Plan::empty();
                let sites = crate::init::Sites::empty();
                let mut budget = ample_budget();
                let walker = Walker {
                    canonical,
                    view,
                    ssa,
                    operations,
                    pool: &[],
                    chains: &chains,
                    sites: &sites,
                    code,
                    method_synchronized: Some(false),
                    has_reachable_explicit_monitor: false,
                    return_is_boolean: false,
                    handlers: &code.exception_handlers,
                    profile: &crate::pass::JAVA_8,
                    budget: &mut budget,
                    catch_joins: BTreeSet::new(),
                    loop_catch_terminal_owners: Vec::new(),
                    fragmented: None,
                    excluded_edge_nodes: BTreeSet::new(),
                    visited: BTreeSet::new(),
                    unclosed_tail_at: None,
                    own_monitor: None,
                    depth: 0,
                };
                walker.loop_exit_nodes(&blocks)
            };
            let mut budget = ample_budget();
            let chains = crate::concat::Plan::empty();
            let sites = crate::init::Sites::empty();
            let mut walker = Walker {
                canonical,
                view,
                ssa,
                operations,
                pool: &[],
                chains: &chains,
                sites: &sites,
                code,
                method_synchronized: Some(false),
                has_reachable_explicit_monitor: false,
                return_is_boolean: false,
                handlers: &code.exception_handlers,
                profile: &crate::pass::JAVA_8,
                budget: &mut budget,
                catch_joins: BTreeSet::new(),
                loop_catch_terminal_owners: Vec::new(),
                fragmented: None,
                excluded_edge_nodes: BTreeSet::new(),
                visited: BTreeSet::new(),
                unclosed_tail_at: None,
                own_monitor: None,
                depth: 0,
            };
            let frame = Frame {
                own_try: Some(view.index_of(&start).unwrap()),
                named_catch_rows: Some(std::sync::Arc::new(BTreeSet::from([0]))),
                ..Frame::default()
            };
            walker.excluded_edge_nodes = canonical
                .edges()
                .iter()
                .filter(|edge| {
                    matches!(
                        edge.kind(),
                        CanonicalEdgeKind::Exception { .. } | CanonicalEdgeKind::Call { .. }
                    )
                })
                .flat_map(|edge| [edge.from().clone(), edge.to().clone()])
                .collect();
            let no_named_context = Frame::default();
            let exit_index = view
                .index_of(
                    &canonical
                        .blocks()
                        .iter()
                        .find(|block| block.id().bci() == 106)
                        .unwrap()
                        .id()
                        .clone(),
                )
                .unwrap();
            assert!(
                walker
                    .loop_break_branch_body(
                        header,
                        &blocks,
                        &exits,
                        Some(exit_index),
                        header,
                        &no_named_context,
                    )
                    .expect("missing named scope is a bounded refusal")
                    .is_none()
            );
            let foreign_row = Frame {
                own_try: Some(view.index_of(&start).unwrap()),
                named_catch_rows: Some(std::sync::Arc::new(BTreeSet::from([1]))),
                ..Frame::default()
            };
            assert!(
                walker
                    .loop_break_branch_body(
                        header,
                        &blocks,
                        &exits,
                        Some(exit_index),
                        header,
                        &foreign_row,
                    )
                    .expect("foreign row is a bounded refusal")
                    .is_none()
            );
            let header_id = view.id_of(header).unwrap().clone();
            assert!(
                walker
                    .prove_for_header(&header_id, header, &blocks, &BTreeSet::new())
                    .expect("ordinary for-header proof is bounded")
                    .is_none()
            );
            let proof = walker
                .loop_break_branch_body(
                    header,
                    &blocks,
                    &exits,
                    Some(
                        view.index_of(
                            &canonical
                                .blocks()
                                .iter()
                                .find(|block| block.id().bci() == 106)
                                .unwrap()
                                .id()
                                .clone(),
                        )
                        .unwrap(),
                    ),
                    header,
                    &frame,
                )
                .expect("candidate proof is bounded")
                .expect("single named-row break side body");
            assert_eq!(
                (
                    proof.header,
                    proof.branch,
                    proof.entry,
                    proof.stay,
                    proof.exit
                ),
                (
                    header,
                    view.index_of(
                        &canonical
                            .blocks()
                            .iter()
                            .find(|block| block.id().bci() == 39)
                            .unwrap()
                            .id()
                            .clone()
                    )
                    .unwrap(),
                    view.index_of(
                        &canonical
                            .blocks()
                            .iter()
                            .find(|block| block.id().bci() == 46)
                            .unwrap()
                            .id()
                            .clone()
                    )
                    .unwrap(),
                    view.index_of(
                        &canonical
                            .blocks()
                            .iter()
                            .find(|block| block.id().bci() == 88)
                            .unwrap()
                            .id()
                            .clone()
                    )
                    .unwrap(),
                    view.index_of(
                        &canonical
                            .blocks()
                            .iter()
                            .find(|block| block.id().bci() == 106)
                            .unwrap()
                            .id()
                            .clone()
                    )
                    .unwrap()
                )
            );
            let mut join_frame = Frame {
                own_try: Some(view.index_of(&start).unwrap()),
                named_catch_rows: Some(std::sync::Arc::new(BTreeSet::from([0]))),
                ..Frame::default()
            };
            join_frame.loop_targets.push(LoopTarget {
                header,
                exits: exits.clone(),
                break_target: Some(proof.exit),
                continue_target: header,
                break_body: Some(std::sync::Arc::new(proof.clone())),
                terminal_returns: std::sync::Arc::new(BTreeSet::new()),
                retry_handler: None,
            });
            let guard23 = view
                .index_of(
                    &canonical
                        .blocks()
                        .iter()
                        .find(|block| block.id().bci() == 23)
                        .unwrap()
                        .id()
                        .clone(),
                )
                .unwrap();
            let arms = view.successors(guard23);
            assert_eq!(
                walker.loop_break_body_local_join(
                    guard23,
                    Some(arms[0]),
                    Some(arms[1]),
                    &join_frame
                ),
                Some(proof.stay)
            );
            assert_eq!(
                walker.loop_break_body_local_join(
                    proof.branch,
                    Some(proof.stay),
                    Some(proof.entry),
                    &join_frame
                ),
                Some(proof.stay)
            );
            let mut scoped: BTreeSet<_> = blocks
                .iter()
                .filter_map(|node| view.id_of(*node).cloned())
                .collect();
            scoped.extend(
                proof
                    .owners
                    .iter()
                    .filter_map(|node| view.id_of(*node).cloned()),
            );
            scoped.insert(view.id_of(proof.exit).unwrap().clone());
            scoped.insert(start.clone());
            assert!(
                walker
                    .named_loop_scope_edges(&frame, &scoped, 39)
                    .expect("row edge proof completes")
            );
            let throwing_owner = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 60)
                .unwrap()
                .id()
                .clone();
            let mut drifted = scoped.clone();
            drifted.remove(&throwing_owner);
            let mut ample = ample_budget();
            walker.budget = &mut ample;
            assert!(
                !walker
                    .named_loop_scope_edges(&frame, &drifted, 39)
                    .expect("omitted exceptional owner is a bounded refusal")
            );
            let mut wrong_code = code.clone();
            wrong_code.exception_handlers[0].catch_type_index = Some(55);
            let mut row_budget = ample_budget();
            let mut row_walker = Walker {
                canonical,
                view,
                ssa,
                operations,
                pool: &[],
                chains: &chains,
                sites: &sites,
                code: &wrong_code,
                method_synchronized: Some(false),
                has_reachable_explicit_monitor: false,
                return_is_boolean: false,
                handlers: &wrong_code.exception_handlers,
                profile: &crate::pass::JAVA_8,
                budget: &mut row_budget,
                catch_joins: BTreeSet::new(),
                loop_catch_terminal_owners: Vec::new(),
                fragmented: None,
                excluded_edge_nodes: BTreeSet::new(),
                visited: BTreeSet::new(),
                unclosed_tail_at: None,
                own_monitor: None,
                depth: 0,
            };
            assert!(
                !row_walker
                    .named_loop_row_matches(&frame, 39)
                    .expect("row drift is a bounded refusal")
            );
        },
    );
}

#[test]
fn loop5_return_tail_certificate_accepts_the_frozen_boundary_return() {
    for class in [LOOP5_DEBUG, LOOP5_NO_DEBUG] {
        with_frozen_method_ir(
            class,
            b"test",
            b"(Ljava/lang/String;)J",
            |canonical, view, ssa, operations, code, _recovered| {
                let mut shape_budget = ample_budget();
                let (start, shape, prefix, loop_blocks, tail, owners, visited) =
                    loop5_frozen_parts(canonical, view, ssa, operations, code, &mut shape_budget);
                assert_eq!(code.exception_handler_count as usize, 1);
                assert_eq!(
                    (
                        code.exception_handlers[0].ordinal,
                        code.exception_handlers[0].start_bci,
                        code.exception_handlers[0].end_bci,
                        code.exception_handlers[0].handler_bci,
                        code.exception_handlers[0].catch_type_index
                    ),
                    (0, 0, 123, 124, Some(54))
                );
                assert_eq!(shape.join, None);
                let row = &canonical.handler_rows()[0];
                let edge_sources: BTreeSet<_> = canonical
                    .edges()
                    .iter()
                    .filter_map(|edge| match edge.kind() {
                        CanonicalEdgeKind::Exception { handler_ordinal: 0 } => {
                            Some(edge.from().bci())
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(edge_sources, BTreeSet::from([0, 14, 23, 60, 81, 88, 106]));
                assert_eq!(
                    row.protected()
                        .iter()
                        .map(CanonicalBlockId::bci)
                        .collect::<BTreeSet<_>>(),
                    edge_sources,
                );
                let candidate_owners: BTreeSet<_> =
                    owners.iter().map(CanonicalBlockId::bci).collect();
                let physical_range_owners: BTreeSet<_> = canonical
                    .blocks()
                    .iter()
                    .filter(|block| {
                        ssa.block(block.id()).is_some_and(|ssa_block| {
                            ssa_block
                                .instructions()
                                .iter()
                                .any(|instruction| instruction.bci() < 123)
                        })
                    })
                    .map(|block| block.id().bci())
                    .collect();
                assert_eq!(candidate_owners, physical_range_owners);
                assert!(edge_sources.is_subset(&candidate_owners));
                assert_ne!(edge_sources, candidate_owners);
                // The entry prefix was already visited by try_level before this candidate is
                // checkpointed. This isolated certificate checks the resulting real delta.
                let mut budget = ample_budget();
                assert!(
                    run_loop5_return_tail_certificate(
                        canonical,
                        view,
                        ssa,
                        operations,
                        code,
                        &start,
                        &shape,
                        &prefix,
                        &loop_blocks,
                        &tail,
                        &owners,
                        &visited,
                        &mut budget,
                    )
                    .expect("the isolated boundary proof completes")
                );
            },
        );
    }
}

#[test]
fn loop5_return_tail_certificate_refuses_drift_duplicate_owners_and_stops_cleanly() {
    use jarde_reader::budget::{CancellationToken, Limits};

    with_frozen_method_ir(
        LOOP5_DEBUG,
        b"test",
        b"(Ljava/lang/String;)J",
        |canonical, view, ssa, operations, code, _| {
            let mut shape_budget = ample_budget();
            let (start, shape, prefix, loop_blocks, tail, owners, visited) =
                loop5_frozen_parts(canonical, view, ssa, operations, code, &mut shape_budget);
            let refuse = |changed_code: &MethodCodeFacts,
                          changed_shape: &crate::guard::Catches,
                          changed_owners: &[CanonicalBlockId],
                          changed_visited: &BTreeSet<usize>| {
                let mut budget = ample_budget();
                assert!(
                    !run_loop5_return_tail_certificate(
                        canonical,
                        view,
                        ssa,
                        operations,
                        changed_code,
                        &start,
                        changed_shape,
                        &prefix,
                        &loop_blocks,
                        &tail,
                        changed_owners,
                        changed_visited,
                        &mut budget,
                    )
                    .expect("mutated certificate evidence is a bounded refusal")
                );
            };

            let mut wrong_start = code.clone();
            wrong_start.exception_handlers[0].start_bci = 1;
            refuse(&wrong_start, &shape, &owners, &visited);
            let mut wrong_end = code.clone();
            wrong_end.exception_handlers[0].end_bci = 122;
            refuse(&wrong_end, &shape, &owners, &visited);
            let mut wrong_type = code.clone();
            wrong_type.exception_handlers[0].catch_type_index = Some(55);
            refuse(&wrong_type, &shape, &owners, &visited);
            let mut wrong_handler = code.clone();
            wrong_handler.exception_handlers[0].handler_bci = 106;
            refuse(&wrong_handler, &shape, &owners, &visited);
            let mut extra_row = code.clone();
            let mut foreign = extra_row.exception_handlers[0].clone();
            foreign.ordinal = 1;
            extra_row.exception_handlers.push(foreign);
            extra_row.exception_handler_count = 2;
            refuse(&extra_row, &shape, &owners, &visited);

            let mut wrong_catch = shape.clone();
            wrong_catch.sites[0].types = crate::guard::CatchTypes::Named(vec![55]);
            refuse(code, &wrong_catch, &owners, &visited);
            let mut wrong_handler_shape = shape.clone();
            wrong_handler_shape.sites[0].handler = tail.clone();
            refuse(code, &wrong_handler_shape, &owners, &visited);
            let mut duplicate_owner = owners.clone();
            duplicate_owner.push(tail.clone());
            refuse(code, &shape, &duplicate_owner, &visited);
            let mut missing_owner = visited.clone();
            missing_owner.remove(&view.index_of(&tail).expect("tail is mapped"));
            refuse(code, &shape, &owners, &missing_owner);

            let mut ample = ample_budget();
            let mut baseline_shape_budget = ample_budget();
            let (start, shape, prefix, loop_blocks, tail, owners, visited) = loop5_frozen_parts(
                canonical,
                view,
                ssa,
                operations,
                code,
                &mut baseline_shape_budget,
            );
            assert!(
                run_loop5_return_tail_certificate(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &start,
                    &shape,
                    &prefix,
                    &loop_blocks,
                    &tail,
                    &owners,
                    &visited,
                    &mut ample,
                )
                .expect("complete isolated proof")
            );
            let cost = ample.usage().analysis_steps;
            assert!(cost > 0);
            let ample_limits = ample_budget().limits().clone();
            let mut short = Budget::new(Limits {
                analysis_steps: cost - 1,
                ..ample_limits.clone()
            });
            assert!(matches!(
                run_loop5_return_tail_certificate(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &start,
                    &shape,
                    &prefix,
                    &loop_blocks,
                    &tail,
                    &owners,
                    &visited,
                    &mut short,
                ),
                Err(StopReason::Budget {
                    dimension: CountedBudgetDimension::AnalysisSteps,
                    ..
                })
            ));
            let token = CancellationToken::new();
            token.cancel();
            let mut cancelled = Budget::with_cancellation_token(ample_limits, token);
            assert!(matches!(
                run_loop5_return_tail_certificate(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &start,
                    &shape,
                    &prefix,
                    &loop_blocks,
                    &tail,
                    &owners,
                    &visited,
                    &mut cancelled,
                ),
                Err(StopReason::Cancelled { .. })
            ));
        },
    );
}

/// Exercises the real try_level candidate checkpoint with a forced join refusal.
/// The candidate walks prefix/loop before it discovers the return-tail gate; it must restore
/// tentative owners before returning to ordinary recovery.
#[test]
fn loop5_try_level_candidate_refusal_restores_tentative_owners() {
    with_frozen_method_ir(
        LOOP5_DEBUG,
        b"test",
        b"(Ljava/lang/String;)J",
        |canonical, view, ssa, operations, code, _| {
            let mut shape_budget = ample_budget();
            let (start, mut shape, _prefix, _loop_blocks, tail, _owners, _visited) =
                loop5_frozen_parts(canonical, view, ssa, operations, code, &mut shape_budget);
            // This deliberate non-empty join reaches the candidate's conservative return-tail
            // refusal without changing the fixture or the real reader inputs.
            shape.join = Some(tail.clone());
            let tail_node = view.index_of(&tail).expect("tail is mapped");
            let loop_node = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 14)
                .and_then(|block| view.index_of(block.id()))
                .expect("loop header is mapped");
            let mut budget = ample_budget();
            let (result, snapshot, published) = run_loop5_try_level(
                canonical,
                view,
                ssa,
                operations,
                code,
                &start,
                &shape,
                &mut budget,
            );
            result.expect("forced refusal remains a bounded recovery result");
            assert!(
                !published,
                "forced join must refuse the return-tail candidate"
            );
            assert!(
                !snapshot.visited.contains(&loop_node),
                "tentative loop owners were not rolled back"
            );
            assert!(
                !snapshot.visited.contains(&tail_node),
                "tentative tail owner was not rolled back"
            );
            assert!(snapshot.catch_joins.is_empty());
            assert_eq!(snapshot.loop_owners, "[]");
            assert_eq!(snapshot.fragmented, "None");
            assert_eq!(snapshot.own_monitor, "None");
            assert_eq!(snapshot.depth, 0);
        },
    );
}

/// A one-step-short budget at the actual publication threshold must stop inside the
/// return-tail candidate and restore the exact try_level checkpoint.
#[test]
fn loop5_try_level_late_analysis_stop_restores_candidate_checkpoint() {
    with_frozen_method_ir(
        LOOP5_DEBUG,
        b"test",
        b"(Ljava/lang/String;)J",
        |canonical, view, ssa, operations, code, _| {
            let mut shape_budget = ample_budget();
            let (start, shape, prefix, _loop_blocks, _tail, _owners, _visited) =
                loop5_frozen_parts(canonical, view, ssa, operations, code, &mut shape_budget);
            let ample = ample_budget();
            let ample_limits = ample.limits().clone();
            let mut full_budget = Budget::new(ample_limits.clone());
            let (full_result, full_snapshot, full_published) = run_loop5_try_level(
                canonical,
                view,
                ssa,
                operations,
                code,
                &start,
                &shape,
                &mut full_budget,
            );
            full_result.expect("ample try_level completes");
            assert!(
                full_published,
                "pending Loop4 proof plus return-tail proof must publish"
            );
            let loop_node = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 14)
                .and_then(|block| view.index_of(block.id()))
                .expect("loop header is mapped");
            assert!(full_snapshot.visited.contains(&loop_node));
            let usage = full_budget.usage().analysis_steps;
            let mut low = 0u64;
            let mut high = usage;
            while low < high {
                let mid = low + (high - low) / 2;
                let mut limits = ample_limits.clone();
                limits.analysis_steps = mid;
                let mut budget = Budget::new(limits);
                let (_, snapshot, _) = run_loop5_try_level(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &start,
                    &shape,
                    &mut budget,
                );
                if snapshot.visited.contains(&loop_node) {
                    high = mid;
                } else {
                    low = mid.saturating_add(1);
                }
            }
            assert!(low > 0, "publication must consume analysis budget");
            let mut limits = ample_limits;
            limits.analysis_steps = low - 1;
            let mut short = Budget::new(limits);
            let (stopped, snapshot, published) = run_loop5_try_level(
                canonical, view, ssa, operations, code, &start, &shape, &mut short,
            );
            assert!(!published);
            assert!(matches!(
                stopped,
                Err(StopReason::Budget {
                    dimension: CountedBudgetDimension::AnalysisSteps,
                    at: Some(106),
                    ..
                })
            ));
            let expected_prefix: BTreeSet<_> = prefix
                .iter()
                .map(|block| view.index_of(block).expect("saved prefix is mapped"))
                .collect();
            assert_eq!(snapshot.visited, expected_prefix);
            assert_eq!(snapshot.unclosed_tail_at, None);
            assert!(snapshot.catch_joins.is_empty());
            assert_eq!(snapshot.loop_owners, "[]");
            assert_eq!(snapshot.fragmented, "None");
            assert_eq!(snapshot.own_monitor, "None");
            assert_eq!(snapshot.depth, 0);
        },
    );
}

/// Full recovery acceptance for the frozen named-row Loop5 case.
#[test]
fn loop5_full_try_owns_prefix_loop_and_return_tail_after_loop4_proof() {
    for class in [LOOP5_DEBUG, LOOP5_NO_DEBUG] {
        with_frozen_method_ir(
            class,
            b"test",
            b"(Ljava/lang/String;)J",
            |canonical, _view, _ssa, _operations, _code, recovered| {
                assert!(recovered.is_structured(), "{:#?}", recovered.regions);
                assert_eq!(recovered.claimed.len(), canonical.blocks().len());
                assert_eq!(recovered.blocks, canonical.blocks().len());
                let tries: Vec<_> = recovered
                    .regions
                    .iter()
                    .filter(|region| matches!(region, Region::Try { .. }))
                    .collect();
                let [
                    Region::Try {
                        body,
                        catches,
                        normal_exit_bci,
                        ..
                    },
                ] = tries.as_slice()
                else {
                    panic!(
                        "one ordinary Try owns the frozen Loop5 body: {:#?}",
                        recovered.regions
                    );
                };
                assert_eq!(*normal_exit_bci, None);
                assert_eq!(catches.len(), 1);
                assert_eq!(catches[0].handler().bci(), 124);
                let Region::Sequence { regions } = body.as_ref() else {
                    panic!("the protected body is prefix + loop + tail: {body:#?}");
                };
                assert_eq!(regions.len(), 3);
                assert!(
                    matches!(&regions[0], Region::Straight { blocks } if blocks.iter().map(CanonicalBlockId::bci).collect::<Vec<_>>() == vec![0])
                );
                fn one_loop_header(region: &Region) -> Option<u32> {
                    match region {
                        Region::Loop { header, .. } => Some(header.bci()),
                        Region::Sequence { regions } if regions.len() == 1 => {
                            one_loop_header(&regions[0])
                        }
                        _ => None,
                    }
                }
                assert_eq!(one_loop_header(&regions[1]), Some(14));
                assert!(matches!(&regions[1], Region::Loop {
                    header, form: LoopForm::While, for_header: None, ..
                } if header.bci() == 14));
                let latch88 = canonical
                    .blocks()
                    .iter()
                    .find(|block| block.id().bci() == 88)
                    .unwrap()
                    .id();
                assert!(
                    regions[1].blocks().contains(&latch88),
                    "the effectful unique latch remains in the loop body"
                );
                assert!(
                    matches!(&regions[2], Region::Straight { blocks } if blocks.iter().map(CanonicalBlockId::bci).collect::<Vec<_>>() == vec![106])
                );
                let mut budget = ample_budget();
                let owners =
                    candidate_region_owner_counts(&recovered.regions, &mut budget, true, true)
                        .expect("owner inventory completes")
                        .expect("all regions have enumerable owners");
                assert_eq!(owners.len(), canonical.blocks().len());
                assert!(owners.values().all(|count| *count == 1));
            },
        );
    }
}
