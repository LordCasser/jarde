//! Real `ireturn Z` consumes the low bit of a rendered int-sized operand.

use jarde::*;
use std::slice;

const CLASS: &[u8] =
    include_bytes!("fixtures/p3-integer-boolean-returns/v8/IntegerBooleanReturns.class");
const RAW_BOUNDARY: &[u8] = include_bytes!(
    "fixtures/p3-narrow-integer-returns/v8/boolean-boundaries/BooleanRawTwoCaller.class"
);
const SWITCH_JOIN: &[u8] = include_bytes!(
    "fixtures/p3-integer-boolean-returns/v8/actual-stack-join/ActualStackJoin.class"
);

fn budget() -> Budget {
    task_budget(&[]).expect("bounded defaults")
}

fn snapshot(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("fixture opens")
}

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

fn source(
    snapshot: &ArtifactSnapshot,
    name: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot, name),
            evidence,
            &mut budget(),
        )
        .expect("source request succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("fixture must select one class: {other:?}"),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing {name}"))
}

#[test]
fn integer_boolean_returns_are_structured_and_keep_operand_and_return_sources() {
    let snapshot = snapshot(CLASS);
    let report = source(
        &snapshot,
        "IntegerBooleanReturns",
        &RecoveryEvidenceRequest::all(),
    );
    for (name, operand_bci, return_bci) in [
        ("direct", 0, 1),
        ("once", 2, 5),
        ("post", 2, 11),
        ("pre", 2, 11),
        ("sync", 0, 1),
        ("syncOn", 4, 7),
    ] {
        let ClassSourceOutcome::Recovered { report: body, .. } = &member(&report, name).outcome
        else {
            panic!("{name} did not recover");
        };
        assert_eq!(
            body.representation,
            Representation::Java,
            "{name}: {}",
            body.text
        );
        assert_eq!(body.quality, Quality::Structured, "{name}: {}", body.text);
        assert!(body.text.contains("% 2 != 0"), "{name}: {}", body.text);
        assert!(!body.source_map.is_empty(), "{name}");
        assert!(
            body.source_map.segments().iter().any(|segment| {
                let bcis = segment.origin().bcis();
                segment.text(&body.text).contains("%")
                    && bcis.contains(&operand_bci)
                    && bcis.contains(&return_bci)
            }),
            "{name} lost operand {operand_bci} or return {return_bci} source: {:?}",
            body.source_map.segments()
        );
    }
    let once = match &member(&report, "once").outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        _ => unreachable!(),
    };
    assert_eq!(once.text.matches("next(").count(), 1, "{}", once.text);
}

#[test]
fn evidence_modes_and_replay_keep_the_same_body() {
    let snapshot = snapshot(CLASS);
    let default = source(
        &snapshot,
        "IntegerBooleanReturns",
        &RecoveryEvidenceRequest::essential(),
    );
    let all = source(
        &snapshot,
        "IntegerBooleanReturns",
        &RecoveryEvidenceRequest::all(),
    );
    assert_eq!(default.text, all.text);
    let recovery_request = jarde::ir::MethodAnalysisRequest {
        environment: request(&snapshot, "IntegerBooleanReturns")
            .environment
            .build(slice::from_ref(&snapshot))
            .expect("environment"),
        method: member(&all, "post").item.identity.clone(),
        stages: jarde::ir::AnalysisStage::ALL.to_vec(),
    };
    let recover = || {
        Engine::new()
            .recover_method_with_evidence(
                slice::from_ref(&snapshot),
                &recovery_request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("recovery succeeds")
    };
    let first = recover();
    let replay = recover();
    assert_eq!(first.recovery().text, replay.recovery().text);
    assert_eq!(first.recovery().source_map, replay.recovery().source_map);
}

#[test]
fn raw_boolean_argument_boundary_remains_refused() {
    let snapshot = snapshot(RAW_BOUNDARY);
    let report = source(
        &snapshot,
        "BooleanRawTwoCaller",
        &RecoveryEvidenceRequest::all(),
    );
    for name in ["byteValue", "charValue", "shortValue"] {
        let ClassSourceOutcome::Recovered { report: body, .. } = &member(&report, name).outcome
        else {
            panic!("{name} did not return a refusal");
        };
        assert_eq!(
            body.representation,
            Representation::Mixed,
            "{name}: {}",
            body.text
        );
        assert_eq!(body.quality, Quality::Fallback, "{name}");
    }
}

#[test]
fn shared_switch_uses_arm_evaluation_and_real_return_sources() {
    let snapshot = snapshot(SWITCH_JOIN);
    let report = source(
        &snapshot,
        "ActualStackJoin",
        &RecoveryEvidenceRequest::all(),
    );
    let ClassSourceOutcome::Recovered { report: body, .. } = &member(&report, "runByte").outcome
    else {
        panic!("shared switch did not return recovery");
    };
    assert_eq!(body.representation, Representation::Java, "{}", body.text);
    assert_eq!(body.quality, Quality::Structured, "{}", body.text);
    assert_eq!(body.text.matches("% 2 != 0").count(), 2, "{}", body.text);
    for bci in [20, 26, 32] {
        assert!(
            !body.text_of_bci(bci).is_empty(),
            "missing BCI {bci}: {}",
            body.text
        );
    }
}

#[test]
fn tight_output_budget_and_precancellation_publish_no_partial_conversion() {
    let snapshot = snapshot(CLASS);
    let report = source(
        &snapshot,
        "IntegerBooleanReturns",
        &RecoveryEvidenceRequest::all(),
    );
    let recovery_request = jarde::ir::MethodAnalysisRequest {
        environment: request(&snapshot, "IntegerBooleanReturns")
            .environment
            .build(slice::from_ref(&snapshot))
            .expect("environment"),
        method: member(&report, "direct").item.identity.clone(),
        stages: jarde::ir::AnalysisStage::ALL.to_vec(),
    };
    let engine = Engine::new();
    let mut full_budget = budget();
    let full = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &recovery_request,
            &RecoveryEvidenceRequest::all(),
            &mut full_budget,
        )
        .expect("full recovery");
    let full_body = full.recovery();
    assert!(full_body.produced());
    assert!(full_body.text.contains("% 2 != 0"));
    let emitted = u64::try_from(full_body.text.len()).expect("text length");
    let mut limited = Budget::new(Limits {
        output_bytes: full_budget.usage().output_bytes - emitted,
        ..task_limits(&[]).expect("bounded defaults")
    });
    let stopped = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &recovery_request,
            &RecoveryEvidenceRequest::all(),
            &mut limited,
        )
        .expect("budget stop");
    assert!(!stopped.recovery().produced());
    assert!(stopped.recovery().text.is_empty());
    assert!(stopped.recovery().source_map.is_empty());
    assert!(matches!(
        stopped.recovery().stop(),
        Some(StopReason::Budget {
            dimension: jarde::budget::CountedBudgetDimension::OutputBytes,
            ..
        })
    ));

    let token = jarde::budget::CancellationToken::new();
    token.cancel();
    let mut cancelled =
        Budget::with_cancellation_token(task_limits(&[]).expect("bounded defaults"), token);
    let stopped = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &recovery_request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation");
    assert!(matches!(
        stopped.analysis().execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert!(!stopped.recovery().produced());
    assert!(stopped.recovery().text.is_empty());
    assert!(stopped.recovery().source_map.is_empty());
}
