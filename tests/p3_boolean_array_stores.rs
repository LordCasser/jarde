//! Complete-class execution oracle for verifier-valid boolean-array stores.
//! The patched Java 8 classes are frozen from the byte-for-byte audit fixture; each recovered
//! class is compiled with its original support runner and compared with that JVM oracle.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const RAW_BOOL: &[u8] = include_bytes!("fixtures/p3-boolean-array-stores/v8/RawBool.class");
const ORDER: &[u8] = include_bytes!("fixtures/p3-boolean-array-stores/v8/Order.class");
const RAW_BOOL_RUNNER: &str = include_str!("fixtures/p3-boolean-array-stores/RawBoolRunner.java");
const ORDER_RUNNER: &str = include_str!("fixtures/p3-boolean-array-stores/OrderRunner.java");
const RAW_BOOL_EXPECTED: &str =
    include_str!("fixtures/p3-boolean-array-stores/expected/RawBool.stdout");
const ORDER_EXPECTED: &str = include_str!("fixtures/p3-boolean-array-stores/expected/Order.stdout");

fn budget() -> Budget {
    task_budget(&[]).expect("task defaults provide bounded recovery")
}

fn recover(bytes: &[u8], class_name: &str) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("patched Java 8 class opens");
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
    match Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("single-class source request completes")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one standalone class resolves exactly: {other:?}"),
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new(class_name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-boolean-array-stores-{}-{class_name}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create private compilation directory");
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

fn javac(dir: &Path, files: &[&str]) {
    let output = std::process::Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir)
        .args(files)
        .current_dir(dir)
        .output()
        .expect("javac is available for the complete-class check");
    assert!(
        output.status.success(),
        "Java 8 complete-class compilation failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_verified(dir: &Path, runner: &str) -> String {
    let output = std::process::Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg(runner)
        .current_dir(dir)
        .output()
        .expect("java is available for strict verification");
    assert!(
        output.status.success(),
        "{runner} failed strict JVM verification:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("runner output is UTF-8")
}

fn assert_complete_recovery(
    class_name: &str,
    bytes: &[u8],
    runner_source: &str,
    runner_name: &str,
    expected: &str,
) {
    let report = recover(bytes, class_name);
    assert!(
        report
            .methods
            .iter()
            .all(|method| method.markers.is_empty()),
        "{class_name} class has no member fallback references: {:?}",
        report
            .methods
            .iter()
            .map(|method| (&method.item.name, &method.markers))
            .collect::<Vec<_>>()
    );
    assert!(
        !report.text.contains("@bytecode"),
        "{class_name} class text contains a fallback reference:\n{}",
        report.text
    );

    let scratch = Scratch::new(class_name);
    let oracle = scratch.path().join("oracle");
    let recovered = scratch.path().join("recovered");
    fs::create_dir_all(&oracle).expect("create patched-class oracle directory");
    fs::create_dir_all(&recovered).expect("create recovered-class directory");
    let fixture = if class_name == "RawBool" {
        RAW_BOOL
    } else {
        ORDER
    };
    fs::write(oracle.join(format!("{class_name}.class")), fixture)
        .expect("write the exact patched class oracle");
    fs::write(oracle.join(format!("{runner_name}.java")), runner_source)
        .expect("write source-only oracle runner");
    javac(&oracle, &[&format!("{runner_name}.java")]);
    let source_output = run_verified(&oracle, runner_name);
    assert_eq!(source_output, expected, "original patched class JVM output");

    fs::write(recovered.join(format!("{class_name}.java")), &report.text)
        .expect("write the complete Jarde class");
    fs::write(recovered.join(format!("{runner_name}.java")), runner_source)
        .expect("write source-only recovered runner");
    javac(
        &recovered,
        &[
            &format!("{class_name}.java"),
            &format!("{runner_name}.java"),
        ],
    );
    let recovered_output = run_verified(&recovered, runner_name);
    assert_eq!(
        recovered_output, source_output,
        "complete Jarde class semantics"
    );
}

#[test]
fn complete_boolean_array_classes_match_the_verified_jvm() {
    assert_complete_recovery(
        "RawBool",
        RAW_BOOL,
        RAW_BOOL_RUNNER,
        "RawBoolRunner",
        RAW_BOOL_EXPECTED,
    );
    assert_complete_recovery("Order", ORDER, ORDER_RUNNER, "OrderRunner", ORDER_EXPECTED);
}
