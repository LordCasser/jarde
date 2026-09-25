//! Direct Iterable.iterator projection keeps next/cast order and refuses unsupported owners.

use jarde::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const RAW_DEBUG: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/original/RawIterableIterator.debug.class"
);
const RAW_NODEBUG: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/original/RawIterableIterator.nodebug.class"
);
const GENERIC_DEBUG: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/iterable-raw-projection/jadx-iterable-for-claim/IterableForEach.class"
);
const GENERIC_NODEBUG: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/iterable-raw-projection/jadx-iterable-for-claim/debug-control/IterableForEach.nodebug.class"
);
const CURSOR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/original/NamedIteratorCursor.nodebug.class"
);
const PROTECTED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/original/CatchNextScope.nodebug.class"
);
const RAW_RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/iterable-raw-projection/remaining-4-1/runners/RawIterableIteratorRunner.java"
);
const GENERIC_RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/iterable-raw-projection/jadx-iterable-for-claim/IterableForEachRunner.java"
);
const BOUNDARY: &str = include_str!("fixtures/p3-iterable-foreach/IterableBoundary.java");
const BOUNDARY_RUNNER: &str =
    include_str!("fixtures/p3-iterable-foreach/IterableBoundaryRunner.java");

fn opened(bytes: &[u8], class: &str) -> (artifact::ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .unwrap();
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
    (snapshot, request)
}

fn recover(bytes: &[u8], class: &str, evidence: RecoveryEvidenceRequest) -> ClassSourceReport {
    let (snapshot, request) = opened(bytes, class);
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

fn compile(dir: &Path, sources: &[&str]) {
    compile_with_debug(dir, sources, false);
}

fn compile_with_debug(dir: &Path, sources: &[&str], debug: bool) {
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg(if debug { "-g" } else { "-g:none" })
        .arg("-Xlint:-options")
        .arg("-cp")
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .args(sources.iter().map(|source| dir.join(source)))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run(dir: &Path, runner: &str) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(runner)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn direct_iterable_projects_with_or_without_debug_names_and_preserves_cast() {
    for (bytes, class, method) in [
        (RAW_DEBUG, "RawIterableIterator", "rawCast"),
        (RAW_NODEBUG, "RawIterableIterator", "rawCast"),
        (GENERIC_DEBUG, "IterableForEach", "test"),
        (GENERIC_NODEBUG, "IterableForEach", "test"),
    ] {
        let report = recover(bytes, class, RecoveryEvidenceRequest::essential());
        let text = &member(&report, method).text;
        assert!(
            text.contains("for (java.lang.Object iteratorElement"),
            "{class}.{method}:\n{text}"
        );
        assert!(text.contains("(java.lang.String)"), "{text}");
        assert!(!text.contains(".iterator()"), "{text}");
        assert!(!text.contains(".next()"), "{text}");
    }
    for bytes in [RAW_DEBUG, RAW_NODEBUG] {
        let report = recover(
            bytes,
            "RawIterableIterator",
            RecoveryEvidenceRequest::essential(),
        );
        for name in [
            "nextTwiceCastBeforeTouch",
            "nextTwiceTouchBeforeCast",
            "touchBeforeNext",
        ] {
            let text = &member(&report, name).text;
            assert!(text.contains("while ("), "{name}:\n{text}");
            assert!(!text.contains("for (java.lang.Object "), "{name}:\n{text}");
            assert!(text.contains(".next()"), "{name}:\n{text}");
        }
    }
    let cursor = recover(
        CURSOR,
        "NamedIteratorCursor",
        RecoveryEvidenceRequest::essential(),
    );
    assert!(!cursor.text.contains(" : "));
    let protected = recover(
        PROTECTED,
        "CatchNextScope",
        RecoveryEvidenceRequest::essential(),
    );
    assert!(!protected.text.contains(" : "));
}

#[test]
fn subtype_owners_project_and_extra_next_or_iterator_escape_refuse() {
    let root = scratch("boundary");
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("IterableBoundary.java"), BOUNDARY).unwrap();
    fs::write(root.join("IterableBoundaryRunner.java"), BOUNDARY_RUNNER).unwrap();
    compile(
        &root,
        &["IterableBoundary.java", "IterableBoundaryRunner.java"],
    );
    let bytes = fs::read(root.join("IterableBoundary.class")).unwrap();
    let report = recover(
        &bytes,
        "IterableBoundary",
        RecoveryEvidenceRequest::essential(),
    );
    let positive = &member(&report, "capturedOnce").text;
    assert!(
        positive.contains("for (java.lang.Object iteratorElement"),
        "{positive}"
    );
    assert_eq!(positive.matches("supply()").count(), 1, "{positive}");
    let transfer = &member(&report, "skipEmpty").text;
    assert!(
        transfer.contains("for (java.lang.Object iteratorElement"),
        "{transfer}"
    );
    for name in ["listOwner", "collectionOwner"] {
        let text = &member(&report, name).text;
        assert!(
            text.contains("for (java.lang.Object iteratorElement"),
            "{name}:\n{text}"
        );
        assert!(text.contains("(java.lang.String)"), "{name}:\n{text}");
    }
    for name in ["twoNext", "iteratorEscapes"] {
        let text = &member(&report, name).text;
        assert!(text.contains("while ("), "{name}:\n{text}");
        assert!(!text.contains("for (java.lang.Object "), "{name}:\n{text}");
    }
    let recovered = root.join("recovered");
    fs::create_dir_all(&recovered).unwrap();
    fs::write(recovered.join("IterableBoundary.java"), report.text).unwrap();
    fs::write(
        recovered.join("IterableBoundaryRunner.java"),
        BOUNDARY_RUNNER,
    )
    .unwrap();
    compile(
        &recovered,
        &["IterableBoundary.java", "IterableBoundaryRunner.java"],
    );
    assert_eq!(
        run(&root, "IterableBoundaryRunner"),
        run(&recovered, "IterableBoundaryRunner")
    );
    fs::remove_dir_all(root).unwrap();
}

fn scratch(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "jarde-iterable-{label}-{}-{nonce}",
        std::process::id()
    ))
}

