//! Interface-super proof stops are public recovery results, before any Java artifact exists.
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const SPECIAL: &[u8] = include_bytes!("fixtures/p3-special-dispatch/v8/SpecialProbe.class");
const BASE: &[u8] = include_bytes!("fixtures/p3-special-dispatch/v8/BaseProbe.class");
const DEFAULT: &[u8] = include_bytes!("fixtures/p3-special-dispatch/v8/DefaultProbe.class");

fn wide_budget() -> Budget {
    let mut limits = task_budget(&[]).unwrap().limits().clone();
    limits.elapsed_millis = u64::MAX;
    Budget::new(limits)
}

fn open(with_dependencies: bool) -> ArtifactSnapshot {
    let mut archive = Cursor::new(Vec::new());
    {
        let mut writer = ZipArchiveWriter::new(&mut archive);
        let entries: &[(&[u8], &[u8])] = if with_dependencies {
            &[
                (b"SpecialProbe.class", SPECIAL),
                (b"BaseProbe.class", BASE),
                (b"DefaultProbe.class", DEFAULT),
            ]
        } else {
            &[(b"SpecialProbe.class", SPECIAL)]
        };
        for (name, bytes) in entries {
            let (mut entry, config) = writer
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut output = config.wrap(&mut entry);
            output.write_all(bytes).unwrap();
            let (_, descriptor) = output.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        writer.finish().unwrap();
    }
    Engine::new()
        .open(
            ArtifactInput::bytes(archive.into_inner()),
            &mut wide_budget(),
        )
        .unwrap()
}

fn environment(snapshot: &ArtifactSnapshot) -> EnvironmentRequest {
    EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::PlainJar,
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".to_owned()),
    }
}

fn recover(
    snapshot: &ArtifactSnapshot,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> MethodRecoveryReport {
    let request = MethodOperationRequest {
        method: MethodRef::Name {
            class: ClassNameQuery::internal("SpecialProbe"),
            name: JvmBytes(b"defaultCall".to_vec()),
            descriptor: Some(JvmBytes(b"()I".to_vec())),
        },
        environment: environment(snapshot),
    };
    let OperationOutcome::Performed(report) = Engine::new()
        .recover_target_with_evidence(std::slice::from_ref(snapshot), &request, evidence, budget)
        .expect("a selected public method returns its recovery report")
    else {
        panic!("the fixture selects one physical method");
    };
    assert!(matches!(
        report.recovered.analysis().execution,
        ExecutionReport::Complete { .. }
    ));
    assert_eq!(report.method.name.0, b"defaultCall");
    report
}

fn assert_not_produced(report: &RecoveryReport, dimension: BudgetDimension) {
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded { dimension: actual },
                ..
            } if actual == dimension
        ),
        "{:?}",
        report.execution
    );
    assert!(matches!(report.outcome, RecoveryOutcome::Stopped(_)));
    assert_eq!(report.content, RecoveryContent::NotProduced);
    assert!(report.text.is_empty());
    assert!(report.source_map.is_empty());
    assert!(report.declaration.is_none());
    assert!(report.artifact.binding().is_none());
    assert!(report.rules.is_empty());
    assert!(report.regions.is_empty());
    assert!(report.fallbacks.is_empty());
    assert_eq!(report.diagnostics.len(), 1);
    assert!(report.diagnostics[0].provenance.is_some());
    for category in report.evidence.categories() {
        let expected = if report.evidence.requested().requests(category.kind) {
            EvidenceState::NotPerformed
        } else {
            EvidenceState::NotRequested
        };
        assert_eq!(category.state, expected);
    }
}

