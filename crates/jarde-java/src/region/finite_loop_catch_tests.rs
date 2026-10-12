use super::*;

const DV: &[u8] =
    include_bytes!("../../../../tests/fixtures/java8/loop-body-catch-patrol/javac8/DV.class");
const DF: &[u8] =
    include_bytes!("../../../../tests/fixtures/java8/loop-body-catch-patrol/javac8/DF.class");

fn ample_budget() -> Budget {
    Budget::new(jarde_reader::budget::Limits {
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
        dependency_depth: 32,
        elapsed_millis: u64::MAX,
    })
}

fn with_frozen_method(
    class_bytes: &[u8],
    method_name: &[u8],
    descriptor: &[u8],
    check: impl FnOnce(
        &CanonicalCfg,
        &NormalFlowView,
        &SsaTable,
        &Operations,
        &MethodCodeFacts,
        &Recovered,
    ),
) {
    use jarde_jvm::engine::analyze_method_ir;
    use jarde_jvm::environment::ResolutionEnvironment;
    use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
    use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
        MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
        RuntimeView,
    };

    let mut reader_budget = ample_budget();
    let snapshot = ArtifactSnapshot::open(
        ArtifactInput::bytes(class_bytes.to_vec()),
        &mut reader_budget,
    )
    .expect("frozen class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class_bytes).to_hex().to_string()),
            length: class_bytes.len() as u64,
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
    let analysis = analyze_method_ir(
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
                owner: definition,
                name: JvmBytes(method_name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut reader_budget,
    )
    .expect("reader analyzes the frozen method");
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("canonical CFG exists");
    let ssa = ir.ssa().expect("SSA exists");
    let code = ir.code().expect("decoded method facts exist");
    let operations = Operations::of(code, ir.constant_pool());
    let chains = crate::concat::plan_four_conditional_strings(
        ssa,
        canonical,
        &operations,
        &crate::build::FieldCopies::default(),
        &mut reader_budget,
    )
    .expect("existing concat planning completes");
    let view = NormalFlowView::build(canonical, &mut reader_budget)
        .expect("normal-flow projection exists");
    let mut recovery_budget = ample_budget();
    let recovered = recover(
        canonical,
        &view,
        ssa,
        &operations,
        ir.constant_pool(),
        &chains,
        &crate::init::Sites::empty(),
        code,
        Some(false),
        false,
        &crate::pass::JAVA_8,
        &mut recovery_budget,
    )
    .expect("frozen method recovery completes");
    assert!(recovered.fallbacks().is_empty(), "{:#?}", recovered.regions);
    assert_eq!(recovered.claimed.len(), canonical.blocks().len());
    check(canonical, &view, ssa, &operations, code, &recovered);
}

fn assert_finite_owner(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
    code: &MethodCodeFacts,
    header_bci: u32,
    handler_bci: u32,
    join_bci: u32,
    throw_bci: u32,
    update_bci: u32,
    recovered: &Recovered,
) {
    assert!(recovered.fallbacks().is_empty());
    assert_eq!(recovered.claimed.len(), canonical.blocks().len());
    let mut budget = ample_budget();
    let proof = proved_loop_catch_joins(canonical, view, ssa, operations, code, &mut budget)
        .expect("finite loop-catch proof completes");
    let header = canonical
        .blocks()
        .iter()
        .find(|block| block.id().bci() == header_bci)
        .expect("loop header exists");
    let header_node = view.index_of(header.id()).expect("header is mapped");
    let handler = canonical
        .handler_rows()
        .iter()
        .find(|row| row.handler_bci() == handler_bci)
        .and_then(|row| row.handler())
        .expect("handler exists");
    let join = view
        .successors(view.index_of(handler).expect("handler is mapped"))
        .first()
        .copied()
        .expect("handler has its ordinary join");
    assert_eq!(view.id_of(join).expect("join identity").bci(), join_bci);
    assert!(
        proof
            .joins
            .contains(&(view.index_of(handler).expect("handler node"), join))
    );
    let owner = proof
        .terminal_owners
        .iter()
        .find(|owner| {
            owner.header == header_node
                && owner.handler == view.index_of(handler).expect("handler node")
                && owner.join == join
        })
        .expect("same proof publishes the finite terminal owner");
    let throw = canonical
        .throw_sites()
        .iter()
        .find(|site| site.bci() == throw_bci)
        .expect("terminal throw site exists");
    assert!(
        owner.blocks.contains(
            &view
                .index_of(throw.block())
                .expect("throw owner is in normal flow")
        )
    );
    let update = canonical
        .blocks()
        .iter()
        .find(|block| block.id().bci() == update_bci)
        .expect("induction update block exists");
    let update_node = view.index_of(update.id()).expect("update is mapped");
    assert!(!owner.blocks.contains(&update_node));
    assert_eq!(
        owner.row_ordinal,
        code.exception_handlers
            .iter()
            .find(|row| row.handler_bci == handler_bci)
            .expect("physical row matches handler")
            .ordinal
    );
}

#[test]
fn frozen_dv_finite_terminal_owners_match_both_loop_spellings() {
    with_frozen_method(
        DV,
        b"plainFor",
        b"([I)I",
        |canonical, view, ssa, operations, code, recovered| {
            assert_finite_owner(
                canonical, view, ssa, operations, code, 4, 35, 39, 25, 39, recovered,
            );
        },
    );
    with_frozen_method(
        DV,
        b"whileLoop",
        b"([I)I",
        |canonical, view, ssa, operations, code, recovered| {
            assert_finite_owner(
                canonical, view, ssa, operations, code, 4, 35, 39, 25, 39, recovered,
            );
        },
    );
}

#[test]
fn frozen_df_loop_catch_terminal_owner_is_inside_its_row() {
    with_frozen_method(
        DF,
        b"loopCatch",
        b"([I)I",
        |canonical, view, ssa, operations, code, recovered| {
            assert_finite_owner(
                canonical, view, ssa, operations, code, 10, 45, 50, 36, 50, recovered,
            );
            let row = code
                .exception_handlers
                .iter()
                .find(|row| row.handler_bci == 45)
                .expect("the frozen DF row exists");
            assert_eq!((row.start_bci, row.end_bci), (22, 42));
        },
    );
}

#[test]
fn finite_owner_proof_stops_atomically_at_budget_and_cancellation() {
    use jarde_reader::budget::{CancellationToken, Limits};

    with_frozen_method(
        DV,
        b"plainFor",
        b"([I)I",
        |canonical, view, ssa, operations, code, _| {
            let mut complete = ample_budget();
            proved_loop_catch_joins(canonical, view, ssa, operations, code, &mut complete)
                .expect("ample budget completes");
            let used = complete.usage().analysis_steps;
            assert!(used > 0);
            let mut stopped = Budget::new(Limits {
                analysis_steps: used - 1,
                elapsed_millis: u64::MAX,
                ..Limits::default()
            });
            assert!(matches!(
                proved_loop_catch_joins(canonical, view, ssa, operations, code, &mut stopped),
                Err(StopReason::Budget {
                    dimension: CountedBudgetDimension::AnalysisSteps,
                    ..
                })
            ));
            let token = CancellationToken::new();
            let mut cancelled = Budget::with_cancellation_token(
                Limits {
                    analysis_steps: u64::MAX,
                    elapsed_millis: u64::MAX,
                    ..Limits::default()
                },
                token.clone(),
            );
            token.cancel();
            assert!(matches!(
                proved_loop_catch_joins(canonical, view, ssa, operations, code, &mut cancelled),
                Err(StopReason::Cancelled { .. })
            ));
        },
    );
}

#[test]
fn finite_terminal_license_rejects_mutated_physical_rows() {
    with_frozen_method(
        DV,
        b"plainFor",
        b"([I)I",
        |canonical, view, ssa, operations, code, _| {
            let owner_count = |changed: MethodCodeFacts| {
                let mut budget = ample_budget();
                proved_loop_catch_joins(canonical, view, ssa, operations, &changed, &mut budget)
                    .expect("mutated physical table is a bounded refusal")
                    .terminal_owners
                    .len()
            };
            let mut wrong_range = code.clone();
            wrong_range.exception_handlers[0].end_bci = 25;
            assert_eq!(
                owner_count(wrong_range),
                0,
                "throw@25 falls outside the changed row"
            );

            let mut wrong_type = code.clone();
            wrong_type.exception_handlers[0].catch_type_index = Some(
                wrong_type.exception_handlers[0]
                    .catch_type_index
                    .expect("named catch")
                    .saturating_add(1),
            );
            assert_eq!(
                owner_count(wrong_type),
                0,
                "Code and canonical catch types disagree"
            );

            let mut shared_handler = code.clone();
            let mut competing = shared_handler.exception_handlers[0].clone();
            competing.ordinal = 99;
            competing.start_bci = 32;
            competing.end_bci = 40;
            shared_handler.exception_handlers.push(competing);
            assert_eq!(
                owner_count(shared_handler),
                0,
                "shared physical handler is refused"
            );

            let mut overlapping = code.clone();
            let mut competitor = overlapping.exception_handlers[0].clone();
            competitor.ordinal = 99;
            competitor.start_bci = 20;
            competitor.end_bci = 24;
            competitor.handler_bci = 60;
            overlapping.exception_handlers.push(competitor);
            assert_eq!(
                owner_count(overlapping),
                0,
                "overlapping physical row is refused"
            );
        },
    );
}

const RETRY_V2_DEBUG_JAVAC8: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-loop-catch-retry/v2-debug-javac8/RetryFixture.class"
);
const RETRY_V2_DEBUG_JAVAC23: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-loop-catch-retry/v2-debug-javac23/RetryFixture.class"
);
const RETRY_V2_NONE_JAVAC8: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-loop-catch-retry/v2-none-javac8/RetryFixture.class"
);
const RETRY_V2_NONE_JAVAC23: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-loop-catch-retry/v2-none-javac23/RetryFixture.class"
);
const RETRY_V3_DEBUG_JAVAC8: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-loop-catch-retry/v3-debug-javac8/RetryFixture.class"
);
const RETRY_V3_DEBUG_JAVAC23: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-loop-catch-retry/v3-debug-javac23/RetryFixture.class"
);
const RETRY_V3_NONE_JAVAC8: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-loop-catch-retry/v3-none-javac8/RetryFixture.class"
);
const RETRY_V3_NONE_JAVAC23: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-loop-catch-retry/v3-none-javac23/RetryFixture.class"
);

