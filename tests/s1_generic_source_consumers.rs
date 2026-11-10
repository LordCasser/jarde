//! Supplementary Java identifiers keep same-class raw receiver and List Signature proofs intact.
use jarde::class_source::ClassSourceOutcome;
use jarde::*;

const RECEIVER_ASCII: &[u8] =
    include_bytes!("fixtures/s1-generic-source-consumers/receiver-ascii.jar");
const RECEIVER_SUPPLEMENTARY: &[u8] =
    include_bytes!("fixtures/s1-generic-source-consumers/receiver-supplementary.jar");
const RECEIVER_DOLLAR_ASCII: &[u8] =
    include_bytes!("fixtures/s1-generic-source-consumers/receiver-dollar-ascii.jar");
const RECEIVER_DOLLAR_SUPPLEMENTARY: &[u8] =
    include_bytes!("fixtures/s1-generic-source-consumers/receiver-dollar-supplementary.jar");
const LIST_ASCII: &[u8] = include_bytes!("fixtures/s1-generic-source-consumers/list-ascii.jar");
const LIST_SUPPLEMENTARY: &[u8] =
    include_bytes!("fixtures/s1-generic-source-consumers/list-supplementary.jar");
const MEMBER_PATH: &[u8] = include_bytes!("fixtures/s1-generic-source-consumers/member-path.jar");

fn request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    }
}

fn source_with_evidence(
    bytes: &[u8],
    class: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).expect("open limits are available");
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut open_budget)
        .expect("the frozen fixture jar opens");
    let request = request(&snapshot, class);
    let mut budget = task_budget(&[]).expect("task limits are available");
    match engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            evidence,
            &mut budget,
        )
        .expect("the bounded class-source request completes")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen class-source request performs: {other:?}"),
    }
}

fn evidence_requests() -> [RecoveryEvidenceRequest; 3] {
    [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        RecoveryEvidenceRequest::all(),
    ]
}

fn method<'a>(report: &'a ClassSourceReport, name: &[u8]) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name)
        .unwrap_or_else(|| panic!("missing physical method {:?}:\n{}", name, report.text))
}

#[test]
fn supplementary_same_class_raw_receivers_keep_the_generic_callee_across_evidence_modes() {
    for (class, bytes, accept_declaration, descriptor) in [
        (
            "Subject",
            RECEIVER_ASCII,
            "public void accept(T arg1)",
            "(Ljava/lang/Object;)V",
        ),
        (
            "𐐀Subject",
            RECEIVER_SUPPLEMENTARY,
            "public void accept(T arg1)",
            "(Ljava/lang/Object;)V",
        ),
        (
            "Top$Subject",
            RECEIVER_DOLLAR_ASCII,
            "public <T extends java.lang.Object> void accept(T arg1, boolean arg2)",
            "(Ljava/lang/Object;Z)V",
        ),
        (
            "𐐀Top$𐐀Subject",
            RECEIVER_DOLLAR_SUPPLEMENTARY,
            "public <T extends java.lang.Object> void accept(T arg1, boolean arg2)",
            "(Ljava/lang/Object;Z)V",
        ),
    ] {
        let mut previous_text: Option<String> = None;
        for evidence in evidence_requests() {
            let report = source_with_evidence(bytes, class, &evidence);
            let accept = method(&report, b"accept");
            assert_eq!(
                accept.item.descriptor.raw(),
                &JvmBytes::from_unicode(descriptor),
                "the generic formal retains its physical erased descriptor:\n{}",
                report.text
            );
            assert!(
                accept
                    .declaration
                    .as_deref()
                    .is_some_and(|declaration| declaration.contains(accept_declaration)),
                "the generic accept header is retained for {class}:\n{}",
                report.text
            );

            let relay = method(&report, b"relay");
            assert_eq!(
                relay.item.descriptor.raw(),
                &JvmBytes::from_unicode(descriptor),
                "the raw caller keeps its physical descriptor:\n{}",
                report.text
            );
            let ClassSourceOutcome::Recovered { report: body, .. } = &relay.outcome else {
                panic!("raw relay body is recovered:\n{}", relay.text);
            };
            assert_eq!(
                body.text.matches(".accept(").count(),
                2,
                "both raw calls remain in the recovered body:\n{}",
                body.text
            );

            if let Some(previous) = &previous_text {
                assert_eq!(
                    &report.text, previous,
                    "evidence selection does not change the class source for {class}"
                );
            }
            previous_text = Some(report.text.clone());
        }
    }
}

#[test]
fn supplementary_list_signature_elements_keep_typed_foreach_across_evidence_modes() {
    for (class, bytes, element) in [
        ("Subject", LIST_ASCII, "Item"),
        ("Subject", LIST_SUPPLEMENTARY, "𐐀Item"),
    ] {
        let mut previous_text: Option<String> = None;
        for evidence in evidence_requests() {
            let report = source_with_evidence(bytes, class, &evidence);
            let consume = method(&report, b"consume");
            assert_eq!(
                consume.item.descriptor.raw(),
                &JvmBytes::from_unicode("(Ljava/util/List;)V"),
                "the List parameter keeps its physical erased descriptor:\n{}",
                report.text
            );
            assert!(
                consume.declaration.as_deref().is_some_and(|declaration| {
                    declaration.contains(&format!("java.util.List<{element}>"))
                }),
                "the List Signature retains {element}:\n{}",
                report.text
            );
            let ClassSourceOutcome::Recovered { .. } = &consume.outcome else {
                panic!("typed foreach body is recovered:\n{}", consume.text);
            };
            assert!(
                consume.text.contains(&format!("for ({element} ")),
                "the projected method text uses the proved List element type {element}:\n{}",
                consume.text
            );

            if let Some(previous) = &previous_text {
                assert_eq!(
                    &report.text, previous,
                    "evidence selection does not change the class source for {class}/{element}"
                );
            }
            previous_text = Some(report.text.clone());
        }
    }
}

#[test]
fn member_path_raw_receivers_keep_the_generic_method_header_across_evidence_modes() {
    let mut previous_text: Option<String> = None;
    for evidence in evidence_requests() {
        let report = source_with_evidence(MEMBER_PATH, "Outer$Member", &evidence);
        let accept = method(&report, b"accept");
        assert_eq!(
            accept.item.descriptor.raw(),
            &JvmBytes::from_unicode("(Ljava/lang/Object;Z)V"),
            "the generic method keeps its physical erased descriptor:\n{}",
            report.text
        );
        assert!(
            accept.declaration.as_deref().is_some_and(|declaration| {
                declaration.contains(
                    "public <T extends java.lang.Object> void accept(T arg1, boolean arg2)",
                )
            }),
            "the member-path generic header is retained:\n{}",
            report.text
        );

        let relay = method(&report, b"relay");
        assert!(
            relay.text.contains("Outer.Member local"),
            "the recovered receiver remains sourceable as Outer.Member:\n{}",
            relay.text
        );
        if let Some(previous) = &previous_text {
            assert_eq!(
                &report.text, previous,
                "evidence selection keeps full text stable"
            );
        }
        previous_text = Some(report.text.clone());
    }
}
