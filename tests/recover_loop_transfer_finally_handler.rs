//! Nonthrowing loop-transfer catch-all ownership preserves complete physical provenance.
//! Complete-source compilation and unchanged-driver execution are separate gates.
use jarde::*;
use std::slice;

const LEGS: &[(&str, &[u8])] = &[
    (
        "javac23 --release8",
        include_bytes!(
            "fixtures/recover-loop-transfer-finally-handler/v8/LoopTransferFinally.class"
        ),
    ),
    (
        "javac8",
        include_bytes!(
            "fixtures/recover-loop-transfer-finally-handler/v8-javac8/LoopTransferFinally.class"
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
            class: ClassNameQuery::internal("LoopTransferFinally"),
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

fn body<'a>(
    source: &'a ClassSourceReport,
    name: &str,
) -> (&'a ClassSourceMethod, &'a RecoveryReport) {
    let method = source
        .methods
        .iter()
        .find(|m| m.item.name.raw().0 == name.as_bytes())
        .unwrap();
    assert_eq!(method.item.descriptor.raw().0, b"([I)I");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!(
            "the physical method must retain its report: {:?}",
            method.outcome
        )
    };
    assert_eq!(
        report.representation,
        Representation::Java,
        "{}",
        report.text
    );
    assert_eq!(report.quality, Quality::Structured, "{}", report.text);
    assert!(report.fallbacks.is_empty(), "{:?}", report.fallbacks);
    (method, report)
}

#[test]
fn loop_transfer_finally_keeps_distinct_control_and_complete_structured_methods() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        assert_eq!(source.methods.len(), 3, "all physical methods remain");
        for name in ["finContinue", "finBreak"] {
            let (_, body) = body(&source, name);
            assert!(
                !body.text.contains("@bytecode"),
                "{leg}/{name}: {}",
                body.text
            );
            assert!(body.text.contains("for ("), "{leg}/{name}: {}", body.text);
            assert_eq!(
                body.text.contains("break;"),
                name == "finBreak",
                "{leg}/{name}: {}",
                body.text
            );
            assert_eq!(
                body.text.matches(" + 100").count(),
                2,
                "both ordinary copies remain"
            );
            assert_eq!(body.text.matches("return ").count(), 1);
            assert_eq!(body.content, RecoveryContent::ContainsStatements);
            let handler = body.regions.iter().find(|region| region.bci == 45).unwrap();
            assert!(handler.structured);
            assert_eq!(handler.blocks, [45]);
            assert!(handler.code.is_none());
            assert_eq!(handler.rule.unwrap().rule(), "finally");
        }
    }
}

#[test]
fn loop_transfer_finally_accounts_for_all_35_physical_origins_in_the_real_method() {
    for (leg, bytes) in LEGS {
        let source = source(bytes);
        for name in ["finContinue", "finBreak"] {
            let (method, body) = body(&source, name);
            for bci in [
                0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 13, 16, 17, 19, 20, 22, 24, 25, 28, 31, 34, 35,
                37, 38, 39, 42, 45, 47, 50, 52, 53, 56, 59, 60,
            ] {
                assert!(
                    !body.source_map.direct_of_bci(bci).is_empty()
                        || !body.source_map.derived_of_bci(bci).is_empty(),
                    "{leg}/{name}: missing {bci}"
                );
            }
            for bci in [42, 45, 47, 50, 52, 56] {
                assert!(
                    body.source_map
                        .derived_of_bci(bci)
                        .iter()
                        .any(|segment| segment.origin().primary().bci() == 13),
                    "{leg}/{name}: physical {bci} must be derived from the real for"
                );
            }
            assert!(
                body.source_map
                    .segments()
                    .iter()
                    .flat_map(|segment| std::iter::once(segment.origin().primary())
                        .chain(segment.origin().derived()))
                    .all(|origin| origin.method() == Some(&method.item.identity))
            );
        }
    }
}
