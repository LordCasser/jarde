//! The double-brace capture anchor of change `recover-capture-ctor-super-order`: a companion
//! constructor whose pre-super prefix is the compiler's certified synthetic-store group is
//! presented **after** the constructor call when the call cannot observe the group's fields, and
//! the family recompiles and runs as the original classes do.
//!
//! The frozen fixture is the patrol's own source
//! (`tests/fixtures/proved-java-structure/double-brace-capture/`, both compilation legs: `v23/`
//! from `javac --release 8` and `v8/` from a real javac 8). javac writes the capture class's
//! constructor with `putfield val$s` before `invokespecial java/util/ArrayList.<init>()V` in both
//! legs, which is legal JVM bytecode and an uncompilable Java source (a flexible constructor
//! body) — the state this change normalizes.
//!
//! What this file pins through the public class-source surface:
//!
//! 1. `DB$2` (the capture form) presents `super();` first and the capture write after it, in both
//!    legs, with the synthetic field declaration and the envelope's own header still presented;
//! 2. `DB$1` (the no-capture form) keeps the presentation it had before the change — the
//!    zero-regression anchor — while `DB`'s two allocation points present the **source-level
//!    double-brace form** (change `recover-double-brace-allocation-site`, which replaced the
//!    `new DB$2(arg0)`/`new DB$1()` call forms this file pinned before it);
//! 3. the three recovered texts compile together under `javac --release 8`, run under
//!    `java -Xverify:all` and print `2/z`, the original classes' own output;
//! 4. the two hand-made super-argument probes
//!    (`tests/fixtures/proved-java-structure/capture-super-arg-probes/`) keep the current
//!    behavior: the field-reading argument is refused by the analysis (the same rule the JVM's
//!    verifier states — the shape is not loadable), and the call-bearing argument keeps the byte
//!    order, because a body this run does not hold is not a proof.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const STORE: u16 = 0;

// The double-brace fixture, both legs: the ambient javac's `--release 8` classes and a real
// javac 8's, frozen by the fixture's own `freeze.py`.
const DB_23_ROOT: &[u8] =
    include_bytes!("fixtures/proved-java-structure/double-brace-capture/v23/DB.class");
const DB_23_ONE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/double-brace-capture/v23/DB$1.class");
const DB_23_TWO: &[u8] =
    include_bytes!("fixtures/proved-java-structure/double-brace-capture/v23/DB$2.class");
const DB_8_ROOT: &[u8] =
    include_bytes!("fixtures/proved-java-structure/double-brace-capture/v8/DB.class");
const DB_8_ONE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/double-brace-capture/v8/DB$1.class");
const DB_8_TWO: &[u8] =
    include_bytes!("fixtures/proved-java-structure/double-brace-capture/v8/DB$2.class");

// The two hand-made super-argument probes: the field-reading argument (unverifiable bytecode)
// and the call-bearing argument (verifiable, and the one a reorder's argument walk refuses).
const READ_ARG_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/capture-super-arg-probes/read-arg/ReadArg.class"
);
const READ_ARG_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/capture-super-arg-probes/read-arg/ReadArg$1.class"
);
const READ_ARG_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/capture-super-arg-probes/read-arg/ReadArg$Base.class"
);
const CALL_ARG_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/capture-super-arg-probes/call-arg/CallArg.class"
);
const CALL_ARG_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/capture-super-arg-probes/call-arg/CallArg$1.class"
);
const CALL_ARG_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/capture-super-arg-probes/call-arg/CallArg$Base.class"
);
const CALL_ARG_HELPER: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/capture-super-arg-probes/call-arg/CallArg$Helper.class"
);

/// The original classes' own output (the patrol's `2/z`).
const DB_BASELINE: &str = "2/z\n";

/// The two compilation legs, as `(label, DB, DB$1, DB$2)`.
fn legs() -> [(&'static str, &'static [u8], &'static [u8], &'static [u8]); 2] {
    [
        ("v23", DB_23_ROOT, DB_23_ONE, DB_23_TWO),
        ("v8", DB_8_ROOT, DB_8_ONE, DB_8_TWO),
    ]
}

