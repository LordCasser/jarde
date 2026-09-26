//! P3 3.3b: compound bottom-tested boolean loops keep body effects in the body.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::slice;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DO_LOOP: &[u8] =
    include_bytes!("fixtures/proved-java-structure/loop-boolean-do/DoLoopBool.class");
const DO_LOOP_SOURCE: &str =
    include_str!("fixtures/proved-java-structure/loop-boolean-do/DoLoopBool.java");
const JADX_DO_LOOP_SOURCE: &str =
    include_str!("../openspec/evidence/java-syntax-2026-09-26/loop-boolean-do/jadx.java");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn report_bytes(bytes: &[u8], class_name: &str) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the Java class fixture opens");
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
fn compound_do_while_tests_keep_the_shared_body_prefix_and_have_execution_parity() {
    let report = report_bytes(DO_LOOP, "DoLoopBool");
    let and_do = method(&report, "andDo");
    let or_do = method(&report, "orDo");

    for (method, operator) in [(and_do, "&&"), (or_do, "||")] {
        assert_eq!(
            recovery(method).quality,
            jarde_jvm::ir::Quality::Structured,
            "{}",
            method.text
        );
        assert!(
            method.text.contains("do {")
                && method
                    .text
                    .contains(&format!("while (arg0 > 0 {operator} arg1 > 0)")),
            "the proved bottom-tested chain stays in its do-while condition:\n{}",
            method.text
        );
        for bci in [2, 5, 8] {
            let mapped = recovery(method).source_map.of_bci(bci);
            assert!(
                !mapped.is_empty(),
                "body effect BCI {bci} is accounted once in {:?}:\n{:?}",
                method.item.name.raw(),
                recovery(method).source_map.segments()
            );
            let statement = match bci {
                2 => "local2 = local2 + 1;",
                5 => "arg0 = arg0 - 1;",
                8 => "arg1 = arg1 - 1;",
                _ => unreachable!(),
            };
            assert_eq!(
                method.text.matches(statement).count(),
                1,
                "body effect at BCI {bci} is emitted once:\n{}",
                method.text
            );
        }
    }
    for method in [and_do, or_do] {
        for bci in [12, 16] {
            let mapped = recovery(method)
                .source_map
                .text_of_bci(&recovery(method).text, bci);
            assert!(
                !mapped.is_empty(),
                "test BCI {bci} maps to the compound do-while condition:\n{:?}",
                recovery(method).source_map.segments()
            );
            assert!(
                mapped.iter().any(|segment| segment.contains("arg")),
                "test BCI {bci} maps to a rendered operand:\n{mapped:?}"
            );
            assert!(
                !recovery(method).source_map.of_bci(bci).is_empty(),
                "test BCI {bci} has a source-map entry"
            );
        }
        let expected_condition = if method.item.name.raw().0 == b"andDo" {
            "arg0 > 0 && arg1 > 0"
        } else {
            "arg0 > 0 || arg1 > 0"
        };
        assert_eq!(method.text.matches(expected_condition).count(), 1);
    }

    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered = scratch.path().join("recovered");
    let jadx = scratch.path().join("jadx");
    fs::create_dir_all(&original).expect("create original comparison directory");
    fs::create_dir_all(&recovered).expect("create recovered comparison directory");
    fs::create_dir_all(&jadx).expect("create JADX comparison directory");
    fs::write(original.join("DoLoopBool.java"), DO_LOOP_SOURCE).expect("write the fixture source");
    fs::write(recovered.join("DoLoopBool.java"), &report.text)
        .expect("write the recovered class source");
    fs::write(jadx.join("DoLoopBool.java"), JADX_DO_LOOP_SOURCE)
        .expect("write the frozen JADX class source");
    for (dir, label) in [(&original, "original"), (&recovered, "recovered")] {
        fs::write(dir.join("DoLoopRunner.java"), RUNNER_SOURCE)
            .expect("write the three-input runner");
        javac(dir, label);
    }
    fs::write(
        jadx.join("DoLoopRunner.java"),
        format!("package defpackage;\n{RUNNER_SOURCE}"),
    )
    .expect("write the package-local JADX runner");
    compile_jadx(&jadx);
    assert_eq!(
        fs::read(original.join("DoLoopBool.class")).expect("read rebuilt original"),
        DO_LOOP,
        "the fixture source reproduces the frozen original class"
    );
    let expected = "3,3:3,3\n3,-1:1,3\n0,0:1,1";
    assert_eq!(run_java(&original, "original").trim(), expected);
    assert_eq!(run_java(&recovered, "recovered").trim(), expected);
    assert_eq!(
        run_java_named(&jadx, "defpackage.DoLoopRunner", "JADX").trim(),
        expected,
        "the frozen JADX projection is a third executable comparison"
    );
}

