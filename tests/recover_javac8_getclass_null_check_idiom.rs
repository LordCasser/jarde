//! Acceptance tests of change `recover-javac8-getclass-null-check-idiom`: the null-check
//! spelling a compiler inserts for a source-qualified `outer.new Inner(…)` construction is
//! matched as one of two pairings — javac 9+'s `invokestatic
//! java/util/Objects.requireNonNull(Object)Object` and real javac 8's `invokevirtual
//! java/lang/Object.getClass()Class` — through one shared predicate
//! (`jarde_java::facts::is_discarded_null_check`), with every positional constraint unchanged.
//!
//! The fixtures under `tests/fixtures/recover-javac8-getclass-null-check-idiom/` are compiled
//! by **real javac 8** (Corretto 1.8.0_432, no `--release` flag) — that leg is the gap this
//! change closes and the leg the earlier anchors (all javac 23 `--release 8` products) could
//! not see. The dual leg (same sources, javac 23 `--release 8`) compiles in-test.
//!
//! Frozen behaviors:
//! 1. the parameter-qualifier form (`outer.new Inner(9)` in a static member's method) folds on
//!    the real javac 8 leg and on the javac 23 leg alike (positive anchors `N1x`, `Wrap` — two
//!    different identifier/shape families, per the not-the-only-positive rule);
//! 2. a user-written `o.getClass();` statement stays a statement while only the compiler's own
//!    check inside the allocation dance is folded (negative anchor `G`, both legs);
//! 3. the check's result not being discarded (`pop` replaced) refuses, on the getClass
//!    spelling, exactly as before (byte-patched probe over the frozen `N1x$Stat` class).

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
            "jarde-javac8-null-check-{label}-{}-{nonce}-{}",
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
        .join("tests/fixtures/recover-javac8-getclass-null-check-idiom")
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
fn n1x_jar() -> Vec<u8> {
    jar_of(&[
        (b"N1x.class", fixture("n1x/N1x.class").as_slice()),
        (
            b"N1x$Inner.class",
            fixture("n1x/N1x$Inner.class").as_slice(),
        ),
        (b"N1x$Stat.class", fixture("n1x/N1x$Stat.class").as_slice()),
    ])
}

fn wrap_jar() -> Vec<u8> {
    jar_of(&[
        (b"Wrap.class", fixture("wrap/Wrap.class").as_slice()),
        (
            b"Wrap$Seed.class",
            fixture("wrap/Wrap$Seed.class").as_slice(),
        ),
    ])
}

fn g_jar() -> Vec<u8> {
    jar_of(&[
        (b"G.class", fixture("g/G.class").as_slice()),
        (b"G$In.class", fixture("g/G$In.class").as_slice()),
    ])
}

/// The dual leg: the same sources compiled by the toolchain `javac` on PATH, which must state
/// the `requireNonNull` spelling for the same construction.
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

const N1X_SOURCE: &str =
    include_str!("fixtures/recover-javac8-getclass-null-check-idiom/n1x/N1x.java");
const G_SOURCE: &str = include_str!("fixtures/recover-javac8-getclass-null-check-idiom/g/G.java");
const WRAP_SOURCE: &str =
    include_str!("fixtures/recover-javac8-getclass-null-check-idiom/wrap/Wrap.java");

#[test]
fn real_javac8_parameter_qualifier_folds_like_the_javac23_leg() {
    for (label, jar, main_class) in [
        ("real8-n1x", n1x_jar(), "N1x"),
        ("real8-wrap", wrap_jar(), "Wrap"),
        (
            "javac23-n1x",
            compile_with_path_javac("n1x23", N1X_SOURCE),
            "N1x",
        ),
        (
            "javac23-wrap",
            compile_with_path_javac("wrap23", WRAP_SOURCE),
            "Wrap",
        ),
    ] {
        let report = source_of(&jar, main_class);
        let fold = if main_class == "N1x" {
            "return arg1.new Inner(9).total();"
        } else {
            "local1.new Seed(2).grow(3)"
        };
        assert!(
            report.text.contains(fold),
            "{label}: the qualified construction must fold:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("not recovered: the recovery run"),
            "{label}: no member stays unrecovered:\n{}",
            report.text
        );
        let nested = if main_class == "N1x" {
            "class Inner"
        } else {
            "class Seed"
        };
        assert!(
            report.text.contains(nested),
            "{label}: the member class folds into the root:\n{}",
            report.text
        );
    }
}

#[test]
fn user_written_getclass_statement_survives_on_both_legs() {
    for (label, jar) in [
        ("real8-g", g_jar()),
        ("javac23-g", compile_with_path_javac("g23", G_SOURCE)),
    ] {
        let report = source_of(&jar, "G");
        // The user's own statement stays a statement — the discarded call is not the check.
        assert!(
            report.text.contains("arg1.getClass();"),
            "{label}: the explicit statement must survive:\n{}",
            report.text
        );
        // Only the compiler-inserted check folds into the qualified construction.
        assert!(
            report.text.contains("return arg1.new In();"),
            "{label}: the qualified construction must fold:\n{}",
            report.text
        );
        assert!(
            report.text.contains("class In extends"),
            "{label}: the member class folds into the root:\n{}",
            report.text
        );
    }
}

#[test]
fn an_undiscarded_check_refuses_on_the_getclass_spelling() {
    // `N1x$Stat.use`'s dance is `aload_1; dup; invokevirtual getClass; pop`. Replacing the
    // `pop` (0x57) with `astore_1` (0x4c) — the parameter local, already consumed — keeps the
    // class decodable and states "the check's result is kept", which the rule must refuse.
    let stat = fixture("n1x/N1x$Stat.class");
    let pattern: &[u8] = &[0x2b, 0x59, 0xb6];
    let at = stat
        .windows(pattern.len())
        .position(|window| window == pattern)
        .expect("the use body states exactly one aload_1; dup; invokevirtual run");
    assert_eq!(
        stat[at + 5],
        0x57,
        "the dance's fifth byte after the run is the pop"
    );
    let mut patched = stat.clone();
    patched[at + 5] = 0x4c;
    let jar = jar_of(&[
        (b"N1x.class", fixture("n1x/N1x.class").as_slice()),
        (
            b"N1x$Inner.class",
            fixture("n1x/N1x$Inner.class").as_slice(),
        ),
        (b"N1x$Stat.class", patched.as_slice()),
    ]);
    let report = source_of(&jar, "N1x");
    assert!(
        !report.text.contains("arg1.new Inner(9).total();"),
        "an undiscarded check must not fold the construction:\n{}",
        report.text
    );
    assert!(
        report.text.contains("not recovered: the recovery run"),
        "the undiscarded shape refuses loudly:\n{}",
        report.text
    );
}
