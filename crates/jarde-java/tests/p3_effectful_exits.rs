//! The frozen CF-08 Java 8 effectful exits preserve both paths and the joined local.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_jvm::method_ir::{Definition, PhiInput, Slot};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension, Limits};
use jarde_reader::model::ExecutionReport;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-effectful-exits/cf08effects/EffectfulExits.class");
const NEGATIVES: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-effectful-exits/cf08effects/EffectfulExitNegatives.class"
);
const NESTED: &[u8] = include_bytes!(
    "../../../tests/fixtures/cf08-nested-effectful/cf08nested/NestedEffectful.class"
);
const NESTED_NEGATIVES: &[u8] = include_bytes!(
    "../../../tests/fixtures/cf08-nested-effectful/cf08nested/NestedEffectfulNegatives.class"
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
        "cf08effects/EffectfulExits",
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
    if class == FIXTURE && name == "pick" {
        let ssa = analysis.ir().ssa().expect("fixture has SSA");
        let phi = ssa
            .phis()
            .iter()
            .find(|phi| phi.block().bci() == 35 && phi.slot() == Slot::Local(1))
            .expect("BCI 35 joins result slot");
        assert_eq!(phi.inputs().len(), 2);
        let inputs: Vec<_> = phi
            .inputs()
            .iter()
            .map(|input| match input {
                PhiInput::Value(value) => value,
                PhiInput::Itself => panic!("result phi cannot be cyclic"),
            })
            .collect();
        assert!(inputs.iter().any(|value| matches!(
            ssa.value(**value).def(),
            Definition::Instruction { bci: 13, .. }
        )));
        assert!(inputs.iter().any(|value| matches!(
            ssa.value(**value).def(),
            Definition::Instruction { bci: 20, .. }
        )));
        assert_eq!(ssa.value(phi.value()).uses().len(), 1);
        assert_eq!(ssa.value(phi.value()).uses()[0].bci(), Some(35));
    }
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
fn effectful_exits_own_each_block_and_source_once() {
    let report = recover_method("pick", "([I)I", RecoveryEvidenceRequest::all());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(report.text.contains("while (true)"), "{}", report.text);
    assert_eq!(report.text.matches("break;").count(), 2, "{}", report.text);
    assert!(report.text.contains("local1 = cost(7);"), "{}", report.text);
    assert!(report.text.contains("return local1 +"), "{}", report.text);
    for bci in [5, 8, 10, 13, 14, 17, 20, 23, 26, 29, 32, 35, 36, 39, 40] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "missing BCI {bci}: {}",
            report.text
        );
    }
    let loop_owner = report
        .regions
        .iter()
        .find(|region| {
            region.rule.is_some_and(|rule| rule.rule() == "loop") && region.blocks.contains(&2)
        })
        .expect("one loop owner");
    assert_eq!(loop_owner.blocks, vec![2, 8, 17, 26, 29]);
    assert!(!loop_owner.blocks.contains(&35));
}

#[test]
fn nested_effectful_exits_keep_the_inner_tail_in_its_if_arm() {
    let report = recover_class_method_with_budget(
        NESTED,
        "cf08nested/NestedEffectful",
        "pick",
        "([I)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(report.text.contains("while (true)"), "{}", report.text);
    assert_eq!(report.text.matches("break;").count(), 2, "{}", report.text);
    assert_eq!(report.text.matches("cost(7)").count(), 1, "{}", report.text);
    for bci in [
        0, 1, 4, 5, 9, 10, 11, 12, 13, 14, 17, 19, 22, 23, 26, 27, 28, 29, 30, 31, 32, 35, 38, 41,
        44, 45, 48, 49, 50, 51,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "missing BCI {bci}: {}",
            report.text
        );
    }
    let arm_owner = report
        .regions
        .iter()
        .find(|region| {
            region.rule.is_some_and(|rule| rule.rule() == "if") && region.blocks.contains(&11)
        })
        .expect("one enclosing arm owner");
    assert_eq!(arm_owner.blocks, vec![0, 4, 9, 11, 17, 26, 35, 38, 44]);
    assert!(!arm_owner.blocks.contains(&50));
    assert_eq!(
        report
            .regions
            .iter()
            .filter(|region| region.blocks.contains(&50))
            .count(),
        1
    );
}

#[test]
fn nested_effectful_exits_reject_unproved_edges() {
    for (name, header) in [
        ("extraEntry", 24),
        ("differentTarget", 11),
        ("bypassTail", 11),
        ("thirdJoinInput", 24),
        ("withHandler", 11),
    ] {
        let report = recover_class_method_with_budget(
            NESTED_NEGATIVES,
            "cf08nested/NestedEffectfulNegatives",
            name,
            "([I)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "{name}: {:?}", report.outcome);
        assert!(
            !report.text.contains("while (true)"),
            "{name}: {}",
            report.text
        );
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
        assert!(
            report
                .regions
                .iter()
                .any(|region| region.blocks.contains(&header)),
            "{name}: missing physical BCI {header}: {:?}",
            report.regions
        );
    }
}

#[test]
fn nested_effectful_certificate_stops_without_partial_source() {
    use jarde_java::StopReason;
    let full = recover_class_method_with_budget(
        NESTED,
        "cf08nested/NestedEffectful",
        "pick",
        "([I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full candidate did not complete: {:?}", full.outcome);
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover_class_method_with_budget(
        NESTED,
        "cf08nested/NestedEffectful",
        "pick",
        "([I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(late)),
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
    let cancelled = recover_class_method_with_budget(
        NESTED,
        "cf08nested/NestedEffectful",
        "pick",
        "([I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}

#[test]
fn non_matching_edges_effects_and_handler_do_not_publish_endless_loop() {
    for name in ["extraCall", "differentTarget", "extraLatch", "withHandler"] {
        let report = recover_class_method_with_budget(
            NEGATIVES,
            "cf08effects/EffectfulExitNegatives",
            name,
            "([I)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "{name}: {:?}", report.outcome);
        assert!(
            !report.text.contains("while (true)"),
            "{name}: {}",
            report.text
        );
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
    }
}

#[test]
fn certificate_budget_and_cancellation_leave_no_partial_source() {
    use jarde_java::StopReason;
    let full = recover_method_with_budget(
        "pick",
        "([I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full candidate did not complete: {:?}", full.outcome);
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover_method_with_budget(
        "pick",
        "([I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(late)),
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
        "pick",
        "([I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}
