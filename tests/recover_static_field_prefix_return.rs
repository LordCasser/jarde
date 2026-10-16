//! The returned sum of one static int update, over both frozen javac outputs.
//! Whole-class compilation and dual-JVM replay are separate acceptance gates.

use jarde::*;
use std::slice;

const LEGS: &[(&str, &[u8])] = &[
    (
        "javac23 --release8",
        include_bytes!("fixtures/recover-static-field-prefix-return/v8/FO.class"),
    ),
    (
        "javac8",
        include_bytes!("fixtures/recover-static-field-prefix-return/v8-javac8/FO.class"),
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
            class: ClassNameQuery::internal("FO"),
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

fn body<'a>(source: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap();
    assert_eq!(method.item.descriptor.raw().0, b"()I");
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the physical method must have its recovery report: {other:?}"),
    }
}

#[test]
fn static_prefix_returns_the_new_value_on_both_compiler_legs() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let bump = body(&source, "bump");
        assert!(bump.text.contains("return ++FO.i;"), "{leg}: {}", bump.text);
        assert!(!bump.text.contains("@bytecode"), "{leg}: {}", bump.text);
        assert!(!bump.text.contains("FO.i ="), "{leg}: {}", bump.text);
        assert!(!bump.text.contains("FO.i++"), "{leg}: {}", bump.text);
        assert_eq!(bump.content, RecoveryContent::ContainsStatements);
    }
}

#[test]
fn static_prefix_keeps_read_write_and_return_origins() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let bump = body(&source, "bump");
        assert_eq!(
            bump.fields
                .iter()
                .map(|field| (field.bci, field.access, field.presented))
                .collect::<Vec<_>>(),
            [(0, "read", true), (6, "write", true)],
            "{leg}: {}",
            bump.text,
        );
        let read = bump.source_map.direct_of_bci(0);
        assert_eq!(read.len(), 1, "{leg}: {:?}", bump.source_map.segments());
        assert_eq!(read[0].text(&bump.text), "FO.i");
        let write = bump.source_map.direct_of_bci(6);
        assert_eq!(write.len(), 1, "{leg}: {:?}", bump.source_map.segments());
        assert_eq!(write[0].text(&bump.text), "++FO.i");
        for bci in [3, 4, 5] {
            assert!(
                !bump.source_map.derived_of_bci(bci).is_empty(),
                "{leg}: folded producer {bci}: {:?}",
                bump.source_map.segments(),
            );
        }
        assert!(!bump.source_map.direct_of_bci(9).is_empty());
    }
}

#[test]
fn saved_return_still_precedes_the_finally_write() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        let saved = body(&source, "evalThenFin");
        assert!(
            saved.text.contains(
                "try {\n        int local0 = bump();\n        return local0;\n    } finally {\n        FO.i = 100;\n    }"
            ),
            "{leg}: {}",
            saved.text,
        );
        assert!(!saved.text.contains("@bytecode"), "{leg}: {}", saved.text);
    }
}
