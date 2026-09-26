//! P3 3.3a: an exit edge from a branch inside a header-tested loop must stay executable.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::slice;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const LOOP_BOOL: &[u8] = include_bytes!("fixtures/p3-loop-boolean-exit/v8/LoopBool.class");
const LOOP_BOOL_SOURCE: &str = include_str!("fixtures/p3-loop-boolean-exit/LoopBool.java");
const INPUTS: &[(i32, i32, i32)] = &[(1, 0, 0), (1, 1, 1), (-1, 1, 0)];

fn report() -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(LOOP_BOOL.to_vec()), &mut budget())
        .expect("the frozen LoopBool class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("LoopBool"),
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
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            &mut budget(),
        )
        .expect("the class-source request succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one class has one source result, got {other:?}"),
    }
}

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}`"))
}

fn recovery(method: &ClassSourceMethod) -> &RecoveryReport {
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("the member has a recovery report, got {other:?}"),
    }
}

#[test]
fn an_inner_branch_to_the_loop_exit_is_a_break_and_refused_compound_loops_stay_refused() {
    let report = report();

    let and_while = method(&report, "andWhile");
    let and_text = &and_while.text;
    assert!(
        and_text.contains("if (arg1 > 0) {")
            && and_text.contains("} else {\n                break;"),
        "the inner false edge reaches the loop's exact exit and must be an explicit break:\n{and_text}"
    );
    assert_eq!(
        recovery(and_while).quality,
        jarde_jvm::ir::Quality::Structured
    );
    assert!(
        recovery(and_while)
            .source_map
            .text_of_bci(&recovery(and_while).text, 7)
            .iter()
            .any(|segment| segment.contains("break;")),
        "the source-map entry for the inner exit branch must name its emitted transfer"
    );

    for name in ["orWhile", "mixedWhile"] {
        let refused = method(&report, name);
        let text = &refused.text;
        assert!(text.contains("@bytecode"), "`{name}` stays quoted:\n{text}");
        assert_eq!(
            recovery(refused).quality,
            jarde_jvm::ir::Quality::Fallback,
            "{name}"
        );
    }
}

#[test]
fn recovered_loop_exit_matches_the_java8_class_for_terminating_and_continuing_inputs() {
    let report = report();
    let and_while = method(&report, "andWhile");
    let recovered_source = format!("public final class LoopBool {{\n{}\n}}\n", and_while.text);
    // The public class-source text contains the recovered declaration and methods. The test runner
    // is compiled beside it so both executions call the same three input pairs.
    assert!(
        recovered_source.contains("break;"),
        "the recovered source keeps the loop exit explicit:\n{recovered_source}"
    );

    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered = scratch.path().join("recovered");
    fs::create_dir_all(&original).expect("create original comparison directory");
    fs::create_dir_all(&recovered).expect("create recovered comparison directory");
    fs::write(original.join("LoopBool.java"), LOOP_BOOL_SOURCE)
        .expect("write the fixture's Java source");
    fs::write(recovered.join("LoopBool.java"), recovered_source.as_bytes())
        .expect("write the recovered class source");

    for (dir, label) in [(&original, "original"), (&recovered, "recovered")] {
        fs::write(dir.join("LoopRunner.java"), RUNNER_SOURCE)
            .expect("write the Java 8 execution runner");
        javac(dir, label);
    }
    assert_eq!(
        fs::read(original.join("LoopBool.class")).expect("read compiled original class"),
        LOOP_BOOL,
        "the Java 8 source compile must reproduce the frozen original class"
    );

    let original_output = run_java(&original, "original");
    let recovered_output = run_java(&recovered, "recovered");
    let expected = INPUTS
        .iter()
        .map(|(a, b, result)| format!("{a},{b}={result}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(
        original_output.trim(),
        expected,
        "the original class is the oracle"
    );
    assert_eq!(
        recovered_output.trim(),
        expected,
        "the recompiled recovered class preserves the loop exit and result"
    );
}

const RUNNER_SOURCE: &str = r#"public final class LoopRunner {
    public static void main(String[] args) {
        System.out.println("1,0=" + LoopBool.andWhile(1, 0));
        System.out.println("1,1=" + LoopBool.andWhile(1, 1));
        System.out.println("-1,1=" + LoopBool.andWhile(-1, 1));
    }
}
"#;

fn javac(dir: &Path, label: &str) {
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir)
        .args(["LoopBool.java", "LoopRunner.java"])
        .current_dir(dir)
        .output()
        .unwrap_or_else(|error| panic!("start javac for {label} fixture: {error}"));
    assert!(
        output.status.success(),
        "javac refused the {label} fixture: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_java(dir: &Path, label: &str) -> String {
    let mut child = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg("LoopRunner")
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("start {label} Java runner: {error}"));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match child.try_wait().expect("poll Java process") {
            Some(_) => break,
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            None => {
                child.kill().expect("stop Java runner after timeout");
                let _ = child.wait_with_output();
                panic!("{label} Java runner exceeded the 5 second timeout");
            }
        }
    }
    let output = child.wait_with_output().expect("collect Java output");
    assert!(
        output.status.success(),
        "{label} Java runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the Java runner writes UTF-8")
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-loop-boolean-exit-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create temporary Java comparison directory");
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
