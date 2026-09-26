//! A named catch whose handler logs and rethrows its parameter is not javac's catch-all
//! `finally` copy. The committed Java 8 class and three-way runtime evidence live in
//! `openspec/evidence/java-syntax-2026-09-26/precise-rethrow/`.

use jarde::*;
use std::slice;

const SAMPLE: &[u8] = include_bytes!("fixtures/p3-precise-rethrow/v8/PreciseRethrowProbe.class");
const CONTROLS: &[u8] =
    include_bytes!("fixtures/p3-precise-rethrow/v8/PreciseRethrowBoundaryProbe.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn class_source(bytes: &[u8], class_name: &str) -> ClassSourceReport {
    let sample = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen Java 8 fixture opens as a class");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name),
        },
        environment: EnvironmentRequest {
            snapshot: sample.id().clone(),
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
        .class_source_with_evidence(
            slice::from_ref(&sample),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("the fixture has one class-source result")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one class fixture selected {} definitions",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one class fixture left {} unfinished candidates",
            candidates.candidates.len()
        ),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing method `{name}`"))
}

#[test]
fn a_named_single_row_rethrow_is_a_user_catch_with_entry_and_throw_origins() {
    let report = class_source(SAMPLE, "PreciseRethrowProbe");
    let method = method(&report, "precise");
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!("precise rethrow was not presented: {:?}", method.outcome);
    };
    assert!(
        method.text.contains("catch (java.lang.Exception e)"),
        "{}",
        method.text
    );
    assert!(method.text.contains("log(e);"), "{}", method.text);
    assert!(method.text.contains("throw e;"), "{}", method.text);
    assert!(
        method
            .text
            .contains("throws java.text.ParseException, java.io.IOException"),
        "{}",
        method.text
    );
    assert!(!method.text.contains("@bytecode"), "{}", method.text);
    assert!(
        !method.text.contains("jre_guard_finally_copy"),
        "{}",
        method.text
    );
    assert_eq!(
        body.content,
        RecoveryContent::ContainsStatements,
        "{}",
        method.text
    );
    assert_eq!(
        body.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    );
    for bci in [34, 40] {
        let segments = body.source_map.of_bci(bci);
        assert!(
            segments.iter().any(|segment| {
                segment.origin().primary().method() == Some(&method.item.identity)
            }),
            "precise rethrow lost its handler-entry store or athrow BCI {bci}: {:#?}",
            body.source_map.segments()
        );
    }
}

#[test]
fn catch_all_and_non_parameter_throw_controls_keep_their_own_shapes() {
    let report = class_source(CONTROLS, "PreciseRethrowBoundaryProbe");
    let finally = method(&report, "anyFinally");
    assert!(
        !finally.text.contains("catch ("),
        "a catch_type == 0 finally copy must not become a user catch:\n{}",
        finally.text
    );
    assert!(
        finally
            .text
            .contains("copy javac emits for a `finally` clause"),
        "the intentionally unsupported any-row shape keeps its refusal:\n{}",
        finally.text
    );

    let multi = method(&report, "multiCatch");
    assert!(
        multi
            .text
            .contains("catch (java.text.ParseException | java.io.IOException e)"),
        "two named rows retain their union catch type:\n{}",
        multi.text
    );
    assert!(
        multi.text.contains("@bytecode"),
        "the multi-row body remains outside the complete one-row recovery shape:\n{}",
        multi.text
    );
    assert!(multi.text.contains("throw e;"), "{}", multi.text);
    assert!(
        multi.text.contains("the parameter 0 of the invocation"),
        "the unproved log argument remains explicitly quoted:\n{}",
        multi.text
    );

    let changed = method(&report, "changedValue");
    assert!(
        changed
            .text
            .contains("throw new java.lang.IllegalStateException(\"replacement\");"),
        "{}",
        changed.text
    );
    assert!(!changed.text.contains("throw e;"), "{}", changed.text);
}
