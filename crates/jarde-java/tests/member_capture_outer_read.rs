//! The frozen Java 8 child has two Outer-typed values: parameter `other` and its captured outer.
//! Its class bytes were extracted unchanged from the stage-one `fixture.jar` for a crate-local
//! proof unit. Only the latter may become a lexical qualified `this`.

use jarde_java::{
    DebugLocal, DeclaringClass, MethodFacts, ProvedCapturedOuterRead, RecoveryEvidenceRequest,
    RecoveryFacts, RecoveryRequest, StopReason, pass::JAVA_8, recover,
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

const CHILD: &[u8] = include_bytes!("fixtures/NamedMemberFamilyStage1-Member.class");
const OWNER: &str = "NamedMemberFamilyStage1$Member";
const OUTER: &str = "NamedMemberFamilyStage1";
const READ_DESCRIPTOR: &str = "(LNamedMemberFamilyStage1;)I";

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
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn physical(definition: &PhysicalDefinitionId, name: &str, descriptor: &str) -> PhysicalMethodId {
    PhysicalMethodId {
        owner: definition.clone(),
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
    }
}

fn run(
    claim: Option<ProvedCapturedOuterRead>,
    all: bool,
) -> (jarde_java::RecoveryReport, ProvedCapturedOuterRead) {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(CHILD.to_vec()), &mut budget)
        .expect("frozen child class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(CHILD).to_hex().to_string()),
            length: CHILD.len() as u64,
        },
        variant: PhysicalVariant::Base,
    };
    let method = physical(&definition, "read", READ_DESCRIPTOR);
    let proven = ProvedCapturedOuterRead {
        method: method.clone(),
        read_bci: 8,
        field_owner: OWNER.to_owned(),
        field_name: "this$0".to_owned(),
        field_descriptor: format!("L{OUTER};"),
        outer_internal_name: OUTER.to_owned(),
        outer_source_name: OUTER.to_owned(),
        constructor: physical(&definition, "<init>", "(LNamedMemberFamilyStage1;)V"),
        constructor_write_bci: 2,
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
    let environment = ResolutionEnvironment {
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
    };
    let analysis = analyze_method_ir(
        &[snapshot],
        &MethodAnalysisRequest {
            environment,
            method,
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("frozen child read analyzes");
    let facts = RecoveryFacts::new(
        MethodFacts::new("read", READ_DESCRIPTOR, 2)
            .with_access_flags(0)
            .with_declaring_class(DeclaringClass::new(OWNER, 0x20)),
    )
    .with_debug_locals(vec![DebugLocal::named(1, "other")]);
    let claims = claim.into_iter().collect::<Vec<_>>();
    let mut request =
        RecoveryRequest::new(analysis.ir(), &facts, JAVA_8).with_captured_outer_reads(&claims);
    if all {
        request = request.with_evidence(RecoveryEvidenceRequest::all());
    }
    (recover(&request, &mut budget), proven)
}

#[test]
fn exact_capture_keeps_other_and_child_origins_in_both_evidence_modes() {
    let (_, proof) = run(None, false);
    let (default, _) = run(Some(proof.clone()), false);
    let (all, _) = run(Some(proof.clone()), true);
    for report in [&default, &all] {
        assert!(report.produced(), "{:?}", report.stop());
        assert!(report.text.contains("other.state"), "{}", report.text);
        assert!(
            report.text.contains("NamedMemberFamilyStage1.this"),
            "{}",
            report.text
        );
        assert!(!report.text.contains("this.this$0"), "{}", report.text);
    }
    assert_eq!(default.text, all.text);
    let mapped = all.source_map.of_bci(8);
    assert!(!mapped.is_empty(), "capture read BCI 8 must remain mapped");
    assert!(
        mapped.iter().any(
            |segment| segment.origin().primary().method() == Some(&proof.method)
                && segment.origin().primary().bci() == 8
                && all.source_map.segment_text(&all.text, segment)
                    == "NamedMemberFamilyStage1.this"
        )
    );
    let (ordinary, _) = run(None, false);
    assert!(ordinary.text.contains("other.state"), "{}", ordinary.text);
    assert!(ordinary.text.contains("this.this$0"), "{}", ordinary.text);
}

#[test]
fn stale_method_bci_and_field_facts_refuse_without_publishing_source() {
    let (_, proof) = run(None, false);
    let mut wrong_method = proof.clone();
    wrong_method.method.name = JvmBytes(b"otherMethod".to_vec());
    let mut wrong_bci = proof.clone();
    wrong_bci.read_bci = 1;
    let mut wrong_field = proof;
    wrong_field.field_name = "state".to_owned();
    for claim in [wrong_method, wrong_bci, wrong_field] {
        let (report, _) = run(Some(claim), false);
        assert!(
            matches!(
                report.stop(),
                Some(StopReason::EvidenceRefused {
                    code: "jre_captured_outer_read_refused",
                    ..
                })
            ),
            "{:?}",
            report.stop()
        );
        assert!(report.text.is_empty());
    }
}
