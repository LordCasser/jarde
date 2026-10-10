//! Reader-backed recovery boundary for javac's real non-adjacent conditional case exit.
//!
//! This source is a private, uncompiled draft. It binds the recovery request to the exact class
//! method and environment, checks the complete canonical graph and physical branch instructions,
//! and then asks the real public recovery caller to handle the non-adjacent shape.

use jarde_java::{
    ArtifactSubject, DebugLocal, DeclaringClass, MethodFacts, RecoveryEvidenceKind,
    RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::{
    LocalDebugTable, LocalDebugTypeTable, MethodSelector, inspect_method_bytecode,
};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
    PhysicalMethodId, PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
    MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
    RuntimeView,
};
use std::collections::BTreeSet;

const CLASS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-switch-boundaries/javac8/B-nonadjacent/ConditionalSwitchBoundaries.class"
);
const CLASS_BLAKE3: &str = "63978aeef832dec2f8a52e34d969576555032b163726ffd75307da433ebdb707";
const OWNER: &str = "ConditionalSwitchBoundaries";
const NAME: &str = "partialBreak";
const DESCRIPTOR: &str = "(II)Ljava/lang/String;";

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
        nested_depth: 16,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

// Facts and debug locals come from the same reader-backed MethodIr that recovery consumes.
fn reader_debug_locals(code: Option<&jarde_reader::classfile::MethodCodeFacts>) -> Vec<DebugLocal> {
    let Some(code) = code else {
        return Vec::new();
    };
    let LocalDebugTable::Read(records) = code.debug() else {
        return Vec::new();
    };
    let generic_records = match code.generic_debug() {
        LocalDebugTypeTable::Read(records) => records.as_slice(),
        LocalDebugTypeTable::Absent | LocalDebugTypeTable::Unstated => &[],
    };
    let mut local_counts = std::collections::BTreeMap::new();
    for record in records {
        let key = (record.slot, record.start_bci, record.end_bci, record.name.0.as_slice());
        *local_counts.entry(key).or_insert(0usize) += 1;
    }
    let mut generic_by_local = std::collections::BTreeMap::new();
    for generic in generic_records {
        let key = (generic.slot, generic.start_bci, generic.end_bci, generic.name.0.as_slice());
        generic_by_local.entry(key).or_insert_with(Vec::new).push(generic);
    }
    records
        .iter()
        .map(|record| {
            let key = (record.slot, record.start_bci, record.end_bci, record.name.0.as_slice());
            let local = DebugLocal::over(
                record.slot,
                record.name_lossy(),
                record.start_bci,
                record.end_bci,
            );
            if local_counts.get(&key) == Some(&1)
                && let Some([generic]) = generic_by_local.get(&key).map(Vec::as_slice)
            {
                local.with_type_metadata(record.descriptor.0.clone(), generic.signature.0.clone())
            } else {
                local
            }
        })
        .collect()
}

struct Run {
    report: jarde_java::RecoveryReport,
    method: PhysicalMethodId,
    ordinal: jarde_reader::prepared::MethodOrdinal,
}

