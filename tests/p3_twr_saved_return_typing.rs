//! `recover-saved-return-value-typing`: the declaration of a `try`-with-resources saved-return
//! local spells the type the saved value's own producer names, not the frame's conservative
//! unknown reference. A `ldc` constant names no class at the instruction (`frame.rs` keeps the
//! reference unknown), so the first write's `written_type` answer fell to `Object` — and a
//! `String`-returning method could not compile `Object local1 = "in"; return local1;`.
//!
//! The committed inputs are the patrol's frozen `T2.class` and this slice's frozen `Wtyping.class`
//! (javac 23.0.1 `--release 8 -g:none`), both SHA-256 in the evidence README. Assertions, per
//! behavior:
//!
//! * the **text**: every non-null constant family (`String` literal, `int` literal, constructor,
//!   call result, class literal) spells its own type; the `null` control keeps the pre-existing
//!   `Object` spelling word for word, and so does every member the change does not touch;
//! * the **compile**: the recovered `T2` recompiles under `javac --release 8` exactly as
//!   presented — no source patch — and runs the original class's paths under `java -Xverify:all`
//!   (the `Wtyping` class carries the `null` control, whose pre-existing spelling is this slice's
//!   pinned non-goal, so its compile leg lives in the evidence directory, not here).

use jarde::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const T2: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/fixture/T2.class"
);
const WTYPING: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/saved-return-typing/original/Wtyping.class"
);

/// The run of the original frozen `T2.main` — the text the recovered class prints too (the
/// compile-and-run leg records it beside the evidence directory's three-way column).
const T2_EXPECTED_RUN: &str = "done\nin\ndone\ndone\n";

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

fn write_source(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).expect("the scratch source writes");
    path
}

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "jarde-saved-return-typing-{tag}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock runs")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("the scratch directory creates");
    dir
}

#[test]
fn the_saved_return_declaration_spells_the_constants_own_type() {
    let text = member_text(
        "T2",
        T2,
        "public static java.lang.String voidBodyReturnInside",
    );
    assert!(
        text.contains(
            "try (T2 local0 = new T2()) {\n            touch(local0);\n            java.lang.String local1 = \"in\";\n            return local1;\n        }"
        ),
        "the saved return presents the constant's type, and the return compiles:\n{text}"
    );
    assert!(
        !text.contains("Object local1"),
        "no conservative spelling stays beside a named constant:\n{text}"
    );
}

#[test]
fn every_saved_value_family_spells_its_producer_and_null_stays_object() {
    let expectations = [
        (
            "public static java.lang.String stringLiteral",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            java.lang.String local1 = \"in\";\n            return local1;\n        }",
        ),
        (
            "public static int intLiteral",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            int local1 = 42;\n            return local1;\n        }",
        ),
        (
            "public static java.lang.StringBuilder constructorValue",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            java.lang.StringBuilder local1 = new java.lang.StringBuilder(\"built\");\n            return local1;\n        }",
        ),
        (
            "public static java.lang.String callReturn",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            java.lang.String local1 = java.lang.String.valueOf(7);\n            return local1;\n        }",
        ),
        (
            "public static java.lang.String nullValue",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            Object local1 = null;\n            return local1;\n        }",
        ),
        (
            "public static java.lang.Class classLiteral",
            "try (Wtyping local0 = new Wtyping()) {\n            touch(local0);\n            java.lang.Class local1 = java.lang.String.class;\n            return local1;\n        }",
        ),
    ];
    for (method, expected) in expectations {
        let text = member_text("Wtyping", WTYPING, method);
        assert!(
            text.contains(expected),
            "{method} presents the saved value's own type:\n{text}"
        );
    }
}

#[test]
fn the_recovered_t2_compiles_as_presented_and_runs_the_original_paths() {
    let jarde_dir = scratch_dir("compile");
    javac(&[write_source(&jarde_dir, "T2.java", &class_text(T2, "T2"))]);

    let original_dir = scratch_dir("original");
    std::fs::write(original_dir.join("T2.class"), T2).expect("the frozen class writes");
    assert_eq!(
        java(&original_dir, "T2", &[]),
        T2_EXPECTED_RUN,
        "the original class's own paths"
    );
    assert_eq!(
        java(&jarde_dir, "T2", &[]),
        T2_EXPECTED_RUN,
        "the recovered class runs the same paths"
    );
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
