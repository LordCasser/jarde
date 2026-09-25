//! The frozen shift slice exercises all six opcodes and Java's width/grouping rules.

use jarde::*;
use std::slice;

const SLICE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/shift-root-core-slice/original/ShiftSlice.class"
);
const WIDTH: &[u8] =
    include_bytes!("../openspec/evidence/java-syntax-2026-09-24/shift-width-cast/ShiftWidth.class");

fn source(bytes: &[u8], class: &str) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded task budget");
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
        .class_source(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("class-source returns an outcome")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("frozen class did not produce source: {other:?}"),
    }
}

fn source_with_evidence(
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
        .expect("class-source returns an outcome")
}

fn method<'a>(report: &'a ClassSourceReport, name: &[u8]) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name)
        .expect("frozen method is present")
}

fn recovered(method: &ClassSourceMethod) -> &RecoveryReport {
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("frozen method was not recovered: {}", method.text);
    };
    report
}

fn method_text<'a>(report: &'a ClassSourceReport, name: &[u8], descriptor: &[u8]) -> &'a str {
    let method = report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name && method.item.descriptor.raw().0 == descriptor
        })
        .expect("frozen method is present");
    let ClassSourceOutcome::Recovered {
        report: recovery, ..
    } = &method.outcome
    else {
        panic!("frozen method was not recovered: {}", method.text);
    };
    assert_eq!(recovery.representation, Representation::Java);
    assert_eq!(recovery.quality, Quality::Structured);
    assert!(!method.text.contains("@bytecode"), "{}", method.text);
    &method.text
}

#[test]
fn all_shift_directions_narrow_operands_and_call_operands_recover() {
    let report = source(SLICE, "ShiftSlice");
    for (name, descriptor, text) in [
        (&b"intLeft"[..], &b"(II)I"[..], "return arg0 << arg1;"),
        (b"intRight", b"(II)I", "return arg0 >> arg1;"),
        (b"intUnsigned", b"(II)I", "return arg0 >>> arg1;"),
        (b"longLeft", b"(JI)J", "return arg0 << arg2;"),
        (b"longRight", b"(JI)J", "return arg0 >> arg2;"),
        (b"longUnsigned", b"(JI)J", "return arg0 >>> arg2;"),
        (b"shortLeft", b"(SI)I", "return arg0 << arg1;"),
        (b"charUnsigned", b"(CI)I", "return arg0 >>> arg1;"),
        (b"nested", b"(III)I", "return arg0 << arg1 >>> arg2;"),
        (
            b"callTarget",
            b"(II)I",
            "return ShiftSliceHelper.value(arg0) << ShiftSliceHelper.distance(arg1);",
        ),
    ] {
        assert!(method_text(&report, name, descriptor).contains(text));
    }
}

#[test]
fn widening_cast_remains_on_the_left_of_the_shift() {
    let report = source(WIDTH, "ShiftWidth");
    assert!(method_text(&report, b"wide", b"(I)J").contains("return (long) arg0 << 32;"));
    assert!(method_text(&report, b"narrow", b"(I)J").contains("return (long) (arg0 << 32);"));
    assert!(method_text(&report, b"mixed", b"(II)J").contains("return (long) arg0 >>> arg1;"));
}

#[test]
fn descriptor_proved_boolean_operands_are_refused() {
    for descriptor in [b"(ZI)I", b"(IZ)I"] {
        let mut bytes = SLICE.to_vec();
        let matches = bytes
            .windows(b"(II)I".len())
            .enumerate()
            .filter_map(|(at, window)| (window == b"(II)I").then_some(at))
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1, "the frozen descriptor has one pool entry");
        bytes[matches[0]..matches[0] + descriptor.len()].copy_from_slice(descriptor);
        let report = source(&bytes, "ShiftSlice");
        let method = report
            .methods
            .iter()
            .find(|method| {
                method.item.name.raw().0 == b"intLeft"
                    && method.item.descriptor.raw().0 == descriptor
            })
            .expect("patched method is present");
        assert!(method.text.contains("@bytecode"), "{}", method.text);
        assert!(
            !method.text.contains("return arg0 << arg1;"),
            "{}",
            method.text
        );
    }
}

