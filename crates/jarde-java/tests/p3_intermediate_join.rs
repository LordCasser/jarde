//! The frozen Java 8 inner join remains owned by the enclosing arm.

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

const FIXTURE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-26/conditional-intermediate-join/ConditionalIntermediateJoin.class"
);
const NEGATIVE: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-intermediate-join/IntermediateJoinNegative.class");
const UNKNOWN_BRIDGE: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-intermediate-join/unknown-operation/ConditionalIntermediateJoin.class"
);

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
        "ConditionalIntermediateJoin",
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
fn inner_join_and_bridge_have_one_outer_arm_owner() {
    let report = recover_method("choose", "(I)I", RecoveryEvidenceRequest::all());
    assert!(report.produced(), "{:?}", report.outcome);
    for bci in [0, 4, 9, 15, 18, 23, 26] {
        let owners: Vec<_> = report
            .regions
            .iter()
            .filter(|region| region.blocks.contains(&bci))
            .collect();
        assert_eq!(
            owners.len(),
            1,
            "BCI {bci} must have one region owner: {:?}",
            report.regions
        );
    }
    let bridge_owner = report
        .regions
        .iter()
        .find(|region| region.blocks.contains(&18))
        .unwrap();
    assert!(bridge_owner.blocks.contains(&0) && bridge_owner.blocks.contains(&23));
    assert!(
        !report
            .regions
            .iter()
            .any(|region| region.code == Some("jre_region_uncovered_blocks"))
    );
    for bci in [18, 19, 20, 26] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no physical origin in the composed value"
        );
    }
}

#[test]
fn intermediate_join_stops_without_publishing_partial_artifact() {
    use jarde_java::StopReason;
    let mut tiny = limits();
    tiny.ir_items = 1;
    let report = recover_method_with_budget(
        "choose",
        "(I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(tiny)),
    );
    assert!(!report.produced());
    assert!(report.text.is_empty() && report.source_map.is_empty());
    assert!(matches!(
        report.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::IrItems,
            ..
        })
    ));
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_method_with_budget(
        "choose",
        "(I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}

#[test]
fn verifier_valid_independent_call_and_division_bridges_remain_referenced() {
    for (method, physical_bcis) in [
        ("independentEffect", &[18, 21, 22, 28][..]),
        ("throwingBridge", &[18, 19, 20, 26][..]),
    ] {
        let report = recover_class_method_with_budget(
            NEGATIVE,
            "IntermediateJoinNegative",
            method,
            "(I)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "{method}: {:?}", report.outcome);
        assert!(
            report.text.contains("@bytecode"),
            "{method}: {}",
            report.text
        );
        assert!(
            !report.text.contains(" ? "),
            "{method}: no partial ternary may publish"
        );
        for bci in physical_bcis {
            assert!(
                !report.source_map.of_bci(*bci).is_empty(),
                "{method}: BCI {bci} lost its physical source"
            );
        }
    }
}

#[test]
fn verifier_valid_unknown_bridge_keeps_the_whole_candidate_referenced() {
    let report = recover_class_method_with_budget(
        UNKNOWN_BRIDGE,
        "ConditionalIntermediateJoin",
        "choose",
        "(I)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(report.text.contains("@bytecode") && !report.text.contains(" ? "));
    for bci in [0, 4, 9, 15, 18, 19, 20, 23, 26] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost its physical source"
        );
    }
}

#[test]
fn external_inner_join_entry_and_handler_edge_refuse_the_whole_region() {
    // javap shows 9→28 in addition to 19→28 and 25→28: BCI 9 enters the inner
    // condition's join from outside that inner If. The caught variant has a real
    // ArithmeticException handler edge from its protected range.
    for (method, physical_bcis) in [
        (
            "externalEntry",
            &[9, 11, 14, 19, 22, 25, 28, 29, 30, 33, 36][..],
        ),
        (
            "caughtBridge",
            &[9, 12, 15, 18, 19, 20, 21, 22, 25, 28, 29, 30, 31][..],
        ),
    ] {
        let report = recover_class_method_with_budget(
            NEGATIVE,
            "IntermediateJoinNegative",
            method,
            "(I)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "{method}: {:?}", report.outcome);
        assert_eq!(
            report.regions[0].code,
            Some("jre_region_arms_do_not_meet"),
            "{method}: {:?}",
            report.regions
        );
        assert!(report.text.contains("@bytecode") && !report.text.contains(" ? "));
        for bci in physical_bcis {
            assert!(
                !report.source_map.of_bci(*bci).is_empty(),
                "{method}: BCI {bci} lost its physical source"
            );
        }
    }
}
