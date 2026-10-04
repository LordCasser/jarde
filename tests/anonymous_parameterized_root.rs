//! The parameterized root-method form of the anonymous superclass projection
//! (`recover-anonymous-parameterized-root`): the allocation sits at the root method's direct
//! return and the capture value arrives through the root method's own single parameter, so the
//! projection spells `new Base() { ... }` with every proved capture read re-spelled as the
//! parameter's AST name.
//!
//! The frozen anchor `anonymous-super-dispatch` is the shape this slice unlocks: `create(final
//! String captured)` has descriptor `(Ljava/lang/String;)LBase;` — refused before this slice by
//! `anonymous_super_return_type_unproved`, which required the exact `()LBase;`. The `-g:none`
//! leg (`anonymous-super-dispatch-nodebug`) carries no `LocalVariableTable`, so its projection
//! proves the parameter name comes from the same-run AST's parameter table, never from debug
//! information. The five refusal fixtures keep their physical class text: the widened gate
//! accepts only `(P)Lparent;`, and its value flow (static root, complete scan, unmodified
//! parameter slot 0 consumed only by the allocation) refuses every shape the descriptor alone
//! cannot prove.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

// The anchor's frozen source: the root class and its `Base` superclass share one file, so the
// recompile legs derive the superclass source from it rather than restating it.
const ANCHOR_SOURCE: &str = include_str!(
    "fixtures/proved-java-structure/anonymous-super-dispatch/AnonymousSuperDispatch.java"
);

/// The frozen source's superclass part: everything before the root class declaration.
fn base_source() -> &'static str {
    let index = ANCHOR_SOURCE
        .find("public final class")
        .expect("the anchor source declares the root class");
    &ANCHOR_SOURCE[..index]
}

// The frozen `-g` anchor: the root method's parameter supplies the capture value.
const DISPATCH_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-super-dispatch/AnonymousSuperDispatch.class"
);
const DISPATCH_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-super-dispatch/AnonymousSuperDispatch$1.class"
);
const DISPATCH_BASE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-super-dispatch/Base.class");

// The same shape compiled with `-g:none`: no LocalVariableTable anywhere, so the projection's
// parameter name is the AST naming channel's own (`arg0`), never the debug table's.
const NODEBUG_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-super-dispatch-nodebug/AnonymousSuperDispatch.class"
);
const NODEBUG_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-super-dispatch-nodebug/AnonymousSuperDispatch$1.class"
);
const NODEBUG_BASE: &[u8] =
    include_bytes!("fixtures/proved-java-structure/anonymous-super-dispatch-nodebug/Base.class");

const MULTI_PARAMS_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/multiple-parameters/ParameterizedMultiParams.class"
);
const MULTI_PARAMS_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/multiple-parameters/ParameterizedMultiParams$1.class"
);
const CAPTURE_FROM_LOCAL_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/capture-from-local/ParameterizedCaptureFromLocal.class"
);
const CAPTURE_FROM_LOCAL_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/capture-from-local/ParameterizedCaptureFromLocal$1.class"
);
const ALSO_CONSUMED_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/parameter-also-consumed/ParameterizedAlsoConsumed.class"
);
const ALSO_CONSUMED_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/parameter-also-consumed/ParameterizedAlsoConsumed$1.class"
);
const INSTANCE_METHOD_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/instance-method/ParameterizedInstanceMethod.class"
);
const INSTANCE_METHOD_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/instance-method/ParameterizedInstanceMethod$1.class"
);
const MULTI_PARAMS_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/multiple-parameters/Base.class"
);
const CAPTURE_FROM_LOCAL_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/capture-from-local/Base.class"
);
const ALSO_CONSUMED_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/parameter-also-consumed/Base.class"
);
const INSTANCE_METHOD_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/instance-method/Base.class"
);
const SUPERTYPE_RETURN_BASE: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/supertype-return/Base.class"
);
const SUPERTYPE_RETURN_RENDERER: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/supertype-return/Renderer.class"
);
const SUPERTYPE_RETURN_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/supertype-return/ParameterizedSupertypeReturn.class"
);
const SUPERTYPE_RETURN_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/anonymous-parameterized-root-refusals/supertype-return/ParameterizedSupertypeReturn$1.class"
);

