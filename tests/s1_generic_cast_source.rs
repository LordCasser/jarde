//! Same-class overload binding keeps the generic source cast for supplementary identifiers.
use jarde::class_source::ClassSourceOutcome;
use jarde::*;

const ASCII: &[u8] = include_bytes!("fixtures/s1-generic-cast-source/ascii.jar");
const SUPPLEMENTARY: &[u8] = include_bytes!("fixtures/s1-generic-cast-source/supplementary.jar");

fn source_with_evidence(bytes: &[u8], evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    let engine = Engine::new();
    let mut open_budget = task_budget(&[]).expect("open limits are available");
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut open_budget)
        .expect("the frozen fixture jar opens");
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal("Subject"),
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
fn same_class_overload_cast_keeps_generic_source_types_across_evidence_modes() {
    for (bytes, item) in [(ASCII, "Item"), (SUPPLEMENTARY, "𐐀Item")] {
        let relay_descriptor = format!("(L{item};Ljava/lang/String;)V");
        let class_header = format!(
            "public class Subject<T extends {item}, E extends java.lang.Number> extends java.lang.Object"
        );
        let typed_cast = format!("this.pick(({item}) arg1, arg2)");
        let mut previous_text: Option<String> = None;

        for evidence in evidence_requests() {
            let report = source_with_evidence(bytes, &evidence);
            let declaration = report
                .declaration
                .as_ref()
                .expect("the class declaration is present");
            assert_eq!(
                declaration.declaration, class_header,
                "the class Signature retains T and E bounds:\n{}",
                report.text
            );

            let item_overload = method(&report, b"pick", &format!("(L{item};Ljava/lang/String;)V"));
            assert!(
                item_overload
                    .declaration
                    .as_deref()
                    .is_some_and(|line| line.contains(&format!("pick({item} arg1"))),
                "the Item overload remains sourceable:\n{}",
                report.text
            );

            let generic_overload =
                method(&report, b"pick", "(Ljava/lang/Number;Ljava/lang/Object;)V");
            assert!(
                generic_overload.declaration.as_deref().is_some_and(|line| {
                    line.contains("public void pick(E arg1, java.lang.Object arg2)")
                }),
                "the E overload keeps its generic header:\n{}",
                report.text
            );

            let relay = method(&report, b"relay", &relay_descriptor);
            assert!(
                relay.declaration.as_deref().is_some_and(
                    |line| line.contains("public void relay(T arg1, java.lang.String arg2)")
                ),
                "the physical relay descriptor still presents T:\n{}",
                report.text
            );
            assert!(
                matches!(&relay.outcome, ClassSourceOutcome::Recovered { .. }),
                "the relay body is recovered:\n{}",
                relay.text
            );
            assert!(
                relay.text.contains(&typed_cast),
                "the recovered body casts to the bound Item type {item}:\n{}",
                relay.text
            );

            if let Some(previous) = &previous_text {
                assert_eq!(
                    &report.text, previous,
                    "evidence selection does not change the class source for {item}"
                );
            }
            previous_text = Some(report.text.clone());
        }
    }
}
