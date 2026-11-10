//! A selected empty-void generic policy must refuse an unproved bound without panic.
use jarde::class_source::ClassSourceOutcome;
use jarde::*;

const ORIGINAL: &[u8] = include_bytes!("fixtures/s1-generic-declaration-policies/original.class");
// Only Signature CP #11 changes; the Code attributes remain identical.
const INTERFACE_BOUND: &[u8] =
    include_bytes!("fixtures/s1-generic-declaration-policies/mutant.class");

fn recover(bytes: &[u8], evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal("methodbodythrows/MethodBodyThrows"),
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
    match engine
        .class_source_with_evidence(&[snapshot], &request, evidence, &mut budget)
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("class recovery must perform: {other:?}"),
    }
}

#[test]
fn interface_only_throwable_bound_refuses_locally_and_preserves_the_method() {
    for (bytes, generic) in [(ORIGINAL, true), (INTERFACE_BOUND, false)] {
        let mut previous = None;
        for evidence in [
            RecoveryEvidenceRequest::essential(),
            RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            RecoveryEvidenceRequest::all(),
        ] {
            let report = recover(bytes, &evidence);
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"run")
                .expect("physical method survives source projection refusal");
            assert_eq!(method.item.descriptor.raw(), &JvmBytes::from_unicode("()V"));
            assert!(matches!(
                method.outcome,
                ClassSourceOutcome::Recovered { .. }
            ));
            if generic {
                assert_eq!(
                    method.declaration.as_deref(),
                    Some("public <X extends java.lang.Exception> void run() throws X")
                );
            } else {
                assert_eq!(
                    method.declaration.as_deref(),
                    Some("public void run() throws java.lang.Exception")
                );
                assert!(method.markers.iter().any(|marker| marker.contains("method variable must have exactly one proved class bound and no additional bounds")), "{}", report.text);
            }
            if let Some(text) = previous {
                assert_eq!(report.text, text);
            }
            previous = Some(report.text);
        }
    }
}
