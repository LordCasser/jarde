//! A proved boolean array target retains the array/index/RHS evaluation order.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-array/MixedArrayValue.class"
);
const SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-array/MixedArrayValue.java"
);
const RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-array/Runner.java"
);
const EXPECTED: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-array/original-run.txt"
);
const CONTROLS: &str = include_str!(
    "fixtures/p3-conditional-values/mixed-short-circuit-array/MixedArrayControls.java"
);
const CONTROL_RUNNER: &str =
    include_str!("fixtures/p3-conditional-values/mixed-short-circuit-array/ControlRunner.java");

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-array-short-circuit-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn recovered_with(
    class: &[u8],
    name: &str,
) -> (ArtifactSnapshot, ClassSourceRequest, ClassSourceReport) {
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .unwrap();
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
    let OperationOutcome::Performed(report) = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .unwrap()
    else {
        panic!("single class binds uniquely");
    };
    (snapshot, request, report)
}

fn recovered() -> (ArtifactSnapshot, ClassSourceRequest, ClassSourceReport) {
    recovered_with(CLASS, "MixedArrayValue")
}

#[test]
fn boolean_array_store_owns_all_three_operands_and_sources() {
    let (_, _, source) = recovered();
    let method = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"one")
        .unwrap();
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("{:?}", method.outcome)
    };
    assert_eq!(
        report.quality,
        jarde_jvm::ir::Quality::Structured,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("array(arg1)[index(arg2)] =").count(),
        1,
        "{}",
        report.text
    );
    assert!(report.text.contains("&&"), "{}", report.text);
    assert!(report.text.contains("||"), "{}", report.text);
    assert!(!report.text.contains("% 2 != 0"), "{}", report.text);
    assert!(report.fallbacks.is_empty(), "{}", report.text);
    for bci in [0, 1, 4, 5, 8, 9, 12, 15, 18, 21, 24, 25, 28, 29, 30] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci}: {}",
            report.text
        );
    }
}

#[test]
fn java8_rebuild_preserves_all_twenty_four_paths() {
    let (_, _, source) = recovered();
    let scratch = Scratch::new();
    let original = scratch.0.join("original");
    let recovered = scratch.0.join("recovered");
    fs::create_dir(&original).unwrap();
    fs::create_dir(&recovered).unwrap();
    fs::write(original.join("MixedArrayValue.java"), SOURCE).unwrap();
    fs::write(original.join("Runner.java"), RUNNER).unwrap();
    fs::write(recovered.join("MixedArrayValue.java"), source.text).unwrap();
    fs::write(recovered.join("Runner.java"), RUNNER).unwrap();
    for directory in [&original, &recovered] {
        let compile = Command::new("javac")
            .args([
                "--release",
                "8",
                "-g:none",
                "-Xlint:-options",
                "MixedArrayValue.java",
                "Runner.java",
            ])
            .current_dir(directory)
            .output()
            .unwrap();
        assert!(
            compile.status.success(),
            "{}",
            String::from_utf8_lossy(&compile.stderr)
        );
    }
    assert_eq!(
        fs::read(original.join("MixedArrayValue.class")).unwrap(),
        CLASS
    );
    for directory in [&original, &recovered] {
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp", ".", "Runner"])
            .current_dir(directory)
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8(run.stdout).unwrap(), EXPECTED);
    }
    assert_eq!(EXPECTED.lines().count(), 24);
}

