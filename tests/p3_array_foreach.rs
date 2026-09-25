//! The array projection keeps its old counted-loop refusal path and the original class behavior.

use jarde::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const INTS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/enhanced-for/int-array/IntArrayForeach.class"
);
const OBJECTS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-22/enhanced-for/object-array-body-if-red/ObjectArrayForeach.class"
);
const REFUSALS: &[u8] =
    include_bytes!("fixtures/p3-enhanced-for-array/v8/ArrayForeachRefusal.class");
const TRANSFERS: &[u8] =
    include_bytes!("fixtures/p3-enhanced-for-array/v8/ArrayForeachTransfers.class");
const INT_RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-22/enhanced-for/IntArrayForeachRunner.java"
);
const OBJECT_RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-22/enhanced-for/ObjectArrayForeachRunner.java"
);
const REFUSAL_RUNNER: &str =
    include_str!("fixtures/p3-enhanced-for-array/ArrayForeachRefusalRunner.java");
const TRANSFER_RUNNER: &str =
    include_str!("fixtures/p3-enhanced-for-array/ArrayForeachTransfersRunner.java");
const REUSE_SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/foreach-cache-declarations/ReuseAfterForEach.java"
);
const REUSE_RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/foreach-cache-declarations/ReuseAfterForEachRunner.java"
);

fn source(bytes: &[u8], class: &str, evidence: RecoveryEvidenceRequest) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("bounded task defaults");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the frozen class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
    match Engine::new()
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, &evidence, &mut budget)
        .expect("class source answers a single class")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one class source is expected: {other:?}"),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing {name}"))
}

fn compile_reuse_fixture() -> Vec<u8> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time follows the epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "jarde-foreach-reuse-fixture-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("temporary fixture directory is created");
    fs::write(dir.join("ReuseAfterForEach.java"), REUSE_SOURCE)
        .expect("the checked-in source is written unchanged");
    let output = Command::new("javac")
        .args(["--release", "8", "-g", "-Xlint:-options", "-d"])
        .arg(&dir)
        .arg(dir.join("ReuseAfterForEach.java"))
        .output()
        .expect("javac is available for the Java 8 fixture");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes =
        fs::read(dir.join("ReuseAfterForEach.class")).expect("javac wrote the source class");
    fs::remove_dir_all(dir).expect("temporary fixture files are removed");
    bytes
}

#[test]
fn primitive_and_reference_arrays_project_but_unsafe_shapes_do_not() {
    for (bytes, class, methods, expected) in [
        (
            INTS,
            "IntArrayForeach",
            ["sum", "sumFrom"],
            "for (int local5 : local2)",
        ),
        (
            OBJECTS,
            "ObjectArrayForeach",
            ["sumHash", "sumHashFrom"],
            "for (java.lang.Object local5 : local2)",
        ),
    ] {
        let report = source(bytes, class, RecoveryEvidenceRequest::essential());
        for name in methods {
            let text = &member(&report, name).text;
            assert!(text.contains(expected), "{class}.{name}:\n{text}");
            assert_eq!(
                text.matches(".get()").count(),
                usize::from(name.ends_with("From")),
                "{text}"
            );
            assert!(!text.contains("local3 = local2.length"), "{text}");
            assert!(!text.contains("local4 = local4 + 1"), "{text}");
            assert!(
                !text.contains("int local3;"),
                "unused length declaration remains:\n{text}"
            );
            assert!(
                !text.contains("int local4;"),
                "unused induction declaration remains:\n{text}"
            );
        }
    }
    let refusals = source(
        REFUSALS,
        "ArrayForeachRefusal",
        RecoveryEvidenceRequest::essential(),
    );
    for name in ["indexInBody", "differentArrays", "escapedElement"] {
        let text = &member(&refusals, name).text;
        assert!(!text.contains(" : "), "{name} was folded:\n{text}");
        assert!(
            text.contains("for ("),
            "{name} lost its counted loop:\n{text}"
        );
        assert!(
            !text.contains("@bytecode"),
            "{name} became a fallback:\n{text}"
        );
    }
    assert!(
        member(&refusals, "effectInBinding")
            .text
            .contains("for (int arrayElement19 : local2)"),
        "the array read precedes tick() and is eligible for wrapped binding"
    );
    assert!(!member(&refusals, "indexAfterLoop").text.contains(" : "));
    let transfers = source(
        TRANSFERS,
        "ArrayForeachTransfers",
        RecoveryEvidenceRequest::essential(),
    );
    assert!(
        member(&transfers, "skipNegative")
            .text
            .contains("for (int local5 : local2)")
    );
    assert!(!member(&transfers, "stopAtNegative").text.contains(" : "));
}

