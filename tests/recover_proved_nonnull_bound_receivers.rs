//! `recover-proved-nonnull-bound-receivers` in one frozen target: the Optional patrol's `OP` anchor
//! and this change's `BRN` negatives, on **both** compiler legs (javac 23.0.1 `--release 8` and real
//! javac 8, Corretto 1.8.0_432, from the same sources).
//!
//! `OP.sideEffect` is the anchor: `o.ifPresent(sb::append)` with `sb = new StringBuilder()`.
//! javac evaluates a bound method reference by checking the receiver for null *while the functional
//! value is created* — `dup; Objects.requireNonNull; pop` (javac 9+) or `dup; Object.getClass; pop`
//! (javac 8) immediately in front of the `invokedynamic` — and the refusal this change lifts states
//! that adapting the site as a lambda would move that null failure from creation to invocation. The
//! receiver's own value chain ends at the `new`, and no store follows the load that read it, so the
//! check is dead and the two presentations agree. The site's plan owns the check's three
//! instructions and reads its receiver **through** them, so the presented text is the lambda the
//! change promises.
//!
//! The change's own negatives stand beside it: `BRN`'s `nullableParameter` (a parameter read),
//! `nullableField` (a field read) and `rewrittenAfterCapture` (the slot written again after the
//! capture) each keep the refusal **verbatim**, because in all three the two presentations really
//! do disagree about a null receiver or about which object they read.
//!
//! The ignored replay strips the presentations the way the patrol's own stripped sources were made
//! (comment lines dropped), compiles each anchor with the installed `javac --release 8` and, when a
//! real javac 8 is present, with that one too, runs both under `-Xverify:all` and compares every
//! answer with the fixture's own class files.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

// -------------------------------------------------------------------------------------------
// The fixtures: the patrol's anchor and this change's negatives, compiled by both javac legs.
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names and the class files it produced.
struct Leg {
    label: &'static str,
    files: &'static [(&'static str, &'static [u8])],
}

impl Leg {
    /// The class files of one fixture family, in container order.
    fn family(&self, class: &str) -> Vec<(String, &'static [u8])> {
        let own = format!("{class}.class");
        let nested = format!("{class}$");
        let files: Vec<(String, &'static [u8])> = self
            .files
            .iter()
            .filter(|(name, _)| {
                let name: &str = name;
                name == own || name.starts_with(&nested)
            })
            .map(|(name, bytes)| ((*name).to_owned(), *bytes))
            .collect();
        assert_eq!(
            files.first().map(|(name, _)| name.as_str()),
            Some(own.as_str()),
            "the fixture family of `{class}` is committed"
        );
        files
    }

    /// One fixture family's container.
    fn fixture(&self, class: &str) -> Vec<u8> {
        let own = format!("{class}.class");
        let nested = format!("{class}$");
        let entries: Vec<(&[u8], &[u8])> = self
            .files
            .iter()
            .filter(|(name, _)| {
                let name: &str = name;
                name == own || name.starts_with(&nested)
            })
            .map(|(name, bytes)| (name.as_bytes(), *bytes))
            .collect();
        assert_eq!(
            entries.first().map(|(name, _)| *name),
            Some(own.as_bytes()),
            "the fixture family of `{class}` is committed"
        );
        zip_of(&entries)
    }
}

/// javac 23.0.1, `javac --release 8 -Xlint:-options -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "OP.class",
        include_bytes!("fixtures/recover-proved-nonnull-bound-receivers/v8/OP.class"),
    ),
    (
        "BRN.class",
        include_bytes!("fixtures/recover-proved-nonnull-bound-receivers/v8/BRN.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "OP.class",
        include_bytes!("fixtures/recover-proved-nonnull-bound-receivers/v8-javac8/OP.class"),
    ),
    (
        "BRN.class",
        include_bytes!("fixtures/recover-proved-nonnull-bound-receivers/v8-javac8/BRN.class"),
    ),
];

const LEGS: &[Leg] = &[
    Leg {
        label: "javac 23.0.1 --release 8",
        files: V8_FILES,
    },
    Leg {
        label: "Corretto 1.8.0_432 (real javac 8)",
        files: V8_JAVAC8_FILES,
    },
];

// -------------------------------------------------------------------------------------------
// The class-source surface.
// -------------------------------------------------------------------------------------------

/// The ordinary request plus the one optional category the member fold reads.
fn evidence() -> RecoveryEvidenceRequest {
    RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap)
}

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a ZIP")
}

/// One class-source presentation over the snapshot's own root container.
fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let request = ClassSourceRequest {
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
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &evidence(),
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition: {other:?}"),
    }
}

/// One member's own record in the assembled source.
fn method_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's text, for the assertions that compare a whole presentation.
fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &method_of(report, name).text
}

/// The one ZIP the class-source request reads, built from the committed class files.
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

// -------------------------------------------------------------------------------------------
// The presented texts, pinned whole (the p3 surface tests' own convention).
// -------------------------------------------------------------------------------------------

