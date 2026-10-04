//! The non-`op` control probe for the DT-12 String-argument enum slice
//! (`recover-enum-string-field-name-generalization`).
//!
//! The accepted `recover-proved-string-arg-enum-constant-bodies` slice hard-coded the String
//! source argument's field name to `op` — exactly its anchor's own field name, so the
//! hard-coding was invisible to that acceptance. The frozen fixture here is the same
//! two-constant anonymous-body enum with the field named `label`: an implementation that takes
//! the name from the constructor bytecode projects it, a hard-coding refuses it and degrades the
//! constants to illegal field declarations (`public static final demo.LabeledOps TIMES;`).

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

const LABELED_OPS: &[u8] = include_bytes!("fixtures/enum-string-field-name/demo/LabeledOps.class");
const LABELED_OPS_1: &[u8] =
    include_bytes!("fixtures/enum-string-field-name/demo/LabeledOps$1.class");
const LABELED_OPS_2: &[u8] =
    include_bytes!("fixtures/enum-string-field-name/demo/LabeledOps$2.class");
const LABELED_VALUE: &[u8] =
    include_bytes!("fixtures/enum-string-field-name/demo/LabeledValue.class");
const PROBE_SOURCE: &str = include_str!("fixtures/enum-string-field-name/Probe.java");
const LABEL_SOURCE: &str = include_str!("fixtures/enum-string-field-name/LabeledValue.java");
const ORIGINAL_BEHAVIOR: &str =
    include_str!("fixtures/enum-string-field-name/original-behavior.txt");

/// A stored-only archive of the frozen fixture, built with the repository's own `rawzip`
/// dev-dependency.
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

fn open(archive: Vec<u8>) -> ArtifactSnapshot {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("task defaults provide a bounded budget");
    engine
        .open(ArtifactInput::bytes(archive), &mut budget)
        .expect("the frozen fixture archive opens")
}

fn class_source_of(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("task defaults provide a bounded budget");
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
    match engine
        .class_source(std::slice::from_ref(snapshot), &request, &mut budget)
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("expected one labeled enum definition, found {other:?}"),
    }
}

#[test]
fn the_labeled_string_argument_fixture_projects_the_proved_field_name() {
    let snapshot = open(zip_of(&[
        (b"demo/LabeledOps.class", LABELED_OPS),
        (b"demo/LabeledOps$1.class", LABELED_OPS_1),
        (b"demo/LabeledOps$2.class", LABELED_OPS_2),
        (b"demo/LabeledValue.class", LABELED_VALUE),
    ]));
    let labeled = class_source_of(&snapshot, "demo/LabeledOps");
    let text = &labeled.text;
    // The constants present as the source-level constant list, each with its own body.
    assert!(text.contains("TIMES(\"*\") {"), "{}", text);
    assert!(text.contains("DIVIDE(\"/\") {"), "{}", text);
    // The constructor text spells the field name the constructor bytecode proved — never a
    // hard-coded one, and never the degraded field-declaration constants.
    assert!(
        text.contains("private LabeledOps(java.lang.String arg0) {"),
        "{}",
        text
    );
    assert!(text.contains("this.label = arg0;"), "{}", text);
    assert!(!text.contains("this.op"), "{}", text);
    assert!(
        !text.contains("public static final demo.LabeledOps TIMES;"),
        "{}",
        text
    );
}

#[test]
fn the_labeled_projection_recompiles_and_runs_as_the_original() {
    let snapshot = open(zip_of(&[
        (b"demo/LabeledOps.class", LABELED_OPS),
        (b"demo/LabeledOps$1.class", LABELED_OPS_1),
        (b"demo/LabeledOps$2.class", LABELED_OPS_2),
        (b"demo/LabeledValue.class", LABELED_VALUE),
    ]));
    let labeled = class_source_of(&snapshot, "demo/LabeledOps");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let scratch = std::env::temp_dir().join(format!(
        "jarde-enum-string-field-name-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(scratch.join("original/demo")).expect("create the scratch directories");
    fs::create_dir_all(scratch.join("out")).expect("create the compile output directory");

    // The original run over the frozen bytes is the behavior baseline.
    for (name, data) in [
        ("LabeledOps.class", LABELED_OPS),
        ("LabeledOps$1.class", LABELED_OPS_1),
        ("LabeledOps$2.class", LABELED_OPS_2),
        ("LabeledValue.class", LABELED_VALUE),
    ] {
        fs::write(scratch.join("original/demo").join(name), data)
            .expect("write the original fixture class");
    }
    fs::write(scratch.join("Probe.java"), PROBE_SOURCE).expect("write the probe source");
    let compile_probe = Command::new("javac")
        .arg("-cp")
        .arg(scratch.join("original"))
        .arg("-d")
        .arg(scratch.join("original"))
        .arg(scratch.join("Probe.java"))
        .output()
        .expect("JDK javac is available for the original probe");
    assert!(
        compile_probe.status.success(),
        "the frozen probe compiles: {}",
        String::from_utf8_lossy(&compile_probe.stderr)
    );
    let original = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(scratch.join("original"))
        .arg("demo.Probe")
        .output()
        .expect("JDK java is available for the original fixture run");
    assert!(
        original.status.success(),
        "the original fixture run: {}",
        String::from_utf8_lossy(&original.stderr)
    );
    let original_out = String::from_utf8_lossy(&original.stdout).into_owned();
    assert_eq!(
        original_out, ORIGINAL_BEHAVIOR,
        "the frozen behavior baseline"
    );

    // The rendered source set: the projected enum beside the frozen interface source.
    let rendered = scratch.join("LabeledOps.java");
    fs::write(&rendered, &labeled.text).expect("write the recovered source");
    fs::write(scratch.join("LabeledValue.java"), LABEL_SOURCE).expect("write the interface source");
    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options"])
        .arg("-d")
        .arg(scratch.join("out"))
        .arg(&rendered)
        .arg(scratch.join("LabeledValue.java"))
        .output()
        .expect("JDK javac is available for the recompile check");
    assert!(
        compile.status.success(),
        "the rendered source set must compile:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let compile_probe_projected = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options"])
        .arg("-cp")
        .arg(scratch.join("out"))
        .arg("-d")
        .arg(scratch.join("out"))
        .arg(scratch.join("Probe.java"))
        .output()
        .expect("JDK javac is available for the projected probe");
    assert!(
        compile_probe_projected.status.success(),
        "the probe compiles against the projection: {}",
        String::from_utf8_lossy(&compile_probe_projected.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(scratch.join("out"))
        .arg("demo.Probe")
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
    let _ = fs::remove_dir_all(&scratch);
}