fn retry_frozen_classes() -> [&'static [u8]; 8] {
    [
        RETRY_V2_DEBUG_JAVAC8,
        RETRY_V2_DEBUG_JAVAC23,
        RETRY_V2_NONE_JAVAC8,
        RETRY_V2_NONE_JAVAC23,
        RETRY_V3_DEBUG_JAVAC8,
        RETRY_V3_DEBUG_JAVAC23,
        RETRY_V3_NONE_JAVAC8,
        RETRY_V3_NONE_JAVAC23,
    ]
}

fn find_retry_loop(region: &Region, header_bci: u32) -> Option<&Region> {
    match region {
        Region::Loop {
            header,
            form: LoopForm::Endless,
            ..
        } if header.bci() == header_bci => Some(region),
        Region::Sequence { regions } => regions
            .iter()
            .find_map(|region| find_retry_loop(region, header_bci)),
        Region::Loop { body, .. } => body
            .iter()
            .find_map(|region| find_retry_loop(region, header_bci)),
        Region::Try { body, catches, .. } => find_retry_loop(body, header_bci).or_else(|| {
            catches
                .iter()
                .find_map(|clause| find_retry_loop(clause.body(), header_bci))
        }),
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => {
            find_retry_loop(then_arm, header_bci).or_else(|| find_retry_loop(else_arm, header_bci))
        }
        Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => groups
            .iter()
            .find_map(|group| find_retry_loop(&group.arm, header_bci)),
        Region::ShortCircuitValue { tail, .. } => tail
            .iter()
            .find_map(|region| find_retry_loop(region, header_bci)),
        Region::Guard {
            body: Some(body), ..
        } => find_retry_loop(body, header_bci),
        _ => None,
    }
}

