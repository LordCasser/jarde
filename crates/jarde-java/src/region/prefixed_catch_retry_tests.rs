//! Reader-backed ownership checks for the narrowly proved prefixed named-catch retry shape.
use super::*;
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{CancellationToken, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-prefixed-catch-retry-loops/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop3$TestCls.class"
);
const NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-prefixed-catch-retry-loops/original/no-debug/classes/jadx/tests/integration/loops/TestBreakInLoop3$TestCls.class"
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
        class_headers: 100,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 32,
        dependency_depth: 32,
        elapsed_millis: u64::MAX,
    }
}

fn with_recovered(
    class_bytes: &[u8],
    method: &[u8],
    descriptor: &[u8],
    java_release: u16,
    check: impl FnOnce(
        &CanonicalCfg,
        &NormalFlowView,
        &SsaTable,
        &Operations,
        &MethodCodeFacts,
        &[CpEntryFacts],
        &crate::concat::Plan,
        &Recovered,
    ),
) {
    let mut reader_budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(
        ArtifactInput::bytes(class_bytes.to_vec()),
        &mut reader_budget,
    )
    .expect("frozen original class opens");
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
    let recovery_profile = RuntimeProfile {
        java_release,
        multi_release: MultiReleasePolicy::Disabled,
        layout: LayoutMode::Generic,
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
                    profile: recovery_profile.clone(),
                    load_domain: domain.clone(),
                },
                domains: vec![domain],
                providers: Vec::new(),
            },
            method: PhysicalMethodId {
                owner: definition,
                name: JvmBytes(method.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut reader_budget,
    )
    .expect("reader analyzes frozen method");
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
    let mut recovery_budget = Budget::new(limits());
    let sites = crate::init::Sites::empty();
    let recovered = recover(
        canonical,
        &view,
        ssa,
        &operations,
        ir.constant_pool(),
        &chains,
        &sites,
        code,
        Some(false),
        false,
        &crate::pass::JAVA_8,
        &mut recovery_budget,
    )
    .expect("recovery completes");
    check(
        canonical,
        &view,
        ssa,
        &operations,
        code,
        ir.constant_pool(),
        &chains,
        &recovered,
    );
}

fn find_retry_loop<'a>(region: &'a Region, found: &mut Vec<&'a Region>) {
    match region {
        Region::Loop { header, body, .. } => {
            if header.bci() == 11 {
                found.push(region);
            }
            for child in body {
                find_retry_loop(child, found);
            }
        }
        Region::Sequence { regions } => regions
            .iter()
            .for_each(|child| find_retry_loop(child, found)),
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => {
            find_retry_loop(then_arm, found);
            find_retry_loop(else_arm, found);
        }
        Region::Try { body, catches, .. } => {
            find_retry_loop(body, found);
            for clause in catches {
                find_retry_loop(clause.body(), found);
            }
        }
        _ => {}
    }
}

fn assert_prefixed_retry(canonical: &CanonicalCfg, recovered: &Recovered) {
    assert!(recovered.is_structured(), "{:#?}", recovered.regions);
    assert!(recovered.fallbacks().is_empty(), "{:#?}", recovered.regions);
    assert_eq!(recovered.claimed.len(), canonical.blocks().len());

    let mut loops = Vec::new();
    for region in &recovered.regions {
        find_retry_loop(region, &mut loops);
    }
    assert_eq!(
        loops.len(),
        1,
        "expected one proved retry loop: {:#?}",
        recovered.regions
    );
    let Region::Loop { exit, body, .. } = loops[0] else {
        panic!("expected one proved retry loop: {:#?}", recovered.regions);
    };
    assert_eq!(exit.as_ref().map(CanonicalBlockId::bci), Some(82));
    assert_eq!(body.len(), 1);
    let Region::Try {
        prefix,
        lead,
        body: protected,
        catches,
        ..
    } = &body[0]
    else {
        panic!("retry loop body must retain the named Try: {:#?}", body);
    };
    assert!(
        prefix.is_empty(),
        "initial file setup must stay outside the retry loop"
    );
    assert_eq!(*lead, (11, 17));
    assert_eq!(catches.len(), 1);
    assert!(protected.is_structured());

    let all: Vec<_> = recovered.regions.iter().flat_map(Region::blocks).collect();
    for bci in [0, 82] {
        assert_eq!(
            all.iter().filter(|block| block.bci() == bci).count(),
            1,
            "owner BCI {bci}"
        );
    }
    let loop_blocks: Vec<_> = loops[0].blocks();
    assert!(
        !loop_blocks
            .iter()
            .any(|block| matches!(block.bci(), 0 | 82))
    );
}

