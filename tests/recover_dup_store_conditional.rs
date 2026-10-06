//! `recover-dup-store-conditional` in one frozen target: the **dup-store dance** on **both**
//! compiler legs (javac 23.0.1 `--release 8` and real javac 8, Corretto 1.8.0_432, from the same
//! sources).
//!
//! javac emits `<expr>; dup; istore x; <consumer>` whenever an assignment's value is consumed —
//! `return (x = x + 1) > 0;` is `iload_0; iconst_1; iadd; dup@3; istore_0; ifle`, and
//! `while ((line = read()) != null)` is `invokestatic; dup@5; astore_1; ifnull`. The copy has
//! exactly two readers: the store and the consumer. The change presents the dance by the **store
//! target's own readers**:
//!
//! * no instruction reads the value the store wrote (directly or through the merges it is carried
//!   into) — the assignment is unobservable, the store is eliminated and the consumer's text is the
//!   value itself: `return x + 1 > 0;`, `while (read() != null)`. This is jadx's own elimination;
//! * the target is read later, and the test is the structure's own (a **parameter** target is the
//!   shape this change newly admits): the assignment is written as its own statement in front of the
//!   structure and the test reads the local — `x = x + 1; if (x > 0) { … }`.
//!
//! The texts the copy family's assignment rule already presented (a **local** target with a later
//! reader, `recover-proved-local-assignments-in-conditions`' in-place assignment expression) stay
//! byte-identical: that change's own replay pins `(local1 = arg0.length()) > 5`, so this change does
//! not move it. A dance in a **loop's** own test whose target is read later stays refused — the
//! split cannot be written in front of a re-evaluated condition, and the assignment expression is
//! that rule's position, not this one's.
//!
//! The fixtures are this change's own `DS` (the int form: the eliminated parameter and local forms,
//! the split form, the eliminated short-circuit form, and the local live control), `REF` (the
//! reference form: the classic `while ((line = read()) != null)`) and `NEG` (the two refusals: a
//! loop test whose target is read later, and a dance inside a short-circuit chain the chain proof
//! refuses). The multi-reader negative is the copy family's own frozen control — CF-06's
//! `ExtraCopy.class`, whose first copy feeds another copy rather than the test — read from the
//! fixture that owns it.
//!
//! The tests pin the presented texts, keep both refusals verbatim, and the ignored replay strips the
//! presentations the way the patrols' own stripped sources were made (comment lines dropped),
//! compiles each anchor with the installed `javac --release 8` and, when a real javac 8 is present,
//! with that one too, runs both under `-Xverify:all` and compares every answer with the fixture's
//! own class files.

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

    /// One fixture family's container.
    fn fixture(&self, class: &str) -> Vec<u8> {
        let family = self.family(class);
        let entries: Vec<(&[u8], &[u8])> = family
            .iter()
            .map(|(name, bytes)| (name.as_bytes(), *bytes))
            .collect();
        zip_of(&entries)
    }
}

/// javac 23.0.1, `javac --release 8 -g -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "DS.class",
        include_bytes!("fixtures/recover-dup-store-conditional/v8/DS.class"),
    ),
    (
        "REF.class",
        include_bytes!("fixtures/recover-dup-store-conditional/v8/REF.class"),
    ),
    (
        "NEG.class",
        include_bytes!("fixtures/recover-dup-store-conditional/v8/NEG.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -g -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "DS.class",
        include_bytes!("fixtures/recover-dup-store-conditional/v8-javac8/DS.class"),
    ),
    (
        "REF.class",
        include_bytes!("fixtures/recover-dup-store-conditional/v8-javac8/REF.class"),
    ),
    (
        "NEG.class",
        include_bytes!("fixtures/recover-dup-store-conditional/v8-javac8/NEG.class"),
    ),
];

/// The copy family's own frozen multi-reader control (CF-06's `ExtraCopy.class`: BCI 13's
/// `iconst_5` patched to `dup`, so the first copy feeds another copy rather than the test). It is
/// read from the fixture that owns it — `tests/fixtures/p3-inner-assignment/`, SHA-pinned in that
/// fixture's README — and presented under the name it declares.
const MULTI_READER: &[u8] = include_bytes!("fixtures/p3-inner-assignment/ExtraCopy.class");
const MULTI_READER_CLASS: &str = "cf06/InnerAssignCases";

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
const CLASSES: &[&str] = &["DS", "REF", "NEG"];

