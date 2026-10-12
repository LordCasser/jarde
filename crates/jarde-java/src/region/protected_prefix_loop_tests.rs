use super::*;

use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const FIXTURE: &[u8] = include_bytes!(
    "../../../../tests/fixtures/cf11-structured-list-controls/UpstreamNestedLoops.class"
);
const DESCRIPTOR: &str = "(Ljava/util/List;Ljava/util/List;)V";
const NEGATIVE_FIXTURE: &[u8] =
    include_bytes!("../../../../tests/fixtures/p3-protected-prefix-loop-negatives/Negative.class");
const RESOURCE_GUARD_FIXTURE: &[u8] = include_bytes!(
    "../../../../openspec/evidence/java-syntax-2026-09-30/finallyonce-main-catches/fixture/near/P5TwoResources.class"
);

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 32,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn test_walker<'a>(
    canonical: &'a CanonicalCfg,
    view: &'a NormalFlowView,
    ssa: &'a SsaTable,
    operations: &'a Operations,
    pool: &'a [CpEntryFacts],
    code: &'a MethodCodeFacts,
    chains: &'a crate::concat::Plan,
    sites: &'a crate::init::Sites,
    budget: &'a mut Budget,
) -> Walker<'a> {
    let excluded_edge_nodes = canonical
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
    Walker {
        canonical,
        view,
        ssa,
        operations,
        pool,
        chains,
        sites,
        code,
        method_synchronized: Some(false),
        has_reachable_explicit_monitor: false,
        return_is_boolean: false,
        handlers: &code.exception_handlers,
        profile: &crate::pass::JAVA_8,
        budget,
        catch_joins: BTreeSet::from([(usize::MAX, usize::MAX)]),
        loop_catch_terminal_owners: vec![LoopCatchTerminalOwner {
            header: usize::MAX,
            handler: usize::MAX,
            join: usize::MAX,
            row_ordinal: u32::MAX,
            blocks: BTreeSet::new(),
        }],
        fragmented: None,
        excluded_edge_nodes,
        visited: BTreeSet::new(),
        unclosed_tail_at: Some(u32::MAX),
        own_monitor: None,
        depth: 0,
    }
}

type WalkerState = (
    BTreeSet<usize>,
    Option<u32>,
    BTreeSet<(usize, usize)>,
    Vec<(usize, usize, usize, u32, BTreeSet<usize>)>,
    bool,
    Option<Vec<CanonicalBlockId>>,
    usize,
);

fn walker_state(walker: &Walker<'_>) -> WalkerState {
    (
        walker.visited.clone(),
        walker.unclosed_tail_at,
        walker.catch_joins.clone(),
        walker
            .loop_catch_terminal_owners
            .iter()
            .map(|owner| {
                (
                    owner.header,
                    owner.handler,
                    owner.join,
                    owner.row_ordinal,
                    owner.blocks.clone(),
                )
            })
            .collect(),
        walker.fragmented.is_some(),
        walker
            .own_monitor
            .as_ref()
            .map(|monitor| monitor.owned.clone()),
        walker.depth,
    )
}

fn analyzed() -> jarde_jvm::method_ir::MethodIrAnalysis {
    analyzed_method(FIXTURE, b"handled", DESCRIPTOR.as_bytes())
}

