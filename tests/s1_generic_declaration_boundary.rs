//! Generic void declaration policy consumes raw sibling facts across evidence modes.
use jarde::class_source::ClassSourceOutcome;
use jarde::*;

const ASCII: &[u8] = include_bytes!("fixtures/s1-generic-declaration-boundary/ascii.jar");
const SUPPLEMENTARY: &[u8] =
    include_bytes!("fixtures/s1-generic-declaration-boundary/supplementary.jar");

fn source_with_evidence(bytes: &[u8], evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).expect("open limits are available");
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut open_budget)
        .expect("the frozen fixture jar opens");
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal("Outer$Owner"),
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
    };
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

fn method<'a>(
    report: &'a ClassSourceReport,
    name: &[u8],
    descriptor: &str,
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name
                && method.item.descriptor.raw() == &JvmBytes::from_unicode(descriptor)
        })
        .unwrap_or_else(|| {
            panic!(
                "missing physical method {:?}{descriptor}:\n{}",
                name, report.text
            )
        })
}

#[test]
fn nested_void_bound_retains_generic_declaration_and_physical_identity() {
    for (bytes, bound) in [(ASCII, "Bound"), (SUPPLEMENTARY, "𐐀Bound")] {
        let mut previous = None;
        for evidence in evidence_requests() {
            let report = source_with_evidence(bytes, &evidence);
            let relay = method(&report, b"relay", &format!("(LOuter${bound};Z)V"));
            assert_eq!(
                relay.declaration.as_deref(),
                Some(
                    format!("public <T extends Outer${bound}> void relay(T arg1, boolean arg2)")
                        .as_str()
                ),
                "{}",
                report.text
            );
            assert!(matches!(
                &relay.outcome,
                ClassSourceOutcome::Recovered { .. }
            ));
            assert!(
                relay
                    .text
                    .contains("java.lang.System.out.println((java.lang.Object) arg1)"),
                "{}",
                relay.text
            );
            assert!(
                !relay
                    .markers
                    .iter()
                    .any(|marker| marker.contains("projection refused")),
                "{}",
                relay.text
            );
            if let Some(text) = previous {
                assert_eq!(report.text, text);
            }
            previous = Some(report.text);
        }
    }
}