fn collect_retry_transfers(
    region: &Region,
    breaks: &mut Vec<(u32, CanonicalBlockId)>,
    continues: &mut Vec<(u32, CanonicalBlockId)>,
) {
    match region {
        Region::LoopBreak {
            source_bci,
            loop_header,
        } => breaks.push((*source_bci, loop_header.clone())),
        Region::LoopContinue {
            source_bci,
            loop_header,
        } => continues.push((*source_bci, loop_header.clone())),
        Region::Sequence { regions } | Region::Loop { body: regions, .. } => {
            for region in regions {
                collect_retry_transfers(region, breaks, continues);
            }
        }
        Region::Try { body, catches, .. } => {
            collect_retry_transfers(body, breaks, continues);
            for clause in catches {
                collect_retry_transfers(clause.body(), breaks, continues);
            }
        }
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => {
            collect_retry_transfers(then_arm, breaks, continues);
            collect_retry_transfers(else_arm, breaks, continues);
        }
        Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
            for group in groups {
                collect_retry_transfers(&group.arm, breaks, continues);
            }
        }
        Region::ShortCircuitValue { tail, .. } => {
            for region in tail {
                collect_retry_transfers(region, breaks, continues);
            }
        }
        Region::Guard {
            body, finally_body, ..
        } => {
            if let Some(body) = body {
                collect_retry_transfers(body, breaks, continues);
            }
            if let Some(body) = finally_body {
                collect_retry_transfers(body, breaks, continues);
            }
        }
        _ => {}
    }
}

