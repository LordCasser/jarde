//! Narrow `ireturn` casts keep the actual return and operand evidence across recovery modes.

use jarde::*;
use std::slice;

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-narrow-integer-returns/v8/NarrowIntegerReturns.class");

fn budget() -> Budget {
    task_budget(&[]).expect("bounded task defaults")
}

fn snapshot() -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(FIXTURE.to_vec()), &mut budget())
        .expect("frozen class opens")
}

fn class_request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("NarrowIntegerReturns"),
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

fn class_source(
    snapshot: &ArtifactSnapshot,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &class_request(snapshot),
            evidence,
            &mut budget(),
        )
        .expect("single-class request succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("frozen class is unique: {other:?}"),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn recovery_request(
    snapshot: &ArtifactSnapshot,
    member: &ClassSourceMethod,
) -> jarde::ir::MethodAnalysisRequest {
    jarde::ir::MethodAnalysisRequest {
        environment: class_request(snapshot)
            .environment
            .build(slice::from_ref(snapshot))
            .expect("single-class environment builds"),
        method: member.item.identity.clone(),
        stages: jarde::ir::AnalysisStage::ALL.to_vec(),
    }
}

#[test]
fn default_all_and_replay_keep_identical_narrow_return_text() {
    let snapshot = snapshot();
    let default = class_source(&snapshot, &RecoveryEvidenceRequest::essential());
    let all = class_source(&snapshot, &RecoveryEvidenceRequest::all());
    assert_eq!(default.text, all.text);
    for name in ["directByte", "byteLocal", "postByte", "syncShort"] {
        let request = recovery_request(&snapshot, member(&all, name));
        let first = Engine::new()
            .recover_method_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("first recovery succeeds");
        let replay = Engine::new()
            .recover_method_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget(),
            )
            .expect("replay succeeds");
        assert!(first.recovery().produced(), "{name}");
        assert_eq!(first.recovery().text, replay.recovery().text, "{name}");
        assert_eq!(
            first.recovery().source_map,
            replay.recovery().source_map,
            "{name}"
        );
    }
}

#[test]
fn cast_sources_contain_operand_and_real_return_bci() {
    let snapshot = snapshot();
    let report = class_source(&snapshot, &RecoveryEvidenceRequest::all());
    for (name, cast, operand_bci, return_bci) in [
        ("directByte", "byte", 0, 1),
        ("directChar", "char", 0, 1),
        ("directShort", "short", 0, 1),
        ("postByte", "byte", 8, 11),
        ("preChar", "char", 8, 11),
        ("postShort", "short", 8, 11),
        ("syncByte", "byte", 4, 7),
        ("syncChar", "char", 4, 7),
        ("syncShort", "short", 5, 9),
    ] {
        let body = match &member(&report, name).outcome {
            ClassSourceOutcome::Recovered { report, .. } => report,
            other => panic!("{name} was not recovered: {other:?}"),
        };
        let prefix = format!("({cast})");
        let cast_segments: Vec<_> = body
            .source_map
            .segments()
            .iter()
            .filter(|segment| segment.text(&body.text).contains(&prefix))
            .collect();
        assert!(
            !cast_segments.is_empty(),
            "{name} has no {prefix} source segment: {}",
            body.text
        );
        assert!(
            cast_segments.iter().any(|segment| {
                let bcis = segment.origin().bcis();
                bcis.contains(&operand_bci) && bcis.contains(&return_bci)
            }),
            "{name} cast lacks operand {operand_bci} or return {return_bci}: {cast_segments:?}"
        );
    }
}

#[test]
fn tight_output_budget_and_precancellation_publish_no_partial_cast() {
    let snapshot = snapshot();
    let report = class_source(&snapshot, &RecoveryEvidenceRequest::all());
    let request = recovery_request(&snapshot, member(&report, "postByte"));
    let engine = Engine::new();
    let mut full_budget = budget();
    let full = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut full_budget,
        )
        .expect("full recovery succeeds");
    let full_report = full.recovery();
    assert!(full_report.produced());
    assert!(full_report.text.contains("(byte)"));
    let emitted = u64::try_from(full_report.text.len()).expect("text fits u64");
    let output_used = full_budget.usage().output_bytes;
    assert!(output_used > emitted);
    let mut limited = Budget::new(Limits {
        output_bytes: output_used - emitted,
        ..task_limits(&[]).expect("bounded defaults")
    });
    let stopped = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut limited,
        )
        .expect("output stop is reported");
    let stopped = stopped.recovery();
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty());
    assert!(stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
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
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation is reported");
    assert!(matches!(
        stopped.analysis().execution,
        ExecutionReport::Cancelled { .. }
    ));
    let stopped = stopped.recovery();
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty());
    assert!(stopped.source_map.is_empty());
}
