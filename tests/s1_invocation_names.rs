//! Raw invocation names and their declarations share one Java 7/8 spelling boundary.
use jarde::*;
use jarde_jvm::ir::Quality;

fn recovered(bytes: &[u8], owner: &str, evidence: RecoveryEvidenceRequest) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(owner),
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
        .class_source_with_evidence(&[snapshot], &request, &evidence, &mut budget)
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source outcome: {other:?}"),
    }
}

fn run_body(report: &ClassSourceReport) -> &RecoveryReport {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .unwrap();
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("run was not recovered: {other:?}"),
    }
}

fn controls(inputs: &[&[u8]], underscore: bool) {
    for input in inputs {
        let mut previous = None;
        for evidence in [
            RecoveryEvidenceRequest::essential(),
            RecoveryEvidenceRequest::all(),
        ] {
            let report = recovered(input, "Subject", evidence);
            let body = run_body(&report);
            assert_eq!(body.quality, Quality::Structured, "{}", body.text);
            if underscore {
                assert!(body.text.contains("._("), "{}", body.text);
                if let Some(declaration) = report
                    .methods
                    .iter()
                    .find(|method| method.item.name.raw().0 == b"_")
                {
                    assert!(
                        declaration.declaration.as_deref().unwrap().contains(" _("),
                        "{}",
                        declaration.text
                    );
                }
            }
            if let Some(text) = previous {
                assert_eq!(report.text, text);
            }
            previous = Some(report.text);
        }
    }
}

fn refuses(inputs: &[&[u8]], raw: &[u8]) {
    for input in inputs {
        let mut previous = None;
        for evidence in [
            RecoveryEvidenceRequest::essential(),
            RecoveryEvidenceRequest::all(),
        ] {
            let report = recovered(input, "Subject", evidence);
            let body = run_body(&report);
            assert_eq!(body.quality, Quality::Fallback, "{}", body.text);
            assert!(body.text.contains("raw method name"), "{}", body.text);
            assert!(body.text.contains(&format!("{raw:?}")), "{}", body.text);
            if let Some(text) = previous {
                assert_eq!(report.text, text);
            }
            previous = Some(report.text);
        }
    }
}

#[test]
fn legal_mutf8_invocation_names_preserve_existing_calls() {
    controls(
        &[
            include_bytes!("fixtures/s1-invocation-names/static-void-ascii-Xcall.jar"),
            include_bytes!("fixtures/s1-invocation-names/static-void-bmp-cafe.jar"),
            include_bytes!("fixtures/s1-invocation-names/static-void-supplementary-U10400.jar"),
            include_bytes!("fixtures/s1-invocation-names/virtual-return-ascii-Xcall.jar"),
            include_bytes!("fixtures/s1-invocation-names/virtual-return-bmp-cafe.jar"),
            include_bytes!("fixtures/s1-invocation-names/virtual-return-supplementary-U10400.jar"),
            include_bytes!("fixtures/s1-invocation-names/private-special-return-ascii-Xcall.jar"),
            include_bytes!("fixtures/s1-invocation-names/private-special-return-bmp-cafe.jar"),
            include_bytes!(
                "fixtures/s1-invocation-names/private-special-return-supplementary-U10400.jar"
            ),
            include_bytes!("fixtures/s1-invocation-names/interface-return-ascii-Xcall.jar"),
            include_bytes!("fixtures/s1-invocation-names/interface-return-bmp-cafe.jar"),
            include_bytes!(
                "fixtures/s1-invocation-names/interface-return-supplementary-U10400.jar"
            ),
        ],
        false,
    );
}

#[test]
fn java8_underscore_declaration_and_calls_agree() {
    controls(
        &[
            include_bytes!("fixtures/s1-invocation-names/static-void-underscore.jar"),
            include_bytes!("fixtures/s1-invocation-names/virtual-return-underscore.jar"),
            include_bytes!("fixtures/s1-invocation-names/private-special-return-underscore.jar"),
            include_bytes!("fixtures/s1-invocation-names/interface-return-underscore.jar"),
        ],
        true,
    );
}

#[test]
fn source_inexpressible_invocation_names_keep_raw_fallback() {
    refuses(
        &[
            include_bytes!("fixtures/s1-invocation-names/static-void-dash.jar"),
            include_bytes!("fixtures/s1-invocation-names/virtual-return-dash.jar"),
            include_bytes!("fixtures/s1-invocation-names/private-special-return-dash.jar"),
            include_bytes!("fixtures/s1-invocation-names/interface-return-dash.jar"),
        ],
        b"q-q",
    );
}

#[test]
fn isolated_surrogate_invocation_names_are_not_diagnostic_source_tokens() {
    refuses(
        &[
            include_bytes!("fixtures/s1-invocation-names/static-void-surrogate.jar"),
            include_bytes!("fixtures/s1-invocation-names/virtual-return-surrogate.jar"),
            include_bytes!("fixtures/s1-invocation-names/private-special-return-surrogate.jar"),
            include_bytes!("fixtures/s1-invocation-names/interface-return-surrogate.jar"),
        ],
        &[0xed, 0xa0, 0x80],
    );
}
