//! A switch's normal join inside a loop is distinct from a case's loop-break target.

use jarde::*;
use std::slice;

const CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/switch-loop-exits/SwitchLoopExits.class"
);
const ADJACENT: &[u8] = include_bytes!("fixtures/p3-switch-loop-exits/v8/SwitchLoopAdjacent.class");

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
        text.contains("jarde_loop_5: while") && text.contains("switch (local3)"),
        "{text}"
    );
    assert!(
        text.contains("if (arg2) {\n                        break jarde_loop_5;"),
        "{text}"
    );
    assert!(
        text.contains("default:\n                    break jarde_loop_5;"),
        "{text}"
    );
    assert_eq!(text.matches("local3 = local3 + 1;").count(), 1, "{text}");
    assert_eq!(text.matches("return local4;").count(), 1, "{text}");
    assert!(
        !text.contains("break jarde_loop_5;\n                    break;"),
        "{text}"
    );
    for bci in [40, 55] {
        assert!(
            body.source_map
                .text_of_bci(&body.text, bci)
                .iter()
                .any(|piece| piece.contains("break jarde_loop_5;")),
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
fn budget_and_cancellation_do_not_publish_a_partial_switch() {
    let (engine, snapshot, request) = opened(CLASS, "SwitchLoopExits");
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
