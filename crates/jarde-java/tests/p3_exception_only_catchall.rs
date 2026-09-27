//! Single exception-only catch-all recovery and its verifier-valid boundaries.

use std::collections::BTreeSet;

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
};
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

const ORIGINAL: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-27/cf16-finally/original/FinallyOnce.class"
);
const ACCEPTANCE: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-exception-only-catchall/v8/FinallyOnce.class");
const IDENTITY: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-exception-only-catchall/v8/IdentityOnce.class");
const NEGATIVES: [(&str, &[u8]); 7] = [
    (
        "normal-exit",
        include_bytes!(
            "../../../tests/fixtures/p3-exception-only-catchall/negatives/normal-exit.class"
        ),
    ),
    (
        "range-shrunk",
        include_bytes!(
            "../../../tests/fixtures/p3-exception-only-catchall/negatives/range-shrunk.class"
        ),
    ),
    (
        "range-expanded",
        include_bytes!(
            "../../../tests/fixtures/p3-exception-only-catchall/negatives/range-expanded.class"
        ),
    ),
    (
        "competing-row",
        include_bytes!(
            "../../../tests/fixtures/p3-exception-only-catchall/negatives/competing-row.class"
        ),
    ),
    (
        "external-entry",
        include_bytes!(
            "../../../tests/fixtures/p3-exception-only-catchall/negatives/external-entry.class"
        ),
    ),
    (
        "wrong-rethrow",
        include_bytes!(
            "../../../tests/fixtures/p3-exception-only-catchall/negatives/wrong-rethrow.class"
        ),
    ),
    (
        "branch-cleanup",
        include_bytes!(
            "../../../tests/fixtures/p3-exception-only-catchall/negatives/branch-cleanup.class"
        ),
    ),
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
fn recover_method(
    class: &[u8],
    name: &str,
    descriptor: &str,
    owner: &str,
    flags: u16,
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("fixture opens");
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: class.len() as u64,
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
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
        MethodFacts::new(name, descriptor, 0)
            .with_access_flags(flags)
            .with_declaring_class(DeclaringClass::new(owner, 0x0021)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}
fn escaping(class: &[u8], budget: Option<Budget>) -> jarde_java::RecoveryReport {
    recover_method(class, "escaping", "()V", "FinallyOnce", 0x0009, budget)
}

#[test]
fn fixed_exception_only_catch_has_complete_source_and_unique_blocks() {
    for class in [ORIGINAL, ACCEPTANCE] {
        let report = escaping(class, None);
        assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
        assert!(
            report.text.contains("catch (java.lang.Throwable local0)"),
            "{}",
            report.text
        );
        assert_eq!(report.text.matches("cleanupCount + 1").count(), 1);
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        assert!(!report.text.contains("finally {"), "{}", report.text);
        let blocks: Vec<_> = report
            .regions
            .iter()
            .flat_map(|region| &region.blocks)
            .copied()
            .collect();
        assert_eq!(blocks.len(), blocks.iter().collect::<BTreeSet<_>>().len());
        for bci in [0, 1, 4, 7, 8, 10, 13, 14, 15, 18, 19, 20, 23, 24] {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "BCI {bci}: {}",
                report.text
            );
        }
    }
}

#[test]
fn caught_object_is_rethrown_unchanged() {
    let report = recover_method(IDENTITY, "escaping", "()V", "IdentityOnce", 0x0009, None);
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert!(report.text.contains("catch (java.lang.Throwable local0)"));
    assert!(report.text.contains("throw local0;"));
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
}

#[test]
fn verifier_valid_neighbors_never_publish_the_wrong_catch() {
    for (name, class) in NEGATIVES {
        let report = escaping(class, None);
        assert!(
            !report.text.contains("catch (java.lang.Throwable"),
            "{name}: {}",
            report.text
        );
        assert!(
            !report.text.contains("finally {"),
            "{name}: {}",
            report.text
        );
    }
}

#[test]
fn stop_cannot_publish_a_partial_catch() {
    for budget in [
        {
            let mut limit = limits();
            limit.analysis_steps = 1;
            Budget::new(limit)
        },
        {
            let mut limit = limits();
            limit.output_bytes = 1;
            Budget::new(limit)
        },
        {
            let token = CancellationToken::new();
            token.cancel();
            Budget::with_cancellation_token(limits(), token)
        },
    ] {
        let report = escaping(ACCEPTANCE, Some(budget));
        assert!(report.stop().is_some());
        assert!(report.text.is_empty() && report.source_map.is_empty());
    }
}
