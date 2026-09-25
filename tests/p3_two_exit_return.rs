//! A shared pair of Java 8 terminal boolean returns has one proved owner and one statement.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const POSITIVE: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/ternary-in-if/TernaryInIfProbe.class"
);
const RETURN2: &[u8] = include_bytes!("fixtures/p3-two-exit-return/TernaryInIfProbe-return2.class");
const INT_RETURN: &[u8] = include_bytes!("fixtures/p3-two-exit-return/TernaryInIfProbe-int.class");
const EFFECT: &[u8] = include_bytes!("fixtures/p3-two-exit-return/EffectProbe.class");
const BOUNDARY: &[u8] = include_bytes!("fixtures/p3-two-exit-return/BoundaryProbe.class");
const COUNTING: &[u8] = include_bytes!("fixtures/p3-two-exit-return/CountingProbe.class");
const EXTRA_ENTRY: &[u8] = include_bytes!("fixtures/p3-two-exit-return/ExtraEntryProbe.class");
const COUNTED_VALUE_SOURCE: &str = include_str!("fixtures/p3-two-exit-return/CountedValue.java");
const COUNTING_SOURCE: &str = include_str!("fixtures/p3-two-exit-return/CountingProbe.java");
const COUNTING_RUNNER: &str = include_str!("fixtures/p3-two-exit-return/CountingRunner.java");
const COUNTING_EXPECTED: &str = include_str!("fixtures/p3-two-exit-return/counting-run.txt");
const SOURCE: &str =
    include_str!("../openspec/evidence/java-syntax-2026-09-25/ternary-in-if/TernaryInIfProbe.java");
const JADX: &str =
    include_str!("../openspec/evidence/java-syntax-2026-09-25/ternary-in-if/jadx.java");
const RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/ternary-in-if/TernaryInIfRunner.java"
);
const EXPECTED: &str =
    include_str!("../openspec/evidence/java-syntax-2026-09-25/ternary-in-if/original-run.txt");

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("jarde-two-exit-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).expect("scratch directory");
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn opened(bytes: &[u8], name: &str) -> (ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).expect("default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("class opens");
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
fn recovered(bytes: &[u8], name: &str) -> ClassSourceReport {
    let (snapshot, request) = opened(bytes, name);
    let mut budget = task_budget(&[]).expect("default budget");
    let OperationOutcome::Performed(source) = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .expect("class source request")
    else {
        panic!("class must bind uniquely")
    };
    source
}

#[test]
fn shared_return_has_one_statement_and_every_physical_source() {
    let source = recovered(POSITIVE, "TernaryInIfProbe");
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"bothMatch")
        .expect("method");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("method recovery")
    };
    assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured);
    assert_eq!(report.representation, Representation::Java);
    assert!(report.fallbacks.is_empty(), "{}", report.text);
    assert_eq!(report.text.matches("return ").count(), 1, "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    let mut owned = std::collections::BTreeSet::new();
    for block in report.regions.iter().flat_map(|region| &region.blocks) {
        assert!(
            owned.insert(*block),
            "physical block {block} has two owners"
        );
    }
    assert_eq!(owned.len(), 10, "positive closure must own all ten blocks");
    for bci in [
        0, 1, 4, 7, 8, 11, 14, 17, 18, 21, 22, 25, 28, 31, 32, 35, 38, 39, 42, 45, 48, 49, 52, 53,
        56, 59, 62, 63, 64, 65,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci}: {}",
            report.text
        );
    }
}

#[test]
fn third_exit_backedge_and_exception_boundary_keep_their_own_structure() {
    let source = recovered(BOUNDARY, "BoundaryProbe");
    for name in [b"third".as_slice(), b"backedge", b"exception"] {
        let method = source
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name)
            .expect("boundary method");
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("method recovery")
        };
        assert!(
            report.text.matches("return ").count() != 1 || !report.fallbacks.is_empty(),
            "boundary unexpectedly collapsed: {}",
            report.text
        );
    }
}

#[test]
fn an_inner_two_return_graph_with_two_outside_predecessors_is_not_claimed() {
    let source = recovered(EXTRA_ENTRY, "ExtraEntryProbe");
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"check")
        .expect("method");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("method recovery")
    };
    // BCI 8 has normal predecessors from the BCI 1 branch and the effect at BCI 4–7.
    // The candidate rooted at BCI 8 has no unique external entrance to claim.
    assert!(
        report.text.matches("return ").count() != 1 || !report.fallbacks.is_empty(),
        "extra-entry graph was folded: {}",
        report.text
    );
    assert!(
        !report.text.contains("jre_region_ownership_overlap"),
        "ordinary ownership was damaged: {}",
        report.text
    );
}

