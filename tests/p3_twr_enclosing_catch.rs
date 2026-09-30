//! `recover-enclosing-named-catch` (CF-17b): the named `catch` a compiler winds around a whole
//! `try`-with-resources lowering is the clause the statement presents beside its header.
//!
//! One named row whose range covers the claim from its first instruction to the end of the cleanup
//! is javac's `try (…) { … } catch (E e) { … }`. The guard reads it as the statement's own clause
//! where its handler proves as one straight block — the entry store binds the clause's parameter,
//! the rest is the statement whitelist plus a value `return`, and the block's only normal
//! continuation is the statement's continuation — and the builder writes the body between the
//! clause's braces with the handler block in the statement's ledger.
//!
//! The committed inputs are the patrol's frozen `T3.class`, `T1.class` and `C4.class` and the
//! `-17b` family's `W17b.class` (javac 23.0.1 `--release 8 -g:none`) and `N17b.class` (JDK 23
//! Class-File API, generator beside the fixtures), all SHA-256 in the evidence README. Assertions,
//! per behavior:
//!
//! * the **text**: the four fixed shapes present `try (…) { … } catch (…) { … }` whole — the
//!   return handler, the call-statement handler (over the 17a discarded-call body), and the
//!   crossing 17a+17b shapes with them;
//! * the **refusal**: every verifier-valid negative shape (a catch-all whole-construct row, a
//!   partial row, a row whose handler is the claim's own, a branching handler body, a double catch,
//!   a multi-catch) keeps the refusal mainline states, with the same fallback list;
//! * the **compile**: the recovered `T3`, `T1`, `C4` and `W17b` classes recompile under
//!   `javac --release 8`;
//! * the **runtime**: the original frozen bytes and the recovered classes print the same lines
//!   under `java -Xverify:all`, the injected `IllegalStateException` path, the close-throwing path
//!   and the suppression path included; the recorded JADX column (`results/three-way/`) printed
//!   the same outputs for the original class on every shape but `W17b.closeThrows`, where the
//!   reference tool moves the normal close inside the protected range and invents a suppression —
//!   its own defect, recorded beside the runs;
//! * the **budget**: a stopped or cancelled run over a recovered body publishes nothing.

use jarde::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

macro_rules! patrol {
    ($rest:literal) => {
        concat!(
            "../openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/",
            $rest
        )
    };
}

const T3: &[u8] = include_bytes!(patrol!("fixture/T3.class"));
const T1: &[u8] = include_bytes!(patrol!("fixture/T1.class"));
const C4: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf15-crossing-patrol/fixture/C4.class"
);
const W17B: &[u8] = include_bytes!(patrol!("enclosing-17b/original/W17b.class"));
const N17B: &[u8] = include_bytes!(patrol!("enclosing-17b/original/N17b.class"));

/// The run of the original frozen `T3.main`, `T1.main` and `C4.main` — the text every compileable
/// leg printed (recorded `results/three-way/`, where the JADX leg printed the same).
const FIXED_RUNS: [(&str, &str); 3] = [
    ("T3", "done\ndone\n"),
    ("T1", "done\ndone\ndone\ndone\n"),
    ("C4", "done\na\n"),
];

/// The run of the original frozen `W17b.main` — the normal path, the body's own `IllegalStateException`
/// into the clause, the call-statement handler, the close's exception the clause catches, and the
/// suppression the lowering performs when body and close both raise.
const W17B_RUN: &str = "\
done
caught:body
done
caught:close:[]
caught:body:[java.lang.IllegalStateException: close]
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
        "jarde-twrcatch-{tag}-{}",
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

fn java(dir: &Path, class: &str) -> String {
    let run = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(class)
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
fn the_fixed_shapes_present_the_statement_and_its_clause() {
    let expectations = [
        (
            "T3",
            T3,
            "public static java.lang.String voidNamed",
            "try (T3 local0 = new T3()) {\n            touch(local0);\n        } catch (java.lang.IllegalStateException local0) {\n            return \"caught\";\n        }\n        return \"done\";",
        ),
        (
            "T3",
            T3,
            "public static java.lang.String voidNamedRecover",
            "try (T3 local0 = new T3()) {\n            touch(local0);\n        } catch (java.lang.IllegalStateException local0) {\n            touch((T3) null);\n        }",
        ),
        (
            "T1",
            T1,
            "public static java.lang.String twrVoidNamed",
            "try (T1 local0 = new T1()) {\n            local0.hashCode();\n        } catch (java.lang.IllegalStateException local0) {\n            return \"caught\";\n        }",
        ),
        (
            "T1",
            T1,
            "public static java.lang.String twrPopNamed",
            "try (T1 local0 = new T1()) {\n            local0.toString();\n        } catch (java.lang.IllegalStateException local0) {\n            return \"caught\";\n        }",
        ),
        (
            "C4",
            C4,
            "public static java.lang.String twrNamed",
            "try (C4 local0 = new C4()) {\n            local0.toString();\n        } catch (java.lang.IllegalStateException local0) {\n            return \"caught\";\n        }",
        ),
    ];
    for (class, bytes, method, expected) in expectations {
        let text = member_text(class, bytes, method);
        assert!(
            text.contains(expected),
            "{class}.{method} presents the whole construct:\n{text}"
        );
    }
}

#[test]
fn the_clause_header_names_the_row_and_the_binding_store() {
    // The parameter is the slot the handler's own entry store fills — javac reuses the resource's
    // slot, and the two headers state the same name because the resource is out of scope in the
    // clause, exactly as the source's own two names are.
    let report = member_report("T3", T3, "voidNamed");
    assert_eq!(report.quality, Quality::Structured, "{report:?}");
    assert!(report.fallbacks.is_empty(), "{:?}", report.fallbacks);
    assert!(
        report
            .text
            .contains("catch (java.lang.IllegalStateException local0)"),
        "{}",
        report.text
    );
    // The clause's handler entry is an anchor of the statement's own text: what the text states is
    // what the table named.
    let anchors: BTreeSet<u32> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis())
        .collect();
    assert!(
        anchors.contains(&38),
        "the handler entry at BCI 38 is recorded beside the text: {anchors:?}"
    );
}