#[test]
fn debug_and_no_debug_retry_ownership_is_stable_for_java8_and_23_profiles() {
    for bytes in [DEBUG, NO_DEBUG] {
        for release in [8, 23] {
            with_recovered(
                bytes,
                b"writeMore",
                b"(Ljava/lang/String;)V",
                release,
                |canonical, _, _, _, _, _, _, recovered| {
                    assert_prefixed_retry(canonical, recovered);
                },
            );
        }
    }
}

#[test]
fn other_original_methods_remain_structured_in_both_metadata_variants() {
    for bytes in [DEBUG, NO_DEBUG] {
        for (method, descriptor) in [
            (b"<init>".as_slice(), b"()V".as_slice()),
            (b"prt".as_slice(), b"(Ljava/lang/String;)V".as_slice()),
            (b"check".as_slice(), b"()V".as_slice()),
        ] {
            with_recovered(
                bytes,
                method,
                descriptor,
                8,
                |canonical, _, _, _, _, _, _, recovered| {
                    assert!(
                        recovered.is_structured(),
                        "{method:?}: {:#?}",
                        recovered.regions
                    );
                    assert!(
                        recovered.fallbacks().is_empty(),
                        "{method:?}: {:#?}",
                        recovered.regions
                    );
                    assert_eq!(recovered.claimed.len(), canonical.blocks().len());
                },
            );
        }
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
        unclosed_tail_at: None,
        own_monitor: None,
        depth: 0,
    }
}

#[test]
fn exact_retry_proof_rejects_row_site_join_and_entry_drift() {
    with_recovered(
        DEBUG,
        b"writeMore",
        b"(Ljava/lang/String;)V",
        8,
        |canonical, view, ssa, operations, code, pool, chains, _| {
            let start = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 11)
                .unwrap()
                .id();
            let prefix = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 0)
                .unwrap()
                .id()
                .clone();
            let mut shape_budget = Budget::new(limits());
            let shape = crate::guard::catches(
                canonical,
                view,
                ssa,
                operations,
                &code.exception_handlers,
                None,
                None,
                &crate::pass::JAVA_8,
                start,
                &mut shape_budget,
            )
            .expect("catch shape is bounded")
            .expect("frozen method has one named catch");
            let sites = crate::init::Sites::empty();
            let entry_prefix_node = view
                .index_of(&prefix)
                .expect("actual outer entry prefix node");
            let proves = |shape: &crate::guard::Catches, prefix: &[CanonicalBlockId]| {
                let mut budget = Budget::new(limits());
                let mut walker = test_walker(
                    canonical,
                    view,
                    ssa,
                    operations,
                    pool,
                    code,
                    chains,
                    &sites,
                    &mut budget,
                );
                walker.visited.insert(entry_prefix_node);
                walker
                    .prefixed_catch_retry_facts(start, shape, prefix)
                    .expect("proof check completes")
                    .is_some()
            };
            assert!(proves(&shape, std::slice::from_ref(&prefix)));
            assert!(
                !proves(&shape, &[]),
                "retry cannot detach from its entry prefix"
            );
            assert!(
                !proves(&shape, std::slice::from_ref(start)),
                "header cannot impersonate the linear entry prefix"
            );

            let mut wrong_lead = shape.clone();
            wrong_lead.lead.1 -= 1;
            assert!(!proves(&wrong_lead, std::slice::from_ref(&prefix)));
            let mut wrong_end = shape.clone();
            wrong_end.protected_end -= 1;
            assert!(!proves(&wrong_end, std::slice::from_ref(&prefix)));
            let mut wrong_type = shape.clone();
            let crate::guard::CatchTypes::Named(types) = &mut wrong_type.sites[0].types else {
                unreachable!()
            };
            types[0] = types[0].wrapping_add(1);
            assert!(!proves(&wrong_type, std::slice::from_ref(&prefix)));
            let mut wrong_handler = shape.clone();
            wrong_handler.sites[0].handler = prefix.clone();
            assert!(!proves(&wrong_handler, std::slice::from_ref(&prefix)));
            let mut wrong_join = shape.clone();
            wrong_join.join = Some(prefix.clone());
            assert!(!proves(&wrong_join, std::slice::from_ref(&prefix)));
            let mut shared_handler = shape.clone();
            shared_handler.sites.push(shared_handler.sites[0].clone());
            assert!(!proves(&shared_handler, std::slice::from_ref(&prefix)));
        },
    );
}

