use super::*;
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-break-branch-bodies/original/debug/classes/jadx/tests/integration/loops/TestBreakInLoop4$TestCls.class"
);
const NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-break-branch-bodies/original/no-debug/classes/jadx/tests/integration/loops/TestBreakInLoop4$TestCls.class"
);
const METHOD: &[u8] = b"test";
const DESCRIPTOR: &[u8] = b"(C)D";

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
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn with_method<R>(
    bytes: &[u8],
    f: impl FnOnce(
        &CanonicalCfg,
        &SsaTable,
        &MethodCodeFacts,
        &[CpEntryFacts],
        &NormalFlowView,
        &Operations,
    ) -> R,
) -> R {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen TestCls opens");
    let owner = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: u64::try_from(bytes.len()).expect("fixture size fits"),
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
                owner,
                name: JvmBytes(METHOD.to_vec()),
                descriptor: JvmBytes(DESCRIPTOR.to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("Reader analyzes the actual test(C)D method");
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("canonical graph from same run");
    let ssa = ir.ssa().expect("SSA from same run");
    let code = ir.code().expect("decoded code from same run");
    let pool = ir.constant_pool();
    let operations = Operations::of(code, pool);
    let view =
        NormalFlowView::build(canonical, &mut budget).expect("normal-flow projection builds");
    f(canonical, ssa, code, pool, &view, &operations)
}

fn walker<'a>(
    canonical: &'a CanonicalCfg,
    view: &'a NormalFlowView,
    ssa: &'a SsaTable,
    code: &'a MethodCodeFacts,
    pool: &'a [CpEntryFacts],
    operations: &'a Operations,
    chains: &'a crate::concat::Plan,
    sites: &'a crate::init::Sites,
    budget: &'a mut Budget,
) -> Walker<'a> {
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

fn find_loop(region: &Region, header_bci: u32) -> Option<&Region> {
    match region {
        Region::Loop { header, .. } if header.bci() == header_bci => Some(region),
        Region::Sequence { regions } | Region::Loop { body: regions, .. } => regions
            .iter()
            .find_map(|child| find_loop(child, header_bci)),
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => find_loop(then_arm, header_bci).or_else(|| find_loop(else_arm, header_bci)),
        _ => None,
    }
}

fn loop_break_sources(region: &Region, out: &mut Vec<u32>) {
    match region {
        Region::LoopBreak { source_bci, .. } => out.push(*source_bci),
        Region::Sequence { regions } | Region::Loop { body: regions, .. } => {
            for child in regions {
                loop_break_sources(child, out);
            }
        }
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => {
            loop_break_sources(then_arm, out);
            loop_break_sources(else_arm, out);
        }
        _ => {}
    }
}

#[test]
fn reader_recovers_complete_loop4_body_for_both_debug_profiles() {
    for bytes in [DEBUG, NO_DEBUG] {
        with_method(bytes, |canonical, ssa, code, pool, view, operations| {
            let mut budget = Budget::new(limits());
            let recovered = recover(
                canonical,
                view,
                ssa,
                operations,
                pool,
                &crate::concat::Plan::empty(),
                &crate::init::Sites::empty(),
                code,
                Some(false),
                false,
                &crate::pass::JAVA_8,
                &mut budget,
            )
            .expect("ample budget recovers the exact Reader method");
            assert_eq!(recovered.claimed, (0..view.len()).collect());
            assert_eq!(recovered.blocks, canonical.blocks().len());
            let loop_region = recovered
                .regions
                .iter()
                .find_map(|region| find_loop(region, 5))
                .expect("header 5 is recovered as the real loop");
            let Region::Loop {
                for_header: Some(for_header),
                body,
                ..
            } = loop_region
            else {
                panic!("the actual induction proof remains a for header");
            };
            assert_eq!(for_header.init_bci, 3);
            assert_eq!(for_header.update_bci, 36);
            assert_eq!(for_header.slot, 4);
            let owned: BTreeSet<_> = loop_region
                .blocks()
                .into_iter()
                .map(|id| id.bci())
                .collect();
            for bci in [5, 11, 17, 23, 29, 36] {
                assert!(owned.contains(&bci), "loop owns physical block {bci}");
            }
            let mut breaks = Vec::new();
            for region in body {
                loop_break_sources(region, &mut breaks);
            }
            breaks.sort_unstable();
            assert_eq!(breaks, [26, 33]);
            let flat: Vec<_> = recovered.regions.iter().flat_map(Region::blocks).collect();
            let unique: BTreeSet<_> = flat.iter().map(|id| (*id).clone()).collect();
            assert_eq!(flat.len(), unique.len(), "no duplicated physical owners");
        });
    }
}

