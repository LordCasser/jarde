//! Field reports describe Java operations in the committed body, including conservative refusals.

use jarde::*;
use std::slice;

const STRUCTURED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/ChainExtraBoundary.class"
);
const EXTRA_ENTRY: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/short-circuit-transfer-gateway-controls/ChainExtraBoundary.class"
);
const EXCEPTION_EDGE: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/short-circuit-transfer-gateway-controls/ChainExtraBoundaryException.class"
);

fn assign(class: &[u8], evidence: &RecoveryEvidenceRequest) -> RecoveryReport {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
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
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, evidence, &mut budget)
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
    *report.clone()
}

fn summary(report: &RecoveryReport) -> &str {
    &report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "jre_field_accesses")
        .expect("field summary exists")
        .message
}

#[test]
fn field_write_tracks_the_final_java_assignment_across_gateway_shapes() {
    let positive = assign(STRUCTURED, &RecoveryEvidenceRequest::all());
    let write = positive
        .fields
        .iter()
        .find(|field| field.bci == 30)
        .expect("BCI 30 field record");
    assert!(
        write.presented,
        "original class emits the field write: {}",
        positive.text
    );
    assert!(positive.text.contains("ChainExtraBoundary.result ="));
    assert!(summary(&positive).contains("1 presented, 0 refused"));

    for class in [EXTRA_ENTRY, EXCEPTION_EDGE] {
        let full = assign(class, &RecoveryEvidenceRequest::all());
        let write = full
            .fields
            .iter()
            .find(|field| field.bci == 30)
            .expect("BCI 30 field record");
        assert!(
            !write.presented,
            "quoted BCI 30 is no assignment: {}",
            full.text
        );
        assert_eq!(
            write.refusal.as_ref().map(|refusal| refusal.code),
            Some("jre_field_not_emitted")
        );
        assert!(
            !full.text.contains("ChainExtraBoundary.result ="),
            "{}",
            full.text
        );
        assert!(
            !full.source_map.of_bci(30).is_empty(),
            "the rejected write remains traceable"
        );
        assert!(
            summary(&full).contains("0 presented, 1 refused"),
            "{}",
            summary(&full)
        );

        let essential = assign(class, &RecoveryEvidenceRequest::essential());
        assert_eq!(summary(&essential), summary(&full));
        assert!(essential.fields.is_empty());
        assert_eq!(essential.text, full.text);

        let ranged = assign(
            class,
            &RecoveryEvidenceRequest::all().with_driver_bci_range(BytecodeRange::new(30, 33)),
        );
        assert_eq!(summary(&ranged), summary(&full));
        assert_eq!(
            ranged
                .fields
                .iter()
                .map(|field| field.bci)
                .collect::<Vec<_>>(),
            [30]
        );
    }
}
