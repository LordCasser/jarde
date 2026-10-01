//! The capture-ctor order patrol's presentation slice: javac's synthetic pre-super captures are
//! presented prologue-first, the family recompiles under `javac --release 8`, and the runs match
//! the original classes' behavior.
//!
//! The fixed fixture is the patrol's own transcription
//! (`openspec/evidence/java-syntax-2026-10-01/capture-ctor-order-patrol/fixture/`): `C1` with two
//! anonymous capture classes (`C1$1`/`C1$2`, synthetic `val$base`/`val$step` stored before the
//! constructor call) and `C2` with one member inner class (`C2$Inner`, synthetic `this$0`). What
//! this file proves through the public class-source surface:
//!
//! 1. every capture constructor is presented `super(…)`-first with the group after it — the text
//!    the source had — while the synthetic field declarations stay presented (faithful, not
//!    hidden) and the classes without the shape keep their verbatim constructor;
//! 2. the family files — the root and its physical members, exactly as recovered — compile
//!    together under `javac --release 8`, which the previous byte-order presentation could not;
//! 3. the recompiled runs verify and print what the original classes print (`25`/`6`, `10`).

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

const C1: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/capture-ctor-order-patrol/fixture/C1.class"
);
const C1_1: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/capture-ctor-order-patrol/fixture/C1$1.class"
);
const C1_2: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/capture-ctor-order-patrol/fixture/C1$2.class"
);
const C1_OP: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/capture-ctor-order-patrol/fixture/C1$Op.class"
);
const C2: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/capture-ctor-order-patrol/fixture/C2.class"
);
const C2_INNER: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/capture-ctor-order-patrol/fixture/C2$Inner.class"
);

/// The output baselines, stated by the patrol for this file's run to match (`orig.out`, plus the
/// original `C2` class's own run).
const C1_BASELINE: &str = "25\n6\n";
const C2_BASELINE: &str = "10\n";

/// The hand-written runner of the C2 family run: the recovered `C2` root keeps its own existing
/// boundary for the nested construction in its main, so the run exercises the member class the
/// way the original program did.
const C2_RUNNER: &str = "public class C2OrderRunner {\n    public static void main(String[] args) {\n        System.out.println(new C2$Inner(new C2(), 6).total());\n    }\n}\n";

#[test]
fn capture_constructors_are_presented_prologue_first_and_the_family_recompiles() {
    let snapshot = open(zip_of(&[
        (b"C1.class", C1),
        (b"C1$1.class", C1_1),
        (b"C1$2.class", C1_2),
        (b"C1$Op.class", C1_OP),
        (b"C2.class", C2),
        (b"C2$Inner.class", C2_INNER),
    ]));

    // The two capture classes: `super();` first, the capture write after it, and the synthetic
    // declaration itself still presented (faithful).
    for (name, write_line, declaration_line) in [
        ("C1$1", "this.val$base = arg1;", "final int val$base;"),
        ("C1$2", "this.val$step = arg1;", "final int val$step;"),
    ] {
        let child = class_source_of(&snapshot, name, EnvironmentPolicy::PlainJar);
        assert!(
            before(&child, "super();", write_line),
            "`{name}` presents the capture write after the constructor call:\n{}",
            child.text
        );
        assert!(
            child.text.contains(declaration_line),
            "`{name}` keeps the synthetic declaration presented:\n{}",
            child.text
        );
        // The envelope's own order is the constructor's: the call is the first statement the
        // body's statements carry, ahead of the capture's assignment.
        assert!(
            statement_lead_is(&child, &format!("{name}(int"), "super();"),
            "`{name}` opens its constructor with the call and nothing else:\n{}",
            child.text
        );
    }

    // The member inner class: `this$0` moves after the call, and the user field's write — which
    // the bytes already ran after it — stays where it was.
    let inner = class_source_of(&snapshot, "C2$Inner", EnvironmentPolicy::PlainJar);
    assert!(
        before(&inner, "super();", "this.this$0 = arg1;"),
        "the enclosing-instance write is presented after the constructor call:\n{}",
        inner.text
    );
    assert!(
        before(&inner, "this.this$0 = arg1;", "this.tag = arg2;"),
        "the user field's write keeps the place its own bytes had:\n{}",
        inner.text
    );
    assert!(
        inner.text.contains("final C2 this$0;"),
        "the synthetic declaration stays presented:\n{}",
        inner.text
    );
    assert!(
        statement_lead_is(&inner, "C2$Inner(C2", "super();"),
        "`C2$Inner` opens its constructor with the call and nothing else:\n{}",
        inner.text
    );

    // The root classes have no pre-super statements: the verbatim constructor is unchanged.
    for name in ["C1", "C2"] {
        let root = class_source_of(&snapshot, name, EnvironmentPolicy::PlainJar);
        assert!(
            !root.text.contains("val$") && !root.text.contains("this$0"),
            "`{name}` presents no synthetic capture at all:\n{}",
            root.text
        );
        assert!(
            root.text.contains("        super();\n"),
            "`{name}` still presents its verbatim constructor:\n{}",
            root.text
        );
    }
    let root = class_source_of(&snapshot, "C1", EnvironmentPolicy::PlainJar);
    assert!(
        root.text.contains("new C1$1(arg0)"),
        "the anonymous use stays a physical-class call (no inlining is claimed):\n{}",
        root.text
    );

    // The family files — the recovered texts, nothing rewritten — compile together under Java 8,
    // and the recompiled runs verify and print the originals' own output.
    if std::env::var("JARDE_ORDER_DEBUG").is_ok() {
        for name in ["C1", "C1$1", "C1$2", "C1$Op", "C2", "C2$Inner"] {
            let report = class_source_of(&snapshot, name, EnvironmentPolicy::PlainJar);
            let dump = format!("/tmp/jarde-order-{}.java", name.replace('$', "_"));
            fs::write(&dump, &report.text).expect("dump the recovered source");
            eprintln!("dumped {dump}");
        }
    }
    recompile_and_run(
        "c1-family",
        "C1",
        &["C1", "C1$1", "C1$2", "C1$Op"],
        &snapshot,
        None,
        C1_BASELINE,
    );
    // The C2 root's own main keeps its existing presentation boundary (the nested construction
    // `new C2$Inner(new C2(), 6)` is quoted, an unrelated recovery gap), so the family run is
    // driven by a runner that exercises the member class the source did.
    recompile_and_run(
        "c2-family",
        "C2OrderRunner",
        &["C2", "C2$Inner"],
        &snapshot,
        Some(C2_RUNNER),
        C2_BASELINE,
    );
}

