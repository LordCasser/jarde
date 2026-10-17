//! Public request-level late AnalysisSteps stop: no target source/map is published partially.
//! Intended as a new tests/p3_switch_nonlocal_late_stop.rs integration test.
use jarde::*;
use jarde_reader::budget::{Budget, BudgetDimension};
use jarde_reader::model::{ExecutionReport, TerminationReason};
use std::slice;

const SN: &[u8] = include_bytes!("fixtures/p3-switch-nonlocal-exits/javac8/SN.class");
const CLASS: &str = "SN";
const METHOD: &str = "retInSwitchNoTail";
const DESCRIPTOR: &str = "([I)I";

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
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
        .expect("class-source stop is an outcome")
}

fn target_method(report: &ClassSourceReport) -> &ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == METHOD.as_bytes()
                && method.item.descriptor.raw().0 == DESCRIPTOR.as_bytes()
        })
        .expect("the exact target method is retained in the report")
}

fn target_is_complete(
    outcome: &OperationOutcome<ClassSourceReport>,
    identity: &PhysicalMethodId,
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
    &method.item.identity == identity
        && matches!(
            &method.outcome,
            ClassSourceOutcome::Recovered { report, .. }
                if matches!(&report.execution, ExecutionReport::Complete { .. })
        )
}

#[test]
fn late_request_budget_does_not_publish_a_partial_switch_return_method() {
    let snapshot = Engine::new()
        .open(
            ArtifactInput::bytes(SN.to_vec()),
            &mut task_budget(&[]).expect("bounded fixture-open budget"),
        )
        .expect("frozen SN class opens");
    let request = request(&snapshot);
    let complete = run_all(
        &snapshot,
        &request,
        &mut task_budget(&[]).expect("bounded complete-run budget"),
    );
    let OperationOutcome::Performed(complete) = complete else {
        panic!("the complete frozen SN class is performed: {complete:?}");
    };
    let target = target_method(&complete);
    let identity = target.item.identity.clone();
    let ClassSourceOutcome::Recovered { report: body, .. } = &target.outcome else {
        panic!(
            "target switch method has a recovery report: {:?}",
            target.outcome
        );
    };
    assert!(matches!(&body.execution, ExecutionReport::Complete { .. }));
    assert!(body.produced());
    assert!(!body.text.is_empty());
    assert!(!body.source_map.is_empty());

    // Search the request-wide ledger boundary. The target body's own AnalysisSteps count is local
    // to that report, so using it as the request limit could stop before this method is reached.
    let mut low = 0;
    let mut high = complete.usage.analysis_steps;
    assert!(high > 1, "the complete request records real work");
    while low + 1 < high {
        let middle = low + (high - low) / 2;
        let mut limits = task_budget(&[])
            .expect("bounded task defaults")
            .limits()
            .clone();
        limits.analysis_steps = middle;
        if target_is_complete(
            &run_all(&snapshot, &request, &mut Budget::new(limits)),
            &identity,
        ) {
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
    let report = match stopped {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Incomplete(report) => panic!(
            "request stopped before the exact switch method: {:?}",
            report.execution
        ),
        OperationOutcome::Ambiguous(candidates) => {
            panic!("the frozen standalone class is unambiguous: {candidates:?}")
        }
    };
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps
            },
            ..
        }
    ));
    let target = target_method(&report);
    assert_eq!(target.item.identity, identity);
    let ClassSourceOutcome::Recovered { report: body, .. } = &target.outcome else {
        panic!(
            "the exact target has its stopped body report: {:?}",
            target.outcome
        );
    };
    assert!(matches!(
        body.execution,
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
        other => panic!("the target stop records its local measured usage: {other:?}"),
    };
    assert!(usage.analysis_steps > 0);
    assert!(usage.analysis_steps <= late_limit);
}
