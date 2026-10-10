//! Reader-backed public-API draft for conditional-switch scope boundaries.
//!
//! The frozen complete class is the javac 8 input recorded by boundary-public-ir-root-v1.
//! This test pins that input and the public IR shape, then reports the four real recovery
//! observations. Outcome, break-scope, return/throw, and refusal assertions are intentionally
//! left for the applying root to set from an execution of the applied candidate.

use jarde_java::{
    ArtifactSubject, DebugLocal, DeclaringClass, MethodFacts, RecoveryEvidenceRequest,
    RecoveryFacts, RecoveryRequest, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::{LocalDebugTable, LocalDebugTypeTable};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};
use std::collections::BTreeSet;

const CLASS: &[u8] = include_bytes!("/private/tmp/jarde-conditional-scope-tests-luna-v1/tests/fixtures/ConditionalSwitchBoundaries.class");
const CLASS_LEN: usize = 1494;
const CLASS_BLAKE3: &str = "49795c66605066655f48ac37b862ccd132e43b74d290653afd51762ab1a409b7";
const OWNER: &str = "ConditionalSwitchBoundaries";
const METHODS: [(&str, &str, usize, usize); 4] = [
    ("innerLoopBreak", "(II)Ljava/lang/String;", 10, 13),
    ("innerSwitchBreak", "(II)Ljava/lang/String;", 8, 10),
    ("terminalCase", "(I)Ljava/lang/String;", 6, 6),
    ("caughtExceptionThenFallthrough", "(II)Ljava/lang/String;", 9, 14),
];

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
    let mut counts = std::collections::BTreeMap::new();
    for record in records {
        let key = (record.slot, record.start_bci, record.end_bci, record.name.0.as_slice());
        *counts.entry(key).or_insert(0usize) += 1;
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
            if counts.get(&key) == Some(&1)
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
    block_bcis: BTreeSet<u32>,
    edges: Vec<(u32, String, u32)>,
    opcodes: Vec<(u32, u8)>,
}

fn run(name: &str, descriptor: &str) -> Run {
    assert_eq!(CLASS.len(), CLASS_LEN, "the complete frozen class is embedded");
    assert_eq!(blake3::hash(CLASS).to_hex().to_string(), CLASS_BLAKE3);

    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("complete frozen class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot { snapshot: snapshot.id().clone() },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(CLASS).to_hex().to_string()),
            length: u64::try_from(CLASS.len()).expect("class length fits u64"),
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
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
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
    .expect("public reader analysis completes for the exact method");
    let method = analysis.report().method.clone();
    assert_eq!(method.name.0.as_slice(), name.as_bytes());
    assert_eq!(method.descriptor.0.as_slice(), descriptor.as_bytes());
    assert_eq!(method.owner.class_bytes.digest.0, CLASS_BLAKE3);
    assert_eq!(method.owner.class_bytes.length, CLASS_LEN as u64);

    let ir = analysis.ir();
    let canonical = ir.canonical().expect("reader publishes canonical CFG");
    let code = ir.code().expect("reader publishes physical decoded code");
    assert_eq!(code.instructions.len(), code.operands().len());
    let block_bcis = canonical
        .blocks()
        .iter()
        .map(|block| {
            assert!(block.id().path().is_empty(), "frozen input uses physical canonical blocks");
            block.id().bci()
        })
        .collect::<BTreeSet<_>>();
    let edges = canonical
        .edges()
        .iter()
        .map(|edge| {
            assert!(edge.from().path().is_empty() && edge.to().path().is_empty());
            (edge.from().bci(), format!("{:?}", edge.kind()), edge.to().bci())
        })
        .collect::<Vec<_>>();
    let opcodes = code
        .instructions
        .iter()
        .map(|instruction| (instruction.bci, instruction.opcode))
        .collect::<Vec<_>>();

    let prepared_read = snapshot
        .prepared_root_class(&mut budget)
        .expect("same complete class prepares");
    let prepared = jarde_reader::prepared::PreparedClass::prepare(&prepared_read, &mut budget)
        .expect("complete class method table reads");
    let ordinals = prepared.locate_method(name.as_bytes(), descriptor.as_bytes());
    assert_eq!(ordinals.len(), 1, "the exact declaration is unique");
    let ordinal = ordinals[0];
    let subject = ArtifactSubject::new(
        method.clone(),
        Some(ordinal),
        analysis.report().environment_identity.clone(),
    );
    let declaration = ir.declaration().expect("reader states the method declaration");
    assert_eq!(declaration.class_name().0.as_slice(), OWNER.as_bytes());
    assert_eq!(declaration.name().0.as_slice(), name.as_bytes());
    assert_eq!(declaration.descriptor().0.as_slice(), descriptor.as_bytes());
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
        .with_evidence(RecoveryEvidenceRequest::all())
        .with_subject(subject);
    let report = recover(&request, &mut budget);
    if report.produced() {
        let binding = report.artifact.binding().expect("public result is bound");
        assert_eq!(binding.method(), &method);
        assert_eq!(binding.member_ordinal(), Some(ordinal));
        assert_eq!(binding.environment(), &analysis.report().environment_identity);
    }

    Run { report, method, ordinal, block_bcis, edges, opcodes }
}

#[test]
fn real_conditional_switch_scope_boundaries_keep_reader_identity_and_ir() {
    for (name, descriptor, expected_blocks, expected_edges) in METHODS {
        let observed = run(name, descriptor);
        assert_eq!(observed.block_bcis.len(), expected_blocks, "{name} block starts");
        assert_eq!(observed.edges.len(), expected_edges, "{name} canonical edge count");
        if name != "caughtExceptionThenFallthrough" {
            assert!(
                observed.edges.iter().all(|(_, kind, _)| kind == "Normal"),
                "{name} preserves its observed all-Normal graph: {:?}",
                observed.edges
            );
        }
        if name == "innerLoopBreak" {
            assert!(
                observed.edges.contains(&(57, "Normal".to_owned(), 38)),
                "the physical loop back-edge remains in the canonical graph"
            );
        }
        if name == "terminalCase" {
            assert!(observed.opcodes.iter().any(|(_, opcode)| *opcode == 0xb0));
            assert!(observed.opcodes.iter().any(|(_, opcode)| *opcode == 0xbf));
        }

        // Observation output lets the applying root turn the candidate's actual behavior into
        // exact assertions. It deliberately does not presume switch admission or a refusal code.
        println!(
            "BOUNDARY name={name} descriptor={descriptor} method={:?} ordinal={:?} outcome={:?} regions={:#?} text={:?}",
            observed.method,
            observed.ordinal,
            observed.report.outcome,
            observed.report.regions,
            observed.report.text,
        );
        println!(
            "BOUNDARY_IR name={name} blocks={:?} edges={:?} opcodes={:?}",
            observed.block_bcis,
            observed.edges,
            observed.opcodes,
        );

        if name == "caughtExceptionThenFallthrough" {
            let exceptional = observed
                .edges
                .iter()
                .filter(|(_, kind, _)| kind.starts_with("Exception"))
                .collect::<Vec<_>>();
            assert_eq!(exceptional.len(), 3, "the reader preserves all three real handler edges");
            assert_eq!(
                exceptional.iter().map(|(from, _, to)| (*from, *to)).collect::<BTreeSet<_>>(),
                BTreeSet::from([(40, 60), (45, 60), (47, 60)]),
                "caught throw sites reach the physical handler entry",
            );
        }
    }
}