#[test]
fn evidence_selection_keeps_shift_text_and_physical_origins() {
    let mut essential_budget = task_budget(&[]).expect("task budget");
    let OperationOutcome::Performed(essential) = source_with_evidence(
        SLICE,
        "ShiftSlice",
        &RecoveryEvidenceRequest::essential(),
        &mut essential_budget,
    ) else {
        panic!("essential source completes")
    };
    let mut all_budget = task_budget(&[]).expect("task budget");
    let OperationOutcome::Performed(all) = source_with_evidence(
        SLICE,
        "ShiftSlice",
        &RecoveryEvidenceRequest::all(),
        &mut all_budget,
    ) else {
        panic!("full evidence source completes")
    };
    let essential_method = method(&essential, b"callTarget");
    let all_method = method(&all, b"callTarget");
    assert_eq!(essential_method.text, all_method.text);
    assert!(recovered(essential_method).source_map.is_empty());
    let report = recovered(all_method);
    assert_eq!(
        report.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    );
    for bci in [1, 5, 8] {
        let segments = report.source_map.of_bci(bci);
        assert!(!segments.is_empty(), "BCI {bci} has no source anchor");
        assert!(
            segments.iter().any(|segment| {
                segment.origin().primary().method() == Some(&all_method.item.identity)
            }),
            "BCI {bci} is not attributed to callTarget's physical member"
        );
    }
    let shift_text = report.text_of_bci(8);
    assert!(
        shift_text.iter().any(|text| text.contains("<<")),
        "{shift_text:?}"
    );
    assert_eq!(
        all_method
            .text
            .matches("ShiftSliceHelper.value(arg0)")
            .count(),
        1
    );
    assert_eq!(
        all_method
            .text
            .matches("ShiftSliceHelper.distance(arg1)")
            .count(),
        1
    );
}

#[test]
fn shift_recovery_stops_cleanly_at_resource_limits_and_before_work() {
    let complete = {
        let mut budget = task_budget(&[]).expect("task budget");
        let OperationOutcome::Performed(report) = source_with_evidence(
            SLICE,
            "ShiftSlice",
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        ) else {
            panic!("baseline source completes")
        };
        report
    };

    let mut source_limits = complete.limits.clone();
    source_limits.ir_items = complete.usage.ir_items.saturating_sub(1);
    let mut source_budget = Budget::new(source_limits);
    let OperationOutcome::Performed(source_stopped) = source_with_evidence(
        SLICE,
        "ShiftSlice",
        &RecoveryEvidenceRequest::all(),
        &mut source_budget,
    ) else {
        panic!("source-map exhaustion is reported with class source")
    };
    assert!(
        source_stopped.methods.iter().any(|method| {
            matches!(
                &method.outcome,
                ClassSourceOutcome::Recovered { report, .. }
                    if matches!(
                        report.evidence.state(RecoveryEvidenceKind::SourceMap),
                        EvidenceState::Partial { delivered } if delivered > 0
                    )
            )
        }),
        "source exhaustion must retain its partial source-map state"
    );

    let mut ir_limits = complete.limits.clone();
    ir_limits.ir_items = 1_000;
    let mut ir_budget = Budget::new(ir_limits);
    let OperationOutcome::Performed(ir_stopped) = source_with_evidence(
        SLICE,
        "ShiftSlice",
        &RecoveryEvidenceRequest::all(),
        &mut ir_budget,
    ) else {
        panic!("IR exhaustion is reported with class source")
    };
    assert!(matches!(
        &method(&ir_stopped, b"callTarget").outcome,
        ClassSourceOutcome::Recovered { report, analysis }
            if report.text.is_empty()
                && matches!(report.outcome, RecoveryOutcome::Stopped(_))
                && matches!(analysis.execution, ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded { dimension: BudgetDimension::IrItems }, ..
                })
    ));

    let mut work_limits = complete.limits.clone();
    work_limits.analysis_steps = 1;
    let mut work_budget = Budget::new(work_limits);
    let OperationOutcome::Performed(work_stopped) = source_with_evidence(
        SLICE,
        "ShiftSlice",
        &RecoveryEvidenceRequest::all(),
        &mut work_budget,
    ) else {
        panic!("analysis exhaustion is reported with class source")
    };
    assert!(matches!(
        work_stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps
            },
            ..
        }
    ));

    let mut output_limits = complete.limits.clone();
    output_limits.output_bytes = 1;
    let mut output_budget = Budget::new(output_limits);
    assert!(matches!(
        source_with_evidence(
            SLICE,
            "ShiftSlice",
            &RecoveryEvidenceRequest::all(),
            &mut output_budget,
        ),
        OperationOutcome::Incomplete(_)
    ));

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(complete.limits.clone(), token);
    assert!(matches!(
        source_with_evidence(
            SLICE,
            "ShiftSlice",
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        ),
        OperationOutcome::Incomplete(_)
    ));
}
