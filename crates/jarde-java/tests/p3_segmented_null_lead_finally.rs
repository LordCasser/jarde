//! The fixed CF-16 Tf3 segmented null-lead: one finally, one folded `close`, the early
//! `return null;` inside the conditions, and every physical BCI owned.

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
    "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/fixture/Tf3.class"
);

const METHOD_DESCRIPTOR: &[u8] = b"()[B";

/// Verifier-valid neighbors the certificate must refuse. Each one breaks exactly one link of
/// the fixed grammar: the first two are compiled sources, the rest are bytecode patches
/// (`negatives/patch-tf3.py`) that rewrite one copy, one completion, one condition, or one
/// exception-table row.
const NEGATIVES: [&[u8]; 9] = [
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3GapExtra/Tf3GapExtra.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3LeadField/Tf3LeadField.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3TargetMismatch/Tf3TargetMismatch.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3ArgOtherSlot/Tf3ArgOtherSlot.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3ReturnIdentity/Tf3ReturnIdentity.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3SelfRowWidened/Tf3SelfRowWidened.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3CondNonNullRewrite/Tf3CondNonNullRewrite.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3CondIfneRewrite/Tf3CondIfneRewrite.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/negatives/src/Tf3RethrowIdentity/Tf3RethrowIdentity.class"
    ),
];

const NEGATIVE_NAMES: [&str; 9] = [
    "Tf3GapExtra",
    "Tf3LeadField",
    "Tf3TargetMismatch",
    "Tf3ArgOtherSlot",
    "Tf3ReturnIdentity",
    "Tf3SelfRowWidened",
    "Tf3CondNonNullRewrite",
    "Tf3CondIfneRewrite",
    "Tf3RethrowIdentity",
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
            1,
        )
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
fn segmented_null_lead_test_has_one_finally_one_folded_close_and_all_origins() {
    let report = recover_test(FIXED, "Tf3");
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert_eq!(report.quality, Quality::Structured);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        report.text.matches("finally {").count(),
        1,
        "{}",
        report.text
    );
    // The three unguarded copies fold into one cleanup call.
    assert_eq!(
        report.text.matches("close(local1);").count(),
        1,
        "{}",
        report.text
    );
    // The lead's `null` initialisation stays the statement's own separated declaration.
    assert!(
        report.text.contains("java.io.InputStream local1;"),
        "the cleanup local's declaration is hoisted: {}",
        report.text
    );
    assert!(
        report.text.contains("local1 = null;"),
        "the lead presents the null initialisation: {}",
        report.text
    );
    // The early return stays the conditions' own `return null;`.
    assert_eq!(
        report.text.matches("return null;").count(),
        1,
        "{}",
        report.text
    );
    // The two saved returns stay separate: the early literal inside the conditions, the
    // body producer's own value after them.
    assert!(
        report.text.contains("if (this.bytes == null) {"),
        "the body's field condition is presented: {}",
        report.text
    );
    assert!(
        report.text.contains("if (!this.validate()) {"),
        "the inner guard's condition is presented: {}",
        report.text
    );
    assert!(
        report.text.contains("local2 = this.convert(this.bytes);"),
        "the normal return's producer assignment is presented: {}",
        report.text
    );
    assert!(
        report.text.contains("return local2;"),
        "the normal return reads the saved slot back: {}",
        report.text
    );
    assert_eq!(report.regions.len(), 1);
    assert!(report.regions[0].structured, "{}", report.text);
    assert_eq!(
        report.regions[0].blocks,
        [0, 9, 16, 24, 38, 53],
        "{}",
        report.text
    );
    // Every physical instruction of the statement's span carries an origin: the lead, both
    // protected segments, the gap's early-return copy, the normal tail's copy, and the
    // handler's store, copy and rethrow.
    for bci in [
        0, 1, 2, 3, 6, 9, 10, 13, 16, 17, 18, 19, 22, 23, 24, 25, 28, 29, 30, 31, 32, 35, 38, 39,
        40, 43, 46, 47, 48, 51, 52, 53, 54, 55, 58, 59,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no origin: {}",
            report.text
        );
    }
}

/// One refusal per broken certificate link, each a verifier-valid class the engine still
/// refuses: the negative list's order covers the seven categories of the change's task 1.2
/// (the gap's extra statement is also the copies' grammar addition, as its fifth and sixth
/// instructions are no longer the fixed four).
#[test]
fn negative_gap_or_lead_grammar_refuses() {
    refuse(0);
    refuse(1);
}

#[test]
fn negative_copy_target_or_argument_slot_refuses() {
    refuse(2);
    refuse(3);
}

#[test]
fn negative_saved_return_or_self_row_or_condition_refuses() {
    refuse(4);
    refuse(5);
    refuse(6);
    refuse(7);
}

#[test]
fn negative_rethrow_identity_rewrite_refuses() {
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
