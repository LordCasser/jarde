use jarde::*;

const FIXED: &[u8] = include_bytes!("fixtures/cf11-structured-list-api/UpstreamNestedLoops.class");
const DISTINCT_STRING: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-api/DistinctStringFixture.class");
const DISTINCT_INTEGER: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-api/DistinctIntegerFixture.class");
const CONTROLS: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-controls/UpstreamNestedLoops.class");

const INNER_OPTION_INTEGER: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-inner-option/DistinctIntegerFixture.class");

const NULL_STRING: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-null-api/NullStringListFixture.class");
const NULL_INTEGER: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-null-api/NullIntegerListFixture.class");

fn request(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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

fn report_with(
    bytes: &[u8],
    name: &str,
    evidence: Option<&RecoveryEvidenceRequest>,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = request(&snapshot, name);
    match evidence {
        Some(evidence) => engine.class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            evidence,
            budget,
        ),
        None => engine.class_source(std::slice::from_ref(&snapshot), &request, budget),
    }
    .unwrap()
}

fn performed(bytes: &[u8], name: &str) -> ClassSourceReport {
    match report_with(bytes, name, None, &mut task_budget(&[]).unwrap()) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source result: {other:?}"),
    }
}

fn method<'a>(report: &'a ClassSourceReport, method_name: &[u8]) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == method_name)
        .unwrap_or_else(|| panic!("missing method {:?}", String::from_utf8_lossy(method_name)))
}

fn raw_body<'a>(method: &'a ClassSourceMethod) -> &'a RecoveryReport {
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!(
            "method {:?} did not recover: {other:?}",
            method.item.name.raw().0
        ),
    }
}

fn assert_utf8_source_map(report: &RecoveryReport) {
    assert!(!report.source_map.is_empty(), "source map is present");
    for segment in report.source_map.segments() {
        assert!(report.text.is_char_boundary(segment.start()));
        assert!(report.text.is_char_boundary(segment.end()));
        assert_eq!(
            segment.text(&report.text),
            &report.text[segment.start()..segment.end()]
        );
    }
}

fn assert_bcis_are_mapped(report: &RecoveryReport, bcis: &[u32]) {
    for bci in bcis {
        assert!(
            !report.source_map.of_bci(*bci).is_empty(),
            "physical BCI {bci} has source-map provenance"
        );
    }
}

fn assert_generic_refusal(method: &ClassSourceMethod, name: &str, descriptor: &str) {
    let marker = format!(
        "generic Signature projection refused for `{name}{descriptor}`: unsupported (ordinary_generic_source_unproved):"
    );
    assert!(
        method.text.contains(&marker),
        "expected exact generic refusal marker {marker:?}: {}",
        method.text
    );
    assert!(
        matches!(&method.outcome, ClassSourceOutcome::Recovered { .. }),
        "generic refusal must retain the original raw RecoveryReport for {name}: {:?}",
        method.outcome
    );
    let raw = raw_body(method);
    assert_utf8_source_map(raw);
    assert!(
        !raw.text.is_empty(),
        "the original raw RecoveryReport is retained"
    );
}

#[test]
fn nearby_list_controls_refuse_only_unproved_generic_projections() {
    let report = performed(CONTROLS, "cf11/UpstreamNestedLoops");

    // These are distinct source-proof failures around the admitted CF11 body. Check only that
    // the generic layer refuses and the original raw RecoveryReport plus map remain available;
    // this does not claim the recovered presentation is a compilable complete source file.
    for (name, descriptor) in [
        ("rawAliasEscape", "(Ljava/util/List;Ljava/util/List;)V"),
        ("reassigned", "(Ljava/util/List;Ljava/util/List;)V"),
        ("wildcardNull", "(Ljava/util/List;Ljava/util/List;)V"),
        (
            "indexOverload",
            "(Ljava/util/List;Ljava/util/List;Ljava/lang/Integer;I)V",
        ),
    ] {
        assert_generic_refusal(method(&report, name.as_bytes()), name, descriptor);
    }

    // A raw-signature method is a raw path, not evidence of a generic refusal.
    let raw = method(&report, b"rawLists");
    assert!(
        !raw.text.contains("generic Signature projection refused"),
        "raw parameters remain on their existing raw path: {}",
        raw.text
    );

    // Keep the physical cast boundary independent of the generic refusal controls. Its raw
    // recovery artifact must still mention the real String conversions and source map.
    let casts = method(&report, b"castBoundary");
    let raw_casts = raw_body(casts);
    assert!(raw_casts.text.contains("java.lang.String") || raw_casts.text.contains("String"));
    assert_utf8_source_map(raw_casts);
    assert_bcis_are_mapped(raw_casts, &[26, 60]);

    // Handler presence independently refuses the List projection. Pin the generic refusal and
    // absence of a generic declaration, without locking in the current body fallback behavior.
    let handled = method(&report, b"handled");
    assert!(handled.text.contains(
        "generic Signature projection refused for `handled(Ljava/util/List;Ljava/util/List;)V`: unsupported (ordinary_generic_source_unproved):"
    ), "{}", handled.text);
    assert!(
        !handled
            .text
            .contains("handled(java.util.List<java.lang.String>"),
        "handler control must not publish an unproved List<String> declaration: {}",
        handled.text
    );
}

