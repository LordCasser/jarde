// Draft additions for applied-permanent-tests-root-v2.rs. These assertions are based on the
// actual proof-trace raw in proof-trace-root-v1 and use its debug/no-debug class inputs.
use jarde_java::{RecoveryContent, RecoveryOutcome};
use jarde_jvm::ir::Quality;

const TYPED_BOUNDARY_DEBUG: &[u8] = include_bytes!(
    "../../../openspec/changes/recover-proved-local-source-types/results/boundary-preflight-root-v1/javac23-debug/classes/LocalSourceTypesBoundaries.class"
);
const TYPED_BOUNDARY_NODEBUG: &[u8] = include_bytes!(
    "../../../openspec/changes/recover-proved-local-source-types/results/boundary-preflight-root-v1/javac23-no-debug/classes/LocalSourceTypesBoundaries.class"
);
const TYPED_BOUNDARY_OWNER: &str = "LocalSourceTypesBoundaries";

fn assert_typed_boundary_structured(
    class: &[u8],
    name: &str,
    descriptor: &str,
    expected_declaration: &str,
    forbidden_declaration: Option<&str>,
) -> String {
    let (report, method) =
        assert_default_equals_all(class, TYPED_BOUNDARY_OWNER, name, descriptor);
    assert_eq!(report.quality, Quality::Structured, "{name}: {}", report.text);
    assert!(report.fallbacks.is_empty(), "{name}: {:?}", report.fallbacks);
    assert!(report.text.contains(expected_declaration), "{name}: {}", report.text);
    if let Some(forbidden) = forbidden_declaration {
        assert!(!report.text.contains(forbidden), "{name}: {}", report.text);
    }
    report.text
}

#[test]
fn typed_boundary_four_char_seeds_have_specific_types_and_full_origins_in_both_debug_profiles() {
    let fixtures = [
        ("javac23-debug", TYPED_BOUNDARY_DEBUG),
        ("javac23-no-debug", TYPED_BOUNDARY_NODEBUG),
    ];
    let cases = [
        ("charCallAndLiteralWrites", "(IZZ)Ljava/lang/String;", "char value;", "char local3;"),
        ("charFieldSeed", "()Ljava/lang/String;", "char value =", "char local0 ="),
        ("charI2cSeed", "(I)I", "char value =", "char local1 ="),
        ("charEntryParameterSeed", "(CZ)Ljava/lang/String;", "char value;", "char local2;"),
    ];
    for (profile, class) in fixtures {
        for (name, descriptor, debug_declaration, nodebug_declaration) in cases {
            let expected = if profile == "javac23-debug" { debug_declaration } else { nodebug_declaration };
            assert_typed_boundary_structured(class, name, descriptor, expected, None);
        }
    }
}

#[test]
fn typed_boundary_range_arithmetic_and_input_copy_remain_int_in_both_debug_profiles() {
    let fixtures = [TYPED_BOUNDARY_DEBUG, TYPED_BOUNDARY_NODEBUG];
    let cases = [
        ("intWithOutOfRangeWrites", "(II)I", "int value;", "int local2;"),
        ("intWithArithmeticWrite", "(Z)I", "int value;", "int local1;"),
        ("intWithUnknownCopyMerge", "(ZI)I", "int value;", "int local3;"),
    ];
    for (profile_index, class) in fixtures.into_iter().enumerate() {
        for (name, descriptor, debug_declaration, nodebug_declaration) in cases {
            let expected = if profile_index == 0 { debug_declaration } else { nodebug_declaration };
            let text = assert_typed_boundary_structured(class, name, descriptor, expected, None);
            assert!(!text.contains("char value"), "{name}: {text}");
            assert!(!text.contains("char local"), "{name}: {text}");
        }
    }
}

#[test]
fn typed_boundary_null_first_exact_string_stays_string_in_both_debug_profiles() {
    for (profile_index, class) in [TYPED_BOUNDARY_DEBUG, TYPED_BOUNDARY_NODEBUG]
        .into_iter()
        .enumerate()
    {
        let expected = if profile_index == 0 { "java.lang.String value;" } else { "java.lang.String local1;" };
        assert_typed_boundary_structured(
            class,
            "exactStringWritesAfterNull",
            "(I)V",
            expected,
            Some("Object value;"),
        );
    }
}