#[test]
fn wide_public_interface_super_has_complete_proof() {
    let report = recover(
        &open(true),
        &RecoveryEvidenceRequest::essential(),
        &mut wide_budget(),
    );
    assert!(matches!(
        report.presentation.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(report.recovered.recovery().produced());
    assert!(report.presentation.stop.is_none());
    assert!(
        report
            .recovered
            .recovery()
            .text
            .contains("DefaultProbe.super.value()")
    );
    assert!(report.usage.dependency_depth > 0);
}

#[test]
fn depth_zero_public_interface_proof_preserves_original_stop() {
    let mut limits = wide_budget().limits().clone();
    limits.dependency_depth = 0;
    let report = recover(
        &open(true),
        &RecoveryEvidenceRequest::essential(),
        &mut Budget::new(limits),
    );
    assert_not_produced(
        report.recovered.recovery(),
        BudgetDimension::DependencyDepth,
    );
    assert!(matches!(
        report.recovered.recovery().outcome,
        RecoveryOutcome::Stopped(StopReason::Interrupted {
            code: "jre_budget_interrupted",
            at: None
        })
    ));
    assert_eq!(report.usage.dependency_depth, 0);
    assert_eq!(report.usage.output_bytes, 0);
    assert_eq!(
        report.presentation.execution,
        report.recovered.recovery().execution
    );
    assert!(report.presentation.stop.is_some());
}

#[test]
fn earlier_depth_stop_wins_over_a_later_output_limit() {
    let mut limits = wide_budget().limits().clone();
    limits.dependency_depth = 0;
    limits.output_bytes = 0;
    let report = recover(
        &open(true),
        &RecoveryEvidenceRequest::all(),
        &mut Budget::new(limits),
    );
    assert_not_produced(
        report.recovered.recovery(),
        BudgetDimension::DependencyDepth,
    );
    assert_eq!(report.usage.output_bytes, 0);
}

#[test]
fn a_proof_stop_keeps_requested_evidence_pending_and_expectation_unverifiable() {
    let snapshot = open(true);
    let wide = recover(
        &snapshot,
        &RecoveryEvidenceRequest::essential(),
        &mut wide_budget(),
    );
    let binding = wide
        .recovered
        .recovery()
        .artifact
        .binding()
        .unwrap()
        .clone();
    let evidence = RecoveryEvidenceRequest::all().with_expected_artifact(binding);
    let mut limits = wide_budget().limits().clone();
    limits.dependency_depth = 0;
    let stopped = recover(&snapshot, &evidence, &mut Budget::new(limits));
    assert_not_produced(
        stopped.recovered.recovery(),
        BudgetDimension::DependencyDepth,
    );
    assert!(matches!(
        stopped.recovered.recovery().artifact.agreement(),
        jarde_java::ArtifactAgreement::Unverifiable { .. }
    ));
}

#[test]
fn missing_interface_evidence_remains_a_complete_ordinary_refusal() {
    let report = recover(
        &open(false),
        &RecoveryEvidenceRequest::essential(),
        &mut wide_budget(),
    );
    assert!(matches!(
        report.presentation.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(report.presentation.stop.is_none());
    assert!(
        !report
            .recovered
            .recovery()
            .text
            .contains("DefaultProbe.super.value()")
    );
}

#[test]
fn class_source_ends_on_the_interface_proof_stop_and_keeps_earlier_members() {
    let snapshot = open(true);
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("SpecialProbe"),
        },
        environment: environment(&snapshot),
    };
    let mut limits = wide_budget().limits().clone();
    limits.dependency_depth = 0;
    let OperationOutcome::Performed(report) = Engine::new()
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits),
        )
        .unwrap()
    else {
        panic!("the fixture selects one physical class");
    };
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::DependencyDepth
            },
            ..
        }
    ));
    let position = report
        .methods
        .iter()
        .position(|member| member.item.identity.name.0 == b"defaultCall")
        .expect("the stopped member retains its physical identity");
    let ClassSourceOutcome::Recovered {
        report: stopped,
        analysis,
    } = &report.methods[position].outcome
    else {
        panic!("the interface proof stop must retain the analyzed member");
    };
    assert!(matches!(
        analysis.execution,
        ExecutionReport::Complete { .. }
    ));
    assert_not_produced(stopped, BudgetDimension::DependencyDepth);
    assert!(report.methods[..position].iter().any(|member|
        matches!(&member.outcome, ClassSourceOutcome::Recovered { report, .. } if report.produced())));
    assert!(
        report.methods[position + 1..]
            .iter()
            .all(|member| !matches!(member.outcome, ClassSourceOutcome::Recovered { .. }))
    );
}