#[test]
fn verifier_valid_surroundings_outside_the_clause_keep_the_refusal() {
    // Every negative kept the refusal mainline states: five at `jre_guard_unexplained_row` (the
    // catch-all row, the row over the claim's own handler, the branching handler body, the double
    // catch and the multi-catch), and the partial row at the uncovered-blocks fallback the region
    // layer states for a handler the statement never claims. None presents a catch.
    let negatives = [
        ("catchAllSurround", "jre_guard_unexplained_row"),
        ("partialRow", "jre_region_uncovered_blocks"),
        ("overlapHandler", "jre_guard_unexplained_row"),
        ("branchingHandler", "jre_guard_unexplained_row"),
        ("doubleCatch", "jre_guard_unexplained_row"),
        ("multiCatch", "jre_guard_unexplained_row"),
    ];
    for (method, code) in negatives {
        let report = member_report("N17b", N17B, method);
        assert_eq!(
            report.quality,
            Quality::Fallback,
            "{method} keeps its refusal, not a recovery: {}",
            report.text
        );
        let fallbacks: Vec<_> = report.fallbacks.iter().copied().collect();
        assert!(
            fallbacks.contains(&code),
            "{method} keeps the {code} refusal: {fallbacks:?}"
        );
        assert!(
            !report.text.contains("} catch ("),
            "{method} presents no catch clause:\n{}",
            report.text
        );
    }
}

#[test]
fn the_negative_shapes_themselves_run() {
    // The negative class is verifier-valid bytecode that runs — it is the *recovery* that refuses.
    let dir = scratch_dir("negative");
    std::fs::write(dir.join("N17b.class"), N17B).expect("the frozen class writes");
    assert_eq!(
        java(&dir, "N17b"),
        "done\ndone\ndone\ndone\ndone\ndone\n",
        "the negative shapes themselves run"
    );
}

#[test]
fn the_recovered_classes_compile_and_run_the_original_paths() {
    let jarde_dir = scratch_dir("compile");
    for (class, bytes) in [("T3", T3), ("T1", T1), ("C4", C4), ("W17b", W17B)] {
        write_source(
            &jarde_dir,
            &format!("{class}.java"),
            &class_text(bytes, class),
        );
    }
    javac(&[
        jarde_dir.join("T3.java"),
        jarde_dir.join("T1.java"),
        jarde_dir.join("C4.java"),
        jarde_dir.join("W17b.java"),
    ]);

    let original_dir = scratch_dir("original");
    for (name, bytes) in [
        ("T3.class", T3),
        ("T1.class", T1),
        ("C4.class", C4),
        ("W17b.class", W17B),
    ] {
        std::fs::write(original_dir.join(name), bytes).expect("the frozen class writes");
    }

    // The original classes and the recovered classes print the same lines on every path.
    for (class, expected) in FIXED_RUNS {
        assert_eq!(java(&original_dir, class), expected, "original {class}");
        assert_eq!(java(&jarde_dir, class), expected, "recovered {class}");
    }
    assert_eq!(java(&original_dir, "W17b"), W17B_RUN, "original W17b");
    assert_eq!(java(&jarde_dir, "W17b"), W17B_RUN, "recovered W17b");
}

#[test]
fn a_stopped_or_cancelled_run_publishes_no_partial_catch() {
    // A cap that bites before the first member's own document fits refuses the whole answer: a
    // stopped run publishes nothing partial — no clause, no half of one.
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(W17B.to_vec()), &mut budget())
        .expect("the frozen class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("W17b"),
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
    let mut bounded =
        task_budget(&[BudgetOverride::new("output_bytes", 1).expect("a legal output cap")])
            .expect("the task budget accepts the cap");
    match Engine::new().class_source(slice::from_ref(&snapshot), &request, &mut bounded) {
        Ok(OperationOutcome::Performed(report)) => {
            panic!(
                "a one-byte cap cannot publish a presentation: {}",
                report.text
            )
        }
        // The ordinary stopped answer: the operation's own document never fit, so nothing below
        // it was published either.
        Ok(_) => {}
        other => panic!("the bounded run answers: {other:?}"),
    }

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    match Engine::new().class_source(slice::from_ref(&snapshot), &request, &mut cancelled) {
        // A run that completed anyway still publishes no recovered clause.
        Ok(OperationOutcome::Performed(report)) => assert!(
            !report.text.contains("} catch ("),
            "a cancelled run publishes no recovered clause: {}",
            report.text
        ),
        // The ordinary cancelled answer: the search never executed, so nothing was published.
        Ok(_) => {}
        other => panic!("the cancelled run answers: {other:?}"),
    }
}
