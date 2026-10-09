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
const RETURNED_RUNNER: &str =
    include_str!("fixtures/returned-int-array-compound-updates/Runner.java");
const RETURNED_EXPECTED_STDOUT: &[u8] = b"ok:plain2:35\nok:plain3:14\nok:scalar:14\nok:traced:123:15\nok:outer-null:1\nok:outer-oob:1\nok:null-row:12\nok:inner-oob:12\nok:replace-row:17:100:17\nok:different:11\n";
const RETURNED_LEGS: &[(&str, &[u8])] = &[
    (
        "v8",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/v8/ReturnedIntArrayUpdates.class"
        ),
    ),
    (
        "v23",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/v23/ReturnedIntArrayUpdates.class"
        ),
    ),
];
const TYPED_CONTROL_LEGS: &[(&str, &str, &[u8], bool)] = &[
    (
        "v8/boolean-index",
        "boolean-index",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/controls/v8/boolean-index.class"
        ),
        false,
    ),
    (
        "v8/boolean-rhs",
        "boolean-rhs",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/controls/v8/boolean-rhs.class"
        ),
        false,
    ),
    (
        "v8/char-index",
        "char-index",
        include_bytes!("fixtures/returned-int-array-compound-updates/controls/v8/char-index.class"),
        true,
    ),
    (
        "v8/char-rhs",
        "char-rhs",
        include_bytes!("fixtures/returned-int-array-compound-updates/controls/v8/char-rhs.class"),
        true,
    ),
    (
        "v23/boolean-index",
        "boolean-index",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/controls/v23/boolean-index.class"
        ),
        false,
    ),
    (
        "v23/boolean-rhs",
        "boolean-rhs",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/controls/v23/boolean-rhs.class"
        ),
        false,
    ),
    (
        "v23/char-index",
        "char-index",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/controls/v23/char-index.class"
        ),
        true,
    ),
    (
        "v23/char-rhs",
        "char-rhs",
        include_bytes!("fixtures/returned-int-array-compound-updates/controls/v23/char-rhs.class"),
        true,
    ),
];
// Refusals retain copy/read/store/return and extra-consumer anchors. Stable parameter loads
// are named by the declaration. Historical fallback omits the rejected iadd producer;
// see consumer-source-map-history-v1 and verification-root.md for the separate debt.
const CONSUMER_CONTROL_LEGS: &[(&str, &str, &[u8], &[u32])] = &[
    (
        "v8",
        "sum-extra-consumer",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/consumer-controls/v8/sum-extra-consumer.class"
        ),
        &[2, 3, 6, 7, 8, 9, 10],
    ),
    (
        "v8",
        "returned-copy-extra-consumer",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/consumer-controls/v8/returned-copy-extra-consumer.class"
        ),
        &[2, 3, 6, 7, 8, 9, 10],
    ),
    (
        "v8",
        "long-return",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/consumer-controls/v8/long-return.class"
        ),
        &[2, 3, 6, 7, 8, 9],
    ),
    (
        "v23",
        "sum-extra-consumer",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/consumer-controls/v23/sum-extra-consumer.class"
        ),
        &[2, 3, 6, 7, 8, 9, 10],
    ),
    (
        "v23",
        "returned-copy-extra-consumer",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/consumer-controls/v23/returned-copy-extra-consumer.class"
        ),
        &[2, 3, 6, 7, 8, 9, 10],
    ),
    (
        "v23",
        "long-return",
        include_bytes!(
            "fixtures/returned-int-array-compound-updates/consumer-controls/v23/long-return.class"
        ),
        &[2, 3, 6, 7, 8, 9],
    ),
];
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

