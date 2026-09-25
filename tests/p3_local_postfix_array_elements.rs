//! A local `iinc` becomes an initializer postfix value only for the proved old-value window.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const POSITIVE: &[u8] =
    include_bytes!("fixtures/p3-local-postfix-array-elements/v8/ArrayPostfixElement.class");
const PLUS_TWO: &[u8] =
    include_bytes!("fixtures/p3-local-postfix-array-elements/v8/ArrayPostfixElement-plus2.class");
const OTHER_SLOT: &[u8] =
    include_bytes!("fixtures/p3-local-postfix-array-elements/v8/DifferentSlotArrayElement.class");
static NEXT: AtomicU64 = AtomicU64::new(0);

fn budget() -> Budget {
    task_budget(&[]).expect("bounded task defaults")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("Java 8 fixture opens")
}

fn request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
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
    }
}

fn source(bytes: &[u8], class: &str) -> ClassSourceReport {
    let snapshot = open(bytes);
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot, class),
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("valid class source request")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("class source incomplete: {other:?}"),
    }
}

fn recovery<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .expect("method exists");
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("no recovery body for {name}: {other:?}"),
    }
}

#[test]
fn local_postfix_is_one_ordered_element_with_complete_sources() {
    let report = source(POSITIVE, "ArrayPostfixElement");
    let body = recovery(&report, "make");
    assert!(body.produced(), "{body:?}");
    assert!(
        body.text.contains("return new int[]{1, arg0++, arg0 * 2};"),
        "{}",
        body.text
    );
    assert_eq!(body.text.matches("++").count(), 1, "{}", body.text);
    assert!(!body.text.contains("arg0 = arg0 + 1"), "{}", body.text);
    for bci in [
        0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 13, 14, 15, 16, 17, 18, 19, 20,
    ] {
        assert!(!body.source_map.of_bci(bci).is_empty(), "missing BCI {bci}");
    }
}

#[test]
fn plus_two_and_other_slot_are_not_local_postfix_values() {
    for (bytes, class) in [
        (PLUS_TWO, "ArrayPostfixElement"),
        (OTHER_SLOT, "DifferentSlotArrayElement"),
    ] {
        let report = source(bytes, class);
        let body = recovery(&report, "make");
        assert!(!body.text.contains("++"), "{class}: {}", body.text);
        assert!(
            !body.text.contains("return new int[]{"),
            "{class}: {}",
            body.text
        );
    }
}

#[test]
fn low_budget_and_pre_cancellation_do_not_commit_a_partial_postfix() {
    let snapshot = open(POSITIVE);
    let request = request(&snapshot, "ArrayPostfixElement");
    let mut low = Budget::new(Limits {
        ir_items: 300,
        ..task_limits(&[]).expect("bounded defaults")
    });
    let stopped = Engine::new().class_source_with_evidence(
        slice::from_ref(&snapshot),
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut low,
    );
    match stopped {
        Ok(OperationOutcome::Performed(report)) => {
            assert!(!report.text.contains("arg0++"), "{}", report.text);
            assert!(
                report.text.contains("budget_exceeded_ir_items"),
                "{}",
                report.text
            );
        }
        Ok(OperationOutcome::Incomplete(_)) | Err(_) => {}
        other => panic!("low budget returned an unexpected complete selection: {other:?}"),
    }
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled =
        Budget::with_cancellation_token(task_limits(&[]).expect("bounded defaults"), token);
    let stopped = Engine::new().class_source_with_evidence(
        slice::from_ref(&snapshot),
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut cancelled,
    );
    assert!(!matches!(stopped, Ok(OperationOutcome::Performed(_))));
}

struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jarde-local-postfix-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&path).expect("isolated Java directory");
        Self(path)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "requires a JDK for complete Java 8 class comparison"]
fn complete_class_matches_five_verified_jvm_rows() {
    let report = source(POSITIVE, "ArrayPostfixElement");
    let dir = TempDir::new();
    fs::write(dir.0.join("ArrayPostfixElement.java"), &report.text).expect("write recovered class");
    fs::write(
        dir.0.join("Runner.java"),
        include_str!("fixtures/p3-local-postfix-array-elements/v8/Runner.java"),
    )
    .expect("write runner");
    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options", "-d"])
        .arg(&dir.0)
        .arg(dir.0.join("ArrayPostfixElement.java"))
        .arg(dir.0.join("Runner.java"))
        .output()
        .expect("javac is available");
    assert!(
        compile.status.success(),
        "{}\n{}",
        report.text,
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&dir.0)
        .arg("Runner")
        .output()
        .expect("java is available");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8(run.stdout).expect("UTF-8"),
        include_str!(
            "../openspec/evidence/java-syntax-2026-09-25/array-postfix-element/original-run.txt"
        )
    );
}

#[test]
#[ignore = "requires a JDK to verify the patched control classes"]
fn patched_controls_remain_verifier_valid_and_distinguishable() {
    let dir = TempDir::new();
    fs::write(dir.0.join("ArrayPostfixElement.class"), PLUS_TWO).expect("write +2 class");
    fs::write(
        dir.0.join("Runner.java"),
        include_str!("fixtures/p3-local-postfix-array-elements/v8/Runner.java"),
    )
    .expect("write runner");
    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options", "-cp"])
        .arg(&dir.0)
        .arg("-d")
        .arg(&dir.0)
        .arg(dir.0.join("Runner.java"))
        .output()
        .expect("javac is available");
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&dir.0)
        .arg("Runner")
        .output()
        .expect("java is available");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8(run.stdout).expect("UTF-8"),
        include_str!(
            "../openspec/evidence/java-syntax-2026-09-25/array-postfix-element/plus2-original-run.txt"
        )
        .strip_suffix("exit=0\n")
        .expect("frozen run records its successful exit")
    );

    let run = Command::new("java")
        .args([
            "-Xverify:all",
            "-cp",
            "tests/fixtures/p3-local-postfix-array-elements/v8",
            "DifferentSlotRunner",
        ])
        .output()
        .expect("java is available");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8(run.stdout).expect("UTF-8"), "[1, 0, 0]\n");
}