#[test]
fn same_slot_later_local_declaration_survives_array_projection() {
    let bytes = compile_reuse_fixture();
    let report = source(
        &bytes,
        "ReuseAfterForEach",
        RecoveryEvidenceRequest::essential(),
    );
    let text = &member(&report, "sumThenReuse").text;
    assert!(
        text.contains(" : "),
        "the proved array loop remains:\n{text}"
    );
    assert!(
        text.contains("int second;"),
        "slot 3's later local declaration was removed:\n{text}"
    );
    assert!(
        text.contains("int first = sum + 1;"),
        "slot 2's later int still shares the array declaration:\n{text}"
    );
    assert!(
        text.contains("second = first + 2;"),
        "the later slot 3 use remains:\n{text}"
    );
}

#[test]
fn reused_array_and_int_slot_recompile_with_and_without_debug_names() {
    for debug in ["-g", "-g:none"] {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "jarde-reused-array-slot-{}-{nonce}",
            std::process::id()
        ));
        let original = root.join("original");
        let recovered = root.join("recovered");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&recovered).unwrap();
        fs::write(original.join("ReuseAfterForEach.java"), REUSE_SOURCE).unwrap();
        fs::write(original.join("ReuseAfterForEachRunner.java"), REUSE_RUNNER).unwrap();
        let compile_original = Command::new("javac")
            .args(["--release", "8", debug, "-Xlint:-options", "-d"])
            .arg(&original)
            .arg(original.join("ReuseAfterForEach.java"))
            .arg(original.join("ReuseAfterForEachRunner.java"))
            .output()
            .unwrap();
        assert!(
            compile_original.status.success(),
            "{}",
            String::from_utf8_lossy(&compile_original.stderr)
        );
        let bytes = fs::read(original.join("ReuseAfterForEach.class")).unwrap();
        let report = source(
            &bytes,
            "ReuseAfterForEach",
            RecoveryEvidenceRequest::essential(),
        );
        let all = source(&bytes, "ReuseAfterForEach", RecoveryEvidenceRequest::all());
        assert_eq!(report.text, all.text, "{debug} evidence changed the source");
        let ClassSourceOutcome::Recovered {
            report: body_report,
            ..
        } = &member(&all, "sumThenReuse").outcome
        else {
            panic!("{debug} did not recover the method body")
        };
        for bci in [3, 4, 16, 36, 37, 40, 42, 44] {
            assert!(
                !body_report
                    .source_map
                    .text_of_bci(&body_report.text, bci)
                    .is_empty(),
                "{debug} lost source BCI {bci}"
            );
        }
        let method = &member(&report, "sumThenReuse").text;
        assert!(method.contains("for (int "), "{debug}:\n{method}");
        assert!(method.contains("int[] local2;"), "{debug}:\n{method}");
        assert!(
            if debug == "-g" {
                method.contains("int first = sum + 1;")
            } else {
                method.contains("int local2_2 = local1 + 1;")
            },
            "{debug}:\n{method}"
        );
        fs::write(recovered.join("ReuseAfterForEach.java"), report.text).unwrap();
        fs::write(recovered.join("ReuseAfterForEachRunner.java"), REUSE_RUNNER).unwrap();
        compile(
            &recovered,
            &["ReuseAfterForEach.java", "ReuseAfterForEachRunner.java"],
        );
        assert_eq!(
            run(&original, "ReuseAfterForEachRunner"),
            run(&recovered, "ReuseAfterForEachRunner"),
            "{debug} changed values or the null exception"
        );
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn projected_origins_and_body_are_identical_under_essential_and_all_evidence() {
    for (bytes, class, cases) in [
        (
            INTS,
            "IntArrayForeach",
            [
                (
                    "sum",
                    &[3, 4, 5, 6, 7, 8, 10, 12, 13, 16, 17, 19, 20, 27][..],
                ),
                (
                    "sumFrom",
                    &[3, 8, 11, 12, 13, 14, 15, 16, 18, 20, 21, 24, 25, 27, 28, 35][..],
                ),
            ],
        ),
        (
            OBJECTS,
            "ObjectArrayForeach",
            [
                (
                    "sumHash",
                    &[3, 4, 5, 6, 7, 8, 10, 12, 13, 16, 17, 19, 20, 35][..],
                ),
                (
                    "sumHashFrom",
                    &[3, 8, 11, 12, 13, 14, 15, 16, 18, 20, 21, 24, 25, 27, 28, 43][..],
                ),
            ],
        ),
    ] {
        let essential = source(bytes, class, RecoveryEvidenceRequest::essential());
        let all = source(bytes, class, RecoveryEvidenceRequest::all());
        assert_eq!(essential.text, all.text, "{class}");
        for (name, bcis) in cases {
            let method = member(&all, name);
            let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
                panic!("{class}.{name} did not recover")
            };
            for bci in bcis {
                assert!(
                    !report.source_map.text_of_bci(&report.text, *bci).is_empty(),
                    "{class}.{name} lost BCI {bci}"
                );
            }
        }
    }
}