fn assert_retry_fixture(
    class: &[u8],
    protected_end: u32,
    handler_bci: u32,
    continue_bci: u32,
    loop_exit_bci: u32,
    caught_join_bci: u32,
) {
    with_frozen_method(
        class,
        b"test",
        b"()I",
        |canonical, view, ssa, operations, code, recovered| {
            assert!(recovered.fallbacks().is_empty());
            assert_eq!(canonical.blocks().len(), 5);
            assert_eq!(recovered.claimed.len(), 5);
            assert_eq!(recovered.claimed.len(), canonical.blocks().len());

            let owners: Vec<_> = recovered
                .regions
                .iter()
                .flat_map(Region::blocks)
                .cloned()
                .collect();
            let unique: BTreeSet<_> = owners.iter().cloned().collect();
            let expected: BTreeSet<_> = canonical
                .blocks()
                .iter()
                .map(|block| block.id().clone())
                .collect();
            assert_eq!(
                owners.len(),
                5,
                "the completed Region tree has five physical owners"
            );
            assert_eq!(
                unique.len(),
                owners.len(),
                "no physical block is owned twice"
            );
            assert_eq!(unique, expected, "every canonical block is owned once");

            let header = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 0)
                .expect("retry loop header block")
                .id();
            let mut guard_budget = ample_budget();
            let shape = crate::guard::catches(
                canonical,
                view,
                ssa,
                operations,
                &code.exception_handlers,
                None,
                None,
                &crate::pass::JAVA_8,
                header,
                &mut guard_budget,
            )
            .expect("the guard catch proof completes")
            .expect("the method begins with its named try/catch");
            assert_eq!(shape.protected_end, protected_end);
            assert_eq!(shape.sites.len(), 1);
            assert_eq!(shape.sites[0].handler.bci(), handler_bci);
            assert_eq!(
                shape.join.as_ref().map(CanonicalBlockId::bci),
                Some(caught_join_bci)
            );

            let recovered_tree = Region::Sequence {
                regions: recovered.regions.clone(),
            };
            let loop_region = find_retry_loop(&recovered_tree, 0)
                .expect("the tree contains an Endless retry loop at header 0");
            let Region::Loop {
                body: loop_body,
                exit,
                ..
            } = loop_region
            else {
                unreachable!()
            };
            assert_eq!(
                exit.as_ref().map(CanonicalBlockId::bci),
                Some(loop_exit_bci)
            );
            let [
                Region::Try {
                    prefix,
                    body,
                    catches,
                    ..
                },
            ] = loop_body.as_slice()
            else {
                panic!("the Endless loop body is exactly the recovered try/catch");
            };
            assert!(prefix.is_empty());
            assert_eq!(catches.len(), 1);
            assert_eq!(catches[0].handler().bci(), handler_bci);

            let mut breaks = Vec::new();
            let mut protected_continues = Vec::new();
            collect_retry_transfers(body, &mut breaks, &mut protected_continues);
            assert_eq!(breaks.len(), 1, "the try's normal exit is one LoopBreak");
            assert!(
                protected_continues.is_empty(),
                "the protected body does not continue the retry loop"
            );
            assert_eq!(breaks[0].0, protected_end);
            assert_eq!(&breaks[0].1, header);
            let header_terminal = ssa
                .block(header)
                .and_then(|block| block.instructions().last())
                .expect("header has its physical terminal");
            assert_eq!(header_terminal.bci(), protected_end);
            assert!(matches!(
                operations.get(protected_end),
                Some(Operation::Transfer)
            ));
            let exit = exit.as_ref().expect("retry loop has a proved exit");
            assert!(
                canonical.edges().iter().any(|edge| edge.from() == header
                    && edge.to() == exit
                    && edge.kind() == CanonicalEdgeKind::Normal),
                "range-end goto has the real normal edge to the loop exit"
            );

            let catch_body = catches[0].body();
            let mut catch_breaks = Vec::new();
            let mut continues = Vec::new();
            collect_retry_transfers(catch_body, &mut catch_breaks, &mut continues);
            assert!(
                catch_breaks.is_empty(),
                "the catch does not break the retry loop"
            );
            assert_eq!(continues.len(), 1, "the catch has one LoopContinue");
            assert_eq!(continues[0].0, continue_bci);
            assert_eq!(&continues[0].1, header);
            let handler = catches[0].handler();
            let handler_terminal = ssa
                .block(handler)
                .and_then(|block| block.instructions().last())
                .expect("handler has its physical terminal");
            assert_eq!(handler_terminal.bci(), continue_bci);
            assert!(matches!(
                operations.get(continue_bci),
                Some(Operation::Transfer)
            ));
            assert!(
                canonical.edges().iter().any(|edge| edge.from() == handler
                    && edge.to() == header
                    && edge.kind() == CanonicalEdgeKind::Normal),
                "catch goto has the real normal back edge to header 0"
            );
        },
    );
}

