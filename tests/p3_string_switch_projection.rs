//! The 2.1 certificate is the only admission to the two-region String projection.

use jarde::*;
use std::slice;

const BASIC: &[u8] = include_bytes!("fixtures/p3-string-switch/v8/StringSwitchProbe.class");
const UNICODE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/string-switch-unicode/StringSwitchUnicode.class"
);
const MIDDLE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/string-switch-variants/StringSwitchMiddleDefault.class"
);
const HASH_USE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/string-switch-negative-fixtures/classes/ExtraHashUse.class"
);
const WRONG_BUCKET: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/string-switch-negative-fixtures/classes/WrongHashBucket.class"
);
const BUCKET_EFFECT: &[u8] = include_bytes!("fixtures/p3-string-switch/v8/ExtraBucketEffect.class");

fn choose(bytes: &[u8], class: &str) -> String {
    let mut budget = task_budget(&[]).expect("bounded task defaults");
    class_source(
        bytes,
        class,
        &RecoveryEvidenceRequest::essential(),
        &mut budget,
    )
    .methods
    .iter()
    .find(|method| method.item.name.raw().0 == b"choose")
    .expect("choose member")
    .text
    .clone()
}

fn class_source(
    bytes: &[u8],
    class: &str,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> ClassSourceReport {
    let OperationOutcome::Performed(report) = class_source_outcome(bytes, class, evidence, budget)
    else {
        panic!("class source request completed")
    };
    report
}

fn class_source_outcome(
    bytes: &[u8],
    class: &str,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).expect("open budget"),
        )
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
    engine
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, evidence, budget)
        .expect("class source request")
}

fn choose_method(report: &ClassSourceReport) -> &ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"choose")
        .expect("choose member")
}

#[test]
fn proved_dispatches_use_one_string_switch_and_original_arm_order() {
    let basic = choose(BASIC, "StringSwitchProbe");
    assert_eq!(basic.matches("switch (").count(), 1, "{basic}");
    for label in ["case \"Aa\":", "case \"BB\":", "case \"z\":", "default:"] {
        assert!(basic.contains(label), "{basic}");
    }
    assert!(
        !basic.contains("hashCode") && !basic.contains(".equals("),
        "{basic}"
    );

    let unicode = choose(UNICODE, "StringSwitchUnicode");
    assert!(unicode.contains("switch (selector(arg0))"), "{unicode}");
    for label in ["case \"\":", "case \"雪\":", "case \"𐐷\":", "default:"] {
        assert!(unicode.contains(label), "{unicode}");
    }
    assert_eq!(unicode.matches("selector(arg0)").count(), 1, "{unicode}");

    let middle = choose(MIDDLE, "StringSwitchMiddleDefault");
    assert!(middle.contains("switch (read(arg0))"), "{middle}");
    assert_eq!(middle.matches("switch (").count(), 1, "{middle}");
    let default = middle.find("default:").expect("default arm");
    let empty = middle.find("case \"\":").expect("empty arm");
    assert!(
        default < empty && !middle[default..empty].contains("break;"),
        "{middle}"
    );
    assert!(
        !middle.contains("case 2:"),
        "unreachable integer hole: {middle}"
    );
}

#[test]
fn refused_certificates_preserve_both_executable_dispatches() {
    for (bytes, class) in [
        (HASH_USE, "ExtraHashUse"),
        (WRONG_BUCKET, "WrongHashBucket"),
        (BUCKET_EFFECT, "ExtraBucketEffect"),
    ] {
        let text = choose(bytes, class);
        assert_eq!(text.matches("switch (").count(), 2, "{class}: {text}");
        assert!(
            text.contains("hashCode()") && text.contains(".equals("),
            "{class}: {text}"
        );
        assert!(!text.contains("switch (arg0)"), "{class}: {text}");
    }
}

#[test]
fn essential_and_all_evidence_emit_identical_string_switch_text() {
    let mut essential_budget = task_budget(&[]).expect("task budget");
    let essential = class_source(
        BASIC,
        "StringSwitchProbe",
        &RecoveryEvidenceRequest::essential(),
        &mut essential_budget,
    );
    let mut all_budget = task_budget(&[]).expect("task budget");
    let all = class_source(
        BASIC,
        "StringSwitchProbe",
        &RecoveryEvidenceRequest::all(),
        &mut all_budget,
    );
    assert_eq!(choose_method(&essential).text, choose_method(&all).text);
    assert!(recovery(choose_method(&essential)).source_map.is_empty());
    assert!(!recovery(choose_method(&all)).source_map.is_empty());
    assert_eq!(choose_method(&all).text.matches("switch (").count(), 1);
}