#[test]
fn low_work_budget_and_cancellation_publish_no_partial_array_loop() {
    let complete = source(
        INTS,
        "IntArrayForeach",
        RecoveryEvidenceRequest::essential(),
    );
    let mut open_budget = task_budget(&[]).unwrap();
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(INTS.to_vec()), &mut open_budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("IntArrayForeach"),
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
            for method in &report.methods {
                if let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome {
                    assert!(body.produced() || body.text.is_empty(), "{}", body.text);
                    if body.text.contains("for (int local5 : local2)") {
                        assert!(!body.text.contains("local3 = local2.length"));
                        assert!(!body.text.contains("local4 = local4 + 1"));
                        assert!(!body.text.contains("local2[local4]"));
                    }
                }
            }
        }
        other => panic!("unexpected bounded result: {other:?}"),
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
        OperationOutcome::Performed(report) => {
            assert!(report.text.is_empty());
        }
        other => panic!("unexpected cancelled result: {other:?}"),
    }
}

fn compile(directory: &Path, sources: &[&str]) {
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-Xlint:-options")
        .arg("-d")
        .arg(directory)
        .arg("-cp")
        .arg(directory)
        .args(sources.iter().map(|source| directory.join(source)))
        .output()
        .expect("javac is available for the Java 8 execution check");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run(directory: &Path, runner: &str) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(directory)
        .arg(runner)
        .output()
        .expect("the Java 8 runner starts");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("runner output is UTF-8")
}

#[test]
fn complete_java8_classes_keep_supplier_and_element_execution_order() {
    for (bytes, class, runner, runner_source) in [
        (INTS, "IntArrayForeach", "IntArrayForeachRunner", INT_RUNNER),
        (
            OBJECTS,
            "ObjectArrayForeach",
            "ObjectArrayForeachRunner",
            OBJECT_RUNNER,
        ),
        (
            REFUSALS,
            "ArrayForeachRefusal",
            "ArrayForeachRefusalRunner",
            REFUSAL_RUNNER,
        ),
        (
            TRANSFERS,
            "ArrayForeachTransfers",
            "ArrayForeachTransfersRunner",
            TRANSFER_RUNNER,
        ),
    ] {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "jarde-array-foreach-{}-{nonce}",
            std::process::id()
        ));
        let original = root.join("original");
        let recovered = root.join("recovered");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&recovered).unwrap();
        fs::write(original.join(format!("{class}.class")), bytes).unwrap();
        fs::write(original.join(format!("{runner}.java")), runner_source).unwrap();
        compile(&original, &[&format!("{runner}.java")]);
        let source = source(bytes, class, RecoveryEvidenceRequest::essential());
        fs::write(recovered.join(format!("{class}.java")), source.text).unwrap();
        fs::write(recovered.join(format!("{runner}.java")), runner_source).unwrap();
        compile(
            &recovered,
            &[&format!("{class}.java"), &format!("{runner}.java")],
        );
        let original = run(&original, runner);
        let recovered = run(&recovered, runner);
        let shape = |output: &str| {
            output
                .lines()
                .map(|line| {
                    if let Some((prefix, _)) = line.split_once("NullPointerException:") {
                        format!("{prefix}NullPointerException")
                    } else {
                        line.to_owned()
                    }
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(shape(&original), shape(&recovered), "{class}");
        fs::remove_dir_all(&root).expect("temporary Java classes are removed");
    }
}
