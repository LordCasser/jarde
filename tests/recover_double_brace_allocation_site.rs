//! The double-brace allocation point of change `recover-double-brace-allocation-site`: at one
//! single-use allocation point whose companion is an anonymous subclass of a spellable superclass
//! with a pure instance initializer block as its body, the presentation writes the source-level
//! double-brace form — `new Super(args…) { { body } }` — and hides the companion from the text.
//!
//! What this file pins through the public class-source surface:
//!
//! 1. the control (`tests/fixtures/proved-java-structure/double-brace-allocation-site/`, both
//!    legs) presents the double-brace form at its allocation point, with the capture read spelled
//!    as the enclosing method's own parameter and the companion named nowhere;
//! 2. the three negatives — a companion that declares a method, a companion constructed twice, a
//!    superclass whose pool form is not a source name — keep the presentation they had: the
//!    companion's own class text and the `new Child(args)` call, byte for byte;
//! 3. the control's text compiles under `javac --release 8` and under a real javac 8, verifies
//!    under `java -Xverify:all` and prints the original classes' own answer;
//! 4. the physical companion stays queryable beside the projection — the presentation hides it
//!    from the *allocation point*, not from the run.
//!
//! The anchor itself is the patrol's `DB` and the `double-brace-capture/` fixture's two legs;
//! `tests/double_brace_capture.rs` owns those assertions.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const STORE: u16 = 0;

const FIXTURE: &str = "tests/fixtures/proved-java-structure/double-brace-allocation-site";

const CONTROL_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v23/controls/single-site/DBS.class"
);
const CONTROL_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v23/controls/single-site/DBS$1.class"
);
const CONTROL_ROOT_8: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v8/controls/single-site/DBS.class"
);
const CONTROL_CHILD_8: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v8/controls/single-site/DBS$1.class"
);
const METHODS_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v23/negatives/methods/DBM.class"
);
const METHODS_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v23/negatives/methods/DBM$1.class"
);
const UNSPELLABLE_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v23/negatives/unspellable-super/DBN.class"
);
const UNSPELLABLE_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v23/negatives/unspellable-super/DBN$1.class"
);
const UNSPELLABLE_CARRIER: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v23/negatives/unspellable-super/Carrier.class"
);
const UNSPELLABLE_NESTED: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v23/negatives/unspellable-super/Carrier$Nested.class"
);
const MULTI_ROOT: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v23/negatives/multi-site/DBS2.class"
);
const MULTI_CHILD: &[u8] = include_bytes!(
    "fixtures/proved-java-structure/double-brace-allocation-site/v23/negatives/multi-site/DBS2$1.class"
);

/// The control's own answer (its `main` prints the one element the block added).
const CONTROL_BASELINE: &str = "z\n";

#[test]
fn the_single_use_control_presents_the_double_brace_form_at_its_allocation_point() {
    for (label, root, child) in [
        ("v23", CONTROL_ROOT, CONTROL_CHILD),
        ("v8", CONTROL_ROOT_8, CONTROL_CHILD_8),
    ] {
        let snapshot = open(jar_of(&[(b"DBS.class", root), (b"DBS$1.class", child)]));
        let report = class_source_of(&snapshot, "DBS");
        // The envelope's own header is the report's, and every place the text is not a full
        // recovery says so — the assertion reads that claim instead of assuming it.
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `DBS` from the class file's own"),
            "`{label}`: the class view's own header leads:\n{}",
            report.text
        );
        // The allocation point: the source form, the superclass's own name, the block, and the
        // capture read spelled as the enclosing method's parameter.
        assert!(
            report.text.contains("return new java.util.ArrayList() {"),
            "`{label}`: the allocation point presents the double-brace form:\n{}",
            report.text
        );
        assert!(
            report
                .text
                .contains("{\n                this.add((java.lang.Object) s);\n            }"),
            "`{label}`: the block is the instance initializer's statement, the capture read spelled as `s`:\n{}",
            report.text
        );
        // The companion hides from the text: no pool-form name survives the projection.
        assert!(
            !report.text.contains("DBS$1"),
            "`{label}`: the companion is hidden from the text:\n{}",
            report.text
        );
        // The physical companion stays queryable beside the projection.
        let physical = class_source_of(&snapshot, "DBS$1");
        assert!(
            physical
                .text
                .contains("class DBS$1 extends java.util.ArrayList"),
            "`{label}`: the physical companion stays presented:\n{}",
            physical.text
        );
    }
}

