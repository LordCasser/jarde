//! A switch's normal join inside a loop is distinct from a case's loop-break target.
//!
//! The labeled loop's label is spelled `loop` since `recover-labeled-loop-tail-coverage`: the
//! presentation replaces the synthesized `jarde_loop_{bci}` with a source-style name; the loop the
//! assertions target is unchanged.

use jarde::*;
use std::slice;

const CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/switch-loop-exits/SwitchLoopExits.class"
);
const ADJACENT: &[u8] = include_bytes!("fixtures/p3-switch-loop-exits/v8/SwitchLoopAdjacent.class");
const LOCAL_CONTINUE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/original/cf13/SwitchLoopExits.class"
);
const LOCAL_BOUNDARIES: &[u8] =
    include_bytes!("fixtures/p3-switch-loop-exits/v8/SwitchLoopLocalBoundaries.class");

fn opened(bytes: &[u8], class: &str) -> (Engine, ArtifactSnapshot, ClassSourceRequest) {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen Java 8 class opens");
    let request = ClassSourceRequest {
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
    };
    (engine, snapshot, request)
}

#[test]
fn switch_loop_exit_and_local_join_keep_their_own_origins() {
    let (engine, snapshot, request) = opened(CLASS, "SwitchLoopExits");
    let report = match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut task_budget(&[]).expect("bounded default budget"),
        )
        .expect("class source succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("expected a full class, got {other:?}"),
    };
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .expect("run method exists");
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!("run must recover: {:?}", method.outcome);
    };
    let text = &method.text;
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        text.contains("loop: while") && text.contains("switch (local3)"),
        "{text}"
    );
    assert!(
        text.contains("if (arg2) {\n                        break loop;"),
        "{text}"
    );
    assert!(
        text.contains("default:\n                    break loop;"),
        "{text}"
    );
    assert_eq!(text.matches("local3 = local3 + 1;").count(), 1, "{text}");
    assert_eq!(text.matches("return local4;").count(), 1, "{text}");
    assert!(
        !text.contains("break loop;\n                    break;"),
        "{text}"
    );
    for bci in [40, 55] {
        assert!(
            body.source_map
                .text_of_bci(&body.text, bci)
                .iter()
                .any(|piece| piece.contains("break loop;")),
            "BCI {bci} must keep its loop-break origin: {:?}",
            body.source_map.of_bci(bci)
        );
    }
    assert!(
        body.source_map
            .text_of_bci(&body.text, 58)
            .iter()
            .any(|piece| piece.contains("local3 = local3 + 1;")),
        "BCI 58 owns the sole loop update"
    );
}

#[test]
fn switch_local_join_precedes_current_loop_continue_without_duplicate_ownership() {
    let (engine, snapshot, request) = opened(LOCAL_CONTINUE, "cf13/SwitchLoopExits");
    let report = match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut task_budget(&[]).expect("bounded default budget"),
        )
        .expect("class source succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("expected a complete class, got {other:?}"),
    };
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"walk")
        .expect("walk method exists");
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!("walk must recover: {:?}", method.outcome);
    };
    let text = &method.text;
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(text.contains("switch (i % 3)"), "{text}");
    assert!(
        text.contains("score = score + 10;\n                    continue;"),
        "{text}"
    );
    assert!(
        !text.contains("continue;\n                    break;"),
        "{text}"
    );
    assert_eq!(text.matches("score = score + 3;").count(), 1, "{text}");
    assert_eq!(text.matches("i = i + 1").count(), 1, "{text}");
    for (bci, expected) in [
        (9, "i"),
        (12, "switch (i % 3)"),
        (49, "continue;"),
        (55, "score = score + 3;"),
        (58, "i = i + 1"),
    ] {
        assert!(
            body.source_map
                .text_of_bci(&body.text, bci)
                .iter()
                .any(|piece| piece.contains(expected)),
            "BCI {bci} must own {expected}: {:?}",
            body.source_map.of_bci(bci)
        );
    }
}

#[test]
fn local_continue_certificate_respects_switch_and_loop_boundaries() {
    let (engine, snapshot, request) = opened(LOCAL_BOUNDARIES, "SwitchLoopLocalBoundaries");
    let report = match engine
        .class_source(
            slice::from_ref(&snapshot),
            &request,
            &mut task_budget(&[]).expect("bounded default budget"),
        )
        .expect("class source succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("expected a class report, got {other:?}"),
    };
    let ordinary = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"ordinaryTail")
        .expect("ordinary tail method exists");
    assert!(
        matches!(ordinary.outcome, ClassSourceOutcome::Recovered { .. })
            && ordinary.text.contains("switch (")
            && !ordinary.text.contains("continue;"),
        "a normal loop tail must not be mistaken for a case continue:\n{}",
        ordinary.text
    );
    let cross_case = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"crossCase")
        .expect("cross-case method exists");
    assert!(
        cross_case.text.contains("@bytecode") && !cross_case.text.contains("switch ("),
        "a cross-case route cannot use this local-join certificate:\n{}",
        cross_case.text
    );
    let extra_entry = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"extraEntry")
        .expect("extra-entry method exists");
    assert!(
        extra_entry.text.contains("@bytecode") && !extra_entry.text.contains("continue;"),
        "an outside path into the shared tail cannot use this certificate:\n{}",
        extra_entry.text
    );
    let multiple_joins = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"multipleJoins")
        .expect("multiple-join method exists");
    assert!(
        multiple_joins.text.contains("@bytecode") && !multiple_joins.text.contains("continue;"),
        "two candidate joins cannot use this certificate:\n{}",
        multiple_joins.text
    );
}

#[test]
fn budget_and_cancellation_do_not_publish_a_partial_switch() {
    for (bytes, class) in [
        (CLASS, "SwitchLoopExits"),
        (LOCAL_CONTINUE, "cf13/SwitchLoopExits"),
    ] {
        let (engine, snapshot, request) = opened(bytes, class);
        let mut limited =
            task_budget(&[BudgetOverride::new("analysis_steps", 1).expect("valid analysis bound")])
                .expect("bounded budget");
        let stopped = engine
            .class_source(slice::from_ref(&snapshot), &request, &mut limited)
            .expect("budget stop is an outcome");
        assert!(
            matches!(stopped, OperationOutcome::Incomplete(_))
                || matches!(stopped, OperationOutcome::Performed(ref report) if matches!(report.execution, ExecutionReport::Partial { .. })),
            "budget stop cannot claim a complete class: {stopped:?}"
        );

        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(Limits::default(), token);
        let stopped = engine
            .class_source(slice::from_ref(&snapshot), &request, &mut cancelled)
            .expect("cancellation is an outcome");
        assert!(
            matches!(stopped, OperationOutcome::Incomplete(ref result) if matches!(result.execution, ExecutionReport::Cancelled { .. })),
            "cancelled request cannot publish a complete class: {stopped:?}"
        );
    }
}

#[test]
fn nested_switch_and_continue_without_a_unique_local_join_stay_quoted() {
    let (engine, snapshot, request) = opened(ADJACENT, "SwitchLoopAdjacent");
    let report = match engine
        .class_source(
            slice::from_ref(&snapshot),
            &request,
            &mut task_budget(&[]).expect("bounded default budget"),
        )
        .expect("class source succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("expected a class report, got {other:?}"),
    };
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"nested")
        .expect("nested method exists");
    assert!(
        method.text.contains("@bytecode")
            && !method.text.contains("switch (")
            && !method.text.contains("break;"),
        "a switch with a nested switch, continue, and no unique in-loop join must not claim executable Java:\n{}",
        method.text
    );
}
