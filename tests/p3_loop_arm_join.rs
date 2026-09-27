//! A nested `if` can keep a single-exit loop in one arm and its local join in the outer arm.

use jarde::*;
use std::slice;

const JOIN: &[u8] = include_bytes!("fixtures/p3-loop-arm-join/v8/LoopIfJoin.class");
const EXTRA_BREAK: &[u8] =
    include_bytes!("fixtures/p3-loop-arm-join/v8/LoopIfJoinExtraBreak.class");

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

fn source(bytes: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("Java 8 class opens");
    match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot, class),
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("class-source request completes")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one class answers the request: {other:?}"),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> (&'a str, &'a RecoveryReport) {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .expect("method exists");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("method recovery is present: {:?}", method.outcome);
    };
    (&method.text, report)
}

#[test]
fn loop_arm_joins_before_the_outer_arm_tail_once() {
    let report = source(JOIN, "cf08join/LoopIfJoin");
    let (text, recovery) = method(&report, "run");
    assert!(
        recovery.fallbacks.is_empty() && !text.contains("@bytecode"),
        "{text}"
    );
    assert_eq!(recovery.quality, jarde_jvm::ir::Quality::Structured);
    assert!(text.contains("if (arg1) {"), "{text}");
    assert!(text.contains("while (local2 < 3)"), "{text}");
    assert_eq!(text.matches("local2 = local2 + 1;").count(), 1, "{text}");
    assert_eq!(text.matches("local2 = local2 + 10;").count(), 1, "{text}");
    assert!(text.find("local2 = local2 + 10;") > text.find("while (local2 < 3)"));
    for bci in [0, 6, 10, 15, 20, 26, 32, 34, 35] {
        assert!(
            !recovery.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost its source: {text}"
        );
    }
}

fn with_branch_target(from: &[u8], to: &[u8]) -> Vec<u8> {
    let occurrences = JOIN
        .windows(from.len())
        .filter(|window| *window == from)
        .count();
    assert_eq!(occurrences, 1, "one fixed bytecode branch is mutated");
    let index = JOIN
        .windows(from.len())
        .position(|window| window == from)
        .unwrap();
    let mut bytes = JOIN.to_vec();
    bytes[index..index + to.len()].copy_from_slice(to);
    bytes
}

#[test]
fn a_second_loop_entry_or_different_exit_keeps_the_method_quoted() {
    // BCI 3 enters the loop header directly; BCI 17 instead exits to the outer sibling.
    // Both mutations retain a valid Java 8 StackMap target, but neither has this local join.
    let cases = [
        (
            "second entry",
            with_branch_target(
                &[0x1a, 0x99, 0, 0x1d, 0x1b, 0x99],
                &[0x1a, 0x99, 0, 0x0c, 0x1b, 0x99],
            ),
        ),
        (
            "different exit",
            with_branch_target(
                &[0x1c, 0x06, 0xa2, 0, 9, 0x84, 2, 1],
                &[0x1c, 0x06, 0xa2, 0, 15, 0x84, 2, 1],
            ),
        ),
    ];
    for (name, bytes) in cases {
        let report = source(&bytes, "cf08join/LoopIfJoin");
        let (text, recovery) = method(&report, "run");
        assert!(
            text.contains("@bytecode") && !recovery.fallbacks.is_empty(),
            "{name}: {text}"
        );
        assert!(!text.contains("local2 = local2 + 10;"), "{name}: {text}");
    }
}

#[test]
fn extra_loop_transfers_do_not_claim_the_inner_join_tail() {
    let report = source(EXTRA_BREAK, "LoopIfJoinExtraBreak");
    for name in ["run", "runReturn", "runException"] {
        let (text, recovery) = method(&report, name);
        assert!(
            text.contains("@bytecode") && !recovery.fallbacks.is_empty(),
            "{name}: {text}"
        );
        assert!(!text.contains("value = value + 10;"), "{name}: {text}");
    }
}

#[test]
fn budget_and_cancellation_do_not_publish_a_partial_loop_arm() {
    let engine = Engine::new();
    for cancelled in [false, true] {
        let mut budget = if cancelled {
            task_budget(&[]).unwrap()
        } else {
            task_budget(&[BudgetOverride::new("analysis_steps", 120).unwrap()]).unwrap()
        };
        let snapshot = engine
            .open(ArtifactInput::bytes(JOIN.to_vec()), &mut budget)
            .expect("fixture opens");
        if cancelled {
            budget.cancellation_token().cancel();
        }
        let outcome = engine
            .class_source(
                slice::from_ref(&snapshot),
                &request(&snapshot, "cf08join/LoopIfJoin"),
                &mut budget,
            )
            .expect("stop is represented in the outcome");
        assert!(
            matches!(outcome, OperationOutcome::Incomplete(_))
                || matches!(outcome, OperationOutcome::Performed(ref report) if !matches!(report.execution, ExecutionReport::Complete { .. })),
            "the loop join never publishes a complete partial class: {outcome:?}"
        );
    }
}
