//! The ctor-reorder dispatch guard, over the frozen fixtures the regression was proved on.
//!
//! `recover-synthetic-ctor-super-order` (daa4fb31) moved every certified synthetic pre-super
//! capture group past the constructor call. When the superclass constructor virtually dispatches
//! on `this` — an anonymous subclass overriding a hook the superclass constructor calls — that
//! move changes what the override reads during construction: the frozen
//! `anonymous-super-dispatch` fixture ran `visibleDuringSuper=true` as bytes and `false`
//! recompiled, a *silent* behavior change behind a newly-compilable text. The guard reverts the
//! order to the bytes' own whenever the constructor call is not `java/lang/Object.<init>()V` —
//! the one call that provably runs no user code — so the presentation may be uncompilable (a
//! loud failure javac refuses) but never compiles into a different program.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

// The dispatch fixture: the anonymous class whose superclass constructor calls the method the
// anonymous class overrides, reading the capture during the superclass's own construction.
const DISPATCH_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-super-dispatch/AnonymousSuperDispatch.class"
);
const DISPATCH_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-super-dispatch/AnonymousSuperDispatch$1.class"
);
const DISPATCH_BASE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-super-dispatch/Base.class");

// Two more frozen fixtures with the same pre-super synthetic group and a user-class super —
// the reorder was safe for these two, and the guard returns them to the byte order regardless.
const ARGS_ROOT: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-super-args/AnonymousSuperArgs.class");
const ARGS_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-super-args/AnonymousSuperArgs$1.class"
);
const ARGS_BASE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-super-args/Base.class");

const CAPTURE_ROOT: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-capture/AnonymousCaptureCases.class");
const CAPTURE_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-capture/AnonymousCaptureCases$1.class"
);
const CAPTURE_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-capture/AnonymousCaptureCases$Base.class"
);

#[test]
fn the_dispatch_fixture_keeps_the_capture_write_before_the_constructor_call() {
    let snapshot = open(zip_of(&[
        (b"AnonymousSuperDispatch.class", DISPATCH_ROOT),
        (b"AnonymousSuperDispatch$1.class", DISPATCH_CHILD),
        (b"Base.class", DISPATCH_BASE),
    ]));
    let child = class_source_of(&snapshot, "AnonymousSuperDispatch$1");
    // The reorder did not happen: the capture write is presented where the bytes ran it, before
    // the constructor call the superclass constructor's virtual dispatch can read it across.
    assert!(
        before(&child, "val$captured =", "super("),
        "the capture write must stay before the constructor call (the reorder is the regression):\n{}",
        child.text
    );

    // And the text is never a silent change: either javac refuses the flexible constructor body
    // (the loud failure the verbatim order leaves), or the recompiled run does exactly what the
    // original classes do — `visibleDuringSuper=true` included.
    let scratch = Scratch::new("dispatch-guard");
    write_originals(
        &scratch,
        &[
            (b"AnonymousSuperDispatch.class", DISPATCH_ROOT),
            (b"AnonymousSuperDispatch$1.class", DISPATCH_CHILD),
            (b"Base.class", DISPATCH_BASE),
        ],
    );
    let original = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(scratch.path())
        .arg("AnonymousSuperDispatch")
        .output()
        .expect("JDK java is available for the original fixture run");
    let original_out = String::from_utf8_lossy(&original.stdout).into_owned();
    assert!(
        original.status.success() && original_out.contains("visibleDuringSuper=true"),
        "the original fixture run is the behavior baseline:\n{original_out}{}",
        String::from_utf8_lossy(&original.stderr)
    );
    let rendered = scratch.path().join("AnonymousSuperDispatch$1.java");
    fs::write(&rendered, &child.text).expect("write the recovered source");
    let compile = Command::new("javac")
        .args(["--release", "8", "-g:none", "-Xlint:-options"])
        .arg("-cp")
        .arg(scratch.path())
        .arg("-d")
        .arg(scratch.path().join("out"))
        .arg(&rendered)
        .output()
        .expect("JDK javac is available for the recompile check");
    if compile.status.success() {
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(format!(
                "{}:{}",
                scratch.path().join("out").display(),
                scratch.path().display()
            ))
            .arg("AnonymousSuperDispatch")
            .output()
            .expect("JDK java is available for the recompiled run");
        assert!(
            run.status.success(),
            "the recompiled program did not verify:\n{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            original_out,
            "a text that compiles must run as the original ran — never a silent change"
        );
    }
    // A refused compile is the accepted loud state: the constructor body keeps the capture write
    // ahead of the call, which javac reads as a flexible constructor body. Nothing to assert
    // beyond the compile having run — the *absence* of (compiles ∧ differs) is the invariant.
}

#[test]
fn user_class_super_fixtures_return_to_the_compiler_s_byte_order() {
    // The two other frozen non-Object-super fixtures: the certified group is present and the
    // super target is a user class, so the guard keeps the verbatim order — the registered state
    // these fixtures' own acceptance evidence records (uncompilable, faithful, loud).
    let cases: [(&str, &[(&[u8], &[u8])], &str); 2] = [
        (
            "anonymous-super-args",
            &[
                (b"AnonymousSuperArgs.class", ARGS_ROOT),
                (b"AnonymousSuperArgs$1.class", ARGS_CHILD),
                (b"Base.class", ARGS_BASE),
            ],
            "AnonymousSuperArgs$1",
        ),
        (
            "anonymous-capture",
            &[
                (b"AnonymousCaptureCases.class", CAPTURE_ROOT),
                (b"AnonymousCaptureCases$1.class", CAPTURE_CHILD),
                (b"AnonymousCaptureCases$Base.class", CAPTURE_BASE),
            ],
            "AnonymousCaptureCases$1",
        ),
    ];
    for (label, entries, child_name) in cases {
        let snapshot = open(zip_of(entries));
        let child = class_source_of(&snapshot, child_name);
        assert!(
            before(&child, "val$captured =", "super("),
            "`{label}` keeps the capture write where its own bytes ran it:\n{}",
            child.text
        );
        // Nothing was dropped to make the order legal: the synthetic field's write is still
        // presented, and the constructor call is still presented.
        assert!(
            child.text.contains("super(") && child.text.contains("val$captured ="),
            "`{label}` presents the whole constructor, in the byte order:\n{}",
            child.text
        );
    }
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

/// Whether `lead`'s first line comes before `needle`'s, both stated by the report.
fn line_of(report: &ClassSourceReport, needle: &str) -> Option<usize> {
    report.text.lines().position(|line| line.contains(needle))
}

fn before(report: &ClassSourceReport, lead: &str, needle: &str) -> bool {
    match (line_of(report, lead), line_of(report, needle)) {
        (Some(lead), Some(needle)) => lead < needle,
        _ => false,
    }
}

/// The original fixture classes on disk, for the javac classpath and the baseline run.
fn write_originals(scratch: &Scratch, entries: &[(&[u8], &[u8])]) {
    for (name, data) in entries {
        let path = scratch.path().join(String::from_utf8_lossy(name).as_ref());
        fs::write(path, data).expect("write the original fixture class");
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
            "jarde-ctor-dispatch-guard-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the dispatch-guard scratch directory");
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