#[test]
fn reader_proof_owns_only_the_closed_route_and_refuses_occupied_or_scoped_candidates() {
    with_method(DEBUG, |canonical, ssa, code, pool, view, operations| {
        let header = (0..view.len())
            .find(|node| view.id_of(*node).is_some_and(|id| id.bci() == 5))
            .expect("header block 5 exists");
        let natural = view
            .loop_entered_at(header)
            .expect("header has natural-loop facts");
        let blocks = natural.blocks().clone();
        let exit = view
            .index_of(
                &canonical
                    .blocks()
                    .iter()
                    .find(|block| block.id().bci() == 42)
                    .expect("exit block 42")
                    .id()
                    .clone(),
            )
            .expect("exit belongs to the graph");
        let update = view
            .index_of(
                &canonical
                    .blocks()
                    .iter()
                    .find(|block| block.id().bci() == 36)
                    .expect("update block 36")
                    .id()
                    .clone(),
            )
            .expect("update belongs to the graph");
        let chains = crate::concat::Plan::empty();
        let sites = crate::init::Sites::empty();
        let mut budget = Budget::new(limits());
        let mut w = walker(
            canonical,
            view,
            ssa,
            code,
            pool,
            operations,
            &chains,
            &sites,
            &mut budget,
        );
        let exits = w.loop_exit_nodes(&blocks);
        let proof = w
            .loop_break_branch_body(
                header,
                &blocks,
                &exits,
                Some(exit),
                update,
                &Frame::default(),
            )
            .expect("route proof completes within budget")
            .expect("the real route has one closed certificate");
        let owners: BTreeSet<_> = proof
            .owners
            .iter()
            .map(|node| view.id_of(*node).expect("owner identity").bci())
            .collect();
        assert_eq!(owners, BTreeSet::from([17, 23, 29]));

        let entry = proof.entry;
        w.visited.insert(entry);
        assert!(
            w.loop_break_branch_body(
                header,
                &blocks,
                &exits,
                Some(exit),
                update,
                &Frame::default()
            )
            .expect("occupied refusal is not a stop")
            .is_none()
        );
        w.visited.clear();
        let parent_scope = Frame {
            scope: Some(blocks.clone()),
            ..Frame::default()
        };
        assert!(
            w.loop_break_branch_body(header, &blocks, &exits, Some(exit), update, &parent_scope)
                .expect("parent scope refusal is not a stop")
                .is_none()
        );
    });
}

fn walker_state(
    walker: &Walker<'_>,
) -> (
    BTreeSet<usize>,
    Option<u32>,
    BTreeSet<(usize, usize)>,
    Vec<(usize, usize, usize, u32, BTreeSet<usize>)>,
    Option<String>,
    BTreeSet<CanonicalBlockId>,
    Option<Vec<CanonicalBlockId>>,
    usize,
) {
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
        walker.fragmented.as_ref().map(|proof| format!("{proof:?}")),
        walker.excluded_edge_nodes.clone(),
        walker
            .own_monitor
            .as_ref()
            .map(|monitor| monitor.owned.clone()),
        walker.depth,
    )
}

fn seed_trial_state(walker: &mut Walker<'_>, exit: &CanonicalBlockId) {
    walker.visited.insert(usize::MAX);
    walker.excluded_edge_nodes.insert(exit.clone());
    walker.depth = 4;
}

