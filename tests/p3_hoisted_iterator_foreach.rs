//! Public API regression for the proved elevated inner-iterator declaration.
//!
//! Uses the accepted frozen CF11 class. The outer iterator's assignment/while remains outside
//! this feature; only the existing inner while is eligible for the Object foreach projection.

use jarde::*;
use jarde_jvm::ir::{Quality, Representation};
use jarde_reader::budget::{Budget, CancellationToken, UsageSnapshot};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::{ExecutionReport, TerminationReason};
use std::slice;

const FIXTURE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1/capture/legs/javac23-original/classes/cf11/UpstreamNestedLoops.class"
);
const INTERNAL_CLASS: &str = "cf11/UpstreamNestedLoops";
const METHOD: &str = "test";
const DESCRIPTOR: &str = "(Ljava/util/List;Ljava/util/List;)V";

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(INTERNAL_CLASS),
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

fn opened() -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(FIXTURE.to_vec()),
            &mut task_budget(&[]).expect("bounded fixture-open budget"),
        )
        .expect("the frozen Java 8 class opens")
}

fn source(
    snapshot: &ArtifactSnapshot,
    request: &ClassSourceRequest,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    Engine::new()
        .class_source_with_evidence(slice::from_ref(snapshot), request, evidence, budget)
        .expect("the public class-source request answers")
}

fn source_complete(
    snapshot: &ArtifactSnapshot,
    request: &ClassSourceRequest,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    match source(
        snapshot,
        request,
        evidence,
        &mut task_budget(&[]).expect("bounded task budget"),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("the frozen class binds exactly once: {other:?}"),
    }
}

fn target<'a>(report: &'a ClassSourceReport) -> (&'a ClassSourceMethod, &'a RecoveryReport) {
    let method = report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == METHOD.as_bytes()
                && method.item.descriptor.raw().0 == DESCRIPTOR.as_bytes()
        })
        .expect("the frozen physical test method is present");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!(
            "test has a method-local recovery report: {:?}",
            method.outcome
        );
    };
    (method, report)
}

fn assert_utf8_map(body: &RecoveryReport) {
    for segment in body.source_map.segments() {
        assert!(segment.start() < segment.end());
        assert!(body.text.is_char_boundary(segment.start()));
        assert!(body.text.is_char_boundary(segment.end()));
        assert!(body.text.get(segment.start()..segment.end()).is_some());
    }
}

fn assert_target_body(body: &RecoveryReport) {
    assert!(body.produced(), "{:?}", body.outcome);
    assert_eq!(body.representation, Representation::Java);
    assert_eq!(body.quality, Quality::Structured);
    assert!(matches!(&body.execution, ExecutionReport::Complete { .. }));
    assert!(
        body.text.contains("for (java.lang.Object iteratorElement"),
        "{}",
        body.text
    );
    assert!(body.text.contains(" : l2)"), "{}", body.text);
    assert_eq!(body.text.matches("for (").count(), 1, "{}", body.text);
    assert!(
        body.text.contains("local3 = l1.iterator()"),
        "{}",
        body.text
    );
    assert!(
        body.text.contains("while (local3.hasNext())"),
        "{}",
        body.text
    );
    assert!(!body.text.contains("l2.iterator()"), "{}", body.text);
    assert!(!body.text.contains("local5.hasNext()"), "{}", body.text);
    assert!(
        body.text
            .contains("s2 = (java.lang.String) iteratorElement"),
        "{}",
        body.text
    );
    assert!(body.text.contains("if (s1.equals("), "{}", body.text);
    assert!(body.text.contains("l2.add("), "{}", body.text);
    assert!(body.text.contains("l1.remove("), "{}", body.text);
    assert!(body.text.contains("l1.clear()"), "{}", body.text);
}

