//! The fixed CF-16 Tf1 local-null conditional: one finally, one `if (local != null)` close,
//! every physical BCI owned.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Quality};
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

const FIXED: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf1.class"
);

const METHOD_DESCRIPTOR: &[u8] = b"(LContext;Ljava/lang/Object;)Ljava/lang/String;";

/// Verifier-valid neighbors the certificate must refuse. Each one breaks exactly one link of
/// the fixed grammar: the first five are compiled sources, the last four are bytecode patches
/// (`negatives/patch-tf1.py`) that rewrite one copy or one completion.
const NEGATIVES: [&[u8]; 9] = [
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf1LeadNew/Tf1LeadNew.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf1QueryBeforeTry/Tf1QueryBeforeTry.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf1SlotMismatch/Tf1SlotMismatch.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf1AssignOutside/Tf1AssignOutside.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf1CleanupExtra/Tf1CleanupExtra.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf1InvertedCondition/Tf1InvertedCondition.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf1TargetMismatch/Tf1TargetMismatch.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf1ReturnIdentity/Tf1ReturnIdentity.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf1RethrowIdentity/Tf1RethrowIdentity.class"
    ),
];

const NEGATIVE_NAMES: [&str; 9] = [
    "Tf1LeadNew",
    "Tf1QueryBeforeTry",
    "Tf1SlotMismatch",
    "Tf1AssignOutside",
    "Tf1CleanupExtra",
    "Tf1InvertedCondition",
    "Tf1TargetMismatch",
    "Tf1ReturnIdentity",
    "Tf1RethrowIdentity",
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

fn recover_test(class: &[u8], owner: &str) -> jarde_java::RecoveryReport {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("fixture opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: class.len() as u64,
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(b"test".to_vec()),
        descriptor: JvmBytes(METHOD_DESCRIPTOR.to_vec()),
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
            method,
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("fixed class analyzes");
    let facts = RecoveryFacts::new(
        MethodFacts::new(
            "test",
            std::str::from_utf8(METHOD_DESCRIPTOR).expect("descriptor is utf-8"),
            3,
        )
        .with_access_flags(0x0000)
        .with_declaring_class(DeclaringClass::new(owner, 0x0021)),
    );
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

#[test]
fn local_null_test_has_one_finally_one_null_guarded_close_and_all_origins() {
    let report = recover_test(FIXED, "Tf1");
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert_eq!(report.quality, Quality::Structured);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        report.text.matches("finally {").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("if (local3 != null)").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("local3.close();").count(),
        1,
        "{}",
        report.text
    );
    assert!(
        report.text.contains("local3 = null;"),
        "the lead presents the null initialisation: {}",
        report.text
    );
    assert!(
        report.text.contains("Cursor local3;"),
        "the cleanup local's declaration is Cursor: {}",
        report.text
    );
    assert!(
        report.text.contains("local3 = arg1.query(arg2, local4);"),
        "the body's same-slot assignment is presented: {}",
        report.text
    );
    assert_eq!(report.regions.len(), 1);
    assert!(report.regions[0].structured, "{}", report.text);
    assert_eq!(
        report.regions[0].blocks,
        [0, 45, 49, 52, 58, 62],
        "{}",
        report.text
    );
    // Every physical instruction of the statement's span carries an origin: the lead, the body,
    // both cleanup copies, the saved return and the pending throwable's store and rethrow.
    for bci in [
        0, 1, 2, 3, 6, 7, 8, 10, 11, 13, 14, 15, 17, 20, 21, 22, 24, 27, 29, 30, 33, 34, 36, 39,
        41, 42, 45, 46, 49, 51, 52, 54, 55, 58, 59, 62, 64,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no origin: {}",
            report.text
        );
    }
}

/// One refusal per broken certificate link, each a verifier-valid class the engine still
/// refuses: the negative list's order covers the six categories of the change's task 1.2.
#[test]
fn negative_lead_is_not_a_null_initialisation_refuses() {
    refuse(0);
    refuse(1);
}

#[test]
fn negative_cleanup_slot_or_body_range_refuses() {
    refuse(2);
    refuse(3);
}

#[test]
fn negative_copy_grammar_addition_refuses() {
    refuse(4);
}

#[test]
fn negative_inverted_or_disagreeing_copies_refuse() {
    refuse(5);
    refuse(6);
}

#[test]
fn negative_saved_or_rethrow_identity_rewrite_refuses() {
    refuse(7);
    refuse(8);
}

fn refuse(index: usize) {
    let name = NEGATIVE_NAMES[index];
    let report = recover_test(NEGATIVES[index], name);
    assert!(
        !report.text.contains("finally {"),
        "{name} recovered a finally: {}",
        report.text
    );
    assert!(
        report.text.contains("@bytecode"),
        "{name} lost its fallback marker: {}",
        report.text
    );
    assert!(
        !report.source_map.is_empty(),
        "{name} lost its fallback origins"
    );
}
