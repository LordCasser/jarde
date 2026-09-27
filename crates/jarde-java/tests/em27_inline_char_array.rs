//! The fixed Java 8 String(char[]) construction keeps allocation identity and physical origins.

use jarde_java::{MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest, recover};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Representation};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::classfile::class_facts;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const PROBE: &[u8] = include_bytes!("../../../tests/fixtures/em27-inline-string/em27/Probe.class");

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
        class_headers: 100,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn analyzed_direct() -> jarde_jvm::method_ir::MethodIrAnalysis {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(PROBE.to_vec()), &mut budget)
        .expect("frozen Java 8 class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(PROBE).to_hex().to_string()),
            length: PROBE.len() as u64,
        },
        variant: PhysicalVariant::Base,
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let request = MethodAnalysisRequest {
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
        method: PhysicalMethodId {
            owner: definition,
            name: JvmBytes(b"direct".to_vec()),
            descriptor: JvmBytes(b"()Ljava/lang/String;".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    analyze_method_ir(&[snapshot], &request, &mut budget).expect("direct method analyzes")
}

fn direct_facts() -> RecoveryFacts {
    let mut budget = Budget::new(limits());
    let header = class_facts(PROBE, &mut budget).expect("class facts");
    let member = header
        .methods
        .iter()
        .find(|member| member.name.raw().0 == b"direct")
        .expect("direct method exists");
    RecoveryFacts::new(MethodFacts::new(
        "direct",
        String::from_utf8_lossy(&member.descriptor.raw().0),
        0,
    ))
}

#[test]
fn direct_string_char_array_has_explicit_new_and_all_source_anchors() {
    let analysis = analyzed_direct();
    let facts = direct_facts();
    let mut budget = Budget::new(limits());
    let report = recover(
        &RecoveryRequest::new(analysis.ir(), &facts, jarde_java::pass::JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(report.representation, Representation::Java);
    assert!(
        report
            .text
            .contains("return new java.lang.String(new char[]{'a', 'b', 'c'});"),
        "{}",
        report.text
    );
    for bci in [0, 3, 5, 11, 16, 21, 22, 25] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "missing physical source at BCI {bci}"
        );
    }
}

#[test]
fn output_budget_and_cancellation_publish_no_partial_string_constructor() {
    let analysis = analyzed_direct();
    let facts = direct_facts();
    let mut tiny = limits();
    tiny.output_bytes = 8;
    let stopped = recover(
        &RecoveryRequest::new(analysis.ir(), &facts, jarde_java::pass::JAVA_8),
        &mut Budget::new(tiny),
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover(
        &RecoveryRequest::new(analysis.ir(), &facts, jarde_java::pass::JAVA_8),
        &mut Budget::with_cancellation_token(limits(), token),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
}
