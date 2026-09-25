//! P3 2c.6: a loop may keep a call or field read in its condition when the branch consumes its
//! value. Test-block stores and increments still refuse the loop.

use jarde::*;
use std::slice;

const ITERABLE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/enhanced-for/string-iterable/StringIterableForeach.class"
);
const LOOP_VALUES: &[u8] = include_bytes!("fixtures/p3-loop-test-values/v8/LoopTestValues.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn report(bytes: &[u8], class: &str) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed class fixture opens");
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
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("the legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed class has one class-source result, got {other:?}"),
    }
}

fn method_text<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}`"))
        .text
}

#[test]
fn an_iterator_call_consumed_by_the_loop_branch_stays_in_the_condition() {
    let source = report(ITERABLE, "StringIterableForeach");
    let method = method_text(&source, "sumLengths");
    assert!(
        method.contains("while (local2.hasNext())"),
        "hasNext is read by the loop branch and must stay in its condition:\n{method}"
    );
    assert!(
        method.contains("local2.next()"),
        "the loop body remains in place:\n{method}"
    );
    assert_eq!(
        method.matches("local2.hasNext()").count(),
        1,
        "the condition call is written once at its loop test, never hoisted or duplicated:\n{method}"
    );
}

#[test]
fn a_field_condition_survives_beside_an_increment_test_refusal_in_the_same_method() {
    let source = report(LOOP_VALUES, "LoopTestValues");
    let method = method_text(&source, "fieldThenIncrementTest");
    assert!(
        method.contains("while (this.n > 0)"),
        "the field read is consumed by the first loop's condition:\n{method}"
    );
    assert_eq!(
        method.matches("this.n").count(),
        1,
        "the field is read once in the condition and is absent from the body:\n{method}"
    );
    assert!(
        method.contains("instruction at BCI 14"),
        "the positive field loop and later increment test are in one method, and the loop with the increment is refused:\n{method}"
    );
}

#[test]
fn a_store_in_a_loop_test_remains_refused() {
    let source = report(LOOP_VALUES, "LoopTestValues");
    let method = method_text(&source, "storeTest");
    assert!(
        method.contains("@bytecode") && method.contains("instruction at BCI 1"),
        "the test assignment is not moved into or out of a loop:\n{method}"
    );
}

#[test]
fn an_iinc_in_a_loop_test_remains_refused() {
    let source = report(LOOP_VALUES, "LoopTestValues");
    let method = method_text(&source, "incrementTest");
    assert!(
        method.contains("@bytecode") && method.contains("instruction at BCI 1"),
        "the test increment is not moved into or out of a loop:\n{method}"
    );
}
