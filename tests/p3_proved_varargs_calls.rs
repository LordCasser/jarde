//! EM-13 call-site varargs are written only after the same class header proves the unique target,
//! its `ACC_VARARGS` array parameter, and the call-owned inline initializer.

use jarde::*;
use std::slice;

const FIXTURE: &[u8] = include_bytes!("fixtures/proved-varargs-calls/v8/VarargsCalls.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn open() -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(FIXTURE.to_vec()), &mut budget())
        .expect("the frozen Java 8 class opens")
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("VarargsCalls"),
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

fn source<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no method `{name}`"));
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => &report.text,
        other => panic!("`{name}` did not produce a body report: {other:?}"),
    }
}

fn recovery<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no method `{name}`"));
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` did not produce a body report: {other:?}"),
    }
}

fn complete_source(snapshot: &ArtifactSnapshot) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot),
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("the class-source request is valid")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("the frozen class is fully presented: {other:?}"),
    }
}

#[test]
fn only_proved_inline_varargs_calls_expand_and_every_other_call_keeps_its_array() {
    let snapshot = open();
    let report = complete_source(&snapshot);

    for (name, expected) in [
        ("ordered", "return count(mark(1), mark(2), mark(3));"),
        ("oneString", "return strings(\"one\");"),
        ("objectValues", "return objects(\"a\", \"b\");"),
        ("explicitVarargsArray", "return count(mark(4), mark(5));"),
    ] {
        assert!(
            source(&report, name).contains(expected),
            "`{name}`:\n{}",
            source(&report, name)
        );
    }

    for (name, expected) in [
        (
            "ordinaryArray",
            "return plain(new int[]{mark(6), mark(7)});",
        ),
        ("heldArray", "return count(local0);"),
        (
            "overloadedArray",
            "return overloaded(new java.lang.Object[]{\"x\"});",
        ),
        ("nullElement", "return shape(new java.lang.Object[]{null});"),
        (
            "arrayElement",
            "return shape((java.lang.Object[]) new java.lang.String[]{\"x\"});",
        ),
        ("exceptionOrder", "return count(mark(1), mark(9), mark(2));"),
    ] {
        assert!(
            source(&report, name).contains(expected),
            "`{name}`:\n{}",
            source(&report, name)
        );
    }

    assert!(report.text.contains("static int count(int... arg0)"));
    assert!(report.text.contains("static int plain(int[] arg0)"));
    assert!(
        report
            .text
            .contains("static int overloaded(java.lang.Object... arg0)")
    );
    assert!(
        report
            .text
            .contains("static int overloaded(java.lang.String arg0)")
    );
}

#[test]
fn expanded_call_retains_the_array_and_element_instruction_origins() {
    let report = complete_source(&open());
    let ordered = recovery(&report, "ordered");
    assert!(ordered.text.contains("count(mark(1), mark(2), mark(3))"));
    for bci in [1, 6, 9, 13, 16, 20, 23, 24] {
        assert!(
            !ordered.source_map.of_bci(bci).is_empty(),
            "expanded call omitted the allocation, element producer/store or invoke BCI {bci}: {:?}",
            ordered.source_map.segments()
        );
    }
}

#[test]
fn bounded_and_cancelled_class_source_requests_publish_no_partial_projection() {
    let snapshot = open();
    let mut limits = budget().limits().clone();
    limits.analysis_steps = 1;
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits),
        )
        .expect("the bounded request reports its stop as an operation outcome");
    let OperationOutcome::Performed(stopped) = stopped else {
        panic!("a work bound still returns its honest class-source report: {stopped:?}");
    };
    assert!(
        !stopped.methods.iter().any(|method| match &method.outcome {
            ClassSourceOutcome::Recovered { report, .. } => {
                report.text.contains("return count(mark(")
            }
            _ => false,
        }),
        "a stopped report cannot contain a half-expanded call"
    );

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::with_cancellation_token(budget().limits().clone(), cancellation),
        )
        .expect("cancellation is an operation outcome");
    assert!(
        matches!(stopped, OperationOutcome::Incomplete(_)),
        "cancellation cannot publish a partial source projection: {stopped:?}"
    );
}
