//! Raw Signature identifiers are JVM MUTF-8, shared by every source scope and reference.
use jarde::*;

fn report(name: &str, evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(
                include_bytes!("fixtures/s1-raw-signature-names/supplementary.jar").to_vec(),
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

#[test]
fn class_scope_keeps_one_raw_variable_across_header_field_and_method() {
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let result = report("ClassScope", &evidence);
        assert!(result.text.contains("ClassScope<𐐀T>"), "{}", result.text);
        assert!(result.text.contains("𐐀T value"), "{}", result.text);
        assert!(result.text.contains("𐐀T echo(𐐀T "), "{}", result.text);
        assert_eq!(
            result.fields[0].item.descriptor.raw().0,
            b"Ljava/lang/Object;"
        );
        let echo = result
            .methods
            .iter()
            .find(|m| m.item.name.raw().0 == b"echo")
            .unwrap();
        assert_eq!(
            echo.item.descriptor.raw().0,
            b"(Ljava/lang/Object;)Ljava/lang/Object;"
        );
    }
}

#[test]
fn method_scope_keeps_the_same_raw_variable_in_bounds_formals_and_result() {
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let result = report("MethodScope", &evidence);
        assert!(
            result
                .text
                .contains("<𐐀T extends java.lang.Number> 𐐀T choose(𐐀T "),
            "{}",
            result.text
        );
        let choose = result
            .methods
            .iter()
            .find(|m| m.item.name.raw().0 == b"choose")
            .unwrap();
        assert_eq!(
            choose.item.descriptor.raw().0,
            b"(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;"
        );
    }
}

#[test]
fn constructor_scope_keeps_the_raw_variable_after_body_and_erasure_proof() {
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let result = report("ConstructorScope", &evidence);
        assert!(
            result
                .text
                .contains("<𐐀T extends java.lang.Number> ConstructorScope(𐐀T "),
            "{}",
            result.text
        );
        assert_eq!(
            result.methods[0].item.descriptor.raw().0,
            b"(Ljava/lang/Number;)V"
        );
    }
}

#[test]
fn throws_scope_keeps_the_raw_method_variable_in_its_declared_throw_type() {
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let result = report("ThrowsScope", &evidence);
        assert!(
            result
                .text
                .contains("<𐐀T extends java.lang.RuntimeException> void run() throws 𐐀T"),
            "{}",
            result.text
        );
        let run = result
            .methods
            .iter()
            .find(|m| m.item.name.raw().0 == b"run")
            .unwrap();
        assert_eq!(run.item.descriptor.raw().0, b"()V");
    }
}