// -------------------------------------------------------------------------------------------
// The presented texts.
// -------------------------------------------------------------------------------------------

/// `DS.condAssign`/`deadLocal`/`deadChain` — the **eliminated** form: the store target has no
/// reader, so the store is dropped and the consumer's text is the value the copy duplicated.
const DS_ELIMINATED: &[&str] = &[
    "        return x + 1 > 0;",
    "        if (x + 1 > 0) {",
    "        boolean b = x + 1 > 0 && x > 10;",
];

/// `DS.split` — the **split** form: the target is read later and the test is the structure's own,
/// so the assignment is a statement of its own in front of the structure.
const DS_SPLIT: &[&str] = &[
    "        x = x + 1;",
    "        if (x > 0) {",
    "            return x;",
];

/// `DS.splitLocal` — the local live control: the copy family's assignment rule already presents it
/// as the in-place assignment expression, and this change leaves that text byte-identical.
const DS_LOCAL_CONTROL: &[&str] = &["        if ((y = x + 1) > 0) {", "            return y;"];

/// `REF.deadLine` — the reference form: the classic `while ((line = read()) != null)` with a dead
/// `line`, presented as the eliminated condition.
const REF_ELIMINATED: &[&str] = &[
    "        while (read() != null) {",
    "            n = n + 1;",
    "        return n;",
];

/// `NEG.liveLine` — a **loop** test whose target is read later: the split cannot be written in front
/// of a re-evaluated condition, so the shape keeps the refusal it had.
const NEG_LIVE_LINE: &str = "// local 0 crosses a quoted fallback region";

/// `NEG.shortChain` — a dance inside a short-circuit chain the chain proof refuses (pre-existing:
/// the same refusal the shape had before this change).
const NEG_SHORT_CHAIN: &str =
    "// the short-circuit chain from BCI 5 through 11 reaches a shared value consumer at BCI 19";

/// The multi-reader control keeps the copy family's own refusal, verbatim.
const MULTI_READER_REFUSAL: &str = "// the copy at BCI 11 has no proved local assignment";

/// The one diagnostic this change exists to remove from the anchors.
const COPY_REFUSAL: &str = "has no proved local assignment";

/// The recovering methods that must present every instruction they have.
const NO_QUOTE_METHODS: &[(&str, &str)] = &[
    ("DS", "condAssign(I)Z"),
    ("DS", "deadLocal(I)I"),
    ("DS", "split(I)I"),
    ("DS", "splitLocal(I)I"),
    ("DS", "deadChain(I)Z"),
    ("REF", "deadLine()I"),
    ("REF", "read()Ljava/lang/String;"),
];

// -------------------------------------------------------------------------------------------
// The class-source surface.
// -------------------------------------------------------------------------------------------

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
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition: {other:?}"),
    }
}

/// One class's presentation, with jarde's own self-header asserted **before** anything is counted
/// in it: a render of nothing is not a render.
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

/// Every bytecode index one presentation quotes.
fn quoted_bcis(text: &str) -> Vec<u32> {
    let mut quoted = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("// @bytecode ") else {
            continue;
        };
        for token in rest.split_whitespace() {
            if let Ok(bci) = token.parse::<u32>() {
                quoted.push(bci);
            }
        }
    }
    quoted
}

/// The body text of one method, from its declaration line to the closing brace.
fn method_body<'a>(text: &'a str, signature: &str) -> &'a str {
    let start = text
        .find(signature)
        .unwrap_or_else(|| panic!("the presentation states `{signature}`:\n{text}"));
    let rest = &text[start..];
    let end = rest
        .find("\n    }")
        .unwrap_or_else(|| panic!("the method `{signature}` closes:\n{text}"));
    &rest[..end]
}

// -------------------------------------------------------------------------------------------
// The anchors: both forms recover, and the two presentations are the ones the design states.
// -------------------------------------------------------------------------------------------

