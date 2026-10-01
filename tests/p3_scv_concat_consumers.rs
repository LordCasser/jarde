//! Complete-class Java 8 replay for the short-circuit locals a concatenation consumes
//! (change `recover-scv-concat-consumers`).
//!
//! The four positive members freeze the consumer shapes the EM-19 patrol (`em19-bitops-patrol`)
//! refused on the baseline: the plain `append(Z)` tail operand, the `"" +` head position (whose
//! javac 8 lowering is the same primitive `append(Z)`, not a boxed overload), two chains over one
//! `append(Z)` each, and the single-test second conditional whose first test lives in the first
//! chain's consumer block. `reRead` is the verifier-valid negative: the local's value reaches a
//! `bastore` before the concatenation, so one read is not an explicit Boolean consumer and the
//! whole method keeps its degradation.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const CLASS: &[u8] =
    include_bytes!("fixtures/p3-conditional-values/scv-concat-consumers/ScvConcatConsumers.class");
const SOURCE: &str =
    include_str!("fixtures/p3-conditional-values/scv-concat-consumers/ScvConcatConsumers.java");
const CONTROLS: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/scv-concat-consumers/ScvConcatReReadControls.class"
);
const CONTROL_SOURCE: &str = include_str!(
    "fixtures/p3-conditional-values/scv-concat-consumers/ScvConcatReReadControls.java"
);
const EXPECTED_RUN: &str = "true:\ntrue:\nfalse:true\nfalse:true\n";

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("jarde-scv-concat-{}-{nonce}", std::process::id()));
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
            class: ClassNameQuery::internal(name.to_string()),
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
    opened_with(CLASS, "ScvConcatConsumers")
}

fn recovered_source() -> ClassSourceReport {
    recovered_source_with(opened())
}

fn recovered_source_with(
    (snapshot, request): (ArtifactSnapshot, ClassSourceRequest),
) -> ClassSourceReport {
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

fn member<'a>(source: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("{name} exists"))
}

#[test]
fn concat_consumed_short_circuit_locals_recover_as_boolean() {
    let source = recovered_source();
    for (name, expected) in [
        (
            "plain",
            vec![
                "boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0;",
                "return \"\" + local1 + \":\";",
            ],
        ),
        (
            "boxedHead",
            vec![
                "boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0;",
                "return \"\" + local1 + \":\";",
            ],
        ),
        (
            "doubleChain",
            vec![
                "boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0;",
                "boolean local2 = (arg0 & 4) != 0 || (arg0 & 8) != 0;",
                "return \"\" + local1 + \":\" + local2;",
            ],
        ),
        (
            "singleTail",
            vec![
                "boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0;",
                "boolean local2 = (arg0 & 4) != 0;",
                "return \"\" + local1 + \":\" + local2;",
            ],
        ),
    ] {
        let outcome = &member(&source, name).outcome;
        let ClassSourceOutcome::Recovered { report, .. } = outcome else {
            panic!("{name} did not recover: {:?}", outcome);
        };
        assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured, "{name}");
        assert_eq!(report.representation, Representation::Java, "{name}");
        assert!(report.fallbacks.is_empty(), "{name}: {}", report.text);
        for line in expected {
            assert!(
                report.text.contains(line),
                "{name}: {line:?} in {}",
                report.text
            );
        }
        // The value is the condition itself, not an int spelling adapted at the operand.
        assert!(!report.text.contains("? 1 : 0"), "{name}: {}", report.text);
        assert!(!report.text.contains("% 2 != 0"), "{name}: {}", report.text);
        assert!(
            !report.text.contains("@bytecode"),
            "{name}: {}",
            report.text
        );
    }
}

#[test]
fn second_read_outside_the_boolean_positions_keeps_the_degradation() {
    let source = recovered_source_with(opened_with(CONTROLS, "ScvConcatReReadControls"));
    let outcome = &member(&source, "reRead").outcome;
    let ClassSourceOutcome::Recovered { report, .. } = outcome else {
        panic!("reRead did not recover: {:?}", outcome);
    };
    assert_ne!(
        report.quality,
        jarde_jvm::ir::Quality::Structured,
        "{}",
        report.text
    );
    assert!(report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        report.text.contains(
            "the short-circuit chain from BCI 3 through 9 reaches a shared value consumer \
             at BCI 17"
        ),
        "{}",
        report.text
    );
    assert!(!report.text.contains("boolean local1 ="), "{}", report.text);
    assert!(
        CONTROL_SOURCE.contains("flags[0] = hasA;"),
        "the control's second read is the array store the Boolean positions reject"
    );
    // Every instruction the quote names stays reachable in the segment table.
    for bci in [0usize, 3, 9, 17, 23, 32, 43] {
        assert!(
            !report.source_map.of_bci(bci as u32).is_empty(),
            "BCI {bci} is unmapped: {}",
            report.text
        );
    }
}

#[test]
fn recovered_class_recompiles_and_runs_like_the_original() {
    let source = recovered_source();
    let scratch = Scratch::new();
    let original = scratch.0.join("original");
    let recovered = scratch.0.join("recovered");
    fs::create_dir(&original).expect("original directory");
    fs::create_dir(&recovered).expect("recovered directory");
    fs::write(original.join("ScvConcatConsumers.java"), SOURCE).expect("original source");
    fs::write(
        recovered.join("ScvConcatConsumers.java"),
        source.text.clone(),
    )
    .expect("recovered source");
    for directory in [&original, &recovered] {
        let compile = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "-Xlint:-options",
                "ScvConcatConsumers.java",
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
        fs::read(original.join("ScvConcatConsumers.class")).expect("rebuilt original"),
        CLASS
    );
    for directory in [&original, &recovered] {
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp", ".", "ScvConcatConsumers"])
            .current_dir(directory)
            .output()
            .expect("java is installed");
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&run.stdout), EXPECTED_RUN);
    }
}

#[test]
fn exhausted_source_budget_cannot_publish_a_partial_concat_recovery() {
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
fn cancellation_cannot_publish_a_partial_concat_recovery() {
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
