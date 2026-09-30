//! The fixed CF-16 Tf2 null-lead straight finally: one finally, one unconditional
//! `closeQuietly(local)` call, the lead's own `null` declaration, and every physical BCI owned.

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
    "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf2.class"
);

/// Verifier-valid neighbors the certificate must refuse. Each one breaks exactly one link of
/// the fixed grammar: the first five are compiled sources, the last four are bytecode patches
/// (`negatives/patch-tf2.py`) that rewrite one copy, one completion, or one row.
const NEGATIVES: [&[u8]; 9] = [
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2LeadField/Tf2LeadField.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2LeadExtra/Tf2LeadExtra.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2SlotMismatch/Tf2SlotMismatch.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2ArgOtherSlot/Tf2ArgOtherSlot.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2CleanupExtra/Tf2CleanupExtra.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2TargetMismatch/Tf2TargetMismatch.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2ReturnIdentity/Tf2ReturnIdentity.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2RethrowIdentity/Tf2RethrowIdentity.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf2SelfRowWidened/Tf2SelfRowWidened.class"
    ),
];

const NEGATIVE_NAMES: [&str; 9] = [
    "Tf2LeadField",
    "Tf2LeadExtra",
    "Tf2SlotMismatch",
    "Tf2ArgOtherSlot",
    "Tf2CleanupExtra",
    "Tf2TargetMismatch",
    "Tf2ReturnIdentity",
    "Tf2RethrowIdentity",
    "Tf2SelfRowWidened",
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
        descriptor: JvmBytes(format!("([B)L{owner}$Result;").into_bytes()),
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
        MethodFacts::new("test", format!("([B)L{owner}$Result;").as_str(), 2)
            .with_access_flags(0x0001)
            .with_declaring_class(DeclaringClass::new(owner, 0x0021)),
    );
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

#[test]
fn null_lead_test_has_one_finally_one_call_and_all_origins() {
    let report = recover_test(FIXED, "Tf2");
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert_eq!(report.quality, Quality::Structured);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        report.text.matches("finally {").count(),
        1,
        "{}",
        report.text
    );
    // The lead presents the local's declaration and its `= null`, in the split form every
    // Guard lead of this family takes.
    assert!(
        report.text.contains("java.io.InputStream local2;"),
        "the cleanup local's declaration is the body assignment's own type: {}",
        report.text
    );
    assert!(
        report.text.contains("local2 = null;"),
        "the lead presents the null initialisation: {}",
        report.text
    );
    // The body's same-slot assignment and the folded single call.
    assert!(
        report.text.contains("local2 = this.getInputStream(arg1);"),
        "the body's same-slot assignment is presented: {}",
        report.text
    );
    assert_eq!(
        report.text.matches("this.closeQuietly(local2);").count(),
        1,
        "the two copies fold into one cleanup call: {}",
        report.text
    );
    // The saved return is the body's own construction, presented inside the body, with the
    // statement's completion the existing saved-return reading writes.
    assert!(
        report.text.contains("new Tf2$Result(400)"),
        "the construction stays inside the body: {}",
        report.text
    );
    assert!(
        report.text.contains("return local3;"),
        "the statement's completion is the saved return: {}",
        report.text
    );
    assert_eq!(report.regions.len(), 1);
    assert!(report.regions[0].structured, "{}", report.text);
    assert_eq!(report.regions[0].blocks, [0, 32], "{}", report.text);
    // Every physical instruction of the statement's span carries an origin: the lead, the body,
    // both cleanup copies, the saved return and the pending throwable's store and rethrow.
    for bci in [
        0, 1, 2, 3, 4, 7, 8, 9, 10, 13, 14, 17, 18, 21, 24, 25, 26, 27, 30, 31, 32, 34, 35, 36, 39,
        41,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no origin: {}",
            report.text
        );
    }
}

/// One refusal per broken certificate link, each a verifier-valid class the engine still
/// refuses: the negative list's order covers the categories of the change's task 1.2.
#[test]
fn negative_lead_is_no_null_constant_or_more_than_two_instructions_refuses() {
    refuse(0);
    refuse(1);
}

#[test]
fn negative_cleanup_slot_or_argument_slot_refuses() {
    refuse(2);
    refuse(3);
}

#[test]
fn negative_copy_grammar_addition_refuses() {
    refuse(4);
}

#[test]
fn negative_disagreeing_copy_targets_refuses() {
    refuse(5);
}

#[test]
fn negative_saved_or_rethrow_identity_rewrite_refuses() {
    refuse(6);
    refuse(7);
}

#[test]
fn negative_self_protecting_row_widening_refuses() {
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
