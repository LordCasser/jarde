//! P3: a nested loop's single exit path can break its containing loop.

use jarde::*;
use jarde_jvm::ir::{Quality, Representation};
use jarde_reader::budget::{Budget, CancellationToken, UsageSnapshot};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::ExecutionReport;
use std::slice;

const UPSTREAM: &[u8] = include_bytes!("fixtures/java8/cf11/UpstreamNestedLoops4.class");
const CONTROLS: &[u8] = include_bytes!("fixtures/java8/cf11/NestedLoopSideExitControls.class");
const UPSTREAM_NESTED2: &[u8] =
    include_bytes!("fixtures/java8/cf11-for-condition/UpstreamNestedLoops2.class");
const CONDITION_EFFECTS: &[u8] =
    include_bytes!("fixtures/java8/cf11-for-condition/ForConditionEffects.class");

fn request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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

fn class_source(bytes: &[u8], class: &str, all: bool) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("bounded task defaults");
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen class opens");
    let evidence = if all {
        RecoveryEvidenceRequest::all()
    } else {
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap)
    };
    match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot, class),
            &evidence,
            &mut budget,
        )
        .expect("single-class source request answers")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one complete class-source report is expected: {other:?}"),
    }
}

fn method<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name.as_bytes()
                && method.item.descriptor.raw().0 == descriptor.as_bytes()
        })
        .unwrap_or_else(|| panic!("missing physical method {name}{descriptor}"))
}

fn recovered(method: &ClassSourceMethod) -> &RecoveryReport {
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("method has a recovery report: {other:?}"),
    }
}

fn assert_structured(method: &ClassSourceMethod) -> &RecoveryReport {
    let report = recovered(method);
    assert!(report.produced(), "method body: {:?}", report.outcome);
    assert_eq!(report.representation, Representation::Java);
    assert_eq!(report.quality, Quality::Structured);
    assert!(matches!(
        &report.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(report.content, RecoveryContent::ContainsStatements);
    assert!(report.fallbacks.is_empty(), "{}", report.text);
    assert!(!method.text.contains("@bytecode"), "{}", method.text);
    report
}

fn assert_physical_origins(
    class_bytes: &[u8],
    name: &str,
    descriptor: &str,
    member: &ClassSourceMethod,
    body: &RecoveryReport,
) {
    let mut budget = task_budget(&[]).expect("bounded reader defaults");
    let inspection = inspect_method_bytecode(
        class_bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut budget,
    )
    .expect("complete physical bytecode is readable");
    assert!(inspection.stopped_at.is_none());

    for segment in body.source_map.segments() {
        assert!(segment.start() < segment.end());
        assert!(body.text.is_char_boundary(segment.start()));
        assert!(body.text.is_char_boundary(segment.end()));
        assert!(body.text.get(segment.start()..segment.end()).is_some());
        for origin in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            if origin.method().is_some() {
                assert_eq!(
                    origin.method(),
                    Some(&member.item.identity),
                    "every physical source names the exact class/method identity"
                );
            }
        }
    }

    for instruction in inspection.instructions {
        let segments = body.source_map.of_bci(instruction.bci);
        assert!(
            segments.iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| {
                        origin.bci() == instruction.bci
                            && origin.method() == Some(&member.item.identity)
                    })
            }),
            "BCI {} lacks an exact physical owner: {}",
            instruction.bci,
            body.text
        );
    }
}

fn assert_same_body_and_map(default: &ClassSourceMethod, all: &ClassSourceMethod) {
    let default = recovered(default);
    let all = recovered(all);
    assert_eq!(default.text, all.text);
    assert_eq!(default.source_map, all.source_map);
}

#[test]
fn upstream_nested_loops4_is_structured_with_complete_physical_sources() {
    let default = class_source(UPSTREAM, "cf11/UpstreamNestedLoops4", false);
    let all = class_source(UPSTREAM, "cf11/UpstreamNestedLoops4", true);
    assert_eq!(
        default.text, all.text,
        "class source is evidence-independent"
    );

    let default_method = method(&default, "testFor", "()I");
    let all_method = method(&all, "testFor", "()I");
    assert_same_body_and_map(default_method, all_method);
    let body = assert_structured(all_method);
    assert!(all_method.text.contains("break;"), "{}", all_method.text);
    assert!(all_method.text.contains("return 0;"), "{}", all_method.text);
    assert!(all_method.text.contains("return"), "{}", all_method.text);
    assert_physical_origins(UPSTREAM, "testFor", "()I", all_method, body);
}

