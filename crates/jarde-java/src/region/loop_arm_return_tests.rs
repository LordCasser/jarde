//! Reader-backed ownership and stop boundaries for terminal returns in loop arms.
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
use std::collections::BTreeSet;

const LOOP_EXCEPTION: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/boundary/javac23/LoopArmExceptionBoundary.class"
);

const ORIGINAL_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/original/debug/classes/jadx/tests/integration/loops/TestNestedLoops5$TestCls.class"
);
const ORIGINAL_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/original/no-debug/classes/jadx/tests/integration/loops/TestNestedLoops5$TestCls.class"
);
const A_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/A/javac8/debug/classes/NestedReturnA$TestCls.class"
);
const A_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/A/javac8/no-debug/classes/NestedReturnA$TestCls.class"
);
const A_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/A/javac23/debug/classes/NestedReturnA$TestCls.class"
);
const A_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/A/javac23/no-debug/classes/NestedReturnA$TestCls.class"
);
const B_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/B/javac8/debug/classes/NestedReturnB$TestCls.class"
);
const B_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/B/javac8/no-debug/classes/NestedReturnB$TestCls.class"
);
const B_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/B/javac23/debug/classes/NestedReturnB$TestCls.class"
);
const B_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/B/javac23/no-debug/classes/NestedReturnB$TestCls.class"
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

