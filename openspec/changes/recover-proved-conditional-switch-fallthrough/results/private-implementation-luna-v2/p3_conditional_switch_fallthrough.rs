//! The frozen CF12 conditional case keeps its adjacent-case path and explicit switch-exit paths.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
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

const CLASS: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchWithFallThroughCase.test/input/TestSwitchWithFallThroughCase$TestCls.class"
);
const OWNER: &str = "jadx/tests/integration/switches/TestSwitchWithFallThroughCase$TestCls";
const DESCRIPTOR: &str = "(IZZ)Ljava/lang/String;";

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

fn report() -> jarde_java::RecoveryReport {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("frozen complete target class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(CLASS).to_hex().to_string()),
            length: u64::try_from(CLASS.len()).expect("fixture length fits"),
        },
        variant: PhysicalVariant::Base,
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
        &[snapshot],
        &MethodAnalysisRequest {
            environment: ResolutionEnvironment {
                runtime: RuntimeView {
                    physical: PhysicalView {
                        snapshot: definition_snapshot(&definition),
                        scope: PhysicalScope::SnapshotAll,
                    },
                    profile: RuntimeProfile {
                        java_release: 23,
                        multi_release: MultiReleasePolicy::Disabled,
                        layout: LayoutMode::Generic,
                    },
                    load_domain: domain.clone(),
                },
                domains: vec![domain],
                providers: Vec::new(),
            },
            method: PhysicalMethodId {
                owner: definition,
                name: JvmBytes(b"test".to_vec()),
                descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
            },
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("complete class method analysis succeeds");
    let facts = RecoveryFacts::new(
        MethodFacts::new("test", DESCRIPTOR, 4)
            .with_access_flags(0x0001)
            .with_declaring_class(DeclaringClass::new(OWNER, 0x0021)),
    );
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

// Keep the requested PhysicalView tied to the exact snapshot already embedded in the owner.
fn definition_snapshot(definition: &PhysicalDefinitionId) -> jarde_reader::model::SnapshotId {
    match &definition.location {
        PhysicalClassLocation::StandaloneRoot { snapshot } => snapshot.clone(),
        _ => unreachable!("the test uses one standalone frozen class"),
    }
}

#[test]
fn cf12_conditional_switch_paths_keep_case_and_join_breaks() {
    assert_eq!(CLASS.len(), 1563, "frozen input class changed");
    let report = report();
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        !report.text.contains("@bytecode"),
        "the real method must be fully structured:\n{}",
        report.text
    );
    assert!(report.text.contains("case 1:"), "{}", report.text);
    assert_eq!(report.text.matches("case 2:").count(), 1, "{}", report.text);
    let switch = report
        .regions
        .iter()
        .find(|region| region.bci == 0 && region.rule.is_some_and(|rule| rule.rule() == "switch"))
        .expect("the real switch has a structured region record");
    assert!(switch.blocks.contains(&117), "case 2 must belong to the switch: {switch:?}");
    assert!(!switch.blocks.contains(&171), "the common join must remain after the switch: {switch:?}");
    assert_eq!(
        report.text.matches("break;").count(),
        2,
        "the two paths to BCI 171 need explicit switch exits:\n{}",
        report.text
    );
    for bci in [89, 114] {
        assert!(
            report
                .source_map
                .of_bci(bci)
                .iter()
                .any(|segment| segment.text(&report.text).contains("break;")),
            "join-transfer BCI {bci} must map to its own break statement:\n{}",
            report.text
        );
    }
    assert!(
        !report
            .regions
            .iter()
            .any(|region| region.rule.is_some_and(|rule| rule.rule() == "loop")),
        "the switch graph is acyclic and must not be reported as a loop: {:?}",
        report.regions
    );
    for bci in [32, 59, 63, 67, 89, 92, 114, 117, 121, 171, 195] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost source provenance:\n{}",
            report.text
        );
    }
}