#[test]
fn inner_list_option_actual_uses_the_receiver_element_type() {
    let report = performed(INNER_OPTION_INTEGER, "cf11/DistinctIntegerFixture");
    let integer_method = method(&report, b"numbers");
    let integer_raw = raw_body(integer_method);
    let declaration = "numbers(java.util.List<java.lang.Integer> selected, java.util.List<java.lang.Integer> available)";
    assert!(report.text.contains(declaration), "{}", report.text);
    assert!(
        integer_method.text.contains(declaration),
        "{}",
        integer_method.text
    );
    assert!(
        integer_method.text.contains("available.add(option)"),
        "{}",
        integer_method.text
    );
    assert!(
        !integer_method.text.contains(".add((java.lang.Object)"),
        "synthetic add(E) wrapper remains: {}",
        integer_method.text
    );
    assert!(
        integer_method
            .text
            .contains("selected.remove((java.lang.Object) option)"),
        "remove(Object) overload wrapper was removed: {}",
        integer_method.text
    );
    assert!(
        !integer_method.text.contains("selected.remove((int)"),
        "remove(int) was selected: {}",
        integer_method.text
    );
    assert_utf8_source_map(integer_raw);
    assert_bcis_are_mapped(integer_raw, &[22, 52, 76, 77, 79, 96, 97, 99, 128, 129]);
    for cast_bci in [22, 52] {
        assert!(
            integer_raw
                .source_map
                .of_bci(cast_bci)
                .iter()
                .any(|segment| {
                    segment
                        .text(&integer_raw.text)
                        .contains("java.lang.Integer")
                        || segment.text(&integer_raw.text).contains("Integer")
                }),
            "physical Integer checkcast at BCI {cast_bci} remains in raw recovery source"
        );
    }
}

