//! Generic source projections preserve raw physical names across declarations and same-run proof joins.
use jarde::*;

fn report(name: &str, evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(
                include_bytes!("fixtures/s1-generic-physical-names/supplementary.jar").to_vec(),
            ),
            &mut budget,
        )
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
            loader: LoaderId("app".into()),
        },
    };
    match engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            evidence,
            &mut budget,
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source outcome: {other:?}"),
    }
}

fn with_both_evidence(name: &str, check: impl Fn(&ClassSourceReport)) {
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        check(&report(name, &evidence));
    }
}

#[test]
fn generic_field_uses_the_same_mutf8_name_as_its_physical_declaration() {
    with_both_evidence("FieldScope", |r| {
        assert!(r.text.contains("public T 𐐀Field;"), "{}", r.text);
        assert_eq!(
            r.fields[0].item.name.raw(),
            &JvmBytes::from_unicode("𐐀Field")
        );
        assert_eq!(r.fields[0].item.descriptor.raw().0, b"Ljava/lang/Object;");
    });
}

#[test]
fn method_generic_constructor_proof_keeps_its_supplementary_owner() {
    with_both_evidence("𐐀Ctor", |r| {
        assert!(
            r.text.contains("<T extends java.lang.Number> 𐐀Ctor(T "),
            "{}",
            r.text
        );
        let method = r
            .methods
            .iter()
            .find(|m| m.item.name.raw().0 == b"<init>")
            .unwrap();
        assert_eq!(method.item.descriptor.raw().0, b"(Ljava/lang/Number;)V");
    });
}

#[test]
fn class_generic_constructor_proof_keeps_its_supplementary_owner() {
    with_both_evidence("𐐀ClassCtor", |r| {
        assert!(r.text.contains("𐐀ClassCtor<T>"), "{}", r.text);
        assert!(r.text.contains("public 𐐀ClassCtor(T "), "{}", r.text);
        let method = r
            .methods
            .iter()
            .find(|m| m.item.name.raw().0 == b"<init>")
            .unwrap();
        assert_eq!(method.item.descriptor.raw().0, b"(Ljava/lang/Object;)V");
    });
}

fn check_relay(r: &ClassSourceReport, helper: &str) {
    assert!(
        r.text.contains("<T extends java.lang.Number> T pass(T "),
        "{}",
        r.text
    );
    assert!(
        r.text
            .contains(&format!("<T extends java.lang.Number> T {helper}(T ")),
        "{}",
        r.text
    );
    let method = r
        .methods
        .iter()
        .find(|m| m.item.name.raw() == &JvmBytes::from_unicode(helper))
        .unwrap();
    assert_eq!(
        method.item.descriptor.raw().0,
        b"(Ljava/lang/Number;)Ljava/lang/Number;"
    );
}

#[test]
fn private_generic_call_keeps_its_supplementary_method_name() {
    with_both_evidence("GenericBodyCallProbe", |r| check_relay(r, "𐐀Helper"));
}

#[test]
fn private_generic_call_keeps_its_supplementary_owner() {
    with_both_evidence("𐐀BodyProbe", |r| check_relay(r, "helper"));
}
