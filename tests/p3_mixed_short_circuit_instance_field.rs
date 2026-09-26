//! Instance-field short-circuit consumer: receiver evaluation precedes lazy RHS and null failure.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const CLASS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/mixed-short-circuit-instance-field/MixedShortCircuitField.class"
);
const SOURCE: &str = include_str!(
    "fixtures/p3-conditional-values/mixed-short-circuit-instance-field/MixedShortCircuitField.java"
);
const RUNNER: &str =
    include_str!("fixtures/p3-conditional-values/mixed-short-circuit-instance-field/Runner.java");
const EXPECTED: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-instance-field/original-run.txt"
);
const CONTROLS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/mixed-short-circuit-instance-field/MixedInstanceControls.class"
);
const CONTROL_SOURCE: &str = include_str!(
    "fixtures/p3-conditional-values/mixed-short-circuit-instance-field/MixedInstanceControls.java"
);
const CONTROL_RUNNER: &str = include_str!(
    "fixtures/p3-conditional-values/mixed-short-circuit-instance-field/ControlRunner.java"
);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-instance-short-circuit-{}-{nonce}",
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
    opened_with(CLASS, "MixedShortCircuitField")
}

fn recovered_with(class: &[u8], name: &str) -> ClassSourceReport {
    let (snapshot, request) = opened_with(class, name);
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let OperationOutcome::Performed(report) = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("class source succeeds")
    else {
        panic!("single class must bind uniquely");
    };
    report
}

fn recovered() -> ClassSourceReport {
    recovered_with(CLASS, "MixedShortCircuitField")
}

#[test]
fn one_instance_write_owns_the_closed_graph_and_all_origins() {
    let source = recovered();
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"one")
        .expect("one method exists");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("one must recover: {:?}", method.outcome);
    };
    assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured);
    assert!(report.fallbacks.is_empty(), "{}", report.text);
    assert_eq!(report.text.matches("target(arg1).result =").count(), 1);
    assert!(report.text.contains("&&"), "{}", report.text);
    assert!(report.text.contains("||"), "{}", report.text);
    assert!(!report.text.contains("% 2 != 0"), "{}", report.text);
    assert_eq!(report.text.matches("target(arg1)").count(), 1);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    for bci in [0, 1, 4, 5, 8, 11, 14, 17, 20, 21, 24, 25, 28] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no source: {}",
            report.text
        );
    }
}

#[test]
fn java8_rebuild_preserves_all_sixteen_nonnull_and_null_paths() {
    let source = recovered();
    let scratch = Scratch::new();
    let original = scratch.0.join("original");
    let recovered = scratch.0.join("recovered");
    fs::create_dir(&original).expect("original directory");
    fs::create_dir(&recovered).expect("recovered directory");
    fs::write(original.join("MixedShortCircuitField.java"), SOURCE).expect("original source");
    fs::write(original.join("Runner.java"), RUNNER).expect("original runner");
    fs::write(recovered.join("MixedShortCircuitField.java"), source.text)
        .expect("recovered source");
    fs::write(
        recovered.join("MixedShortCircuitField$Box.java"),
        "final class MixedShortCircuitField$Box { boolean result; }\n",
    )
    .expect("binary-name companion");
    fs::write(
        recovered.join("Runner.java"),
        RUNNER.replace("MixedShortCircuitField.Box", "MixedShortCircuitField$Box"),
    )
    .expect("recovered runner");

    for directory in [&original, &recovered] {
        let compile = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "MixedShortCircuitField.java",
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
        fs::read(original.join("MixedShortCircuitField.class")).expect("rebuilt original"),
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
    assert_eq!(EXPECTED.lines().count(), 16);
}

#[test]
fn non_boolean_field_copy_and_compound_update_keep_the_entire_candidate_quoted() {
    let scratch = Scratch::new();
    fs::write(scratch.0.join("MixedInstanceControls.java"), CONTROL_SOURCE)
        .expect("control source");
    fs::write(scratch.0.join("ControlRunner.java"), CONTROL_RUNNER).expect("control runner");
    let compile = Command::new("javac")
        .args([
            "--release",
            "8",
            "-g:none",
            "MixedInstanceControls.java",
            "ControlRunner.java",
        ])
        .current_dir(&scratch.0)
        .output()
        .expect("javac is installed");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    assert_eq!(
        fs::read(scratch.0.join("MixedInstanceControls.class")).expect("rebuilt control"),
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
    let source = recovered_with(CONTROLS, "MixedInstanceControls");
    for (name, bcis) in [
        (
            "numeric",
            &[0, 1, 4, 5, 8, 11, 14, 17, 20, 21, 24, 25, 28][..],
        ),
        (
            "duplicated",
            &[0, 1, 4, 5, 8, 11, 14, 17, 20, 21, 24, 25, 26, 29][..],
        ),
        (
            "compound",
            &[0, 1, 4, 5, 8, 9, 12, 15, 18, 21, 24, 25, 28, 29, 30, 33][..],
        ),
        (
            "sharedReceiver",
            &[0, 1, 4, 5, 8, 9, 12, 15, 18, 21, 24, 25, 28, 29, 32][..],
        ),
        (
            "protectedWrite",
            &[0, 1, 4, 5, 8, 11, 14, 17, 20, 21, 24, 25, 28][..],
        ),
        (
            "inheritedOwner",
            &[0, 1, 4, 5, 8, 11, 14, 17, 20, 21, 24, 25, 28][..],
        ),
    ] {
        let method = source
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name.as_bytes())
            .expect("control method exists");
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("{name} did not recover: {:?}", method.outcome);
        };
        assert_eq!(report.quality, jarde_jvm::ir::Quality::Fallback, "{name}");
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
        if name == "protectedWrite" {
            assert!(report.regions.iter().any(|region| {
                region.code == Some("jre_region_exception_edge")
                    && !region.structured
                    && region.blocks.contains(&20)
                    && region.blocks.contains(&25)
            }));
        }
        assert!(
            !report.text.contains(".result ="),
            "{name}: {}",
            report.text
        );
        for bci in bcis {
            assert!(
                !report.source_map.of_bci(*bci).is_empty(),
                "{name}: unmapped BCI {bci}: {}",
                report.text
            );
        }
    }
}

#[test]
fn budget_and_cancellation_never_publish_a_partial_instance_assignment() {
    let complete = recovered();
    let (snapshot, request) = opened();
    let mut limits = complete.limits.clone();
    limits.analysis_steps = complete.usage.analysis_steps - 1;
    let mut budget = Budget::new(limits);
    let bounded = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("bounded request answers");
    match bounded {
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
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancelled request answers");
    assert!(
        matches!(outcome, OperationOutcome::Incomplete(candidates)
        if matches!(candidates.execution, ExecutionReport::Cancelled { .. })),
        "a cancelled request cannot publish a class"
    );
}