#[test]
fn retry_try_catch_ownership_is_stable_across_eight_frozen_lanes() {
    for class in [
        RETRY_V2_DEBUG_JAVAC8,
        RETRY_V2_DEBUG_JAVAC23,
        RETRY_V2_NONE_JAVAC8,
        RETRY_V2_NONE_JAVAC23,
    ] {
        assert_retry_fixture(class, 46, 49, 92, 95, 95);
    }
    for class in [
        RETRY_V3_DEBUG_JAVAC8,
        RETRY_V3_DEBUG_JAVAC23,
        RETRY_V3_NONE_JAVAC8,
        RETRY_V3_NONE_JAVAC23,
    ] {
        assert_retry_fixture(class, 26, 29, 52, 55, 55);
    }
}

fn retry_shape_for_test(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
    code: &MethodCodeFacts,
) -> (CanonicalBlockId, crate::guard::Catches) {
    let header = canonical
        .blocks()
        .iter()
        .find(|block| block.id().bci() == 0)
        .expect("frozen retry header exists")
        .id()
        .clone();
    let mut budget = ample_budget();
    let shape = crate::guard::catches(
        canonical,
        view,
        ssa,
        operations,
        &code.exception_handlers,
        None,
        None,
        &crate::pass::JAVA_8,
        &header,
        &mut budget,
    )
    .expect("the real frozen guard facts complete")
    .expect("the real frozen method has one named retry catch");
    (header, shape)
}

