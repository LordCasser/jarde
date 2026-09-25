//! Complete-class Java 8 replay for a mixed short-circuit Boolean local.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const CLASS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/mixed-short-circuit-local/MixedBooleanLocal.class"
);
const SOURCE: &str =
    include_str!("fixtures/p3-conditional-values/mixed-short-circuit-local/MixedBooleanLocal.java");
const RUNNER: &str =
    include_str!("fixtures/p3-conditional-values/mixed-short-circuit-local/Runner.java");
const EXPECTED: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-local/original-run.txt"
);
const CONTROLS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/mixed-short-circuit-local-controls/MixedLocalControls.class"
);
const CONTROL_SOURCE: &str = include_str!(
    "fixtures/p3-conditional-values/mixed-short-circuit-local-controls/MixedLocalControls.java"
);
const CONTROL_RUNNER: &str = include_str!(
    "fixtures/p3-conditional-values/mixed-short-circuit-local-controls/ControlRunner.java"
);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("jarde-mixed-local-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).expect("scratch directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn opened_with(class: &[u8], name: &str) -> (ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("frozen Java 8 class opens");
    let request = ClassSourceRequest {
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
    };
    (snapshot, request)
}

fn opened() -> (ArtifactSnapshot, ClassSourceRequest) {
    opened_with(CLASS, "MixedBooleanLocal")
}

fn recovered_source_with(class: &[u8], name: &str) -> ClassSourceReport {
    let (snapshot, request) = opened_with(class, name);
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

fn recovered_source() -> ClassSourceReport {
    recovered_source_with(CLASS, "MixedBooleanLocal")
}

#[test]
fn mixed_local_stores_once_and_reads_the_same_boolean_name_twice() {
    let source = recovered_source();
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"one")
        .expect("one method exists");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("one did not recover: {:?}", method.outcome);
    };
    assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured);
    assert_eq!(report.representation, Representation::Java);
    assert!(report.fallbacks.is_empty(), "{}", report.text);
    assert_eq!(
        report.text.matches("boolean local1 =").count(),
        1,
        "{}",
        report.text
    );
    assert!(
        report.text.contains("MixedBooleanLocal.result = local1;"),
        "{}",
        report.text
    );
    assert!(report.text.contains("return local1;"), "{}", report.text);
    assert_eq!(report.text.matches("return ").count(), 1, "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    for bci in [0, 1, 4, 7, 10, 13, 16, 17, 20, 21, 22, 23, 26, 27] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} is unmapped: {}",
            report.text
        );
    }
}

#[test]
fn numeric_use_second_write_duplicated_phi_and_crossing_scope_refuse_the_local_graph() {
    let scratch = Scratch::new();
    fs::write(scratch.0.join("MixedLocalControls.java"), CONTROL_SOURCE).expect("control source");
    fs::write(scratch.0.join("ControlRunner.java"), CONTROL_RUNNER).expect("control runner");
    let compiled = Command::new("javac")
        .args([
            "--release",
            "8",
            "-g:none",
            "-Xlint:-options",
            "MixedLocalControls.java",
            "ControlRunner.java",
        ])
        .current_dir(&scratch.0)
        .output()
        .expect("javac is installed");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert_eq!(
        fs::read(scratch.0.join("MixedLocalControls.class")).expect("rebuilt control"),
        CONTROLS
    );
    let verified = Command::new("java")
        .args(["-Xverify:all", "-cp", ".", "ControlRunner"])
        .current_dir(&scratch.0)
        .output()
        .expect("java is installed");
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    let source = recovered_source_with(CONTROLS, "MixedLocalControls");
    for (name, bcis) in [
        ("numeric", &[0, 1, 4, 7, 10, 13, 16, 17, 20, 21][..]),
        ("rewritten", &[0, 1, 4, 7, 10, 13, 16, 17, 20, 21][..]),
        ("duplicated", &[0, 1, 4, 7, 10, 13, 16, 17, 20, 21][..]),
        ("crossing", &[0, 4, 10, 16, 20, 21, 25, 28][..]),
    ] {
        let method = source
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name.as_bytes())
            .expect("negative control method exists");
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("{name} did not recover: {:?}", method.outcome);
        };
        assert_ne!(
            report.quality,
            jarde_jvm::ir::Quality::Structured,
            "{name}: {}",
            report.text
        );
        assert!(
            !report.text.contains("boolean local1 ="),
            "{name}: {}",
            report.text
        );
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
        for bci in bcis {
            assert!(
                !report.source_map.of_bci(*bci).is_empty(),
                "{name}: unmapped {bci}: {}",
                report.text
            );
        }
    }
}

#[test]
fn exhausted_source_budget_cannot_publish_a_partial_local() {
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
fn cancellation_cannot_publish_a_partial_local() {
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
fn mixed_local_complete_class_matches_all_eight_jvm_paths() {
    let source = recovered_source();
    let scratch = Scratch::new();
    let original = scratch.0.join("original");
    let recovered = scratch.0.join("recovered");
    fs::create_dir(&original).expect("original directory");
    fs::create_dir(&recovered).expect("recovered directory");
    fs::write(original.join("MixedBooleanLocal.java"), SOURCE).expect("original source");
    fs::write(original.join("Runner.java"), RUNNER).expect("original runner");
    fs::write(recovered.join("MixedBooleanLocal.java"), source.text).expect("recovered source");
    fs::write(recovered.join("Runner.java"), RUNNER).expect("recovered runner");
    for directory in [&original, &recovered] {
        let compile = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "MixedBooleanLocal.java",
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
        fs::read(original.join("MixedBooleanLocal.class")).expect("rebuilt original"),
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