#[test]
fn nested_inner_iterator_projects_without_changing_outer_loop_or_binding() {
    let snapshot = opened();
    let request = request(&snapshot);
    let default = match Engine::new()
        .class_source(
            slice::from_ref(&snapshot),
            &request,
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("default request completes: {other:?}"),
    };
    let all = source_complete(&snapshot, &request, &RecoveryEvidenceRequest::all());
    let no_rule_details = source_complete(
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::essential()
            .with_kind(RecoveryEvidenceKind::SourceMap)
            .with_kind(RecoveryEvidenceKind::RegionDetails)
            .with_kind(RecoveryEvidenceKind::NameDetails),
    );
    let (default_method, default_body) = target(&default);
    let (all_method, all_body) = target(&all);
    let (plain_method, plain_body) = target(&no_rule_details);
    for body in [default_body, all_body, plain_body] {
        assert_target_body(body);
        assert_utf8_map(body);
    }
    assert_eq!(default.text, all.text);
    assert_eq!(default.text, no_rule_details.text);
    assert_eq!(default_body.text, all_body.text);
    assert_eq!(default_body.text, plain_body.text);
    assert_eq!(default_body.source_map, all_body.source_map);
    assert_eq!(default_body.source_map, plain_body.source_map);
    assert_eq!(default_method.item.identity, all_method.item.identity);
    assert_eq!(default_method.item.identity, plain_method.item.identity);
    assert_eq!(
        all_body.evidence.state(RecoveryEvidenceKind::RuleDetails),
        EvidenceState::Complete
    );
    assert_eq!(
        plain_body.evidence.state(RecoveryEvidenceKind::RuleDetails),
        EvidenceState::NotRequested
    );

    let mut bytecode_budget = task_budget(&[]).expect("bounded bytecode inspection budget");
    let physical = inspect_method_bytecode(
        FIXTURE,
        MethodSelector {
            name: JvmBytes(METHOD.as_bytes().to_vec()),
            descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
        },
        &mut bytecode_budget,
    )
    .expect("the frozen method's physical instructions are inspectable");
    assert!(physical.stopped_at.is_none());
    assert_eq!(
        physical.instructions.len(),
        45,
        "the accepted class method identity is pinned"
    );
    for instruction in &physical.instructions {
        let segments = all_body.source_map.of_bci(instruction.bci);
        assert!(
            !segments.is_empty(),
            "physical BCI {} has no source map",
            instruction.bci
        );
        assert!(
            segments.iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| {
                        origin.bci() == instruction.bci
                            && origin.method() == Some(&all_method.item.identity)
                    })
            }),
            "physical BCI {} has no exact target-method origin: {segments:?}",
            instruction.bci
        );
    }
}

#[test]
fn nested_foreach_late_stop_and_precancel_leave_no_partial_method_artifact() {
    let snapshot = opened();
    let request = request(&snapshot);
    let complete = source_complete(&snapshot, &request, &RecoveryEvidenceRequest::all());
    let (complete_method, complete_body) = target(&complete);
    let expected_identity = complete_method.item.identity.clone();
    let ExecutionReport::Complete { usage } = &complete_body.execution else {
        panic!(
            "complete method reports measured usage: {:?}",
            complete_body.execution
        );
    };
    assert!(
        usage.analysis_steps > 1,
        "the measured method has analysis work"
    );

    let mut limits = task_budget(&[])
        .expect("bounded task budget")
        .limits()
        .clone();
    limits.analysis_steps = usage.analysis_steps - 1;
    let analysis_step_limit = limits.analysis_steps;
    let stopped = source(
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut Budget::new(limits),
    );
    let report = match stopped {
        OperationOutcome::Performed(report) => report,
        other => panic!("the late budget stop reaches a class report: {other:?}"),
    };
    let method = report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == METHOD.as_bytes()
                && method.item.descriptor.raw().0 == DESCRIPTOR.as_bytes()
        })
        .expect("the exact target remains present in the partial report");
    assert_eq!(method.item.identity, expected_identity);
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!("target has a body-level stop report: {:?}", method.outcome);
    };
    assert!(
        matches!(
            &body.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        ),
        "target itself stops for AnalysisSteps: {:?}",
        body.execution
    );
    let stop = body.stop().expect("target carries its stop");
    assert!(stop.at().is_some(), "stop retains a location: {stop:?}");
    let method_usage = match &body.execution {
        ExecutionReport::Partial { usage, .. } => usage,
        other => panic!("partial target carries counted usage: {other:?}"),
    };
    assert!(method_usage.analysis_steps > 0);
    assert!(method_usage.analysis_steps <= analysis_step_limit);
    eprintln!(
        "[hoisted-iterator-api-stop] limit={analysis_step_limit} usage={} stop={stop:?}",
        method_usage.analysis_steps
    );
    assert!(!body.produced());
    assert_eq!(body.text, "");
    assert!(body.source_map.is_empty());
    assert_eq!(body.content, RecoveryContent::NotProduced);
    assert!(body.news.is_empty());

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(
        task_budget(&[])
            .expect("bounded task budget")
            .limits()
            .clone(),
        token,
    );
    let outcome = source(
        &snapshot,
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut cancelled,
    );
    let execution = match &outcome {
        OperationOutcome::Incomplete(report) => &report.execution,
        OperationOutcome::Performed(report) => &report.execution,
        OperationOutcome::Ambiguous(candidates) => {
            panic!("fixed fixture is not ambiguous: {candidates:?}")
        }
    };
    let usage = match execution {
        ExecutionReport::Cancelled { usage } => usage,
        other => panic!("pre-cancel is reported as Cancelled: {other:?}"),
    };
    let mut counted = usage.clone();
    counted.elapsed_millis = 0;
    assert_eq!(
        counted,
        UsageSnapshot::default(),
        "pre-cancel charges no counted work"
    );
    match outcome {
        OperationOutcome::Incomplete(report) => assert!(report.candidates.is_empty()),
        OperationOutcome::Performed(report) => {
            assert!(report.text.is_empty());
            assert!(report.methods.is_empty());
            assert!(report.fields.is_empty());
        }
        OperationOutcome::Ambiguous(_) => unreachable!(),
    }
}