fn retry_proof_for_test(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
    code: &MethodCodeFacts,
    handlers: &[jarde_reader::classfile::ExceptionHandlerFact],
    header: &CanonicalBlockId,
    shape: &crate::guard::Catches,
    budget: &mut Budget,
) -> Result<Option<RetryLoopFacts>, StopReason> {
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
        method_synchronized: None,
        has_reachable_explicit_monitor: false,
        return_is_boolean: false,
        handlers,
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
    };
    walker.retry_loop_facts(header, shape)
}

fn assert_retry_facts_refuse(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
    code: &MethodCodeFacts,
    handlers: &[jarde_reader::classfile::ExceptionHandlerFact],
    header: &CanonicalBlockId,
    shape: &crate::guard::Catches,
    label: &str,
) {
    let mut budget = ample_budget();
    assert!(
        retry_proof_for_test(
            canonical,
            view,
            ssa,
            operations,
            code,
            handlers,
            header,
            shape,
            &mut budget,
        )
        .expect("proof-level mutation is a bounded refusal")
        .is_none(),
        "retry proof admitted mutated evidence: {label}"
    );
}

fn assert_retry_refusal_preserves_completed_owners(
    canonical: &CanonicalCfg,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
    code: &MethodCodeFacts,
    handlers: Vec<jarde_reader::classfile::ExceptionHandlerFact>,
) {
    let mut changed_code = code.clone();
    changed_code.exception_handler_count = handlers.len() as u32;
    changed_code.exception_handlers = handlers;
    let chains = crate::concat::Plan::empty();
    let sites = crate::init::Sites::empty();
    let mut budget = ample_budget();
    let recovered = recover(
        canonical,
        view,
        ssa,
        operations,
        &[],
        &chains,
        &sites,
        &changed_code,
        Some(false),
        false,
        &crate::pass::JAVA_8,
        &mut budget,
    )
    .expect("inconsistent physical facts produce a bounded recovery result");
    let tree = Region::Sequence {
        regions: recovered.regions.clone(),
    };
    assert!(
        find_retry_loop(&tree, 0).is_none(),
        "the refused physical row cannot publish the retry loop"
    );
    let owners: Vec<_> = recovered
        .regions
        .iter()
        .flat_map(Region::blocks)
        .cloned()
        .collect();
    let unique: BTreeSet<_> = owners.iter().cloned().collect();
    let expected: BTreeSet<_> = canonical
        .blocks()
        .iter()
        .map(|block| block.id().clone())
        .collect();
    assert_eq!(
        owners.len(),
        unique.len(),
        "ordinary overlap gate stays active"
    );
    assert_eq!(
        unique, expected,
        "fallback recovery still accounts for each real block"
    );
    assert!(
        recovered.claimed.is_empty(),
        "a refused retry proof leaves fallback owners without structured claims"
    );
}