/// Every eliminated and split anchor is written on both legs.
#[test]
fn the_dup_store_anchors_write_the_eliminated_and_split_forms_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("DS"));
        let report = presented(&snapshot, "DS");
        for anchor in DS_ELIMINATED.iter().chain(DS_SPLIT).chain(DS_LOCAL_CONTROL) {
            assert!(
                report.text.contains(anchor),
                "`DS` on {} lost the anchor {anchor:?}:\n{}",
                leg.label,
                report.text
            );
        }
        // The eliminated form writes no assignment at all, and the split form writes it as its own
        // statement — never as the in-place expression the *parameter* target would otherwise get.
        assert!(
            !report.text.contains("(x = x + 1)"),
            "`DS` on {} wrote an assignment expression this slice does not present:\n{}",
            leg.label,
            report.text
        );
        assert!(
            report.text.contains("        if ((y = x + 1) > 0) {"),
            "`DS.splitLocal` on {} lost the assignment rule's own text:\n{}",
            leg.label,
            report.text
        );

        let snapshot = open(&leg.fixture("REF"));
        let report = presented(&snapshot, "REF");
        for anchor in REF_ELIMINATED {
            assert!(
                report.text.contains(anchor),
                "`REF` on {} lost the reference-form anchor {anchor:?}:\n{}",
                leg.label,
                report.text
            );
        }
    }
}

/// The recovering methods present every instruction they have: an anchor that still quotes one
/// would be a partial recovery counted as one.
#[test]
fn no_recovering_method_keeps_a_quote() {
    for leg in LEGS {
        for &(class, signature) in NO_QUOTE_METHODS {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let body = method_body(&report.text, signature);
            let quoted = quoted_bcis(body);
            assert!(
                quoted.is_empty(),
                "`{class}.{signature}` on {} still quotes {quoted:?}:\n{body}",
                leg.label
            );
            assert!(
                !body.contains("not recovered"),
                "`{class}.{signature}` on {} is not a whole recovery:\n{body}",
                leg.label
            );
            assert!(
                !body.contains(COPY_REFUSAL),
                "`{class}.{signature}` on {} still carries the copy family's refusal:\n{body}",
                leg.label
            );
        }
    }
}

// -------------------------------------------------------------------------------------------
// The negatives keep their refusals.
// -------------------------------------------------------------------------------------------

/// The loop-test shape with a later reader and the short-circuit chain keep the refusals they had,
/// and the copy family's own multi-reader control keeps its own.
#[test]
fn the_negatives_keep_their_refusals_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("NEG"));
        let report = presented(&snapshot, "NEG");
        for (method, signature, refusal) in [
            ("liveLine", "liveLine()I", NEG_LIVE_LINE),
            ("shortChain", "shortChain(I)Z", NEG_SHORT_CHAIN),
        ] {
            assert!(
                report.text.contains(refusal),
                "`NEG.{method}` on {} lost the refusal {refusal:?}:\n{}",
                leg.label,
                report.text
            );
            assert!(
                report.text.contains(&format!(
                    "not recovered: the recovery run for `{signature}` produced no statement"
                )),
                "`NEG.{method}` on {} is not quoted whole:\n{}",
                leg.label,
                report.text
            );
        }
        // The eliminated form is a *presentation*, never a licence: no refused method may write the
        // store's target as if the assignment had been presented.
        for line in report.text.lines().filter(|line| !line.contains("//")) {
            assert!(
                !line.contains("= x + 1") && !line.contains("= read()"),
                "`NEG` on {} presented an assignment its own refusal does not cover: {line:?}",
                leg.label
            );
        }

        // The multi-reader control is the copy family's own frozen shape: it must keep the refusal
        // this change's two-reader criterion exists for.
        let jar = zip_of(&[(b"cf06/InnerAssignCases.class", MULTI_READER)]);
        let snapshot = open(&jar);
        let report = presented(&snapshot, MULTI_READER_CLASS);
        assert!(
            report.text.contains(MULTI_READER_REFUSAL),
            "the multi-reader control on {} lost its refusal:\n{}",
            leg.label,
            report.text
        );
        assert!(
            !report.text.contains("local1 ="),
            "the multi-reader control on {} was presented:\n{}",
            leg.label,
            report.text
        );
    }
}

// -------------------------------------------------------------------------------------------
// The replay: both compiler legs, real execution, the fixture's own answers.
// -------------------------------------------------------------------------------------------

