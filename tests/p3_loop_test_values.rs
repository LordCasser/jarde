//! P3 2c.6: a loop may keep a call or field read in its condition when the branch consumes its
//! value. Test-block stores and increments still refuse the loop.
//!
//! The condition requirement itself outlived the shape that first carried it: the accepted
//! enhanced-`for` projection (`project-proved-enhanced-for-loops` 4.2/4.3) folds a proved direct
//! `Iterable.iterator()`/`hasNext()`/`next()` loop into a `for (element : iterable)` whose header
//! evaluates the iterable and the condition once per iteration and keeps the element cast in the
//! body, so `sumLengths` — raw `Iterable`, element cast at its original position — presents as a
//! `for` instead of the `while` the 2026-09-24 evidence recorded. The call is still never hoisted
//! out of the loop test or duplicated; that is what the projection's own proof states.

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
        method.contains("for (java.lang.Object iteratorElement"),
        "the proved iterator loop projects: the condition call stays the loop's own once-per-\niteration test, never hoisted or duplicated:\n{method}"
    );
    assert!(
        method.contains(": arg0)"),
        "the iterable expression is the projected loop's source:\n{method}"
    );
    assert!(
        method.contains("(java.lang.String)"),
        "the element cast stays in the body at its original position:\n{method}"
    );
    assert!(
        method.contains("local3.length();"),
        "the loop body remains in place:\n{method}"
    );
    assert!(
        !method.contains(".hasNext()")
            && !method.contains(".next()")
            && !method.contains(".iterator()"),
        "the iterator calls are folded into the header exactly once, not copied:\n{method}"
    );
    assert!(
        !method.contains("@bytecode"),
        "the projection is complete, with no refused region left beside it:\n{method}"
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
