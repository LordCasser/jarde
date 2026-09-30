//! `recover-twrcall-statement-bodies` (CF-17a): the guarded-body statement subset of a
//! `try`-with-resources certificate admits one discarded call — a non-`void` invocation whose
//! result the very next instruction of the same block, a `pop`, drops whole — and the presentation
//! already writes that statement through its own discard channel (P3 2c.31).
//!
//! The committed inputs are the patrol's frozen `T2.class` and the `-17a` family's `V17a.class`
//! (javac 23.0.1 `--release 8 -g:none`) and `N17a.class` (JDK 23 Class-File API, generator beside
//! the fixtures), all SHA-256 in the evidence README. Assertions, per behavior:
//!
//! * the **text**: `T2.popBody` presents `local0.toString();` and the mixed body presents the
//!   discard statement beside the `void` call; the void-only controls keep their exact text;
//! * the **refusal**: every verifier-valid negative shape (`invoke; dup; pop; pop`,
//!   `invoke; checkcast; pop`, `invoke; dup; astore; pop`, `invokestatic; pop2`) keeps the whole
//!   body refusal the change found on mainline, with the same diagnostic and BCI;
//! * the **compile**: the recovered `T2` and `V17a` classes recompile under `javac --release 8`
//!   as presented — `recover-saved-return-value-typing` spells the saved-return declaration from
//!   the constant's own type, so the text no longer needs the patch the evidence README once
//!   recorded for this leg (`return "X";`);
//! * the **runtime**: the original frozen bytes and the recovered classes print the same lines
//!   under `java -Xverify:all`, normal and exception-injected paths included; the recorded JADX
//!   column (`results/three-way/run-sha256.txt`) printed the same outputs for the original class;
//! * the **budget**: a stopped or cancelled run over a recovered body publishes nothing.

use jarde::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const T2: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/fixture/T2.class"
);
const V17A: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/twrcall-17a/original/V17a.class"
);
const V17A_G17: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/twrcall-17a/original/V17a$G17.class"
);
const V17A_G17_IMPL: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/twrcall-17a/original/V17a$G17Impl.class"
);
const V17A_RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/twrcall-17a/V17aRunner.java"
);
const N17A: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/twrcall-17a/original/N17a.class"
);

/// The run of the original frozen `T2.main` — the text every compileable leg printed (recorded
/// `results/three-way/run-sha256.txt`, where the JADX leg printed the same). `voidBodyReturnInside`
/// returns `"in"`, and `main` prints every result.
const T2_EXPECTED_RUN: &str = "done\nin\ndone\ndone\n";

/// The runs of the original frozen `V17aRunner` — normal, then with the body's own calls and the
/// close throwing (the suppression chain the lowering's two handlers really run).
const V17A_NORMAL_RUN: &str = "\
pureCalls:done
touched
mixedVoidAndCall:done
touched
callBeforeReturnInside:in
callOnlyReturnInside:solo
staticCall:done
virtualCall:done
interfaceCall:done
receivedLocal:done
";
const V17A_BOOM_RUN: &str = "\
pureCalls:caught:close
mixedVoidAndCall:caught:touch|sup:close
callBeforeReturnInside:caught:touch|sup:close
callOnlyReturnInside:caught:close
staticCall:caught:give|sup:close
virtualCall:caught:close
interfaceCall:caught:pick|sup:close
receivedLocal:caught:close
";

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn class_text(bytes: &[u8], class: &str) -> String {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen class opens");
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
    match Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("the frozen class answers the class-source request")
    {
        OperationOutcome::Performed(report) => report.text,
        other => panic!("one frozen class produced a complete answer: {other:?}"),
    }
}

fn member_text(class: &str, bytes: &[u8], method: &str) -> String {
    let text = class_text(bytes, class);
    let start = text
        .find(&format!("{method}()"))
        .unwrap_or_else(|| panic!("{class}.{method} is presented"));
    text[start..]
        .split_once("\n    }")
        .map(|(body, _)| body.to_owned())
        .expect("the member's own text ends")
}

/// One member's own recovery report, as the class-source run performed it.
fn member_report(class: &str, bytes: &[u8], method: &str) -> RecoveryReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen class opens");
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
    let report = match Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("the frozen class answers the class-source request")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen class produced a complete answer: {other:?}"),
    };
    let member = report
        .methods
        .iter()
        .find(|member| member.item.name.raw().0 == method.as_bytes())
        .unwrap_or_else(|| panic!("the fixture has {method}()"));
    match &member.outcome {
        ClassSourceOutcome::Recovered { report, .. } => (**report).clone(),
        other => panic!("{method}() has a recovery report: {other:?}"),
    }
}

fn write_source(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).expect("the scratch source writes");
    path
}

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "jarde-twrcall-{tag}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock runs")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("the scratch directory creates");
    dir
}