#[test]
fn exact_normal_catch_dag_rejects_foreign_edges_cycles_and_call_return_crossings() {
    with_recovered(
        DEBUG,
        b"writeMore",
        b"(Ljava/lang/String;)V",
        8,
        |canonical, view, ssa, operations, code, pool, chains, _| {
            let start = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 11)
                .unwrap()
                .id();
            let prefix = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 0)
                .unwrap()
                .id()
                .clone();
            let mut shape_budget = Budget::new(limits());
            let shape = crate::guard::catches(
                canonical,
                view,
                ssa,
                operations,
                &code.exception_handlers,
                None,
                None,
                &crate::pass::JAVA_8,
                start,
                &mut shape_budget,
            )
            .expect("catch shape is bounded")
            .expect("named catch exists");
            let sites = crate::init::Sites::empty();
            let mut budget = Budget::new(limits());
            let mut walker = test_walker(
                canonical,
                view,
                ssa,
                operations,
                pool,
                code,
                chains,
                &sites,
                &mut budget,
            );
            walker.visited.insert(
                view.index_of(&prefix)
                    .expect("actual outer entry prefix node"),
            );
            let proof = walker
                .prefixed_catch_retry_facts(start, &shape, std::slice::from_ref(&prefix))
                .expect("proof completes")
                .expect("closed normal catch DAG is proved");
            let RetryHandlerBranchFacts::RetryOrNormal {
                normal_entry,
                normal_join,
                normal_owners,
                ..
            } = proof.handler_branch.as_ref().expect("branch proof exists")
            else {
                unreachable!()
            };
            let edges: Vec<_> = canonical
                .edges()
                .iter()
                .map(|edge| (edge.kind(), edge.from().clone(), edge.to().clone()))
                .collect();
            assert!(
                loop_break_body_edges_closed(
                    &proof.handler,
                    normal_entry,
                    normal_join,
                    normal_owners,
                    &edges,
                    &mut budget
                )
                .expect("closed edge proof is bounded")
            );
            assert!(
                loop_break_body_acyclic(normal_owners, &edges, &mut budget)
                    .expect("acyclic proof is bounded")
            );

            let foreign = prefix.clone();
            let mut foreign_entry = edges.clone();
            foreign_entry.push((
                CanonicalEdgeKind::Normal,
                foreign.clone(),
                normal_entry.clone(),
            ));
            assert!(
                !loop_break_body_edges_closed(
                    &proof.handler,
                    normal_entry,
                    normal_join,
                    normal_owners,
                    &foreign_entry,
                    &mut budget
                )
                .expect("foreign entry is a refusal")
            );
            let mut foreign_exit = edges.clone();
            foreign_exit.push((
                CanonicalEdgeKind::Normal,
                normal_entry.clone(),
                proof.handler.clone(),
            ));
            assert!(
                !loop_break_body_edges_closed(
                    &proof.handler,
                    normal_entry,
                    normal_join,
                    normal_owners,
                    &foreign_exit,
                    &mut budget
                )
                .expect("foreign exit is a refusal")
            );
            let owner = normal_owners
                .iter()
                .next()
                .expect("normal DAG has owners")
                .clone();
            let mut cycle = edges.clone();
            cycle.push((
                CanonicalEdgeKind::Normal,
                owner.clone(),
                normal_entry.clone(),
            ));
            assert!(
                !loop_break_body_acyclic(normal_owners, &cycle, &mut budget)
                    .expect("cycle is a refusal")
            );
            for kind in [
                CanonicalEdgeKind::Call { call_site: 1 },
                CanonicalEdgeKind::Return { call_site: 1 },
            ] {
                let mut crossed = edges.clone();
                crossed.push((kind, owner.clone(), normal_entry.clone()));
                assert!(
                    !loop_break_body_edges_closed(
                        &proof.handler,
                        normal_entry,
                        normal_join,
                        normal_owners,
                        &crossed,
                        &mut budget
                    )
                    .expect("non-normal crossing is a refusal")
                );
            }
        },
    );
}

