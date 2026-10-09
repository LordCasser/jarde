//! Evidence-gated int compound updates through nested array row loads.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
const RUNNER: &str = include_str!("fixtures/nested-int-array-compound-updates/Runner.java");
const EXPECTED_STDOUT: &[u8] = b"ok:plain2:35\nok:plain3:14\nok:scalar:14\nok:traced:123:15\nok:outer-null:1\nok:outer-oob:1\nok:null-row:12\nok:inner-oob:12\nok:replace-row:17:100:7\nok:different:11\n";
const EXPECTED_STDERR: &[u8] = b"";
const LEGS: &[(&str, &[u8])] = &[
    (
        "v8",
        include_bytes!("fixtures/nested-int-array-compound-updates/v8/NestedIntUpdates.class"),
    ),
    (
        "v23",
        include_bytes!("fixtures/nested-int-array-compound-updates/v23/NestedIntUpdates.class"),
    ),
];
const BOUNDARY_LEGS: &[(&str, &[u8])] = &[
    (
        "v8",
        include_bytes!("fixtures/nested-int-array-compound-updates/v8/NestedIntBoundaries.class"),
    ),
    (
        "v23",
        include_bytes!("fixtures/nested-int-array-compound-updates/v23/NestedIntBoundaries.class"),
    ),
];
const METHODS: &[&str] = &[
    "<init>",
    "plain2",
    "plain3",
    "scalar",
    "traced",
    "row",
    "index",
    "rhs",
    "swap",
    "replaceRow",
    "different",
];

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-nested-int-array-compound-{}-{label}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("private fixture scratch directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn opened_class_source(
    class_bytes: &[u8],
    class_name: &str,
) -> (ArtifactSnapshot, ClassSourceRequest) {
    let snapshot = Engine::new()
        .open(
            ArtifactInput::bytes(class_bytes.to_vec()),
            &mut task_budget(&[]).expect("task defaults are bounded"),
        )
        .expect("one fixture class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name),
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

fn class_source_named(class_bytes: &[u8], class_name: &str) -> ClassSourceReport {
    let (snapshot, request) = opened_class_source(class_bytes, class_name);
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut task_budget(&[]).expect("task defaults are bounded"),
        )
        .expect("valid fixture class-source request answers")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("fixture class source did not complete: {other:?}"),
    }
}

fn class_source(class_bytes: &[u8]) -> ClassSourceReport {
    class_source_named(class_bytes, "NestedIntUpdates")
}

fn body<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("class has no method `{name}`"));
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("{name} has no complete recovery record: {other:?}"),
    }
}

fn assert_nested_updates(report: &ClassSourceReport, leg: &str) {
    assert!(
        !report.text.contains("@bytecode"),
        "{leg}: class source contains a bytecode fallback:\n{}",
        report.text
    );
    for name in METHODS {
        let method_body = body(report, name);
        assert_eq!(
            method_body.quality,
            Quality::Structured,
            "{leg}/{name}: {method_body:?}"
        );
        assert_eq!(
            method_body.representation,
            Representation::Java,
            "{leg}/{name}"
        );
        assert!(
            !method_body.text.contains("@bytecode"),
            "{leg}/{name}: {}",
            method_body.text
        );
    }

    for name in ["plain2", "plain3", "scalar", "traced", "replaceRow"] {
        assert!(
            body(report, name).text.contains(" += "),
            "{leg}/{name} must remain a compound update:\n{}",
            body(report, name).text
        );
    }
    let traced = body(report, "traced");
    for (call, expected_count) in [("row(", 1), ("index(", 1), ("rhs(", 1)] {
        assert_eq!(
            traced.text.matches(call).count(),
            expected_count,
            "{leg}/traced preserves one {call} evaluation:\n{}",
            traced.text
        );
    }
    let replace_row = body(report, "replaceRow");
    assert_eq!(
        replace_row.text.matches("swap(").count(),
        1,
        "{leg}: {replace_row:?}"
    );
    let different = body(report, "different");
    assert!(
        !different.text.contains(" += "),
        "{leg}/different must not fold separate row reads and writes:\n{}",
        different.text
    );
    assert!(
        different.text.contains(" = ") && different.text.matches("[0]").count() >= 1,
        "{leg}/different keeps ordinary assignment syntax:\n{}",
        different.text
    );
}