#[test]
fn verifier_valid_near_misses_keep_the_array_candidate_quoted() {
    let scratch = Scratch::new();
    fs::write(scratch.0.join("MixedArrayControls.java"), CONTROLS).unwrap();
    fs::write(scratch.0.join("ControlRunner.java"), CONTROL_RUNNER).unwrap();
    let compile = Command::new("javac")
        .args([
            "--release",
            "8",
            "-g:none",
            "-Xlint:-options",
            "MixedArrayControls.java",
            "ControlRunner.java",
        ])
        .current_dir(&scratch.0)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let verified = Command::new("java")
        .args(["-Xverify:all", "-cp", ".", "ControlRunner"])
        .current_dir(&scratch.0)
        .output()
        .unwrap();
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    let class = fs::read(scratch.0.join("MixedArrayControls.class")).unwrap();
    let (_, _, source) = recovered_with(&class, "MixedArrayControls");
    let plain = source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"plainWrite")
        .unwrap();
    let ClassSourceOutcome::Recovered { report: plain, .. } = &plain.outcome else {
        panic!("{:?}", plain.outcome)
    };
    assert_eq!(
        plain.quality,
        jarde_jvm::ir::Quality::Structured,
        "{}",
        plain.text
    );
    for name in [
        "byteTarget",
        "sharedArray",
        "sharedIndex",
        "duplicatedValue",
        "protectedWrite",
        "guardedWrite",
    ] {
        let method = source
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name.as_bytes())
            .unwrap();
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("{name}: {:?}", method.outcome)
        };
        assert_eq!(
            report.quality,
            jarde_jvm::ir::Quality::Fallback,
            "{name}: {}",
            report.text
        );
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
        assert!(!report.text.contains("[index("), "{name}: {}", report.text);
        for bci in report.regions.iter().flat_map(|region| &region.blocks) {
            assert!(
                !report.source_map.of_bci(*bci).is_empty(),
                "{name}: BCI {bci}"
            );
        }
    }

    // `aconst_null` is a verifier-valid array reference for bastore. Removing only the
    // redundant checkcast leaves the exact same control flow and store opcode, but no `[Z`
    // source fact. The StackMapTable still verifies because null is assignable to `[Z`.
    let mut unknown = class.clone();
    let sites = unknown
        .windows(6)
        .enumerate()
        .filter_map(|(at, bytes)| {
            (bytes[0] == 0x01
                && bytes[1] == 0xc0
                && bytes[2] == 0x00
                && bytes[4] == 0x1b
                && bytes[5] == 0xb8)
                .then_some(at)
        })
        .collect::<Vec<_>>();
    let [site] = sites.as_slice() else {
        panic!("expected one nullTarget checkcast: {sites:?}")
    };
    unknown[site + 1..site + 4].fill(0x00);
    fs::write(scratch.0.join("MixedArrayControls.class"), &unknown).unwrap();
    let verified = Command::new("java")
        .args(["-Xverify:all", "-cp", ".", "ControlRunner"])
        .current_dir(&scratch.0)
        .output()
        .unwrap();
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    let (_, _, unknown_source) = recovered_with(&unknown, "MixedArrayControls");
    let method = unknown_source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"nullTarget")
        .unwrap();
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("{:?}", method.outcome)
    };
    assert_eq!(
        report.quality,
        jarde_jvm::ir::Quality::Fallback,
        "{}",
        report.text
    );
    assert!(!report.text.contains("[index("), "{}", report.text);

    // JVM names may be Java keywords. Rename the method and its call site together in the
    // constant pool; the class still verifies and executes, but the index call cannot be Java.
    let mut unspellable = class.clone();
    let sites = unspellable
        .windows(7)
        .enumerate()
        .filter_map(|(at, bytes)| (bytes == b"\x00\x05index").then_some(at))
        .collect::<Vec<_>>();
    let [site] = sites.as_slice() else {
        panic!("expected one index name: {sites:?}")
    };
    unspellable[site + 2..site + 7].copy_from_slice(b"class");
    fs::write(scratch.0.join("MixedArrayControls.class"), &unspellable).unwrap();
    let verified = Command::new("java")
        .args(["-Xverify:all", "-cp", ".", "ControlRunner"])
        .current_dir(&scratch.0)
        .output()
        .unwrap();
    assert!(
        verified.status.success(),
        "{}",
        String::from_utf8_lossy(&verified.stderr)
    );
    let (_, _, unspellable_source) = recovered_with(&unspellable, "MixedArrayControls");
    let method = unspellable_source
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"plainWrite")
        .unwrap();
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("{:?}", method.outcome)
    };
    assert_eq!(
        report.quality,
        jarde_jvm::ir::Quality::Fallback,
        "{}",
        report.text
    );
    assert!(!report.text.contains("[class("), "{}", report.text);
}

#[test]
fn budget_and_cancellation_do_not_publish_a_partial_array_assignment() {
    let (snapshot, request, complete) = recovered();
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
        other => panic!("unexpected bounded result: {other:?}"),
    }
    let mut cancelled = task_budget(&[]).unwrap();
    cancelled.cancellation_token().cancel();
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .unwrap();
    assert!(matches!(outcome, OperationOutcome::Incomplete(candidates)
        if matches!(candidates.execution, ExecutionReport::Cancelled { .. })));
}
