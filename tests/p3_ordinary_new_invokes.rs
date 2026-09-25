//! `new@1` must keep calls between allocation and construction attached to the constructor's
//! actual arguments. The negative input is the verifier-valid evidence class frozen by the
//! `refuse-unconsumed-construction-invokes` change; the positive input is javac output whose
//! constructor arguments are real method-call results.

use jarde::*;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::model::{
    JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId, PhysicalVariant,
};
use std::slice;

const VOID_BETWEEN: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/ordinary-new-void-effect/VoidBetween.class"
);
const CALL_ARGUMENTS: &[u8] = include_bytes!("fixtures/p3-two-exit-return/CountingRunner.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults to a bounded budget")
}

fn source(bytes: &[u8], name: &str) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("fixture bytes open as a class");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("class-source request completes")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one standalone class has one source report: {other:?}"),
    }
}

fn detailed_recovery(bytes: &[u8], method: &[u8], descriptor: &[u8]) -> RecoveryReport {
    let engine = Engine::new();
    let mut budget = budget();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("fixture bytes open as a class");
    let header = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the class header is readable");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: header.source.class_bytes,
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
            name: JvmBytes(method.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("the bounded recovery request completes")
        .recovery()
        .clone()
}

fn recovered<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let member = report
        .methods
        .iter()
        .find(|member| member.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing method `{name}`"));
    match &member.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("method `{name}` was not recovered: {other:?}"),
    }
}

#[test]
fn an_independent_void_call_refuses_the_construction_and_keeps_all_origins() {
    let report = source(VOID_BETWEEN, "VoidBetween");
    let class_source_body = recovered(&report, "make");
    let body = detailed_recovery(VOID_BETWEEN, b"make", b"()LTarget;");
    let site = body
        .news
        .iter()
        .find(|site| site.head == 0)
        .unwrap_or_else(|| panic!("no new@1 candidate; report={body:#?}"));

    assert!(!site.presented());
    let refusal = site
        .refusal
        .as_ref()
        .expect("the candidate carries its refusal");
    assert_eq!(refusal.code, "jre_new_interleaved_effect");
    assert_eq!(
        refusal.requirement.as_deref(),
        Some("a test block whose every instruction is part of a value expression")
    );
    assert!(refusal.message.contains("BCI 4"), "{refusal:?}");
    assert_eq!(body.quality, jarde::ir::Quality::Fallback);
    assert_eq!(body.representation, jarde::ir::Representation::Mixed);
    assert!(body.text.contains("@bytecode"), "{}", body.text);
    assert_eq!(class_source_body.quality, jarde::ir::Quality::Fallback);
    assert_eq!(
        class_source_body.representation,
        jarde::ir::Representation::Mixed
    );
    assert!(
        class_source_body.text.contains("// @bytecode 0"),
        "{}",
        class_source_body.text
    );
    assert!(
        class_source_body.text.contains("// @bytecode 3"),
        "{}",
        class_source_body.text
    );
    assert!(
        class_source_body.text.contains("Side.effect()"),
        "{}",
        class_source_body.text
    );
    assert!(
        class_source_body.text.contains("// @bytecode 11 8"),
        "{}",
        class_source_body.text
    );

    for bci in [0, 3, 4, 8] {
        assert!(
            body.source_map
                .of_bci(bci)
                .iter()
                .any(|segment| segment.mentions(bci).is_some()),
            "the rejected allocation, effect, argument and constructor retain BCI {bci}: {}",
            body.text
        );
    }
}

#[test]
fn a_call_whose_value_is_a_constructor_argument_remains_presented() {
    let report = source(CALL_ARGUMENTS, "CountingRunner");
    let class_source_body = recovered(&report, "check");
    let body = detailed_recovery(
        CALL_ARGUMENTS,
        b"check",
        b"(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
    );
    let site = body
        .news
        .iter()
        .find(|site| site.class == "CountingProbe")
        .unwrap_or_else(|| panic!("no CountingProbe site; report={body:#?}"));

    assert!(site.presented(), "{site:?}");
    assert!(!site.arguments.is_empty(), "{site:?}");
    assert!(
        body.text
            .contains("new CountingProbe((CountedValue) value(arg0),"),
        "{}",
        body.text
    );
    assert_eq!(body.text.matches("value(arg0)").count(), 1, "{}", body.text);
    assert_eq!(body.text.matches("value(arg1)").count(), 1, "{}", body.text);
    assert_eq!(class_source_body.quality, jarde::ir::Quality::Structured);
    assert!(
        class_source_body.text.contains("new CountingProbe("),
        "{}",
        class_source_body.text
    );
}