#[test]
fn typed_boundary_mixed_all_null_and_unknown_reference_stay_object_in_both_debug_profiles() {
    let cases = [
        ("mixedReferenceWrites", "(Z)V", "Object value;", "Object local1;"),
        ("allNullWrites", "(Z)V", "Object value;", "Object local1;"),
        ("unknownReferenceCopy", "(ZLjava/lang/Object;)V", "Object value;", "Object local2;"),
    ];
    for (profile_index, class) in [TYPED_BOUNDARY_DEBUG, TYPED_BOUNDARY_NODEBUG]
        .into_iter()
        .enumerate()
    {
        for (name, descriptor, debug_declaration, nodebug_declaration) in cases {
            let expected = if profile_index == 0 { debug_declaration } else { nodebug_declaration };
            let text = assert_typed_boundary_structured(class, name, descriptor, expected, None);
            assert!(!text.contains("String value;"), "{name}: {text}");
            assert!(!text.contains("String local"), "{name}: {text}");
        }
    }
}

fn missing_physical_bcis(
    report: &jarde_java::RecoveryReport,
    method: &PhysicalMethodId,
    bcis: &[u32],
) -> Vec<u32> {
    bcis.iter()
        .copied()
        .filter(|bci| {
            !report.source_map.of_bci(*bci).iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| origin.bci() == *bci && origin.method() == Some(method))
            })
        })
        .collect()
}

fn recover_slot_reuse(
    class: &[u8],
    evidence: RecoveryEvidenceRequest,
) -> (jarde_java::RecoveryReport, PhysicalMethodId) {
    recover_class_method_with_budget(
        class,
        TYPED_BOUNDARY_OWNER,
        "possibleSlotReuse",
        "(Z)Ljava/lang/String;",
        evidence,
        None,
    )
}

#[test]
fn typed_boundary_real_slot_conflict_is_preserved_as_the_exact_fallback_in_default_and_all() {
    let fixtures = [
        (TYPED_BOUNDARY_DEBUG, "value"),
        (TYPED_BOUNDARY_NODEBUG, "local2"),
    ];
    let conflict = "local 2 is treated as one source variable, but BCI 10 writes `int` and BCI 17 writes `Object`; no Java declaration can hold both, so this region is refused instead of publishing a contradictory local";
    let expected_missing = [0, 1, 4, 6, 9, 16, 22, 28, 31, 32, 34];
    for (class, local_name) in fixtures {
        let (default, method) = recover_slot_reuse(
            class,
            RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        );
        let (all, all_method) = recover_slot_reuse(class, RecoveryEvidenceRequest::all());
        assert_eq!(method, all_method);
        assert_eq!(default.quality, Quality::Fallback);
        assert_eq!(all.quality, Quality::Fallback);
        assert_eq!(default.outcome, RecoveryOutcome::Produced);
        assert_eq!(all.outcome, RecoveryOutcome::Produced);
        assert_eq!(default.text, all.text);
        assert_eq!(default.source_map, all.source_map);
        assert_eq!(default.fallbacks, all.fallbacks);
        assert!(default.text.contains(conflict), "{}", default.text);
        let missing_explanation = format!(
            "the statement at BCI 61 reads `{local_name}`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)"
        );
        assert!(default.text.contains(&missing_explanation), "{}", default.text);
        assert!(default.text.contains("java.lang.String number;"), "{}", default.text);
        assert!(!default.text.contains("int value;"), "{}", default.text);
        assert!(!default.text.contains("String value;"), "{}", default.text);
        assert!(!default.text.contains("char value;"), "{}", default.text);
        assert!(!default.text.contains("value ="), "{}", default.text);
        assert!(!default.text.contains("local2 ="), "{}", default.text);
        assert_eq!(
            missing_physical_bcis(
                &default,
                &method,
                &method_bcis(class, "possibleSlotReuse", "(Z)Ljava/lang/String;"),
            ),
            expected_missing,
            "{}",
            default.text
        );
    }
}

fn assert_typed_boundary_budget_stop(
    class: &[u8],
    name: &str,
    descriptor: &str,
    limit: u64,
    at: u32,
) {
    let (report, _) = recover_class_method_with_budget(
        class,
        TYPED_BOUNDARY_OWNER,
        name,
        descriptor,
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(Limits { analysis_steps: limit, ..limits() })),
    );
    assert!(matches!(
        &report.outcome,
        RecoveryOutcome::Stopped(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            written: 0,
            limit: observed_limit,
            at: Some(observed_at),
        }) if *observed_limit == limit && *observed_at == at
    ));
    assert_eq!(report.content, RecoveryContent::NotProduced);
    assert!(report.artifact.binding().is_none());
    assert!(report.text.is_empty());
    assert!(report.source_map.is_empty());
}

#[test]
fn typed_boundary_proof_budget_stops_before_publication_at_observed_sites() {
    assert_typed_boundary_budget_stop(
        TYPED_BOUNDARY_DEBUG,
        "charCallAndLiteralWrites",
        "(IZZ)Ljava/lang/String;",
        321,
        29,
    );
    assert_typed_boundary_budget_stop(
        TYPED_BOUNDARY_DEBUG,
        "exactStringWritesAfterNull",
        "(I)V",
        94,
        1,
    );
}
