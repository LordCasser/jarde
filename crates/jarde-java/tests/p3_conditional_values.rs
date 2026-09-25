//! `recover-conditional-values`: only an `if` whose stack join, arms and consumer are proved once
//! becomes a lazy Java `?:` expression.

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
    include_bytes!("../../../tests/fixtures/p3-conditional-values/v8/TernaryValues.class");

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
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(FIXTURE.to_vec()), &mut budget)
        .expect("fixture opens");
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(FIXTURE).to_hex().to_string()),
                length: u64::try_from(FIXTURE.len()).expect("fixture length fits"),
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
            .with_declaring_class(DeclaringClass::new("TernaryValues", 0x0021)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8).with_evidence(evidence),
        &mut budget,
    )
}

#[test]
fn proved_values_are_one_lazy_typed_expression_in_each_supported_position() {
    for (name, descriptor, source_bcis) in [
        ("returned", "(Z)I", &[1, 4, 7, 10, 13][..]),
        ("assigned", "(Z)I", &[1, 4, 7, 10, 13][..]),
        ("arithmetic", "(Z)I", &[2, 5, 8, 11, 14][..]),
        ("callArgument", "(Z)I", &[3, 6, 9, 12, 15][..]),
        ("reference", "(Z)Ljava/lang/String;", &[1, 4, 5, 8, 10][..]),
        (
            "overloadChoice",
            "(Z)Ljava/lang/String;",
            &[1, 4, 5, 8, 10][..],
        ),
        ("throwing", "(Z)I", &[1, 4, 7, 10, 13][..]),
    ] {
        let essential = recover_method(name, descriptor, RecoveryEvidenceRequest::essential());
        let all = recover_method(name, descriptor, RecoveryEvidenceRequest::all());
        assert!(essential.produced(), "{name}: {:?}", essential.outcome);
        assert!(all.produced(), "{name}: {:?}", all.outcome);
        assert_eq!(
            essential.text, all.text,
            "{name}: evidence changed the body"
        );
        for bci in source_bcis {
            assert!(
                !all.source_map.of_bci(*bci).is_empty(),
                "{name}: branch/arm/transfer/consumer BCI {bci} is absent from the all-evidence source map"
            );
        }
        assert!(
            essential.text.contains(" ? ")
                && essential.text.contains(" : ")
                && !essential.text.contains("@bytecode"),
            "{name}: expected one complete conditional expression:\n{}",
            essential.text
        );
        if matches!(
            name,
            "returned" | "assigned" | "arithmetic" | "callArgument" | "throwing"
        ) {
            assert_eq!(
                essential.text.matches("a()").count(),
                1,
                "{name}: true producer must appear once"
            );
            assert_eq!(
                essential.text.matches("b()").count(),
                1,
                "{name}: false producer must appear once"
            );
        } else {
            assert!(
                essential.text.contains("null") && essential.text.contains("\"value\""),
                "{name}: both reference arms must appear once:\n{}",
                essential.text
            );
        }
    }
    let overload = recover_method(
        "overloadChoice",
        "(Z)Ljava/lang/String;",
        RecoveryEvidenceRequest::essential(),
    );
    assert!(
        overload
            .text
            .contains("overload((arg0 != 0) ? null : \"value\")")
            || overload.text.contains("overload(arg0 ? null : \"value\")"),
        "the proved String conditional must preserve the Methodref overload:\n{}",
        overload.text
    );
}

#[test]
fn a_conditional_value_cannot_publish_partial_text_on_budget_or_cancellation() {
    use jarde_java::StopReason;

    let mut output_limits = limits();
    output_limits.output_bytes = 1;
    let output = recover_method_with_budget(
        "returned",
        "(Z)I",
        RecoveryEvidenceRequest::essential(),
        Some(Budget::new(output_limits)),
    );
    assert!(!output.produced());
    assert!(output.text.is_empty() && output.source_map.is_empty());
    assert!(matches!(
        output.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::OutputBytes,
            ..
        })
    ));

    let mut ir_limits = limits();
    ir_limits.ir_items = 1;
    let ir = recover_method_with_budget(
        "returned",
        "(Z)I",
        RecoveryEvidenceRequest::essential(),
        Some(Budget::new(ir_limits)),
    );
    assert!(!ir.produced());
    assert!(ir.text.is_empty() && ir.source_map.is_empty());
    assert!(matches!(
        ir.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::IrItems,
            ..
        })
    ));

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_method_with_budget(
        "returned",
        "(Z)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(
        cancelled
            .stop()
            .expect("cancelled run has a stop")
            .is_cancelled()
    );
}