/// `OP.sideEffect`: the change's anchor — the `ifPresent` statement comes back whole, with the
/// bound receiver adapted as the lambda the change promises (`x -> sb.append(x)`) and the site's own
/// creation-time null check owned rather than quoted. Both legs answer this text byte for byte.
const OP_SIDE_EFFECT: &str = "    // jarde: generic Signature projection refused for `sideEffect(Ljava/util/Optional;)Ljava/lang/String;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types\n    static java.lang.String sideEffect(java.util.Optional arg0) {\n        // @method sideEffect(Ljava/util/Optional;)Ljava/lang/String;\n        // @declaration a static method of `OP`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.StringBuilder local1 = new java.lang.StringBuilder();\n        arg0.ifPresent((java.util.function.Consumer) ((java.lang.Object p0) -> local1.append((java.lang.String) p0)));\n        return local1.toString();\n    }\n";

/// The refusal every negative below keeps: the one creation-time sentence this layer writes, with
/// the site's BCI in front of it, unchanged by this change.
const REFUSAL: &str = "adapting this bound receiver would move its null failure from functional-value creation to invocation";

/// The three negatives of `BRN`, pinned whole: each keeps its refusal and quotes the producers
/// needed to explain it (the check's `dup` remains an unproved copy).
const BRN_NULLABLE_PARAMETER: &str = "    // jarde: generic Signature projection refused for `nullableParameter(Ljava/util/Optional;Ljava/lang/StringBuilder;)Ljava/lang/String;`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof\n    static java.lang.String nullableParameter(java.util.Optional arg0, java.lang.StringBuilder arg1) {\n        // jarde: not recovered: the recovery run for `nullableParameter(Ljava/util/Optional;Ljava/lang/StringBuilder;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method nullableParameter(Ljava/util/Optional;Ljava/lang/StringBuilder;)Ljava/lang/String;\n        // @declaration a static method of `BRN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 2\n        // the instruction at BCI 2 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds\n        // @bytecode 3 6 1 0 2 7 12 15 16 19\n        // the copy at BCI 2 has no proved local assignment\n        // @bytecode 12 7 2 0 1\n        // adapting this bound receiver would move its null failure from functional-value creation to invocation\n    }\n";

const BRN_NULLABLE_FIELD: &str = "    // jarde: generic Signature projection refused for `nullableField(Ljava/util/Optional;)Ljava/lang/String;`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof\n    java.lang.String nullableField(java.util.Optional arg1) {\n        // jarde: not recovered: the recovery run for `nullableField(Ljava/util/Optional;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method nullableField(Ljava/util/Optional;)Ljava/lang/String;\n        // @declaration an instance method of `BRN`, member flags 0x0000\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 5 2\n        // the instruction at BCI 5 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds\n        // @bytecode 6 9 2 1 0 5 10 15 18 19 22 25\n        // the copy at BCI 5 has no proved local assignment\n        // @bytecode 15 2 0 1\n        // adapting this bound receiver would move its null failure from functional-value creation to invocation\n    }\n";

const BRN_REWRITTEN_AFTER_CAPTURE: &str = "    // jarde: generic Signature projection refused for `rewrittenAfterCapture(Ljava/util/Optional;)Ljava/lang/String;`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof\n    static java.lang.String rewrittenAfterCapture(java.util.Optional arg0) {\n        // jarde: not recovered: the recovery run for `rewrittenAfterCapture(Ljava/util/Optional;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method rewrittenAfterCapture(Ljava/util/Optional;)Ljava/lang/String;\n        // @declaration a static method of `BRN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 10\n        // the instruction at BCI 10 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds\n        // @bytecode 11 14 9 0 3 4 7 8 10 15 20 23 26 27 29 32 33 34 37\n        // the copy at BCI 10 has no proved local assignment\n        // @bytecode 20 8 9\n        // adapting this bound receiver would move its null failure from functional-value creation to invocation\n    }\n";

// -------------------------------------------------------------------------------------------
// The anchor and the negatives.
// -------------------------------------------------------------------------------------------

/// The anchor recovers on both legs: the statement is presented whole, the diagnostic the change
/// lifts is gone from the class, and the construction the receiver comes from is still written as
/// the declaration it was.
#[test]
fn the_bound_receiver_of_a_proved_allocation_is_adapted() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("OP"));
        let report = class_source_of(&snapshot, "OP");
        assert_eq!(
            text_of(&report, "sideEffect"),
            OP_SIDE_EFFECT,
            "{}",
            leg.label
        );
        assert_eq!(
            report.text.matches(REFUSAL).count(),
            0,
            "`OP` keeps no bound-receiver refusal on {}:\n{}",
            leg.label,
            report.text
        );
        assert!(
            !report.text.contains("@bytecode"),
            "`OP` is presented whole on {}:\n{}",
            leg.label,
            report.text
        );
        // The zero-regression controls: the patrol's already recovering members, byte for byte as
        // the patrol recorded them.
        assert!(
            text_of(&report, "len").contains("if (arg0.isPresent())"),
            "{}",
            leg.label
        );
        assert!(
            text_of(&report, "parse").contains("catch (java.lang.NumberFormatException local1)"),
            "{}",
            leg.label
        );
    }
}

