//! Acceptance tests of change `recover-javac8-allocation-qualifier-null-check`: the discarded
//! null check real javac 8 spells over a **freshly allocated enclosing instance** — the
//! contiguous tail `dup; <discarded null check>; pop` immediately after the nested constructor
//! call of `new Outer().new Inner(…)` — is owned by the construction that builds the instance
//! (the same predicate `jarde_java::facts::is_discarded_null_check` the parameter-qualifier
//! rule projects), so the allocation-qualifier form recovers on the real javac 8 leg exactly as
//! it already did on the javac 9+ leg that writes no check.
//!
//! The fixtures under `tests/fixtures/recover-javac8-allocation-qualifier-null-check/` are
//! compiled by **real javac 8** (Corretto 1.8.0_432, no `--release` flag); the dual legs
//! compile in-test with the toolchain `javac --release 8`.
//!
//! Frozen behaviors:
//! 1. the main anchor (`N1`, allocation qualifier **with** an int argument) folds on the real
//!    javac 8 leg, together with the parameter-qualifier `use` method of the same family;
//! 2. the control positive (`Pod`, allocation qualifier with **no** argument, different
//!    identifiers) folds on both legs;
//! 3. a construction whose instance nothing renders (`D3`: the statement `new D3().new In();`)
//!    keeps its refusal on both legs — the tail dance alone admits nothing;
//! 4. a kept check result (`E1` byte-patched `pop` → `astore_1` into a Class-typed local) never
//!    folds — the tail joins the site only when the check's result is discarded;
//! 5. user statements over a multi-read local stay statements while the member construction
//!    folds (`D2`, the same semantics the `G` fixture freezes for the parameter qualifier).

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
            "jarde-alloc-qualifier-{label}-{}-{nonce}-{}",
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
        .join("tests/fixtures/recover-javac8-allocation-qualifier-null-check")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("fixture {name} reads: {error}"))
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

/// The real javac 8 leg, straight from the frozen class files.
fn n1_jar() -> Vec<u8> {
    jar_of(&[
        (b"N1.class", fixture("n1/N1.class").as_slice()),
        (b"N1$Inner.class", fixture("n1/N1$Inner.class").as_slice()),
        (b"N1$Stat.class", fixture("n1/N1$Stat.class").as_slice()),
    ])
}

fn pod_jar() -> Vec<u8> {
    jar_of(&[
        (b"Pod.class", fixture("pod/Pod.class").as_slice()),
        (b"Pod$Nut.class", fixture("pod/Pod$Nut.class").as_slice()),
    ])
}

fn d3_jar() -> Vec<u8> {
    jar_of(&[
        (b"D3.class", fixture("d3/D3.class").as_slice()),
        (b"D3$In.class", fixture("d3/D3$In.class").as_slice()),
    ])
}

fn e1_kept_jar() -> Vec<u8> {
    jar_of(&[
        (b"E1.class", fixture("e1-kept/E1.class").as_slice()),
        (b"E1$In.class", fixture("e1-kept/E1$In.class").as_slice()),
    ])
}

fn d2_jar() -> Vec<u8> {
    jar_of(&[
        (b"D2.class", fixture("d2/D2.class").as_slice()),
        (b"D2$In.class", fixture("d2/D2$In.class").as_slice()),
    ])
}

/// The dual leg: the same sources compiled by the toolchain `javac --release 8`, which writes
/// no enclosing-instance check for the allocation qualifier.
fn compile_with_path_javac(label: &str, source: &str) -> Vec<u8> {
    let temp = TestDirectory::new(label);
    let file = source
        .split_once("class ")
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .map(|name| format!("{name}.java"))
        .expect("the source states a class");
    std::fs::write(temp.path().join(&file), source).expect("the source is written");
    let compiled = Command::new("javac")
        .args(["--release", "8"])
        .arg(&file)
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let mut entries = Vec::new();
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
            entries.push((
                name.into_bytes(),
                std::fs::read(&path).expect("the class reads"),
            ));
        }
    }
    let borrowed: Vec<(&[u8], &[u8])> = entries
        .iter()
        .map(|(name, bytes)| (name.as_slice(), bytes.as_slice()))
        .collect();
    jar_of(&borrowed)
}

const N1_SOURCE: &str =
    include_str!("fixtures/recover-javac8-allocation-qualifier-null-check/n1/N1.java");