/// Compiles the named classes' recovered texts — plus, when one is supplied, a hand-written
/// runner — together, and runs the family's main class.
fn recompile_and_run(
    case: &str,
    main: &str,
    names: &[&str],
    snapshot: &ArtifactSnapshot,
    runner: Option<&str>,
    baseline: &str,
) {
    let scratch = Scratch::new(case);
    let package = scratch.path().join("p");
    fs::create_dir_all(&package).expect("create the recompile directory");
    for name in names {
        let report = class_source_of(snapshot, name, EnvironmentPolicy::PlainJar);
        fs::write(package.join(format!("{name}.java")), &report.text)
            .expect("write the recovered source");
    }
    if let Some(runner) = runner {
        fs::write(package.join(format!("{main}.java")), runner).expect("write the family runner");
    }
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none"])
        .arg("-d")
        .arg(&package)
        .args(
            names
                .iter()
                .map(|name| package.join(format!("{name}.java"))),
        )
        .arg(package.join(format!("{main}.java")))
        .output()
        .expect("JDK javac is available for the Java 8 family regression");
    assert!(
        compile.status.success(),
        "javac rejected the recovered {case} family:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&package)
        .arg(main)
        .output()
        .expect("JDK java is available for the verified family run");
    assert!(
        run.status.success(),
        "the recompiled {case} family did not verify:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&run.stdout), baseline);
}

// ---------------------------------------------------------------------------------------------
// The library side of every case
// ---------------------------------------------------------------------------------------------

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget())
        .expect("the fixture snapshot opens")
}

fn performed<T>(outcome: OperationOutcome<T>) -> T {
    match outcome {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "expected one bound class, got {} candidate(s) and no execution",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "expected one bound class, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

fn request(
    snapshot: &ArtifactSnapshot,
    class: ClassRef,
    policy: EnvironmentPolicy,
) -> ClassSourceRequest {
    ClassSourceRequest {
        class,
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
    }
}

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    name: &str,
    policy: EnvironmentPolicy,
) -> ClassSourceReport {
    performed(
        Engine::new()
            .class_source(
                slice::from_ref(snapshot),
                &request(
                    snapshot,
                    ClassRef::Name {
                        class: ClassNameQuery::internal(name),
                    },
                    policy,
                ),
                &mut budget(),
            )
            .expect("a legal class-source request is answered"),
    )
}

/// A stored-only archive, built with the repository's own `rawzip` dev-dependency.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer
                .write_all(data)
                .expect("the fixture entry is written");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

/// The first line that contains one piece of text, when the text carries it.
fn line_of(report: &ClassSourceReport, needle: &str) -> Option<usize> {
    report.text.lines().position(|line| line.contains(needle))
}

/// Whether the first statement after `declaration`'s line is the constructor call itself: the
/// artifact's own envelope comments sit between the declaration and the statements, and the first
/// statement line after them is `super(…)`/`this(…)` exactly when the body starts with it.
fn statement_lead_is(report: &ClassSourceReport, declaration: &str, statement: &str) -> bool {
    let want = format!("{};", statement.trim_end().trim_end_matches(';'));
    let mut lines = report
        .text
        .lines()
        .skip_while(|line| !line.trim_start().starts_with(declaration));
    lines.next();
    lines
        .find(|line| {
            let line = line.trim_start();
            !line.is_empty() && !line.starts_with("//")
        })
        .is_some_and(|line| line.trim_start() == want)
}

/// Whether `lead`'s first line comes before `needle`'s, both stated by the report.
fn before(report: &ClassSourceReport, lead: &str, needle: &str) -> bool {
    match (line_of(report, lead), line_of(report, needle)) {
        (Some(lead), Some(needle)) => lead < needle,
        _ => false,
    }
}

/// One throwaway directory per compile-and-run case, removed with the test.
struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-ctor-super-order-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the family comparison directory");
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
