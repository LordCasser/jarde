//! The frozen shift slice exercises all six opcodes and Java's width/grouping rules.

use jarde::*;
use std::slice;

const SLICE: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/evidence/java-syntax-2026-09-24/shift-root-core-slice/original/ShiftSlice.class"
);
const WIDTH: &[u8] = include_bytes!(
    "../tests/fixtures/recovered-inputs/evidence/java-syntax-2026-09-24/shift-width-cast/ShiftWidth.class"
);

fn source(bytes: &[u8], class: &str) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded task budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen class opens");
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
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
        source_target: jarde::SourceTarget::JAVA_8,
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

    let class_needed = complete.usage.ir_items;
    assert!(class_needed > 0);
    for limit in [class_needed, class_needed - 1] {
        let mut limits = complete.limits.clone();
        limits.ir_items = limit;
        let mut budget = Budget::new(limits);
        let OperationOutcome::Performed(report) = source_with_evidence(
            SLICE,
            "ShiftSlice",
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        ) else {
            panic!("late class IR exhaustion preserves a class report")
        };
        assert!(budget.usage().ir_items <= limit);
        if limit == class_needed {
            assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
            assert_eq!(report.text, complete.text);
            assert_eq!(budget.usage().ir_items, class_needed);
        } else {
            assert!(
                matches!(
                    report.execution,
                    ExecutionReport::Partial {
                        reason: TerminationReason::BudgetExceeded {
                            dimension: BudgetDimension::IrItems
                        },
                        ..
                    }
                ),
                "{:?}",
                report.execution
            );
        }
    }

    // Class assembly can charge after a member's source map. A class-wide N-1
    // therefore cannot identify that member's evidence or source-commit boundary.
    // Exercise those boundaries directly through the public method entry point.
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(SLICE.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let header = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut task_budget(&[]).unwrap(),
            InspectionMode::Strict,
        )
        .unwrap();
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let request = MethodAnalysisRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        },
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: header.source.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"callTarget".to_vec()),
            descriptor: JvmBytes(b"(II)I".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let recover = |evidence: &RecoveryEvidenceRequest, budget: &mut Budget| {
        engine
            .recover_method_with_evidence(
                slice::from_ref(&snapshot),
                SourceTarget::JAVA_8,
                &request,
                evidence,
                budget,
            )
            .expect("bounded method recovery returns its outcome")
    };
    for (evidence, has_map) in [
        (RecoveryEvidenceRequest::essential(), false),
        (
            RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            true,
        ),
    ] {
        let mut measured = task_budget(&[]).unwrap();
        let full = recover(&evidence, &mut measured);
        let full_report = full.recovery();
        assert!(full_report.produced());
        assert!(matches!(
            full_report.execution,
            ExecutionReport::Complete { .. }
        ));
        assert_eq!(full_report.representation, Representation::Java);
        assert!(full_report.text.contains("<<"));
        assert_eq!(!full_report.source_map.is_empty(), has_map);
        if has_map {
            assert_eq!(
                full_report.evidence.state(RecoveryEvidenceKind::SourceMap),
                EvidenceState::Complete
            );
        }
        let needed = measured.usage().ir_items;
        assert!(needed > 0);
        for limit in [needed, needed - 1] {
            let mut limits = task_limits(&[]).unwrap();
            limits.ir_items = limit;
            let mut bounded = Budget::new(limits);
            let rerun = recover(&evidence, &mut bounded);
            let report = rerun.recovery();
            assert!(
                matches!(rerun.analysis().execution, ExecutionReport::Complete { .. }),
                "the boundary is after analysis: {:?}",
                rerun.analysis().execution
            );
            assert!(bounded.usage().ir_items <= limit);
            if limit == needed {
                assert!(report.produced(), "{report:?}");
                assert!(matches!(report.execution, ExecutionReport::Complete { .. }));
                assert_eq!(report.text, full_report.text);
                assert_eq!(
                    report.source_map.segments(),
                    full_report.source_map.segments()
                );
                assert_eq!(bounded.usage().ir_items, needed);
            } else {
                assert!(
                    matches!(
                        report.execution,
                        ExecutionReport::Partial {
                            reason: TerminationReason::BudgetExceeded {
                                dimension: BudgetDimension::IrItems
                            },
                            ..
                        }
                    ),
                    "{report:?}"
                );
                if has_map {
                    assert!(
                        report.produced(),
                        "committed text survives evidence exhaustion"
                    );
                    assert_eq!(report.text, full_report.text);
                    let delivered = report.source_map.segments().len();
                    assert!(delivered > 0 && delivered < full_report.source_map.segments().len());
                    assert!(
                        matches!(report.evidence.state(RecoveryEvidenceKind::SourceMap),
                        EvidenceState::Partial { delivered: count } if count == delivered as u64)
                    );
                    assert_eq!(
                        report.source_map.segments(),
                        &full_report.source_map.segments()[..delivered]
                    );
                } else {
                    assert!(!report.produced());
                    assert!(report.text.is_empty());
                    assert!(report.source_map.is_empty());
                    assert!(matches!(
                        report.stop(),
                        Some(StopReason::Budget {
                            dimension: BudgetDimension::IrItems,
                            ..
                        })
                    ));
                }
            }
        }
    }

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