#[test]
fn reader_late_implicit_latch_stop_restores_trial_state_without_refunding_usage() {
    with_method(DEBUG, |canonical, ssa, code, pool, view, operations| {
        let header_node = (0..view.len())
            .find(|node| view.id_of(*node).is_some_and(|id| id.bci() == 5))
            .expect("Reader graph has Loop4 header 5");
        let header = view.id_of(header_node).cloned().expect("header identity");
        let blocks = view
            .loop_entered_at(header_node)
            .expect("natural loop")
            .blocks()
            .clone();
        let exit = canonical
            .blocks()
            .iter()
            .find(|block| block.id().bci() == 42)
            .expect("break target 42")
            .id()
            .clone();
        let chains = crate::concat::Plan::empty();
        let sites = crate::init::Sites::empty();

        let mut full_budget = Budget::new(limits());
        let mut full = walker(
            canonical,
            view,
            ssa,
            code,
            pool,
            operations,
            &chains,
            &sites,
            &mut full_budget,
        );
        seed_trial_state(&mut full, &exit);
        let before_full = walker_state(&full);
        let full_output = full
            .header_tested_loop(&header, header_node, &blocks, &Frame::default())
            .expect("unbounded Reader path completes")
            .expect("the actual loop produces its header-tested region");
        assert!(
            full_output.0.iter().any(|region| matches!(
                find_loop(region, 5),
                Some(Region::Loop {
                    for_header: Some(_),
                    ..
                })
            )),
            "the measured unlimited run must be the actual certified for-loop, not a fallback"
        );
        let full_steps = full.budget.usage().analysis_steps;
        assert!(full_steps > canonical.edges().len() as u64);
        drop(full);

        // The asserted real Loop rules out measuring a fallback. The actual final charged work
        // on this path is `implicit_tail_latch_origin`'s canonical-edge scan, so a limit one below
        // the measured complete usage refuses that indivisible charge after body/owner proof.
        let mut late_budget = Budget::new(Limits {
            analysis_steps: full_steps - 1,
            ..limits()
        });
        let mut late = walker(
            canonical,
            view,
            ssa,
            code,
            pool,
            operations,
            &chains,
            &sites,
            &mut late_budget,
        );
        seed_trial_state(&mut late, &exit);
        let checkpoint = walker_state(&late);
        assert_eq!(
            checkpoint, before_full,
            "both attempts start from the same state"
        );
        let stopped = late.header_tested_loop(&header, header_node, &blocks, &Frame::default());
        assert!(
            matches!(
                stopped,
                Err(StopReason::Budget {
                    dimension: CountedBudgetDimension::AnalysisSteps,
                    ..
                })
            ),
            "one-unit-short late path must stop at Budget"
        );
        let used = late.budget.usage().analysis_steps;
        assert_eq!(
            used,
            full_steps - canonical.edges().len() as u64,
            "the refused final edge-scan charge remains unspent while prior usage is retained"
        );
        assert!(used > 0, "rollback never refunds admitted analysis work");
        assert_eq!(
            walker_state(&late),
            checkpoint,
            "late stop restores all mutable walker ownership and auxiliary state"
        );
    });
}

