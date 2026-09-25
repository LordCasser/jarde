//! Complete-class Java 8 replay for a mixed short-circuit Boolean return.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const CLASS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/mixed-short-circuit-return/MixedLocalReturn.class"
);
const SOURCE: &str =
    include_str!("fixtures/p3-conditional-values/mixed-short-circuit-return/MixedLocalReturn.java");
const RUNNER: &str =
    include_str!("fixtures/p3-conditional-values/mixed-short-circuit-return/Runner.java");
const EXPECTED: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-return/original-run.txt"
);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("jarde-mixed-return-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).expect("scratch directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn opened() -> (ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("frozen Java 8 class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("MixedLocalReturn"),
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
    (snapshot, request)
}

fn recovered_source() -> ClassSourceReport {
    let (snapshot, request) = opened();
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let OperationOutcome::Performed(source) = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("class source request succeeds")
    else {
        panic!("one standalone class must bind uniquely");
    };
    source
}

#[test]
fn mixed_return_has_one_structured_return_and_complete_sources() {
    let source = recovered_source();
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"value")
        .expect("value method exists");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("value did not recover: {:?}", method.outcome);
    };
    assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured);
    assert_eq!(report.representation, Representation::Java);
    assert!(report.fallbacks.is_empty(), "{}", report.text);
    assert_eq!(report.text.matches("return ").count(), 1, "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    for bci in [0, 1, 4, 7, 10, 13, 16, 17, 20, 21] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} is unmapped: {}",
            report.text
        );
    }
}

#[test]
fn exhausted_source_budget_cannot_publish_a_partial_return() {
    let complete = recovered_source();
    let (snapshot, request) = opened();
    let mut limits = complete.limits.clone();
    limits.analysis_steps = complete.usage.analysis_steps - 1;
    let mut budget = Budget::new(limits);
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("bounded class-source request is answered");
    match outcome {
        OperationOutcome::Incomplete(candidates) => assert!(matches!(
            candidates.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        )),
        OperationOutcome::Performed(report) => assert!(matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        )),
        other => panic!("unexpected bounded result: {other:?}"),
    }
}

#[test]
fn cancellation_cannot_publish_a_partial_return() {
    let (snapshot, request) = opened();
    let mut budget = task_budget(&[]).expect("bounded default budget");
    budget.cancellation_token().cancel();
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("cancelled class-source request is answered");
    assert!(
        matches!(outcome, OperationOutcome::Incomplete(candidates)
        if matches!(candidates.execution, ExecutionReport::Cancelled { .. })),
        "a cancelled request cannot publish a class"
    );
}

#[test]
fn mixed_return_complete_class_matches_all_eight_jvm_paths() {
    let source = recovered_source();
    let scratch = Scratch::new();
    let original = scratch.0.join("original");
    let recovered = scratch.0.join("recovered");
    fs::create_dir(&original).expect("original directory");
    fs::create_dir(&recovered).expect("recovered directory");
    fs::write(original.join("MixedLocalReturn.java"), SOURCE).expect("original source");
    fs::write(original.join("Runner.java"), RUNNER).expect("original runner");
    fs::write(recovered.join("MixedLocalReturn.java"), source.text).expect("recovered source");
    fs::write(recovered.join("Runner.java"), RUNNER).expect("recovered runner");
    for directory in [&original, &recovered] {
        let compile = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "MixedLocalReturn.java",
                "Runner.java",
            ])
            .current_dir(directory)
            .output()
            .expect("javac is installed");
        assert!(
            compile.status.success(),
            "{}",
            String::from_utf8_lossy(&compile.stderr)
        );
    }
    assert_eq!(
        fs::read(original.join("MixedLocalReturn.class")).expect("rebuilt original"),
        CLASS
    );
    for directory in [&original, &recovered] {
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp", ".", "Runner"])
            .current_dir(directory)
            .output()
            .expect("java is installed");
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8(run.stdout).expect("UTF-8 trace"),
            EXPECTED
        );
    }
    assert_eq!(EXPECTED.lines().count(), 8);
}
