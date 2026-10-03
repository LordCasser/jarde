//! Acceptance tests of change `recover-assert-statement-sugar`: javac's `assert` lowering —
//! the synthetic `$assertionsDisabled` field, its `<clinit>` initialization line, and the
//! guarded `throw` at each use site — is recognized as one pattern and presented as the
//! `assert cond [: message];` statement it was compiled from.
//!
//! The anchors the change's spec fixes:
//!
//! 1. the A1 family (both message forms, plus the nested class's own switch) presents `assert`
//!    statements, hides every synthetic layer, recompiles under `javac --release 8`, verifies
//!    under `-Xverify:all`, and prints exactly what the original class files print in **both**
//!    assertion states;
//! 2. a message with evaluation side effects keeps its order: the detail call runs only on the
//!    failure path;
//! 3. the shapes recognition refuses — a guard with an extra statement (hand-patched bytecode),
//!    a switch whose `<clinit>` reads another class's status (hand-patched bytecode), and a
//!    class mixing foldable guards with ones that do not fold — keep the physical presentation
//!    of the field, the line and the guard;
//! 4. a class without asserts stages nothing.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-assert-sugar-{label}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("a private compilation directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn jar_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in entries {
            let (mut entry, config) = zip
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(bytes).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        zip.finish().unwrap();
    }
    output.into_inner()
}

/// One family compiled exactly the way the assert patrol's fixtures were: `--release 8` with
/// `-g:none`, one jar entry per produced class file.
fn compile_family(label: &str, source: &str) -> (Vec<u8>, Vec<(String, Vec<u8>)>) {
    let temp = TestDirectory::new(label);
    let file = source
        .split_once("class ")
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .map(|name| format!("{name}.java"))
        .expect("the source states a class");
    std::fs::write(temp.path().join(&file), source).expect("the family source is written");
    let compiled = Command::new("javac")
        .args(["--release", "8", "-g:none"])
        .arg(&file)
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let mut owned = Vec::new();
    let mut directories = vec![temp.path().to_owned()];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(&directory).expect("the directory reads") {
            let entry = entry.expect("the directory entry reads");
            if entry.file_type().unwrap().is_dir() {
                directories.push(entry.path());
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("class") {
                continue;
            }
            let name = path
                .strip_prefix(temp.path())
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let bytes = std::fs::read(&path).expect("the class file reads");
            owned.push((name, bytes));
        }
    }
    let entries = owned
        .iter()
        .map(|(name, bytes)| (name.as_bytes() as &[u8], bytes.as_slice()))
        .collect::<Vec<_>>();
    (jar_of(&entries), owned)
}

/// The class-source report of one named class of one jar, under the patrol's evidence
/// selection.
fn source_of(jar: &[u8], class: &str) -> jarde::class_source::ClassSourceReport {
    source_under(jar, class, EnvironmentPolicy::PlainJar)
}

/// The same report for one standalone `.class` artifact.
fn source_of_class_file(bytes: &[u8], class: &str) -> jarde::class_source::ClassSourceReport {
    source_under(bytes, class, EnvironmentPolicy::SingleClass)
}

fn source_under(
    artifact: &[u8],
    class: &str,
    policy: EnvironmentPolicy,
) -> jarde::class_source::ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(jarde::facade::task_limits(&[]).expect("bounded task limits"));
    let snapshot = engine
        .open(ArtifactInput::bytes(artifact.to_vec()), &mut budget)
        .expect("the artifact opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    let evidence = RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap);
    match engine.class_source_with_evidence(
        std::slice::from_ref(&snapshot),
        &request,
        &evidence,
        &mut budget,
    ) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("one class answers one class-source request: {other:?}"),
    }
}

/// Runs one class of one jar with extra JVM arguments, answering `(stdout, exited normally)`.
/// An `AssertionError` under `-ea` is a failure the caller compares by its exit, never by the
/// stack text it prints (line numbers differ between the original and any recompiled text).
fn run_class(label: &str, jar: &[u8], class: &str, extra: &[&str]) -> (String, bool) {
    let temp = TestDirectory::new(label);
    let jar_path = temp.path().join("family.jar");
    std::fs::write(&jar_path, jar).expect("the jar is written");
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&jar_path)
        .args(extra)
        .arg(class)
        .current_dir(temp.path())
        .output()
        .expect("java runs");
    (
        String::from_utf8(run.stdout).expect("the run prints text"),
        run.status.success(),
    )
}

