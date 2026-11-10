//! The frozen Java 8 child has two Outer-typed values: parameter `other` and its captured outer.
//! Its class bytes were extracted unchanged from the stage-one `fixture.jar` for a crate-local
//! proof unit. Only the latter may become a lexical qualified `this`.

use jarde_java::{
    CapturedOuterReadInput, DebugLocal, DeclaringClass, MethodFacts, RecoveryEvidenceRequest,
    RecoveryFacts, RecoveryRequest, pass::JAVA_8, recover,
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

const CHILD: &[u8] =
    include_bytes!("../crates/jarde-java/tests/fixtures/NamedMemberFamilyStage1-Member.class");
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

fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
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
    ResolutionEnvironment {
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
    }
}

fn run_entry(
    claim: Option<CapturedOuterReadInput>,
    all: bool,
    case: &str,
    expected: Option<jarde_java::ArtifactBinding>,
    entry: &str,
) -> (jarde_java::RecoveryReport, CapturedOuterReadInput) {
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
    let proven = CapturedOuterReadInput {
        method: method.clone(),
        read_bci: 8,
        field_owner: JvmBytes::from_unicode(OWNER),
        field_name: jarde_reader::model::JvmBytes::from_unicode("this$0"),
        field_descriptor: JvmBytes::from_unicode(&format!("L{OUTER};")),
        outer_internal_name: JvmBytes::from_unicode(OUTER),
        outer_source_name: OUTER.to_owned(),
        constructor: physical(&definition, "<init>", "(LNamedMemberFamilyStage1;)V"),
        constructor_write_bci: 2,
    };
    let environment = environment(&snapshot);
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
        MethodFacts::new(
            if case == "facts" {
                "caller_forged"
            } else {
                "read"
            },
            READ_DESCRIPTOR,
            2,
        )
        .with_access_flags(0)
        .with_declaring_class(DeclaringClass::new(OWNER, 0x20)),
    )
    .with_debug_locals(vec![DebugLocal::named(1, "other")]);
    let claims = claim.into_iter().collect::<Vec<_>>();
    let mut subject_method = analysis.report().method.clone();
    if case == "subject" {
        subject_method.name = JvmBytes(b"caller_forged".to_vec());
    }
    let subject = jarde_java::ArtifactSubject::new(
        subject_method,
        None,
        analysis.report().environment_identity.clone(),
    );
    let mut request = RecoveryRequest::new(
        analysis.ir(),
        &facts,
        JAVA_8,
        jarde_reader::view::RuntimeProfile {
            java_release: 8,
            multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
            layout: jarde_reader::view::LayoutMode::Generic,
        },
    )
    .with_captured_outer_reads(&claims)
    .with_subject(subject);
    if case == "runtime" {
        request.runtime_profile.java_release = 9;
    }
    if all {
        request = request.with_evidence(RecoveryEvidenceRequest::all());
    }
    if let Some(expected) = expected {
        request =
            request.with_evidence(RecoveryEvidenceRequest::all().with_expected_artifact(expected));
    }
    let mut allow = |_: &str, _: jarde_java::ArtifactConfiguration| true;
    let report = match entry {
        "method" => recover(&request, &mut budget),
        "method_gate" => {
            jarde_java::report::recover_with_evidence_gate(&request, &mut budget, Some(&mut allow))
        }
        "class" => {
            jarde_java::report::recover_for_class_source(
                &request,
                &mut budget,
                jarde_java::report::ClassSourceOptions::default(),
                None,
            )
            .report
        }
        "class_ast" => {
            jarde_java::report::recover_for_class_source(
                &request,
                &mut budget,
                jarde_java::report::ClassSourceOptions {
                    ast_retention: jarde_java::report::ClassSourceAstRetention::All,
                    ..Default::default()
                },
                None,
            )
            .report
        }
        "class_gate" => {
            jarde_java::report::recover_for_class_source(
                &request,
                &mut budget,
                jarde_java::report::ClassSourceOptions::default(),
                Some(&mut allow),
            )
            .report
        }
        _ => panic!("unknown test entry"),
    };
    (report, proven)
}