#[test]
fn the_capture_companion_presents_the_capture_write_after_the_constructor_call() {
    for (label, root, one, two) in legs() {
        let snapshot = open(jar_of(&[
            (b"DB.class", root),
            (b"DB$1.class", one),
            (b"DB$2.class", two),
        ]));
        let child = class_source_of(&snapshot, "DB$2");

        // The envelope's own header is the report's: the class view is not a compilable project
        // and every recovered body says so — the assertion reads that claim instead of assuming
        // it, so the day the presentation starts claiming compilability the anchor is revisited.
        assert!(
            child
                .text
                .starts_with("// jarde: presentation of `DB$2` from the class file's own"),
            "`{label}`: the class view's own header leads:\n{}",
            child.text
        );
        assert!(
            child
                .text
                .contains("presentation is not claimed to compile"),
            "`{label}`: the recovered body's own claim stays:\n{}",
            child.text
        );
        assert!(
            child
                .text
                .contains("class DB$2 extends java.util.ArrayList {")
                && child.text.contains("final java.lang.String val$s;"),
            "`{label}`: the declaration and the synthetic field stay presented:\n{}",
            child.text
        );

        // The order the change normalizes: the constructor call first, the certified group after
        // it, in both legs — javac 8 and javac 23 write the same byte order.
        assert_eq!(
            body_statements(&child, "DB$2(java.lang.String arg1)"),
            vec![
                "super();",
                "this.val$s = arg1;",
                "this.add((java.lang.Object) this.val$s);",
                "return;",
            ],
            "`{label}`: the capture write is presented after the constructor call:\n{}",
            child.text
        );
    }
}

#[test]
fn the_companions_keep_their_presentation_and_the_host_presents_the_double_brace_form() {
    // Updated by change `recover-double-brace-allocation-site` (assertion update, not deletion):
    // both anchors now present the **source-level double-brace form** at their allocation points,
    // so the host assertions below read that form instead of the physical-class calls. Every
    // companion-side assertion is unchanged: the companions' own class texts keep the order this
    // change's slice (`recover-capture-ctor-super-order`) presented, and the physical companions
    // stay queryable.
    for (label, root, one, two) in legs() {
        let snapshot = open(jar_of(&[
            (b"DB.class", root),
            (b"DB$1.class", one),
            (b"DB$2.class", two),
        ]));

        // The no-capture companion: no group at all, so the instance block's statements keep the
        // place the bytes gave them — the zero-regression anchor.
        let no_capture = class_source_of(&snapshot, "DB$1");
        assert_eq!(
            body_statements(&no_capture, "DB$1()"),
            vec![
                "super();",
                "this.add((java.lang.Object) \"a\");",
                "this.add((java.lang.Object) \"b\");",
                "return;",
            ],
            "`{label}`: the no-capture companion keeps its presentation:\n{}",
            no_capture.text
        );

        // The host: both allocation points present the double-brace form, the capture read spelled
        // as the enclosing method's own parameter, and neither companion is named anywhere in the
        // text. The physical-class calls are gone, and with them every pool-form name.
        let host = class_source_of(&snapshot, "DB");
        assert!(
            host.text
                .contains("return new java.util.ArrayList() {\n            {\n                this.add((java.lang.Object) s);\n            }\n        };"),
            "`{label}`: the capture anchor presents the double-brace form:\n{}",
            host.text
        );
        assert!(
            host.text.contains(
                "DB.dbl = new java.util.ArrayList() {\n            {\n                this.add((java.lang.Object) \"a\");\n                this.add((java.lang.Object) \"b\");\n            }\n        };"
            ),
            "`{label}`: the no-capture anchor presents the double-brace form:\n{}",
            host.text
        );
        assert!(
            !host.text.contains("DB$1")
                && !host.text.contains("DB$2")
                && !host.text.contains("val$"),
            "`{label}`: no companion name and no synthetic capture survives in the host:\n{}",
            host.text
        );
    }
}