const POD_SOURCE: &str =
    include_str!("fixtures/recover-javac8-allocation-qualifier-null-check/pod/Pod.java");
const D2_SOURCE: &str =
    include_str!("fixtures/recover-javac8-allocation-qualifier-null-check/d2/D2.java");
const D3_SOURCE: &str =
    include_str!("fixtures/recover-javac8-allocation-qualifier-null-check/d3/D3.java");

#[test]
fn real_javac8_allocation_qualifier_folds_with_its_int_argument() {
    let report = source_of(&n1_jar(), "N1");
    assert!(
        report
            .text
            .contains("java.lang.System.out.println(new N1().new Inner(3).total());"),
        "the allocation qualifier with an argument must fold on the real javac 8 leg:\n{}",
        report.text
    );
    assert!(
        report.text.contains("return arg1.new Inner(9).total();"),
        "the parameter qualifier of the same family must fold with it:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("not recovered: the recovery run"),
        "no member stays unrecovered:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("@bytecode"),
        "no instruction of the family stays quoted:\n{}",
        report.text
    );
}

#[test]
fn allocation_qualifier_without_arguments_folds_on_both_legs() {
    for (label, jar) in [
        ("real8-pod", pod_jar()),
        ("javac23-pod", compile_with_path_javac("pod23", POD_SOURCE)),
    ] {
        let report = source_of(&jar, "Pod");
        assert!(
            report
                .text
                .contains("java.lang.System.out.println(new Pod().new Nut().mark());"),
            "{label}: the no-argument allocation qualifier must fold:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("not recovered: the recovery run"),
            "{label}: no member stays unrecovered:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("@bytecode"),
            "{label}: no instruction of the family stays quoted:\n{}",
            report.text
        );
    }
}

#[test]
fn a_construction_with_no_rendering_reader_refuses_on_both_legs() {
    for (label, jar) in [
        ("real8-d3", d3_jar()),
        ("javac23-d3", compile_with_path_javac("d323", D3_SOURCE)),
    ] {
        let report = source_of(&jar, "D3");
        assert!(
            !report.text.contains("new In()"),
            "{label}: the discarded construction must not fold:\n{}",
            report.text
        );
        assert!(
            report.text.contains("@bytecode"),
            "{label}: the refusal quotes the construction loudly:\n{}",
            report.text
        );
    }
}

#[test]
fn a_kept_check_result_never_joins_the_site() {
    // The frozen `E1.class` carries the equal-length `pop` → `astore_1` patch documented in the
    // fixture README: the check's result is kept in the Class-typed local 1, so the tail must
    // not join the nested site and the construction must keep its loud refusal.
    let report = source_of(&e1_kept_jar(), "E1");
    assert!(
        !report.text.contains("new In()"),
        "a kept check result must not fold the construction:\n{}",
        report.text
    );
    assert!(
        report.text.contains("@bytecode 13"),
        "the dance stays quoted when its result is kept:\n{}",
        report.text
    );
}

#[test]
fn user_statements_survive_while_the_member_construction_folds() {
    for (label, jar) in [
        ("real8-d2", d2_jar()),
        ("javac23-d2", compile_with_path_javac("d223", D2_SOURCE)),
    ] {
        let report = source_of(&jar, "D2");
        assert!(
            report.text.contains("local1.hit();"),
            "{label}: the user's own call stays a statement:\n{}",
            report.text
        );
        assert!(
            report.text.contains("return local1.new In().v();"),
            "{label}: the qualified construction folds beside it:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("not recovered: the recovery run"),
            "{label}: no member stays unrecovered:\n{}",
            report.text
        );
    }
}

#[test]
fn the_javac23_leg_of_the_anchor_keeps_its_recovery() {
    // The anchor source compiled by javac 23 `--release 8` writes no dance; its recovery must
    // stay exactly as it was (zero regression on the javac 9+ leg).
    let report = source_of(&compile_with_path_javac("n123", N1_SOURCE), "N1");
    assert!(
        report
            .text
            .contains("java.lang.System.out.println(new N1().new Inner(3).total());"),
        "the javac 23 leg keeps folding the anchor:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("@bytecode"),
        "no instruction of the family stays quoted:\n{}",
        report.text
    );
}
