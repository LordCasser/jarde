//! The frozen CF-03 Java 8 shared tail has one owner and one observable write.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
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
    include_bytes!("../../../tests/fixtures/p3-shared-tail/cf03/BranchShapes.class");
const EXTRA_ENTRY: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-shared-tail/Cf03Negative.class");
const INCOMPARABLE: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-shared-tail/Cf03Ambiguous.class");

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
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
) -> jarde_java::RecoveryReport {
    recover_method_with_budget(name, descriptor, evidence, None)
}

fn recover_method_with_budget(
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
    recover_class_method_with_budget(
        FIXTURE,
        "cf03/BranchShapes",
        name,
        descriptor,
        evidence,
        recovery_budget,
    )
}

fn recover_class_method_with_budget(
    class: &[u8],
    owner: &str,
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
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
                length: u64::try_from(class.len()).expect("fixture length fits"),
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
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
    .expect("fixture method analysis completes");
    let facts = RecoveryFacts::new(
        MethodFacts::new(name, descriptor, 1)
            .with_access_flags(0x0008)
            .with_declaring_class(DeclaringClass::new(owner, 0x0031)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8).with_evidence(evidence),
        &mut budget,
    )
}

#[test]
fn cf03_nested_claims_each_shared_leaf_and_tail_once() {
    let report = recover_method("nested", "(ZII)Z", RecoveryEvidenceRequest::all());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    for bci in [0, 4, 8, 12, 14, 18, 22, 24] {
        let owners: Vec<_> = report
            .regions
            .iter()
            .filter(|region| region.blocks.contains(&bci))
            .collect();
        assert_eq!(owners.len(), 1, "BCI {bci} owners: {:?}", report.regions);
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost source: {}",
            report.text
        );
    }
    for bci in [27, 28, 29, 32, 33] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "tail BCI {bci} lost source"
        );
    }
    assert_eq!(
        report.text.matches("BranchShapes.hits =").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("return true;").count(),
        1,
        "{}",
        report.text
    );
}

#[test]
fn cf03_independent_guards_stay_recovered() {
    let report = recover_method(
        "guards",
        "(Ljava/lang/String;)I",
        RecoveryEvidenceRequest::all(),
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
}

#[test]
fn cf03_candidate_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;
    let full = recover_method_with_budget(
        "nested",
        "(ZII)Z",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("the full candidate did not complete: {:?}", full.outcome);
    };
    assert!(usage.analysis_steps > 1);
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let late_stop = recover_method_with_budget(
        "nested",
        "(ZII)Z",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(late)),
    );
    assert!(!late_stop.produced());
    assert!(late_stop.text.is_empty() && late_stop.source_map.is_empty());
    assert!(matches!(
        late_stop.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));
    let mut tiny = limits();
    tiny.analysis_steps = 1;
    let stopped = recover_method_with_budget(
        "nested",
        "(ZII)Z",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(tiny)),
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_method_with_budget(
        "nested",
        "(ZII)Z",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}

#[test]
fn tail_entry_outside_the_outer_branch_does_not_gain_a_second_owner() {
    // The verifier-valid patched class has BCI 0→30 around the BCI 6 branch. BCI 30 also
    // receives the two inner true edges, so it is not owned by the branch at BCI 6.
    let report = recover_class_method_with_budget(
        EXTRA_ENTRY,
        "Cf03Negative",
        "extra",
        "(ZZII)Z",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        report.text.matches("Cf03Negative.hits =").count(),
        1,
        "{}",
        report.text
    );
    for bci in [0, 4, 10, 14, 18, 20, 24, 28, 30] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost source: {}",
            report.text
        );
    }
}

#[test]
fn incomparable_shared_candidates_do_not_select_an_arbitrary_tail() {
    // BCI 4 and 19 can both reach return leaves 23, 30 and 34. The first two are
    // incomparable: neither is a continuation of the other.
    let report = recover_class_method_with_budget(
        INCOMPARABLE,
        "Cf03Ambiguous",
        "f",
        "(ZII)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(report.text.contains("@bytecode"), "{}", report.text);
    for bci in [0, 4, 8, 11, 15, 19, 23, 26, 30, 34] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost source: {}",
            report.text
        );
    }
}
