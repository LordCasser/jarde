//! Fresh Java 8 recovery of the transfer gateway's instruction origins.

use jarde::*;
use std::slice;

const CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/ChainExtraBoundary.class"
);

fn opened() -> (ArtifactSnapshot, ClassSourceRequest) {
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
    (snapshot, request)
}

fn source(evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    let (snapshot, request) = opened();
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let OperationOutcome::Performed(report) = Engine::new()
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, evidence, &mut budget)
        .expect("class source request succeeds")
    else {
        panic!("one standalone class must bind uniquely");
    };
    report
}

fn assign(evidence: &RecoveryEvidenceRequest) -> ClassSourceMethod {
    let report = source(evidence);
    report
        .methods
        .into_iter()
        .find(|method| method.item.name.raw().0 == b"assign")
        .expect("assign method exists")
}

#[test]
fn transfer_gateway_keeps_its_second_instruction_in_both_evidence_modes() {
    let essential = assign(&RecoveryEvidenceRequest::essential());
    let full = assign(&RecoveryEvidenceRequest::all());
    assert_eq!(
        essential.text, full.text,
        "evidence selection changed the body"
    );
    let ClassSourceOutcome::Recovered { report, .. } = &full.outcome else {
        panic!(
            "assign did not produce a recovery report: {:?}",
            full.outcome
        );
    };
    // BCI 25 contains the BCI 26 goto; both remain mapped after structured recovery.
    assert!(
        !report.source_map.of_bci(26).is_empty(),
        "the structured region lost its decoded goto at BCI 26"
    );
    assert!(!report.source_map.of_bci(8).is_empty());
    assert!(!report.text.contains("@bytecode"));
}

#[test]
fn exhausted_source_budget_cannot_publish_a_complete_recovery() {
    let complete = source(&RecoveryEvidenceRequest::essential());
    let (snapshot, request) = opened();
    let mut limits = complete.limits.clone();
    limits.analysis_steps = complete.usage.analysis_steps - 1;
    let mut budget = Budget::new(limits);
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential(),
            &mut budget,
        )
        .expect("bounded class-source request is answered");
    match outcome {
        OperationOutcome::Incomplete(candidates) => assert!(matches!(
            candidates.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        )),
        OperationOutcome::Performed(report) => assert!(matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        )),
        other => panic!("unexpected bounded result: {other:?}"),
    }
}