#[test]
fn edge_gate_rejects_external_entries_extra_exits_exceptions_and_cycles() {
    with_method(
        DEBUG,
        |canonical, _ssa, _code, _pool, _view, _operations| {
            let id = |bci| {
                canonical
                    .blocks()
                    .iter()
                    .find(|block| block.id().bci() == bci)
                    .map(|block| block.id().clone())
                    .expect("pinned BCI exists")
            };
            let branch = id(11);
            let entry = id(17);
            let exit = id(42);
            let owners = BTreeSet::from([entry.clone(), id(23), id(29)]);
            let original: Vec<_> = canonical
                .edges()
                .iter()
                .map(|edge| (edge.kind(), edge.from().clone(), edge.to().clone()))
                .collect();
            let mut budget = Budget::new(limits());
            assert!(
                loop_break_body_edges_closed(
                    &branch,
                    &entry,
                    &exit,
                    &owners,
                    &original,
                    &mut budget
                )
                .expect("the fixture edge scan remains within budget")
            );
            assert!(
                loop_break_body_acyclic(&owners, &original, &mut budget)
                    .expect("the fixture cycle scan remains within budget")
            );

            let mut rows = original.clone();
            rows.push((CanonicalEdgeKind::Normal, exit.clone(), entry.clone()));
            assert!(
                !loop_break_body_edges_closed(
                    &branch,
                    &entry,
                    &exit,
                    &owners,
                    &rows,
                    &mut Budget::new(limits())
                )
                .expect("external-entry refusal is not a stop"),
                "an external predecessor cannot enter the route"
            );

            let mut rows = original.clone();
            rows.push((CanonicalEdgeKind::Normal, id(29), id(5)));
            assert!(
                !loop_break_body_edges_closed(
                    &branch,
                    &entry,
                    &exit,
                    &owners,
                    &rows,
                    &mut Budget::new(limits())
                )
                .expect("extra-exit refusal is not a stop"),
                "an owner cannot escape to another loop block"
            );

            let mut rows = original.clone();
            rows.push((
                CanonicalEdgeKind::Exception { handler_ordinal: 0 },
                id(23),
                exit.clone(),
            ));
            assert!(
                !loop_break_body_edges_closed(
                    &branch,
                    &entry,
                    &exit,
                    &owners,
                    &rows,
                    &mut Budget::new(limits())
                )
                .expect("exception refusal is not a stop"),
                "unexplained exceptional rows are refused"
            );

            let mut rows = original;
            rows.push((CanonicalEdgeKind::Normal, id(29), id(23)));
            assert!(
                loop_break_body_edges_closed(
                    &branch,
                    &entry,
                    &exit,
                    &owners,
                    &rows,
                    &mut Budget::new(limits())
                )
                .expect("closed cyclic graph still passes edge closure"),
                "the internal cycle keeps exact external entry/exit closure"
            );
            assert!(
                !loop_break_body_acyclic(&owners, &rows, &mut Budget::new(limits()))
                    .expect("cycle refusal is not a stop"),
                "internal back-edge is refused"
            );
        },
    );
}

#[test]
fn comparison_gate_refuses_switch_and_budget_or_cancellation_stays_a_stop() {
    assert!(!loop_break_body_branch_eligible(
        &Operation::Switch {
            cases: vec![(0, 10)],
            default: 12
        },
        2
    ));
    assert!(!loop_break_body_branch_eligible(
        &Operation::Comparison {
            op: CompareOp::JumpIfSame,
            target: 10
        },
        3
    ));
    assert!(loop_break_body_branch_eligible(
        &Operation::Comparison {
            op: CompareOp::JumpIfSame,
            target: 10
        },
        2
    ));

    with_method(DEBUG, |canonical, ssa, code, pool, view, operations| {
        let header = (0..view.len())
            .find(|node| view.id_of(*node).is_some_and(|id| id.bci() == 5))
            .expect("header block 5 exists");
        let blocks = view
            .loop_entered_at(header)
            .expect("natural loop")
            .blocks()
            .clone();
        let exit = (0..view.len())
            .find(|node| view.id_of(*node).is_some_and(|id| id.bci() == 42))
            .expect("exit block 42 exists");
        let update = (0..view.len())
            .find(|node| view.id_of(*node).is_some_and(|id| id.bci() == 36))
            .expect("update block 36 exists");

        let chains = crate::concat::Plan::empty();
        let sites = crate::init::Sites::empty();
        let mut tiny = Budget::new(Limits {
            analysis_steps: 0,
            ..limits()
        });
        let mut tiny_walker = walker(
            canonical, view, ssa, code, pool, operations, &chains, &sites, &mut tiny,
        );
        let exits = tiny_walker.loop_exit_nodes(&blocks);
        assert!(matches!(
            tiny_walker.loop_break_branch_body(
                header,
                &blocks,
                &exits,
                Some(exit),
                update,
                &Frame::default()
            ),
            Err(StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                ..
            })
        ));
        assert!(tiny_walker.visited.is_empty());

        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(limits(), token);
        let mut cancelled_walker = walker(
            canonical,
            view,
            ssa,
            code,
            pool,
            operations,
            &chains,
            &sites,
            &mut cancelled,
        );
        let exits = cancelled_walker.loop_exit_nodes(&blocks);
        assert!(matches!(
            cancelled_walker.loop_break_branch_body(
                header,
                &blocks,
                &exits,
                Some(exit),
                update,
                &Frame::default()
            ),
            Err(StopReason::Cancelled { at: Some(5) })
        ));
        assert!(cancelled_walker.visited.is_empty());
    });
}