#[test]
fn controlled_returns_keep_full_class_maps_for_both_physical_methods() {
    let default = class_source(CONTROLS, "cf11/NestedLoopSideExitControls", false);
    let all = class_source(CONTROLS, "cf11/NestedLoopSideExitControls", true);
    assert_eq!(
        default.text, all.text,
        "class source is evidence-independent"
    );

    for (name, descriptor) in [("testFor", "(III)I"), ("runCounts", "(III)I")] {
        let default_method = method(&default, name, descriptor);
        let all_method = method(&all, name, descriptor);
        assert_same_body_and_map(default_method, all_method);
        let body = assert_structured(all_method);
        assert!(all_method.text.contains("return"), "{}", all_method.text);
        assert_physical_origins(CONTROLS, name, descriptor, all_method, body);
    }
}

#[test]
fn a_late_public_budget_stop_and_precancellation_publish_no_partial_body() {
    let complete = class_source(UPSTREAM, "cf11/UpstreamNestedLoops4", true);
    let mut open_budget = task_budget(&[]).expect("bounded task defaults");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(UPSTREAM.to_vec()), &mut open_budget)
        .expect("frozen class opens");
    let request = request(&snapshot, "cf11/UpstreamNestedLoops4");

    let mut late_limits = complete.limits.clone();
    let source_steps = complete
        .usage
        .analysis_steps
        .saturating_sub(open_budget.usage().analysis_steps);
    assert!(
        source_steps > 1,
        "the public source request has analysis work"
    );
    late_limits.analysis_steps = source_steps - 1;
    let mut late = Budget::new(late_limits);
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut late,
        )
        .expect("late budget stop is returned as an operation outcome");
    match &stopped {
        OperationOutcome::Incomplete(report) => eprintln!(
            "CF11 public late Stop: source_steps={source_steps}; execution={:?}",
            report.execution
        ),
        OperationOutcome::Performed(report) => eprintln!(
            "CF11 public late Stop: source_steps={source_steps}; execution={:?}",
            report.execution
        ),
        _ => {}
    }
    assert_late_stop(stopped);

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(complete.limits.clone(), token);
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("pre-cancellation is returned as an operation outcome");
    match &stopped {
        OperationOutcome::Incomplete(report) => {
            eprintln!("CF11 public precancel: execution={:?}", report.execution)
        }
        OperationOutcome::Performed(report) => {
            eprintln!("CF11 public precancel: execution={:?}", report.execution)
        }
        _ => {}
    }
    assert_precancel(stopped);
}

fn assert_late_stop(outcome: OperationOutcome<ClassSourceReport>) {
    match &outcome {
        OperationOutcome::Incomplete(candidates) => assert!(matches!(
            &candidates.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        )),
        OperationOutcome::Performed(report) => assert!(matches!(
            &report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        )),
        other => panic!("unexpected late-stop result: {other:?}"),
    }
    assert_no_partial_body(outcome);
}

fn log_public_late_stop(outcome: &OperationOutcome<ClassSourceReport>) {
    match outcome {
        OperationOutcome::Performed(report) => {
            let body_stops = report
                .methods
                .iter()
                .filter_map(|method| match &method.outcome {
                    ClassSourceOutcome::Recovered { report, .. } => {
                        report.outcome.stop().map(|reason| {
                            format!(
                                "method={:?} body={} bci={:?} stop={reason:?}",
                                method.item.identity,
                                report.method,
                                reason.at()
                            )
                        })
                    }
                    ClassSourceOutcome::Refused {
                        execution,
                        diagnostics,
                    } => Some(format!(
                        "method={:?} refused={execution:?}; diagnostics={diagnostics:?}",
                        method.item.identity
                    )),
                    _ => None,
                })
                .collect::<Vec<_>>();
            eprintln!(
                "CF11 for-condition public late Stop: execution={:?}; usage={:?}; body_stops={body_stops:?}; diagnostics={:?}",
                report.execution, report.usage, report.diagnostics
            );
        }
        OperationOutcome::Incomplete(candidates) => eprintln!(
            "CF11 for-condition public late Stop: execution={:?}; diagnostics={:?}; candidates={:?}",
            candidates.execution, candidates.diagnostics, candidates.candidates
        ),
        OperationOutcome::Ambiguous(candidates) => {
            panic!("the exact frozen class is not ambiguous: {candidates:?}")
        }
    }
}