fn javac_tool() -> PathBuf {
    std::env::var_os("JARDE_JAVAC23")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("javac"))
}

fn java_tool() -> PathBuf {
    std::env::var_os("JARDE_JAVA23")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("java"))
}

fn compile_sources(dir: &Path) -> Output {
    let empty = dir.join("empty-classpath-sourcepath");
    let classes = dir.join("classes");
    fs::create_dir_all(&empty).expect("empty classpath/sourcepath exists");
    fs::create_dir_all(&classes).expect("isolated classes directory exists");
    Command::new(javac_tool())
        .args([
            "-source",
            "8",
            "-target",
            "8",
            "-g:none",
            "-Xlint:-options",
            "-classpath",
        ])
        .env_remove("JAVA_TOOL_OPTIONS")
        .env_remove("_JAVA_OPTIONS")
        .env_remove("JDK_JAVA_OPTIONS")
        .env_remove("CLASSPATH")
        .arg(&empty)
        .arg("-sourcepath")
        .arg(&empty)
        .arg("-d")
        .arg(&classes)
        .arg(dir.join("NestedIntUpdates.java"))
        .arg(dir.join("Runner.java"))
        .current_dir(dir)
        .output()
        .expect("javac is available for Java 8 compilation")
}

fn run_verified(classes: &Path) -> Output {
    Command::new(java_tool())
        .arg("-Xverify:all")
        .env_remove("JAVA_TOOL_OPTIONS")
        .env_remove("_JAVA_OPTIONS")
        .env_remove("JDK_JAVA_OPTIONS")
        .env_remove("CLASSPATH")
        .arg("-cp")
        .arg(classes)
        .arg("Runner")
        .output()
        .expect("java is available for verifier execution")
}

#[test]
fn nested_int_array_updates_are_structured_and_preserve_lvalue_boundaries() {
    for (leg, class_bytes) in LEGS {
        let report = class_source(class_bytes);
        assert_nested_updates(&report, leg);
    }
}

#[test]
fn nested_int_array_boundaries_do_not_claim_unproved_compound_updates() {
    for (leg, class_bytes) in BOUNDARY_LEGS {
        let report = class_source_named(class_bytes, "NestedIntBoundaries");
        assert_eq!(
            report.methods.len(),
            3,
            "{leg}: keep constructor and both methods"
        );
        for name in ["<init>", "returned", "merged"] {
            assert!(
                report
                    .methods
                    .iter()
                    .any(|method| method.item.name.raw().0 == name.as_bytes()),
                "{leg}: missing physical member {name}"
            );
        }

        for (name, bcis) in [
            ("returned", &[2u32, 4, 5, 8, 9, 10][..]),
            ("merged", &[18u32, 19, 22][..]),
        ] {
            let physical = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == name.as_bytes())
                .expect("the physical method remains in the class report");
            let ClassSourceOutcome::Recovered {
                report: recovery, ..
            } = &physical.outcome
            else {
                panic!(
                    "{leg}/{name}: missing recovery record: {:?}",
                    physical.outcome
                )
            };
            assert!(
                !physical.text.contains(" += "),
                "{leg}/{name}: the dup_x2 return / row-Phi boundary is not this proven compound update:\n{}",
                physical.text
            );
            for bci in bcis {
                assert!(
                    !recovery
                        .source_map
                        .text_of_bci(&recovery.text, *bci)
                        .is_empty(),
                    "{leg}/{name}: lost observable boundary BCI {bci}: {:?}\n{}",
                    recovery.source_map.segments(),
                    physical.text
                );
            }
        }

        let merged = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"merged")
            .expect("merged method is retained");
        let ClassSourceOutcome::Recovered {
            report: recovery, ..
        } = &merged.outcome
        else {
            unreachable!("the merged recovery record was checked above")
        };
        assert!(
            !recovery.source_map.direct_of_bci(18).is_empty(),
            "{leg}/merged: the row-Phi Other stays directly anchored: {:?}",
            recovery.source_map.segments()
        );
        for bci in [
            0u32, 1, 4, 5, 6, 7, 10, 11, 12, 13, 15, 17, 18, 19, 20, 21, 22, 23,
        ] {
            assert!(
                !recovery.source_map.derived_of_bci(bci).is_empty(),
                "{leg}/merged: the row-read closure lost derived BCI {bci}: {:?}",
                recovery.source_map.segments()
            );
        }
        assert!(
            !recovery.source_map.direct_of_bci(22).is_empty(),
            "{leg}/merged: the final update consumer stays directly anchored: {:?}",
            recovery.source_map.segments()
        );
    }
}