fn compile_sources(dir: &Path, source_name: &str) -> Output {
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
        .arg(dir.join(source_name))
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
            if name == "returned" {
                assert!(
                    physical.text.contains("return ") && physical.text.contains(" += "),
                    "{leg}/returned is the proved int array update returned as its new value:\n{}",
                    physical.text
                );
            } else {
                assert!(
                    !physical.text.contains(" += "),
                    "{leg}/merged remains outside the returned update proof:\n{}",
                    physical.text
                );
            }
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
fn returned_int_array_updates_are_structured_and_keep_store_return_origins() {
    for (leg, class_bytes) in RETURNED_LEGS {
        let report = class_source_named(class_bytes, "ReturnedIntArrayUpdates");
        assert_eq!(
            report.methods.len(),
            11,
            "{leg}: retain all physical members"
        );
        assert!(!report.text.contains("@bytecode"), "{leg}: {}", report.text);
        for name in [
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
        ] {
            let method = body(&report, name);
            assert_eq!(method.quality, Quality::Structured, "{leg}/{name}");
            assert_eq!(method.representation, Representation::Java, "{leg}/{name}");
            assert!(
                !method.text.contains("@bytecode"),
                "{leg}/{name}: {}",
                method.text
            );
        }
        for name in ["plain2", "plain3", "scalar", "traced", "replaceRow"] {
            let method = body(&report, name);
            assert!(
                method.text.contains("return ") && method.text.contains(" += "),
                "{leg}/{name} writes and returns the same compound expression:\n{}",
                method.text
            );
        }
        let traced = body(&report, "traced");
        for (call, expected_count) in [("row(", 1), ("index(", 1), ("rhs(", 1)] {
            assert_eq!(
                traced.text.matches(call).count(),
                expected_count,
                "{leg}/traced"
            );
        }
        let replace_row = body(&report, "replaceRow");
        assert_eq!(
            replace_row.text.matches("swap(").count(),
            1,
            "{leg}/replaceRow"
        );
        let different = body(&report, "different");
        assert!(
            !different.text.contains(" += "),
            "{leg}/different: {}",
            different.text
        );

        for (name, store, returns, derived) in [
            ("plain2", 9, 10, &[4u32, 5, 7, 8][..]),
            ("plain3", 12, 13, &[6u32, 7, 10, 11][..]),
            ("scalar", 7, 8, &[2u32, 3, 5, 6][..]),
            ("traced", 18, 19, &[10u32, 11, 16, 17][..]),
            ("replaceRow", 12, 13, &[4u32, 5, 10, 11][..]),
        ] {
            let method = body(&report, name);
            for bci in [store, returns] {
                assert!(
                    !method.source_map.direct_of_bci(bci).is_empty(),
                    "{leg}/{name}: store/return BCI {bci} is directly anchored: {:?}",
                    method.source_map.segments()
                );
            }
            for bci in derived {
                assert!(
                    !method.source_map.derived_of_bci(*bci).is_empty(),
                    "{leg}/{name}: copy/read/add BCI {bci} is retained: {:?}",
                    method.source_map.segments()
                );
            }
        }
    }
}

#[test]
fn returned_int_array_updates_use_java_types_beyond_category_one_frames() {
    for (leg, variant, class_bytes, accepted) in TYPED_CONTROL_LEGS {
        let report = class_source_named(class_bytes, "ReturnedIntArrayUpdates");
        let scalar = body(&report, "scalar");
        if *accepted {
            assert!(
                scalar.text.contains("return ") && scalar.text.contains(" += "),
                "{leg}/{variant}: char is int-compatible in this Java position:\n{}",
                scalar.text
            );
        } else {
            assert!(
                !scalar.text.contains(" += "),
                "{leg}/{variant}: boolean cannot be an array index or compound RHS:\n{}",
                scalar.text
            );
            assert!(
                scalar.text.contains("@bytecode"),
                "{leg}/{variant}: the unsupported typed expression stays quoted:\n{}",
                scalar.text
            );
        }
    }
}

#[test]
fn returned_int_array_updates_reject_extra_consumers_and_non_int_returns() {
    for (leg, variant, class_bytes, bcis) in CONSUMER_CONTROL_LEGS {
        let report = class_source_named(class_bytes, "ReturnedIntArrayUpdates");
        let scalar = body(&report, "scalar");
        if *variant == "long-return" {
            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"scalar")
                .expect("long-return control retains its scalar method");
            assert_eq!(method.item.descriptor.raw().0, b"([III)J");
        }
        assert!(
            !scalar
                .text
                .lines()
                .any(|line| line.contains("return ") && line.contains(" += ")),
            "{leg}/{variant}: unsupported consumer/return shape cannot publish returned +=:\n{}",
            scalar.text
        );
        assert!(
            scalar.text.contains("@bytecode"),
            "{leg}/{variant}: unsupported scalar shape stays explicitly quoted:\n{}",
            scalar.text
        );
        for bci in *bcis {
            assert!(
                !scalar.source_map.of_bci(*bci).is_empty(),
                "{leg}/{variant}: scalar bytecode BCI {bci} remains represented: {:?}",
                scalar.source_map.segments()
            );
        }
    }
}

