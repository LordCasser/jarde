//! A completed static field assignment before a typed catch is an ordinary prefix statement.
use jarde::*;
use std::slice;

const SAMPLE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/catch-after-field-store/CatchAfterFieldAssignment.class"
);

fn budget() -> Budget {
    task_budget(&[]).expect("bounded task budget")
}

fn class_source(bytes: &[u8]) -> ClassSourceReport {
    let sample = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("frozen Java 8 class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("CatchAfterFieldAssignment"),
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
    let OperationOutcome::Performed(source) = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&sample),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("class source is available")
    else {
        panic!("frozen class must select one definition");
    };
    source
}

#[test]
fn a_complete_field_assignment_precedes_the_named_catch() {
    let source = class_source(SAMPLE);
    for (name, prefix, assigned) in [
        (
            "fieldAssignment",
            "CatchAfterFieldAssignment.field = 7;",
            "CatchAfterFieldAssignment.field = 18;",
        ),
        ("localAssignment", "local1 = 7;", "local1 = 18;"),
    ] {
        let method = source
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name.as_bytes())
            .unwrap_or_else(|| panic!("missing {name}"));
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("{name} did not recover: {:?}", method.outcome);
        };
        let text = &method.text;
        let prefix_at = text
            .find(prefix)
            .unwrap_or_else(|| panic!("missing prefix: {text}"));
        let try_at = text
            .find("try {")
            .unwrap_or_else(|| panic!("missing try: {text}"));
        let catch_at = text
            .find("catch (java.lang.IllegalArgumentException")
            .unwrap_or_else(|| panic!("missing catch: {text}"));
        let assigned_at = text
            .find(assigned)
            .unwrap_or_else(|| panic!("missing catch assignment: {text}"));
        assert!(
            prefix_at < try_at && try_at < catch_at && catch_at < assigned_at,
            "{text}"
        );
        assert_eq!(text.matches(prefix).count(), 1, "{text}");
        assert!(
            !text.contains("@bytecode") && !text.contains("jre_guard_resource_init"),
            "{text}"
        );
        assert_eq!(
            report.content,
            RecoveryContent::ContainsStatements,
            "{text}"
        );
        if name == "fieldAssignment" {
            assert!(
                report
                    .source_map
                    .text_of_bci(&report.text, 2)
                    .iter()
                    .any(|part| part.contains("field = 7")),
                "{:?}",
                report.source_map.segments()
            );
            assert!(
                report
                    .source_map
                    .text_of_bci(&report.text, 12)
                    .iter()
                    .any(|part| part.contains("catch (java.lang.IllegalArgumentException")),
                "{:?}",
                report.source_map.segments()
            );
        }
    }
}

#[test]
fn a_range_beginning_inside_the_field_assignment_is_not_a_plain_catch() {
    // The field method's row is [5, 9) -> 12. Moving its start to BCI 2 protects the
    // `putstatic` itself, so the field assignment no longer ends before the range.
    let mut changed = SAMPLE.to_vec();
    let original = [0, 5, 0, 9, 0, 12];
    let offsets: Vec<_> = changed
        .windows(original.len())
        .enumerate()
        .filter_map(|(at, bytes)| (bytes == original).then_some(at))
        .collect();
    assert_eq!(
        offsets.len(),
        1,
        "one frozen exception row has this contour"
    );
    changed[offsets[0] + 1] = 2;
    let source = class_source(&changed);
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"fieldAssignment")
        .expect("field method remains selected");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!(
            "field method must retain a diagnostic: {:?}",
            method.outcome
        );
    };
    assert!(
        report.fallbacks.contains(&"jre_guard_resource_init"),
        "{:?}\n{}",
        report.fallbacks,
        report.text
    );
    assert!(!method.text.contains("catch ("), "{}", method.text);
}