#[test]
fn retry_proof_budget_used_minus_one_and_precancel_stop_inside_the_proof() {
    use jarde_reader::budget::{CancellationToken, Limits};

    for class in retry_frozen_classes() {
        with_frozen_method(
            class,
            b"test",
            b"()I",
            |canonical, view, ssa, operations, code, _| {
                let (header, shape) = retry_shape_for_test(canonical, view, ssa, operations, code);
                let mut complete = ample_budget();
                assert!(
                    retry_proof_for_test(
                        canonical,
                        view,
                        ssa,
                        operations,
                        code,
                        &code.exception_handlers,
                        &header,
                        &shape,
                        &mut complete,
                    )
                    .expect("unbounded retry proof completes")
                    .is_some()
                );
                let used = complete.usage().analysis_steps;
                assert!(used > 0);

                let mut exhausted = Budget::new(Limits {
                    analysis_steps: used - 1,
                    elapsed_millis: u64::MAX,
                    ..Limits::default()
                });
                assert!(matches!(
                    retry_proof_for_test(
                        canonical,
                        view,
                        ssa,
                        operations,
                        code,
                        &code.exception_handlers,
                        &header,
                        &shape,
                        &mut exhausted,
                    ),
                    Err(StopReason::Budget {
                        dimension: CountedBudgetDimension::AnalysisSteps,
                        ..
                    })
                ));

                let token = CancellationToken::new();
                token.cancel();
                let mut cancelled = Budget::with_cancellation_token(
                    Limits {
                        analysis_steps: u64::MAX,
                        elapsed_millis: u64::MAX,
                        ..Limits::default()
                    },
                    token,
                );
                assert!(matches!(
                    retry_proof_for_test(
                        canonical,
                        view,
                        ssa,
                        operations,
                        code,
                        &code.exception_handlers,
                        &header,
                        &shape,
                        &mut cancelled,
                    ),
                    Err(StopReason::Cancelled { .. })
                ));
            },
        );
    }
}

