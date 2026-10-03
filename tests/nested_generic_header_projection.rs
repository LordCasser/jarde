//! Acceptance tests of change `recover-nested-generic-class-headers`: a `$`-nested class
//! whose own `InnerClasses` self row proves the member position (the fold's row criteria)
//! projects its class `Signature` type-parameter header in the separated unit, its members
//! receive the class type-variable scope, and the member fold carries the proved header into
//! its nested declaration — while a row that does not join keeps the refusal chain exactly.
//!
//! The three-way anchors:
//!
//! 1. every recompiled presentation runs under `-Xverify:all` and prints exactly what the
//!    original family prints;
//! 2. a join-failure row, and a family whose owner declares a local of the child type, keep
//!    the presentations they had before this change (the chain head for the former, the fold
//!    token boundary — a conservative position this change does not touch — for the latter);
//! 3. top-level generic headers and the existing fold shapes are untouched.

use jarde::class_source::{ClassSourceMemberFamily, ClassSourceMemberProjection};
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
            "jarde-nested-generic-headers-{label}-{}-{nonce}-{}",
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

/// One family compiled the way the patrol's fixtures were: `--release 8` with `-g:none`, one
/// jar entry per produced class file, in the writer's own order.
fn compile_family(label: &str, source: &str) -> (Vec<u8>, Vec<(String, Vec<u8>)>) {
    let temp = TestDirectory::new(label);
    let file = source
        .split_once("class ")
        .and_then(|(_, rest)| {
            let name: String = rest
                .chars()
                .take_while(|character| {
                    character.is_alphanumeric() || *character == '_' || *character == '$'
                })
                .collect();
            (!name.is_empty()).then_some(name)
        })
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
        .map(|(name, bytes)| (name.as_bytes(), bytes.as_slice()))
        .collect::<Vec<_>>();
    (jar_of(&entries), owned)
}

fn source_of(jar: &[u8], class: &str) -> ClassSourceReport {
    source_with(
        jar,
        class,
        jarde::facade::task_limits(&[]).expect("bounded"),
    )
}

fn source_with(jar: &[u8], class: &str, limits: Limits) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits);
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.to_vec()), &mut budget)
        .expect("the family jar opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
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
        other => panic!("one family answers one class-source request: {other:?}"),
    }
}

