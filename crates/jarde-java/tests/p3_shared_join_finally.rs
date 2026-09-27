//! The fixed JADX shared-join finally class has one owner for every physical block.

use std::collections::BTreeSet;

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    StopReason, pass::JAVA_8, recover,
};
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

const FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoin.class");
const NEGATIVES: [&[u8]; 3] = [
    include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoinValueMismatch.class"
    ),
    include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoinRow.class"
    ),
    include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoinEdge.class"
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

fn recover_test(class: &[u8], recovery_budget: Option<Budget>) -> jarde_java::RecoveryReport {
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
        name: JvmBytes(b"test".to_vec()),
        descriptor: JvmBytes(b"(Ljava/lang/Object;)Z".to_vec()),
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
        MethodFacts::new("test", "(Ljava/lang/Object;)Z", 2)
            .with_access_flags(0x0002)
            .with_declaring_class(DeclaringClass::new("TestTryCatchFinally$TestCls", 0x0021)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

#[test]
fn fixed_shared_join_has_one_finally_and_complete_physical_ownership() {
    let report = recover_test(FIXTURE, None);
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert_eq!(report.text.matches("this.f = true;").count(), 1);
    assert!(report.text.contains("finally {"), "{}", report.text);
    assert!(
        report.text.contains("}\n    return this.f;"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    let blocks: Vec<_> = report
        .regions
        .iter()
        .flat_map(|region| &region.blocks)
        .copied()
        .collect();
    assert_eq!(
        blocks.iter().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([0, 18, 31, 39])
    );
    assert_eq!(blocks.len(), 4, "each physical block has one region owner");
    for bci in [
        0, 1, 2, 5, 6, 9, 10, 11, 12, 15, 18, 19, 20, 23, 24, 25, 28, 31, 32, 33, 34, 37, 38, 39,
        40, 43,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no source origin"
        );
    }
}

#[test]
fn verifier_valid_boundary_variants_do_not_publish_a_partial_finally() {
    for class in NEGATIVES {
        let report = recover_test(class, None);
        assert!(!report.text.contains("finally {"), "{}", report.text);
        assert!(
            report.text.contains("BCI 31") && report.text.contains("@bytecode"),
            "{}",
            report.text
        );
    }
}

#[test]
fn shared_join_stop_discards_text_and_source_map() {
    let mut tiny = limits();
    tiny.ir_items = 1;
    let stopped = recover_test(FIXTURE, Some(Budget::new(tiny)));
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::IrItems,
            ..
        })
    ));
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_test(
        FIXTURE,
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}