#[test]
fn unproved_leaves_descriptor_and_independent_effect_do_not_fold() {
    for (bytes, name) in [
        (RETURN2, "TernaryInIfProbe"),
        (INT_RETURN, "TernaryInIfProbe"),
        (EFFECT, "EffectProbe"),
    ] {
        let source = recovered(bytes, name);
        let method = source
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"bothMatch")
            .expect("method");
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("method recovery")
        };
        assert!(
            !(report.quality == jarde_jvm::ir::Quality::Structured
                && report.text.matches("return ").count() == 1),
            "control unexpectedly folded: {}",
            report.text
        );
        if bytes != EFFECT {
            let quoted = report
                .text
                .lines()
                .filter(|line| line.contains("@bytecode"))
                .flat_map(|line| {
                    line.split_whitespace()
                        .filter_map(|part| part.parse::<u32>().ok())
                })
                .collect::<std::collections::BTreeSet<_>>();
            for bci in [
                0, 1, 4, 7, 8, 11, 14, 17, 18, 21, 22, 25, 28, 31, 32, 35, 38, 39, 42, 45, 48, 49,
                52, 53, 56, 59, 62, 63, 64, 65,
            ] {
                assert!(
                    quoted.contains(&bci),
                    "refused graph lost BCI {bci}: {}",
                    report.text
                );
            }
        }
    }
}

#[test]
fn low_budget_and_pre_cancel_publish_no_partial_return() {
    let complete = recovered(POSITIVE, "TernaryInIfProbe");
    let (snapshot, request) = opened(POSITIVE, "TernaryInIfProbe");
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
        .expect("bounded request");
    match bounded {
        OperationOutcome::Incomplete(candidates) => assert!(matches!(
            candidates.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded { .. },
                ..
            }
        )),
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded { .. },
                    ..
                }
            ));
            if let Some(method) = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"bothMatch")
            {
                let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
                    panic!("partial method has no recovery report")
                };
                if body.text.contains("return ") {
                    assert_eq!(
                        body.quality,
                        jarde_jvm::ir::Quality::Structured,
                        "{}",
                        body.text
                    );
                    for bci in [0, 4, 11, 14, 17, 28, 31, 38, 45, 48, 59, 62, 63, 64, 65] {
                        assert!(
                            !body.source_map.of_bci(bci).is_empty(),
                            "partial return lost BCI {bci}: {}",
                            body.text
                        );
                    }
                }
            }
        }
        _ => unreachable!(),
    }
    let mut cancelled = task_budget(&[]).expect("default budget");
    cancelled.cancellation_token().cancel();
    let result = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancelled request");
    assert!(
        matches!(result, OperationOutcome::Incomplete(candidates) if matches!(candidates.execution, ExecutionReport::Cancelled { .. }))
    );
}

#[test]
fn complete_class_compiles_and_matches_eight_verified_paths() {
    let recovered = recovered(POSITIVE, "TernaryInIfProbe");
    let scratch = Scratch::new();
    for (label, text) in [
        ("original", SOURCE),
        ("jadx", JADX),
        ("jarde", recovered.text.as_str()),
    ] {
        let directory = scratch.0.join(label);
        fs::create_dir(&directory).expect("case directory");
        fs::write(directory.join("TernaryInIfProbe.java"), text).expect("source");
        fs::write(directory.join("TernaryInIfRunner.java"), RUNNER).expect("runner");
        let compile = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "-Xlint:-options",
                "TernaryInIfProbe.java",
                "TernaryInIfRunner.java",
            ])
            .current_dir(&directory)
            .output()
            .expect("javac");
        assert!(
            compile.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        if label == "original" {
            assert_eq!(
                fs::read(directory.join("TernaryInIfProbe.class")).expect("class"),
                POSITIVE
            );
        }
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp", ".", "TernaryInIfRunner"])
            .current_dir(&directory)
            .output()
            .expect("java");
        assert!(
            run.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8(run.stdout).expect("UTF-8 trace"),
            EXPECTED,
            "{label}"
        );
    }
    assert_eq!(EXPECTED.lines().count(), 8);
}

#[test]
fn observable_equals_calls_keep_their_lazy_order_and_count() {
    let recovered = recovered(COUNTING, "CountingProbe");
    let method = recovered
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"bothMatch")
        .expect("method");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("method recovery")
    };
    assert_eq!(
        report.quality,
        jarde_jvm::ir::Quality::Structured,
        "{}",
        report.text
    );
    assert_eq!(report.text.matches("return ").count(), 1, "{}", report.text);
    let scratch = Scratch::new();
    for (label, text) in [
        ("original", COUNTING_SOURCE),
        ("jarde", recovered.text.as_str()),
    ] {
        let directory = scratch.0.join(label);
        fs::create_dir(&directory).expect("case directory");
        fs::write(directory.join("CountedValue.java"), COUNTED_VALUE_SOURCE).expect("value source");
        fs::write(directory.join("CountingProbe.java"), text).expect("probe source");
        fs::write(directory.join("CountingRunner.java"), COUNTING_RUNNER).expect("runner source");
        let compile = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "-Xlint:-options",
                "CountedValue.java",
                "CountingProbe.java",
                "CountingRunner.java",
            ])
            .current_dir(&directory)
            .output()
            .expect("javac");
        assert!(
            compile.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        if label == "original" {
            assert_eq!(
                fs::read(directory.join("CountingProbe.class")).expect("class"),
                COUNTING
            );
        }
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp", ".", "CountingRunner"])
            .current_dir(&directory)
            .output()
            .expect("java");
        assert!(
            run.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8(run.stdout).expect("UTF-8 trace"),
            COUNTING_EXPECTED,
            "{label}"
        );
    }
    assert_eq!(COUNTING_EXPECTED.lines().count(), 8);
}