/// Every negative keeps the refusal **verbatim**: a parameter read, a field read and a slot written
/// again after the capture all present the two texts differently, so none of them is adapted.
#[test]
fn a_receiver_no_allocation_proves_keeps_its_creation_time_refusal() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BRN"));
        let report = class_source_of(&snapshot, "BRN");
        assert_eq!(
            text_of(&report, "nullableParameter"),
            BRN_NULLABLE_PARAMETER,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "nullableField"),
            BRN_NULLABLE_FIELD,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "rewrittenAfterCapture"),
            BRN_REWRITTEN_AFTER_CAPTURE,
            "{}",
            leg.label
        );
        assert_eq!(
            report.text.matches(REFUSAL).count(),
            3,
            "only the three unproved receivers stay refused on {}:\n{}",
            leg.label,
            report.text
        );
    }
}

// -------------------------------------------------------------------------------------------
// The replay: the presented text, stripped, compiled and run (needs a JDK on PATH).
// -------------------------------------------------------------------------------------------

static UNIQUE: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-proved-nonnull-bound-receivers-{label}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the replay directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// One class text with every `//` comment line dropped — the strip the patrols' own stripped
/// sources were made by.
fn comment_lines_dropped(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::to_owned)
        .collect()
}

/// The fixture's own class files, written where a JVM can load them.
fn original_classes(leg: &Leg, class: &str, directory: &Path) {
    for (name, bytes) in leg.family(class) {
        fs::write(directory.join(name), bytes).expect("the fixture class is written");
    }
}

/// The real javac 8 the frozen `v8-javac8` leg was compiled by, when this machine holds it.
fn javac8() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("JARDE_JAVAC8") {
        return Some(PathBuf::from(path));
    }
    let default = PathBuf::from(
        "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac",
    );
    default.is_file().then_some(default)
}

/// Compile one replay's stripped unit: the installed javac under `--release 8`, or a real javac 8
/// (whose default target is Java 8, and which has no `--release` flag). The unit is written under
/// the class's own name, which is what a public class's declaration requires.
fn compile_with(javac: &Path, release_8: bool, directory: &Path, class: &str, source: &str) {
    let path = directory.join(format!("{class}.java"));
    fs::write(&path, source).expect("the stripped unit is written");
    let mut command = Command::new(javac);
    if release_8 {
        command.args(["--release", "8"]);
    }
    let output = command
        .arg("-Xlint:-options")
        .arg("-d")
        .arg(directory)
        .arg(&path)
        .output()
        .expect("the named javac runs");
    assert!(
        output.status.success(),
        "javac rejected the presented text:\n{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        source
    );
}

fn run(classpath: &Path, main_class: &str) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(classpath)
        .arg(main_class)
        .output()
        .expect("the installed JVM runs");
    assert!(
        output.status.success(),
        "the stripped text did not run:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the fixture prints text")
}

/// The anchor's stripped text compiled and run by both compiler legs this machine holds, beside the
/// fixture's own class — one answer per compilation, all of them the same.
#[test]
#[ignore = "needs a JDK on PATH: it strips the class-source comment lines, compiles the text with \
            the installed javac --release 8 (and with a real javac 8 when one is present) and runs \
            both, comparing every answer with the fixture's own class files"]
fn every_stripped_anchor_answers_what_its_class_answers() {
    eprintln!(
        "the real javac 8 leg is {}",
        javac8().map_or_else(
            || "absent on this machine".to_owned(),
            |path| path.display().to_string()
        )
    );
    for leg in LEGS {
        let snapshot = open(&leg.fixture("OP"));
        let report = class_source_of(&snapshot, "OP");
        let presented = comment_lines_dropped(&report.text).join("\n") + "\n";
        // The stripped text is a whole unit: the class the request named, with its members.
        assert!(
            presented.contains("public class OP"),
            "the strip lost the class declaration on {}",
            leg.label
        );

        let original_dir = TempDir::new(&format!("{}-original", leg.label.replace(' ', "-")));
        original_classes(leg, "OP", original_dir.path());
        let original_run = run(original_dir.path(), "OP");
        // The patrol's own recorded answer for this fixture, pinned: a fixture whose own run moved
        // is a different fixture, not a replay failure.
        assert_eq!(
            original_run, "hi/none/none/3/0/42/false/S/\n",
            "{}: the fixture's own run moved",
            leg.label
        );

        for (label, javac, release_8) in [
            (
                "the installed javac --release 8".to_owned(),
                PathBuf::from("javac"),
                true,
            ),
            match javac8() {
                Some(path) => (
                    format!("the real javac 8 at {}", path.display()),
                    path,
                    false,
                ),
                None => continue,
            },
        ] {
            let directory = TempDir::new(&format!(
                "{}-{}",
                leg.label.replace(' ', "-"),
                javac.display()
            ));
            compile_with(&javac, release_8, directory.path(), "OP", &presented);
            let answer = run(directory.path(), "OP");
            assert_eq!(
                answer, original_run,
                "{} / {label}: the stripped text diverges",
                leg.label
            );
        }
    }
}
