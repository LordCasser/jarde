//! Acceptance tests of change `recover-unicode-identifiers`: the identifier judgment
//! (`jarde_java::is_java_identifier`, JLS §3.8) is Java's own table and not ASCII's alone, so a
//! class file that declares a Chinese field, method or local spells that name in its **declaration**
//! exactly as its own statements already spelled it — one name, one spelling, one source.
//!
//! The fixtures under `tests/fixtures/recover-unicode-identifiers/` are compiled by **real javac 8**
//! (Corretto 1.8.0_432, `javac -encoding UTF-8`, plus `-g` for the local names the `Loc` fixture
//! pins); the second leg compiles the same frozen sources in-test with the toolchain
//! `javac --release 8 -encoding UTF-8`.
//!
//! Frozen behaviors:
//! 1. the anchor (`UT`, the unicode-identifier patrol's fixture) renders `变量`/`描述`/`方法`/
//!    `内部类`/`名字` in its declarations, with no `is not a Java identifier` marker and no `__`
//!    stand-in, on **both** legs; the whole class, stripped of the presentation comments, compiles
//!    and prints `变量=1/42/中文` under `-Xverify:all` — the exact line the frozen class prints;
//! 2. `Loc` (a Chinese local name out of the `LocalVariableTable`) is spelled as declared *and* as
//!    used, which is the naming table's half of the same predicate;
//! 3. the byte-patched `Loc.punct` — the local's name `数量` replaced in place by `名。`, same byte
//!    length, README recipe — still aliases, and declaration and uses share the alias: the widening
//!    is Java's *letters and digits*, not "every non-ASCII character";
//! 4. the ASCII presentations are byte-for-byte the pre-change ones, including the class whose own
//!    name is no identifier at all (`com/example/package-info`, whose hyphen the class-declaration
//!    line spells as the pool states it — a boundary this change neither widens nor closes).

use jarde::class_source::ClassSourceReport;
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
            "jarde-unicode-identifier-{label}-{}-{nonce}-{}",
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

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/recover-unicode-identifiers")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("fixture {name} reads: {error}"))
}

/// The frozen class files real javac 8 wrote for the anchor source.
fn ut_jar() -> Vec<u8> {
    jar_of(&[
        (b"UT.class", fixture("ut/UT.class").as_slice()),
        (
            "UT$内部类.class".as_bytes(),
            fixture("ut/UT$内部类.class").as_slice(),
        ),
    ])
}

fn loc_jar() -> Vec<u8> {
    jar_of(&[(b"Loc.class", fixture("loc/Loc.class").as_slice())])
}

/// The same class, with the local's `LocalVariableTable` name `数量` replaced in place by `名。`
/// (both six bytes of UTF-8): a name whose characters are a Java letter and Java punctuation.
fn loc_punct_jar() -> Vec<u8> {
    jar_of(&[(b"Loc.class", fixture("loc/Loc.punct.class").as_slice())])
}

fn task_limits() -> Limits {
    jarde::facade::task_limits(&[]).expect("the task defaults are bounded")
}

fn source_of(jar: &[u8], class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(task_limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.to_vec()), &mut budget)
        .expect("the fixture jar opens");
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

/// The same frozen source through the toolchain leg, `javac --release 8 -encoding UTF-8`, as the jar
/// the recovery reads.
fn compile_with_path_javac(label: &str, source: &str) -> Vec<u8> {
    let temp = TestDirectory::new(label);
    let file = source
        .split_once("class ")
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .map(|name| format!("{name}.java"))
        .expect("the source states a class");
    std::fs::write(temp.path().join(&file), source).expect("the source is written");
    let compiled = Command::new("javac")
        .args(["--release", "8", "-encoding", "UTF-8"])
        .arg(&file)
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let classes = classes_of(temp.path());
    let borrowed: Vec<(&[u8], &[u8])> = classes
        .iter()
        .map(|(name, bytes)| (name.as_bytes(), bytes.as_slice()))
        .collect();
    jar_of(&borrowed)
}