#[test]
fn fixed_nested_string_body_projects_source_and_preserves_physical_origins() {
    let default = performed(FIXED, "cf11/UpstreamNestedLoops");
    let all = match report_with(
        FIXED,
        "cf11/UpstreamNestedLoops",
        Some(&RecoveryEvidenceRequest::all()),
        &mut task_budget(&[]).unwrap(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected all-evidence result: {other:?}"),
    };
    let no_rule_details_request =
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap);
    let no_rule_details = match report_with(
        FIXED,
        "cf11/UpstreamNestedLoops",
        Some(&no_rule_details_request),
        &mut task_budget(&[]).unwrap(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected no-RuleDetails result: {other:?}"),
    };

    assert_eq!(default.text, all.text);
    assert_eq!(default.text, no_rule_details.text);
    let default_method = method(&default, b"test");
    let all_method = method(&all, b"test");
    let no_rule_details_method = method(&no_rule_details, b"test");
    let default_raw = raw_body(default_method);
    let all_raw = raw_body(all_method);
    let no_rule_details_raw = raw_body(no_rule_details_method);
    assert_eq!(default_method.text, all_method.text);
    assert_eq!(default_method.text, no_rule_details_method.text);
    assert_eq!(default_raw.text, all_raw.text);
    assert_eq!(default_raw.text, no_rule_details_raw.text);
    assert_eq!(default_raw.source_map, all_raw.source_map);
    assert_eq!(default_raw.source_map, no_rule_details_raw.source_map);

    let declaration =
        "test(java.util.List<java.lang.String> l1, java.util.List<java.lang.String> l2)";
    assert!(
        default.text.contains(declaration),
        "full List<String> class declaration: {}",
        default.text
    );
    assert!(
        default_method.text.contains(declaration),
        "full List<String> method text: {}",
        default_method.text
    );
    assert!(
        default_method.text.contains("l2.add(s1)"),
        "{}",
        default_method.text
    );
    assert!(
        !default_method.text.contains(".add((java.lang.Object)"),
        "add(E) must not retain the synthetic Object argument wrapper: {}",
        default_method.text
    );
    assert!(
        default_method
            .text
            .contains("l1.remove((java.lang.Object) s2)"),
        "remove(Object) must retain the object-overload wrapper: {}",
        default_method.text
    );
    assert!(
        !default_method.text.contains("l1.remove((int)"),
        "{}",
        default_method.text
    );

    // The source map is defined over the unprojected RecoveryReport::text. It proves physical
    // provenance; it is not a map for ClassSourceMethod::text or ClassSourceReport::text.
    assert_utf8_source_map(default_raw);
    assert_bcis_are_mapped(
        default_raw,
        &[
            0, 1, 22, 27, 28, 52, 76, 77, 79, 88, 89, 91, 103, 104, 112, 113,
        ],
    );
    for cast_bci in [22, 52] {
        assert!(
            default_raw
                .source_map
                .of_bci(cast_bci)
                .iter()
                .any(|segment| {
                    segment.text(&default_raw.text).contains("java.lang.String")
                        || segment.text(&default_raw.text).contains("String")
                }),
            "real String checkcast at BCI {cast_bci} remains tied to its raw recovery source"
        );
    }
}

#[test]
fn differently_named_string_and_integer_classes_follow_their_list_element_type() {
    let string = performed(DISTINCT_STRING, "cf11/DistinctStringFixture");
    let string_method = method(&string, b"renamed");
    let string_raw = raw_body(string_method);
    let string_declaration = "renamed(java.util.List<java.lang.String> chosen, java.util.List<java.lang.String> offered)";
    assert!(string.text.contains(string_declaration), "{}", string.text);
    assert!(
        string_method.text.contains(string_declaration),
        "{}",
        string_method.text
    );
    assert!(
        string_method.text.contains("offered.add(candidate)"),
        "{}",
        string_method.text
    );
    assert!(
        !string_method.text.contains(".add((java.lang.Object)"),
        "{}",
        string_method.text
    );
    assert!(
        string_method
            .text
            .contains("chosen.remove((java.lang.Object) option)"),
        "remove(Object) is not confused with remove(int): {}",
        string_method.text
    );
    assert_utf8_source_map(string_raw);

    let integer = performed(DISTINCT_INTEGER, "cf11/DistinctIntegerFixture");
    let integer_method = method(&integer, b"numbers");
    let integer_raw = raw_body(integer_method);
    let integer_declaration = "numbers(java.util.List<java.lang.Integer> selected, java.util.List<java.lang.Integer> available)";
    assert!(
        integer.text.contains(integer_declaration),
        "{}",
        integer.text
    );
    assert!(
        integer_method.text.contains(integer_declaration),
        "{}",
        integer_method.text
    );
    assert!(
        integer_method.text.contains("available.add(value)"),
        "{}",
        integer_method.text
    );
    assert!(
        !integer_method.text.contains(".add((java.lang.Object)"),
        "{}",
        integer_method.text
    );
    assert!(
        integer_method
            .text
            .contains("selected.remove((java.lang.Object) option)"),
        "remove(Object) is not confused with remove(int): {}",
        integer_method.text
    );
    assert_utf8_source_map(integer_raw);
}

#[test]
fn exact_list_null_add_removes_only_the_synthetic_object_wrapper() {
    let string = performed(NULL_STRING, "cf11/NullStringListFixture");
    let string_method = method(&string, b"strings");
    let string_raw = raw_body(string_method);
    let string_declaration = "strings(java.util.List<java.lang.String> selected, java.util.List<java.lang.String> available)";
    assert!(string.text.contains(string_declaration), "{}", string.text);
    assert!(
        string_method.text.contains("available.add(null)"),
        "{}",
        string_method.text
    );
    assert!(
        !string_method
            .text
            .contains("available.add((java.lang.Object) null)"),
        "synthetic add(E) wrapper remains: {}",
        string_method.text
    );
    assert!(
        string_method
            .text
            .contains("selected.remove((java.lang.Object) option)"),
        "remove(Object) overload wrapper was removed: {}",
        string_method.text
    );
    assert_utf8_source_map(string_raw);

    let integer = performed(NULL_INTEGER, "cf11/NullIntegerListFixture");
    let integer_method = method(&integer, b"integers");
    let integer_raw = raw_body(integer_method);
    let integer_declaration = "integers(java.util.List<java.lang.Integer> selected, java.util.List<java.lang.Integer> available)";
    assert!(
        integer.text.contains(integer_declaration),
        "{}",
        integer.text
    );
    assert!(
        integer_method.text.contains("available.add(null)"),
        "{}",
        integer_method.text
    );
    assert!(
        !integer_method
            .text
            .contains("available.add((java.lang.Object) null)"),
        "synthetic add(E) wrapper remains: {}",
        integer_method.text
    );
    assert!(
        integer_method
            .text
            .contains("selected.remove((java.lang.Object) option)"),
        "remove(Object) overload wrapper was removed: {}",
        integer_method.text
    );
    assert_utf8_source_map(integer_raw);
}

#[test]
fn pre_cancel_does_not_install_a_partial_generic_declaration_or_method_projection() {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(FIXED.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = request(&snapshot, "cf11/UpstreamNestedLoops");
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::with_cancellation_token(task_limits(&[]).unwrap(), token),
        )
        .unwrap();
    match cancelled {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                &report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            let generic_declaration =
                "test(java.util.List<java.lang.String> l1, java.util.List<java.lang.String> l2)";
            assert!(
                !report.text.contains(generic_declaration),
                "{}",
                report.text
            );
            if let Some(method) = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"test")
            {
                assert!(
                    !method.text.contains(generic_declaration),
                    "{}",
                    method.text
                );
                // A completed raw RecoveryReport may be retained with its original source map.
                // Cancellation only forbids installing a partial generic source projection.
                if let ClassSourceOutcome::Recovered { report, .. } = &method.outcome {
                    assert_utf8_source_map(report);
                }
            }
        }
        OperationOutcome::Ambiguous(_) => panic!("frozen class must bind uniquely"),
    }
}