#[test]
fn the_controls_recompiled_text_compiles_and_runs_as_the_original_classes() {
    for (label, root, child) in [
        ("v23", CONTROL_ROOT, CONTROL_CHILD),
        ("v8", CONTROL_ROOT_8, CONTROL_CHILD_8),
    ] {
        let snapshot = open(jar_of(&[(b"DBS.class", root), (b"DBS$1.class", child)]));
        let report = class_source_of(&snapshot, "DBS");
        let scratch = Scratch::new(label);
        fs::write(scratch.path().join("DBS.java"), &report.text)
            .expect("write the recovered class");
        let compile = Command::new("javac")
            .args(["--release", "8", "-g:none", "-Xlint:-options"])
            .arg("-d")
            .arg(scratch.path())
            .arg(scratch.path().join("DBS.java"))
            .output()
            .expect("JDK javac is available for the Java 8 recompile");
        assert!(
            compile.status.success(),
            "`{label}`: javac rejected the recovered text:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(scratch.path())
            .arg("DBS")
            .output()
            .expect("JDK java is available for the verified run");
        assert!(
            run.status.success(),
            "`{label}`: the recompiled text did not verify:\n{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&run.stdout),
            CONTROL_BASELINE,
            "`{label}`: the recompiled text runs as the original classes do"
        );
        // The real javac 8 leg, when the install is present (the fixture's README names it).
        let javac8 = Path::new(
            "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac",
        );
        let java8 = Path::new(
            "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java",
        );
        if javac8.is_file() {
            let scratch8 = Scratch::new(&format!("{label}-javac8"));
            fs::write(scratch8.path().join("DBS.java"), &report.text)
                .expect("write the recovered class for javac 8");
            let compile8 = Command::new(javac8)
                .args(["-g:none"])
                .arg("-d")
                .arg(scratch8.path())
                .arg(scratch8.path().join("DBS.java"))
                .output()
                .expect("the javac 8 install runs");
            assert!(
                compile8.status.success(),
                "`{label}`: javac 8 rejected the recovered text:\n{}",
                String::from_utf8_lossy(&compile8.stderr)
            );
            let run8 = Command::new(java8)
                .args(["-Xverify:all", "-cp"])
                .arg(scratch8.path())
                .arg("DBS")
                .output()
                .expect("the java 8 install runs");
            assert!(
                run8.status.success(),
                "`{label}`: the javac 8 recompile did not verify:\n{}",
                String::from_utf8_lossy(&run8.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&run8.stdout),
                CONTROL_BASELINE,
                "`{label}`: the javac 8 recompile runs as the original classes do"
            );
        }
    }
}

#[test]
fn the_three_negatives_keep_the_presentation_they_had() {
    // A companion that declares a method: its body is not a pure instance block.
    let methods = open(jar_of(&[
        (b"DBM.class", METHODS_ROOT),
        (b"DBM$1.class", METHODS_CHILD),
    ]));
    let report = class_source_of(&methods, "DBM");
    assert!(
        report.text.contains("return new DBM$1(s);"),
        "the companion's call stays presented:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("new java.util.ArrayList()"),
        "no double-brace form is claimed for this shape:\n{}",
        report.text
    );
    let child = class_source_of(&methods, "DBM$1");
    assert!(
        child.text.contains("void mark()") && child.text.contains("super();"),
        "the companion's own text keeps its declaration and its constructor order:\n{}",
        child.text
    );

    // A superclass whose pool form is not a source name (`Carrier$Nested`).
    let unspellable = open(jar_of(&[
        (b"DBN.class", UNSPELLABLE_ROOT),
        (b"DBN$1.class", UNSPELLABLE_CHILD),
        (b"Carrier.class", UNSPELLABLE_CARRIER),
        (b"Carrier$Nested.class", UNSPELLABLE_NESTED),
    ]));
    let report = class_source_of(&unspellable, "DBN");
    assert!(
        report.text.contains("return new DBN$1(s);"),
        "the companion's call stays presented:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("Carrier$Nested() {") && !report.text.contains("Carrier.Nested() {"),
        "no double-brace form names the unspellable superclass:\n{}",
        report.text
    );

    // A companion constructed twice: hiding it from the text would drop one of the two uses.
    let multi = open(jar_of(&[
        (b"DBS2.class", MULTI_ROOT),
        (b"DBS2$1.class", MULTI_CHILD),
    ]));
    let report = class_source_of(&multi, "DBS2");
    assert_eq!(
        report.text.matches("new DBS2$1(arg0)").count(),
        2,
        "both allocation points keep the companion's call:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("new java.util.ArrayList()"),
        "no double-brace form is claimed for this shape:\n{}",
        report.text
    );
}

/// The fixture path the two files above read their bytes from (a guard against a moved fixture).
#[test]
fn the_fixture_directory_is_the_one_this_file_reads() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    assert!(
        root.join("README.md").is_file() && root.join("freeze.py").is_file(),
        "the fixture keeps its own README and freeze script: {}",
        root.display()
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
            "jarde-double-brace-site-{label}-{}-{nonce}-{}",
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
