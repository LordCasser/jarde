//! A conditional return and fallthrough share the same physical cleanup copies.
//! Complete-source compilation and unchanged-driver execution are separate gates.
use jarde::*;
use std::slice;

const LEGS: &[(&str, &[u8])] = &[
    (
        "javac23 --release8",
        include_bytes!(
            "fixtures/recover-conditional-return-finally-copies/v8/LooplessFinally.class"
        ),
    ),
    (
        "javac8",
        include_bytes!(
            "fixtures/recover-conditional-return-finally-copies/v8-javac8/LooplessFinally.class"
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
            class: ClassNameQuery::internal("LooplessFinally"),
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
        .find(|m| m.item.name.raw().0 == b"loopless")
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
fn conditional_return_and_fallthrough_keep_one_structured_finally() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let (_, body) = body(&source);
        assert!(!body.text.contains("@bytecode"), "{leg}: {}", body.text);
        assert!(body.text.contains("try {"), "{leg}: {}", body.text);
        assert!(body.text.contains("finally {"), "{leg}: {}", body.text);
        assert_eq!(
            body.text.matches("println(\"f\")").count(),
            1,
            "{leg}: {}",
            body.text
        );
        assert!(body.text.contains("return -1;"), "{leg}: {}", body.text);
        let returned = body
            .text
            .find("return ")
            .expect("the conditional return is retained");
        let cleanup = body.text.find("finally {").unwrap();
        assert!(returned < cleanup, "{leg}: {}", body.text);
        assert_eq!(body.content, RecoveryContent::ContainsStatements);
        assert_eq!(source.methods.len(), 2, "all physical methods remain");
    }
}

#[test]
fn conditional_finally_keeps_all_three_copy_and_value_origins() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let (method, body) = body(&source);
        assert!(
            body.fields
                .iter()
                .any(|f| f.bci == 12 && f.access == "read" && f.presented),
            "{leg}: {}",
            body.text
        );
        for bci in [
            0, 1, 2, 5, 6, 7, 8, 9, 12, 15, 16, 19, 21, 24, 25, 26, 29, 31, 34, 37, 38, 41, 43, 46,
            47, 48, 49,
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
