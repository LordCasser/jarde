//! The fixed CF-16 Tf4 flag conditional: one finally, one `-=`, every physical BCI owned.

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
    "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf4.class"
);

/// The same-layout probe whose cleanup can itself fail: the certificate's guarded-throw tail.
const CLEANUP_PROBE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/probe/original/Tf4CleanupProbe.class"
);

/// Verifier-valid neighbors the certificate must refuse. Each one breaks exactly one link of the
/// fixed grammar; the first six are compiled sources, the last three are bytecode patches
/// (`negatives/patch.py`) whose handler copy disagrees with the normal copy.
const NEGATIVES: [&[u8]; 11] = [
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4NoSetTrue.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4TwoSetTrue.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4StatementAfterSetTrue.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4InvertedCondition.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4LeadTrue.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4LeadExtra.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4CleanupCall.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4CleanupDoubleUpdate.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4KMismatch.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4FieldMismatch.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/Tf4GuardFieldMismatch.class"
    ),
];

const NEGATIVE_NAMES: [&str; 11] = [
    "Tf4NoSetTrue",
    "Tf4TwoSetTrue",
    "Tf4StatementAfterSetTrue",
    "Tf4InvertedCondition",
    "Tf4LeadTrue",
    "Tf4LeadExtra",
    "Tf4CleanupCall",
    "Tf4CleanupDoubleUpdate",
    "Tf4KMismatch",
    "Tf4FieldMismatch",
    "Tf4GuardFieldMismatch",
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

fn recover_test(
    class: &[u8],
    owner: &str,
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
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
        descriptor: JvmBytes(b"()Ljava/lang/String;".to_vec()),
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
        MethodFacts::new("test", "()Ljava/lang/String;", 1)
            .with_access_flags(0x0001)
            .with_declaring_class(DeclaringClass::new(owner, 0x0021)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

#[test]
fn flag_conditional_test_has_one_finally_one_field_update_and_all_origins() {
    let report = recover_test(FIXED, "Tf4", None);
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
        report.text.matches("if (!local1)").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("this.result -= 2;").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("local1 = true;").count(),
        1,
        "{}",
        report.text
    );
    assert!(
        report.text.contains("local1 = false;"),
        "the lead presents the false initialisation: {}",
        report.text
    );
    assert!(
        report.text.contains("boolean local1;"),
        "the flag's declaration is boolean: {}",
        report.text
    );
    assert_eq!(report.regions.len(), 1);
    assert!(report.regions[0].structured, "{}", report.text);
    assert_eq!(
        report.regions[0].blocks,
        [0, 25, 35, 37, 43, 53],
        "{}",
        report.text
    );
    // Every physical instruction of the statement's span carries an origin: the lead, the body,
    // both cleanup copies, the saved return and the pending throwable's store and rethrow.
    for bci in [
        0, 1, 2, 3, 6, 7, 8, 9, 12, 13, 14, 17, 18, 19, 20, 21, 22, 25, 26, 27, 30, 31, 32, 35, 36,
        37, 39, 40, 43, 44, 45, 48, 49, 50, 53, 55,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no origin: {}",
            report.text
        );
    }
}

/// The same-layout probe whose cleanup copies carry the identical guarded-throw tail: the
/// certificate's parameterized tail lets the one `finally` fold both copies, the injectable
/// cleanup failure included, and every physical instruction of both copies keeps an origin.
#[test]
fn cleanup_probe_recovers_one_finally_with_the_guarded_throw_tail() {
    let report = recover_test(CLEANUP_PROBE, "Tf4CleanupProbe", None);
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
        report.text.matches("if (!local1)").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("this.result -= 2;").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report
            .text
            .matches("if (Tf4CleanupProbe.failCleanup)")
            .count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report
            .text
            .matches(r#"throw new java.lang.RuntimeException("cleanup");"#)
            .count(),
        1,
        "{}",
        report.text
    );
    assert!(
        report.text.contains("local1 = false;") && report.text.contains("boolean local1;"),
        "the lead presents the false initialisation: {}",
        report.text
    );
    assert_eq!(report.regions.len(), 1);
    assert!(report.regions[0].structured, "{}", report.text);
    assert_eq!(
        report.regions[0].blocks,
        [0, 25, 41, 51, 53, 59, 75, 85],
        "{}",
        report.text
    );
    for bci in [
        0, 1, 2, 3, 6, 7, 8, 9, 12, 13, 14, 17, 18, 19, 20, 21, 22, 25, 26, 27, 30, 31, 32, 35,
        38, 41, 44, 45, 47, 50, 51, 52, 53, 55, 56, 59, 60, 61, 64, 65, 66, 69, 72, 75, 78, 79,
        81, 84, 85, 87,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no origin: {}",
            report.text
        );
    }
}

/// One refusal per broken certificate link, each a verifier-valid class the engine still
/// refuses: the negative list's order matches the six categories of the change's task 1.2.
#[test]
fn negative_without_a_set_true_write_refuses_the_conditional_finally() {
    refuse(0);
}

#[test]
fn negative_double_set_true_or_statement_after_the_write_refuses() {
    refuse(1);
    refuse(2);
}

#[test]
fn negative_inverted_condition_ifeq_refuses() {
    refuse(3);
}

#[test]
fn negative_copy_constant_or_field_mismatch_refuses() {
    refuse(8);
    refuse(9);
    refuse(10);
}

#[test]
fn negative_lead_not_false_or_multi_instruction_refuses() {
    refuse(4);
    refuse(5);
}

#[test]
fn negative_cleanup_grammar_changed_refuses() {
    refuse(6);
    refuse(7);
}

fn refuse(index: usize) {
    let name = NEGATIVE_NAMES[index];
    let report = recover_test(NEGATIVES[index], name, None);
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
