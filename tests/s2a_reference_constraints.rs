//! Public-entry regression for complete write/use constraints, independent of debug metadata.
use jarde::*;

#[test]
fn reference_declarations_reconcile_all_producers_copies_and_uses() {
    for bytes in [
        include_bytes!("fixtures/s2a-reference-constraints/jars/corretto8-debug.jar").as_slice(),
        include_bytes!("fixtures/s2a-reference-constraints/jars/corretto8-no-debug.jar"),
        include_bytes!("fixtures/s2a-reference-constraints/jars/openjdk23-debug.jar"),
        include_bytes!("fixtures/s2a-reference-constraints/jars/openjdk23-no-debug.jar"),
    ] {
        let report = source(bytes, "ReferenceCases", EnvironmentPolicy::PlainJar);
        for (name, declaration) in [
            ("interfaceCall", "ReferenceCases$Answer "),
            ("nullCopyChain", "java.lang.String "),
            ("covariantArrayRead", "java.lang.CharSequence[] "),
            ("objectExchange", "java.lang.Object "),
            ("explicitCast", "java.lang.Object "),
        ] {
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == name.as_bytes())
                .unwrap();
            let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
                panic!("{name} has no body");
            };
            assert_eq!(
                body.quality,
                jarde_jvm::ir::Quality::Structured,
                "{name}: {}",
                body.text
            );
            assert!(method.markers.is_empty(), "{name}: {:?}", method.markers);
            assert!(body.text.contains(declaration), "{name}: {}", body.text);
            if name == "nullCopyChain" {
                assert!(!body.text.contains("java.lang.Object "), "{}", body.text);
            }
            if name == "explicitCast" {
                assert!(body.text.contains("(java.lang.String) "), "{}", body.text);
            }
        }
    }
}

#[test]
fn null_and_differing_reference_phis_recover_as_typed_declarations() {
    for bytes in [
        include_bytes!("fixtures/s2a-reference-constraints/null-phi/jars/corretto8-debug.jar")
            .as_slice(),
        include_bytes!("fixtures/s2a-reference-constraints/null-phi/jars/corretto8-no-debug.jar"),
        include_bytes!("fixtures/s2a-reference-constraints/null-phi/jars/openjdk23-debug.jar"),
        include_bytes!("fixtures/s2a-reference-constraints/null-phi/jars/openjdk23-no-debug.jar"),
    ] {
        let report = source(bytes, "NullPhiChoices", EnvironmentPolicy::PlainJar);
        for (name, declaration, expression) in [
            ("bothNull", "java.lang.Object ", "? null : null;"),
            (
                "differingReferences",
                "java.lang.CharSequence ",
                "? \"left\" : new java.lang.StringBuilder(\"right\");",
            ),
        ] {
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == name.as_bytes())
                .unwrap();
            let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
                panic!("{name} has no body");
            };
            assert_eq!(
                body.quality,
                jarde_jvm::ir::Quality::Structured,
                "{name}: {}",
                body.text
            );
            assert!(method.markers.is_empty(), "{name}: {:?}", method.markers);
            assert!(body.text.contains(declaration), "{name}: {}", body.text);
            assert!(body.text.contains(expression), "{name}: {}", body.text);
        }
    }
}

