//! A call whose result is discarded immediately before `invokestatic` must not also be written as
//! that static call's expression qualifier. The permanent whole-class source/runner and Java 8
//! class are in `tests/fixtures/p3-popped-static-qualifier/`.

use jarde::*;
use std::slice;

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-popped-static-qualifier/v8/StaticQualifierProbe.class");
const OWNER_MISMATCH_FIXTURE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/static-field-qualifier/non-invoke-owner/NonInvokeQualifierProbe.owner-other.class"
);
const INTERFACE_FIXTURE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/static-field-qualifier/boundary/InterfaceBoundaryProbe.class"
);

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn source() -> ClassSourceReport {
    source_named(FIXTURE, "StaticQualifierProbe", None)
}

fn source_named(
    class_bytes: &[u8],
    class_name: &str,
    evidence: Option<&RecoveryEvidenceRequest>,
) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class_bytes.to_vec()), &mut budget())
        .expect("the committed qualifier fixture opens as a standalone CLASS");
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
    let mut run_budget = budget();
    let result = match evidence {
        Some(evidence) => Engine::new().class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            evidence,
            &mut run_budget,
        ),
        None => Engine::new().class_source(slice::from_ref(&snapshot), &request, &mut run_budget),
    }
    .expect("the fixture answers one legal class-source request");
    match result {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "the standalone class states one definition, got {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "the standalone class states one definition, got {} unfinished candidates",
            candidates.candidates.len()
        ),
    }
}

fn method_text<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no `{name}` method in the class-source report"))
        .text
}

fn method_report<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no `{name}` method in the class-source report"))
        .outcome
    {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` did not reach recovery: {other:?}"),
    }
}

#[test]
fn an_invocation_discarded_before_static_call_is_written_once() {
    let report = source();
    let write = method_text(&report, "write");

    assert_eq!(
        write.matches("receiver()").count(),
        1,
        "the call before `pop` is already the discard statement; embedding it again as the RHS call qualifier duplicates its effect:\n{write}"
    );
    assert_eq!(
        write.matches("rhs()").count(),
        1,
        "the static RHS call remains one invocation:\n{write}"
    );
    assert!(
        write.contains("StaticQualifierProbe.value = rhs();"),
        "the discarded call precedes an owner-correct static RHS call:\n{write}"
    );
}

#[test]
fn static_read_and_compound_store_controls_still_evaluate_the_receiver_once() {
    let report = source();
    for name in ["read", "writeSum"] {
        let body = method_text(&report, name);
        assert_eq!(
            body.matches("receiver()").count(),
            1,
            "`{name}` receiver count:\n{body}"
        );
    }
    let write_sum = method_text(&report, "writeSum");
    assert_eq!(write_sum.matches("rhs()").count(), 1, "{write_sum}");
}

#[test]
fn a_popped_invocation_before_interface_static_call_is_an_independent_statement() {
    let report = source_named(INTERFACE_FIXTURE, "InterfaceBoundaryProbe", None);
    let call = method_text(&report, "call");

    assert_eq!(call.matches("receiver()").count(), 1, "{call}");
    assert!(
        call.contains("receiver();") && call.contains("InterfaceStaticOwner.value();"),
        "the producer statement precedes the static interface call by owner name:\n{call}"
    );
    assert!(
        !call.contains("receiver().value()") && !call.contains("@bytecode"),
        "a static interface method is never instance-qualified:\n{call}"
    );
}

#[test]
fn an_unmatched_non_invoke_qualifier_is_refused_with_its_full_source_chain() {
    let default = source_named(OWNER_MISMATCH_FIXTURE, "NonInvokeQualifierProbe", None);
    let all = source_named(
        OWNER_MISMATCH_FIXTURE,
        "NonInvokeQualifierProbe",
        Some(&RecoveryEvidenceRequest::all()),
    );
    let default_text = method_text(&default, "call");
    let all_text = method_text(&all, "call");
    assert_eq!(default_text, all_text, "source-map selection changed text");
    assert!(
        all_text.contains("@bytecode") && !all_text.contains("arg0.ping()"),
        "the Other Methodref must not rebind through Child.ping:\n{all_text}"
    );

    let recovery = method_report(&all, "call");
    assert_eq!(
        recovery.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete,
        "all evidence requests a complete map"
    );
    assert!(
        method_report(&default, "call").source_map.is_empty(),
        "the default request does not materialize optional source maps"
    );
    for bci in [0, 1, 2, 5] {
        assert!(
            !recovery.source_map.of_bci(bci).is_empty(),
            "the refusal must retain producer, pop, static call and final consumer BCI {bci}:\n{all_text}"
        );
    }
}