fn proof_budget(steps: u64) -> Budget {
    Budget::new(Limits {
        analysis_steps: steps,
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
        .expect("whole frozen class parses");
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
        loader: LoaderId("app".into()),
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
    .expect("Reader analyzes the exact original method");
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("real canonical CFG");
    let ssa = ir.ssa().expect("real SSA");
    let code = ir.code().expect("real Code facts");
    let pool = ir.constant_pool();
    let operations = Operations::of(code, pool);
    let mut view_budget = reader_budget();
    let view =
        NormalFlowView::build(canonical, &mut view_budget).expect("real normal-flow projection");
    f(canonical, ssa, code, pool, &view, &operations)
}

fn with_canonical_without_ssa<R>(
    bytes: &[u8],
    name: &[u8],
    descriptor: &[u8],
    f: impl FnOnce(&CanonicalCfg, &MethodCodeFacts, &[CpEntryFacts], &NormalFlowView, &Operations) -> R,
) -> R {
    let mut budget = reader_budget();
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("whole frozen class parses");
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
        loader: LoaderId("app".into()),
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
    .expect("Reader analyzes the exact original method");
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("real canonical CFG");
    let code = ir.code().expect("real Code facts");
    let pool = ir.constant_pool();
    let operations = Operations::of(code, pool);
    let mut view_budget = reader_budget();
    let view =
        NormalFlowView::build(canonical, &mut view_budget).expect("real normal-flow projection");
    f(canonical, code, pool, &view, &operations)
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

struct Candidate {
    arm: Region,
    branch: CanonicalBlockId,
    entry: CanonicalBlockId,
    join: CanonicalBlockId,
    header: CanonicalBlockId,
    owners: BTreeSet<usize>,
    return_nodes: BTreeSet<usize>,
}

fn collect_loop_regions<'a>(region: &'a Region, out: &mut Vec<&'a Region>) {
    match region {
        Region::Sequence { regions } => {
            for child in regions {
                collect_loop_regions(child, out);
            }
        }
        Region::Loop { body, .. } => {
            out.push(region);
            for child in body {
                collect_loop_regions(child, out);
            }
        }
        Region::If {
            then_arm, else_arm, ..
        }
        | Region::CompoundIf {
            then_arm, else_arm, ..
        } => {
            collect_loop_regions(then_arm, out);
            collect_loop_regions(else_arm, out);
        }
        Region::Switch { groups, .. } | Region::StringSwitch { groups, .. } => {
            for group in groups {
                collect_loop_regions(&group.arm, out);
            }
        }
        Region::ShortCircuitValue { tail, .. } => {
            for child in tail {
                collect_loop_regions(child, out);
            }
        }
        Region::Guard {
            body, finally_body, ..
        } => {
            if let Some(body) = body {
                collect_loop_regions(body, out);
            }
            if let Some(body) = finally_body {
                collect_loop_regions(body, out);
            }
        }
        Region::Try { body, catches, .. } => {
            collect_loop_regions(body, out);
            for clause in catches {
                collect_loop_regions(&clause.body, out);
            }
        }
        Region::Straight { .. }
        | Region::TwoExitReturn { .. }
        | Region::SharedTailEarlyReturn { .. }
        | Region::LoopBreak { .. }
        | Region::LoopContinue { .. }
        | Region::SwitchBreak { .. }
        | Region::EntrySplitLoop { .. }
        | Region::Fallback { .. } => {}
    }
}

fn real_candidate(
    canonical: &CanonicalCfg,
    ssa: &SsaTable,
    code: &MethodCodeFacts,
    pool: &[CpEntryFacts],
    view: &NormalFlowView,
    operations: &Operations,
    chains: &crate::concat::Plan,
    sites: &crate::init::Sites,
) -> Candidate {
    // Use the exact same full region recovery that the public request exercises. Calling
    // loop_region(header) with a default Frame invents an incomplete child context for these
    // nested-loop fixtures, which omitted the outer loop's return ownership.
    let mut recovery_budget = proof_budget(1 << 20);
    let recovered = recover(
        canonical,
        view,
        ssa,
        operations,
        pool,
        chains,
        sites,
        code,
        Some(false),
        false,
        &crate::pass::JAVA_8,
        &mut recovery_budget,
    )
    .expect("full Reader region recovery completes for the public positive fixture");
    assert!(
        recovered.is_structured(),
        "all public positive fixture methods remain structured"
    );
    let mut loops = Vec::new();
    for region in &recovered.regions {
        collect_loop_regions(region, &mut loops);
    }
    let mut choices = Vec::new();
    for arm in loops {
        let Region::Loop {
            header,
            exit: Some(join),
            gateway_origins,
            for_header,
            ..
        } = arm
        else {
            continue;
        };
        let Some(header_node) = view.index_of(header) else {
            continue;
        };
        let Some(natural) = view.loop_entered_at(header_node) else {
            continue;
        };
        let Some(join_node) = view.index_of(join) else {
            continue;
        };
        let mut proof_budget = proof_budget(1 << 20);
        let mut w = walker(
            canonical,
            view,
            ssa,
            code,
            pool,
            operations,
            chains,
            sites,
            &mut proof_budget,
        );
        let returns = w
            .loop_terminal_returns(natural.blocks(), Some(join_node), &Frame::default())
            .expect("real terminal-return census completes");
        if returns.is_empty() {
            continue;
        }
        let arm_blocks = arm.blocks();
        let owners: BTreeSet<_> = arm_blocks
            .iter()
            .filter_map(|id| view.index_of(id))
            .collect();
        if owners.len() != arm_blocks.len() {
            continue;
        }
        let expected: BTreeSet<_> = natural.blocks().union(&returns).copied().collect();
        if owners != expected {
            continue;
        }
        if returns
            .iter()
            .any(|node| natural.blocks().contains(node) || !view.successors(*node).is_empty())
        {
            continue;
        }
        let normal_entries: Vec<_> = canonical
            .edges()
            .iter()
            .filter(|edge| {
                edge.kind() == CanonicalEdgeKind::Normal
                    && edge.to() == header
                    && !owners.contains(&view.index_of(edge.from()).unwrap_or(usize::MAX))
            })
            .map(|edge| edge.from().clone())
            .collect();
        let [entry] = normal_entries.as_slice() else {
            continue;
        };
        let entry_node = view
            .index_of(entry)
            .expect("physical entry source is in Reader view");
        let branches: Vec<_> = view
            .predecessors(entry_node)
            .into_iter()
            .filter_map(|node| view.id_of(node).cloned())
            .filter(|source| {
                view.successors(view.index_of(source).expect("branch node"))
                    .iter()
                    .any(|successor| view.id_of(*successor) == Some(join))
            })
            .collect();
        let [branch] = branches.as_slice() else {
            continue;
        };
        choices.push((
            owners.len(),
            Candidate {
                arm: (*arm).clone(),
                branch: branch.clone(),
                entry: entry.clone(),
                join: join.clone(),
                header: header.clone(),
                owners,
                return_nodes: returns,
            },
            gateway_origins.len(),
            for_header.is_some(),
        ));
    }
    choices.sort_by_key(|(count, _, _, _)| *count);
    let (_, candidate, _, _) = choices.into_iter().next()
        .expect("full public region tree contains a loop whose exact owners are natural plus certified return leaves");
    candidate
}

fn run_proof<'a>(
    candidate: &Candidate,
    canonical: &'a CanonicalCfg,
    view: &'a NormalFlowView,
    ssa: &'a SsaTable,
    code: &'a MethodCodeFacts,
    pool: &'a [CpEntryFacts],
    operations: &'a Operations,
    chains: &'a crate::concat::Plan,
    sites: &'a crate::init::Sites,
    budget: &'a mut Budget,
    frame: &Frame,
    arm: &Region,
    entry: &CanonicalBlockId,
    join: &CanonicalBlockId,
) -> Result<Option<CanonicalBlockId>, StopReason> {
    let mut w = walker(
        canonical, view, ssa, code, pool, operations, chains, sites, budget,
    );
    w.visited = candidate.owners.clone();
    w.loop_arm_join_source(arm, &candidate.branch, entry, join, frame)
}

fn branch_target_variant(from_bci: u32, target_bci: u32) -> Vec<u8> {
    let code_start = with_method(ORIGINAL_DEBUG, b"testFor", b"()I", |_, _, code, _, _, _| {
        code.code_span.start
    });
    let mut bytes = ORIGINAL_DEBUG.to_vec();
    let start = usize::try_from(code_start).expect("Code span fits host usize");
    let at = start + usize::try_from(from_bci).expect("BCI fits host usize");
    assert!(
        matches!(bytes[at], 0x99..=0xa6 | 0xc6 | 0xc7),
        "source BCI is a short conditional branch"
    );
    let displacement = i16::try_from(i64::from(target_bci) - i64::from(from_bci))
        .expect("target displacement fits signed short branch");
    bytes[at + 1..at + 3].copy_from_slice(&displacement.to_be_bytes());
    bytes
}

#[test]
fn reader_loop_arm_return_leaf_and_latch_are_one_exact_owner_set() {
    for (fixture, bytes, name, descriptor) in [
        (
            "original-debug",
            ORIGINAL_DEBUG,
            b"testFor".as_slice(),
            b"()I".as_slice(),
        ),
        (
            "original-no-debug",
            ORIGINAL_NO_DEBUG,
            b"testFor".as_slice(),
            b"()I".as_slice(),
        ),
        (
            "A-javac8-debug",
            A_JAVAC8_DEBUG,
            b"run".as_slice(),
            b"(IIIII)I".as_slice(),
        ),
        (
            "A-javac8-no-debug",
            A_JAVAC8_NO_DEBUG,
            b"run".as_slice(),
            b"(IIIII)I".as_slice(),
        ),
        (
            "A-javac23-debug",
            A_JAVAC23_DEBUG,
            b"run".as_slice(),
            b"(IIIII)I".as_slice(),
        ),
        (
            "A-javac23-no-debug",
            A_JAVAC23_NO_DEBUG,
            b"run".as_slice(),
            b"(IIIII)I".as_slice(),
        ),
        (
            "B-javac8-debug",
            B_JAVAC8_DEBUG,
            b"run".as_slice(),
            b"(IIIII)I".as_slice(),
        ),
        (
            "B-javac8-no-debug",
            B_JAVAC8_NO_DEBUG,
            b"run".as_slice(),
            b"(IIIII)I".as_slice(),
        ),
        (
            "B-javac23-debug",
            B_JAVAC23_DEBUG,
            b"run".as_slice(),
            b"(IIIII)I".as_slice(),
        ),
        (
            "B-javac23-no-debug",
            B_JAVAC23_NO_DEBUG,
            b"run".as_slice(),
            b"(IIIII)I".as_slice(),
        ),
    ] {
        with_method(
            bytes,
            name,
            descriptor,
            |canonical, ssa, code, pool, view, operations| {
                let chains =
                    crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
                let sites = crate::init::Sites::empty();
                let candidate = real_candidate(
                    canonical, ssa, code, pool, view, operations, &chains, &sites,
                );
                assert!(!candidate.return_nodes.is_empty());
                let header = view.index_of(&candidate.header).expect("real header");
                let natural = view.loop_entered_at(header).expect("same natural loop");
                assert!(candidate.owners.is_superset(natural.blocks()));
                assert_eq!(
                    candidate
                        .owners
                        .difference(natural.blocks())
                        .copied()
                        .collect::<BTreeSet<_>>(),
                    candidate.return_nodes
                );
                assert!(
                    candidate
                        .return_nodes
                        .iter()
                        .all(|node| view.successors(*node).is_empty())
                );
                let mut scope = candidate.owners.clone();
                scope.extend(candidate.return_nodes.iter().copied());
                let frame = Frame {
                    scope: Some(scope),
                    ..Frame::default()
                };
                let mut budget = proof_budget(1 << 20);
                let actual = run_proof(
                    &candidate,
                    canonical,
                    view,
                    ssa,
                    code,
                    pool,
                    operations,
                    &chains,
                    &sites,
                    &mut budget,
                    &frame,
                    &candidate.arm,
                    &candidate.entry,
                    &candidate.join,
                )
                .expect("proof completes")
                .expect("exact return/latch child joins parent arm");
                assert_eq!(
                    actual, candidate.header,
                    "the decoded normal exit source is the loop test block"
                );
                let Region::Loop {
                    body,
                    gateway_origins,
                    for_header,
                    ..
                } = &candidate.arm
                else {
                    unreachable!()
                };
                let tail = body.last().expect("actual recovered loop body has a tail");
                let tail_kind = match tail {
                    Region::If { .. } => "If",
                    Region::CompoundIf { .. } => "CompoundIf",
                    Region::Straight { .. } => "Straight",
                    _ => "other",
                };
                eprintln!(
                    "return-loop fixture={fixture} header={} tail={tail_kind} gateway_origins={gateway_origins:?} for_header={}",
                    candidate.header.bci(),
                    for_header.is_some()
                );
                assert!(
                    !gateway_origins.is_empty(),
                    "physical latch gateway remains represented"
                );
                assert!(
                    for_header.is_some(),
                    "the original and supplemental child retain ForHeader evidence"
                );
            },
        );
    }
}

#[test]
fn reader_loop_arm_return_proof_refuses_missing_leaf_scope_wrong_entry_and_latch() {
    with_method(
        ORIGINAL_DEBUG,
        b"testFor",
        b"()I",
        |canonical, ssa, code, pool, view, operations| {
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            let candidate = real_candidate(
                canonical, ssa, code, pool, view, operations, &chains, &sites,
            );
            let mut scope = candidate.owners.clone();
            scope.extend(candidate.return_nodes.iter().copied());
            let mut incomplete = Frame::default();
            let leaf = *candidate
                .return_nodes
                .iter()
                .next()
                .expect("certified terminal return");
            scope.remove(&leaf);
            incomplete.scope = Some(scope);
            let mut budget = proof_budget(1 << 20);
            assert!(
                run_proof(
                    &candidate,
                    canonical,
                    view,
                    ssa,
                    code,
                    pool,
                    operations,
                    &chains,
                    &sites,
                    &mut budget,
                    &incomplete,
                    &candidate.arm,
                    &candidate.entry,
                    &candidate.join
                )
                .expect("missing frame evidence is a refusal")
                .is_none()
            );

            let frame = Frame {
                scope: Some(
                    candidate
                        .owners
                        .union(&candidate.return_nodes)
                        .copied()
                        .collect(),
                ),
                ..Frame::default()
            };
            let mut budget = proof_budget(1 << 20);
            assert!(
                run_proof(
                    &candidate,
                    canonical,
                    view,
                    ssa,
                    code,
                    pool,
                    operations,
                    &chains,
                    &sites,
                    &mut budget,
                    &frame,
                    &candidate.arm,
                    &candidate.branch,
                    &candidate.join
                )
                .expect("wrong actual entry is a refusal")
                .is_none()
            );

            let mut duplicate_owner = candidate.arm.clone();
            let Region::Loop { body, .. } = &mut duplicate_owner else {
                unreachable!()
            };
            let duplicate_added = match body.last_mut().expect("final return/latch branch") {
                Region::If {
                    prefix,
                    branch,
                    join: None,
                    ..
                } => {
                    prefix.push(branch.clone());
                    true
                }
                Region::CompoundIf {
                    prefix,
                    tests,
                    join: None,
                    ..
                } => {
                    let Some(test) = tests.last() else {
                        panic!("compound tail has a final test")
                    };
                    prefix.push(test.block.clone());
                    true
                }
                _ => false,
            };
            assert!(
                duplicate_added,
                "the real tail branch permits a same-IR duplicate-owner mutation"
            );
            let mut budget = proof_budget(1 << 20);
            assert!(
                run_proof(
                    &candidate,
                    canonical,
                    view,
                    ssa,
                    code,
                    pool,
                    operations,
                    &chains,
                    &sites,
                    &mut budget,
                    &frame,
                    &duplicate_owner,
                    &candidate.entry,
                    &candidate.join
                )
                .expect("duplicate physical owner is a refusal")
                .is_none()
            );

            let mut wrong_target = candidate.arm.clone();
            let Region::Loop { header, .. } = &mut wrong_target else {
                unreachable!()
            };
            *header = candidate.join.clone();
            let mut budget = proof_budget(1 << 20);
            assert!(
                run_proof(
                    &candidate,
                    canonical,
                    view,
                    ssa,
                    code,
                    pool,
                    operations,
                    &chains,
                    &sites,
                    &mut budget,
                    &frame,
                    &wrong_target,
                    &candidate.entry,
                    &candidate.join
                )
                .expect("wrong natural-loop target is a refusal")
                .is_none()
            );

            let mut wrong_gateway = candidate.arm.clone();
            let Region::Loop {
                gateway_origins, ..
            } = &mut wrong_gateway
            else {
                unreachable!()
            };
            assert_eq!(gateway_origins.len(), 1);
            gateway_origins[0] = gateway_origins[0].wrapping_add(1);
            let mut budget = proof_budget(1 << 20);
            assert!(
                run_proof(
                    &candidate,
                    canonical,
                    view,
                    ssa,
                    code,
                    pool,
                    operations,
                    &chains,
                    &sites,
                    &mut budget,
                    &frame,
                    &wrong_gateway,
                    &candidate.entry,
                    &candidate.join
                )
                .expect("wrong physical latch gateway is a refusal")
                .is_none()
            );
        },
    );
}

#[test]
fn reader_branch_offset_metamorphisms_refuse_under_an_explicit_same_bci_proof_context() {
    // These are byte-level Reader metamorphisms. Two variants do not produce complete SSA, so
    // the consumer receives the changed canonical graph/view/code and the original SSA/Region
    // identities at the same physical BCIs. This tests refusal under altered canonical evidence;
    // it is not a CompleteSSA or JVM-validity claim.
    with_method(
        ORIGINAL_DEBUG,
        b"testFor",
        b"()I",
        |original, original_ssa, original_code, original_pool, original_view, original_ops| {
            let original_chains = crate::concat::plan(
                original_ssa,
                original_ops,
                &crate::build::FieldCopies::default(),
            );
            let sites = crate::init::Sites::empty();
            let candidate = real_candidate(
                original,
                original_ssa,
                original_code,
                original_pool,
                original_view,
                original_ops,
                &original_chains,
                &sites,
            );
            assert!(!candidate.return_nodes.is_empty());
            for (from_bci, target_bci) in [(6, 29), (11, 21), (26, 37)] {
                let variant = branch_target_variant(from_bci, target_bci);
                with_canonical_without_ssa(
                    &variant,
                    b"testFor",
                    b"()I",
                    |canonical, code, pool, view, operations| {
                        let source_id = canonical
                            .blocks()
                            .iter()
                            .find(|block| {
                                block.blocks().len() == 1
                                    && block.id().bci() <= from_bci
                                    && from_bci < block.end_bci()
                                    && block.id().path() == candidate.branch.path()
                            })
                            .map(|block| block.id())
                            .expect("modified instruction belongs to a real canonical block");
                        let target_id = canonical
                            .blocks()
                            .iter()
                            .map(|block| block.id())
                            .find(|id| id.bci() == target_bci && id.path() == source_id.path())
                            .expect("modified target is a real canonical block start");
                        assert!(
                            canonical.edges().iter().any(|edge| {
                                edge.kind() == CanonicalEdgeKind::Normal
                                    && edge.from() == source_id
                                    && edge.to() == target_id
                            }),
                            "the changed displacement is represented by an actual Reader canonical edge"
                        );
                        match from_bci {
                            6 => {
                                assert!(candidate.arm.blocks().contains(&target_id));
                                assert!(
                                    canonical
                                        .edges()
                                        .iter()
                                        .filter(|edge| edge.to() == target_id)
                                        .count()
                                        > 1,
                                    "this mutation gives the original terminal leaf an additional physical predecessor"
                                );
                            }
                            11 => {
                                assert!(candidate.arm.blocks().contains(&target_id));
                                assert!(
                                    !candidate.arm.blocks().contains(&source_id),
                                    "this mutation adds an outside predecessor into the original owner set"
                                );
                            }
                            26 => {
                                assert!(candidate.arm.blocks().contains(&source_id));
                                assert_eq!(
                                    target_id, &candidate.join,
                                    "this mutation adds another ordinary exit to the original join"
                                );
                            }
                            _ => unreachable!(),
                        }
                        let owner_nodes: BTreeSet<_> = candidate
                            .arm
                            .blocks()
                            .iter()
                            .filter_map(|id| view.index_of(id))
                            .collect();
                        let mut frame = Frame::default();
                        let return_nodes = candidate.return_nodes.iter().filter_map(|node| {
                            original_view.id_of(*node).and_then(|id| view.index_of(id))
                        });
                        frame.scope =
                            Some(owner_nodes.iter().copied().chain(return_nodes).collect());
                        let chains = crate::concat::Plan::empty();
                        let mut budget = proof_budget(1 << 20);
                        let mut w = walker(
                            canonical,
                            view,
                            original_ssa,
                            code,
                            pool,
                            operations,
                            &chains,
                            &sites,
                            &mut budget,
                        );
                        w.visited = owner_nodes;
                        assert!(
                            w.loop_arm_join_source(
                                &candidate.arm,
                                &candidate.branch,
                                &candidate.entry,
                                &candidate.join,
                                &frame
                            )
                            .expect("mutated canonical proof context completes")
                            .is_none(),
                            "no mutated canonical edge set may reuse the original loop-arm claim"
                        );
                    },
                );
            }
        },
    );
}

#[test]
fn real_jsr_clone_as_foreign_branch_context_is_refused_by_the_loop_consumer() {
    let candidate = with_method(
        ORIGINAL_DEBUG,
        b"testFor",
        b"()I",
        |canonical, ssa, code, pool, view, operations| {
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            real_candidate(
                canonical, ssa, code, pool, view, operations, &chains, &sites,
            )
        },
    );
    let cloned_branch = with_method(
        include_bytes!(
            "../../../../tests/fixtures/p3-switch-jsr-clones/v49/SwitchLoopJsrClone.class"
        ),
        b"run",
        b"(I)I",
        |_, ssa, _, _, _view, operations| {
            ssa.blocks()
                .iter()
                .find_map(|block| {
                    let terminal = block.instructions().last()?;
                    operations.get(terminal.bci()).and_then(Operation::switch)?;
                    let id = block.block();
                    id.is_clone().then(|| id.clone())
                })
                .expect("Reader provides a real cloned switch block")
        },
    );
    assert!(cloned_branch.is_clone());
    assert!(!cloned_branch.path().is_empty());
    assert!(
        candidate
            .arm
            .blocks()
            .iter()
            .all(|owner| owner.path() != cloned_branch.path())
    );

    with_method(
        ORIGINAL_DEBUG,
        b"testFor",
        b"()I",
        |canonical, ssa, code, pool, view, operations| {
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            let owner_nodes: BTreeSet<_> = candidate
                .arm
                .blocks()
                .iter()
                .map(|id| view.index_of(id).expect("same NL5 Reader owner"))
                .collect();
            let frame = Frame {
                scope: Some(owner_nodes.clone()),
                ..Frame::default()
            };
            let mut budget = proof_budget(1 << 20);
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
            w.visited = owner_nodes;
            // This is a foreign-clone-context proof metamorphism over genuine Reader IDs. It is not
            // a same-method JVM control-flow case and does not claim whole jsr-loop recovery.
            assert!(
                w.loop_arm_join_source(
                    &candidate.arm,
                    &cloned_branch,
                    &candidate.entry,
                    &candidate.join,
                    &frame,
                )
                .expect("clone/path mismatch is an ordinary refusal")
                .is_none()
            );
        },
    );
}

#[test]
fn reader_compound_tail_requires_return_certificate_but_legacy_if_still_works() {
    with_method(
        ORIGINAL_DEBUG,
        b"testFor",
        b"()I",
        |canonical, ssa, code, pool, view, operations| {
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            let candidate = real_candidate(
                canonical, ssa, code, pool, view, operations, &chains, &sites,
            );
            let header_node = view.index_of(&candidate.header).expect("real child header");
            let Region::Loop {
                body,
                gateway_origins,
                ..
            } = &candidate.arm
            else {
                unreachable!()
            };
            let tail = body.last().expect("final child tail");
            let [gateway] = gateway_origins.as_slice() else {
                panic!("one real gateway")
            };
            assert!(
                matches!(tail, Region::If { .. }),
                "the frozen original exercises the legacy If variant"
            );

            // If the actual Reader tail is an If, exercise its pre-existing no-ForHeader path with
            // the same block/SSA/normal-flow facts and no new terminal-return certificate.
            if matches!(tail, Region::If { .. }) {
                let mut budget = proof_budget(1 << 20);
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
                assert_eq!(
                    w.implicit_tail_latch_origin(header_node, body, None, None)
                        .expect("legacy If proof completes without a return certificate"),
                    Some(*gateway),
                );
            }

            // Exercise the CompoundIf variant gate over Reader-derived branch/arm identities. If
            // Reader already produced CompoundIf, retain it exactly. Otherwise re-express the same
            // one physical If test as a one-test CompoundIf solely as a private context mutation;
            // this is not a classfile or JVM-validity claim.
            let compound_body = match tail {
                Region::CompoundIf { .. } => vec![tail.clone()],
                Region::If {
                    prefix,
                    branch,
                    branch_bci,
                    then_arm,
                    else_arm,
                    ..
                } => {
                    let successors = view.successor_ids(branch);
                    assert_eq!(
                        successors.len(),
                        2,
                        "the original branch has two real successors"
                    );
                    vec![Region::CompoundIf {
                        prefix: prefix.clone(),
                        tests: vec![CompoundTest {
                            block: branch.clone(),
                            branch_bci: *branch_bci,
                            then: successors[0].clone(),
                            otherwise: successors[1].clone(),
                        }],
                        condition: CompoundCondition::Test(0),
                        then_arm: then_arm.clone(),
                        else_arm: else_arm.clone(),
                        join: None,
                    }]
                }
                _ => panic!("bounded return/latch candidate has an If tail"),
            };
            let mut mutated_body = body[..body.len() - 1].to_vec();
            mutated_body.extend(compound_body);
            let mut budget = proof_budget(1 << 20);
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
            assert_eq!(
                w.implicit_tail_latch_origin(header_node, &mutated_body, None, None)
                    .expect("missing CompoundIf certificate is an ordinary refusal"),
                None,
                "CompoundIf must not inherit the legacy no-ForHeader allowance",
            );
        },
    );
}

#[test]
fn real_exception_edge_incident_to_a_loop_refuses_the_arm_proof() {
    with_method(
        LOOP_EXCEPTION,
        b"run",
        b"(II)I",
        |canonical, ssa, code, pool, view, operations| {
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            assert!(!code.exception_handlers.is_empty());
            let mut selected = None;
            for header_node in 0..view.len() {
                let Some(natural) = view.loop_entered_at(header_node) else {
                    continue;
                };
                let header = view.id_of(header_node).unwrap();
                let owners: BTreeSet<_> = natural
                    .blocks()
                    .iter()
                    .map(|n| view.id_of(*n).unwrap().clone())
                    .collect();
                if owners.iter().any(|id| id.path() != header.path()) {
                    continue;
                }
                let incident: Vec<_> = canonical
                    .edges()
                    .iter()
                    .filter(|edge| owners.contains(edge.from()) || owners.contains(edge.to()))
                    .collect();
                if !incident
                    .iter()
                    .any(|e| matches!(e.kind(), CanonicalEdgeKind::Exception { .. }))
                {
                    continue;
                }
                assert!(incident.iter().all(|e| matches!(
                    e.kind(),
                    CanonicalEdgeKind::Normal | CanonicalEdgeKind::Exception { .. }
                )));
                let entries: Vec<_> = incident
                    .iter()
                    .filter(|e| e.kind() == CanonicalEdgeKind::Normal && !owners.contains(e.from()))
                    .collect();
                let exits: Vec<_> = incident
                    .iter()
                    .filter(|e| e.kind() == CanonicalEdgeKind::Normal && !owners.contains(e.to()))
                    .collect();
                let ([entry], [exit]) = (entries.as_slice(), exits.as_slice()) else {
                    continue;
                };
                if entry.to() != header {
                    continue;
                }
                let branch = entry.from();
                let join = exit.to();
                if owners.iter().any(|id| {
                    id.path() != branch.path() || id.bci() <= branch.bci() || id.bci() >= join.bci()
                }) {
                    continue;
                }
                selected = Some((
                    header.clone(),
                    owners,
                    branch.clone(),
                    join.clone(),
                    natural.blocks().clone(),
                ));
                break;
            }
            let (header, owners, entry, join, owner_nodes) = selected.expect(
                "Reader loop has unique normal entry/exit and actual exceptional incidence",
            );
            // A Region proof-context metamorphism, not the method's recovered lexical tree:
            // collapse exactly the real natural owners into the consumer's closed Straight grammar.
            // Canonical edges, exception rows, SSA and Code stay unchanged. Every pre-census gate
            // is satisfied; charging the full census below proves the exceptional edge is reached.
            let arm = Region::Loop {
                header: header.clone(),
                tests: Vec::new(),
                test_operator: None,
                form: LoopForm::While,
                for_header: None,
                body: vec![Region::Straight {
                    blocks: owners.iter().filter(|id| *id != &header).cloned().collect(),
                }],
                exit: Some(join.clone()),
                gateway_origins: Vec::new(),
            };
            let blocks = arm.blocks();
            assert_eq!(blocks.len(), owners.len());
            assert_eq!(blocks.into_iter().cloned().collect::<BTreeSet<_>>(), owners);
            assert!(owner_nodes.iter().all(|n| !view.successors(*n).is_empty()));
            let frame = Frame {
                scope: Some(owner_nodes.clone()),
                ..Frame::default()
            };
            let mut budget = proof_budget(1 << 20);
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
            w.visited = owner_nodes.clone();
            assert!(
                w.loop_arm_join_source(&arm, &entry, &entry, &join, &frame)
                    .expect("canonical Exception is an ordinary refusal")
                    .is_none()
            );
            assert_eq!(w.visited, owner_nodes);
            assert_eq!(
                w.budget.usage().analysis_steps,
                u64::try_from(canonical.edges().len() + owners.len()).unwrap(),
                "the full canonical edge census was actually entered"
            );
        },
    );
}

#[test]
fn reader_loop_arm_return_proof_publishes_no_partial_claim_on_budget_or_stop() {
    with_method(
        ORIGINAL_DEBUG,
        b"testFor",
        b"()I",
        |canonical, ssa, code, pool, view, operations| {
            let chains =
                crate::concat::plan(ssa, operations, &crate::build::FieldCopies::default());
            let sites = crate::init::Sites::empty();
            let candidate = real_candidate(
                canonical, ssa, code, pool, view, operations, &chains, &sites,
            );
            let frame = Frame {
                scope: Some(
                    candidate
                        .owners
                        .union(&candidate.return_nodes)
                        .copied()
                        .collect(),
                ),
                ..Frame::default()
            };
            let mut saw_late_budget = false;
            for limit in 0..512 {
                let mut budget = proof_budget(limit);
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
                w.visited = candidate.owners.clone();
                let initial = w.visited.clone();
                let result = w.loop_arm_join_source(
                    &candidate.arm,
                    &candidate.branch,
                    &candidate.entry,
                    &candidate.join,
                    &frame,
                );
                if matches!(
                    result,
                    Err(StopReason::Budget {
                        dimension: CountedBudgetDimension::AnalysisSteps,
                        ..
                    })
                ) && w.budget.usage().analysis_steps > 0
                {
                    assert_eq!(w.visited, initial);
                    assert!(w.catch_joins.is_empty());
                    assert!(w.loop_catch_terminal_owners.is_empty());
                    assert!(w.fragmented.is_none());
                    assert_eq!(w.unclosed_tail_at, None);
                    saw_late_budget = true;
                    break;
                }
                if result.is_ok_and(|value| value.is_some()) {
                    break;
                }
            }
            assert!(
                saw_late_budget,
                "positive-charge proof scan has a budget stop"
            );

            // These are explicitly proof-entry stop checks after Reader preparation. They are not
            // represented as mid-proof asynchronous cancellation/deadline evidence.
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
            w.visited = candidate.owners.clone();
            assert!(matches!(
                w.loop_arm_join_source(
                    &candidate.arm,
                    &candidate.branch,
                    &candidate.entry,
                    &candidate.join,
                    &frame
                ),
                Err(StopReason::Cancelled { .. })
            ));
            assert_eq!(w.visited, candidate.owners);

            let mut budget = Budget::new(Limits {
                analysis_steps: u64::MAX,
                elapsed_millis: 1,
                ..Limits::default()
            });
            std::thread::sleep(std::time::Duration::from_millis(3));
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
            w.visited = candidate.owners.clone();
            assert!(matches!(
                w.loop_arm_join_source(
                    &candidate.arm,
                    &candidate.branch,
                    &candidate.entry,
                    &candidate.join,
                    &frame
                ),
                Err(StopReason::Interrupted { .. })
            ));
            assert_eq!(w.visited, candidate.owners);
        },
    );
}
