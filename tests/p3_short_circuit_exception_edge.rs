//! A short-circuit field write protected by a real exception-table edge stays quoted as a whole.

use jarde::*;
use std::slice;

const CLASS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/short-circuit-exception-edge/ExceptionShortCircuit.class"
);

fn recover() -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ExceptionShortCircuit"),
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
fn throwing_short_circuit_field_write_is_refused_with_complete_evidence() {
    let report = recover();
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
                && [0, 4, 10, 14, 15]
                    .iter()
                    .all(|bci| region.blocks.contains(bci))
        }),
        "the protected short-circuit field-write candidate must be refused as one region; got {:?}",
        recovered.regions
    );

    for bci in [0, 1, 4, 7, 10, 11, 14, 15] {
        assert!(
            !recovered.source_map.of_bci(bci).is_empty(),
            "refused candidate lost BCI {bci} from the complete source map"
        );
    }
    assert!(
        !recovered.text.contains("ExceptionShortCircuit.result ="),
        "refused candidate must not emit a structured field assignment:\n{}",
        recovered.text
    );
}