fn analyzed_method(
    class: &[u8],
    method_name: &[u8],
    descriptor: &[u8],
) -> jarde_jvm::method_ir::MethodIrAnalysis {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("the frozen full CF11 class opens");
    let owner = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("fixture length fits"),
        },
        variant: PhysicalVariant::Base,
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    analyze_method_ir(
        &[snapshot.clone()],
        &MethodAnalysisRequest {
            environment: ResolutionEnvironment {
                runtime: RuntimeView {
                    physical: PhysicalView {
                        snapshot: snapshot.id().clone(),
                        scope: PhysicalScope::SnapshotAll,
                    },
                    profile: RuntimeProfile {
                        java_release: 8,
                        multi_release: MultiReleasePolicy::Disabled,
                        layout: LayoutMode::Generic,
                    },
                    load_domain: domain.clone(),
                },
                domains: vec![domain],
                providers: Vec::new(),
            },
            method: PhysicalMethodId {
                owner,
                name: JvmBytes(method_name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("the requested fixture method analyzes")
}

#[test]
fn candidate_named_row_scope_is_bound_to_the_current_real_sites_and_range() {
    let analysis = analyzed();
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("same-run canonical graph");
    let code = ir.code().expect("handled has decoded code");
    let ssa = ir.ssa().expect("same-run SSA");
    let operations = Operations::of(code, ir.constant_pool());
    let start = canonical
        .blocks()
        .first()
        .expect("method has an entry")
        .id();
    let mut flow_budget = Budget::new(limits());
    let view = NormalFlowView::build(canonical, &mut flow_budget)
        .expect("the same graph projects to normal flow");
    let mut shape_budget = Budget::new(limits());
    let shape = crate::guard::catches(
        canonical,
        &view,
        ssa,
        &operations,
        &code.exception_handlers,
        None,
        None,
        &crate::pass::JAVA_8,
        start,
        &mut shape_budget,
    )
    .expect("the table shape reads within budget")
    .expect("the actual handler is one named catch");
    assert_eq!(shape.lead, (0, 0));
    assert_eq!(shape.protected_end, 47);
    assert_eq!(shape.sites.len(), 1);

    let mut budget = Budget::new(limits());
    let rows = named_catch_rows_for_shape(canonical, &code.exception_handlers, &shape, &mut budget)
        .expect("the exact current sites bind within budget")
        .expect("the actual row belongs to these sites");
    assert_eq!(rows.as_ref(), &BTreeSet::from([0_u32]));

    let mut wrong_start = shape.clone();
    wrong_start.lead.1 = 1;
    assert!(
        named_catch_rows_for_shape(
            canonical,
            &code.exception_handlers,
            &wrong_start,
            &mut Budget::new(limits()),
        )
        .expect("range refusal is not a stop")
        .is_none()
    );

    let mut wrong_end = shape.clone();
    wrong_end.protected_end -= 1;
    assert!(
        named_catch_rows_for_shape(
            canonical,
            &code.exception_handlers,
            &wrong_end,
            &mut Budget::new(limits()),
        )
        .expect("range refusal is not a stop")
        .is_none()
    );

    let mut wrong_type = shape.clone();
    let crate::guard::CatchTypes::Named(types) = &mut wrong_type.sites[0].types else {
        unreachable!()
    };
    types[0] = types[0].wrapping_add(1);
    assert!(
        named_catch_rows_for_shape(
            canonical,
            &code.exception_handlers,
            &wrong_type,
            &mut Budget::new(limits()),
        )
        .expect("type refusal is not a stop")
        .is_none()
    );

    let mut wrong_handler = shape.clone();
    wrong_handler.sites[0].handler = canonical
        .blocks()
        .iter()
        .find(|block| block.id().bci() == 59)
        .expect("the real try join exists")
        .id()
        .clone();
    assert!(
        named_catch_rows_for_shape(
            canonical,
            &code.exception_handlers,
            &wrong_handler,
            &mut Budget::new(limits()),
        )
        .expect("handler refusal is not a stop")
        .is_none()
    );

    let mut same_handler_different_range = code.exception_handlers.clone();
    let mut extra = same_handler_different_range[0].clone();
    extra.ordinal = 1;
    extra.start_bci += 1;
    same_handler_different_range.push(extra);
    assert!(
        named_catch_rows_for_shape(
            canonical,
            &same_handler_different_range,
            &shape,
            &mut Budget::new(limits()),
        )
        .expect("overlapping-row refusal is not a stop")
        .is_none()
    );

    let mut catch_all = code.exception_handlers.clone();
    let mut extra = catch_all[0].clone();
    extra.ordinal = 1;
    extra.catch_type_index = None;
    catch_all.push(extra);
    assert!(
        named_catch_rows_for_shape(canonical, &catch_all, &shape, &mut Budget::new(limits()),)
            .expect("catch-all refusal is not a stop")
            .is_none()
    );
}

#[test]
fn candidate_row_binding_stops_without_publishing_a_partial_scope() {
    let analysis = analyzed();
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("same-run canonical graph");
    let code = ir.code().expect("handled has decoded code");
    let ssa = ir.ssa().expect("same-run SSA");
    let operations = Operations::of(code, ir.constant_pool());
    let start = canonical.blocks().first().expect("entry exists").id();
    let mut flow_budget = Budget::new(limits());
    let view = NormalFlowView::build(canonical, &mut flow_budget)
        .expect("the same graph projects to normal flow");
    let mut shape_budget = Budget::new(limits());
    let shape = crate::guard::catches(
        canonical,
        &view,
        ssa,
        &operations,
        &code.exception_handlers,
        None,
        None,
        &crate::pass::JAVA_8,
        start,
        &mut shape_budget,
    )
    .expect("the table shape reads within budget")
    .expect("the actual handler is one named catch");
    let mut complete_budget = Budget::new(limits());
    let complete_rows = named_catch_rows_for_shape(
        canonical,
        &code.exception_handlers,
        &shape,
        &mut complete_budget,
    )
    .expect("the real row binding completes within its fixed budget")
    .expect("the real handler belongs to these sites");
    assert_eq!(complete_rows.as_ref(), &BTreeSet::from([0_u32]));
    let successful_usage = complete_budget.usage().analysis_steps;
    assert!(successful_usage > 0);

    // One step below the measured successful cost reaches Stop during the same paid scan. The
    // helper's Result carries no scope on Err, so no partially matched row set can escape.
    let mut stopped_budget = Budget::new(Limits {
        analysis_steps: successful_usage - 1,
        elapsed_millis: u64::MAX,
        ..limits()
    });
    let stopped = named_catch_rows_for_shape(
        canonical,
        &code.exception_handlers,
        &shape,
        &mut stopped_budget,
    );
    assert!(matches!(
        stopped,
        Err(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));
    assert!(stopped_budget.usage().analysis_steps > 0);
    assert!(stopped_budget.usage().analysis_steps < successful_usage);
}

#[test]
fn late_prefix_loop_child_stop_restores_all_walker_state() {
    let analysis = analyzed();
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("same-run canonical graph");
    let code = ir.code().expect("handled has decoded code");
    let ssa = ir.ssa().expect("same-run SSA");
    let operations = Operations::of(code, ir.constant_pool());
    let start = canonical.blocks().first().expect("entry exists").id();
    let mut flow_budget = Budget::new(limits());
    let view = NormalFlowView::build(canonical, &mut flow_budget)
        .expect("the same graph projects to normal flow");
    let mut shape_budget = Budget::new(limits());
    let shape = crate::guard::catches(
        canonical,
        &view,
        ssa,
        &operations,
        &code.exception_handlers,
        None,
        None,
        &crate::pass::JAVA_8,
        start,
        &mut shape_budget,
    )
    .expect("the table shape reads within budget")
    .expect("the actual handler is one named catch");
    let start_node = view.index_of(start).expect("protected start in view");
    let boundary = shape.join.as_ref().and_then(|join| view.index_of(join));
    let frame = Frame::default();
    let protected_frame = frame.protected(boundary, start_node);
    let chains = crate::concat::Plan::empty();
    let sites = crate::init::Sites::empty();

    // At the maximum outer depth, the prefix completes, but the loop's body descent hits the
    // recursion guard after the loop header has been walked. The speculative checkpoint must
    // restore that already-mutated child state.
    let mut prefix_budget = Budget::new(limits());
    let mut prefix_walker = test_walker(
        canonical,
        &view,
        ssa,
        &operations,
        ir.constant_pool(),
        code,
        &chains,
        &sites,
        &mut prefix_budget,
    );
    prefix_walker.depth = MAX_REGION_DEPTH - 1;
    let (prefix_run, prefix_next) = prefix_walker
        .region_at(start, &protected_frame)
        .expect("the ordinary protected prefix completes at the bound");
    assert!(matches!(prefix_run.as_slice(), [Region::Straight { .. }]));
    assert_eq!(prefix_next.as_ref().map(CanonicalBlockId::bci), Some(7));
    let prefix_state = walker_state(&prefix_walker);
    drop(prefix_walker);

    let mut child_budget = Budget::new(limits());
    let mut child_walker = test_walker(
        canonical,
        &view,
        ssa,
        &operations,
        ir.constant_pool(),
        code,
        &chains,
        &sites,
        &mut child_budget,
    );
    child_walker.depth = MAX_REGION_DEPTH - 1;
    let stop = child_walker
        .try_level(start, start_node, &frame, &shape, None)
        .expect_err("the nested loop body reaches the recursion bound");
    assert!(matches!(
        stop,
        StopReason::Interrupted {
            code: crate::stop::RECURSION_BOUND_CODE,
            at: Some(16),
        }
    ));
    assert_eq!(walker_state(&child_walker), prefix_state);
    let loop_header = canonical
        .blocks()
        .iter()
        .find(|block| block.id().bci() == 7)
        .expect("the actual loop header exists")
        .id();
    assert!(
        !child_walker
            .visited
            .contains(&view.index_of(loop_header).expect("loop header in view"))
    );
}

#[test]
fn handled_keeps_its_prefix_loop_catch_bridge_and_join_owned_once() {
    let analysis = analyzed();
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("same-run canonical graph");
    let view = NormalFlowView::build(canonical, &mut Budget::new(limits()))
        .expect("the same graph projects to normal flow");
    let code = ir.code().expect("handled has decoded code");
    let mut recovery_budget = Budget::new(limits());
    let recovered = crate::region::recover(
        canonical,
        &view,
        ir.ssa().expect("same-run SSA"),
        &Operations::of(code, ir.constant_pool()),
        ir.constant_pool(),
        &crate::concat::Plan::empty(),
        &crate::init::Sites::empty(),
        code,
        Some(false),
        false,
        &crate::pass::JAVA_8,
        &mut recovery_budget,
    )
    .expect("the bounded recovery completes");

    let tries: Vec<_> = recovered
        .regions
        .iter()
        .filter(|region| matches!(region, Region::Try { .. }))
        .collect();
    assert_eq!(tries.len(), 1, "one named protected try: {recovered:?}");
    let Region::Try {
        body,
        catches,
        normal_exit_bci,
        ..
    } = tries[0]
    else {
        unreachable!()
    };
    assert!(
        body.is_structured(),
        "the entire try body is structured: {body:?}"
    );
    assert_eq!(*normal_exit_bci, Some(47));
    assert_eq!(catches.len(), 1);
    assert_eq!(catches[0].handler().bci(), 50);
    let Region::Sequence { regions } = body.as_ref() else {
        panic!("prefix and loop must occupy one ordered Try.body: {body:?}");
    };
    assert!(matches!(
        regions.as_slice(),
        [Region::Straight { .. }, Region::Loop { .. }]
    ));
    assert!(protected_prefix_loop_shape(regions));
    let loop_blocks = candidate_region_blocks(&regions[1], &mut recovery_budget)
        .expect("candidate tree scan stays within budget")
        .expect("the actual loop contains no fallback, nested try or guard");
    assert_eq!(
        loop_blocks
            .iter()
            .map(CanonicalBlockId::bci)
            .collect::<Vec<_>>(),
        [7, 16]
    );
    let mut with_tail_child = regions.clone();
    with_tail_child.push(Region::Straight { blocks: Vec::new() });
    assert!(!protected_prefix_loop_shape(&with_tail_child));
    let mut incomplete_child = regions.clone();
    let Region::Loop {
        body: loop_body, ..
    } = &mut incomplete_child[1]
    else {
        unreachable!()
    };
    loop_body.push(Region::Fallback {
        blocks: Vec::new(),
        reason: FallbackReason::LoopShape { block_bci: 7 },
    });
    assert!(protected_prefix_loop_shape(&incomplete_child));
    assert!(
        candidate_region_blocks(&incomplete_child[1], &mut recovery_budget)
            .expect("the incomplete child is a refusal, not a stop")
            .is_none()
    );
    let mut nested_try_child = regions[1].clone();
    let Region::Loop {
        body: loop_body, ..
    } = &mut nested_try_child
    else {
        unreachable!()
    };
    loop_body.push(Region::Try {
        prefix: Vec::new(),
        lead: (0, 0),
        body: Box::new(Region::Straight { blocks: Vec::new() }),
        catches: Vec::new(),
        normal_exit_bci: None,
        transparent: None,
    });
    assert!(
        candidate_region_blocks(&nested_try_child, &mut recovery_budget)
            .expect("nested try refusal is not a stop")
            .is_none()
    );
    assert_eq!(
        body.blocks()
            .iter()
            .map(|block| block.bci())
            .collect::<Vec<_>>(),
        [0, 7, 16]
    );
    assert_eq!(
        catches[0]
            .body()
            .blocks()
            .iter()
            .map(|block| block.bci())
            .collect::<Vec<_>>(),
        [50]
    );

    let mut owners = std::collections::BTreeMap::<u32, usize>::new();
    for region in &recovered.regions {
        for block in region.blocks() {
            *owners.entry(block.bci()).or_default() += 1;
        }
    }
    for bci in [0, 7, 16, 47, 50, 59] {
        assert_eq!(
            owners.get(&bci),
            Some(&1),
            "physical block {bci}: {owners:?}"
        );
    }
    assert_eq!(code.exception_handlers.len(), 1);
    assert_eq!(code.exception_handlers[0].start_bci, 0);
    assert_eq!(code.exception_handlers[0].end_bci, 47);
    assert_eq!(code.exception_handlers[0].handler_bci, 50);

    let operations = Operations::of(code, ir.constant_pool());
    let mut shape_budget = Budget::new(limits());
    let shape = crate::guard::catches(
        canonical,
        &view,
        ir.ssa().expect("same-run SSA"),
        &operations,
        &code.exception_handlers,
        None,
        None,
        &crate::pass::JAVA_8,
        canonical.blocks().first().expect("entry exists").id(),
        &mut shape_budget,
    )
    .expect("the table shape reads within budget")
    .expect("the actual handler is one named catch");
    let bridge = canonical
        .blocks()
        .iter()
        .find(|block| block.id().bci() == 47)
        .expect("the actual protected-range bridge exists")
        .id();
    let start = canonical.blocks().first().expect("entry exists").id();
    let start_node = view.index_of(start).expect("protected start in view");
    let boundary = shape.join.as_ref().and_then(|join| view.index_of(join));
    let mut frame = Frame::default().protected(boundary, start_node);
    let mut rows_budget = Budget::new(limits());
    frame.named_catch_rows = Some(
        named_catch_rows_for_shape(
            canonical,
            &code.exception_handlers,
            &shape,
            &mut rows_budget,
        )
        .expect("real catch sites bind within budget")
        .expect("the real row belongs to the protected range"),
    );
    let candidate_blocks = body.blocks().into_iter().cloned().collect::<Vec<_>>();
    let chains = crate::concat::Plan::empty();
    let sites = crate::init::Sites::empty();
    let mut scope_budget = Budget::new(limits());
    let mut scope_walker = test_walker(
        canonical,
        &view,
        ir.ssa().expect("same-run SSA"),
        &operations,
        ir.constant_pool(),
        code,
        &chains,
        &sites,
        &mut scope_budget,
    );
    for block in &candidate_blocks {
        scope_walker
            .visited
            .insert(view.index_of(block).expect("candidate block in view"));
    }
    assert!(
        scope_walker
            .candidate_blocks_have_named_catch_scope(&candidate_blocks, &frame)
            .expect("the actual protected edge set is checked")
    );
    assert!(
        !scope_walker
            .candidate_blocks_have_named_catch_scope(&candidate_blocks, &Frame::default())
            .expect("a frame without the named catch scope is an ordinary refusal"),
        "the same real candidate blocks must not acquire named scope from block identity alone"
    );
    let mut duplicate_loop = regions[1].clone();
    let repeated_physical_block = canonical
        .blocks()
        .iter()
        .find(|block| block.id().bci() == 16)
        .expect("the loop body block exists")
        .id()
        .clone();
    let Region::Loop {
        body: loop_body, ..
    } = &mut duplicate_loop
    else {
        unreachable!()
    };
    loop_body.push(Region::Straight {
        blocks: vec![repeated_physical_block.clone()],
    });
    let duplicate_blocks = candidate_region_blocks(&duplicate_loop, &mut recovery_budget)
        .expect("duplicate physical owners are an ordinary refusal")
        .expect("the duplicate remains visible to the ownership gate");
    assert_eq!(
        duplicate_blocks
            .iter()
            .filter(|block| *block == &repeated_physical_block)
            .count(),
        2,
        "the collector preserves repeated owners: {duplicate_blocks:?}"
    );
    assert!(
        !scope_walker
            .candidate_blocks_have_named_catch_scope(&duplicate_blocks, &frame)
            .expect("the physical ownership gate sees duplicate block 16")
    );
    let mut competing_frame = frame.clone();
    competing_frame.named_catch_rows = Some(std::sync::Arc::new(BTreeSet::from([1_u32])));
    let mut competing_scope_budget = Budget::new(limits());
    let mut competing_scope_walker = test_walker(
        canonical,
        &view,
        ir.ssa().expect("same-run SSA"),
        &operations,
        ir.constant_pool(),
        code,
        &chains,
        &sites,
        &mut competing_scope_budget,
    );
    competing_scope_walker.visited = scope_walker.visited.clone();
    assert!(
        !competing_scope_walker
            .candidate_blocks_have_named_catch_scope(&candidate_blocks, &competing_frame)
            .expect("a competing exception ordinal is an ordinary refusal")
    );

    let mut exact_owner_blocks = regions[0].blocks().into_iter().cloned().collect::<Vec<_>>();
    exact_owner_blocks.extend(
        candidate_region_blocks(&regions[1], &mut recovery_budget)
            .expect("the same complete loop candidate scan remains within budget")
            .expect("the real loop has complete physical ownership"),
    );
    assert_eq!(
        exact_owner_blocks,
        body.blocks().into_iter().cloned().collect::<Vec<_>>(),
        "the prefix-plus-loop owner slice is exactly the candidate Try.body ownership"
    );
    let mut bridge_budget = Budget::new(limits());
    let mut bridge_walker = test_walker(
        canonical,
        &view,
        ir.ssa().expect("same-run SSA"),
        &operations,
        ir.constant_pool(),
        code,
        &chains,
        &sites,
        &mut bridge_budget,
    );
    assert!(
        bridge_walker
            .certified_try_exit(bridge, regions, &shape, &frame, Some(&exact_owner_blocks),)
            .expect("the real pure range-end bridge is checked")
    );

    let mut competing_rows = code.exception_handlers.clone();
    let mut competing = competing_rows[0].clone();
    competing.ordinal = 1;
    competing.start_bci = 47;
    competing.end_bci = 48;
    competing_rows.push(competing);
    let mut competing_budget = Budget::new(limits());
    let mut competing_walker = test_walker(
        canonical,
        &view,
        ir.ssa().expect("same-run SSA"),
        &operations,
        ir.constant_pool(),
        code,
        &chains,
        &sites,
        &mut competing_budget,
    );
    competing_walker.handlers = &competing_rows;
    assert!(
        !competing_walker
            .certified_try_exit(bridge, regions, &shape, &frame, Some(&exact_owner_blocks),)
            .expect("the competing physical row is an ordinary refusal")
    );
}

#[test]
fn real_resource_guard_region_is_rejected_by_candidate_collector() {
    let analysis = analyzed_method(RESOURCE_GUARD_FIXTURE, b"two", b"()Ljava/lang/String;");
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("same-run canonical graph");
    let code = ir.code().expect("resource method has decoded code");
    let view = NormalFlowView::build(canonical, &mut Budget::new(limits()))
        .expect("resource method projects to normal flow");
    let mut recovery_budget = Budget::new(limits());
    let recovered = crate::region::recover(
        canonical,
        &view,
        ir.ssa().expect("same-run SSA"),
        &Operations::of(code, ir.constant_pool()),
        ir.constant_pool(),
        &crate::concat::Plan::empty(),
        &crate::init::Sites::empty(),
        code,
        Some(false),
        false,
        &crate::pass::JAVA_8,
        &mut recovery_budget,
    )
    .expect("the existing resource guard recognition fixture recovers");
    let guard = recovered
        .regions
        .iter()
        .find(|region| matches!(region, Region::Guard { .. }))
        .unwrap_or_else(|| panic!("the real TWR method is recognized as a Guard: {recovered:?}"));
    assert!(
        candidate_region_blocks(guard, &mut recovery_budget)
            .expect("guard refusal is a bounded scan")
            .is_none()
    );
}

#[test]
fn effect_suffix_inside_try_refuses_prefix_loop_and_restores_full_state() {
    let analysis = analyzed_method(
        NEGATIVE_FIXTURE,
        b"stateWriteInsideTryAfterLoop",
        b"(Ljava/util/List;)V",
    );
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("same-run canonical graph");
    let code = ir.code().expect("the negative method has decoded code");
    let ssa = ir.ssa().expect("same-run SSA");
    let operations = Operations::of(code, ir.constant_pool());
    let chains = crate::concat::Plan::empty();
    let sites = crate::init::Sites::empty();
    let start = canonical.blocks().first().expect("entry exists").id();
    let mut flow_budget = Budget::new(limits());
    let view = NormalFlowView::build(canonical, &mut flow_budget)
        .expect("the method projects to normal flow");
    let mut shape_budget = Budget::new(limits());
    let shape = crate::guard::catches(
        canonical,
        &view,
        ssa,
        &operations,
        &code.exception_handlers,
        None,
        None,
        &crate::pass::JAVA_8,
        start,
        &mut shape_budget,
    )
    .expect("the named catch shape is bounded")
    .expect("the source has one named catch");
    assert!(shape.inner.is_none());
    assert!(shape.transparent.is_none());
    assert_eq!(shape.sites.len(), 1);
    assert_eq!(shape.protected_end, 34);

    let mut rows_budget = Budget::new(limits());
    let rows = named_catch_rows_for_shape(
        canonical,
        &code.exception_handlers,
        &shape,
        &mut rows_budget,
    )
    .expect("the exact physical row binds within budget")
    .expect("the row belongs to this ordinary try");
    assert_eq!(rows.as_ref(), &BTreeSet::from([0_u32]));
    let start_node = view
        .index_of(start)
        .expect("protected entry is in normal flow");
    let boundary = shape.join.as_ref().and_then(|join| view.index_of(join));
    let root_frame = Frame::default();
    let protected_frame = root_frame.protected(boundary, start_node);

    // Verify every material precondition before treating the final refusal as evidence about
    // the candidate branch. This diagnostic walk is separate from the actual try_level call.
    let mut probe_budget = Budget::new(limits());
    let mut probe = test_walker(
        canonical,
        &view,
        ssa,
        &operations,
        ir.constant_pool(),
        code,
        &chains,
        &sites,
        &mut probe_budget,
    );
    let (prefix_run, prefix_next) = probe
        .region_at(start, &protected_frame)
        .expect("the protected prefix probe completes");
    let [
        Region::Straight {
            blocks: prefix_blocks,
        },
    ] = prefix_run.as_slice()
    else {
        panic!("the actual candidate requires one straight prefix: {prefix_run:?}");
    };
    assert_eq!(
        prefix_blocks
            .iter()
            .map(CanonicalBlockId::bci)
            .collect::<Vec<_>>(),
        [0]
    );
    let loop_header = prefix_next
        .as_ref()
        .expect("the prefix must lead to a loop header");
    assert_eq!(loop_header.bci(), 7);
    assert!(loop_header.bci() < shape.protected_end);
    let loop_node = view
        .index_of(loop_header)
        .expect("loop header is in normal flow");
    assert!(view.is_loop_header(loop_node));

    let mut loop_frame = protected_frame.clone();
    loop_frame.named_catch_rows = Some(rows);
    assert!(
        probe
            .candidate_blocks_have_named_catch_scope(prefix_blocks, &loop_frame)
            .expect("prefix exception edges are checked")
    );
    let prefix_nodes = probe.visited.clone();
    let (loop_run, loop_next) = probe
        .region_at(loop_header, &loop_frame)
        .expect("the actual loop child is bounded");
    let one_loop = match loop_run.as_slice() {
        [Region::Loop { .. }] => true,
        [Region::Sequence { regions }] => matches!(regions.as_slice(), [Region::Loop { .. }]),
        _ => false,
    };
    assert!(
        one_loop,
        "the tested child must reach the loop candidate: {loop_run:?}"
    );
    let child_blocks = candidate_region_blocks(&loop_run[0], probe.budget)
        .expect("the child ownership scan stays within budget")
        .expect("the child has complete structured ownership");
    assert_eq!(
        child_blocks
            .iter()
            .map(CanonicalBlockId::bci)
            .collect::<Vec<_>>(),
        [7, 16]
    );
    assert!(
        probe
            .candidate_blocks_have_named_catch_scope(&child_blocks, &loop_frame)
            .expect("loop exception edges are checked against the same named row")
    );
    let child_nodes = child_blocks
        .iter()
        .map(|block| view.index_of(block).expect("child block is in normal flow"))
        .collect::<BTreeSet<_>>();
    let observed_delta = probe
        .visited
        .difference(&prefix_nodes)
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(observed_delta, child_nodes);
    assert!(child_nodes.is_disjoint(&prefix_nodes));
    let suffix = loop_next
        .as_ref()
        .expect("the visible post-loop effect is a continuation");
    assert_eq!(suffix.bci(), 26);
    assert!(suffix.bci() < shape.protected_end);

    // Build the expected post-rollback state from precisely the successful prefix followed by
    // the catch walk. Candidate-only visited nodes and structural bookkeeping must not survive.
    let mut expected_budget = Budget::new(limits());
    let mut expected = test_walker(
        canonical,
        &view,
        ssa,
        &operations,
        ir.constant_pool(),
        code,
        &chains,
        &sites,
        &mut expected_budget,
    );
    let (expected_prefix, expected_next) = expected
        .region_at(start, &protected_frame)
        .expect("the expected prefix completes");
    assert_eq!(expected_prefix, prefix_run);
    assert_eq!(expected_next, prefix_next);
    let prefix_nodes = prefix_blocks
        .iter()
        .map(|block| {
            view.index_of(block)
                .expect("prefix block is in normal flow")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(walker_state(&expected).0, prefix_nodes);
    let handler = &shape.sites[0].handler;
    let (expected_catch, _) = expected
        .region_at(handler, &root_frame.arm(boundary, None))
        .expect("the expected ordinary catch completes");

    let mut actual_budget = Budget::new(limits());
    let mut actual = test_walker(
        canonical,
        &view,
        ssa,
        &operations,
        ir.constant_pool(),
        code,
        &chains,
        &sites,
        &mut actual_budget,
    );
    let (body, catches, join, _, _) = actual
        .try_level(start, start_node, &root_frame, &shape, None)
        .expect("a complete candidate refusal returns the original Try body");
    assert_eq!(join, shape.join);
    assert_eq!(catches.len(), 1);
    assert_eq!(
        body.blocks()
            .iter()
            .map(|block| block.bci())
            .collect::<Vec<_>>(),
        [0],
        "the suffix and loop must not be transferred into the protected body"
    );
    assert_eq!(
        walker_state(&actual),
        walker_state(&expected),
        "the candidate rollback restores all WalkerState after the catch is walked"
    );
    let actual_catch_blocks = catches[0]
        .body()
        .blocks()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let expected_catch_blocks = expected_catch
        .iter()
        .flat_map(Region::blocks)
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(actual_catch_blocks, expected_catch_blocks);
    assert!(
        actual.budget.usage().analysis_steps > expected.budget.usage().analysis_steps,
        "candidate scans remain charged after an ordinary refusal"
    );
}
