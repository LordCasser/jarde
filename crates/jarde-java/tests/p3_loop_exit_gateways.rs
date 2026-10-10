//! The frozen CF-08 Java 8 loop gateways preserve both exit paths.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryFacts,
    RecoveryRequest, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension, Limits};
use jarde_reader::classfile::{DescriptorKind, descriptor_facts};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const FIXTURE: &[u8] = include_bytes!("../../../tests/fixtures/p3-loop-gateway/EndlessInts.class");
const NEGATIVES: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-loop-gateway/cf08/LoopGatewayNegatives.class");
// From the same Java 8 class: redirect sharedGateway's BCI 12 to its BCI 20 goto,
// add a BCI 20 StackMapTable frame, and update the enclosing attribute lengths.
// This gives the gateway a second normal predecessor and passes `java -Xverify:all`.
const EXTRA_ENTRY: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-loop-gateway/LoopGatewayExtraEntry.class");
const NO_PREFIX: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-10/one-arm-loop-controls/baseline-root-v1/cases/javac23-original/classes/PlainOneArmLoops.class",
);
const NO_PREFIX_BCIS: [u32; 11] = [0, 1, 2, 3, 6, 7, 8, 11, 14, 17, 18];

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
        "cf08/EndlessInts",
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
    let parameter_slots = descriptor_facts(descriptor.as_bytes(), DescriptorKind::Method)
        .expect("fixture method descriptor is valid")
        .parameter_slots()
        .expect("method descriptor states its parameter slots");
    let facts = RecoveryFacts::new(
        MethodFacts::new(name, descriptor, parameter_slots)
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
fn cf08_two_gateways_have_one_loop_owner_and_complete_sources() {
    let report = recover_method("find", "(I)I", RecoveryEvidenceRequest::all());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(report.text.contains("break;"), "{}", report.text);
    assert_eq!(report.text.matches("break;").count(), 1, "{}", report.text);
    assert_eq!(
        report.text.matches("local1 = local1 + 1;").count(),
        1,
        "{}",
        report.text
    );
    for bci in [4, 7, 12, 15, 18, 21, 24, 25] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost source: {}",
            report.text
        );
    }
    for bci in [2, 10, 15, 18] {
        let owners: Vec<_> = report
            .regions
            .iter()
            .filter(|r| r.blocks.contains(&bci))
            .collect();
        assert_eq!(owners.len(), 1, "BCI {bci} owners: {:?}", report.regions);
        assert!(owners[0].structured, "BCI {bci} is not structured");
    }
    let loop_owner = report
        .regions
        .iter()
        .find(|r| r.blocks.contains(&15))
        .unwrap();
    assert!(!loop_owner.blocks.contains(&7));
    assert!(!loop_owner.blocks.contains(&24));
}

#[test]
fn cf08_unproved_gateways_keep_physical_quotes() {
    for (class, name, gateway) in [
        (EXTRA_ENTRY, "sharedGateway", 20),
        (NEGATIVES, "differentTarget", 15),
        (NEGATIVES, "effectGateway", 15),
        (NEGATIVES, "exceptionalGateway", 23),
    ] {
        let report = recover_class_method_with_budget(
            class,
            "cf08/LoopGatewayNegatives",
            name,
            "(I)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "{name}: {:?}", report.outcome);
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
        assert!(
            !report.source_map.of_bci(gateway).is_empty(),
            "{name} lost BCI {gateway}"
        );
        assert!(
            !report.regions.iter().any(|region| region.structured
                && region.blocks.contains(&gateway)
                && region.rule.is_some_and(|rule| rule.rule() == "loop")),
            "{name} falsely claimed BCI {gateway}: {:?}",
            report.regions
        );
    }
}

#[test]
fn cf08_gateway_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;
    let full = recover_method_with_budget(
        "find",
        "(I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full candidate did not complete: {:?}", full.outcome);
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover_method_with_budget(
        "find",
        "(I)I",
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
        "find",
        "(I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}

#[test]
fn no_prefix_while_latch_keeps_physical_source_and_one_owner() {
    let default = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "noPrefix",
        "(ZI)I",
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        None,
    );
    let all = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "noPrefix",
        "(ZI)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(default.produced(), "default: {:?}", default.outcome);
    assert!(all.produced(), "all: {:?}", all.outcome);
    assert_eq!(default.text, all.text);
    assert_eq!(default.source_map, all.source_map);
    assert!(!all.text.contains("@bytecode"), "{}", all.text);

    for bci in NO_PREFIX_BCIS {
        assert!(
            !all.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost source: {}",
            all.text
        );
    }
    let latch_spans = all.source_map.derived_of_bci(14);
    assert!(
        latch_spans
            .iter()
            .any(|segment| segment.text(&all.text).trim_start().starts_with("while (")),
        "goto@14 is not derived from the loop statement: {}",
        all.text
    );
    let mut owner_budget = Budget::new(limits());
    let owner_snapshot =
        ArtifactSnapshot::open(ArtifactInput::bytes(NO_PREFIX.to_vec()), &mut owner_budget)
            .expect("frozen class snapshot opens");
    let expected_owner = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: owner_snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(NO_PREFIX).to_hex().to_string()),
            length: NO_PREFIX.len() as u64,
        },
        variant: PhysicalVariant::Base,
    };
    for segment in all.source_map.segments() {
        for origin in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert!(
                NO_PREFIX_BCIS.contains(&origin.bci()),
                "unexpected source-map BCI {}",
                origin.bci()
            );
            let method = origin
                .method()
                .expect("source-map origin has a physical method");
            assert_eq!(method.name.0.as_slice(), b"noPrefix");
            assert_eq!(method.descriptor.0.as_slice(), b"(ZI)I");
            assert_eq!(&method.owner, &expected_owner);
        }
    }
    for block_bci in [0, 6, 11, 17] {
        let owners: Vec<_> = all
            .regions
            .iter()
            .filter(|region| region.blocks.contains(&block_bci))
            .collect();
        assert_eq!(owners.len(), 1, "BCI {block_bci}: {:?}", all.regions);
        assert!(owners[0].structured, "BCI {block_bci}: {:?}", all.regions);
    }
}

#[test]
fn no_prefix_latch_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;

    let full = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "noPrefix",
        "(ZI)I",
        RecoveryEvidenceRequest::essential(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("full candidate did not complete: {:?}", full.outcome);
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps.saturating_sub(1);
    let stopped = recover_class_method_with_budget(
        NO_PREFIX,
        "PlainOneArmLoops",
        "noPrefix",
        "(ZI)I",
        RecoveryEvidenceRequest::essential(),
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
        NO_PREFIX,
        "PlainOneArmLoops",
        "noPrefix",
        "(ZI)I",
        RecoveryEvidenceRequest::essential(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}
