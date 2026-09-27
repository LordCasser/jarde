//! CF-10's stride-two indexed loop remains a counted loop and keeps the throwing length read in
//! its condition. The checked-in class and source are frozen in the replay evidence directory.

use jarde::*;
use jarde_jvm::ir::Representation;
use jarde_reader::budget::{Budget, CancellationToken};
use jarde_reader::model::ExecutionReport;
use std::slice;

const STEP_INDEX: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-27/cf10-foreach/isolation/step-index/original/StepIndex.class"
);

fn source(evidence: RecoveryEvidenceRequest) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded task defaults");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(STEP_INDEX.to_vec()), &mut budget)
        .expect("the frozen Java 8 class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("StepIndex"),
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
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, &evidence, &mut budget)
        .expect("the class-source request answers")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one complete class-source report is expected: {other:?}"),
    }
}

fn every_other(report: &ClassSourceReport) -> (&ClassSourceMethod, &jarde_java::RecoveryReport) {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"everyOther")
        .expect("everyOther member");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!(
            "everyOther must have a recovered body: {:?}",
            method.outcome
        );
    };
    (method, report)
}

fn request_for(snapshot: SnapshotId) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("StepIndex"),
        },
        environment: EnvironmentRequest {
            snapshot,
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    }
}

#[test]
fn stride_two_loop_keeps_array_length_index_update_return_and_origins() {
    let report = source(RecoveryEvidenceRequest::all());
    let (method, body) = every_other(&report);
    assert!(body.produced(), "{:?}", body.outcome);
    assert_eq!(body.representation, Representation::Java);
    assert!(
        body.fallbacks.is_empty(),
        "{:?}\n{}",
        body.fallbacks,
        body.text
    );
    assert!(body.text.contains("values.length"), "{}", body.text);
    assert!(
        body.text.contains("+ 2") || body.text.contains("+= 2"),
        "the original stride remains: {}",
        body.text
    );
    assert!(body.text.contains("return"), "{}", body.text);
    assert!(
        !body.text.contains(" : "),
        "the loop stays indexed:\n{}",
        body.text
    );
    for bci in [6, 7, 16, 22] {
        assert!(
            !body.source_map.text_of_bci(&body.text, bci).is_empty(),
            "BCI {bci} remains mapped in {}",
            body.text
        );
    }
    assert!(method.text.contains("values.length"), "{}", method.text);
    assert!(
        !method
            .text
            .contains("local 1 crosses a quoted fallback region")
    );

    let essential = source(RecoveryEvidenceRequest::essential());
    assert_eq!(
        report.text, essential.text,
        "evidence choice cannot change the source"
    );
}

#[test]
fn budget_and_cancellation_do_not_publish_a_partial_array_length_loop() {
    let mut open_budget = task_budget(&[]).expect("bounded task defaults");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(STEP_INDEX.to_vec()), &mut open_budget)
        .expect("the frozen Java 8 class opens");
    let request = request_for(snapshot.id().clone());

    let mut limits = task_budget(&[])
        .expect("bounded task defaults")
        .limits()
        .clone();
    limits.analysis_steps = 1;
    let mut low_budget = Budget::new(limits);
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut low_budget,
        )
        .expect("a budget stop is reported as an outcome");
    match stopped {
        OperationOutcome::Incomplete(report) => assert!(matches!(
            report.execution,
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

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(
        task_budget(&[])
            .expect("bounded task defaults")
            .limits()
            .clone(),
        token,
    );
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation is reported as an outcome");
    match stopped {
        OperationOutcome::Incomplete(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
        }
        OperationOutcome::Performed(report) => {
            assert!(
                report.text.is_empty(),
                "no source is published after cancellation"
            );
        }
        other => panic!("unexpected cancelled result: {other:?}"),
    }
}
