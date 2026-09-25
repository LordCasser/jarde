//! Wrapped array reads may move into an enhanced-for binding only before observable work.

use jarde::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/array-wrapped-binding/ArrayWrappedBinding.class"
);
const RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/array-wrapped-binding/ArrayWrappedBindingRunner.java"
);

fn opened() -> (artifact::ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ArrayWrappedBinding"),
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

fn recover(evidence: RecoveryEvidenceRequest) -> ClassSourceReport {
    let (snapshot, request) = opened();
    let mut budget = task_budget(&[]).unwrap();
    match Engine::new()
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, &evidence, &mut budget)
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("complete class source expected: {other:?}"),
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
fn only_array_first_wrappers_project() {
    let report = recover(RecoveryEvidenceRequest::essential());
    for (name, read_bci) in [("plusArrayFirst", 19), ("wrappedOnlyRead", 20)] {
        let text = &member(&report, name).text;
        let header = format!("for (int arrayElement{read_bci} : local2)");
        assert!(text.contains(&header), "{name}:\n{text}");
        assert!(
            text.contains(&format!("arrayElement{read_bci} + tick()")),
            "{text}"
        );
        assert_eq!(text.matches("tick()").count(), 1, "{text}");
        assert!(!text.contains("local3 = local2.length"), "{text}");
        assert!(!text.contains("local2[local4]"), "{text}");
        assert!(
            !text.contains("int local3;"),
            "unused length declaration remains:\n{text}"
        );
        assert!(
            !text.contains("int local4;"),
            "unused induction declaration remains:\n{text}"
        );
    }
    for name in [
        "plusTickFirst",
        "callTickThenArray",
        "callMutateThenArray",
        "readAfterEffect",
        "twoReadsWithMutation",
    ] {
        let text = &member(&report, name).text;
        assert!(!text.contains(" : local2)"), "{name} moved a read:\n{text}");
        assert!(text.contains("for (local4 = 0;"), "{name}:\n{text}");
        assert!(text.contains("local2[local4]"), "{name}:\n{text}");
    }
}

#[test]
fn wrapped_read_origins_and_essential_all_bodies_agree() {
    let essential = recover(RecoveryEvidenceRequest::essential());
    let all = recover(RecoveryEvidenceRequest::all());
    assert_eq!(essential.text, all.text);
    for (name, bcis) in [
        (
            "plusArrayFirst",
            &[5, 6, 7, 8, 10, 12, 13, 16, 17, 19, 20, 23, 24, 31][..],
        ),
        (
            "wrappedOnlyRead",
            &[5, 6, 7, 8, 10, 12, 13, 16, 17, 18, 20, 21, 24, 25, 27][..],
        ),
    ] {
        let method = member(&all, name);
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("{name} did not recover");
        };
        for bci in bcis {
            assert!(
                !report.source_map.text_of_bci(&report.text, *bci).is_empty(),
                "{name} lost BCI {bci}"
            );
        }
    }
}

fn compile(dir: &Path, files: &[&str]) {
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-Xlint:-options")
        .arg("-d")
        .arg(dir)
        .arg("-cp")
        .arg(dir)
        .args(files.iter().map(|file| dir.join(file)))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run(dir: &Path) -> Vec<String> {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg("ArrayWrappedBindingRunner")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| match line.split_once("NullPointerException:") {
            Some((before, after)) => format!(
                "{before}NullPointerException{}",
                after
                    .split_once(",calls=")
                    .map(|(_, rest)| format!(",calls={rest}"))
                    .unwrap_or_default()
            ),
            None => line.to_owned(),
        })
        .collect()
}

#[test]
fn complete_java8_class_preserves_all_thirteen_observations() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "jarde-wrapped-array-{}-{nonce}",
        std::process::id()
    ));
    let original = root.join("original");
    let recovered = root.join("recovered");
    fs::create_dir_all(&original).unwrap();
    fs::create_dir_all(&recovered).unwrap();
    fs::write(original.join("ArrayWrappedBinding.class"), CLASS).unwrap();
    fs::write(original.join("ArrayWrappedBindingRunner.java"), RUNNER).unwrap();
    compile(&original, &["ArrayWrappedBindingRunner.java"]);
    fs::write(
        recovered.join("ArrayWrappedBinding.java"),
        recover(RecoveryEvidenceRequest::essential()).text,
    )
    .unwrap();
    fs::write(recovered.join("ArrayWrappedBindingRunner.java"), RUNNER).unwrap();
    compile(
        &recovered,
        &["ArrayWrappedBinding.java", "ArrayWrappedBindingRunner.java"],
    );
    let expected = run(&original);
    let actual = run(&recovered);
    assert_eq!(expected.len(), 13);
    assert_eq!(actual, expected);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn low_budget_and_cancellation_do_not_publish_half_folds() {
    let complete = recover(RecoveryEvidenceRequest::essential());
    let (snapshot, request) = opened();
    let mut limits = complete.limits.clone();
    limits.analysis_steps = complete.usage.analysis_steps - 1;
    let mut low = Budget::new(limits);
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential(),
            &mut low,
        )
        .unwrap();
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
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: BudgetDimension::AnalysisSteps
                    },
                    ..
                }
            ));
            for name in ["plusArrayFirst", "wrappedOnlyRead"] {
                let text = &member(&report, name).text;
                if text.contains(" : local2)") {
                    assert!(!text.contains("local3 = local2.length"), "{text}");
                    assert!(!text.contains("local2[local4]"), "{text}");
                    assert_eq!(text.matches("tick()").count(), 1, "{text}");
                }
            }
        }
        other => panic!("unexpected bounded outcome: {other:?}"),
    }
    let mut cancelled = task_budget(&[]).unwrap();
    cancelled.cancellation_token().cancel();
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential(),
            &mut cancelled,
        )
        .unwrap();
    match outcome {
        OperationOutcome::Incomplete(candidates) => assert!(matches!(
            candidates.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => assert!(report.text.is_empty()),
        other => panic!("unexpected cancelled outcome: {other:?}"),
    }
}