#[test]
fn lambda_captures_preserve_the_original_overload_targets() {
    for bytes in [
        include_bytes!(
            "fixtures/s2a-reference-constraints/capture-upcasts/jars/corretto8-debug.jar"
        )
        .as_slice(),
        include_bytes!(
            "fixtures/s2a-reference-constraints/capture-upcasts/jars/corretto8-no-debug.jar"
        ),
        include_bytes!(
            "fixtures/s2a-reference-constraints/capture-upcasts/jars/openjdk23-debug.jar"
        ),
        include_bytes!(
            "fixtures/s2a-reference-constraints/capture-upcasts/jars/openjdk23-no-debug.jar"
        ),
    ] {
        let report = source(bytes, "CaptureUpcasts", EnvironmentPolicy::PlainJar);
        for (
            name,
            declaration,
            site_call,
            expanded_call,
            helper,
            helper_descriptor,
            implementation_call,
        ) in [
            (
                "one",
                "java.lang.Integer ",
                "lambda$one$0((java.lang.Number) ",
                "pick((java.lang.Number) ",
                "lambda$one$0",
                b"(Ljava/lang/Number;)Ljava/lang/String;".as_slice(),
                "pick(",
            ),
            (
                "two",
                "java.lang.String ",
                "lambda$two$1((java.lang.Object) ",
                "kind((java.lang.Object) ",
                "lambda$two$1",
                b"(Ljava/lang/Object;)Ljava/lang/String;".as_slice(),
                "kind(",
            ),
        ] {
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == name.as_bytes())
                .unwrap();
            let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
                panic!("{name} has no body");
            };
            assert_eq!(
                body.quality,
                jarde_jvm::ir::Quality::Structured,
                "{name}: {}",
                body.text
            );
            assert!(method.markers.is_empty(), "{name}: {:?}", method.markers);
            assert!(body.text.contains(declaration), "{name}: {}", body.text);
            assert!(body.text.contains(site_call), "{name}: {}", body.text);
            assert!(
                report.text.contains(expanded_call),
                "{name}: {}",
                report.text
            );

            let implementation = report
                .methods
                .iter()
                .find(|candidate| {
                    candidate.item.name.raw().0 == helper.as_bytes()
                        && candidate.item.descriptor.raw().0 == helper_descriptor
                })
                .unwrap();
            let ClassSourceOutcome::Recovered {
                report: implementation_body,
                ..
            } = &implementation.outcome
            else {
                panic!("{helper} has no body");
            };
            assert_eq!(
                implementation_body.quality,
                jarde_jvm::ir::Quality::Structured,
                "{helper}: {}",
                implementation_body.text
            );
            assert!(
                implementation.markers.is_empty(),
                "{helper}: {:?}",
                implementation.markers
            );
            assert!(
                implementation_body.text.contains(implementation_call),
                "{helper}: {}",
                implementation_body.text
            );
        }
    }
}

fn source(bytes: &[u8], class: &str, policy: EnvironmentPolicy) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".into()),
        },
    };
    let OperationOutcome::Performed(report) = engine
        .class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
        .unwrap()
    else {
        panic!("one physical root is selected");
    };
    report
}

#[test]
fn refined_locals_keep_original_reference_checks_legal() {
    let report = source(
        include_bytes!(
            "fixtures/s2a-reference-constraints/checked-reference/CheckedReference.class"
        ),
        "CheckedReference",
        EnvironmentPolicy::SingleClass,
    );
    for name in ["cast", "test"] {
        let method = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name.as_bytes())
            .unwrap();
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!("{name}");
        };
        assert_eq!(
            body.quality,
            jarde_jvm::ir::Quality::Structured,
            "{}",
            body.text
        );
        assert!(method.markers.is_empty(), "{:?}", method.markers);
        assert!(
            body.text.contains("java.lang.String local0"),
            "{}",
            body.text
        );
        assert!(
            body.text.contains("(java.lang.Object) local0"),
            "{}",
            body.text
        );
        assert!(body.text.contains("java.lang.Integer"), "{}", body.text);
    }
}

#[test]
fn captures_use_final_declarations_and_mutually_exclusive_writes() {
    let report = source(
        include_bytes!(
            "fixtures/s2a-reference-constraints/capture-use-boundary-v1/CapturedType.class"
        ),
        "CapturedType",
        EnvironmentPolicy::SingleClass,
    );
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .unwrap();
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!("run");
    };
    assert_eq!(
        body.quality,
        jarde_jvm::ir::Quality::Structured,
        "{}",
        body.text
    );
    assert!(method.markers.is_empty(), "{:?}", method.markers);
    assert!(
        body.text.contains("java.lang.Number local1"),
        "{}",
        body.text
    );
    assert!(
        report.text.contains("() -> pick(local1)"),
        "{:?}",
        report.text
    );
}
