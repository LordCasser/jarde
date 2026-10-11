//! P3 task 2.16: a switch's shared forward join is owned by the switch continuation.

use jarde::*;
use std::slice;

const SAMPLE: &[u8] = include_bytes!("fixtures/p3-join-ownership/v8/Join.class");

fn recover(bytes: &[u8], class_name: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("fixture opens as a standalone class");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name),
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
    match engine
        .class_source(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("class-source request succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one fixture class answers the request: {other:?}"),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("fixture does not contain `{name}`"))
}

fn statements(text: &str) -> String {
    let body = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let opened = body.find('{').map_or(body.len(), |index| index + 1);
    body[opened..]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn switch_arms_stop_before_the_shared_forward_join() {
    let report = recover(SAMPLE, "Join");
    let method = method(&report, "mixedExits");
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!(
            "mixedExits must recover executable statements: {:?}\n{}",
            method.outcome, method.text
        );
    };
    assert_eq!(
        body.content,
        RecoveryContent::ContainsStatements,
        "{:?}\n{}",
        body.fallbacks,
        method.text
    );
    assert!(
        body.fallbacks.is_empty(),
        "{:?}\n{}",
        body.fallbacks,
        method.text
    );
    let text = statements(&method.text);
    assert_eq!(
        text,
        "int local1; local1 = 0; switch (arg0) { case 1: local1 = 10; break; case 2: return 20; default: local1 = 30; break; } return local1; }",
        "the shared BCI 40 return follows the switch and is emitted once:\n{}",
        method.text
    );
    assert_eq!(text.matches("return local1;").count(), 1, "{text}");
    assert!(!text.contains("@bytecode"), "{text}");
}

fn with_switch_goto_target(offset: i16) -> Vec<u8> {
    let mut bytes = SAMPLE.to_vec();
    let pattern = [0x10, 10, 0x3c, 0xa7, 0, 9];
    let locations: Vec<_> = bytes
        .windows(pattern.len())
        .enumerate()
        .filter_map(|(index, window)| (window == pattern).then_some(index))
        .collect();
    assert_eq!(locations.len(), 1, "the fixture has one case-1 goto");
    let offset_at = locations[0] + 4;
    bytes[offset_at..offset_at + 2].copy_from_slice(&offset.to_be_bytes());
    bytes
}

#[test]
fn cross_case_goto_and_looping_arm_do_not_gain_a_forward_join() {
    for (name, offset) in [("cross_case", 6), ("backedge", -3)] {
        let bytes = with_switch_goto_target(offset);
        let report = recover(&bytes, "Join");
        let method = method(&report, "mixedExits");
        assert!(
            method.text.contains("@bytecode") && !method.text.contains("switch (arg0)"),
            "{name} has no supported forward join and remains quoted:\n{}",
            method.text
        );
        assert!(
            !method.text.contains("return local1;"),
            "a refused join is not separately emitted:\n{}",
            method.text
        );
    }
}

#[test]
fn analysis_budget_during_switch_join_proof_does_not_publish_a_complete_class() {
    let engine = Engine::new();
    let mut budget =
        task_budget(&[BudgetOverride::new("analysis_steps", 70).expect("valid analysis bound")])
            .expect("bounded budget");
    let snapshot = engine
        .open(ArtifactInput::bytes(SAMPLE.to_vec()), &mut budget)
        .expect("fixture opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("Join"),
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
    let outcome = engine
        .class_source(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("budget exhaustion is reported as an outcome");
    assert!(
        matches!(outcome, OperationOutcome::Incomplete(_))
            || matches!(outcome, OperationOutcome::Performed(ref report) if matches!(report.execution, ExecutionReport::Partial { .. })),
        "join proof stops before publishing a complete report: {outcome:?}"
    );
}
