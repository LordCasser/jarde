//! CF-16 transparent empty-finally recovery and verifier-valid boundaries.

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
    "../../../openspec/evidence/java-syntax-2026-09-28/cf16-testemptyfinally/original/TestEmptyFinally$TestCls.class"
);
const NEIGHBORS: [(&str, &[u8]); 3] = [
    (
        "changed-throwable",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-testemptyfinally/neighbors/changed-throwable.class"
        ),
    ),
    (
        "rows-swapped",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-testemptyfinally/neighbors/rows-swapped.class"
        ),
    ),
    (
        "handler-normal-exit",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-testemptyfinally/neighbors/handler-normal-exit.class"
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
    parameters: u16,
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
        MethodFacts::new(name, descriptor, parameters)
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
fn target(class: &[u8], budget: Option<Budget>) -> jarde_java::RecoveryReport {
    recover_method(
        class,
        "test",
        "(Ljava/io/FileInputStream;)V",
        "jadx/tests/integration/trycatch/TestEmptyFinally$TestCls",
        0x0001,
        budget,
        2,
    )
}

#[test]
fn fixed_empty_finally_has_one_plain_catch_and_all_physical_origins() {
    let report = target(ORIGINAL, None);
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert!(report.fallbacks.is_empty(), "{}", report.text);
    assert!(report.text.contains(".close();"), "{}", report.text);
    assert_eq!(report.text.matches("catch (java.io.IOException").count(), 1);
    assert!(!report.text.contains("finally {"), "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    let blocks: Vec<_> = report
        .regions
        .iter()
        .flat_map(|region| &region.blocks)
        .copied()
        .collect();
    assert_eq!(blocks.len(), blocks.iter().collect::<BTreeSet<_>>().len());
    for bci in [0, 1, 4, 7, 8, 11, 12, 13, 14] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci}: {}",
            report.text
        );
    }
}

#[test]
fn verifier_valid_neighbors_keep_a_stated_refusal() {
    for (name, class) in NEIGHBORS {
        let report = target(class, None);
        assert!(!report.fallbacks.is_empty(), "{name}: {}", report.text);
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
        assert!(
            !report.text.contains(".close();"),
            "{name}: {}",
            report.text
        );
    }
}

#[test]
fn failed_body_build_quotes_the_whole_candidate() {
    // An intentionally incomplete caller handoff cannot place the parameter's declaration.
    let report = recover_method(
        ORIGINAL,
        "test",
        "(Ljava/io/FileInputStream;)V",
        "jadx/tests/integration/trycatch/TestEmptyFinally$TestCls",
        0x0001,
        None,
        0,
    );
    assert!(report.text.contains("@bytecode"), "{}", report.text);
    assert!(!report.text.contains(".close();"), "{}", report.text);
    assert!(
        !report.text.contains("catch (java.io.IOException"),
        "{}",
        report.text
    );
    for bci in [0, 1, 4, 7, 8, 11, 12, 13] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci}: {}",
            report.text
        );
    }
}

#[test]
fn budget_and_cancellation_stop_before_any_partial_source() {
    for budget in [
        {
            let mut limit = limits();
            limit.analysis_steps = 1;
            Budget::new(limit)
        },
        {
            let mut limit = limits();
            limit.analysis_steps = 64;
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
        let report = target(ORIGINAL, Some(budget));
        assert!(report.stop().is_some());
        assert!(report.text.is_empty() && report.source_map.is_empty());
    }
}