#[test]
fn complete_java8_debug_and_nodebug_classes_match_original_execution() {
    for (bytes, class, runner, runner_source, rows) in [
        (
            RAW_DEBUG,
            "RawIterableIterator",
            "RawIterableIteratorRunner",
            RAW_RUNNER,
            6,
        ),
        (
            RAW_NODEBUG,
            "RawIterableIterator",
            "RawIterableIteratorRunner",
            RAW_RUNNER,
            6,
        ),
        (
            GENERIC_DEBUG,
            "IterableForEach",
            "IterableForEachRunner",
            GENERIC_RUNNER,
            4,
        ),
        (
            GENERIC_NODEBUG,
            "IterableForEach",
            "IterableForEachRunner",
            GENERIC_RUNNER,
            4,
        ),
    ] {
        let root = scratch(class);
        let original = root.join("original");
        let recovered = root.join("recovered");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&recovered).unwrap();
        fs::write(original.join(format!("{class}.class")), bytes).unwrap();
        fs::write(original.join(format!("{runner}.java")), runner_source).unwrap();
        compile(&original, &[&format!("{runner}.java")]);
        let source = recover(bytes, class, RecoveryEvidenceRequest::essential());
        fs::write(recovered.join(format!("{class}.java")), source.text).unwrap();
        fs::write(recovered.join(format!("{runner}.java")), runner_source).unwrap();
        compile(
            &recovered,
            &[&format!("{class}.java"), &format!("{runner}.java")],
        );
        let expected = run(&original, runner);
        assert_eq!(expected.lines().count(), rows);
        assert_eq!(run(&recovered, runner), expected);
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn platform_collection_owners_project_only_with_matching_proof_and_preserve_execution() {
    let fixture = include_str!("fixtures/p3-iterable-foreach/PlatformIterableOwners.java");
    let runner = include_str!("fixtures/p3-iterable-foreach/PlatformIterableOwnersRunner.java");
    for debug in [false, true] {
        let root = scratch(if debug {
            "owners-debug"
        } else {
            "owners-nodebug"
        });
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("PlatformIterableOwners.java"), fixture).unwrap();
        fs::write(root.join("PlatformIterableOwnersRunner.java"), runner).unwrap();
        fs::write(
            root.join("TextIterable.java"),
            include_str!("fixtures/p3-iterable-foreach/TextIterable.java"),
        )
        .unwrap();
        fs::write(
            root.join("IteratorSurface.java"),
            include_str!("fixtures/p3-iterable-foreach/IteratorSurface.java"),
        )
        .unwrap();
        compile_with_debug(
            &root,
            &[
                "PlatformIterableOwners.java",
                "PlatformIterableOwnersRunner.java",
                "TextIterable.java",
                "IteratorSurface.java",
            ],
            debug,
        );
        let bytes = fs::read(root.join("PlatformIterableOwners.class")).unwrap();
        let report = recover(
            &bytes,
            "PlatformIterableOwners",
            RecoveryEvidenceRequest::essential(),
        );
        let all = recover(
            &bytes,
            "PlatformIterableOwners",
            RecoveryEvidenceRequest::all(),
        );
        assert_eq!(report.text, all.text);
        for name in ["listCast", "collectionCast", "listSkipEmpty"] {
            let text = &member(&report, name).text;
            assert!(
                text.contains("for (java.lang.Object iteratorElement"),
                "{name}:\n{text}"
            );
            assert!(text.contains("(java.lang.String)"), "{name}:\n{text}");
            assert!(!text.contains(".iterator()"), "{name}:\n{text}");
            assert!(!text.contains(".next()"), "{name}:\n{text}");
        }
        let list_method = member(&all, "listCast");
        let ClassSourceOutcome::Recovered {
            report: recovered, ..
        } = &list_method.outcome
        else {
            panic!("listCast did not recover")
        };
        for bci in [1, 10, 19] {
            assert!(
                !recovered
                    .source_map
                    .text_of_bci(&recovered.text, bci)
                    .is_empty(),
                "lost owner/hasNext/next BCI {bci}"
            );
        }
        for name in [
            "listConsumesTwice",
            "collectionEscapes",
            "userSubtype",
            "sameNameOnly",
        ] {
            let text = &member(&report, name).text;
            assert!(text.contains("while ("), "{name}:\n{text}");
            assert!(!text.contains("for (java.lang.Object "), "{name}:\n{text}");
        }
        let recovered = scratch(if debug {
            "owners-recovered-debug"
        } else {
            "owners-recovered-nodebug"
        });
        fs::create_dir_all(&recovered).unwrap();
        fs::write(recovered.join("PlatformIterableOwners.java"), &report.text).unwrap();
        fs::write(recovered.join("PlatformIterableOwnersRunner.java"), runner).unwrap();
        fs::write(
            recovered.join("TextIterable.java"),
            include_str!("fixtures/p3-iterable-foreach/TextIterable.java"),
        )
        .unwrap();
        fs::write(
            recovered.join("IteratorSurface.java"),
            include_str!("fixtures/p3-iterable-foreach/IteratorSurface.java"),
        )
        .unwrap();
        compile(
            &recovered,
            &[
                "PlatformIterableOwners.java",
                "PlatformIterableOwnersRunner.java",
                "TextIterable.java",
                "IteratorSurface.java",
            ],
        );
        assert_eq!(
            run(&root, "PlatformIterableOwnersRunner"),
            run(&recovered, "PlatformIterableOwnersRunner")
        );
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(recovered).unwrap();

        if !debug {
            let (snapshot, request) = opened(&bytes, "PlatformIterableOwners");
            let mut limits = report.limits.clone();
            limits.analysis_steps = report.usage.analysis_steps.saturating_sub(1);
            let mut low = Budget::new(limits);
            let bounded = Engine::new()
                .class_source_with_evidence(
                    slice::from_ref(&snapshot),
                    &request,
                    &RecoveryEvidenceRequest::essential(),
                    &mut low,
                )
                .unwrap();
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
                other => panic!("unexpected bounded outcome: {other:?}"),
            }
            let mut cancelled = task_budget(&[]).unwrap();
            cancelled.cancellation_token().cancel();
            let stopped = Engine::new()
                .class_source_with_evidence(
                    slice::from_ref(&snapshot),
                    &request,
                    &RecoveryEvidenceRequest::essential(),
                    &mut cancelled,
                )
                .unwrap();
            match stopped {
                OperationOutcome::Incomplete(candidates) => assert!(matches!(
                    candidates.execution,
                    ExecutionReport::Cancelled { .. }
                )),
                OperationOutcome::Performed(report) => assert!(report.text.is_empty()),
                other => panic!("unexpected cancelled outcome: {other:?}"),
            }
        }
    }

    let boundary = scratch("handler-boundary");
    fs::create_dir_all(&boundary).unwrap();
    fs::write(
        boundary.join("PlatformIterableHandlerBoundary.java"),
        include_str!("fixtures/p3-iterable-foreach/PlatformIterableHandlerBoundary.java"),
    )
    .unwrap();
    fs::write(
        boundary.join("PlatformIterableHandlerBoundaryRunner.java"),
        include_str!("fixtures/p3-iterable-foreach/PlatformIterableHandlerBoundaryRunner.java"),
    )
    .unwrap();
    compile(
        &boundary,
        &[
            "PlatformIterableHandlerBoundary.java",
            "PlatformIterableHandlerBoundaryRunner.java",
        ],
    );
    assert_eq!(
        run(&boundary, "PlatformIterableHandlerBoundaryRunner"),
        "1\n"
    );
    let bytes = fs::read(boundary.join("PlatformIterableHandlerBoundary.class")).unwrap();
    let report = recover(
        &bytes,
        "PlatformIterableHandlerBoundary",
        RecoveryEvidenceRequest::essential(),
    );
    let text = &member(&report, "caught").text;
    assert!(
        text.contains("@method caught(Ljava/util/Collection;)I"),
        "{text}"
    );
    assert!(text.contains("not recovered"), "{text}");
    assert!(text.contains("explanation only"), "{text}");
    assert!(!text.contains("for (java.lang.Object "), "{text}");
    fs::remove_dir_all(boundary).unwrap();
}

#[test]
fn source_selection_and_bounded_stops_keep_the_loop_atomic() {
    let essential = recover(
        RAW_NODEBUG,
        "RawIterableIterator",
        RecoveryEvidenceRequest::essential(),
    );
    let all = recover(
        RAW_NODEBUG,
        "RawIterableIterator",
        RecoveryEvidenceRequest::all(),
    );
    assert_eq!(essential.text, all.text);
    let method = member(&all, "rawCast");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("rawCast did not recover");
    };
    for bci in [1, 6, 9, 10, 15, 18, 19, 24, 25, 26, 29] {
        assert!(
            !report.source_map.text_of_bci(&report.text, bci).is_empty(),
            "lost BCI {bci}"
        );
    }
    let (snapshot, request) = opened(RAW_NODEBUG, "RawIterableIterator");
    let mut limits = essential.limits.clone();
    limits.analysis_steps = essential.usage.analysis_steps - 1;
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
            let text = &member(&report, "rawCast").text;
            if text.contains(" : ") {
                assert!(!text.contains(".iterator()"), "{text}");
                assert!(!text.contains(".next()"), "{text}");
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