fn javac(files: &[PathBuf]) {
    let dir = files[0].parent().expect("the scratch directory exists");
    let compile = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-cp")
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .args(files)
        .output()
        .expect("the installed JDK provides javac");
    assert!(
        compile.status.success(),
        "javac accepted the recovered source:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}

fn java(dir: &Path, class: &str, args: &[&str]) -> String {
    let run = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(class)
        .args(args)
        .output()
        .expect("the installed JDK provides java");
    assert!(
        run.status.success(),
        "`java -Xverify:all {class}` ran clean:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run's output is text")
}

#[test]
fn a_discarded_call_body_presents_the_call_statement() {
    let text = member_text("T2", T2, "public static java.lang.String popBody");
    assert!(
        text.contains("try (T2 local0 = new T2()) {\n            local0.toString();\n        }"),
        "the popBody body presents the discarded call as the statement it is:\n{text}"
    );
    let mixed = member_text("T2", T2, "public static java.lang.String popBodyVoidTouch");
    let call = mixed
        .find("local0.hashCode();")
        .expect("the discarded call statement is presented");
    let touch = mixed
        .find("touch(local0);")
        .expect("the void call keeps its statement");
    assert!(
        call < touch,
        "the body presents its statements in the order the bytecode runs them:\n{mixed}"
    );
}

#[test]
fn the_void_call_controls_keep_their_exact_text() {
    let text = class_text(T2, "T2");
    for control in [
        "try (T2 local0 = new T2()) {\n            touch(local0);\n        }\n        return \"done\";",
        "try (T2 local0 = new T2()) {\n            touch(local0);\n            java.lang.String local1 = \"in\";\n            return local1;\n        }",
    ] {
        assert!(
            text.contains(control),
            "the void-call control keeps its exact text:\n{text}"
        );
    }
}

#[test]
fn a_result_the_source_receives_keeps_the_assignment_channel() {
    let text = member_text("V17a", V17A, "public static java.lang.String receivedLocal");
    assert!(
        text.contains("java.lang.String local1 = local0.toString();"),
        "a received result stays the declaration the assignment channel already presents:\n{text}"
    );
    assert!(
        !text.contains("local1;\n            local1"),
        "no discard statement is written beside the declaration:\n{text}"
    );
}

#[test]
fn the_variant_family_presents_every_discarded_call_shape() {
    let expectations = [
        (
            "pureCalls",
            "try (V17a local0 = new V17a()) {\n            local0.toString();\n            local0.hashCode();\n        }",
        ),
        (
            "mixedVoidAndCall",
            "try (V17a local0 = new V17a()) {\n            touch(local0);\n            local0.toString();\n        }",
        ),
        (
            "callBeforeReturnInside",
            "touch(local0);\n            local0.hashCode();\n            java.lang.String local1 = \"in\";",
        ),
        (
            "callOnlyReturnInside",
            "try (V17a local0 = new V17a()) {\n            local0.toString();\n            java.lang.String local1 = \"solo\";",
        ),
        (
            "staticCall",
            "try (V17a local0 = new V17a()) {\n            give();\n        }",
        ),
        (
            "virtualCall",
            "try (V17a local0 = new V17a()) {\n            local0.toString();\n        }",
        ),
        (
            "interfaceCall",
            "try (V17a local0 = new V17a()) {\n            pick().get();\n        }",
        ),
    ];
    for (method, expected) in expectations {
        let text = member_text(
            "V17a",
            V17A,
            &format!("public static java.lang.String {method}"),
        );
        assert!(
            text.contains(expected),
            "{method} presents the recovered body:\n{text}"
        );
    }
}

#[test]
fn verifier_valid_pop_shapes_outside_the_criterion_keep_the_body_refusal() {
    // Every negative body kept the refusal mainline states (`jre_guard_body` at the body's own
    // first BCI): the pairing demands an adjacent non-void call whose written value the pop alone
    // reads, so a `pop` fed by a `dup` or a `checkcast`, a stored copy beside the discard, and the
    // two-slot `pop2` discard each stay a body this subset does not carry.
    let negatives = [
        (
            "popWrongValue",
            "the first `pop` reads the copy, not the call's result",
        ),
        (
            "popAfterCast",
            "a `checkcast` stands between the call and the `pop`",
        ),
        (
            "popSecondReader",
            "the stored copy reads the value beside the `pop`",
        ),
        (
            "pop2Discard",
            "the two-slot discard is no `pop` of this subset",
        ),
    ];
    for (method, why) in negatives {
        let report = member_report("N17a", N17A, method);
        assert_eq!(
            report.quality,
            Quality::Fallback,
            "{method} ({why}) keeps its refusal, not a recovery"
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|item| item.code == "jre_guard_body"),
            "{method} keeps the guarded-body diagnostic: {:?}",
            report.diagnostics
        );
        assert!(
            report
                .text
                .contains("local 0 crosses a quoted fallback region"),
            "{method} degrades as one quoted fallback, whole:\n{}",
            report.text
        );
    }
}

#[test]
fn the_negative_refusal_is_the_guard_body_diagnostic_at_the_body_bci() {
    for method in [
        "popWrongValue",
        "popAfterCast",
        "popSecondReader",
        "pop2Discard",
    ] {
        let report = member_report("N17a", N17A, method);
        let body = report
            .diagnostics
            .iter()
            .find(|item| item.code == "jre_guard_body")
            .unwrap_or_else(|| panic!("{method} keeps the guarded-body diagnostic"));
        assert!(
            body.message.contains("BCI 8:"),
            "the refusal names the body's own first BCI: {:?}",
            body.message
        );
    }
}

#[test]
fn the_recovered_classes_compile_and_run_the_original_paths() {
    let jarde_dir = scratch_dir("compile");
    write_source(&jarde_dir, "V17a.java", &class_text(V17A, "V17a"));
    write_source(
        &jarde_dir,
        "V17a$G17.java",
        &class_text(V17A_G17, "V17a$G17"),
    );
    write_source(
        &jarde_dir,
        "V17a$G17Impl.java",
        &class_text(V17A_G17_IMPL, "V17a$G17Impl"),
    );
    write_source(&jarde_dir, "V17aRunner.java", V17A_RUNNER);
    javac(&[
        write_source(&jarde_dir, "T2.java", &class_text(T2, "T2")),
        jarde_dir.join("V17a.java"),
        jarde_dir.join("V17a$G17.java"),
        jarde_dir.join("V17a$G17Impl.java"),
        jarde_dir.join("V17aRunner.java"),
    ]);

    let original_dir = scratch_dir("original");
    let original_class = original_dir.join("T2.class");
    std::fs::write(&original_class, T2).expect("the frozen class writes");
    for (name, bytes) in [
        ("V17a.class", V17A),
        ("V17a$G17.class", V17A_G17),
        ("V17a$G17Impl.class", V17A_G17_IMPL),
    ] {
        std::fs::write(original_dir.join(name), bytes).expect("the frozen class writes");
    }
    write_source(&original_dir, "V17aRunner.java", V17A_RUNNER);
    javac(&[original_dir.join("V17aRunner.java")]);

    // The negative class is verifier-valid bytecode that runs — it is the *recovery* that refuses.
    let negative_dir = scratch_dir("negative");
    std::fs::write(negative_dir.join("N17a.class"), N17A).expect("the frozen class writes");
    assert_eq!(
        java(&negative_dir, "N17a", &[]),
        "done\ndone\ndone\ndone\n",
        "the negative shapes themselves run"
    );

    // The original classes and the recovered classes print the same lines on every path.
    assert_eq!(java(&original_dir, "T2", &[]), T2_EXPECTED_RUN);
    assert_eq!(java(&jarde_dir, "T2", &[]), T2_EXPECTED_RUN);
    assert_eq!(java(&original_dir, "V17aRunner", &[]), V17A_NORMAL_RUN);
    assert_eq!(java(&jarde_dir, "V17aRunner", &[]), V17A_NORMAL_RUN);
    assert_eq!(java(&original_dir, "V17aRunner", &["boom"]), V17A_BOOM_RUN);
    assert_eq!(java(&jarde_dir, "V17aRunner", &["boom"]), V17A_BOOM_RUN);
}

#[test]
fn a_stopped_or_cancelled_run_publishes_no_partial_pop_body() {
    let complete = class_text(T2, "T2");
    let mut bounded =
        task_budget(&[
            BudgetOverride::new("output_bytes", complete.len() as u64 - 1)
                .expect("a legal output cap"),
        ])
        .expect("the task budget accepts the cap");
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(T2.to_vec()), &mut budget())
        .expect("the frozen class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("T2"),
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
    let stopped =
        match Engine::new().class_source(slice::from_ref(&snapshot), &request, &mut bounded) {
            Ok(OperationOutcome::Performed(report)) => report,
            other => panic!("the bounded run answers: {other:?}"),
        };
    assert!(
        !stopped.text.contains("local0.toString();"),
        "a stopped run publishes no recovered discard statement: {}",
        stopped.text
    );
    assert!(!matches!(
        stopped.execution,
        ExecutionReport::Complete { .. }
    ));

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    match Engine::new().class_source(slice::from_ref(&snapshot), &request, &mut cancelled) {
        // A run that completed anyway still publishes no recovered discard statement.
        Ok(OperationOutcome::Performed(report)) => assert!(
            !report.text.contains("local0.toString();"),
            "a cancelled run publishes no recovered discard statement: {}",
            report.text
        ),
        // The ordinary cancelled answer: the search never executed, so nothing was published.
        Ok(_) => {}
        other => panic!("the cancelled run answers: {other:?}"),
    }
}