#[test]
fn retry_normal_inner_join_accepts_only_the_active_loop_break_tail() {
    with_recovered(
        DEBUG,
        b"writeMore",
        b"(Ljava/lang/String;)V",
        8,
        |canonical, view, ssa, operations, code, pool, chains, _| {
            let start = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 11)
                .expect("real retry header")
                .id();
            let prefix = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 0)
                .expect("real entry prefix")
                .id()
                .clone();
            let prefix_node = view.index_of(&prefix).expect("entry prefix node");
            let mut shape_budget = Budget::new(limits());
            let shape = crate::guard::catches(
                canonical,
                view,
                ssa,
                operations,
                &code.exception_handlers,
                None,
                None,
                &crate::pass::JAVA_8,
                start,
                &mut shape_budget,
            )
            .expect("catch shape is bounded")
            .expect("frozen method has one named catch");
            let sites = crate::init::Sites::empty();
            let mut budget = Budget::new(limits());
            let mut walker = test_walker(
                canonical,
                view,
                ssa,
                operations,
                pool,
                code,
                chains,
                &sites,
                &mut budget,
            );
            walker.visited.insert(prefix_node);
            let proof = walker
                .prefixed_catch_retry_facts(start, &shape, std::slice::from_ref(&prefix))
                .expect("the exact retry proof completes")
                .expect("the frozen retry shape is proved");
            let RetryHandlerBranchFacts::RetryOrNormal {
                branch_bci,
                normal_entry,
                normal_join,
                normal_owners,
                ..
            } = proof
                .handler_branch
                .as_ref()
                .expect("proved handler branch")
            else {
                unreachable!()
            };
            let loop_frame = Frame::default().loop_body(
                &proof.owners,
                proof.exit_node,
                proof.header_node,
                proof.header_node,
                Some(proof.exit_node),
                Some(proof.exit_node),
                BTreeSet::new(),
                &proof.transfer_sources,
                &BTreeSet::new(),
                &BTreeSet::new(),
            );
            let normal_frame = loop_frame.arm(Some(proof.exit_node), Some(*branch_bci));
            let inner_join = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 76)
                .expect("real inner normal join")
                .id()
                .clone();
            let (mut run, next) = walker
                .region_at(normal_entry, &normal_frame)
                .expect("normal branch prefix is readable");
            assert_eq!(next.as_ref(), Some(&inner_join));
            assert!(
                matches!(run.as_slice(), [Region::If { join: Some(join), .. }] if join == &inner_join)
            );
            assert_eq!(
                normal_join.bci(),
                82,
                "the proved Try join follows the inner join"
            );
            assert!(
                walker
                    .continue_inner_join_arm(&mut run, next.as_ref(), &normal_frame)
                    .expect("exact normal completion tail is bounded")
            );
            assert_eq!(
                run.len(),
                3,
                "inner If, its straight join block, and the proved loop exit"
            );
            let Region::Straight { blocks } = &run[1] else {
                panic!("the exact inner join block remains straight");
            };
            assert_eq!(
                blocks.iter().map(CanonicalBlockId::bci).collect::<Vec<_>>(),
                vec![76]
            );
            let Region::LoopBreak {
                source_bci,
                loop_header,
            } = &run[2]
            else {
                panic!("the terminal route is the active retry-loop break");
            };
            assert_eq!(
                *source_bci, 79,
                "the physical terminal instruction owns the transfer"
            );
            assert_eq!(loop_header, start);
            let owned: BTreeSet<_> = run.iter().flat_map(Region::blocks).cloned().collect();
            assert_eq!(
                &owned, normal_owners,
                "the continuation retains precisely the proved normal DAG owners"
            );
            assert_eq!(normal_join, &proof.exit);

            // A mismatched frame boundary cannot authorize the exact physical tail.
            walker.visited = BTreeSet::from([prefix_node]);
            let (mut wrong_boundary_run, wrong_boundary_next) = walker
                .region_at(normal_entry, &normal_frame)
                .expect("the normal branch prefix remains readable");
            let mut wrong_boundary_frame = normal_frame.clone();
            wrong_boundary_frame.boundary = Some(prefix_node);
            let before_boundary_len = wrong_boundary_run.len();
            assert!(
                !walker
                    .continue_inner_join_arm(
                        &mut wrong_boundary_run,
                        wrong_boundary_next.as_ref(),
                        &wrong_boundary_frame,
                    )
                    .expect("wrong-boundary refusal is bounded")
            );
            assert_eq!(wrong_boundary_run.len(), before_boundary_len);

            // Keep the real outer transfer to header 11, but put an unrelated nested target
            // last in the frame. The helper must reject that physical LoopBreak instead of
            // treating the old straight-only tail as a valid substitute.
            walker.visited = BTreeSet::from([prefix_node]);
            let (mut wrong_target_run, wrong_target_next) = walker
                .region_at(normal_entry, &normal_frame)
                .expect("the normal branch prefix remains readable");
            let mut wrong_target_frame = normal_frame.clone();
            let mut fake_nested_target = wrong_target_frame
                .loop_targets
                .last()
                .expect("the proved outer loop target")
                .clone();
            fake_nested_target.header = prefix_node;
            fake_nested_target.break_target = Some(prefix_node);
            fake_nested_target.continue_target = prefix_node;
            fake_nested_target.exits.clear();
            fake_nested_target.break_body = None;
            fake_nested_target.retry_handler = None;
            wrong_target_frame.loop_targets.push(fake_nested_target);
            let before_target_len = wrong_target_run.len();
            assert!(
                !walker
                    .continue_inner_join_arm(
                        &mut wrong_target_run,
                        wrong_target_next.as_ref(),
                        &wrong_target_frame,
                    )
                    .expect("wrong active target refusal is bounded")
            );
            assert_eq!(wrong_target_run.len(), before_target_len);
        },
    );
}

