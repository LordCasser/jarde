//! Reader-backed tests for the bounded statementful loop-header consumer.
// v3 measures the exact proof cost and adds a gateway physical-opcode mismatch case.
// v2 adds a physical opcode/Operation mismatch refusal case to guard the proof's opcode identity
// checks alongside its branch-target checks.
use super::*;
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};
use std::collections::BTreeSet;

const ORIGINAL_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-statementful-loop-headers/original/debug/classes/jadx/tests/integration/loops/TestSequentialLoops$TestCls.class"
);
const ORIGINAL_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-statementful-loop-headers/original/no-debug/classes/jadx/tests/integration/loops/TestSequentialLoops$TestCls.class"
);
const SUPPLEMENT_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac8-debug/classes/SequentialLoopEffects.class"
);
const SUPPLEMENT_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac8-no-debug/classes/SequentialLoopEffects.class"
);
const SUPPLEMENT_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac23-debug/classes/SequentialLoopEffects.class"
);
const SUPPLEMENT_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-statementful-loop-headers/supplement/javac23-no-debug/classes/SequentialLoopEffects.class"
);

fn reader_budget() -> Budget {
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
        dependency_depth: 32,
        elapsed_millis: u64::MAX,
    })
}

fn proof_budget(analysis_steps: u64) -> Budget {
    Budget::new(Limits {
        analysis_steps,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    })
}

fn with_method<R>(
    bytes: &[u8],
    name: &[u8],
    descriptor: &[u8],
    f: impl FnOnce(
        &CanonicalCfg,
        &SsaTable,
        &MethodCodeFacts,
        &[CpEntryFacts],
        &NormalFlowView,
        &Operations,
    ) -> R,
) -> R {
    let mut budget = reader_budget();
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen whole class parses");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(bytes).to_hex().to_string()),
            length: u64::try_from(bytes.len()).expect("fixture size fits u64"),
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
                name: JvmBytes(name.to_vec()),
                descriptor: JvmBytes(descriptor.to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("Reader analyzes this exact physical method");
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("canonical graph from same analysis");
    let ssa = ir.ssa().expect("SSA from same analysis");
    let code = ir.code().expect("complete Code facts from same analysis");
    let pool = ir.constant_pool();
    let operations = Operations::of(code, pool);
    let mut view_budget = reader_budget();
    let view = NormalFlowView::build(canonical, &mut view_budget)
        .expect("normal-flow view from same canonical graph");
    f(canonical, ssa, code, pool, &view, &operations)
}

fn make_walker<'a>(
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

fn loop_at<'a>(
    view: &'a NormalFlowView,
    header_bci: u32,
) -> (&'a CanonicalBlockId, &'a BTreeSet<usize>, usize) {
    (0..view.len())
        .find_map(|node| {
            let natural = view.loop_entered_at(node)?;
            let header = view.id_of(node)?;
            (natural.header() == node && header.bci() == header_bci).then_some((
                header,
                natural.blocks(),
                node,
            ))
        })
        .expect("the pinned target Code has this natural loop")
}

fn entry_node(view: &NormalFlowView, header_node: usize, blocks: &BTreeSet<usize>) -> usize {
    let entries: Vec<_> = view
        .predecessors(header_node)
        .into_iter()
        .filter(|node| !blocks.contains(node))
        .collect();
    let [entry] = entries.as_slice() else {
        panic!("candidate has one external predecessor")
    };
    *entry
}

fn assert_break_arm(region: &Region, expected_gateway: Option<(u32, u32)>) {
    let Region::Loop { body, .. } = region else {
        panic!("proof returns the existing Loop region")
    };
    let [
        Region::If {
            branch_bci,
            then_arm,
            else_arm,
            ..
        },
        Region::Straight { .. },
    ] = body.as_slice()
    else {
        panic!("candidate uses the existing If + Straight body")
    };
    if let Some((gateway_bci, break_bci)) = expected_gateway {
        let arm = [then_arm.as_ref(), else_arm.as_ref()]
            .into_iter()
            .find(|arm| matches!(arm, Region::Sequence { .. }))
            .expect("pure exit gateway is retained in a sequence arm");
        let Region::Sequence { regions } = arm else {
            unreachable!()
        };
        let other = if matches!(then_arm.as_ref(), Region::Sequence { .. }) {
            else_arm.as_ref()
        } else {
            then_arm.as_ref()
        };
        assert!(matches!(other, Region::Straight { blocks } if blocks.is_empty()));
        let [
            Region::Straight { blocks },
            Region::LoopBreak { source_bci, .. },
        ] = regions.as_slice()
        else {
            panic!("gateway source and loop break stay in one arm")
        };
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].bci(), gateway_bci);
        assert_eq!(*source_bci, break_bci);
        assert_eq!(*branch_bci, 11);
    }
}