/// Every class file below one directory, as `(entry name, bytes)` pairs.
fn classes_of(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut entries = Vec::new();
    let mut directories = vec![root.to_owned()];
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
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            entries.push((name, std::fs::read(&path).expect("the class reads")));
        }
    }
    entries.sort();
    entries
}

/// The class-source text as the compile-and-run protocol reads it: every `//` comment line dropped,
/// which is the strip the patrol's own manual fix was made by.
fn stripped(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| format!("{line}\n"))
        .collect()
}

/// Compiles one presented class with `javac --release 8 -encoding UTF-8` and runs it under
/// `java -Xverify:all`, returning the run's exact stdout. The compilation is asserted, so a text
/// that is not Java fails here rather than silently comparing outputs.
fn compile_and_run(tag: &str, class: &str, text: &str) -> String {
    let temp = TestDirectory::new(tag);
    let source = temp.path().join(format!("{class}.java"));
    std::fs::write(&source, text).expect("the presented source writes");
    let compile = Command::new("javac")
        .args(["--release", "8", "-encoding", "UTF-8", "-d"])
        .arg(temp.path())
        .arg(&source)
        .output()
        .expect("the installed JDK provides javac");
    assert!(
        compile.status.success(),
        "{class} must compile as presented: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    run_under_verify(temp.path(), class)
}

/// Runs one class from one directory under `-Xverify:all`, returning its exact stdout.
fn run_under_verify(directory: &Path, class: &str) -> String {
    let run = Command::new("java")
        .args(["-Xverify:all", "-Dfile.encoding=UTF-8", "-cp"])
        .arg(directory)
        .arg(class)
        .output()
        .expect("the installed JDK provides java");
    assert!(
        run.status.success(),
        "{class} must run: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run prints UTF-8")
}

/// The same class out of the frozen jar, run under the same JVM flags: the behavior the presented
/// text has to reproduce.
fn run_jar(tag: &str, class: &str, jar: &[u8]) -> String {
    let temp = TestDirectory::new(tag);
    let path = temp.path().join("fixture.jar");
    std::fs::write(&path, jar).expect("the fixture jar writes");
    let run = Command::new("java")
        .args(["-Xverify:all", "-Dfile.encoding=UTF-8", "-cp"])
        .arg(&path)
        .arg(class)
        .output()
        .expect("the installed JDK provides java");
    assert!(
        run.status.success(),
        "{class} must run from its own class file: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run prints UTF-8")
}

const UT_SOURCE: &str = include_str!("fixtures/recover-unicode-identifiers/ut/UT.java");

/// The anchor's declarations, on both javac legs: the names the class file's own pool states, in the
/// positions a declaration writes them, with the reference spellings of the same run beside them.
#[test]
fn the_cjk_declarations_are_presented_as_the_pool_states_them() {
    for (label, jar) in [
        ("real-javac8", ut_jar()),
        ("toolchain", compile_with_path_javac("ut23", UT_SOURCE)),
    ] {
        let report = source_of(&jar, "UT");
        for expected in [
            "static int 变量 = 1;",
            "static java.lang.String 描述 = new java.lang.StringBuilder().append(\"变量=\").append(UT.变量).toString();",
            "static int 方法(int arg0)",
            "static class 内部类 extends java.lang.Object",
            "java.lang.String 名字;",
            "UT.描述",
            "方法(21)",
            "local1.名字",
        ] {
            assert!(
                report.text.contains(expected),
                "{label}: the text must state `{expected}`:\n{}",
                report.text
            );
        }
        assert!(
            !report.text.contains("is not a Java identifier"),
            "{label}: no name is refused as an identifier any more:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("__"),
            "{label}: no declaration is spelled with the alias stand-in:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("not recovered: the recovery run"),
            "{label}: every member of the family recovers:\n{}",
            report.text
        );
    }
}

/// The anchor's whole text, stripped of the comments the compile-and-run protocol drops, is Java
/// that both javac legs accept and that prints exactly what the frozen class prints.
#[test]
fn the_anchor_round_trips_through_both_javac_legs() {
    for (label, jar) in [
        ("real-javac8", ut_jar()),
        ("toolchain", compile_with_path_javac("utrt", UT_SOURCE)),
    ] {
        let report = source_of(&jar, "UT");
        let printed = compile_and_run(&format!("{label}-run"), "UT", &stripped(&report.text));
        assert_eq!(
            printed, "变量=1/42/中文\n",
            "{label}: the recovered behavior"
        );
        assert_eq!(
            printed,
            run_jar(&format!("{label}-jar"), "UT", &jar),
            "{label}: the recovered text prints what its own class file prints"
        );
    }
}

/// The naming table's half of the same predicate, on the local names of the `LocalVariableTable`:
/// a name of Java letters is spelled as declared and as used, and one that carries punctuation
/// keeps the alias — the same alias in both positions, so the two spellings cannot disagree.
#[test]
fn a_letter_name_is_spelled_and_a_punctuation_name_keeps_its_alias() {
    let letters = source_of(&loc_jar(), "Loc");
    for expected in [
        "static int 名字(int arg0)",
        "int 数量 = arg0 + 1;",
        "return 数量;",
    ] {
        assert!(
            letters.text.contains(expected),
            "the letter name must be spelled as the table states it: `{expected}`:\n{}",
            letters.text
        );
    }

    let punctuation = source_of(&loc_punct_jar(), "Loc");
    assert!(
        punctuation.text.contains("static int 名字(int arg0)"),
        "the method name of the same class is untouched by the patch:\n{}",
        punctuation.text
    );
    assert!(
        punctuation.text.contains("int __ = arg0 + 1;") && punctuation.text.contains("return __;"),
        "the punctuation name keeps one alias in its declaration and its uses:\n{}",
        punctuation.text
    );
    assert!(
        !punctuation.text.contains('。'),
        "a character outside the identifier table never takes an identifier position:\n{}",
        punctuation.text
    );
}

/// The ASCII presentations, byte-for-byte the ones the pre-change layer wrote — the control the
/// widening is bounded by (`README.md` records which binary rendered each frozen text, and that the
/// standalone class bytes go in through the same single-entry jar shape the per-class CLI sweep
/// reads them with).
#[test]
fn the_ascii_presentations_are_byte_for_byte_the_pre_change_ones() {
    let f1: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/fixture/F1.class"
    );
    let sb: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-10-05/charsequence-arg-widening-patrol/fixture/SB.class"
    );
    let rg: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-10-05/recursive-generic-patrol/fixture/RG.class"
    );
    let package_info: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-10-05/package-info-patrol/fixture/pi.jar"
    );
    for (label, bytes, class, frozen) in [
        (
            "F1",
            jar_of(&[(b"F1.class", f1)]),
            "F1",
            include_str!("fixtures/recover-unicode-identifiers/ascii/F1.jarde.java"),
        ),
        (
            "SB",
            jar_of(&[(b"SB.class", sb)]),
            "SB",
            include_str!("fixtures/recover-unicode-identifiers/ascii/SB.jarde.java"),
        ),
        (
            "RG",
            jar_of(&[(b"RG.class", rg)]),
            "RG",
            include_str!("fixtures/recover-unicode-identifiers/ascii/RG.jarde.java"),
        ),
        (
            "package-info",
            package_info.to_vec(),
            "com/example/package-info",
            include_str!("fixtures/recover-unicode-identifiers/ascii/package-info.jarde.java"),
        ),
    ] {
        let report = source_of(&bytes, class);
        assert_eq!(
            report.text, frozen,
            "{label}: an ASCII presentation must not move by one byte"
        );
    }
    // The boundary this change states and does not close: the hyphenated name is no identifier, and
    // the class-declaration line spells it as the pool states it (the projection gate refuses a
    // *generic header* on such a name, never the declaration line itself). The frozen text above is
    // what pins it; this assertion says out loud which line that is.
    let package_info_report = source_of(package_info, "com/example/package-info");
    assert!(
        package_info_report
            .text
            .contains("interface package-info {"),
        "the package-info boundary stays where it was:\n{}",
        package_info_report.text
    );
}
