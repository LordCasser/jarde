//! Public proof-to-builder stop boundary for the throw-only finally fixture.
use jarde::*;
use jarde_reader::budget::{Budget, BudgetDimension, CancellationToken, UsageSnapshot};
use jarde_reader::model::{ExecutionReport, TerminationReason};
use std::slice;

const FIXTURE: &[u8] = include_bytes!(
    "fixtures/recover-exception-only-finally-completion/v8/ExceptionOnlyFinally.class"
);
const CLASS: &str = "ExceptionOnlyFinally";
const METHOD: &str = "tryThrowsFinallyRuns";
const DESCRIPTOR: &str = "()I";

fn open_fixture() -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(FIXTURE.to_vec()),
            &mut task_budget(&[]).expect("bounded fixture-open budget"),
        )
        .expect("the frozen independent class opens")
}

fn exception_only_stop_request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(CLASS),
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
    }
}

fn run_all(
    snapshot: &ArtifactSnapshot,
    request: &ClassSourceRequest,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            request,
            &RecoveryEvidenceRequest::all(),
            budget,
        )
        .expect("class-source stop is reported as an ordinary outcome")
}

fn exact_method(report: &ClassSourceReport) -> &ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == METHOD.as_bytes()
                && method.item.descriptor.raw().0 == DESCRIPTOR.as_bytes()
        })
        .expect("the exact target method is retained in the report")
}

fn target_body_is_complete(
    outcome: &OperationOutcome<ClassSourceReport>,
    expected_identity: &PhysicalMethodId,
) -> bool {
    let OperationOutcome::Performed(report) = outcome else {
        return false;
    };
    let Some(method) = report.methods.iter().find(|method| {
        method.item.name.raw().0 == METHOD.as_bytes()
            && method.item.descriptor.raw().0 == DESCRIPTOR.as_bytes()
    }) else {
        return false;
    };
    if &method.item.identity != expected_identity {
        return false;
    }
    matches!(
        &method.outcome,
        ClassSourceOutcome::Recovered { report, .. }
            if matches!(&report.execution, ExecutionReport::Complete { .. })
    )
}

fn assert_target_body_stopped(
    outcome: &OperationOutcome<ClassSourceReport>,
    expected_identity: &PhysicalMethodId,
    analysis_step_limit: u64,
) {
    let report = match outcome {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Incomplete(report) => panic!(
            "class-source stopped before the exact method: {:?}",
            report.execution
        ),
        OperationOutcome::Ambiguous(candidates) => {
            panic!("standalone class selection is unambiguous: {candidates:?}")
        }
    };
    assert!(matches!(
        &report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps
            },
            ..
        }
    ));
    let method = exact_method(report);
    assert_eq!(&method.item.identity, expected_identity);
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!(
            "the exact target has a body recovery report: {:?}",
            method.outcome
        );
    };
    assert!(matches!(
        &body.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps
            },
            ..
        }
    ));
    assert!(!body.produced());
    assert!(body.text.is_empty());
    assert!(body.source_map.is_empty());
    assert_eq!(body.content, RecoveryContent::NotProduced);
    assert!(body.news.is_empty());
    let usage = match &body.execution {
        ExecutionReport::Partial { usage, .. } => usage,
        other => panic!("the method stop carries its own measured usage: {other:?}"),
    };
    assert!(usage.analysis_steps > 0);
    assert!(usage.analysis_steps <= analysis_step_limit);
}

#[test]
fn public_late_analysis_stop_does_not_publish_a_partial_finally_body_or_map() {
    let snapshot = open_fixture();
    let request = exception_only_stop_request(&snapshot);
    let complete = run_all(
        &snapshot,
        &request,
        &mut task_budget(&[]).expect("bounded complete-run budget"),
    );
    let complete = match complete {
        OperationOutcome::Performed(report) => report,
        other => panic!("the complete independent fixture is performed: {other:?}"),
    };
    let complete_method = exact_method(&complete);
    let expected_identity = complete_method.item.identity.clone();
    let ClassSourceOutcome::Recovered {
        report: complete_body,
        ..
    } = &complete_method.outcome
    else {
        panic!(
            "the complete target has its body report: {:?}",
            complete_method.outcome
        );
    };
    let ExecutionReport::Complete { usage } = &complete_body.execution else {
        panic!(
            "the complete target supplies its actual analysis charge: {:?}",
            complete_body.execution
        );
    };
    let measured_target_steps = usage.analysis_steps;
    assert!(
        measured_target_steps > 0,
        "the target body records real work"
    );

    // Search the full class-source request's charge boundary. A method body's usage is local to
    // that report and is not the request's cumulative AnalysisSteps counter; using it directly can
    // stop before this method or after it has completed.
    let mut low = 0;
    let mut high = complete.usage.analysis_steps;
    assert!(
        high > 1,
        "the complete class-source request has charged work"
    );
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        let mut limits = task_budget(&[])
            .expect("bounded task defaults")
            .limits()
            .clone();
        limits.analysis_steps = middle;
        let candidate = run_all(&snapshot, &request, &mut Budget::new(limits));
        if target_body_is_complete(&candidate, &expected_identity) {
            high = middle;
        } else {
            low = middle;
        }
    }
    let late_limit = high - 1;
    let mut limits = task_budget(&[])
        .expect("bounded task defaults")
        .limits()
        .clone();
    limits.analysis_steps = late_limit;
    let stopped = run_all(&snapshot, &request, &mut Budget::new(limits));
    assert_target_body_stopped(&stopped, &expected_identity, late_limit);
}

#[test]
fn public_precancel_reports_no_class_text_or_target_method() {
    let snapshot = open_fixture();
    let request = exception_only_stop_request(&snapshot);
    let token = CancellationToken::new();
    token.cancel();
    let limits = task_budget(&[])
        .expect("bounded task defaults")
        .limits()
        .clone();
    let outcome = run_all(
        &snapshot,
        &request,
        &mut Budget::with_cancellation_token(limits, token),
    );
    let execution = match &outcome {
        OperationOutcome::Incomplete(report) => &report.execution,
        OperationOutcome::Performed(report) => &report.execution,
        OperationOutcome::Ambiguous(candidates) => {
            panic!("standalone class selection is unambiguous: {candidates:?}")
        }
    };
    let usage = match execution {
        ExecutionReport::Cancelled { usage } => usage,
        other => panic!("pre-cancellation is reported as Cancelled: {other:?}"),
    };
    let mut counted = usage.clone();
    counted.elapsed_millis = 0;
    assert_eq!(counted, UsageSnapshot::default());
    match outcome {
        OperationOutcome::Incomplete(report) => assert!(report.candidates.is_empty()),
        OperationOutcome::Performed(report) => {
            assert!(report.declaration.is_none());
            assert!(report.fields.is_empty());
            assert!(report.methods.is_empty());
            assert!(report.text.is_empty());
        }
        OperationOutcome::Ambiguous(_) => unreachable!(),
    }
}