#[test]
fn original_debug_and_no_debug_keep_gateway_and_latch_sources() {
    for bytes in [ORIGINAL_DEBUG, ORIGINAL_NO_DEBUG] {
        with_method(
            bytes,
            b"test",
            b"(II)I",
            |canonical, ssa, code, pool, view, operations| {
                let (header, blocks, header_node) = loop_at(view, 2);
                let entry = entry_node(view, header_node, blocks);
                let chains =
                    crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
                let sites = crate::init::Sites::empty();
                let mut budget = proof_budget(1 << 20);
                let mut walker = make_walker(
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
                walker.visited.insert(entry);
                let (regions, next) = walker
                    .header_tested_loop(header, header_node, blocks, &Frame::default())
                    .expect("the exact StatementFree refusal reaches the local consumer")
                    .expect("the complete candidate produces a loop");
                assert_eq!(next.as_ref().map(CanonicalBlockId::bci), Some(23));
                let mut expected_owners = blocks.clone();
                let gateway_node = view
                    .ids()
                    .iter()
                    .find(|id| id.bci() == 14)
                    .and_then(|id| view.index_of(id))
                    .expect("gateway BCI 14 is in the exact graph");
                expected_owners.insert(gateway_node);
                expected_owners.insert(entry);
                assert_eq!(walker.visited, expected_owners);
                let [region] = regions.as_slice() else {
                    panic!("one loop region")
                };
                let Region::Loop {
                    exit,
                    gateway_origins,
                    ..
                } = region
                else {
                    panic!("existing Loop representation")
                };
                assert_eq!(exit.as_ref().map(CanonicalBlockId::bci), Some(23));
                assert_eq!(gateway_origins, &[20]);
                assert_break_arm(region, Some((14, 14)));
            },
        );
    }
}

#[test]
fn supplemental_profiles_keep_a_direct_exit_without_a_gateway() {
    for bytes in [
        SUPPLEMENT_JAVAC8_DEBUG,
        SUPPLEMENT_JAVAC8_NO_DEBUG,
        SUPPLEMENT_JAVAC23_DEBUG,
        SUPPLEMENT_JAVAC23_NO_DEBUG,
    ] {
        with_method(
            bytes,
            b"run",
            b"(II)I",
            |canonical, ssa, code, pool, view, operations| {
                let mut accepted = Vec::new();
                for node in 0..view.len() {
                    let Some(natural) = view.loop_entered_at(node) else {
                        continue;
                    };
                    if natural.header() != node {
                        continue;
                    }
                    let Some(header) = view.id_of(node) else {
                        continue;
                    };
                    let blocks = natural.blocks();
                    let Some(test_bci) = ssa
                        .block(header)
                        .and_then(|block| block.instructions().last())
                        .map(|i| i.bci())
                    else {
                        continue;
                    };
                    if operations
                        .get(test_bci)
                        .and_then(Operation::comparison)
                        .is_none()
                    {
                        continue;
                    }
                    let outside: Vec<_> = view
                        .successors(node)
                        .into_iter()
                        .filter(|next| !blocks.contains(next))
                        .collect();
                    let [outside_node] = outside.as_slice() else {
                        continue;
                    };
                    let Some(outside_id) = view.id_of(*outside_node) else {
                        continue;
                    };
                    let Some(outside_bci) = ssa
                        .block(outside_id)
                        .and_then(|block| block.instructions().last())
                        .map(|i| i.bci())
                    else {
                        continue;
                    };
                    if matches!(operations.get(outside_bci), Some(Operation::Transfer)) {
                        continue;
                    }
                    let entry = entry_node(view, node, blocks);
                    let chains =
                        crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
                    let sites = crate::init::Sites::empty();
                    let mut budget = proof_budget(1 << 20);
                    let mut walker = make_walker(
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
                    walker.visited.insert(entry);
                    if let Some((regions, Some(exit))) = walker
                        .statementful_loop_header(header, node, blocks, &Frame::default())
                        .expect("a refusal is local; Stop remains an error")
                    {
                        if exit == *outside_id {
                            accepted.push((header.bci(), regions));
                        }
                    }
                }
                assert_eq!(
                    accepted
                        .iter()
                        .map(|(header, _)| *header)
                        .collect::<Vec<_>>(),
                    vec![10, 51],
                    "both actual loop headers have a direct outside successor"
                );
                for (header_bci, regions) in accepted {
                    let [region] = regions.as_slice() else {
                        panic!("one direct-exit loop")
                    };
                    let Region::Loop {
                        gateway_origins, ..
                    } = region
                    else {
                        panic!("Loop")
                    };
                    assert_eq!(
                        gateway_origins.as_slice(),
                        if header_bci == 10 {
                            &[43, 32][..]
                        } else {
                            &[78][..]
                        },
                        "the fused exit goto is attributed without moving its successor statements"
                    );
                    let Region::Loop { body, .. } = region else {
                        unreachable!()
                    };
                    let [
                        Region::If {
                            branch_bci,
                            then_arm,
                            else_arm,
                            ..
                        },
                        Region::Straight { .. },
                    ] = body.as_slice()
                    else {
                        unreachable!()
                    };
                    assert!([then_arm.as_ref(), else_arm.as_ref()].iter().any(|arm| matches!(arm, Region::LoopBreak { source_bci, .. } if source_bci == branch_bci)));
                }
            },
        );
    }
}

#[test]
fn unsupported_contexts_missing_facts_and_visited_gateway_refuse_without_claims() {
    with_method(
        ORIGINAL_DEBUG,
        b"test",
        b"(II)I",
        |canonical, ssa, code, pool, view, operations| {
            let (header, blocks, header_node) = loop_at(view, 2);
            let entry = entry_node(view, header_node, blocks);
            let gateway = view
                .successors(header_node)
                .into_iter()
                .find(|node| !blocks.contains(node))
                .expect("one external gateway");
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            let mut budget = proof_budget(1 << 20);
            let mut wrong_frame = make_walker(
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
            wrong_frame.visited.insert(entry);
            assert!(
                wrong_frame
                    .statementful_loop_header(
                        header,
                        header_node,
                        blocks,
                        &Frame {
                            boundary: Some(gateway),
                            ..Frame::default()
                        }
                    )
                    .expect("boundary is a local refusal")
                    .is_none()
            );
            assert_eq!(wrong_frame.visited, BTreeSet::from([entry]));

            let mut visited_gateway = make_walker(
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
            visited_gateway.visited.insert(entry);
            visited_gateway.visited.insert(gateway);
            assert!(
                visited_gateway
                    .statementful_loop_header(header, header_node, blocks, &Frame::default())
                    .expect("claimed gateway is a local refusal")
                    .is_none()
            );
            assert_eq!(visited_gateway.visited, BTreeSet::from([entry, gateway]));

            // The physical target offset alone is insufficient: a mismatched instruction fact must
            // not inherit the original SSA/Operation branch classification. Keep its operand offset
            // and all graph facts intact while changing only the decoded physical opcode.
            let branch_bci = ssa
                .block(header)
                .and_then(|block| block.instructions().last())
                .map(|instruction| instruction.bci())
                .expect("header has a terminal branch instruction");
            let mut mismatched_code = code.clone();
            let physical_branch = mismatched_code
                .instructions
                .iter_mut()
                .find(|instruction| instruction.bci == branch_bci)
                .expect("physical Code facts contain the branch BCI");
            assert_ne!(physical_branch.opcode, 0x00);
            physical_branch.opcode = 0x00; // nop, while the same operand offset remains present.
            let mut mismatched_physical = make_walker(
                canonical,
                view,
                ssa,
                &mismatched_code,
                pool,
                operations,
                &chains,
                &sites,
                &mut budget,
            );
            mismatched_physical.visited.insert(entry);
            assert!(
                mismatched_physical
                    .statementful_loop_header(header, header_node, blocks, &Frame::default())
                    .expect("physical opcode disagreement is a local refusal")
                    .is_none()
            );
            assert_eq!(mismatched_physical.visited, BTreeSet::from([entry]));

            // A gateway must also agree across physical and semantic facts. Keep the physical goto
            // offset while changing its raw opcode to nop; the proof must not emit a source-bearing
            // gateway/LoopBreak from a mismatched operation.
            let gateway_bci = view
                .id_of(gateway)
                .expect("gateway node has identity")
                .bci();
            let mut mismatched_gateway_code = code.clone();
            let physical_gateway = mismatched_gateway_code
                .instructions
                .iter_mut()
                .find(|instruction| instruction.bci == gateway_bci)
                .expect("physical Code facts contain the gateway BCI");
            assert_ne!(physical_gateway.opcode, 0x00);
            physical_gateway.opcode = 0x00;
            let mut mismatched_gateway = make_walker(
                canonical,
                view,
                ssa,
                &mismatched_gateway_code,
                pool,
                operations,
                &chains,
                &sites,
                &mut budget,
            );
            mismatched_gateway.visited.insert(entry);
            assert!(
                mismatched_gateway
                    .statementful_loop_header(header, header_node, blocks, &Frame::default())
                    .expect("physical gateway opcode disagreement is a local refusal")
                    .is_none()
            );
            assert_eq!(mismatched_gateway.visited, BTreeSet::from([entry]));

            // Cross-method Reader facts are deliberate proof-context mutations, not JVM fixtures.
            // The original method's code is below paired with its constructor code, so the branch BCI
            // has no matching physical operand. The same mismatch with constructor SSA checks missing
            // block identity without manufacturing a CanonicalCfg edge.
            with_method(
                ORIGINAL_DEBUG,
                b"<init>",
                b"()V",
                |foreign_canonical,
                 foreign_ssa,
                 foreign_code,
                 _foreign_pool,
                 foreign_view,
                 foreign_ops| {
                    let foreign_chains = crate::concat::plan(
                        foreign_ssa,
                        foreign_ops,
                        &crate::build::FieldCopies::default(),
                    );
                    let foreign_sites = crate::init::Sites::empty();
                    let mut foreign_budget = proof_budget(1 << 20);
                    let mut missing_physical = make_walker(
                        canonical,
                        view,
                        ssa,
                        foreign_code,
                        pool,
                        operations,
                        &chains,
                        &sites,
                        &mut foreign_budget,
                    );
                    missing_physical.visited.insert(entry);
                    assert!(
                        missing_physical
                            .statementful_loop_header(
                                header,
                                header_node,
                                blocks,
                                &Frame::default()
                            )
                            .expect("missing physical target refuses")
                            .is_none()
                    );
                    assert_eq!(missing_physical.visited, BTreeSet::from([entry]));

                    let mut missing_ssa = make_walker(
                        canonical,
                        view,
                        foreign_ssa,
                        code,
                        pool,
                        operations,
                        &foreign_chains,
                        &foreign_sites,
                        &mut foreign_budget,
                    );
                    missing_ssa.visited.insert(entry);
                    assert!(
                        missing_ssa
                            .statementful_loop_header(
                                header,
                                header_node,
                                blocks,
                                &Frame::default()
                            )
                            .expect("missing SSA block refuses")
                            .is_none()
                    );
                    assert_eq!(missing_ssa.visited, BTreeSet::from([entry]));

                    // A different same-class NormalFlowView is an explicit foreign proof context.
                    let mut foreign_view_walker = make_walker(
                        canonical,
                        foreign_view,
                        ssa,
                        code,
                        pool,
                        operations,
                        &chains,
                        &sites,
                        &mut foreign_budget,
                    );
                    foreign_view_walker.visited.insert(entry);
                    assert!(
                        foreign_view_walker
                            .statementful_loop_header(
                                header,
                                header_node,
                                blocks,
                                &Frame::default()
                            )
                            .expect("foreign view identity refuses")
                            .is_none()
                    );
                    assert_eq!(foreign_view_walker.visited, BTreeSet::from([entry]));

                    let mut foreign_canonical_walker = make_walker(
                        foreign_canonical,
                        view,
                        ssa,
                        code,
                        pool,
                        operations,
                        &chains,
                        &sites,
                        &mut foreign_budget,
                    );
                    foreign_canonical_walker.visited.insert(entry);
                    assert!(
                        foreign_canonical_walker
                            .statementful_loop_header(
                                header,
                                header_node,
                                blocks,
                                &Frame::default()
                            )
                            .expect("foreign canonical rows refuse the expected owner closure")
                            .is_none()
                    );
                    assert_eq!(foreign_canonical_walker.visited, BTreeSet::from([entry]));
                },
            );
        },
    );
}

#[test]
fn canonical_edge_budget_cancel_and_deadline_stops_leave_owners_unclaimed() {
    with_method(
        ORIGINAL_DEBUG,
        b"test",
        b"(II)I",
        |canonical, ssa, code, pool, view, operations| {
            let (header, blocks, header_node) = loop_at(view, 2);
            let entry = entry_node(view, header_node, blocks);
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            let mut full_budget = proof_budget(u64::MAX);
            let mut full = make_walker(
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
            full.visited.insert(entry);
            let proof = full
                .statementful_loop_header(header, header_node, blocks, &Frame::default())
                .expect("unlimited proof completes");
            assert!(
                proof.is_some(),
                "the exact candidate is accepted before measuring its proof cost"
            );
            let full_cost = full.budget.usage().analysis_steps;
            assert!(full_cost > 0);
            let last_edge_bci = canonical
                .edges()
                .last()
                .expect("canonical graph has edges")
                .from()
                .bci();

            let mut budget = proof_budget(full_cost - 1);
            let mut stopped = make_walker(
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
            stopped.visited.insert(entry);
            let result =
                stopped.statementful_loop_header(header, header_node, blocks, &Frame::default());
            assert!(
                matches!(
                    result,
                    Err(StopReason::Budget {
                        dimension: CountedBudgetDimension::AnalysisSteps,
                        at: Some(at),
                        ..
                    }) if at == last_edge_bci
                ),
                "cost minus one stops on the final canonical edge row: {result:?}"
            );
            assert_eq!(stopped.visited, BTreeSet::from([entry]));
            assert!(stopped.catch_joins.is_empty());
            assert!(stopped.loop_catch_terminal_owners.is_empty());
            assert!(stopped.fragmented.is_none());
            assert_eq!(stopped.unclosed_tail_at, None);

            let token = CancellationToken::new();
            let mut budget = Budget::with_cancellation_token(
                Limits {
                    analysis_steps: u64::MAX,
                    elapsed_millis: u64::MAX,
                    ..Limits::default()
                },
                token.clone(),
            );
            token.cancel();
            let mut cancelled = make_walker(
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
            cancelled.visited.insert(entry);
            assert!(matches!(
                cancelled.statementful_loop_header(header, header_node, blocks, &Frame::default()),
                Err(StopReason::Cancelled { .. })
            ));
            assert_eq!(cancelled.visited, BTreeSet::from([entry]));

            let mut budget = Budget::new(Limits {
                analysis_steps: u64::MAX,
                elapsed_millis: 1,
                ..Limits::default()
            });
            std::thread::sleep(std::time::Duration::from_millis(2));
            let mut expired = make_walker(
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
            expired.visited.insert(entry);
            assert!(matches!(
                expired.statementful_loop_header(header, header_node, blocks, &Frame::default()),
                Err(StopReason::Interrupted {
                    code: crate::stop::BUDGET_INTERRUPTED_CODE,
                    ..
                })
            ));
            assert_eq!(expired.visited, BTreeSet::from([entry]));
        },
    );
}
const EXTRA_EXIT_AND_LATCH_CLASS: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-statementful-loop-headers/boundaries/javac8-debug/StatementfulLoopBoundary.class"
);
const DUAL_ENTRY_CLASS: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-statementful-loop-headers/boundaries/v49/StatementfulDualEntry.class"
);
const JSR_CLONE_HEADER_CLASS: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-statementful-loop-headers/boundaries/v49/StatementfulJsrCloneHeader.class"
);

fn assert_natural_boundary(class: &[u8], method: &[u8], owners: &[u32], latches: &[u32]) {
    with_method(
        class,
        method,
        b"(I)I",
        |canonical, ssa, code, pool, view, operations| {
            let (header, blocks, node) = loop_at(view, 2);
            let mut actual_owners = blocks
                .iter()
                .map(|n| view.id_of(*n).unwrap().bci())
                .collect::<Vec<_>>();
            actual_owners.sort_unstable();
            assert_eq!(actual_owners, owners);
            let natural = view.loop_entered_at(node).unwrap();
            let mut actual_latches = natural
                .latches()
                .iter()
                .map(|n| view.id_of(*n).unwrap().bci())
                .collect::<Vec<_>>();
            actual_latches.sort_unstable();
            assert_eq!(actual_latches, latches);
            if method == b"extraBodyExit" {
                assert!(
                    blocks.iter().any(|n| *n != node
                        && view
                            .successors(*n)
                            .iter()
                            .any(|next| !blocks.contains(next))),
                    "the real body has an exit in addition to the header test"
                );
            }
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            let mut budget = proof_budget(1 << 20);
            let mut walker = make_walker(
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
            walker.visited.insert(entry_node(view, node, blocks));
            let before = walker.visited.clone();
            assert!(
                walker
                    .statementful_loop_header(header, node, blocks, &Frame::default())
                    .expect("the real boundary is a local refusal")
                    .is_none()
            );
            assert_eq!(walker.visited, before);
        },
    );
}

#[test]
fn real_effectful_header_extra_body_exit_and_multiple_latches_refuse_directly() {
    assert_natural_boundary(
        EXTRA_EXIT_AND_LATCH_CLASS,
        b"extraBodyExit",
        &[2, 16, 24],
        &[24],
    );
    assert_natural_boundary(
        EXTRA_EXIT_AND_LATCH_CLASS,
        b"multipleLatches",
        &[2, 16, 22, 28],
        &[22, 28],
    );
}

#[test]
fn actual_dual_entry_view_is_recorded_and_any_natural_candidates_refuse() {
    with_method(
        DUAL_ENTRY_CLASS,
        b"run",
        b"(II)I",
        |_canonical, _ssa, _code, _pool, view, _operations| {
            // The genuine second body entry prevents domination: this view supplies no natural-loop
            // candidate at all. This is an upstream ownership boundary, not a fabricated helper call.
            assert!(view.ids().iter().any(|id| id.bci() == 9));
            assert!((0..view.len()).all(|node| view.loop_entered_at(node).is_none()));
        },
    );
}

#[test]
fn actual_jsr_clone_bci23_can_be_used_only_as_a_cross_context_negative() {
    let clones = with_method(
        JSR_CLONE_HEADER_CLASS,
        b"run",
        b"(I)I",
        |_canonical, ssa, _code, _pool, view, _operations| {
            let ids = ssa
                .blocks()
                .iter()
                .filter_map(|block| {
                    let id = block.block();
                    (id.bci() == 23 && id.is_clone()).then(|| id.clone())
                })
                .collect::<Vec<_>>();
            assert_eq!(ids.len(), 2);
            assert_eq!(
                ids.iter().map(|id| id.path().to_vec()).collect::<Vec<_>>(),
                vec![vec![2], vec![5]]
            );
            for id in &ids {
                assert!(view.loop_entered_at(view.index_of(id).unwrap()).is_none());
            }
            ids
        },
    );
    // These are two real Reader-issued clone IDs. Mixing them with another analysis is explicitly
    // a proof-context negative, not a legal JVM graph or evidence of whole jsr-loop recovery.
    with_method(
        ORIGINAL_DEBUG,
        b"test",
        b"(II)I",
        |canonical, ssa, code, pool, view, operations| {
            let (header, blocks, node) = loop_at(view, 23);
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            for cloned in &clones {
                assert_eq!(header.bci(), cloned.bci());
                assert_ne!(header.path(), cloned.path());
                let mut budget = proof_budget(1 << 20);
                let mut walker = make_walker(
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
                walker.visited.insert(entry_node(view, node, blocks));
                let before = walker.visited.clone();
                assert!(
                    walker
                        .statementful_loop_header(cloned, node, blocks, &Frame::default())
                        .expect("wrong clone identity is a local refusal")
                        .is_none()
                );
                assert_eq!(walker.visited, before);
            }
        },
    );
}

const COPY_CONTROLS_V8: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-loop-test-copy-store/v8/ProbeControls.class"
);
const COPY_CONTROLS_JAVAC8: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-loop-test-copy-store/v8-javac8/ProbeControls.class"
);
const MULTI_COPY_V8: &[u8] =
    include_bytes!("../../../../tests/fixtures/recover-loop-test-copy-store/v8/MultiCopy.class");
const MULTI_COPY_JAVAC8: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-loop-test-copy-store/v8-javac8/MultiCopy.class"
);

/// Find only real natural-loop headers whose own terminal is a comparison and whose header SSA
/// contains an adjacent decoded `dup; store` pair. This discovers the Reader's facts; it does not
/// prescribe how many SSA uses the duplicate must have.
fn copy_test_headers<'a>(
    view: &'a NormalFlowView,
    ssa: &'a SsaTable,
    operations: &Operations,
) -> Vec<(&'a CanonicalBlockId, &'a BTreeSet<usize>, usize)> {
    let mut candidates = Vec::new();
    for node in 0..view.len() {
        let Some(natural) = view.loop_entered_at(node) else {
            continue;
        };
        if natural.header() != node {
            continue;
        }
        let Some(header) = view.id_of(node) else {
            continue;
        };
        let Some(names) = ssa.block(header) else {
            continue;
        };
        let Some(test) = names.instructions().last() else {
            continue;
        };
        if operations
            .get(test.bci())
            .and_then(Operation::comparison)
            .is_none()
        {
            continue;
        }
        let adjacent_copy_store = names.instructions().windows(2).any(|pair| {
            matches!(operations.get(pair[0].bci()), Some(Operation::Duplicate))
                && matches!(operations.get(pair[1].bci()), Some(Operation::Store { .. }))
        });
        if adjacent_copy_store {
            candidates.push((header, natural.blocks(), node));
        }
    }
    candidates
}

fn print_copy_test_facts(
    label: &str,
    header: &CanonicalBlockId,
    blocks: &BTreeSet<usize>,
    node: usize,
    view: &NormalFlowView,
    ssa: &SsaTable,
    operations: &Operations,
) {
    let mut block_ids = blocks
        .iter()
        .filter_map(|node| view.id_of(*node).cloned())
        .collect::<Vec<_>>();
    block_ids.sort();
    let latches = view
        .loop_entered_at(node)
        .expect("candidate is a real natural loop")
        .latches()
        .iter()
        .filter_map(|node| view.id_of(*node).cloned())
        .collect::<Vec<_>>();
    eprintln!("copy-refusal {label}: header={header:?} natural={block_ids:?} latches={latches:?}");
    let names = ssa.block(header).expect("candidate header has SSA");
    for instruction in names.instructions() {
        eprintln!(
            "copy-refusal {label}: bci={} opcode=0x{:02x} operation={:?} reads={:?} writes={:?}",
            instruction.bci(),
            instruction.opcode(),
            operations.get(instruction.bci()),
            instruction.reads(),
            instruction.writes(),
        );
        for (_, value_id) in instruction.reads().iter().chain(instruction.writes()) {
            let value = ssa.value(*value_id);
            eprintln!(
                "copy-refusal {label}: value={value_id:?} def={:?} uses={:?} replaced={:?}",
                value.def(),
                value.uses(),
                value.replaced_by(),
            );
        }
    }
}

fn assert_copy_header_refuses_without_claim(
    bytes: &[u8],
    method_name: &[u8],
    descriptor: &[u8],
    label: &str,
) {
    with_method(
        bytes,
        method_name,
        descriptor,
        |canonical, ssa, code, pool, view, operations| {
            assert!(
                canonical.completeness().is_complete(),
                "{label}: full canonical graph"
            );
            assert!(code.stopped_at.is_none(), "{label}: complete physical Code");
            assert_eq!(
                code.instructions.len(),
                code.operands().len(),
                "{label}: physical facts"
            );
            assert_eq!(
                code.exception_handler_count, 0,
                "{label}: this is the no-handler parameter/copy control"
            );
            let candidates = copy_test_headers(view, ssa, operations);
            assert!(
                !candidates.is_empty(),
                "{label}: Reader found the real loop-test copy candidate"
            );
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            for (header, blocks, node) in candidates {
                print_copy_test_facts(label, header, blocks, node, view, ssa, operations);
                let entry = entry_node(view, node, blocks);
                let mut budget = proof_budget(1 << 20);
                let mut walker = make_walker(
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
                walker.visited.insert(entry);
                let before = (
                    walker.visited.clone(),
                    walker.catch_joins.clone(),
                    format!("{:?}", walker.loop_catch_terminal_owners),
                    format!("{:?}", walker.fragmented),
                    walker.unclosed_tail_at,
                    walker.excluded_edge_nodes.clone(),
                    walker.depth,
                );
                let result =
                    walker.statementful_loop_header(header, node, blocks, &Frame::default());
                assert!(
                    matches!(result, Ok(None)),
                    "{label}: the full Reader candidate is refused before any claim: {result:?}"
                );
                let after = (
                    walker.visited.clone(),
                    walker.catch_joins.clone(),
                    format!("{:?}", walker.loop_catch_terminal_owners),
                    format!("{:?}", walker.fragmented),
                    walker.unclosed_tail_at,
                    walker.excluded_edge_nodes.clone(),
                    walker.depth,
                );
                assert_eq!(
                    after, before,
                    "{label}: refusal leaves walker state unchanged"
                );
            }
        },
    );
}

#[test]
fn actual_parameter_and_multi_copy_loop_test_candidates_refuse_without_claims() {
    for (bytes, label) in [
        (COPY_CONTROLS_V8, "ProbeControls javac23 parameterTarget"),
        (COPY_CONTROLS_JAVAC8, "ProbeControls javac8 parameterTarget"),
    ] {
        assert_copy_header_refuses_without_claim(
            bytes,
            b"parameterTarget",
            b"(ILjava/lang/String;)I",
            label,
        );
    }
    for (bytes, label) in [
        (MULTI_COPY_V8, "MultiCopy javac23 readAll"),
        (MULTI_COPY_JAVAC8, "MultiCopy javac8 readAll"),
    ] {
        assert_copy_header_refuses_without_claim(
            bytes,
            b"readAll",
            b"(Ljava/lang/String;)Ljava/lang/String;",
            label,
        );
    }
}

#[test]
fn real_sequential_loop_copy_candidate_accepts_and_stops_atomically() {
    with_method(
        ORIGINAL_DEBUG,
        b"test",
        b"(II)I",
        |canonical, ssa, code, pool, view, operations| {
            assert!(canonical.completeness().is_complete());
            let (header, blocks, node) = loop_at(view, 23);
            print_copy_test_facts(
                "Seq1 header23 direct proof-context positive",
                header,
                blocks,
                node,
                view,
                ssa,
                operations,
            );
            let names = ssa.block(header).expect("real header has SSA");
            assert!(
                names.instructions().windows(2).any(|pair| {
                    matches!(operations.get(pair[0].bci()), Some(Operation::Duplicate))
                        && matches!(operations.get(pair[1].bci()), Some(Operation::Store { .. }))
                }),
                "the exact Reader candidate contains its actual adjacent dup/store pair"
            );
            let entry = entry_node(view, node, blocks);
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();

            // This deliberately exercises the private consumer directly. Ordinary pure-condition
            // recovery does not need to call this helper; it remains on the existing path.
            let mut full_budget = proof_budget(u64::MAX);
            let mut full = make_walker(
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
            full.visited.insert(entry);
            let full_result = full
                .statementful_loop_header(header, node, blocks, &Frame::default())
                .expect("the complete real candidate finishes");
            assert!(
                full_result.is_some(),
                "the proved local assignment candidate is admitted"
            );
            let cost = full.budget.usage().analysis_steps;
            assert!(cost > 0);

            let duplicate_bcis = names
                .instructions()
                .iter()
                .filter(|instruction| {
                    matches!(
                        operations.get(instruction.bci()),
                        Some(Operation::Duplicate)
                    )
                })
                .map(|instruction| instruction.bci())
                .collect::<BTreeSet<_>>();
            let mut copy_stop = None;
            for limit in 0..cost {
                let mut budget = proof_budget(limit);
                let mut stopped = make_walker(
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
                stopped.visited.insert(entry);
                let result =
                    stopped.statementful_loop_header(header, node, blocks, &Frame::default());
                if matches!(&result, Err(StopReason::Budget {
                    dimension: CountedBudgetDimension::AnalysisSteps, at: Some(at), ..
                }) if duplicate_bcis.contains(at))
                {
                    assert_eq!(stopped.visited, BTreeSet::from([entry]));
                    assert!(stopped.catch_joins.is_empty());
                    assert!(stopped.loop_catch_terminal_owners.is_empty());
                    assert!(stopped.fragmented.is_none());
                    assert_eq!(stopped.unclosed_tail_at, None);
                    copy_stop = Some(limit);
                    break;
                }
            }
            assert!(
                copy_stop.is_some(),
                "a real budget stop reaches the new copy-proof charge"
            );
            eprintln!(
                "copy-refusal Seq1 direct proof-context: complete_cost={cost} copy_stop_limit={copy_stop:?}"
            );

            let mut limited_budget = proof_budget(cost - 1);
            let mut limited = make_walker(
                canonical,
                view,
                ssa,
                code,
                pool,
                operations,
                &chains,
                &sites,
                &mut limited_budget,
            );
            limited.visited.insert(entry);
            let before = (
                limited.visited.clone(),
                limited.catch_joins.clone(),
                format!("{:?}", limited.loop_catch_terminal_owners),
                format!("{:?}", limited.fragmented),
                limited.unclosed_tail_at,
                limited.excluded_edge_nodes.clone(),
                limited.depth,
            );
            let result = limited.statementful_loop_header(header, node, blocks, &Frame::default());
            assert!(
                matches!(
                    result,
                    Err(StopReason::Budget {
                        dimension: CountedBudgetDimension::AnalysisSteps,
                        ..
                    })
                ),
                "one less than the measured complete proof cost stops: {result:?}"
            );
            let after = (
                limited.visited.clone(),
                limited.catch_joins.clone(),
                format!("{:?}", limited.loop_catch_terminal_owners),
                format!("{:?}", limited.fragmented),
                limited.unclosed_tail_at,
                limited.excluded_edge_nodes.clone(),
                limited.depth,
            );
            assert_eq!(after, before, "the budget stop publishes no partial claim");
        },
    );
}
