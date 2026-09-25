//! A real exception edge inside a three-test shared-true OR chain stays quoted as a whole.

use jarde::*;
use std::slice;

const CLASS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/short-circuit-exception-chain/ExceptionChain.class"
);

fn fresh_recovery() -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ExceptionChain"),
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
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("class-source request succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected outcome: {other:?}"),
    }
}

#[test]
fn throwing_three_test_or_field_write_is_refused_with_complete_evidence() {
    // A fresh snapshot, budget, and recovery report keep this check independent from
    // the two-test exception fixture and the successful three-test shared-true case.
    let report = fresh_recovery();
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"assign")
        .expect("assign member");
    let ClassSourceOutcome::Recovered {
        report: recovered, ..
    } = &method.outcome
    else {
        panic!("assign did not recover: {:?}", method.outcome);
    };

    assert!(
        recovered.regions.iter().any(|region| {
            !region.structured
                && region.code == Some("jre_region_exception_edge")
                && [0, 4, 8, 14, 18, 19, 25]
                    .iter()
                    .all(|bci| region.blocks.contains(bci))
        }),
        "the protected three-test field-write candidate must be refused as one region; got {:?}",
        recovered.regions
    );
    assert_eq!(recovered.quality, jarde_jvm::ir::Quality::Fallback);
    assert!(
        recovered
            .text
            .contains("// @bytecode 0 1 4 5 8 11 14 15 18 19 22"),
        "the protected chain, both producers, and its field write must be quoted together:\n{}",
        recovered.text
    );

    for bci in [
        0, 1, 4, 5, 8, 11, 14, 15, 18, 19, 22, 25, 26, 29, 30, 31, 34,
    ] {
        assert!(
            !recovered.source_map.of_bci(bci).is_empty(),
            "refused candidate lost BCI {bci} from the complete source map"
        );
    }
    assert!(
        !recovered.text.contains("ExceptionChain.result ="),
        "refused candidate must not emit a structured field assignment:\n{}",
        recovered.text
    );
    assert!(
        recovered.text.contains("catch"),
        "the catch path must remain visible in fallback text:\n{}",
        recovered.text
    );
}