fn recover_boundary(evidence: RecoveryEvidenceRequest, recovery_budget: Option<Budget>) -> Run {
    assert_eq!(CLASS.len(), 1494, "the whole javac 8 class is embedded");
    assert_eq!(blake3::hash(CLASS).to_hex().to_string(), CLASS_BLAKE3);

    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("complete frozen class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot { snapshot: snapshot.id().clone() },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(CLASS).to_hex().to_string()),
            length: u64::try_from(CLASS.len()).expect("frozen class length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass { snapshot: snapshot.id().clone() }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let requested_method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(NAME.as_bytes().to_vec()),
        descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
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
            method: requested_method,
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("reader completes the exact method analysis");
    let method = analysis.report().method.clone();
    assert_eq!(method.name.0.as_slice(), NAME.as_bytes());
    assert_eq!(method.descriptor.0.as_slice(), DESCRIPTOR.as_bytes());
    assert_eq!(method.owner.class_bytes.digest.0, CLASS_BLAKE3);
    assert_eq!(method.owner.class_bytes.length, CLASS.len() as u64);

    let ir = analysis.ir();
    let canonical = ir.canonical().expect("full canonical CFG is present");
    assert!(canonical.unreachable().is_empty(), "fixture records no unreachable blocks");
    let blocks = canonical
        .blocks()
        .iter()
        .map(|block| {
            assert!(block.id().path().is_empty(), "all six observed blocks have empty clone paths");
            block.id().bci()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(blocks, BTreeSet::from([0, 36, 40, 57, 67, 74]));
    let edges = canonical
        .edges()
        .iter()
        .map(|edge| {
            assert_eq!(format!("{:?}", edge.kind()), "Normal");
            assert!(edge.from().path().is_empty() && edge.to().path().is_empty());
            (edge.from().bci(), edge.to().bci())
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        edges,
        BTreeSet::from([
            (0, 36), (0, 57), (0, 67), (36, 40), (36, 67), (40, 67), (57, 74), (67, 74),
        ]),
        "all eight canonical edge rows for the B variant are pinned",
    );

    // These offsets are read from the complete physical instruction stream for this exact method.
    let inspection = inspect_method_bytecode(
        CLASS,
        MethodSelector {
            name: JvmBytes(NAME.as_bytes().to_vec()),
            descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
        },
        &mut Budget::new(limits()),
    )
    .expect("reader decodes the exact physical method");
    assert!(inspection.stopped_at.is_none(), "physical instruction stream is complete");
    let branch37 = inspection.instructions.iter().find(|instruction| instruction.bci == 37)
        .expect("physical conditional branch at BCI 37");
    assert_eq!(branch37.opcode, 0x99);
    assert_eq!(branch37.operands.branch_offset, Some(30));
    let branch47 = inspection.instructions.iter().find(|instruction| instruction.bci == 47)
        .expect("physical goto at BCI 47");
    assert_eq!(branch47.opcode, 0xa7);
    assert_eq!(branch47.operands.branch_offset, Some(20));

    let prepared_read = snapshot
        .prepared_root_class(&mut budget)
        .expect("same frozen root class prepares");
    let prepared = jarde_reader::prepared::PreparedClass::prepare(&prepared_read, &mut budget)
        .expect("same class method table reads completely");
    let ordinals = prepared.locate_method(NAME.as_bytes(), DESCRIPTOR.as_bytes());
    assert_eq!(ordinals.len(), 1, "target declaration is unique");
    let ordinal = ordinals[0];
    let subject = ArtifactSubject::new(
        method.clone(),
        Some(ordinal),
        analysis.report().environment_identity.clone(),
    );

    let declaration = ir.declaration().expect("reader states declaration");
    assert_eq!(declaration.class_name().0.as_slice(), OWNER.as_bytes());
    assert_eq!(declaration.name().0.as_slice(), NAME.as_bytes());
    assert_eq!(declaration.descriptor().0.as_slice(), DESCRIPTOR.as_bytes());
    let facts = RecoveryFacts::new(
        MethodFacts::new(
            String::from_utf8_lossy(&declaration.name().0).into_owned(),
            String::from_utf8_lossy(&declaration.descriptor().0).into_owned(),
            declaration.parameter_slots(),
        )
        .with_access_flags(declaration.access_flags())
        .with_declaring_class(DeclaringClass::new(
            String::from_utf8_lossy(&declaration.class_name().0).into_owned(),
            declaration.class_access_flags(),
        )),
    )
    .with_debug_locals(reader_debug_locals(ir.code()));
    let request = RecoveryRequest::new(ir, &facts, JAVA_8)
        .with_evidence(evidence)
        .with_subject(subject);
    let report = match recovery_budget {
        Some(mut recovery_budget) => recover(&request, &mut recovery_budget),
        None => recover(&request, &mut budget),
    };
    if report.produced() {
        let binding = report.artifact.binding().expect("produced result is bound");
        assert_eq!(binding.method(), &method);
        assert_eq!(binding.member_ordinal(), Some(ordinal));
        assert_eq!(binding.environment(), &analysis.report().environment_identity);
    }
    Run { report, method, ordinal }
}

#[test]
fn nonadjacent_conditional_switch_is_refused_by_the_public_caller() {
    let default = recover_boundary(RecoveryEvidenceRequest::essential(), None);
    let all = recover_boundary(RecoveryEvidenceRequest::all(), None);
    assert_eq!(default.method, all.method);
    assert_eq!(default.ordinal, all.ordinal);
    assert_eq!(default.report.text, all.report.text);
    assert_eq!(default.report.source_map, all.report.source_map);

    // The actual Walker caller may compute 36 -> 67, but valid_order must refuse label sorting
    // because the independent entry at 57 lies between them. Test published structure: there is
    // no admitted switch owning dispatch block 0, no consumed SwitchBreak source at 37 or 47, and
    // no loop rule claiming this acyclic graph. Do not pin the refusal code before a root run.
    assert!(
        all.report.regions.iter().all(|region| {
            !(region.structured && region.blocks.contains(&0) && region.rule.is_some_and(|rule| rule.rule() == "switch"))
        }),
        "non-adjacent map must not publish a structured switch: {:?}",
        all.report.regions,
    );
    assert!(
        all.report.regions.iter().all(|region| {
            !(region.structured
                && region.blocks.is_empty()
                && [37, 47].contains(&region.bci)
                && region.rule.is_some_and(|rule| rule.rule() == "switch"))
        }),
        "rejected conditional exits must not be consumed as SwitchBreak leaves: {:?}",
        all.report.regions,
    );
    assert!(
        all.report.regions.iter().all(|region| !(region.structured && region.rule.is_some_and(|rule| rule.rule() == "loop"))),
        "the fixture has no loop and must not be presented as one: {:?}",
        all.report.regions,
    );
}

#[test]
fn pre_cancelled_public_recovery_publishes_no_partial_artifact() {
    use jarde_java::StopReason;
    use jarde_reader::budget::CancellationToken;

    let token = CancellationToken::new();
    token.cancel();
    let stopped = recover_boundary(
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    )
    .report;
    assert!(stopped.stop().is_some_and(StopReason::is_cancelled));
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
}