#[test]
fn returned_int_array_updates_invalid_copy_category_stops_before_java_publication() {
    for (leg, class_bytes) in RETURNED_LEGS {
        // The verified scalar uses three category-1 operands. dup2_x2 cannot replace its
        // dup_x2: it requires four category-1 operands or a category-2 operand.
        let scalar_code = [0x2a, 0x1b, 0x5c, 0x2e, 0x1c, 0x60, 0x5b, 0x4f, 0xac];
        let offsets: Vec<_> = class_bytes
            .windows(scalar_code.len())
            .enumerate()
            .filter_map(|(at, bytes)| (bytes == scalar_code).then_some(at))
            .collect();
        assert_eq!(
            offsets.len(),
            1,
            "{leg}: the scalar Code has one exact occurrence"
        );
        let mut invalid = class_bytes.to_vec();
        invalid[offsets[0] + 6] = 0x5e;
        let report = class_source_named(&invalid, "ReturnedIntArrayUpdates");
        let scalar = body(&report, "scalar");
        assert!(
            matches!(
                scalar.outcome,
                RecoveryOutcome::Stopped(StopReason::IrTableMissing { table: "frames" })
            ),
            "{leg}: invalid stack copy must stop at frames: {:?}",
            scalar.outcome
        );
        assert!(
            scalar.text.is_empty(),
            "{leg}: no Java is published for invalid copies"
        );
        assert!(scalar.source_map.segments().is_empty());
    }
}

#[test]
fn returned_int_array_updates_public_budget_and_cancellation_do_not_publish_partial_claims() {
    for (leg, class_bytes) in RETURNED_LEGS {
        let (snapshot, request) = opened_class_source(class_bytes, "ReturnedIntArrayUpdates");
        let complete = class_source_named(class_bytes, "ReturnedIntArrayUpdates");
        let mut limits = complete.limits.clone();
        limits.analysis_steps = complete
            .usage
            .analysis_steps
            .checked_sub(1)
            .expect("the complete returned-array request used analysis steps");
        let mut budget = Budget::new(limits);
        let bounded = Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut budget,
            )
            .expect("the public budget-stopped request answers");
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
                    if let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome
                        && !matches!(body.execution, ExecutionReport::Complete { .. })
                    {
                        assert!(
                            !(body.text.contains("return ") && body.text.contains(" += ")),
                            "{leg}: a stopped returned proof cannot publish a partial update:\n{}",
                            body.text
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
            .expect("the pre-cancelled public request answers");
        assert!(
            matches!(answer, OperationOutcome::Incomplete(candidates)
                if matches!(candidates.execution, ExecutionReport::Cancelled { .. })),
            "{leg}: cancellation returns no class-source artifact"
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
        let compile = compile_sources(scratch.path(), "NestedIntUpdates.java");
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

#[test]
#[ignore = "requires a real JDK to compile the complete generated class with the fixed Runner and verify behavior"]
fn returned_int_array_updates_compile_and_execute_as_a_complete_class() {
    for (leg, class_bytes) in RETURNED_LEGS {
        let report = class_source_named(class_bytes, "ReturnedIntArrayUpdates");
        let scratch = Scratch::new(&format!("returned-{leg}"));
        fs::write(
            scratch.path().join("ReturnedIntArrayUpdates.java"),
            &report.text,
        )
        .expect("complete returned-array source is written");
        fs::write(scratch.path().join("Runner.java"), RETURNED_RUNNER)
            .expect("fixed complete-class runner is written");
        let compile = compile_sources(scratch.path(), "ReturnedIntArrayUpdates.java");
        assert!(
            compile.status.success(),
            "{leg} complete generated class did not compile:\n{}\n{}",
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = run_verified(&scratch.path().join("classes"));
        assert!(
            run.status.success(),
            "{leg} verified complete-class execution failed:\n{}\n{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            run.stdout, RETURNED_EXPECTED_STDOUT,
            "{leg} stdout differs from oracle"
        );
        assert_eq!(
            run.stderr, EXPECTED_STDERR,
            "{leg} stderr differs from oracle"
        );
    }
}