#[test]
fn retry_proof_budget_and_precancellation_leave_walker_state_unpublished() {
    with_recovered(
        DEBUG,
        b"writeMore",
        b"(Ljava/lang/String;)V",
        8,
        |canonical, view, ssa, operations, code, pool, chains, _| {
            let start = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 11)
                .unwrap()
                .id();
            let prefix = canonical
                .blocks()
                .iter()
                .find(|block| block.id().bci() == 0)
                .unwrap()
                .id()
                .clone();
            let mut shape_budget = Budget::new(limits());
            let shape = crate::guard::catches(
                canonical,
                view,
                ssa,
                operations,
                &code.exception_handlers,
                None,
                None,
                &crate::pass::JAVA_8,
                start,
                &mut shape_budget,
            )
            .expect("catch shape is bounded")
            .expect("named catch exists");
            let sites = crate::init::Sites::empty();

            let token = CancellationToken::new();
            token.cancel();
            let mut cancelled_budget = Budget::with_cancellation_token(limits(), token);
            let mut cancelled = test_walker(
                canonical,
                view,
                ssa,
                operations,
                pool,
                code,
                chains,
                &sites,
                &mut cancelled_budget,
            );
            assert!(matches!(
                cancelled.prefixed_catch_retry_facts(start, &shape, std::slice::from_ref(&prefix)),
                Err(StopReason::Cancelled { .. })
            ));
            let prefix_node = view.index_of(&prefix).expect("entry prefix node");
            assert!(
                cancelled.visited.is_empty(),
                "cancellation occurs before the direct proof claims owners"
            );
            assert_eq!(cancelled.unclosed_tail_at, None);

            let mut limited = limits();
            limited.analysis_steps = 128;
            let mut limited_budget = Budget::new(limited);
            let mut stopped = test_walker(
                canonical,
                view,
                ssa,
                operations,
                pool,
                code,
                chains,
                &sites,
                &mut limited_budget,
            );
            stopped.visited.insert(prefix_node);
            let result =
                stopped.prefixed_catch_retry_facts(start, &shape, std::slice::from_ref(&prefix));
            assert!(
                result.is_err(),
                "bounded scan must stop before publishing the proof"
            );
            assert_eq!(stopped.visited, BTreeSet::from([prefix_node]));
            assert_eq!(stopped.unclosed_tail_at, None);
            assert!(
                stopped.budget.usage().analysis_steps > 0,
                "charged work is retained on stop"
            );

            // Exercise the real try consumer with a late stop, after it has claimed child regions.
            // The new prefixed branch must restore the full walker checkpoint while retaining cost.
            let node = view.index_of(start).expect("try header node");
            let frame = Frame::default();
            let mut complete_budget = Budget::new(limits());
            let mut complete = test_walker(
                canonical,
                view,
                ssa,
                operations,
                pool,
                code,
                chains,
                &sites,
                &mut complete_budget,
            );
            complete.visited = BTreeSet::from([prefix_node]);
            let completed = complete
                .try_region(start, node, &frame, std::slice::from_ref(&prefix))
                .expect("unlimited try walk completes")
                .expect("the closed retry candidate is assembled");
            let complete_steps = complete.budget.usage().analysis_steps;
            let body_blocks = completed.0.blocks().len();
            let catch_blocks = completed
                .2
                .iter()
                .map(|catch| catch.body().blocks().len())
                .sum::<usize>();
            let completion_check_steps = body_blocks
                .saturating_add(catch_blocks)
                .saturating_add(complete.visited.len())
                .saturating_add(1); // the exact preclaimed prefix checkpoint
            assert!(
                completion_check_steps > 1,
                "completion check has a measurable final budget boundary"
            );
            let assembly_complete_at = complete_steps.saturating_sub(
                u64::try_from(completion_check_steps).expect("small fixture completion work"),
            );
            let late_limit = complete_steps - 1;
            assert!(
                late_limit >= assembly_complete_at,
                "late stop is after structured try assembly"
            );

            let mut late_limits = limits();
            late_limits.analysis_steps = late_limit;
            let mut late_budget = Budget::new(late_limits);
            let mut late = test_walker(
                canonical,
                view,
                ssa,
                operations,
                pool,
                code,
                chains,
                &sites,
                &mut late_budget,
            );
            late.visited = BTreeSet::from([prefix_node]);
            let before_visited = late.visited.clone();
            let before_tail = late.unclosed_tail_at;
            let before_joins = late.catch_joins.clone();
            let before_owner_fields = late
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
                .collect::<Vec<_>>();
            let before_fragmented = late.fragmented.is_some();
            let before_excluded = late.excluded_edge_nodes.clone();
            let before_monitor = late.own_monitor.is_some();
            let before_depth = late.depth;
            let stopped_late = late.try_region(start, node, &frame, std::slice::from_ref(&prefix));
            assert!(
                matches!(
                    stopped_late,
                    Err(StopReason::Budget {
                        dimension: CountedBudgetDimension::AnalysisSteps,
                        ..
                    })
                ),
                "one step below complete try cost stops late"
            );
            assert_eq!(late.visited, before_visited);
            assert_eq!(late.unclosed_tail_at, before_tail);
            assert_eq!(late.catch_joins, before_joins);
            assert_eq!(
                late.loop_catch_terminal_owners
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
                    .collect::<Vec<_>>(),
                before_owner_fields
            );
            assert_eq!(late.fragmented.is_some(), before_fragmented);
            assert_eq!(late.excluded_edge_nodes, before_excluded);
            assert_eq!(late.own_monitor.is_some(), before_monitor);
            assert_eq!(late.depth, before_depth);
            assert!(
                late.budget.usage().analysis_steps > 0,
                "rollback does not refund analysis work"
            );
        },
    );
}
