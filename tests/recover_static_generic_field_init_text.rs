//! `recover-static-generic-field-init-text` in one frozen target: the generic-static-field-init
//! patrol's `MN`/`RG` anchors and this change's `SG`/`P1` controls, on **both** compiler legs
//! (javac 23.0.1 `--release 8` and real javac 8, Corretto 1.8.0_432, from the same sources).
//!
//! The patrol recorded that a static generic field's initializer is rendered as uncompilable text:
//! `static Hold<String> f1 = new Hold<>("a");` came out as `static Hold f1 = new MN$Holdava.lang.Object) "a");`
//! — the type arguments and a `(` gone, the class name entangled with the argument cast. The
//! change's task 1.1 located the splice: the static member fold re-spells every reference to a
//! folded member in a member's own text ([`project_static_fold_owner_texts`]), and its field loop
//! rewrote the declaration **in place** while still replacing at the byte offsets the scan had
//! stated on the *unrewritten* declaration. From the second occurrence on, every replacement landed
//! `prefix_delta` bytes late and ate the text that followed the name it meant to re-spell: `MN$Hold`
//! is three bytes shorter than the `Hold` it is spelled as, so `((j` of `((java.lang.Object) "a")`
//! disappeared — the missing `(` and the `j` of the patrol's verbatim sample.
//!
//! The trigger is not the field's `Signature` refusal (`field_generic_body_unproved`, whose comment
//! this change leaves verbatim): the declaration the fold re-spells carries the initializer the
//! `<clinit>` proof inlined into it, so a refused field's declaration names the folded class twice —
//! once as its own pool-spelled type, once inside the initializer. `P1` (a **non-generic** nested
//! class, no `Signature` anywhere) is the control for that reading: at the parent commit `dccd21c3`
//! it renders `static Box b = new P1$Box;`, the whole argument list eaten.
//!
//! The tests below pin the fixed declarations whole, assert that no non-comment line of a fold
//! render keeps a `$`-pool name (the fold's own contract: every reference to a folded member is
//! re-spelled), and keep the patrol's `f3` instance-field degradation and the refusal markers byte
//! for byte. The ignored replay strips the presentations the way the patrol's own stripped sources
//! were made (comment lines dropped), compiles each anchor with the installed `javac --release 8`
//! and, when a real javac 8 is present, with that one too, runs both under `-Xverify:all` and
//! compares every answer with the fixture's own class files.
//!
//! `SG` and `P1` compile as whole units, so their replay needs no edit beyond the strip. `MN` and
//! `RG` do **not**: with the parse error gone their only remaining error is the nested `Hold<T>`'s
//! own erasure pair (`T v;` projected beside `Hold(java.lang.Object arg1)`, whose `Signature` the
//! projection domain refuses) — a pre-existing open item of that domain, present in the class's
//! single-class presentation too, which this change does not touch. Their replay therefore
//! hand-corrects that one pair (`Hold(java.lang.Object arg1)` → `Hold(T arg1)`) and compares the
//! answer with the fixture's own run, so the change's field-initializer text is measured end to end
//! without claiming the projection domain's gap.
//!
//! `project_static_fold_owner_texts` is private to `src/facade.rs`; this file names it in prose only.