/// Compiles one recovered text as `--release 8` Java beside the original jar, verifies it under
/// `-Xverify:all`, and runs it with extra JVM arguments.
fn recompile_and_run(
    label: &str,
    text: &str,
    class_name: &str,
    jar: &[u8],
    extra: &[&str],
) -> (String, bool) {
    let temp = TestDirectory::new(label);
    std::fs::write(temp.path().join(format!("{class_name}.java")), text)
        .expect("the recovered text is written");
    let compiled = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg(format!("{class_name}.java"))
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "the recovered {} recompiles under --release 8:\n{}",
        class_name,
        String::from_utf8_lossy(&compiled.stderr)
    );
    let jar_path = temp.path().join("family.jar");
    std::fs::write(&jar_path, jar).expect("the original jar is written");
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp", &format!("{}:.", jar_path.display())])
        .args(extra)
        .arg(class_name)
        .current_dir(temp.path())
        .output()
        .expect("java runs");
    (
        String::from_utf8(run.stdout).expect("the run prints text"),
        run.status.success(),
    )
}

/// The patrol's A1 fixture source, verbatim.
const A1_SOURCE: &str = r#"public class A1 {
    public static int check(int x) {
        assert x > 0 : "positive: " + x;
        return x * 2;
    }
    public static int plainAssert(int x) {
        assert x != 0;
        return 100 / x;
    }
    static class Sub {
        int m(int v) {
            assert v > 1 : v;
            return v - 1;
        }
    }
    public static void main(String[] a) {
        System.out.println(check(5));
        System.out.println(plainAssert(4));
        System.out.println(new Sub().m(3));
        System.out.println(check(-1));
    }
}
"#;

/// The 2026-09-23 audit's `AssertProbe`: condition and detail each bump a counter, so the two
/// assertion states and the failure path's evaluation order are all observable.
const PROBE_SOURCE: &str = r#"public class AssertProbe {
    private static int guardCalls;
    private static int detailCalls;
    private static boolean guard(int value) {
        guardCalls++;
        return value > 0;
    }
    private static String detail(int value) {
        detailCalls++;
        return "bad:" + value;
    }
    static String check(int value) {
        assert guard(value) : detail(value);
        return guardCalls + "|" + detailCalls;
    }
    public static void main(String[] args) {
        System.out.print(check(1) + ";");
        try {
            System.out.print(check(-1));
        } catch (AssertionError error) {
            System.out.print(error.getMessage() + "|" + guardCalls + "|" + detailCalls);
        }
        System.out.println();
    }
}
"#;

/// A class whose guards do not all fold: `disj`'s `assert a || b` lowers to two nested tests,
/// `negated`'s message is a field read the build could not present, and `both` does not recover
/// at all — the all-or-nothing census keeps every member physical.
const MIXED_SOURCE: &str = r#"public class MixedShapes {
    static void disj(boolean a, boolean b) {
        assert a || b;
    }
    static void local(boolean f) {
        assert f;
    }
    static void call(int v) {
        assert check(v) : "call:" + v;
    }
    static boolean check(int v) { return v > 0; }
    public static void main(String[] a) {
        disj(true, false);
        local(true);
        call(1);
        System.out.println("mixed");
    }
}
"#;

/// One class without asserts, for the zero-change anchor.
const PLAIN_SOURCE: &str = "public class PlainNoAssert {\n    public static void main(String[] a) {\n        System.out.println(\"plain\");\n    }\n}\n";

/// The hand-patched negative fixtures: `AssertProbe` with one extra statement inside the guard
/// (`iinc 0, 0` — a no-op `value += 0` before the inner test), and the audit's wrong-owner
/// switch (`StringBuilder.class.desiredAssertionStatus()` in `<clinit>`).
const EXTRA_GUARD_CLASS: &[u8] =
    include_bytes!("fixtures/assert-statement-sugar/AssertProbeExtraGuard.class");
const WRONG_OWNER_CLASS: &[u8] =
    include_bytes!("fixtures/assert-statement-sugar/AssertProbeWrongOwner.class");