fn assert_precancel(outcome: OperationOutcome<ClassSourceReport>) {
    let usage = match &outcome {
        OperationOutcome::Incomplete(candidates) => execution_usage(&candidates.execution),
        OperationOutcome::Performed(report) => {
            assert_zero_counted_usage(&report.usage);
            execution_usage(&report.execution)
        }
        OperationOutcome::Ambiguous(candidates) => {
            panic!("the exact frozen class is not ambiguous: {candidates:?}")
        }
    };
    eprintln!("CF11 public precancel counted usage (wall clock excluded): {usage:?}");
    assert_zero_counted_usage(usage);
    match &outcome {
        OperationOutcome::Incomplete(candidates) => {
            assert!(matches!(
                &candidates.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(candidates.candidates.is_empty());
        }
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                &report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert!(report.declaration.is_none());
            assert!(report.fields.is_empty());
            assert!(report.methods.is_empty());
            assert!(report.text.is_empty());
        }
        other => panic!("unexpected pre-cancellation result: {other:?}"),
    }
    assert_no_partial_body(outcome);
}

fn execution_usage(execution: &ExecutionReport) -> &UsageSnapshot {
    match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    }
}

fn assert_zero_counted_usage(usage: &UsageSnapshot) {
    let mut counted = usage.clone();
    counted.elapsed_millis = 0;
    assert_eq!(
        counted,
        UsageSnapshot::default(),
        "precancellation must charge no counted work"
    );
}

fn assert_no_partial_body(outcome: OperationOutcome<ClassSourceReport>) {
    match outcome {
        OperationOutcome::Incomplete(candidates) => {
            assert!(matches!(
                candidates.execution,
                ExecutionReport::Partial { .. } | ExecutionReport::Cancelled { .. }
            ));
        }
        OperationOutcome::Performed(report) => {
            for method in &report.methods {
                if let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome {
                    if !body.produced() {
                        assert!(body.text.is_empty(), "stopped body has no text");
                        assert!(body.source_map.is_empty(), "stopped body has no map");
                        assert_eq!(body.content, RecoveryContent::NotProduced);
                    }
                }
            }
            if matches!(&report.execution, ExecutionReport::Cancelled { .. }) {
                assert!(
                    report.text.is_empty(),
                    "cancelled class has no assembled text"
                );
            }
        }
        other => panic!("unexpected class-source stop: {other:?}"),
    }
}

#[test]
fn upstream_nested_loops2_keeps_outer_for_around_inner_while() {
    let default = class_source(UPSTREAM_NESTED2, "cf11/UpstreamNestedLoops2", false);
    let all = class_source(UPSTREAM_NESTED2, "cf11/UpstreamNestedLoops2", true);
    assert_eq!(
        default.text, all.text,
        "class source is evidence-independent"
    );
    let default_method = method(&default, "test", "(Ljava/util/List;)Z");
    let all_method = method(&all, "test", "(Ljava/util/List;)Z");
    assert_same_body_and_map(default_method, all_method);
    let body = assert_structured(all_method);
    assert_eq!(
        all_method.text.matches("for (").count(),
        1,
        "{}",
        all_method.text
    );
    assert!(all_method.text.contains("for ("), "{}", all_method.text);
    assert!(
        all_method.text.contains("list.size()"),
        "{}",
        all_method.text
    );
    assert_eq!(
        all_method.text.matches("while (").count(),
        1,
        "{}",
        all_method.text
    );
    assert!(all_method.text.contains("while ("), "{}", all_method.text);
    assert!(
        all_method.text.contains("s.length()"),
        "{}",
        all_method.text
    );
    assert_physical_origins(
        UPSTREAM_NESTED2,
        "test",
        "(Ljava/util/List;)Z",
        all_method,
        body,
    );
}

#[test]
fn a_for_condition_public_late_stop_reports_dimension_usage_and_location() {
    let complete = class_source(CONDITION_EFFECTS, "cf11/ForConditionEffects", true);
    let mut open_budget = task_budget(&[]).expect("bounded task defaults");
    let snapshot = Engine::new()
        .open(
            ArtifactInput::bytes(CONDITION_EFFECTS.to_vec()),
            &mut open_budget,
        )
        .expect("the frozen Java 8 class opens");
    let request = request(&snapshot, "cf11/ForConditionEffects");
    let source_steps = complete
        .usage
        .analysis_steps
        .saturating_sub(open_budget.usage().analysis_steps);
    assert!(
        source_steps > 1,
        "the public source request has analysis work"
    );

    let mut late_limits = complete.limits.clone();
    late_limits.analysis_steps = source_steps - 1;
    let mut late = Budget::new(late_limits);
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut late,
        )
        .expect("late budget stop is returned as an operation outcome");
    log_public_late_stop(&stopped);
    assert_late_stop(stopped);

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(complete.limits.clone(), token);
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("pre-cancellation is returned as an operation outcome");
    assert_precancel(stopped);
}