/// A scratch directory under the target tree.
fn scratch(label: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let ordinal = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("jarde-dup-store-{label}-{stamp}-{ordinal}"));
    fs::create_dir_all(&path).expect("the scratch directory is created");
    path
}

/// The comment-stripped text one class's presentation becomes.
fn stripped(report: &ClassSourceReport) -> String {
    report
        .text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Run one committed fixture under `-Xverify:all`, answering its standard output and exit status.
fn run_class(runner: &str, dir: &Path, class: &str) -> String {
    let run = Command::new(runner)
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(class)
        .output()
        .expect("the JVM runs");
    format!(
        "{}\nstatus={}",
        String::from_utf8_lossy(&run.stdout).trim_end(),
        run.status.code().unwrap_or(-1)
    )
}

/// Compile one stripped presentation and run it under `-Xverify:all`, answering its standard
/// output and exit status.
fn compile_and_run(compiler: &str, runner: &str, release: bool, dir: &Path, class: &str) -> String {
    let source = dir.join(format!("{class}.java"));
    let mut command = Command::new(compiler);
    if release {
        command.arg("--release").arg("8");
    }
    let output = command
        .arg("-nowarn")
        .arg("-d")
        .arg(dir)
        .arg(&source)
        .output()
        .expect("the compiler runs");
    assert!(
        output.status.success(),
        "{class}: javac failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    run_class(runner, dir, class)
}

/// The fixture's own class files, extracted once.
fn original_dir(label: &str, leg: &Leg, class: &str) -> PathBuf {
    let dir = scratch(&format!("original-{label}-{class}"));
    for (name, bytes) in leg.family(class) {
        fs::write(dir.join(&name), bytes).expect("the fixture class is written");
    }
    dir
}

#[test]
#[ignore = "compiles and runs the recovered text; needs the installed JDK (and javac 8 for the second leg)"]
fn the_recovered_text_compiles_and_runs_identically_on_both_legs() {
    let javac8 =
        Path::new("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac");
    let java8 =
        Path::new("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java");
    for leg in LEGS {
        for &class in CLASSES {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let text = stripped(&report);
            // `NEG` is the negatives' own class: every body it holds is quoted, so its stripped text
            // is the *safe* form the soundness invariant asks for — a method whose body a reader
            // cannot compile, never one that compiles and behaves differently. The refusal texts are
            // pinned by the tests above; the replay's job for it is exactly that it does not compile.
            if class == "NEG" {
                let work = scratch(&format!("negative-{class}"));
                fs::write(work.join(format!("{class}.java")), &text)
                    .expect("the presentation is written");
                let compiled = Command::new("/usr/bin/javac")
                    .arg("--release")
                    .arg("8")
                    .arg("-nowarn")
                    .arg("-d")
                    .arg(&work)
                    .arg(work.join(format!("{class}.java")))
                    .output()
                    .expect("the compiler runs");
                assert!(
                    !compiled.status.success(),
                    "`{class}` on {} is quoted whole and its stripped text must not compile:\n{text}",
                    leg.label
                );
                continue;
            }
            assert!(
                !text.contains("jarde_refused_body"),
                "`{class}` on {} still holds a refused body marker:\n{text}",
                leg.label
            );
            let want_dir = original_dir(leg.label, leg, class);
            let want = run_class("/usr/bin/java", &want_dir, class);
            let work = scratch(&format!("recovered-{class}"));
            fs::write(work.join(format!("{class}.java")), &text)
                .expect("the presentation is written");
            let got = compile_and_run("/usr/bin/javac", "/usr/bin/java", true, &work, class);
            assert_eq!(
                got, want,
                "`{class}` on {} answers differently after the round trip:\n{text}",
                leg.label
            );
            if javac8.is_file() {
                let work8 = scratch(&format!("recovered-8-{class}"));
                fs::write(work8.join(format!("{class}.java")), &text)
                    .expect("the presentation is written");
                let got8 = compile_and_run(
                    javac8.to_str().expect("the path is UTF-8"),
                    java8.to_str().expect("the path is UTF-8"),
                    false,
                    &work8,
                    class,
                );
                assert_eq!(
                    got8, want,
                    "`{class}` on {} answers differently under javac 8:\n{text}",
                    leg.label
                );
            }
        }
    }
}