#[test]
fn nested_int_array_class_source_stops_do_not_claim_complete_members() {
    for (leg, class_bytes) in LEGS {
        let (snapshot, request) = opened_class_source(class_bytes, "NestedIntUpdates");
        let complete = class_source(class_bytes);
        let mut limits = complete.limits.clone();
        limits.analysis_steps = complete
            .usage
            .analysis_steps
            .checked_sub(1)
            .expect("the completed class-source request used analysis steps");
        let mut budget = Budget::new(limits);
        let bounded = Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget,
            )
            .expect("the budget-stopped class-source operation answers");
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
            OperationOutcome::Performed(report) => {
                assert!(
                    matches!(
                        report.execution,
                        ExecutionReport::Partial {
                            reason: TerminationReason::BudgetExceeded {
                                dimension: BudgetDimension::AnalysisSteps
                            },
                            ..
                        }
                    ),
                    "{leg}: stopped class presentation cannot claim completion"
                );
                for method in &report.methods {
                    let stopped = match &method.outcome {
                        ClassSourceOutcome::Recovered { report: body, .. } => {
                            !matches!(body.execution, ExecutionReport::Complete { .. })
                        }
                        ClassSourceOutcome::Refused { .. } => true,
                        _ => false,
                    };
                    if stopped {
                        assert!(
                            !method.text.contains(" += "),
                            "{leg}: a stopped method cannot publish a partial compound update:\n{}",
                            method.text
                        );
                    }
                }
            }
            other => panic!("{leg}: unexpected budget-stopped result: {other:?}"),
        }

        let mut cancelled = task_budget(&[]).expect("task defaults are bounded");
        cancelled.cancellation_token().cancel();
        let answer = Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut cancelled,
            )
            .expect("the pre-cancelled class-source operation answers");
        assert!(
            matches!(answer, OperationOutcome::Incomplete(candidates)
                if matches!(candidates.execution, ExecutionReport::Cancelled { .. })),
            "{leg}: pre-cancellation returns no class-source artifact"
        );
    }
}

#[test]
#[ignore = "requires a real JDK to compile the complete generated class with the fixed Runner and verify behavior"]
fn nested_int_array_updates_match_the_frozen_runner_oracle() {
    for (leg, class_bytes) in LEGS {
        let report = class_source(class_bytes);
        assert_nested_updates(&report, leg);
        let scratch = Scratch::new(leg);
        fs::write(scratch.path().join("NestedIntUpdates.java"), &report.text)
            .expect("complete generated source is written");
        fs::write(scratch.path().join("Runner.java"), RUNNER)
            .expect("fixed external Runner source is written");
        let compile = compile_sources(scratch.path());
        assert!(
            compile.status.success(),
            "{leg} complete generated sources did not compile:\n{}\n{}",
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = run_verified(&scratch.path().join("classes"));
        assert!(
            run.status.success(),
            "{leg} generated source verifier run failed:\n{}\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            run.stdout, EXPECTED_STDOUT,
            "{leg} stdout differs from oracle"
        );
        assert_eq!(
            run.stderr, EXPECTED_STDERR,
            "{leg} stderr differs from oracle"
        );
    }
}