#[test]
fn the_recompiled_family_compiles_and_runs_as_the_original_classes() {
    for (label, root, one, two) in legs() {
        let snapshot = open(jar_of(&[
            (b"DB.class", root),
            (b"DB$1.class", one),
            (b"DB$2.class", two),
        ]));
        let scratch = Scratch::new(label);
        for name in ["DB", "DB$1", "DB$2"] {
            let report = class_source_of(&snapshot, name);
            fs::write(scratch.path().join(format!("{name}.java")), &report.text)
                .expect("write the recovered class");
        }
        let compile = Command::new("javac")
            .args(["--release", "8", "-g:none", "-Xlint:-options"])
            .arg("-d")
            .arg(scratch.path())
            .arg(scratch.path().join("DB.java"))
            .arg(scratch.path().join("DB$1.java"))
            .arg(scratch.path().join("DB$2.java"))
            .output()
            .expect("JDK javac is available for the Java 8 family recompile");
        assert!(
            compile.status.success(),
            "`{label}`: javac rejected the recovered family:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(scratch.path())
            .arg("DB")
            .output()
            .expect("JDK java is available for the verified family run");
        assert!(
            run.status.success(),
            "`{label}`: the recompiled family did not verify:\n{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            DB_BASELINE,
            "`{label}`: the recompiled family runs as the original classes do"
        );
    }
}

#[test]
fn the_super_argument_probes_keep_the_current_refusal_and_the_byte_order() {
    // The field-reading argument: the shape is not verifiable (JVMS 4.10.1.9 refuses `getfield`
    // on `uninitializedThis`), and the analysis refuses the whole constructor rather than moving
    // anything. This is the *current fact* of a hand-made shape — the day the frame pass carries
    // it, the reorder's own argument walk has to answer for it and this anchor is revisited.
    let read = open(jar_of(&[
        (b"ReadArg.class", READ_ARG_ROOT),
        (b"ReadArg$1.class", READ_ARG_CHILD),
        (b"ReadArg$Base.class", READ_ARG_BASE),
    ]));
    let child = class_source_of(&read, "ReadArg$1");
    assert!(
        child
            .text
            .contains("not recovered: the recovery run for `<init>(Ljava/lang/String;)V`")
            && child.text.contains("ir_frame_deferred"),
        "the field-reading argument keeps the analysis' own refusal:\n{}",
        child.text
    );
    assert!(
        !child.text.contains("super(") && !child.text.contains("val$s = arg1;"),
        "no constructor statement is presented for the refused shape:\n{}",
        child.text
    );

    // The call-bearing argument: the class declares nothing but its constructor, so the only
    // refusal left is the argument walk's — and the write stays where the bytes put it.
    let call = open(jar_of(&[
        (b"CallArg.class", CALL_ARG_ROOT),
        (b"CallArg$1.class", CALL_ARG_CHILD),
        (b"CallArg$Base.class", CALL_ARG_BASE),
        (b"CallArg$Helper.class", CALL_ARG_HELPER),
    ]));
    let child = class_source_of(&call, "CallArg$1");
    assert_eq!(
        body_statements(&child, "CallArg$1(java.lang.String arg1)"),
        vec![
            "this.val$s = arg1;",
            "super((java.lang.String) CallArg$Helper.compute());",
            "return;",
        ],
        "a call-bearing argument keeps the byte order:\n{}",
        child.text
    );
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

fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    performed(
        Engine::new()
            .class_source(
                slice::from_ref(snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal(name),
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
                },
                &mut budget(),
            )
            .expect("a legal class-source request is answered"),
    )
}

/// The statements of one member's body, as the report's own lines spell them: the lines between
/// the member's declaration line and the closing brace, with the comment and blank lines dropped.
fn body_statements(report: &ClassSourceReport, declaration: &str) -> Vec<String> {
    let start = report
        .text
        .lines()
        .position(|line| line.trim_start().starts_with(declaration))
        .unwrap_or_else(|| panic!("the member `{declaration}` is presented:\n{}", report.text));
    let mut statements = Vec::new();
    for line in report.text.lines().skip(start + 1) {
        let trimmed = line.trim();
        if trimmed == "}" {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        statements.push(trimmed.to_owned());
    }
    statements
}

/// A stored-only archive, built with the repository's own `rawzip` dev-dependency.
fn jar_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
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

/// One throwaway directory per compile-and-run case, removed with the test.
struct Scratch(PathBuf);

static NEXT_SCRATCH: AtomicU64 = AtomicU64::new(0);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-double-brace-{label}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("create the double-brace scratch directory");
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
