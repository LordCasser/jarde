//! P3: only a proved same-block `int` field/array update becomes `+=`.

use jarde::*;
use std::slice;

const PROBE: &[u8] = include_bytes!("fixtures/p3-compound-lvalue-updates/v8/CompoundProbe.class");
const BOUNDARY_GAPS: [(&str, &[u8], &[u32]); 6] = [
    (
        "field-gap-before-dup",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-gap-before-dup/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 4, 8, 9, 12, 16],
    ),
    (
        "field-gap-after-dup",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-gap-after-dup/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 4, 5, 9, 12, 16],
    ),
    (
        "field-gap-before-store",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-gap-before-store/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 4, 7, 11, 12, 16],
    ),
    (
        "array-gap-before-index",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-gap-before-index/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 4, 8, 11, 12, 13, 17],
    ),
    (
        "array-gap-after-dup",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-gap-after-dup/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 6, 7, 8, 12, 13, 17],
    ),
    (
        "array-gap-before-store",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-gap-before-store/CompoundBoundaryProbe.class"
        ),
        &[0, 3, 6, 7, 8, 12, 13, 17],
    ),
];
const IDENTITY_BOUNDARIES: [(&str, &[u8], &str, &[u32]); 5] = [
    (
        "field-different-member",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-different-member/CompoundBoundaryProbe.class"
        ),
        "fieldDifferentMember",
        &[0, 3, 4, 8, 12],
    ),
    (
        "array-different-index-copy",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-different-index-copy/CompoundBoundaryProbe.class"
        ),
        "arrayDifferentIndex",
        &[0, 3, 6, 9, 11, 15],
    ),
    (
        "array-different-array-copy",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-different-array-copy/CompoundBoundaryProbe.class"
        ),
        "arrayDifferentArray",
        &[0, 3, 6, 8, 12, 14, 18],
    ),
    (
        "field-multi-consumer",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/field-multi-consumer/CompoundBoundaryProbe.class"
        ),
        "fieldMultiConsumer",
        &[0, 3, 4, 5, 8, 12, 16],
    ),
    (
        "array-multi-consumer",
        include_bytes!(
            "fixtures/p3-compound-lvalue-updates/boundaries/patched/array-multi-consumer/CompoundBoundaryProbe.class"
        ),
        "arrayMultiConsumer",
        &[0, 3, 6, 7, 8, 11, 13, 17],
    ),
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen class opens")
}

fn request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
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
            loader: LoaderId("app".to_string()),
        },
    }
}

fn class_source(
    snapshot: &ArtifactSnapshot,
    class: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot, class),
            evidence,
            &mut budget(),
        )
        .expect("the fixture class-source request is valid")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen class resolves uniquely: {other:?}"),
    }
}

fn recovery_request(
    snapshot: &ArtifactSnapshot,
    member: &ClassSourceMethod,
) -> jarde::ir::MethodAnalysisRequest {
    jarde::ir::MethodAnalysisRequest {
        environment: request(snapshot, "CompoundProbe")
            .environment
            .build(slice::from_ref(snapshot))
            .expect("the fixture's single-class environment is valid"),
        method: member.item.identity.clone(),
        stages: jarde::ir::AnalysisStage::ALL.to_vec(),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no method `{name}`"))
}

fn recovered(method: &ClassSourceMethod) -> &RecoveryReport {
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the fixture method was not recovered: {other:?}"),
    }
}

#[test]
fn same_block_field_and_int_array_updates_are_single_evaluation_source_mapped_statements() {
    let snapshot = open(PROBE);
    let essential = class_source(
        &snapshot,
        "CompoundProbe",
        &RecoveryEvidenceRequest::essential(),
    );
    let complete = class_source(&snapshot, "CompoundProbe", &RecoveryEvidenceRequest::all());
    assert_eq!(
        essential.text, complete.text,
        "evidence selection changes no body"
    );

    for (name, statement, once) in [
        ("field", "receiver().value += rhs(2);", "receiver()"),
        ("array", "data[index()] += rhs(4);", "index()"),
        (
            "fieldSnapshot",
            "receiver().value += rhsFieldMutation();",
            "receiver()",
        ),
        (
            "arraySnapshot",
            "data[index()] += rhsArrayMutation();",
            "index()",
        ),
    ] {
        let body = &method(&essential, name).text;
        assert!(body.contains(statement), "`{name}` omitted update:\n{body}");
        assert_eq!(
            body.matches(once).count(),
            1,
            "`{name}` evaluates lhs twice:\n{body}"
        );
        let mapped = recovered(method(&complete, name));
        assert_eq!(mapped.text, recovered(method(&essential, name)).text);
        if name.starts_with("field") {
            let accesses: Vec<_> = mapped
                .fields
                .iter()
                .filter(|field| field.name == "value")
                .map(|field| (field.access, field.presented))
                .collect();
            assert!(
                accesses.contains(&("read", true))
                    && accesses.contains(&("write", true))
                    && accesses.iter().all(|(_, presented)| *presented),
                "{name}: {accesses:?}\n{}",
                mapped.text
            );
        }
        assert!(
            !mapped.source_map.is_empty(),
            "`{name}` has no selected source map"
        );
    }

    for (name, bcis) in [
        ("field", &[0, 3, 4, 7, 8, 11, 12][..]),
        ("fieldSnapshot", &[0, 3, 4, 7, 10, 11][..]),
        ("array", &[0, 3, 6, 7, 8, 9, 12, 13][..]),
        ("arraySnapshot", &[0, 3, 6, 7, 8, 11, 12][..]),
    ] {
        let report = recovered(method(&complete, name));
        for bci in bcis {
            assert!(
                !report.source_map.text_of_bci(&report.text, *bci).is_empty(),
                "`{name}` update lost source BCI {bci}: {:?}",
                report.source_map.segments()
            );
        }
    }
}