#[test]
fn retry_proof_rejects_mutated_physical_and_transfer_evidence() {
    use jarde_reader::classfile::ExceptionHandlerFact;

    for class in retry_frozen_classes() {
        with_frozen_method(
            class,
            b"test",
            b"()I",
            |canonical, view, ssa, operations, code, _| {
                let (header, shape) = retry_shape_for_test(canonical, view, ssa, operations, code);
                let base_rows = code.exception_handlers.clone();
                let row = base_rows.first().expect("one physical named row").clone();
                let handler_bci = shape.sites[0].handler.bci();
                let alternate_block = canonical
                    .blocks()
                    .iter()
                    .map(|block| block.id())
                    .find(|block| {
                        block.bci() != header.bci()
                            && block.bci() != handler_bci
                            && Some(*block) != shape.join.as_ref()
                    })
                    .expect("a real non-header, non-handler block exists")
                    .clone();

                let mut wrong_start_rows = base_rows.clone();
                wrong_start_rows[0].start_bci = row.start_bci.saturating_add(1);
                assert_retry_facts_refuse(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &wrong_start_rows,
                    &header,
                    &shape,
                    "physical named row start disagrees with the protected header",
                );

                let mut wrong_type_rows = base_rows.clone();
                wrong_type_rows[0].catch_type_index = Some(
                    row.catch_type_index
                        .expect("the frozen row is named")
                        .wrapping_add(1),
                );
                assert_retry_facts_refuse(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &wrong_type_rows,
                    &header,
                    &shape,
                    "physical named catch type disagrees with the proved CatchSite",
                );

                let mut wrong_handler_rows = base_rows.clone();
                wrong_handler_rows[0].handler_bci = alternate_block.bci();
                assert_retry_facts_refuse(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &wrong_handler_rows,
                    &header,
                    &shape,
                    "physical handler target disagrees with canonical CatchSite",
                );

                let mut shared_handler_rows = base_rows.clone();
                let mut shared = row.clone();
                shared.ordinal = shared_handler_rows
                    .iter()
                    .map(|candidate| candidate.ordinal)
                    .max()
                    .unwrap_or(0)
                    .saturating_add(1);
                shared.start_bci = row.end_bci;
                shared.end_bci = row.end_bci.saturating_add(1);
                shared_handler_rows.push(shared);
                assert_retry_facts_refuse(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &shared_handler_rows,
                    &header,
                    &shape,
                    "a second physical row shares the handler",
                );

                let mut overlapping_rows = base_rows.clone();
                let mut competing: ExceptionHandlerFact = row.clone();
                competing.ordinal = competing.ordinal.saturating_add(1);
                competing.start_bci = row.start_bci.saturating_add(1);
                competing.end_bci = row.end_bci.saturating_sub(1);
                competing.handler_bci = alternate_block.bci();
                overlapping_rows.push(competing);
                assert_retry_facts_refuse(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &overlapping_rows,
                    &header,
                    &shape,
                    "an overlapping physical handler row competes for protected bytes",
                );

                let mut missing_exit = shape.clone();
                missing_exit.join = None;
                assert_retry_facts_refuse(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &base_rows,
                    &header,
                    &missing_exit,
                    "retry shape has no proved normal exit",
                );

                let mut wrong_exit = shape.clone();
                wrong_exit.join = Some(alternate_block.clone());
                assert_retry_facts_refuse(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &base_rows,
                    &header,
                    &wrong_exit,
                    "guard normal exit disagrees with the header's real normal successor",
                );

                // Preserve the real SSA and canonical CFG, but vary only the decoded opcode fact used
                // by the retry certificate. This deliberately inconsistent proof input must not be
                // treated as a new physical class or a JVM behavior example.
                let changed_transfer_code = jarde_reader::classfile::MethodCodeFacts::from_parts(
                    code.max_stack,
                    code.max_locals,
                    code.code_span.clone(),
                    code.instructions
                        .iter()
                        .cloned()
                        .zip(code.operands().iter().cloned())
                        .map(|(instruction, mut operands)| {
                            if instruction.bci == row.end_bci {
                                operands.effective_opcode = 0x00;
                            }
                            (instruction, operands)
                        })
                        .collect(),
                    base_rows.clone(),
                    code.exception_handler_count,
                    code.execution.clone(),
                    code.stopped_at.clone(),
                    code.debug().clone(),
                );
                let changed_transfer_operations = Operations::of(&changed_transfer_code, &[]);
                assert_retry_facts_refuse(
                    canonical,
                    view,
                    ssa,
                    &changed_transfer_operations,
                    &changed_transfer_code,
                    &base_rows,
                    &header,
                    &shape,
                    "physical range-end opcode no longer states a goto transfer",
                );

                // Keep the real canonical graph and SSA. Change the end in both the physical row and
                // guard fact so the proof reaches its independently read SSA-tail check; the end is a
                // proof-layer mutation, not a new or JVM-verifier-valid class-file claim.
                let last_protected_throw = canonical
                    .throw_sites()
                    .iter()
                    .filter(|site| row.start_bci <= site.bci() && site.bci() < row.end_bci)
                    .map(|site| site.bci())
                    .max()
                    .expect("the real retry row protects a throw site");
                let mut short_rows = base_rows.clone();
                short_rows[0].end_bci = last_protected_throw.saturating_add(1);
                assert!(short_rows[0].end_bci < row.end_bci);
                let mut short_shape = shape.clone();
                short_shape.protected_end = short_rows[0].end_bci;
                assert_retry_facts_refuse(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    &short_rows,
                    &header,
                    &short_shape,
                    "physical range end disagrees with the real SSA terminal transfer BCI",
                );

                // The unmodified real handler row is then passed through full recovery once with one
                // inconsistent physical target, proving the ordinary completed-tree owner gate remains
                // on the path after retry refusal. No JVM behavior is claimed for this mutation.
                assert_retry_refusal_preserves_completed_owners(
                    canonical,
                    view,
                    ssa,
                    operations,
                    code,
                    wrong_handler_rows,
                );
            },
        );
    }
}