#[test]
fn the_parameterized_root_fixture_projects_the_capture_parameter_form() {
    let snapshot = open(zip_of(&[
        (b"AnonymousSuperDispatch.class", DISPATCH_ROOT),
        (b"AnonymousSuperDispatch$1.class", DISPATCH_CHILD),
        (b"Base.class", DISPATCH_BASE),
    ]));
    let root = class_source_of(&snapshot, "AnonymousSuperDispatch");
    // The direct-return allocation inlines with no argument list (the capture argument hides
    // behind the proved parameter) and every capture read re-spells as the root method's own
    // parameter name.
    assert!(root.text.contains("return new Base() {"), "{}", root.text);
    assert!(
        root.text
            .contains("AnonymousSuperDispatch.observed = captured;"),
        "{}",
        root.text
    );
    assert!(
        !root.text.contains("AnonymousSuperDispatch$1"),
        "{}",
        root.text
    );
    // The widened gate is the parameterized shape's: the refusal state is not what refused the
    // baseline, and no physical constructor remains to explain a silent drop.
    assert!(
        !matches!(
            root.anonymous_interface_projection,
            class_source::ClassSourceAnonymousInterfaceProjection::Refused { .. }
        ),
        "{:?}",
        root.anonymous_interface_projection
    );
}

#[test]
fn the_parameterized_root_projection_recompiles_and_runs_as_the_original() {
    let snapshot = open(zip_of(&[
        (b"AnonymousSuperDispatch.class", DISPATCH_ROOT),
        (b"AnonymousSuperDispatch$1.class", DISPATCH_CHILD),
        (b"Base.class", DISPATCH_BASE),
    ]));
    let root = class_source_of(&snapshot, "AnonymousSuperDispatch");
    let scratch = Scratch::new("parameterized-root");
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
        original.status.success()
            && original_out.contains("observed=captured-value")
            && original_out.contains("visibleDuringSuper=true"),
        "the original fixture run is the behavior baseline:\n{original_out}{}",
        String::from_utf8_lossy(&original.stderr)
    );
    // The rendered source set: the projected root beside the frozen superclass source (the
    // anchor source declares `Base` in the same file; the render claims only the root class).
    let rendered = scratch.path().join("AnonymousSuperDispatch.java");
    fs::write(&rendered, projected_source(&root.text)).expect("write the recovered source");
    fs::write(scratch.path().join("Base.java"), base_source())
        .expect("write the superclass source");
    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options"])
        .arg("-d")
        .arg(scratch.path().join("out"))
        .arg(&rendered)
        .arg(scratch.path().join("Base.java"))
        .output()
        .expect("JDK javac is available for the recompile check");
    assert!(
        compile.status.success(),
        "the rendered source set must compile (the baseline refused to):\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(scratch.path().join("out"))
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

#[test]
fn the_parameterized_root_projection_does_not_need_debug_information() {
    // The `-g:none` leg of the same shape: three classes, zero LocalVariableTable entries (the
    // fixture README records the `javap -l` count). The projection must succeed with the AST
    // naming channel's own parameter name, proving the name never comes from debug information.
    let snapshot = open(zip_of(&[
        (b"AnonymousSuperDispatch.class", NODEBUG_ROOT),
        (b"AnonymousSuperDispatch$1.class", NODEBUG_CHILD),
        (b"Base.class", NODEBUG_BASE),
    ]));
    let root = class_source_of(&snapshot, "AnonymousSuperDispatch");
    assert!(
        root.text
            .contains("private static Base create(java.lang.String arg0)"),
        "{}",
        root.text
    );
    assert!(root.text.contains("return new Base() {"), "{}", root.text);
    assert!(
        root.text
            .contains("AnonymousSuperDispatch.observed = arg0;"),
        "{}",
        root.text
    );
    assert!(
        !root.text.contains("AnonymousSuperDispatch$1"),
        "{}",
        root.text
    );
    // And the projected text is whole: the same source set compiles and replays the original
    // event log line for line.
    let scratch = Scratch::new("parameterized-root-nodebug");
    write_originals(
        &scratch,
        &[
            (b"AnonymousSuperDispatch.class", NODEBUG_ROOT),
            (b"AnonymousSuperDispatch$1.class", NODEBUG_CHILD),
            (b"Base.class", NODEBUG_BASE),
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
        "the original fixture run is the behavior baseline:\n{original_out}",
    );
    let rendered = scratch.path().join("AnonymousSuperDispatch.java");
    fs::write(&rendered, projected_source(&root.text)).expect("write the recovered source");
    fs::write(scratch.path().join("Base.java"), base_source())
        .expect("write the superclass source");
    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options"])
        .arg("-d")
        .arg(scratch.path().join("out"))
        .arg(&rendered)
        .arg(scratch.path().join("Base.java"))
        .output()
        .expect("JDK javac is available for the recompile check");
    assert!(
        compile.status.success(),
        "the `-g:none` leg's rendered source set must compile:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(scratch.path().join("out"))
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

#[test]
fn the_parameterized_root_refusals_keep_their_physical_text() {
    // Each widened-gate negative keeps its physical class text (the `$1` spelling javac
    // refuses) — the descriptor widening must not silently accept a shape its value flow
    // cannot prove, and the ring 2 return-type boundary must stay shut.
    let cases: [(&str, &str, &[(&[u8], &[u8])]); 5] = [
        (
            "ParameterizedMultiParams",
            "anonymous_super_return_type_unproved",
            &[
                (b"ParameterizedMultiParams.class", MULTI_PARAMS_ROOT),
                (b"ParameterizedMultiParams$1.class", MULTI_PARAMS_CHILD),
                (b"Base.class", MULTI_PARAMS_BASE),
            ],
        ),
        (
            "ParameterizedCaptureFromLocal",
            "anonymous_capture_argument_unproved",
            &[
                (
                    b"ParameterizedCaptureFromLocal.class",
                    CAPTURE_FROM_LOCAL_ROOT,
                ),
                (
                    b"ParameterizedCaptureFromLocal$1.class",
                    CAPTURE_FROM_LOCAL_CHILD,
                ),
                (b"Base.class", CAPTURE_FROM_LOCAL_BASE),
            ],
        ),
        (
            "ParameterizedAlsoConsumed",
            "anonymous_capture_argument_unproved",
            &[
                (b"ParameterizedAlsoConsumed.class", ALSO_CONSUMED_ROOT),
                (b"ParameterizedAlsoConsumed$1.class", ALSO_CONSUMED_CHILD),
                (b"Base.class", ALSO_CONSUMED_BASE),
            ],
        ),
        (
            "ParameterizedInstanceMethod",
            "anonymous_super_child_shape_unproved",
            &[
                (b"ParameterizedInstanceMethod.class", INSTANCE_METHOD_ROOT),
                (
                    b"ParameterizedInstanceMethod$1.class",
                    INSTANCE_METHOD_CHILD,
                ),
                (b"Base.class", INSTANCE_METHOD_BASE),
            ],
        ),
        (
            "ParameterizedSupertypeReturn",
            "anonymous_super_return_type_unproved",
            &[
                (b"ParameterizedSupertypeReturn.class", SUPERTYPE_RETURN_ROOT),
                (
                    b"ParameterizedSupertypeReturn$1.class",
                    SUPERTYPE_RETURN_CHILD,
                ),
                (b"Base.class", SUPERTYPE_RETURN_BASE),
                (b"Renderer.class", SUPERTYPE_RETURN_RENDERER),
            ],
        ),
    ];
    for (class, code, entries) in cases {
        let snapshot = open(zip_of(entries));
        let root = class_source_of(&snapshot, class);
        assert!(
            root.text.contains(&format!("new {class}$1(")),
            "`{class}` keeps the physical class text:\n{}",
            root.text
        );
        assert!(
            root.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == code),
            "`{class}` refuses with `{code}`: {:?}",
            root.diagnostics
        );
    }
}

/// The report's text minus the `// jarde:` bookkeeping preamble: the compilable source region.
fn projected_source(text: &str) -> String {
    text.lines()
        .filter(|line| !line.starts_with("// jarde:"))
        .map(|line| format!("{line}\n"))
        .collect()
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
            "jarde-parameterized-root-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the parameterized-root scratch directory");
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