#[test]
fn all_evidence_maps_each_folded_dispatch_instruction_and_selector_producer() {
    let mut budget = task_budget(&[]).expect("task budget");
    let report = class_source(
        BASIC,
        "StringSwitchProbe",
        &RecoveryEvidenceRequest::all(),
        &mut budget,
    );
    let method = choose_method(&report);
    // javap -c on the frozen choose method: selector load at 0, saved selector at 1, the
    // discriminator initialization and hash dispatch at 2..8, bucket comparisons/writes and join
    // at 36..75, followed by the final integer switch at 76. All of these instructions disappear
    // into the one Java String switch and therefore need an emitted source anchor.
    let folded_bcis = [
        0, 1, 2, 3, 4, 5, 8, 36, 37, 39, 42, 45, 46, 47, 50, 51, 53, 56, 59, 60, 61, 64, 65, 67,
        70, 73, 74, 75, 76,
    ];
    for bci in folded_bcis {
        assert!(
            !recovery(method).source_map.of_bci(bci).is_empty(),
            "folded BCI {bci} has no source-map segment"
        );
    }
    assert!(
        recovery(method)
            .source_map
            .text_of_bci(&recovery(method).text, 76)
            .iter()
            .any(|text| text.contains("switch (")),
        "the final dispatch BCI anchors the emitted switch"
    );
    assert!(
        recovery(method)
            .source_map
            .text_of_bci(&recovery(method).text, 0)
            .iter()
            .any(|text| text.contains("arg0")),
        "the selector producer anchors the selector expression: {:?}",
        recovery(method).source_map.text_of_bci(&method.text, 0)
    );
}

#[test]
fn low_ir_and_output_budgets_stop_without_committing_a_partial_switch() {
    let mut full_budget = task_budget(&[]).expect("task budget");
    let complete = class_source(
        BASIC,
        "StringSwitchProbe",
        &RecoveryEvidenceRequest::all(),
        &mut full_budget,
    );

    let mut ir_limits = complete.limits.clone();
    ir_limits.ir_items = 1_000;
    let mut ir_budget = Budget::new(ir_limits);
    let OperationOutcome::Performed(ir_stopped) = class_source_outcome(
        BASIC,
        "StringSwitchProbe",
        &RecoveryEvidenceRequest::all(),
        &mut ir_budget,
    ) else {
        panic!("IR budget stop is reported with the class source")
    };
    let ir_method = choose_method(&ir_stopped);
    assert!(matches!(
        &ir_method.outcome,
        ClassSourceOutcome::Recovered { report, analysis }
            if matches!(analysis.execution, ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded { dimension: BudgetDimension::IrItems }, ..
            })
                && report.text.is_empty()
                && matches!(report.outcome, RecoveryOutcome::Stopped(_))
    ));
    assert!(!recovery(ir_method).text.contains("switch (arg0)"));

    let mut output_limits = complete.limits.clone();
    output_limits.output_bytes = complete.usage.output_bytes.saturating_sub(1);
    let mut output_budget = Budget::new(output_limits);
    let OperationOutcome::Performed(output_stopped) = class_source_outcome(
        BASIC,
        "StringSwitchProbe",
        &RecoveryEvidenceRequest::all(),
        &mut output_budget,
    ) else {
        panic!("output budget stop is reported with the class source")
    };
    assert!(matches!(
        output_stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::OutputBytes
            },
            ..
        }
    ));
    let output_method = choose_method(&output_stopped);
    assert!(matches!(
        &output_method.outcome,
        ClassSourceOutcome::Recovered { report, .. }
            if report.text.is_empty() || report.text.matches("switch (").count() == 1
    ));
    let switch_count = recovery(output_method).text.matches("switch (").count();
    assert!(switch_count == 0 || switch_count == 1);
}

#[test]
fn cancellation_before_string_projection_is_reported_without_a_committed_switch() {
    let complete = {
        let mut budget = task_budget(&[]).expect("task budget");
        class_source(
            BASIC,
            "StringSwitchProbe",
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
    };
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(complete.limits.clone(), token);
    assert!(matches!(
        class_source_outcome(
            BASIC,
            "StringSwitchProbe",
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        ),
        OperationOutcome::Incomplete(_)
    ));
}

fn recovery(method: &ClassSourceMethod) -> &RecoveryReport {
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("choose has a recovery report")
    };
    report
}