#[test]
fn budget_stop_during_list_proof_keeps_raw_body_and_installs_no_projection() {
    let complete = performed(FIXED, "cf11/UpstreamNestedLoops");
    let complete_method = method(&complete, b"test");
    let complete_raw = raw_body(complete_method);

    // Root's CLI probe located this inside the final List proof charge.
    // After adding declaration initializers, root v8 measured 25299 steps before that charge.
    // This is an observed proof-stage boundary, not a usage-minus-one guess.
    let mut limits = task_limits(&[]).unwrap();
    limits.analysis_steps = 32_531;
    let mut budget = Budget::new(limits);
    let partial = match report_with(FIXED, "cf11/UpstreamNestedLoops", None, &mut budget) {
        OperationOutcome::Performed(report) => report,
        other => panic!("proof-stage budget stop must keep its class report: {other:?}"),
    };
    assert!(matches!(
        &partial.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
            },
            ..
        }
    ));
    assert_eq!(partial.usage.analysis_steps, 25_299);

    let partial_method = method(&partial, b"test");
    let partial_raw = raw_body(partial_method);
    assert_eq!(partial_raw.text, complete_raw.text);
    assert_eq!(partial_raw.source_map, complete_raw.source_map);
    assert_utf8_source_map(partial_raw);

    let generic_declaration =
        "test(java.util.List<java.lang.String> l1, java.util.List<java.lang.String> l2)";
    assert!(
        !partial.text.contains(generic_declaration),
        "{}",
        partial.text
    );
    assert!(
        !partial_method.text.contains(generic_declaration),
        "{}",
        partial_method.text
    );
    assert!(
        partial_method
            .text
            .contains("l2.add((java.lang.Object) s1)"),
        "the raw method presentation keeps its original add argument cast: {}",
        partial_method.text
    );
}

/// Root-only probe for locating the exact newly added projection stop after the production change.
/// Its output is diagnostic; it deliberately makes no claim that usage - 1 reaches that phase.
#[test]
#[ignore = "root diagnostic: inspect observed stop phase and method outcome before pinning lateStop"]
fn diagnose_late_analysis_stop_phase_without_assuming_raw_map_is_empty() {
    let complete = performed(FIXED, "cf11/UpstreamNestedLoops");
    eprintln!("complete analysis_steps={}", complete.usage.analysis_steps);
    for distance in [1_u64, 2, 4, 8, 16, 32, 64] {
        if complete.usage.analysis_steps <= distance {
            continue;
        }
        let limit = complete.usage.analysis_steps - distance;
        let mut limits = task_limits(&[]).unwrap();
        limits.analysis_steps = limit;
        let outcome = report_with(
            FIXED,
            "cf11/UpstreamNestedLoops",
            None,
            &mut Budget::new(limits),
        );
        match outcome {
            OperationOutcome::Performed(report) => {
                eprintln!(
                    "limit={limit} usage={} execution={:?}",
                    report.usage.analysis_steps, report.execution
                );
                eprintln!("class text={}", report.text);
                if let Some(method) = report
                    .methods
                    .iter()
                    .find(|method| method.item.name.raw().0 == b"test")
                {
                    eprintln!(
                        "method declaration={:?} text={}",
                        method.declaration, method.text
                    );
                    match &method.outcome {
                        ClassSourceOutcome::Recovered { report, .. } => eprintln!(
                            "raw outcome=recovered raw_text={} raw_source_map_bcis_79={:?}",
                            report.text,
                            report
                                .source_map
                                .of_bci(79)
                                .iter()
                                .map(|segment| segment.text(&report.text))
                                .collect::<Vec<_>>()
                        ),
                        other => eprintln!("raw/member outcome={other:?}"),
                    }
                }
            }
            other => eprintln!("limit={limit} outcome={other:?}"),
        }
    }
}