/// Compiles one recovered text as `--release 8` Java, runs it under `-Xverify:all`, and returns
/// what the run printed. The classpath holds exactly the original compiled family.
fn recompile_and_run(label: &str, text: &str, class_name: &str, jar: &[u8]) -> String {
    let temp = TestDirectory::new(label);
    std::fs::write(temp.path().join(format!("{class_name}.java")), text)
        .expect("the recovered text is written");
    let deps = temp.path().join("deps");
    std::fs::create_dir_all(&deps).expect("the dependency directory is created");
    std::fs::write(deps.join("family.jar"), jar).expect("the family jar is written");
    let compiled = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-cp")
        .arg(deps.join("family.jar"))
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
    let run = Command::new("java")
        .args([
            "-Xverify:all",
            "-cp",
            &format!("{}:.", deps.join("family.jar").display()),
        ])
        .arg(class_name)
        .current_dir(temp.path())
        .output()
        .expect("java runs");
    assert!(
        run.status.success(),
        "the recovered {} runs under -Xverify:all:\n{}",
        class_name,
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run prints text")
}

fn child_bytes(family: &[(String, Vec<u8>)], name: &str) -> Vec<u8> {
    family
        .iter()
        .find(|(entry, _)| entry == name)
        .expect("the child class file exists")
        .1
        .clone()
}

/// One same-length byte patch of the compiled child, in the family's own byte order.
fn changed(bytes: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    let mut result = bytes.to_vec();
    let count = result
        .windows(needle.len())
        .filter(|window| *window == needle)
        .count();
    assert_eq!(count, 1, "the needle occurs exactly once");
    let index = result
        .windows(needle.len())
        .position(|window| window == needle)
        .expect("the needle occurs");
    result.splice(index..index + needle.len(), replacement.iter().copied());
    result
}

const STATIC_BOX: &str = "public class NG4 {\n    static class Box<U> { U value; }\n    public static void main(String[] a) {\n        Box<String> box = new Box<String>();\n        box.value = \"y\";\n        System.out.println(box.value);\n    }\n}\n";

/// Compiles one recovered child-unit text as `--release 8` Java — beside its own family jar
/// for the units that name their owner — and reads the class type parameter and the field's
/// generic type back under `-Xverify:all`: the header the unit states is the reflection the
/// compiled class answers with.
fn compile_and_reflect_type_parameter(
    label: &str,
    text: &str,
    binary_name: &str,
    field_name: &str,
    jar: &[u8],
) -> String {
    let temp = TestDirectory::new(label);
    std::fs::write(temp.path().join("Unit.java"), text).expect("the unit text is written");
    std::fs::write(temp.path().join("family.jar"), jar).expect("the family jar is written");
    std::fs::write(
        temp.path().join("Runner.java"),
        format!(
            "public class Runner {{ public static void main(String[] a) throws Exception {{\n\
             System.out.println(Class.forName(\"{binary_name}\").getTypeParameters()[0].getName());\n\
             System.out.println(Class.forName(\"{binary_name}\").getDeclaredField(\"{field_name}\").getGenericType());\n\
             }} }}\n"
        ),
    )
    .expect("the runner is written");
    for file in ["Unit.java", "Runner.java"] {
        let compiled = Command::new("javac")
            .args(["--release", "8", "-cp", "family.jar"])
            .arg(file)
            .current_dir(temp.path())
            .output()
            .expect("javac runs");
        assert!(
            compiled.status.success(),
            "{label}: {file} recompiles under --release 8:\n{}\n{}",
            String::from_utf8_lossy(&compiled.stderr),
            text
        );
    }
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp", ".:family.jar"])
        .arg("Runner")
        .current_dir(temp.path())
        .output()
        .expect("java runs");
    assert!(
        run.status.success(),
        "{label}: the runner verifies:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the runner prints text")
}

#[test]
fn separated_nested_generic_units_project_the_header_and_member_scope() {
    for (label, source, child_name, header, field, field_name, reflected) in [
        (
            "static-single",
            STATIC_BOX,
            "NG4$Box",
            "class NG4$Box<U> extends java.lang.Object {",
            "U value;",
            "value",
            "U\nU\n",
        ),
        (
            "static-two-parameters",
            "public class NG2 {\n    static class Pair<U, V> { U first; V second; }\n    public static void main(String[] a) {\n        Pair<String, Integer> pair = new Pair<String, Integer>();\n        pair.first = \"a\";\n        pair.second = 1;\n        System.out.println(pair.first + \" \" + pair.second);\n    }\n}\n",
            "NG2$Pair",
            "class NG2$Pair<U, V> extends java.lang.Object {",
            "U first;",
            "first",
            "U\nU\n",
        ),
        (
            "static-bound",
            "public class NG3 {\n    static class Num<N extends Number> { N n; N get() { return n; } }\n    public static void main(String[] a) {\n        Num<java.lang.Integer> num = new Num<java.lang.Integer>();\n        num.n = 7;\n        System.out.println(num.get());\n    }\n}\n",
            "NG3$Num",
            "class NG3$Num<N extends java.lang.Number> extends java.lang.Object {",
            // The same-class read `get` performs on `n` is an areturn consumer: the value keeps
            // accepting the parameterized type, so `recover-same-class-generic-bindings` proves
            // the binding and the member scope reaches the field too.
            "N n;",
            "n",
            "N\nN\n",
        ),
        (
            "non-static",
            "public class NG1 {\n    class Inner<U> { U value; U get() { return value; } }\n    public static void main(String[] a) {\n        Inner<String> inner = new NG1().new Inner<String>();\n        inner.value = \"x\";\n        System.out.println(inner.get());\n    }\n}\n",
            "NG1$Inner",
            "class NG1$Inner<U> extends java.lang.Object {",
            "U value;",
            "value",
            "U\nU\n",
        ),
    ] {
        let (jar, _) = compile_family(label, source);
        let report = source_of(&jar, child_name);
        assert!(
            report.text.contains(header),
            "{label}: the nested header projects:\n{}",
            report.text
        );
        assert!(
            report.text.contains(field),
            "{label}: the member scope is established:\n{}",
            report.text
        );
        if label == "static-two-parameters" {
            assert!(
                report.text.contains("V second;"),
                "{label}: the second parameter projects:\n{}",
                report.text
            );
        }
        assert!(
            report
                .text
                .contains("projected after physical parent erasure and nested member proof"),
            "{label}: the marker states the nested proof:\n{}",
            report.text
        );
        let declaration = report.declaration.as_ref().expect("the unit is declared");
        assert_eq!(
            declaration.generic_refusal, None,
            "{label}: the header refusal is gone"
        );
        // The compiled unit answers reflection with the projected header and member type.
        assert_eq!(
            compile_and_reflect_type_parameter(label, &report.text, child_name, field_name, &jar),
            reflected,
            "{label}: the unit's reflection"
        );
    }
}

#[test]
fn folded_family_carries_the_nested_generic_header_and_scope() {
    let source = "public class NG5 {\n    static class Box<U> { U value; }\n    static Box<String> box;\n    public static void main(String[] a) {\n        box = new Box<String>();\n        box.value = \"z\";\n        System.out.println(box.value);\n    }\n}\n";
    let (jar, _) = compile_family("ng5", source);
    let report = source_of(&jar, "NG5");
    assert!(
        report
            .text
            .contains("static class Box<U> extends java.lang.Object {"),
        "the fold carries the type-parameter header:\n{}",
        report.text
    );
    assert!(
        report.text.contains("U value;"),
        "the fold carries the projected field:\n{}",
        report.text
    );
    assert!(
        report.text.contains("Box() {"),
        "the constructor takes the member's source name:\n{}",
        report.text
    );
    assert!(matches!(
        &report.member_family,
        ClassSourceMemberFamily::PreparedStatic {
            members,
            projection: ClassSourceMemberProjection::Projected { .. },
        } if members.len() == 1
    ));
    assert_eq!(recompile_and_run("ng5", &report.text, "NG5", &jar), "z\n");
}

#[test]
fn join_failure_row_keeps_the_refusal_chain() {
    let (jar, family) = compile_family("ng4", STATIC_BOX);
    let original = child_bytes(&family, "NG4$Box.class");
    // The self row's `inner_name` Utf8 no longer joins: `NG4$B0x` is not this class's name.
    let patched = changed(&original, b"\x00\x03Box", b"\x00\x03B0x");
    let entries: Vec<(&[u8], &[u8])> = family
        .iter()
        .map(|(name, bytes)| {
            (
                name.as_bytes(),
                if name == "NG4$Box.class" {
                    patched.as_slice()
                } else {
                    bytes.as_slice()
                },
            )
        })
        .collect();
    let negative = jar_of(&entries);
    let report = source_of(&negative, "NG4$Box");
    // The chain head keeps its exact code and message, and the member scope stays unproved.
    assert!(
        report.text.contains(
            "// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position"
        ),
        "{}",
        report.text
    );
    assert!(
        report.text.contains(
            "// jarde: field Signature projection refused for `valueLjava/lang/Object;`: unsupported (jvm_signature_scope_unproved): type variable `U` is not declared in the available Signature scope"
        ),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("class NG4$Box extends java.lang.Object {"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("java.lang.Object value;"),
        "{}",
        report.text
    );
    // The same family's owner does not fold the unprojected child either.
    let root = source_of(&negative, "NG4");
    assert!(
        !root.text.contains("static class Box"),
        "the fold does not carry an unproved header:\n{}",
        root.text
    );
    // The untouched jar is the control: the same bytes without the patch do project.
    let control = source_of(&jar, "NG4$Box");
    assert!(
        control
            .text
            .contains("class NG4$Box<U> extends java.lang.Object {"),
        "{}",
        control.text
    );
}

#[test]
fn owner_local_of_the_child_type_keeps_the_fold_open_and_the_unit_projected() {
    // The fold slice's own token anchor does not prove a *local declaration* of a folded
    // member type (a pre-existing conservative position, generics aside — a non-generic
    // `Solo` local refuses the same way). The header this change projects must not change
    // that boundary — the separated presentations carry it.
    let (jar, _) = compile_family("ng4-fold", STATIC_BOX);
    let report = source_of(&jar, "NG4");
    assert!(
        report.text.contains("NG4$Box local1 = new NG4$Box();"),
        "the owner text keeps the physical spelling:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("static class Box"),
        "the fold stays refused at its own token boundary:\n{}",
        report.text
    );
    let child = source_of(&jar, "NG4$Box");
    assert!(
        child
            .text
            .contains("class NG4$Box<U> extends java.lang.Object {"),
        "{}",
        child.text
    );
    assert!(child.text.contains("U value;"), "{}", child.text);
}

#[test]
fn budget_stop_keeps_the_physical_nested_header() {
    let (jar, _) = compile_family("ng4-budget", STATIC_BOX);
    let full = source_of(&jar, "NG4$Box");
    assert!(
        full.text
            .contains("class NG4$Box<U> extends java.lang.Object {"),
        "{}",
        full.text
    );
    let mut limited = jarde::facade::task_limits(&[]).expect("bounded");
    limited.analysis_steps = 8;
    let stopped = source_with(&jar, "NG4$Box", limited);
    assert!(
        !matches!(stopped.execution, ExecutionReport::Complete { .. }),
        "the tight budget stops the request"
    );
    assert!(
        stopped
            .text
            .contains("class NG4$Box extends java.lang.Object {"),
        "a stopped run keeps the physical header:\n{}",
        stopped.text
    );
    assert!(
        !stopped.text.contains("class NG4$Box<U>"),
        "{}",
        stopped.text
    );
}

#[test]
fn top_level_generic_header_and_plain_fold_shapes_are_unchanged() {
    // The top-level projection and the non-generic fold keep their exact spellings.
    let (jar, _) = compile_family(
        "plain",
        "public class M2 {\n    static class Solo { int v() { return 7; } }\n    public static void main(String[] a) { System.out.println(new Solo().v()); }\n}\n",
    );
    let report = source_of(&jar, "M2");
    assert!(
        report
            .text
            .contains("static class Solo extends java.lang.Object {"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("Solo<"), "{}", report.text);
    let top = compile_family(
        "top",
        "public class Top<T extends java.lang.Comparable<T>> {\n    T pick(T input) { return input; }\n    public static void main(String[] a) { System.out.println(\"ok\"); }\n}\n",
    );
    let top_report = source_of(&top.0, "Top");
    assert!(
        top_report.text.contains(
            "public class Top<T extends java.lang.Comparable<T>> extends java.lang.Object"
        ),
        "{}",
        top_report.text
    );
    assert!(
        top_report
            .text
            .contains("projected after physical parent erasure proof"),
        "{}",
        top_report.text
    );
}