#[test]
fn a_mixed_latch_chain_is_not_presented_as_one_short_circuit_condition() {
    let scratch = Scratch::new();
    fs::create_dir_all(scratch.path()).expect("create mixed-loop fixture directory");
    fs::write(
        scratch.path().join("MixedDo.java"),
        r#"public final class MixedDo {
    static int run(int a, int b, int c) {
        int n = 0;
        do { n++; a--; b--; c--; } while (a > 0 && b > 0 || c > 0);
        return n;
    }
}
"#,
    )
    .expect("write the mixed-chain negative case");
    compile_source(scratch.path(), "MixedDo.java", "mixed do-while");
    let bytes = fs::read(scratch.path().join("MixedDo.class")).expect("read compiled case");
    let mixed_report = report_bytes(&bytes, "MixedDo");
    let mixed = method(&mixed_report, "run");
    assert!(
        !mixed.text.contains("do {") || mixed.text.contains("@bytecode"),
        "mixed AND/OR latch topology remains an explicit refusal:\n{}",
        mixed.text
    );
}

#[test]
fn an_effect_between_latch_tests_is_not_moved_into_or_out_of_the_condition() {
    let scratch = Scratch::new();
    fs::create_dir_all(scratch.path()).expect("create effectful-loop fixture directory");
    fs::write(
        scratch.path().join("EffectDo.java"),
        r#"public final class EffectDo {
    static int effects;
    static int run(int a, int b) {
        int n = 0;
        do { n++; a--; b--; } while (a > 0 && (effects++ >= 0) && b > 0);
        return n;
    }
}
"#,
    )
    .expect("write the effectful-chain negative case");
    compile_source(scratch.path(), "EffectDo.java", "effectful do-while");
    let bytes = fs::read(scratch.path().join("EffectDo.class")).expect("read compiled case");
    let effect_report = report_bytes(&bytes, "EffectDo");
    let effect = method(&effect_report, "run");
    assert!(
        !effect.text.contains("do {") || effect.text.contains("@bytecode"),
        "the unsupported test-side increment does not become a condition or disappear:\n{}",
        effect.text
    );
}

#[test]
fn a_continue_edge_that_enters_the_latch_chain_by_a_second_route_is_quoted() {
    let scratch = Scratch::new();
    fs::create_dir_all(scratch.path()).expect("create extra-entry loop directory");
    fs::write(
        scratch.path().join("ExtraDo.java"),
        r#"public final class ExtraDo {
    static int run(int a, int b, boolean skip) {
        int n = 0;
        do {
            n++;
            a--;
            b--;
            if (skip) continue;
            n += 2;
        } while (a > 0 && b > 0);
        return n;
    }
}
"#,
    )
    .expect("write the additional-entry control case");
    compile_source(scratch.path(), "ExtraDo.java", "extra-entry do-while");
    let bytes = fs::read(scratch.path().join("ExtraDo.class")).expect("read compiled case");
    let extra_report = report_bytes(&bytes, "ExtraDo");
    let extra = method(&extra_report, "run");
    assert_eq!(recovery(extra).quality, jarde_jvm::ir::Quality::Fallback);
    assert!(
        extra.text.contains("@bytecode") && !extra.text.contains("do {"),
        "the continue edge adds a second normal predecessor into the latch tests, so the compound
         test chain is quoted instead of being presented with one entry:\n{}",
        extra.text
    );
}

const RUNNER_SOURCE: &str = r#"public final class DoLoopRunner {
    public static void main(String[] args) {
        int[][] inputs = {{3, 3}, {3, -1}, {0, 0}};
        for (int[] input : inputs) {
            System.out.println(input[0] + "," + input[1] + ":"
                    + DoLoopBool.andDo(input[0], input[1]) + ","
                    + DoLoopBool.orDo(input[0], input[1]));
        }
    }
}
"#;

fn compile_source(dir: &Path, source_file: &str, label: &str) {
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir)
        .arg(source_file)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|error| panic!("start javac for {label}: {error}"));
    assert!(
        output.status.success(),
        "javac refused {label}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn javac(dir: &Path, label: &str) {
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir)
        .args(["DoLoopBool.java", "DoLoopRunner.java"])
        .current_dir(dir)
        .output()
        .unwrap_or_else(|error| panic!("start javac for {label}: {error}"));
    assert!(
        output.status.success(),
        "javac refused the {label} fixture: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn compile_jadx(dir: &Path) {
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir)
        .args(["DoLoopBool.java", "DoLoopRunner.java"])
        .current_dir(dir)
        .output()
        .expect("start javac for the frozen JADX projection");
    assert!(
        output.status.success(),
        "javac refused the frozen JADX projection: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_java(dir: &Path, label: &str) -> String {
    run_java_named(dir, "DoLoopRunner", label)
}

fn run_java_named(dir: &Path, main: &str, label: &str) -> String {
    let mut child = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg(main)
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
    let output = child
        .wait_with_output()
        .expect("collect Java runner output");
    assert!(
        output.status.success(),
        "{label} JVM verification or execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("runner output is UTF-8")
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock follows the epoch")
            .as_nanos();
        Self(std::env::temp_dir().join(format!("jarde-loop-do-{nonce}")))
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