#[test]
fn a1_family_presents_assert_statements_and_hides_every_synthetic_layer() {
    let (jar, _) = compile_family("a1", A1_SOURCE);
    let report = source_of(&jar, "A1");
    // Both message forms of the root, and the nested class's own independent switch.
    for statement in [
        "assert arg0 > 0 : \"positive: \" + arg0;",
        "assert arg0 != 0;",
        "assert arg1 > 1 : arg1;",
    ] {
        assert!(report.text.contains(statement), "{}", report.text);
    }
    // The three synthetic layers are gone from the whole family: the field declarations, the
    // initialization lines (both `<clinit>` members go — nothing else remained in either), and
    // the guards.
    assert!(
        !report.text.contains("$assertionsDisabled"),
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("desiredAssertionStatus"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("static {"), "{}", report.text);
    // The root's own switch field is the one hidden field, and the projection publishes its
    // physical anchor beside the header the declaration would have followed.
    assert_eq!(report.projection_inputs.hidden_fields, vec![0],);
    assert!(
        report.projection_inputs.omitted_methods.contains(
            &report
                .methods
                .iter()
                .find(|method| method.item.identity.name.0 == b"<clinit>")
                .expect("the root holds its `<clinit>` record")
                .item
                .index
        )
    );
    // The nested child's field and `<clinit>` are hidden through the fold's own channels.
    assert!(report.text.contains("static class Sub"), "{}", report.text);
}

#[test]
fn a1_family_recompiles_and_prints_the_original_output_in_both_assertion_states() {
    let (jar, _) = compile_family("a1-run", A1_SOURCE);
    let original_default = run_class("a1-orig-default", &jar, "A1", &[]);
    let original_ea = run_class("a1-orig-ea", &jar, "A1", &["-ea"]);
    assert_eq!(original_default.0, "10\n25\n2\n-2\n");
    assert!(original_default.1);
    assert_eq!(original_ea.0, "10\n25\n2\n");
    assert!(!original_ea.1, "the original throws under -ea");
    let report = source_of(&jar, "A1");
    let text = report
        .text
        .lines()
        .filter(|line| !line.starts_with("// jarde:"))
        .collect::<Vec<_>>()
        .join("\n");
    let recovered_default = recompile_and_run("a1-rec-default", &text, "A1", &jar, &[]);
    let recovered_ea = recompile_and_run("a1-rec-ea", &text, "A1", &jar, &["-ea"]);
    assert_eq!(recovered_default, original_default);
    assert_eq!(recovered_ea, original_ea);
}

#[test]
fn message_side_effects_keep_their_evaluation_order() {
    let (jar, _) = compile_family("probe", PROBE_SOURCE);
    let original_default = run_class("probe-orig-default", &jar, "AssertProbe", &[]);
    let original_ea = run_class("probe-orig-ea", &jar, "AssertProbe", &["-ea"]);
    assert_eq!(original_default.0, "0|0;0|0\n");
    assert_eq!(original_ea.0, "1|0;bad:-1|2|1\n");
    let report = source_of(&jar, "AssertProbe");
    assert!(
        report.text.contains("assert guard(arg0) : detail(arg0);"),
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("$assertionsDisabled"),
        "{}",
        report.text
    );
    let text = report
        .text
        .lines()
        .filter(|line| !line.starts_with("// jarde:"))
        .collect::<Vec<_>>()
        .join("\n");
    let recovered_default = recompile_and_run("probe-rec-default", &text, "AssertProbe", &jar, &[]);
    let recovered_ea = recompile_and_run("probe-rec-ea", &text, "AssertProbe", &jar, &["-ea"]);
    assert_eq!(recovered_default, original_default);
    assert_eq!(recovered_ea, original_ea);
}

#[test]
fn guards_that_do_not_all_fold_keep_the_whole_class_physical() {
    let (jar, _) = compile_family("mixed", MIXED_SOURCE);
    let report = source_of(&jar, "MixedShapes");
    // `local` and `call` alone would fold; `disj`'s nested lowering and the un-recovered
    // conjunctive member keep the class on the physical presentation, all-or-nothing.
    assert!(
        !report.text.contains("\n        assert "),
        "no member folds alone:\n{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("if (!MixedShapes.$assertionsDisabled) {"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains(
            "$assertionsDisabled = (!MixedShapes.class.desiredAssertionStatus() ? 1 : 0) % 2 != 0;"
        ),
        "{}",
        report.text
    );
    assert!(report.projection_inputs.hidden_fields.is_empty());
    assert!(report.projection_inputs.omitted_methods.is_empty());
}

#[test]
fn extra_statement_inside_the_guard_keeps_the_physical_presentation() {
    let report = source_of_class_file(EXTRA_GUARD_CLASS, "AssertProbe");
    // The guard holds a real statement before the inner test, which javac never lowers from an
    // `assert` — the whole pattern stays physical.
    assert!(
        report.text.contains("value = value + 0;"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("if (!AssertProbe.$assertionsDisabled) {"),
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("\n        assert "),
        "{}",
        report.text
    );
    assert!(
        report.text.contains(
            "$assertionsDisabled = (!AssertProbe.class.desiredAssertionStatus() ? 1 : 0) % 2 != 0;"
        ),
        "{}",
        report.text
    );
}

#[test]
fn wrong_owner_switch_keeps_the_physical_presentation() {
    let report = source_of_class_file(WRONG_OWNER_CLASS, "AssertProbe");
    // The guard is perfectly shaped, but the `<clinit>` line reads another class's status —
    // folding would move the switch, so the proof refuses the whole class.
    assert!(
        report
            .text
            .contains("(!java.lang.StringBuilder.class.desiredAssertionStatus() ? 1 : 0)"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("if (!AssertProbe.$assertionsDisabled) {"),
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("\n        assert "),
        "{}",
        report.text
    );
}

#[test]
fn class_without_asserts_stages_no_projection() {
    let (jar, _) = compile_family("plain", PLAIN_SOURCE);
    let report = source_of(&jar, "PlainNoAssert");
    assert!(!report.text.contains("assert "), "{}", report.text);
    assert!(report.projection_inputs.hidden_fields.is_empty());
    assert!(report.projection_inputs.omitted_methods.is_empty());
    assert!(report.projection_inputs.member_texts.is_empty());
}
