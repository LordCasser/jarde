//! Body type references preserve the raw InnerClasses boundaries supplied by the selected class.
use jarde::*;

fn recovered(bytes: &[u8], evidence: RecoveryEvidenceRequest) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal("Client"),
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

fn both(bytes: &[u8], expected: &str, invented: &str) {
    for evidence in [
        RecoveryEvidenceRequest::essential(),
        RecoveryEvidenceRequest::all(),
    ] {
        let report = recovered(bytes, evidence);
        for fragment in [
            format!("return {expected}.class;"),
            format!("({expected})"),
            format!("instanceof {expected}"),
            format!("new {expected}[1]"),
        ] {
            assert!(
                report.text.contains(&fragment),
                "missing {fragment}: {}",
                report.text
            );
        }
        assert!(!report.text.contains(invented), "{}", report.text);
    }
}

#[test]
fn member_dollar_is_preserved_in_every_body_type_position() {
    both(
        include_bytes!("fixtures/s1-body-type-names/member-dollar.jar"),
        "Outer.Node$Part",
        "Outer.Node.Part",
    );
}

#[test]
fn top_level_owner_dollar_is_not_an_extra_nesting_boundary() {
    both(
        include_bytes!("fixtures/s1-body-type-names/owner-dollar.jar"),
        "Family$Top.Node$Part",
        "Family.Top",
    );
}