use jarde::class_source::ClassSourceReport;
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
// The fixtures and the two compiler legs.
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

    /// One fixture family's container: the posture the patrol's broken text was recorded in.
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
        "MN.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8/MN.class"),
    ),
    (
        "MN$Hold.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8/MN$Hold.class"),
    ),
    (
        "P1.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8/P1.class"),
    ),
    (
        "P1$Box.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8/P1$Box.class"),
    ),
    (
        "RG.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8/RG.class"),
    ),
    (
        "RG$Hold.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8/RG$Hold.class"),
    ),
    (
        "SG.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8/SG.class"),
    ),
    (
        "SG$Hold.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8/SG$Hold.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "MN.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8-javac8/MN.class"),
    ),
    (
        "MN$Hold.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8-javac8/MN$Hold.class"),
    ),
    (
        "P1.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8-javac8/P1.class"),
    ),
    (
        "P1$Box.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8-javac8/P1$Box.class"),
    ),
    (
        "RG.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8-javac8/RG.class"),
    ),
    (
        "RG$Hold.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8-javac8/RG$Hold.class"),
    ),
    (
        "SG.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8-javac8/SG.class"),
    ),
    (
        "SG$Hold.class",
        include_bytes!("fixtures/recover-static-generic-field-init-text/v8-javac8/SG$Hold.class"),
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

/// Every class this change pins, in the order the tests read them.
const CLASSES: &[&str] = &["MN", "RG", "SG", "P1"];

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

/// One class's presentation, with jarde's own self-header asserted **before** anything is counted
/// in it: a render of nothing is not a render, and counting broken text in one would be a false
/// zero.
fn presented(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let report = class_source_of(snapshot, name);
    assert!(
        report
            .text
            .starts_with(&format!("// jarde: presentation of `{name}`")),
        "the render of `{name}` carries jarde's own self-header:\n{}",
        report.text
    );
    report
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

/// `MN.f1`/`MN.f2`: the patrol's broken declarations, now the bare-type correct form the instance
/// field `f3` has always rendered — the constructor's erased descriptor, cast written out, the
/// folded class spelled as the source nesting its declaration states. Both legs answer byte for
/// byte.
const MN_F1: &str = "    static Hold f1 = new Hold((java.lang.Object) \"a\");\n";
const MN_F2: &str = "    static Hold f2 = new Hold((java.lang.Object) \"b\");\n";

/// `MN.f3`: the instance field's own degradation path, byte for byte what it was before the change
/// — its initializer stays in the constructor and never enters the declaration.
const MN_F3: &str = "    Hold f3;\n";
const MN_F3_CTOR: &str =
    "        this.f3 = new Hold((java.lang.Object) java.lang.Integer.valueOf(5));\n";

/// `RG.field`: the patrol's second anchor, same shape, same fix.
const RG_FIELD: &str = "    static Hold field = new Hold((java.lang.Object) \"sf\");\n";

/// `RG.nested`: the nested diamond the patrol recorded as `new RG$Holdava.lang.Object) new
/// RG$HolHold.lang.Object) java.lang.Integer.valueOf(5)))` — two argument prefixes eaten, one `)`
/// too many. Re-spelled, both occurrences land where the scan stated them.
const RG_NESTED: &str = "    static Hold nested = new Hold((java.lang.Object) new Hold((java.lang.Object) java.lang.Integer.valueOf(5)));\n";

/// `RG.consume`'s local and `RG.newInner`'s returned construction: the method channels were never
/// broken (they rewrite from the last edit backwards), and they stay exactly as they were.
const RG_LOCAL: &str = "        Hold local3 = new RG().newInner();\n";
const RG_NEW_INNER: &str = "        return new Hold((java.lang.Object) \"in\");\n";

/// `SG` — this change's compile-clean anchor: the same refused static generic fields as `MN`, over
/// a nested `Hold<T>` whose members carry no `Signature` at all.
const SG_F1: &str = "    static Hold f1 = new Hold((java.lang.Object) \"a\");\n";
const SG_F2: &str = "    static Hold f2 = new Hold((java.lang.Object) \"b\");\n";
const SG_F3: &str = "    Hold f3;\n";

/// `P1` — the mechanism's non-generic control: `static Box b = new Box(1);` at the parent commit
/// rendered as `static Box b = new P1$Box;`.
const P1_B: &str = "    static Box b = new Box(1);\n";

/// The projection refusal the change keeps verbatim: the field `Signature` is still refused, and
/// the comment still states why. The initializer text moves; the refusal does not.
const MN_REFUSAL_F1: &str = "// jarde: field Signature projection refused for `f1LMN$Hold;`: unsupported (field_generic_body_unproved): a same-class Fieldref names this field and descriptor";
const MN_REFUSAL_F2: &str = "// jarde: field Signature projection refused for `f2LMN$Hold;`: unsupported (field_generic_body_unproved): a same-class Fieldref names this field and descriptor";
const MN_REFUSAL_F3: &str = "// jarde: field Signature projection refused for `f3LMN$Hold;`: unsupported (field_generic_body_unproved): a same-class Fieldref names this field and descriptor";
const RG_REFUSAL_FIELD: &str = "// jarde: field Signature projection refused for `fieldLRG$Hold;`: unsupported (field_generic_body_unproved): a same-class Fieldref names this field and descriptor";
const RG_REFUSAL_NESTED: &str = "// jarde: field Signature projection refused for `nestedLRG$Hold;`: unsupported (field_generic_body_unproved): a same-class Fieldref names this field and descriptor";
const RG_REFUSAL_NAMES: &str = "// jarde: field Signature projection refused for `namesLjava/util/List;`: unsupported (field_generic_body_unproved): a same-class Fieldref names this field and descriptor";

/// Every residue the patrol's samples carried: the `Holdava` entanglement the acceptance names,
/// and the nested form's `Hold.lang.Object` fragment.
const BROKEN_RESIDUES: &[&str] = &["Holdava", "Hold.lang.Object"];

/// The declaration lines this change pins, class by class.
const PINNED_DECLARATIONS: &[(&str, &[&str])] = &[
    ("MN", &[MN_F1, MN_F2, MN_F3]),
    ("RG", &[RG_FIELD, RG_NESTED]),
    ("SG", &[SG_F1, SG_F2, SG_F3]),
    ("P1", &[P1_B]),
];

// -------------------------------------------------------------------------------------------
// The broken text is gone, and the fold's own spelling contract holds.
// -------------------------------------------------------------------------------------------

/// Every fixture's presentation is legal text on both legs: the pinned declarations are the ones
/// the scan meant, no residue of the broken forms survives, and no non-comment line keeps a
/// `$`-pool name — the fold re-spells every reference to a folded member, so a pool name left in
/// the unit is exactly the signature the eaten text left behind.
#[test]
fn the_broken_text_is_gone_on_both_legs() {
    for leg in LEGS {
        for &class in CLASSES {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let (_, pinned) = PINNED_DECLARATIONS
                .iter()
                .find(|(name, _)| *name == class)
                .expect("every fixture states its own declarations");
            for &declaration in *pinned {
                assert!(
                    report.text.contains(declaration),
                    "`{class}` on {} lost the declaration {declaration:?}:\n{}",
                    leg.label,
                    report.text
                );
            }
            for residue in BROKEN_RESIDUES {
                assert!(
                    !report.text.contains(residue),
                    "`{class}` on {} keeps the broken-text residue {residue:?}:\n{}",
                    leg.label,
                    report.text
                );
            }
            for line in report
                .text
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
            {
                assert!(
                    !line.contains('$'),
                    "`{class}` on {} keeps a folded member's pool name in its unit: {line:?}",
                    leg.label
                );
            }
        }
    }
}

/// The change is a text repair and nothing else: the field `Signature` refusals keep their
/// verbatim comments, the instance field's degradation path is untouched, and the nested form's
/// method channels — which were never broken — stay exactly as they were.
#[test]
fn the_refusals_and_the_instance_field_control_survive() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("MN"));
        let report = presented(&snapshot, "MN");
        for refusal in [MN_REFUSAL_F1, MN_REFUSAL_F2, MN_REFUSAL_F3] {
            assert!(
                report.text.contains(refusal),
                "`MN` on {} lost the refusal comment {refusal:?}:\n{}",
                leg.label,
                report.text
            );
        }
        assert!(
            report.text.contains(MN_F3) && report.text.contains(MN_F3_CTOR),
            "`MN.f3`'s instance-field degradation moved on {}:\n{}",
            leg.label,
            report.text
        );

        let snapshot = open(&leg.fixture("RG"));
        let report = presented(&snapshot, "RG");
        for refusal in [RG_REFUSAL_FIELD, RG_REFUSAL_NESTED, RG_REFUSAL_NAMES] {
            assert!(
                report.text.contains(refusal),
                "`RG` on {} lost the refusal comment {refusal:?}:\n{}",
                leg.label,
                report.text
            );
        }
        for line in [RG_LOCAL, RG_NEW_INNER] {
            assert!(
                report.text.contains(line),
                "`RG` on {} lost the method text {line:?}:\n{}",
                leg.label,
                report.text
            );
        }
    }
}

