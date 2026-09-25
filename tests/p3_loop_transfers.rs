//! P3 3.1: only proved loop exits become Java break statements, with the target loop kept.

use jarde::*;
use std::slice;

const GRID: &[u8] =
    include_bytes!("../openspec/evidence/java-syntax-2026-09-24/loop-transfers/Grid.class");
const OUTER_CONTINUE: &[u8] = include_bytes!("fixtures/p3-loop-transfers/v8/OuterContinue.class");
const OBJECT_ARRAY: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/enhanced-for/object-array-body-if-red/ObjectArrayForeach.class"
);
const SWITCH_LOOP: &[u8] = include_bytes!("fixtures/p3-loop-transfers/v8/SwitchLoopTransfer.class");

fn class_source(bytes: &[u8], class: &str) -> ClassSourceReport {
    class_source_with_evidence(bytes, class, &RecoveryEvidenceRequest::essential())
}

fn class_source_with_evidence(
    bytes: &[u8],
    class: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen class opens");
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
    match Engine::new()
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, evidence, &mut budget)
        .expect("single-class request succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one class-source result is expected, got {other:?}"),
    }
}

#[test]
fn the_for_header_and_continue_keep_their_bytecode_origins() {
    let essential = class_source(OUTER_CONTINUE, "OuterContinue");
    let all = class_source_with_evidence(
        OUTER_CONTINUE,
        "OuterContinue",
        &RecoveryEvidenceRequest::all(),
    );
    assert_eq!(
        essential.text, all.text,
        "evidence selection keeps the same source"
    );
    let method = all
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"run")
        .expect("the class has run");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("run has no recovery body: {:?}", method.outcome)
    };
    for (bci, text) in [
        (3, "local2 = 0"),
        (21, "continue jarde_loop_4"),
        (36, "local2 = local2 + 1"),
    ] {
        assert!(
            report
                .source_map
                .text_of_bci(&report.text, bci)
                .iter()
                .any(|piece| piece.contains(text)),
            "BCI {bci} lost `{text}` in the projected source: {:?}",
            report.source_map.of_bci(bci)
        );
    }
}

fn method_text<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("the class has no `{name}` member"))
        .text
}

#[test]
fn nested_breaks_stay_in_the_taken_arm_and_outer_break_keeps_its_loop_target() {
    let report = class_source(GRID, "Grid");

    let inner = method_text(&report, "nestedBreak");
    assert!(!inner.contains("@bytecode"), "{inner}");
    assert!(
        inner.contains("if (local2 + local3 > 5) {\n                    break;"),
        "{inner}"
    );

    // This frozen source's `continue outer` targets the inner loop's natural exit, which is also
    // the outer update entry. The CFG proves the same behavior as an inner break; source labels
    // alone do not justify changing that target.
    let equivalent = method_text(&report, "labeledContinue");
    assert!(!equivalent.contains("@bytecode"), "{equivalent}");
    assert!(
        equivalent.contains("if (local3 == 2) {\n                    break;"),
        "{equivalent}"
    );

    let outer = method_text(&report, "labeledBreak");
    assert!(!outer.contains("@bytecode"), "{outer}");
    assert!(
        outer.contains("if (local3 == 3) {\n                    break jarde_loop_4;"),
        "{outer}"
    );
    assert!(outer.contains("jarde_loop_4: while"), "{outer}");
}

#[test]
fn an_outer_continue_runs_the_proved_for_update_and_skips_the_body_tail() {
    let report = class_source(OUTER_CONTINUE, "OuterContinue");
    let method = method_text(&report, "run");
    assert!(
        !method.contains("@bytecode")
            && method
                .contains("jarde_loop_4: for (local2 = 0; local2 < arg0; local2 = local2 + 1)"),
        "BCI 36 is the only for update, including on the non-local continue path:\n{method}"
    );
    assert!(
        method.contains("continue jarde_loop_4;")
            && method.contains("local1 = local1 + 100;")
            && !method.contains("for (local2 = 0; local2 < arg0; local1"),
        "the continue skips BCI 33, while a normal iteration keeps that statement in the body:\n{method}"
    );
}

#[test]
fn a_nested_if_remains_in_the_loop_body_after_a_nested_region_returns() {
    let report = class_source(OBJECT_ARRAY, "ObjectArrayForeach");
    let method = method_text(&report, "sumHash");
    assert!(!method.contains("@bytecode"), "{method}");
    assert!(
        method.contains("for (java.lang.Object local5 : local2)"),
        "{method}"
    );
    assert!(method.contains("if (local5 != null)"), "{method}");
    assert!(method.contains("return local1;"), "{method}");
}

#[test]
fn a_switch_capturing_a_loop_break_keeps_the_loop_target_and_body_update() {
    let report = class_source(SWITCH_LOOP, "SwitchLoopTransfer");
    let method = method_text(&report, "run");
    assert!(
        !method.contains("@bytecode")
            && method.contains("switch (local1)")
            && method.contains("break jarde_loop_4;")
            && method.matches("local1 = local1 + 1;").count() == 1,
        "the switch's normal join stays inside the loop, and its loop breaks retain their target:\n{method}"
    );
}
