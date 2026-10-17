//! Reader-backed negative/stop seams for the private switch-loop completion proof.
//! This module is meant to be placed at crates/jarde-java/src/region/switch_nonlocal_completion_tests.rs
//! and declared from region.rs after the v4 join candidate is integrated.
use super::*;
use jarde_reader::budget::CancellationToken;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

const SN: &[u8] =
    include_bytes!("../../../../tests/fixtures/p3-switch-nonlocal-exits/javac8/SN.class");
const EXTRA_ENTRY: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-switch-loop-exits/v8/SwitchLoopLocalBoundaries.class"
);
const SN_SWITCH_BCI: u32 = 24;

fn ample() -> Budget {
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

fn with_method<R>(
    bytes: &[u8],
    method_name: &[u8],
    descriptor: &[u8],
    f: impl FnOnce(
        &CanonicalCfg,
        &SsaTable,
        &MethodCodeFacts,
        &[CpEntryFacts],
        &NormalFlowView,
        &Operations,
        u16,
    ) -> R,
) -> R {
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

    let mut reader_budget = ample();
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut reader_budget)
        .expect("frozen whole class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: u64::try_from(bytes.len()).expect("fixture length fits u64"),
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
    .expect("Reader analyzes this exact fixture method");
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("canonical CFG from same analysis");
    let ssa = ir.ssa().expect("SSA from same analysis");
    let code = ir.code().expect("decoded code from same analysis");
    let declaration = ir.declaration().expect("real physical method declaration");
    assert_eq!(declaration.name().0.as_slice(), method_name);
    assert_eq!(declaration.descriptor().0.as_slice(), descriptor);
    let access_flags = declaration.access_flags();
    let pool = ir.constant_pool();
    let operations = Operations::of(code, pool);
    assert!(
        code.max_locals > 0,
        "reader supplied real local-slot bounds"
    );
    for instruction in &code.instructions {
        if let Some(
            Operation::Load { slot }
            | Operation::Store { slot }
            | Operation::Increment { slot, .. },
        ) = operations.get(instruction.bci)
        {
            assert!(
                *slot < code.max_locals,
                "decoded local slot is within the real Code.max_locals"
            );
        }
    }
    let view = NormalFlowView::build(canonical, &mut reader_budget)
        .expect("normal-flow view projects the exact canonical graph");
    f(canonical, ssa, code, pool, &view, &operations, access_flags)
}

fn loop_frame(
    view: &NormalFlowView,
    header: usize,
    blocks: &BTreeSet<usize>,
    returns: &BTreeSet<usize>,
    transfer_sources: &BTreeSet<usize>,
) -> (Frame, usize, BTreeSet<usize>) {
    let external: BTreeSet<_> = blocks
        .iter()
        .flat_map(|node| view.successors(*node))
        .filter(|successor| !blocks.contains(successor))
        .collect();
    let header_exit = view
        .successors(header)
        .into_iter()
        .find(|successor| !blocks.contains(successor))
        .expect("the real loop header has a normal exit");
    let frame = Frame::default().loop_body(
        blocks,
        header,
        header,
        header,
        Some(header_exit),
        Some(header_exit),
        external.clone(),
        transfer_sources,
        returns,
        &BTreeSet::new(),
    );
    (frame, header_exit, external)
}

macro_rules! walker {
    ($budget:ident, $canonical:ident, $view:ident, $ssa:ident, $code:ident, $pool:ident, $operations:ident, $chains:ident, $sites:ident, $synchronized:ident) => {
        Walker {
            canonical: $canonical,
            view: $view,
            ssa: $ssa,
            operations: $operations,
            pool: $pool,
            chains: &$chains,
            sites: &$sites,
            code: $code,
            method_synchronized: Some($synchronized),
            has_reachable_explicit_monitor: false,
            return_is_boolean: false,
            handlers: &$code.exception_handlers,
            profile: &crate::pass::JAVA_8,
            budget: &mut $budget,
            catch_joins: BTreeSet::new(),
            loop_catch_terminal_owners: Vec::new(),
            fragmented: None,
            excluded_edge_nodes: BTreeSet::new(),
            visited: BTreeSet::new(),
            unclosed_tail_at: None,
            own_monitor: None,
            depth: 0,
        }
    };
}

#[test]
fn real_switch_return_certificate_and_target_boundaries_refuse_without_claims() {
    with_method(
        SN,
        b"retInSwitchNoTail",
        b"([I)I",
        |canonical, ssa, code, pool, view, operations, access_flags| {
            assert_ne!(
                access_flags & 0x0008,
                0,
                "the real fixture method is static"
            );
            let method_synchronized = access_flags & 0x0020 != 0;
            assert!(
                !method_synchronized,
                "the real declaration is not synchronized"
            );
            let switch_block = ssa
                .blocks()
                .iter()
                .find(|block| {
                    block
                        .instructions()
                        .last()
                        .is_some_and(|ins| ins.bci() == SN_SWITCH_BCI)
                })
                .expect("the pinned switch BCI belongs to a real SSA block")
                .block();
            assert!(
                operations
                    .get(SN_SWITCH_BCI)
                    .and_then(Operation::switch)
                    .is_some()
            );
            let branch = view
                .index_of(switch_block)
                .expect("switch block is in normal flow");
            assert_eq!(
                view.id_of(branch).unwrap().bci(),
                16,
                "physical block start differs from switch BCI"
            );
            let successors: Vec<_> = view
                .successors(branch)
                .into_iter()
                .map(|node| view.id_of(node).expect("successor identity").clone())
                .collect();
            assert_eq!(
                successors
                    .iter()
                    .map(CanonicalBlockId::bci)
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from([44, 49])
            );

            let natural = (0..view.len())
                .filter_map(|header| view.loop_entered_at(header))
                .filter(|loop_| loop_.blocks().contains(&branch))
                .min_by_key(|loop_| loop_.blocks().len())
                .expect("switch dispatch lies in one Reader-derived natural loop");
            let header = natural.header();
            let blocks = natural.blocks().clone();
            assert_eq!(view.id_of(header).unwrap().bci(), 10);
            let normal_exit = view
                .successors(header)
                .into_iter()
                .find(|successor| !blocks.contains(successor))
                .expect("the real loop header has one normal exit");
            let mut terminal_budget = ample();
            let chains = crate::concat::Plan::empty();
            let sites = crate::init::Sites::empty();
            let terminal_returns = walker!(
                terminal_budget,
                canonical,
                view,
                ssa,
                code,
                pool,
                operations,
                chains,
                sites,
                method_synchronized
            )
            .loop_terminal_returns(&blocks, Some(normal_exit), &Frame::default())
            .expect("the real switch return leaf certificate completes");
            let return_bcis: BTreeSet<_> = terminal_returns
                .iter()
                .filter_map(|node| view.id_of(*node).map(CanonicalBlockId::bci))
                .collect();
            assert_eq!(return_bcis, BTreeSet::from([44]));

            let external: BTreeSet<_> = blocks
                .iter()
                .flat_map(|node| view.successors(*node))
                .filter(|successor| !blocks.contains(successor))
                .collect();
            let mut targets = external.clone();
            targets.insert(header);
            let transfer_sources: BTreeSet<_> = (0..view.len())
                .filter(|source| {
                    let next = view.successors(*source);
                    next.len() == 1 && targets.contains(&next[0])
                })
                .collect();
            let (frame, _normal_exit, _external) =
                loop_frame(view, header, &blocks, &terminal_returns, &transfer_sources);

            let mut complete_budget = ample();
            let result = walker!(
                complete_budget,
                canonical,
                view,
                ssa,
                code,
                pool,
                operations,
                chains,
                sites,
                method_synchronized
            )
            .switch_loop_join(branch, &successors, &frame, SN_SWITCH_BCI, None, false)
            .expect("real switch proof completes");
            assert!(
                matches!(
                    result,
                    SwitchLoopJoin::Closed { continue_target: Some(target) } if target == header
                ),
                "real case return + loop continue closes only with the Reader return certificate"
            );
            let measured = complete_budget.usage().analysis_steps;
            assert!(measured > 1, "strict proof charged real candidate work");

            let mut missing = frame.clone();
            missing
                .loop_targets
                .last_mut()
                .expect("own loop target")
                .terminal_returns = Arc::new(BTreeSet::new());
            let mut missing_budget = ample();
            let mut missing_walker = walker!(
                missing_budget,
                canonical,
                view,
                ssa,
                code,
                pool,
                operations,
                chains,
                sites,
                method_synchronized
            );
            assert!(
                matches!(
                    missing_walker.switch_loop_join(
                        branch,
                        &successors,
                        &missing,
                        SN_SWITCH_BCI,
                        None,
                        false
                    ),
                    Ok(SwitchLoopJoin::Refused)
                ),
                "the exact real return terminal cannot be inferred after its certificate is removed"
            );
            assert!(missing_walker.visited.is_empty());
            assert!(missing_walker.catch_joins.is_empty());

            let wrong_target = _normal_exit;
            assert_ne!(
                wrong_target, header,
                "normal exit and continue target have distinct Reader identities"
            );
            let mut wrong_budget = ample();
            let mut wrong_walker = walker!(
                wrong_budget,
                canonical,
                view,
                ssa,
                code,
                pool,
                operations,
                chains,
                sites,
                method_synchronized
            );
            assert!(
                matches!(
                    wrong_walker.switch_loop_join(
                        branch,
                        &successors,
                        &frame,
                        SN_SWITCH_BCI,
                        Some(wrong_target),
                        false
                    ),
                    Ok(SwitchLoopJoin::Refused)
                ),
                "a real loop exit cannot be smuggled in as the current continue target"
            );
            assert!(wrong_walker.visited.is_empty());

            let mut ambiguous = frame.clone();
            ambiguous
                .loop_targets
                .push(ambiguous.loop_targets.last().expect("own target").clone());
            let mut ambiguous_budget = ample();
            let mut ambiguous_walker = walker!(
                ambiguous_budget,
                canonical,
                view,
                ssa,
                code,
                pool,
                operations,
                chains,
                sites,
                method_synchronized
            );
            assert!(
                matches!(
                    ambiguous_walker.switch_loop_join(
                        branch,
                        &successors,
                        &ambiguous,
                        SN_SWITCH_BCI,
                        None,
                        false
                    ),
                    Ok(SwitchLoopJoin::Refused)
                ),
                "duplicate enclosing loop destinations are ambiguous proof context"
            );
            assert!(ambiguous_walker.visited.is_empty());

            // Sweep real counted limits. The first charge may be refused at entry; require a later
            // refusal with positive usage to prove that this test reaches work inside the certificate.
            let mut saw_late_stop = false;
            for limit in 1..measured {
                let mut budget = Budget::new(jarde_reader::budget::Limits {
                    analysis_steps: limit,
                    elapsed_millis: u64::MAX,
                    ..Default::default()
                });
                let mut stopped = walker!(
                    budget,
                    canonical,
                    view,
                    ssa,
                    code,
                    pool,
                    operations,
                    chains,
                    sites,
                    method_synchronized
                );
                let result = stopped.switch_loop_join(
                    branch,
                    &successors,
                    &frame,
                    SN_SWITCH_BCI,
                    None,
                    false,
                );
                if matches!(
                    result,
                    Err(StopReason::Budget {
                        dimension: CountedBudgetDimension::AnalysisSteps,
                        at: Some(SN_SWITCH_BCI),
                        ..
                    })
                ) && stopped.budget.usage().analysis_steps > 0
                {
                    assert!(stopped.budget.usage().analysis_steps <= limit);
                    assert_eq!(stopped.visited, BTreeSet::new());
                    assert!(stopped.catch_joins.is_empty());
                    assert!(stopped.loop_catch_terminal_owners.is_empty());
                    assert!(stopped.fragmented.is_none());
                    assert!(stopped.excluded_edge_nodes.is_empty());
                    assert_eq!(stopped.unclosed_tail_at, None);
                    assert_eq!(stopped.depth, 0);
                    saw_late_stop = true;
                    break;
                }
            }
            assert!(
                saw_late_stop,
                "a swept positive-charge proof stop is required; entry-only refusal is insufficient"
            );

            let token = CancellationToken::new();
            let mut cancel_budget = Budget::with_cancellation_token(
                jarde_reader::budget::Limits {
                    analysis_steps: u64::MAX,
                    elapsed_millis: u64::MAX,
                    ..Default::default()
                },
                token.clone(),
            );
            token.cancel(); // Reader facts and the complete target/frame are already prepared.
            let mut cancel_walker = walker!(
                cancel_budget,
                canonical,
                view,
                ssa,
                code,
                pool,
                operations,
                chains,
                sites,
                method_synchronized
            );
            let cancel = match cancel_walker.switch_loop_join(
                branch,
                &successors,
                &frame,
                SN_SWITCH_BCI,
                None,
                false,
            ) {
                Err(stop) => stop,
                Ok(_) => panic!("post-analysis proof-entry cancellation is observed"),
            };
            assert_eq!(
                cancel,
                StopReason::Cancelled {
                    at: Some(SN_SWITCH_BCI)
                }
            );
            assert!(cancel_walker.visited.is_empty());
            assert!(cancel_walker.catch_joins.is_empty());
            assert!(cancel_walker.loop_catch_terminal_owners.is_empty());
            assert!(cancel_walker.fragmented.is_none());
            assert!(cancel_walker.excluded_edge_nodes.is_empty());
            assert_eq!(cancel_walker.unclosed_tail_at, None);
            assert_eq!(cancel_walker.depth, 0);
            let mut cancel_usage = cancel_walker.budget.usage().clone();
            cancel_usage.elapsed_millis = 0;
            assert_eq!(cancel_usage, jarde_reader::budget::UsageSnapshot::default());

            let mut deadline_budget = Budget::new(jarde_reader::budget::Limits {
                analysis_steps: u64::MAX,
                elapsed_millis: 1,
                ..Default::default()
            });
            std::thread::sleep(std::time::Duration::from_millis(3));
            let mut deadline_walker = walker!(
                deadline_budget,
                canonical,
                view,
                ssa,
                code,
                pool,
                operations,
                chains,
                sites,
                method_synchronized
            );
            let deadline = match deadline_walker.switch_loop_join(
                branch,
                &successors,
                &frame,
                SN_SWITCH_BCI,
                None,
                false,
            ) {
                Err(stop) => stop,
                Ok(_) => {
                    panic!("deadline expires after complete Reader preparation, before proof work")
                }
            };
            assert_eq!(
                deadline,
                StopReason::Interrupted {
                    code: crate::stop::BUDGET_INTERRUPTED_CODE,
                    at: Some(SN_SWITCH_BCI)
                }
            );
            assert!(deadline_walker.visited.is_empty());
            assert!(deadline_walker.catch_joins.is_empty());
            assert!(deadline_walker.loop_catch_terminal_owners.is_empty());
            assert!(deadline_walker.fragmented.is_none());
            assert!(deadline_walker.excluded_edge_nodes.is_empty());
            assert_eq!(deadline_walker.unclosed_tail_at, None);
            assert_eq!(deadline_walker.depth, 0);
            let mut deadline_usage = deadline_walker.budget.usage().clone();
            deadline_usage.elapsed_millis = 0;
            assert_eq!(
                deadline_usage,
                jarde_reader::budget::UsageSnapshot::default()
            );
        },
    );
}

#[test]
fn real_extra_entry_is_not_owned_by_the_switch_join_certificate() {
    with_method(
        EXTRA_ENTRY,
        b"extraEntry",
        b"(I)I",
        |canonical, ssa, code, pool, view, operations, access_flags| {
            assert_ne!(
                access_flags & 0x0008,
                0,
                "the real fixture method is static"
            );
            let method_synchronized = access_flags & 0x0020 != 0;
            assert!(
                !method_synchronized,
                "the real declaration is not synchronized"
            );
            let (switch_bci, branch) = ssa
                .blocks()
                .iter()
                .filter_map(|block| {
                    let ins = block.instructions().last()?;
                    operations
                        .get(ins.bci())
                        .and_then(Operation::switch)
                        .map(|_| {
                            (
                                ins.bci(),
                                view.index_of(block.block()).expect("switch block in view"),
                            )
                        })
                })
                .find(|(_, branch)| {
                    (0..view.len()).any(|header| {
                        view.loop_entered_at(header)
                            .is_some_and(|natural| natural.blocks().contains(branch))
                    })
                })
                .expect("the real extraEntry method has one switch inside its loop");
            let dispatch_successors = view.successors(branch).into_iter().collect::<BTreeSet<_>>();
            assert!(dispatch_successors.len() >= 2);
            let natural = (0..view.len())
                .filter_map(|header| view.loop_entered_at(header))
                .filter(|natural| natural.blocks().contains(&branch))
                .min_by_key(|natural| natural.blocks().len())
                .expect("Reader natural loop owns this dispatch");
            let loop_nodes = natural.blocks();

            // Locate a real shared normal-flow node reached from the switch but also entered along
            // the `if (i != 4)` bypass. This is an existing valid class, not a synthetic CFG edge.
            let candidate = (0..view.len())
                .filter(|node| {
                    loop_nodes.contains(node)
                        && *node != branch
                        && !dispatch_successors.contains(node)
                })
                .find(|node| {
                    let preds = view.predecessors(*node);
                    preds.len() >= 2
                        && preds.iter().any(|pred| view.dominates(branch, *pred))
                        && preds.iter().any(|pred| !view.dominates(branch, *pred))
                })
                .expect("Reader facts expose a normal join with an outside-switch predecessor");
            let preds = view.predecessors(candidate);
            let arm_predecessors: Vec<_> = preds
                .iter()
                .copied()
                .filter(|pred| view.dominates(branch, *pred))
                .collect();
            let outside_predecessor = preds
                .iter()
                .copied()
                .find(|pred| !view.dominates(branch, *pred))
                .expect("bypass predecessor");
            assert!(!arm_predecessors.is_empty());
            assert!(!dispatch_successors.contains(&candidate));
            let mut owners = BTreeMap::new();
            for (arm, pred) in arm_predecessors.iter().copied().enumerate() {
                owners.insert(pred, arm);
            }
            assert!(!owners.contains_key(&outside_predecessor));

            let chains = crate::concat::Plan::empty();
            let sites = crate::init::Sites::empty();
            let mut budget = ample();
            let mut walker = walker!(
                budget,
                canonical,
                view,
                ssa,
                code,
                pool,
                operations,
                chains,
                sites,
                method_synchronized
            );
            assert!(
                !walker
                    .switch_join_incoming_owned(candidate, &owners, switch_bci)
                    .expect("real outside predecessor is an ordinary ownership refusal")
            );
            assert!(walker.visited.is_empty(), "the proof seam is read-only");
        },
    );
}

#[test]
fn exception_edge_incident_to_real_loop_switch_refuses_strict_completion() {
    for class in [
        include_bytes!(
            "../../../../tests/fixtures/switch-loop-exception-boundary/javac8/SwitchLoopExceptionBoundary.class"
        ) as &[u8],
        include_bytes!(
            "../../../../tests/fixtures/switch-loop-exception-boundary/javac23/SwitchLoopExceptionBoundary.class"
        ) as &[u8],
    ] {
        with_method(
            class,
            b"run",
            b"([I)I",
            |canonical, ssa, code, pool, view, operations, access_flags| {
                assert_ne!(access_flags & 0x0008, 0, "run is static");
                let method_synchronized = access_flags & 0x0020 != 0;
                assert!(!method_synchronized, "run is not synchronized");
                assert!(
                    !code.exception_handlers.is_empty(),
                    "Reader supplies the actual catch row"
                );

                let switch_block = ssa
                    .blocks()
                    .iter()
                    .find(|block| {
                        block.instructions().last().is_some_and(|instruction| {
                            operations
                                .get(instruction.bci())
                                .and_then(Operation::switch)
                                .is_some()
                        })
                    })
                    .expect("the exact analyzed method has its switch block")
                    .block();
                let branch = view
                    .index_of(switch_block)
                    .expect("real switch in normal flow");
                let switch_bci = ssa
                    .block(switch_block)
                    .and_then(|block| block.instructions().last())
                    .expect("switch block has a terminal instruction")
                    .bci();
                assert!(
                    operations
                        .get(switch_bci)
                        .and_then(Operation::switch)
                        .is_some()
                );
                let successors: Vec<_> = view
                    .successors(branch)
                    .into_iter()
                    .map(|successor| view.id_of(successor).expect("real switch target").clone())
                    .collect();
                assert!(
                    successors.len() >= 2,
                    "the Reader graph keeps the switch targets"
                );

                let natural = (0..view.len())
                    .filter_map(|header| view.loop_entered_at(header))
                    .filter(|loop_| loop_.blocks().contains(&branch))
                    .min_by_key(|loop_| loop_.blocks().len())
                    .expect("the switch lies in a Reader-derived natural loop");
                let header = natural.header();
                let blocks = natural.blocks().clone();
                let external: BTreeSet<_> = blocks
                    .iter()
                    .flat_map(|node| view.successors(*node))
                    .filter(|successor| !blocks.contains(successor))
                    .collect();
                let mut destinations = external.clone();
                destinations.insert(header);
                let transfer_sources: BTreeSet<_> = (0..view.len())
                    .filter(|source| {
                        let next = view.successors(*source);
                        next.len() == 1 && destinations.contains(&next[0])
                    })
                    .collect();
                let (frame, _, _) =
                    loop_frame(view, header, &blocks, &BTreeSet::new(), &transfer_sources);

                let mut census_budget = ample();
                let chains = crate::concat::Plan::empty();
                let sites = crate::init::Sites::empty();
                let mut census = walker!(
                    census_budget,
                    canonical,
                    view,
                    ssa,
                    code,
                    pool,
                    operations,
                    chains,
                    sites,
                    method_synchronized
                );
                let non_normal = census
                    .switch_non_normal_nodes(switch_bci)
                    .expect("the exact canonical exceptional-edge census completes")
                    .expect("all canonical edge endpoints are in this normal-flow view");
                assert!(
                    canonical.edges().iter().any(|edge| matches!(
                        edge.kind(),
                        CanonicalEdgeKind::Exception { .. }
                    ) && (edge.from() == switch_block
                        || edge.to() == switch_block)),
                    "actual canonical exception edge is incident to dispatch"
                );
                assert!(
                    non_normal.contains(&branch),
                    "the real catch edge is incident to the switch dispatch block"
                );

                let mut proof_budget = ample();
                let mut proof = walker!(
                    proof_budget,
                    canonical,
                    view,
                    ssa,
                    code,
                    pool,
                    operations,
                    chains,
                    sites,
                    method_synchronized
                );
                let result = proof
                    .switch_loop_join(branch, &successors, &frame, switch_bci, None, false)
                    .expect("strict proof refusal is an ordinary result");
                assert!(
                    matches!(result, SwitchLoopJoin::Refused),
                    "a real exceptional edge incident to dispatch refuses the strict completion proof"
                );
                assert!(proof.visited.is_empty());
                assert!(proof.catch_joins.is_empty());
                assert!(proof.loop_catch_terminal_owners.is_empty());
                assert!(proof.fragmented.is_none());
                assert!(proof.excluded_edge_nodes.is_empty());
            },
        );
    }
}

#[test]
fn direct_completion_edge_is_bound_to_its_real_jsr_clone() {
    with_method(
        include_bytes!(
            "../../../../tests/fixtures/p3-switch-jsr-clones/v49/SwitchLoopJsrClone.class"
        ),
        b"run",
        b"(I)I",
        |canonical, ssa, code, pool, view, operations, access_flags| {
            assert_ne!(access_flags & 0x0008, 0, "run is static");
            let method_synchronized = access_flags & 0x0020 != 0;
            assert!(!method_synchronized);
            let mut switches: Vec<_> = ssa
                .blocks()
                .iter()
                .filter_map(|block| {
                    let instruction = block.instructions().last()?;
                    operations
                        .get(instruction.bci())
                        .and_then(Operation::switch)
                        .map(|_| {
                            let id = block.block().clone();
                            let node = view
                                .index_of(&id)
                                .expect("Reader clone switch is in normal flow");
                            (instruction.bci(), id, node)
                        })
                })
                .collect();
            switches.sort_by(|left, right| left.1.path().cmp(right.1.path()));
            assert_eq!(
                switches.len(),
                2,
                "Reader publishes one subroutine switch clone per jsr site"
            );
            let (first_bci, first_id, first_branch) = &switches[0];
            let (second_bci, second_id, second_branch) = &switches[1];
            assert_eq!(
                first_bci, second_bci,
                "both blocks derive from the same physical switch"
            );
            assert!(first_id.is_clone() && second_id.is_clone());
            assert_ne!(
                first_id.path(),
                second_id.path(),
                "the two call-site contexts remain distinct"
            );

            let first_successors: Vec<_> = view
                .successors(*first_branch)
                .into_iter()
                .map(|node| view.id_of(node).expect("first clone target").clone())
                .collect();
            let second_successors: Vec<_> = view
                .successors(*second_branch)
                .into_iter()
                .map(|node| view.id_of(node).expect("second clone target").clone())
                .collect();
            assert_eq!(first_successors.len(), second_successors.len());
            assert_eq!(
                first_successors
                    .iter()
                    .map(CanonicalBlockId::bci)
                    .collect::<BTreeSet<_>>(),
                second_successors
                    .iter()
                    .map(CanonicalBlockId::bci)
                    .collect::<BTreeSet<_>>(),
                "the cross-context comparison preserves physical target BCIs"
            );
            assert!(
                second_successors
                    .iter()
                    .all(|id| id.path() != first_id.path())
            );

            // Current clone contexts are checked at the edge certificate; the whole-loop
            // admission of legacy subroutines is a separate NormalFlowView concern.

            let switch_bci = operations
                .get(*first_bci)
                .and_then(Operation::switch)
                .map(|_| *first_bci)
                .expect("the source instruction remains a decoded switch");
            let chains = crate::concat::Plan::empty();
            let sites = crate::init::Sites::empty();
            let mut budget = ample();
            let mut walker = walker!(
                budget,
                canonical,
                view,
                ssa,
                code,
                pool,
                operations,
                chains,
                sites,
                method_synchronized
            );
            let non_normal = walker
                .switch_non_normal_nodes(switch_bci)
                .expect("the actual canonical edge census completes")
                .expect("canonical edge endpoints are present in this view");

            for successor in &first_successors {
                let node = view
                    .index_of(successor)
                    .expect("same-clone successor identity");
                assert!(
                    walker.switch_completion_edge(*first_branch, node, node, None, &non_normal,),
                    "each direct successor is an actual completion edge of its own clone"
                );
            }
            for successor in &second_successors {
                let node = view
                    .index_of(successor)
                    .expect("other-clone successor identity");
                assert!(
                    !walker.switch_completion_edge(*first_branch, node, node, None, &non_normal,),
                    "same physical target BCIs from another clone are not current-branch successors"
                );
            }
        },
    );
}