fn run(
    claim: Option<CapturedOuterReadInput>,
    case: &str,
) -> (jarde_java::RecoveryReport, CapturedOuterReadInput) {
    run_entry(claim, true, case, None, "method")
}

#[test]
fn caller_inputs_cannot_mint_read_bindings_even_at_a_matching_consumer() {
    let (control, proof) = run(None, "control");
    let mut forged = proof;
    // This constructor does not exist; the site-local shape check cannot establish its capture.
    forged.constructor.descriptor = JvmBytes(b"(LNamedMemberFamilyStage1;I)V".to_vec());
    forged.constructor_write_bci = u32::MAX;
    let mut rows = vec![("control", control)];
    rows.push(("forged_capture_constructor", run(Some(forged), "capture").0));
    for case in ["facts", "subject", "runtime"] {
        rows.push((case, run(None, case).0));
    }
    for (_, report) in &rows {
        assert!(
            report.produced(),
            "preview capability is preserved: {:?}",
            report.stop()
        );
        assert_eq!(
            report.input_origin,
            jarde_java::report::RecoveryInputOrigin::CallerSupplied
        );
        assert!(report.artifact.binding().is_none());
    }
    assert!(
        rows[1].1.text.contains("NamedMemberFamilyStage1.this"),
        "the fake reaches the actual consumer; isolation, not a non-matching shape, protects the boundary"
    );
    if let Ok(output) = std::env::var("JARDE_TRUST_PROBE_OUTPUT") {
        let rows: Vec<_> = rows.into_iter().map(|(case, report)| serde_json::json!({
            "case":case,"produced":report.produced(),"binding_present":report.artifact.binding().is_some(),
            "qualified_outer_this":report.text.contains("NamedMemberFamilyStage1.this"),"report":report})).collect();
        std::fs::write(output, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
}

#[test]
fn only_the_read_owner_can_verify_an_expected_artifact() {
    let (_, proof) = run(None, "control");
    let mut budget = Budget::new(limits());
    let snapshot =
        ArtifactSnapshot::open(ArtifactInput::bytes(CHILD.to_vec()), &mut budget).unwrap();
    let request = MethodAnalysisRequest {
        environment: environment(&snapshot),
        method: proof.method.clone(),
        stages: AnalysisStage::ALL.to_vec(),
    };
    let engine = jarde::Engine::new();
    let read = engine
        .recover_method_with_evidence(
            std::slice::from_ref(&snapshot),
            JAVA_8,
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .unwrap();
    assert_eq!(
        read.recovery().input_origin,
        jarde_java::report::RecoveryInputOrigin::SelectedReads
    );
    let binding = read.recovery().artifact.binding().unwrap().clone();
    let again = engine
        .recover_method_with_evidence(
            std::slice::from_ref(&snapshot),
            JAVA_8,
            &request,
            &RecoveryEvidenceRequest::all().with_expected_artifact(binding.clone()),
            &mut Budget::new(limits()),
        )
        .unwrap();
    assert_eq!(
        again.recovery().artifact.agreement(),
        &jarde_java::ArtifactAgreement::Agreed
    );
    assert_eq!(again.recovery().text, read.recovery().text);
    for entry in ["method", "method_gate", "class", "class_ast", "class_gate"] {
        let (report, _) = run_entry(None, true, "control", Some(binding.clone()), entry);
        assert!(report.produced());
        assert_eq!(
            report.input_origin,
            jarde_java::report::RecoveryInputOrigin::CallerSupplied
        );
        assert!(report.artifact.binding().is_none(), "{entry}");
        assert!(
            matches!(
                report.artifact.agreement(),
                jarde_java::ArtifactAgreement::Unverifiable { .. }
            ),
            "{entry}: {:?}",
            report.artifact.agreement()
        );
        for kind in jarde_java::RecoveryEvidenceKind::SUPPORTED {
            assert_eq!(
                report.evidence.state(kind),
                jarde_java::EvidenceState::NotPerformed,
                "{entry}: {kind:?}"
            );
        }
    }
}