/// The parent commit's own renders, pinned as the negatives: the two residues the acceptance
/// names, and the two shapes the reading's scope note records (`P1`'s eaten argument list has no
/// `Holdava` in it, which is why the compile-and-run legs below are what settles it).
#[test]
fn the_parent_commit_texts_are_the_broken_forms_this_change_removes() {
    for leg in LEGS {
        for &class in CLASSES {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            // The declarations the parent commit rendered are absent, and the declarations this
            // change states are present: the pair is what makes the fix falsifiable.
            let broken = match class {
                "MN" => "    static Hold f1 = new MN$Holdava.lang.Object) \"a\");",
                "RG" => {
                    "    static Hold nested = new RG$Holdava.lang.Object) new RG$HolHold.lang.Object) java.lang.Integer.valueOf(5)));"
                }
                "SG" => "    static Hold f1 = new SG$Holdava.lang.Object) \"a\");",
                "P1" => "    static Box b = new P1$Box;",
                other => panic!("no recorded broken form for `{other}`"),
            };
            assert!(
                !report.text.contains(broken),
                "`{class}` on {} still renders the parent commit's broken declaration:\n{}",
                leg.label,
                report.text
            );
        }
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
            "jarde-static-generic-field-init-{label}-{}-{nonce}-{sequence}",
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

/// The two compilations this machine holds: the installed javac under `--release 8`, and a real
/// javac 8 when one is present.
fn compilers() -> Vec<(String, PathBuf, bool)> {
    let mut compilers = vec![(
        "the installed javac --release 8".to_owned(),
        PathBuf::from("javac"),
        true,
    )];
    if let Some(path) = javac8() {
        compilers.push((
            format!("the real javac 8 at {}", path.display()),
            path,
            false,
        ));
    }
    compilers
}

/// The nested `Hold<T>`'s own erasure pair, hand-corrected for the two anchors that carry it: the
/// field `Signature` is projected (`T v;`) while the constructor's is refused
/// (`ordinary_generic_source_unproved`), so `this.v = arg1;` is `Object` into `T`. That pair is the
/// projection domain's own open item — present in the class's single-class presentation too — and
/// this change does not touch it; correcting it here is what makes the change's own text
/// measurable end to end.
fn with_the_projection_gap_corrected(text: &str) -> String {
    let corrected = text.replace(
        "        Hold(java.lang.Object arg1) {\n",
        "        Hold(T arg1) {\n",
    );
    assert!(
        corrected != text,
        "the anchor's nested constructor carries the erasure pair this replay corrects:\n{text}"
    );
    corrected
}

/// Every stripped anchor answers what its own class answers, on both compiler legs this machine
/// holds: `SG`'s and `P1`'s whole-class presentations compile as one unit and are compared with
/// the fixture's own run, and `MN`/`RG` are compared the same way after the one out-of-scope
/// erasure pair is hand-corrected. `MN`'s own class prints `ab5` — the values `SG` prints too, so
/// the anchor's behavior is measured on the same string through a class that compiles whole.
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
        // The fixtures' own classes, run first: their answers are what every replay is compared
        // with, and a fixture whose own run moved is a different fixture.
        let original_dir = TempDir::new(&format!("{}-original", leg.label.replace(' ', "-")));
        for &class in CLASSES {
            original_classes(leg, class, original_dir.path());
        }
        let answers: Vec<(&str, String)> = CLASSES
            .iter()
            .map(|class| (*class, run(original_dir.path(), class)))
            .collect();
        let answer_of = |class: &str| -> String {
            answers
                .iter()
                .find(|(name, _)| *name == class)
                .map(|(_, printed)| printed.clone())
                .expect("the fixture's own class ran")
        };
        assert_eq!(
            answer_of("MN"),
            "ab5\n",
            "{}: `MN`'s own run moved",
            leg.label
        );
        assert_eq!(
            answer_of("RG"),
            "y7insf5\n2\n",
            "{}: `RG`'s own run moved",
            leg.label
        );
        assert_eq!(
            answer_of("SG"),
            "ab5\n",
            "{}: `SG`'s own run moved",
            leg.label
        );
        assert_eq!(
            answer_of("P1"),
            "1\n",
            "{}: `P1`'s own run moved",
            leg.label
        );

        for &class in CLASSES {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let stripped = comment_lines_dropped(&report.text).join("\n") + "\n";
            assert!(
                stripped.contains(&format!("public class {class}")),
                "the strip lost the class declaration on {}",
                leg.label
            );
            let unit = match class {
                // The two anchors whose nested constructor keeps the projection domain's erasure
                // pair; the two controls compile as they are rendered.
                "MN" | "RG" => with_the_projection_gap_corrected(&stripped),
                _ => stripped,
            };
            for (label, javac, release_8) in compilers() {
                let directory = TempDir::new(&format!(
                    "{}-{class}-{}",
                    leg.label.replace(' ', "-"),
                    javac.display()
                ));
                compile_with(&javac, release_8, directory.path(), class, &unit);
                assert_eq!(
                    run(directory.path(), class),
                    answer_of(class),
                    "{} / {label}: `{class}`'s stripped text diverges",
                    leg.label
                );
            }
        }
    }
}
