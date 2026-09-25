//! Java 8 acceptance for ordered nested initializer chains and their refusal boundaries.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "jarde-nested-arrays-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&path).expect("isolated Java directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const NESTED: &[u8] =
    include_bytes!("fixtures/p3-nested-array-initializers/v8/NestedArrayInitializer.class");
const ORDERING: &[u8] =
    include_bytes!("fixtures/p3-nested-array-initializers/v8/JadxOrderingControl.class");
const EXTRA_USE: &[u8] =
    include_bytes!("fixtures/p3-nested-array-initializers/v8/NestedArrayExtraUse.class");

fn budget() -> Budget {
    task_budget(&[]).expect("bounded task defaults")
}

fn source(bytes: &[u8], class: &str) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("standalone Java 8 class opens");
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
    let mut source_budget = budget();
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut source_budget,
        )
        .expect("class source request is valid");
    match outcome {
        OperationOutcome::Performed(report) => report,
        other => panic!("class source is incomplete: {other:?}"),
    }
}

fn recovery<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let member = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .expect("method exists");
    match &member.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("method {name} has no recovery report: {other:?}"),
    }
}

#[test]
fn nested_arrays_are_one_ordered_expression_with_complete_sources() {
    let report = source(NESTED, "NestedArrayInitializer");
    for (name, expected, bcis) in [
        (
            "dynamic",
            "return new int[][]{new int[]{element(1), element(2)}, new int[]{element(3)}};",
            &[
                0, 1, 4, 5, 6, 7, 9, 10, 11, 12, 15, 16, 17, 18, 19, 22, 23, 24, 25, 26, 27, 29,
                30, 31, 32, 35, 36, 37,
            ][..],
        ),
        (
            "literal",
            "return new java.lang.String[][]{new java.lang.String[]{\"a\", \"b\"}, new java.lang.String[]{\"c\"}};",
            &[
                0, 1, 4, 5, 6, 7, 10, 11, 12, 14, 15, 16, 17, 19, 20, 21, 22, 23, 24, 27, 28, 29,
                31, 32, 33,
            ][..],
        ),
    ] {
        let recovered = recovery(&report, name);
        assert!(recovered.produced(), "{name}: {recovered:?}");
        assert!(
            recovered.text.contains(expected),
            "{name}: {}",
            recovered.text
        );
        for bci in bcis {
            assert!(
                !recovered.source_map.of_bci(*bci).is_empty(),
                "{name} missed BCI {bci}"
            );
        }
        assert!(
            !recovered.text.contains("@bytecode"),
            "{name}: {}",
            recovered.text
        );
    }
}

#[test]
fn ordering_and_child_extra_use_do_not_become_nested_literals() {
    let ordering = source(ORDERING, "JadxOrderingControl");
    let body = recovery(&ordering, "build");
    assert!(
        !body.text.contains("new int[]{mark(2), mark(1)}"),
        "{}",
        body.text
    );
    assert!(
        body.text.find("mark(1)") < body.text.find("mark(2)"),
        "{}",
        body.text
    );

    let extra = source(EXTRA_USE, "NestedArrayExtraUse");
    let body = recovery(&extra, "build");
    assert!(!body.text.contains("new int[][]{"), "{}", body.text);
}

#[test]
fn budget_and_pre_cancellation_publish_no_partial_nested_expression() {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(NESTED.to_vec()), &mut budget())
        .expect("fixture opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("NestedArrayInitializer"),
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
    let mut tight = Budget::new(Limits {
        output_bytes: 0,
        ..task_limits(&[]).expect("bounded defaults")
    });
    let stopped = Engine::new().class_source_with_evidence(
        slice::from_ref(&snapshot),
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut tight,
    );
    assert!(
        !matches!(stopped, Ok(OperationOutcome::Performed(_))),
        "output budget committed a class"
    );

    let mut low_ir = Budget::new(Limits {
        ir_items: 1_000,
        ..task_limits(&[]).expect("bounded defaults")
    });
    let stopped = Engine::new().class_source_with_evidence(
        slice::from_ref(&snapshot),
        &request,
        &RecoveryEvidenceRequest::all(),
        &mut low_ir,
    );
    match stopped {
        Ok(OperationOutcome::Performed(report)) => {
            assert!(!report.text.contains("new int[][]{"), "{}", report.text);
            assert!(
                report.text.contains("budget_exceeded_ir_items"),
                "{}",
                report.text
            );
        }
        other => panic!("the low IR budget should yield an explicit partial class: {other:?}"),
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
    assert!(
        !matches!(stopped, Ok(OperationOutcome::Performed(_))),
        "pre-cancelled request committed a class"
    );
}

#[test]
#[ignore = "requires a JDK to compile and run complete Java 8 classes"]
fn complete_java8_classes_match_verified_jvm_execution() {
    for (bytes, class, runner, expected) in [
        (
            NESTED,
            "NestedArrayInitializer",
            "Runner",
            "[[1, 2], [3]]:3:123\n[[a, b], [c]]\n",
        ),
        (
            ORDERING,
            "JadxOrderingControl",
            "OrderingRunner",
            "[2, 1]:12\n",
        ),
    ] {
        let report = source(bytes, class);
        let dir = TempDir::new();
        fs::write(dir.path().join(format!("{class}.java")), &report.text)
            .expect("write recovered full class");
        fs::write(
            dir.path().join(format!("{runner}.java")),
            if runner == "Runner" {
                include_str!("fixtures/p3-nested-array-initializers/v8/Runner.java")
            } else {
                include_str!("fixtures/p3-nested-array-initializers/v8/OrderingRunner.java")
            },
        )
        .expect("write runner");
        let compile = Command::new("javac")
            .args(["--release", "8", "-Xlint:-options", "-d"])
            .arg(dir.path())
            .arg(dir.path().join(format!("{class}.java")))
            .arg(dir.path().join(format!("{runner}.java")))
            .output()
            .expect("javac is available");
        assert!(
            compile.status.success(),
            "{class}: {}\n{}",
            report.text,
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(dir.path())
            .arg(runner)
            .output()
            .expect("java is available");
        assert!(
            run.status.success(),
            "{class}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8(run.stdout).expect("UTF-8 output"),
            expected,
            "{class}"
        );
    }
}
