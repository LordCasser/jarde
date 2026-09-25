//! Complete-class Java 8 replay for a short-circuit transfer gateway.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/ChainExtraBoundary.class"
);
const SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/ChainExtraBoundary.java"
);
const RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/ChainExtraRunner.java"
);
const EXPECTED: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/runner-output.txt"
);
const MULTI_ENTRY: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/short-circuit-transfer-gateway-controls/ChainExtraBoundary.class"
);
const EXCEPTION_GATEWAY: &[u8] = include_bytes!(
    "fixtures/p3-conditional-values/short-circuit-transfer-gateway-controls/ChainExtraBoundaryException.class"
);

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-transfer-gateway-{}-{nonce}",
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

fn opened(class: &[u8]) -> (ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).expect("bounded default budget");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("frozen Java 8 class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ChainExtraBoundary"),
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

fn recovered_source_from(class: &[u8]) -> ClassSourceReport {
    let (snapshot, request) = opened(class);
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
    recovered_source_from(CLASS)
}

#[test]
fn gateway_has_one_structured_field_write_and_complete_sources() {
    let source = recovered_source();
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"assign")
        .expect("assign method exists");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("assign did not recover: {:?}", method.outcome);
    };
    assert_eq!(report.quality, jarde_jvm::ir::Quality::Structured);
    assert_eq!(report.representation, Representation::Java);
    assert!(report.fallbacks.is_empty(), "{}", report.text);
    assert_eq!(
        report.text.matches("ChainExtraBoundary.result =").count(),
        1,
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    for bci in [0, 1, 4, 5, 8, 11, 12, 15, 16, 19, 22, 25, 26, 29, 30, 33] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} is unmapped: {}",
            report.text
        );
    }
}

#[test]
fn independent_effect_in_transfer_slot_is_not_a_gateway() {
    // Replace the sole BCI 8 goto with the same-width iinc. The verifier accepts this
    // Java 8 class; the block now changes a local and falls through to the other arm.
    let mut changed = CLASS.to_vec();
    let prefix = [
        0x1a, 0x99, 0x00, 0x0a, 0x1b, 0x99, 0x00, 0x0a, 0xa7, 0x00, 0x11,
    ];
    let matches = changed
        .windows(prefix.len())
        .enumerate()
        .filter_map(|(index, window)| (window == prefix).then_some(index))
        .collect::<Vec<_>>();
    assert_eq!(matches.len(), 1, "the frozen assign bytecode changed");
    changed[matches[0] + 8..matches[0] + 11].copy_from_slice(&[0x84, 0x00, 0x01]);
    let scratch = Scratch::new();
    fs::write(scratch.0.join("ChainExtraBoundary.class"), &changed).expect("changed class");
    fs::write(
        scratch.0.join("ChainExtraRunner.class"),
        include_bytes!("../openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/ChainExtraRunner.class"),
    )
    .expect("runner class");
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp", ".", "ChainExtraRunner"])
        .current_dir(&scratch.0)
        .output()
        .expect("java is installed");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let source = recovered_source_from(&changed);
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"assign")
        .expect("assign method exists");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("assign did not recover: {:?}", method.outcome);
    };
    assert_ne!(
        report.quality,
        jarde_jvm::ir::Quality::Structured,
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("ChainExtraBoundary.result ="),
        "{}",
        report.text
    );
}

#[test]
fn backward_second_entry_to_gateway_is_refused() {
    // BCI 12 now targets BCI 8, so the otherwise pure goto has two physical
    // predecessors. The class has recomputed Java 8 stack frames and verifies.
    let scratch = Scratch::new();
    fs::write(scratch.0.join("ChainExtraBoundary.class"), MULTI_ENTRY).expect("changed class");
    fs::write(
        scratch.0.join("ChainExtraRunner.class"),
        include_bytes!("../openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/ChainExtraRunner.class"),
    )
    .expect("runner class");
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp", ".", "ChainExtraRunner"])
        .current_dir(&scratch.0)
        .output()
        .expect("java is installed");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let source = recovered_source_from(MULTI_ENTRY);
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"assign")
        .expect("assign method exists");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("assign did not recover: {:?}", method.outcome);
    };
    assert_ne!(
        report.quality,
        jarde_jvm::ir::Quality::Structured,
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("ChainExtraBoundary.result ="),
        "{}",
        report.text
    );
}

#[test]
fn protected_gateway_with_exception_edge_is_refused() {
    let scratch = Scratch::new();
    fs::write(
        scratch.0.join("ChainExtraBoundary.class"),
        EXCEPTION_GATEWAY,
    )
    .expect("changed class");
    fs::write(
        scratch.0.join("ChainExtraRunner.class"),
        include_bytes!("../openspec/evidence/java-syntax-2026-09-25/short-circuit-chain-extra-entry/ChainExtraRunner.class"),
    )
    .expect("runner class");
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp", ".", "ChainExtraRunner"])
        .current_dir(&scratch.0)
        .output()
        .expect("java is installed");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );

    let source = recovered_source_from(EXCEPTION_GATEWAY);
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"assign")
        .expect("assign method exists");
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("assign did not recover: {:?}", method.outcome);
    };
    assert_ne!(
        report.quality,
        jarde_jvm::ir::Quality::Structured,
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("ChainExtraBoundary.result ="),
        "{}",
        report.text
    );
}

#[test]
fn exhausted_budget_and_cancellation_do_not_publish_a_partial_gateway() {
    let complete = recovered_source();
    let (snapshot, request) = opened(CLASS);
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
fn gateway_complete_class_matches_all_thirty_two_jvm_paths() {
    let source = recovered_source();
    let scratch = Scratch::new();
    let original = scratch.0.join("original");
    let recovered = scratch.0.join("recovered");
    fs::create_dir(&original).expect("original directory");
    fs::create_dir(&recovered).expect("recovered directory");
    fs::write(original.join("ChainExtraBoundary.java"), SOURCE).expect("original source");
    fs::write(original.join("ChainExtraRunner.java"), RUNNER).expect("original runner");
    fs::write(recovered.join("ChainExtraBoundary.java"), source.text).expect("recovered source");
    fs::write(recovered.join("ChainExtraRunner.java"), RUNNER).expect("recovered runner");
    for directory in [&original, &recovered] {
        let compile = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "ChainExtraBoundary.java",
                "ChainExtraRunner.java",
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
        fs::read(original.join("ChainExtraBoundary.class")).expect("rebuilt original"),
        CLASS
    );
    for directory in [&original, &recovered] {
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp", ".", "ChainExtraRunner"])
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
    assert_eq!(EXPECTED.lines().count(), 32);
}
