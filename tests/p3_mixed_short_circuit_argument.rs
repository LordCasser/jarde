//! Complete-class Java 8 replay for a mixed short-circuit call argument.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const CLASS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/mixed-short-circuit-argument/MixedBooleanArgument.class"
);
const SOURCE: &str = include_str!(
    "fixtures/p3-conditional-values/mixed-short-circuit-argument/MixedBooleanArgument.java"
);
const RUNNER: &str =
    include_str!("fixtures/p3-conditional-values/mixed-short-circuit-argument/Runner.java");
const EXPECTED: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-argument/original-run.txt"
);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-mixed-argument-{}-{nonce}",
            std::process::id()
        ));
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
            class: ClassNameQuery::internal("MixedBooleanArgument"),
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
fn mixed_argument_has_one_structured_sink_call_and_complete_sources() {
    let source = recovered_source();
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"call")
        .expect("call method exists");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("call did not recover: {:?}", method.outcome);
    };
    assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured);
    assert_eq!(report.representation, Representation::Java);
    assert!(report.fallbacks.is_empty(), "{}", report.text);
    assert_eq!(report.text.matches("sink(").count(), 1, "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    for bci in [0, 1, 4, 7, 10, 13, 16, 17, 20, 21, 24] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} is unmapped: {}",
            report.text
        );
    }
}

#[test]
fn exhausted_budget_and_cancellation_do_not_publish_a_partial_sink() {
    let complete = recovered_source();
    let (snapshot, request) = opened();
    let mut limits = complete.limits.clone();
    limits.analysis_steps = complete.usage.analysis_steps - 1;
    let mut budget = Budget::new(limits);
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("bounded class-source request is answered");
    match stopped {
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
    let mut cancelled = task_budget(&[]).expect("bounded default budget");
    cancelled.cancellation_token().cancel();
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancelled class-source request is answered");
    assert!(matches!(stopped, OperationOutcome::Incomplete(candidates)
        if matches!(candidates.execution, ExecutionReport::Cancelled { .. })));
}

#[test]
fn mixed_argument_complete_class_matches_all_eight_jvm_paths() {
    let source = recovered_source();
    let scratch = Scratch::new();
    let original = scratch.0.join("original");
    let recovered = scratch.0.join("recovered");
    fs::create_dir(&original).expect("original directory");
    fs::create_dir(&recovered).expect("recovered directory");
    fs::write(original.join("MixedBooleanArgument.java"), SOURCE).expect("original source");
    fs::write(original.join("Runner.java"), RUNNER).expect("original runner");
    fs::write(recovered.join("MixedBooleanArgument.java"), source.text).expect("recovered source");
    fs::write(recovered.join("Runner.java"), RUNNER).expect("recovered runner");
    for directory in [&original, &recovered] {
        let compile = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "MixedBooleanArgument.java",
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
        fs::read(original.join("MixedBooleanArgument.class")).expect("rebuilt original"),
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
