//! A loop-local cleanup keeps the pending return and each iteration.
//! Complete-source compilation and unchanged-driver execution are separate gates.
use jarde::*;
use std::slice;

const LEGS: &[(&str, &[u8])] = &[
    (
        "javac23 --release8",
        include_bytes!("fixtures/recover-local-increment-loop-finally/v8/NoCallFinally.class"),
    ),
    (
        "javac8",
        include_bytes!(
            "fixtures/recover-local-increment-loop-finally/v8-javac8/NoCallFinally.class"
        ),
    ),
];

fn source(bytes: &[u8]) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("NoCallFinally"),
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
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen physical class must be selected: {other:?}"),
    }
}

fn body(source: &ClassSourceReport) -> (&ClassSourceMethod, &RecoveryReport) {
    let method = source
        .methods
        .iter()
        .find(|m| m.item.name.raw().0 == b"noCall")
        .unwrap();
    assert_eq!(method.item.descriptor.raw().0, b"([I)I");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!(
            "the physical method must retain its report: {:?}",
            method.outcome
        )
    };
    (method, report)
}

#[test]
fn loop_local_finally_keeps_the_saved_return_and_loop_continuation() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let (_, body) = body(&source);
        assert!(!body.text.contains("@bytecode"), "{leg}: {}", body.text);
        assert_eq!(
            body.text.matches("finally {").count(),
            1,
            "{leg}: {}",
            body.text
        );
        assert!(body.text.contains("try {"), "{leg}: {}", body.text);
        assert!(
            body.text.contains("while (") || body.text.contains("for ("),
            "{leg}: {}",
            body.text
        );
        let returned = body.text.find("return ").unwrap();
        let cleanup = body.text.find("finally {").unwrap();
        assert!(returned < cleanup, "{leg}: {}", body.text);
        assert_eq!(
            body.text.matches("return ").count(),
            2,
            "{leg}: {}",
            body.text
        );
        assert_eq!(body.content, RecoveryContent::ContainsStatements);
        assert_eq!(source.methods.len(), 2, "all physical methods remain");
    }
}

#[test]
fn loop_local_finally_keeps_all_physical_copy_and_loop_origins() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let (method, body) = body(&source);
        for bci in [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 13, 16, 17, 19, 20, 22, 24, 25, 28, 29, 31, 32, 34,
            37, 39, 40, 43, 46, 48, 51, 53, 54, 57, 60, 61,
        ] {
            assert!(
                !body.source_map.direct_of_bci(bci).is_empty()
                    || !body.source_map.derived_of_bci(bci).is_empty(),
                "{leg}: missing physical {bci}: {:?}",
                body.source_map.segments()
            );
        }
        assert!(
            body.source_map
                .segments()
                .iter()
                .flat_map(|s| std::iter::once(s.origin().primary()).chain(s.origin().derived()))
                .all(|origin| origin.method() == Some(&method.item.identity))
        );
    }
}