#[test]
fn lvalue_prefix_gaps_refuse_compound_claims_and_keep_the_complete_chain() {
    for (name, bytes, bcis) in BOUNDARY_GAPS {
        let snapshot = open(bytes);
        let report = class_source(
            &snapshot,
            "CompoundBoundaryProbe",
            &RecoveryEvidenceRequest::all(),
        );
        let is_field = name.starts_with("field-");
        let method_name = if is_field {
            "fieldSnapshot"
        } else {
            "arraySnapshot"
        };
        let method = method(&report, method_name);
        let recovery = recovered(method);
        assert!(
            !method.text.contains("+= "),
            "uninterrupted update proof crossed {name}:\n{}",
            method.text
        );
        for bci in bcis {
            assert!(
                !recovery
                    .source_map
                    .text_of_bci(&recovery.text, *bci)
                    .is_empty(),
                "{name} lost original BCI {bci}: {:?}\n{}",
                recovery.source_map.segments(),
                method.text
            );
        }
    }
}

#[test]
fn mismatched_and_shared_lvalue_copies_are_refused_with_their_source_anchors() {
    for (name, bytes, method_name, bcis) in IDENTITY_BOUNDARIES {
        let snapshot = open(bytes);
        let report = class_source(
            &snapshot,
            "CompoundBoundaryProbe",
            &RecoveryEvidenceRequest::all(),
        );
        let method = method(&report, method_name);
        let recovery = recovered(method);
        assert!(
            !method.text.contains("+= "),
            "identity/consumer boundary {name} was presented as compound update:\n{}",
            method.text
        );
        for bci in bcis {
            assert!(
                !recovery
                    .source_map
                    .text_of_bci(&recovery.text, *bci)
                    .is_empty(),
                "{name} lost original BCI {bci}: {:?}\n{}",
                recovery.source_map.segments(),
                method.text
            );
        }
    }
}

#[test]
fn compound_recovery_keeps_output_evidence_and_cancellation_stops() {
    let snapshot = open(PROBE);
    let complete = class_source(&snapshot, "CompoundProbe", &RecoveryEvidenceRequest::all());
    let method = method(&complete, "array");
    let request = recovery_request(&snapshot, method);
    let engine = Engine::new();
    let evidence = RecoveryEvidenceRequest::all();
    let mut full_budget = task_budget(&[]).expect("the task defaults are bounded");
    let full = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut full_budget,
        )
        .expect("the compound method is recovered");
    let full_report = full.recovery();
    assert!(full_report.produced());
    assert!(full_report.text.contains("data[index()] += rhs(4);"));
    assert!(matches!(
        full_report.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    ));
    let used_items = full_budget
        .usage()
        .counted_usage(jarde::budget::CountedBudgetDimension::IrItems);
    assert!(used_items > 1, "the proved chain charges bounded work");

    let emitted = u64::try_from(full_report.text.len()).expect("the artifact length fits u64");
    let complete_output = full_budget.usage().output_bytes;
    assert!(
        complete_output > emitted,
        "analysis output precedes the artifact"
    );
    let mut output_budget = Budget::new(Limits {
        output_bytes: complete_output - emitted,
        ..task_limits(&[]).expect("the task defaults are bounded")
    });
    let stopped_output = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut output_budget,
        )
        .expect("an output stop is reported");
    let stopped_report = stopped_output.recovery();
    assert!(!stopped_report.produced());
    assert!(stopped_report.text.is_empty());
    assert!(stopped_report.source_map.is_empty());
    assert!(
        matches!(
            stopped_report.stop(),
            Some(StopReason::Budget {
                dimension: jarde::budget::CountedBudgetDimension::OutputBytes,
                ..
            })
        ),
        "the body stop keeps its budget dimension: {:?}",
        stopped_report.stop()
    );

    let mut map_budget = Budget::new(Limits {
        ir_items: used_items - 1,
        ..task_limits(&[]).expect("the task defaults are bounded")
    });
    let partial_map = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut map_budget,
        )
        .expect("an evidence stop is reported");
    let partial_report = partial_map.recovery();
    assert!(partial_report.produced(), "text survives an evidence stop");
    assert_eq!(partial_report.text, full_report.text);
    assert!(partial_report.source_map.len() < full_report.source_map.len());
    assert!(matches!(
        partial_report
            .evidence
            .state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Partial { .. }
    ));

    let token = jarde::budget::CancellationToken::new();
    token.cancel();
    let mut cancelled_budget = Budget::with_cancellation_token(
        task_limits(&[]).expect("the task defaults are bounded"),
        token,
    );
    let cancelled = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut cancelled_budget,
        )
        .expect("cancellation is reported");
    assert!(matches!(
        cancelled.analysis().execution,
        ExecutionReport::Cancelled { .. }
    ));
    let cancelled_report = cancelled.recovery();
    assert!(!cancelled_report.produced());
    assert!(cancelled_report.text.is_empty());
    assert!(cancelled_report.source_map.is_empty());
}
