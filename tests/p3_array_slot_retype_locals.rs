//! Complete-class execution oracle for array slot retype locals (change
//! `recover-array-slot-retype-locals`). One local slot that an `int[]` fills and a `boolean[]`
//! refills afterwards is two source locals in the original program; the recovered text presents
//! each definition as its own declaration, and the whole class recompiles with `javac --release 8`
//! and runs like the original. The boundaries stay exactly as they were: a slot whose array writes
//! share one element shape, and a slot whose reads a phi carries across two definitions, keep the
//! one variable — and the one text — they had before.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const A1: &[u8] = include_bytes!("fixtures/p3-array-slot-retype-locals/v8/A1.class");
const A2: &[u8] = include_bytes!("fixtures/p3-array-slot-retype-locals/v8/A2.class");
const V1: &[u8] = include_bytes!("fixtures/p3-array-slot-retype-locals/v8/V1.class");
const V2: &[u8] = include_bytes!("fixtures/p3-array-slot-retype-locals/v8/V2.class");
const V3: &[u8] = include_bytes!("fixtures/p3-array-slot-retype-locals/v8/V3.class");
const V4: &[u8] = include_bytes!("fixtures/p3-array-slot-retype-locals/v8/V4.class");

const A1_BASELINE: &str =
    include_str!("fixtures/p3-array-slot-retype-locals/baseline/A1.jarde.java");
const V2_BASELINE: &str =
    include_str!("fixtures/p3-array-slot-retype-locals/baseline/V2.jarde.java");
const V3_BASELINE: &str =
    include_str!("fixtures/p3-array-slot-retype-locals/baseline/V3.jarde.java");
const V4_BASELINE: &str =
    include_str!("fixtures/p3-array-slot-retype-locals/baseline/V4.jarde.java");

const A2_EXPECTED: &str = include_str!("fixtures/p3-array-slot-retype-locals/expected/A2.stdout");
const A1_EXPECTED: &str = include_str!("fixtures/p3-array-slot-retype-locals/expected/A1.stdout");
const V1_EXPECTED: &str = include_str!("fixtures/p3-array-slot-retype-locals/expected/V1.stdout");
const V2_EXPECTED: &str = include_str!("fixtures/p3-array-slot-retype-locals/expected/V2.stdout");
const V3_EXPECTED: &str = include_str!("fixtures/p3-array-slot-retype-locals/expected/V3.stdout");
const V4_EXPECTED: &str = include_str!("fixtures/p3-array-slot-retype-locals/expected/V4.stdout");

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
            "jarde-p3-array-slot-retype-locals-{}-{class_name}-{nonce}",
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

fn javac(dir: &Path, class_name: &str) {
    let output = std::process::Command::new("javac")
        .args(["--release", "8", "-Xlint:-options", "-d"])
        .arg(dir)
        .arg(format!("{class_name}.java"))
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

/// Runs one class's own `main` under the strict verifier and returns its stdout.
fn run_verified(dir: &Path, class_name: &str) -> String {
    let output = std::process::Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg(class_name)
        .current_dir(dir)
        .output()
        .expect("java is available for strict verification");
    assert!(
        output.status.success(),
        "{class_name} failed strict JVM verification:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("runner output is UTF-8")
}

fn assert_complete(class_name: &str, bytes: &[u8], expected: &str) -> String {
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
    fs::create_dir_all(&oracle).expect("create the original-class oracle directory");
    fs::create_dir_all(&recovered).expect("create the recovered-class directory");
    fs::write(oracle.join(format!("{class_name}.class")), bytes)
        .expect("write the exact original class");
    let original = run_verified(&oracle, class_name);
    assert_eq!(original, expected, "original {class_name} JVM output");
    fs::write(recovered.join(format!("{class_name}.java")), &report.text)
        .expect("write the complete Jarde class");
    javac(&recovered, class_name);
    let recovered_output = run_verified(&recovered, class_name);
    assert_eq!(
        recovered_output, expected,
        "complete Jarde {class_name} semantics"
    );
    report.text
}

#[test]
fn the_retyped_slot_presents_one_declaration_per_definition_and_recompiles() {
    let text = assert_complete("A2", A2, A2_EXPECTED);
    // The first definition keeps the slot's own declaration and the copy the source's for-each
    // lowered to; the second definition presents its own element type under a fresh name the
    // namer already resolves deterministically (two variables one slot holds cannot share one
    // source name — javac --release 8 refuses the re-declaration).
    assert!(
        text.contains("        int[] local2;\n"),
        "the first definition keeps the slot declaration:\n{text}"
    );
    assert!(
        text.contains("        local2 = local0;\n"),
        "the first definition's store stays:\n{text}"
    );
    assert!(
        text.contains("        boolean[] local2_2 = new boolean[3];\n"),
        "the second definition presents its own declaration and type:\n{text}"
    );
    assert!(
        text.contains("        local2_2[side() - 1] = true;\n"),
        "the second definition's reads take its own name:\n{text}"
    );
    assert!(
        text.contains(".append(local2_2[1])"),
        "the return's read of the second definition takes its own name:\n{text}"
    );
    assert!(
        !text.contains("local2 = new boolean"),
        "no write of the second definition's value may present under the first definition's name:\n{text}"
    );
}

#[test]
fn the_three_type_alternation_presents_three_declarations_and_recompiles() {
    let text = assert_complete("V1", V1, V1_EXPECTED);
    assert!(
        text.contains("        int[] local1 = new int[]{side(), side() + 1, side() * 2};\n"),
        "the first segment declares its own array:\n{text}"
    );
    assert!(
        text.contains("        boolean[] local1_2 = new boolean[3];\n"),
        "the second segment declares its own array:\n{text}"
    );
    assert!(
        text.contains(
            "        java.lang.Object[] local1_3 = new java.lang.Object[]{\"x\", \"y\"};\n"
        ),
        "the third segment declares its own array:\n{text}"
    );
}

/// The boundary fixtures are frozen with the recovery text the mainline wrote **before** the
/// change; the same bytes must recover to the same text after it, verbatim, and the class itself
/// must still run like the oracle its expected output states.
fn assert_verbatim(class_name: &str, bytes: &[u8], baseline: &str, expected: &str) {
    let scratch = Scratch::new(class_name);
    fs::write(scratch.path().join(format!("{class_name}.class")), bytes)
        .expect("write the exact boundary class");
    let original = run_verified(scratch.path(), class_name);
    assert_eq!(original, expected, "original {class_name} JVM output");
    let report = recover(bytes, class_name);
    assert_eq!(
        report.text, baseline,
        "{class_name} recovers verbatim across the change"
    );
}

#[test]
fn the_same_element_shape_keeps_one_variable_verbatim() {
    assert_verbatim("V2", V2, V2_BASELINE, V2_EXPECTED);
}

#[test]
fn a_join_that_merges_two_definitions_keeps_one_variable_verbatim() {
    assert_verbatim("V3", V3, V3_BASELINE, V3_EXPECTED);
}

#[test]
fn a_loop_that_merges_two_definitions_keeps_one_variable_verbatim() {
    assert_verbatim("V4", V4, V4_BASELINE, V4_EXPECTED);
}

#[test]
fn the_dynamic_dimension_control_keeps_its_text_verbatim() {
    assert_verbatim("A1", A1, A1_BASELINE, A1_EXPECTED);
    assert_complete("A1", A1, A1_EXPECTED);
}
