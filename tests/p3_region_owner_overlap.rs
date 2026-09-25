//! A shared producer reached from outside a short-circuit chain has one refusal owner.

use jarde::*;
use std::slice;

const CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/ChainExtraBoundary.class"
);

#[test]
fn transfer_gateway_is_owned_by_one_structured_field_chain() {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("frozen Java 8 class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ChainExtraBoundary"),
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
    let OperationOutcome::Performed(source) = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("class source request succeeds")
    else {
        panic!("one standalone class must bind uniquely");
    };
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"assign")
        .expect("assign method exists");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("assign did not recover: {:?}", method.outcome);
    };

    assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured);
    assert_eq!(report.representation, Representation::Java);
    assert!(report.fallbacks.is_empty(), "{}", report.text);
    assert_eq!(
        report.text.matches("ChainExtraBoundary.result =").count(),
        1,
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    for bci in [0, 1, 4, 5, 8, 11, 12, 15, 16, 19, 22, 25, 26, 29, 30, 33] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "decoded BCI {bci} is absent from the structured source map: {}",
            report.text
        );
    }
}
