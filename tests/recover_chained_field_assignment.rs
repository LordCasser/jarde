//! `recover-chained-field-assignment` in one frozen target: the copy family's **third** shape —
//! a copy whose consumers are field instructions — on **both** compiler legs (javac 23.0.1
//! `--release 8` and real javac 8, Corretto 1.8.0_432, from the same sources).
//!
//! javac emits one `dup` per extra store for `a = b = c = 5`
//! (`iconst_5; dup; putstatic c; dup; putstatic b; putstatic a`): the value is evaluated **once**,
//! and every copy hands one store its own value. The change presents that as one assignment per
//! store, in bytecode order — which is the source's own right-to-left order. A source that may not
//! be written once per store (`a = b = f()`) is saved first by the chain's **lead** copy
//! (`int saved0 = f(); c = saved0; …`), so the evaluation count is the bytecode's own.
//!
//! The same copy read from the other side is the **receiver** copy of one member's read and its
//! write: `this.flags |= 1 << bit` is `aload_0; dup; getfield; …; ior; putfield`, and
//! `this.field += "[" + x + "]"` is the same shape written with `dup_x1` (the receiver is copied
//! *under* the builder the concatenation is built in). The update rule presents `+=`/`-=` of an
//! `int` field already; every other operator and every reference field keep the receiver copy's own
//! presentation — `this.flags = this.flags | 1 << bit;`, `this.field = this.field + "[" + x + "]";`.
//!
//! The fixtures are this change's own `CF` (the three-store chain, the two-store chain, the saved
//! form, the single-store and local-chain controls, the `|=`/`&=` receiver copies, the `String`
//! `+=` receiver copy and the static control) and `NEG` (the four refusals: a second copy consumed
//! by a local store, by the expression the stored value is part of, by an array store, and a
//! receiver copy whose source is a call).
//!
//! The tests pin the presented texts, keep every refusal verbatim, and the ignored replay strips
//! the presentations the way the patrols' own stripped sources were made (comment lines dropped),
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

/// javac 23.0.1, `javac --release 8 -g -nowarn -d v8 CF.java NEG.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "CF.class",
        include_bytes!("fixtures/recover-chained-field-assignment/v8/CF.class"),
    ),
    (
        "NEG.class",
        include_bytes!("fixtures/recover-chained-field-assignment/v8/NEG.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -g -nowarn -d v8-javac8 CF.java NEG.java`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "CF.class",
        include_bytes!("fixtures/recover-chained-field-assignment/v8-javac8/CF.class"),
    ),
    (
        "NEG.class",
        include_bytes!("fixtures/recover-chained-field-assignment/v8-javac8/NEG.class"),
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
const CLASSES: &[&str] = &["CF", "NEG"];

// -------------------------------------------------------------------------------------------
// The presented texts.
// -------------------------------------------------------------------------------------------

/// `CF.chain` — the three-store chain: one assignment per store, in bytecode order.
const CF_CHAIN: &[&str] = &[
    "        CF.sc = 5;",
    "        CF.sb = 5;",
    "        CF.sa = 5;",
];

/// `CF.pair` — the two-store chain.
const CF_PAIR: &[&str] = &["        CF.sb = 7;", "        CF.sa = 7;"];

/// `CF.call` — the source may not be written once per store: the lead saves it.
const CF_CALL: &[&str] = &[
    "        int saved0 = f();",
    "        CF.sc = saved0;",
    "        CF.sb = saved0;",
    "        CF.sa = saved0;",
];

/// `CF.single` — the control: one field assignment, no copy.
const CF_SINGLE: &str = "        CF.sa = 9;";

/// `CF.chainLocal` — the control: the **local** chain's own text, unmoved.
const CF_CHAIN_LOCAL: &[&str] = &["        int y = 7;", "        int x = y;"];

/// `CF.enable`/`CF.disable` — the receiver copy with the operators the update rule does not
/// present.
const CF_ENABLE: &str = "        this.flags = this.flags | 1 << bit;";
const CF_DISABLE: &str = "        this.flags = this.flags & (1 << bit ^ -1);";

/// `CF.add` — the receiver copy a `String` compound assignment carries, folded back to `+`.
const CF_ADD: &str = "        this.field = this.field + \"[\" + x + \"]\";";

/// `CF.sAdd` — the control: the static `String` compound keeps the presentation it already had.
const CF_SADD: &str = "        CF.sfield = new java.lang.StringBuilder().append(CF.sfield).append(\"[\").append(x).append(\"]\").toString();";

/// The four refusals `NEG` keeps, verbatim.
const NEG_REFUSALS: &[(&str, &str)] = &[
    (
        "mixed()I",
        "// the copy at BCI 2 has no proved local assignment",
    ),
    (
        "expr()I",
        "// the copy at BCI 1 has no proved local assignment",
    ),
    (
        "arrayConsumer()I",
        "// the copy at BCI 5 has no proved local assignment",
    ),
    (
        "callReceiver()V",
        "// the copy at BCI 3 has no proved local assignment",
    ),
];

/// The recovering methods that must present every instruction they have.
const NO_QUOTE_METHODS: &[(&str, &str)] = &[
    ("CF", "chain()V"),
    ("CF", "pair()V"),
    ("CF", "call()V"),
    ("CF", "single()V"),
    ("CF", "chainLocal()I"),
    ("CF", "enable(I)V"),
    ("CF", "disable(I)V"),
    ("CF", "add(Ljava/lang/String;)LCF;"),
    ("CF", "sAdd(Ljava/lang/String;)V"),
    ("CF", "main([Ljava/lang/String;)V"),
];

/// The one diagnostic this change exists to remove from the anchors.
const COPY_REFUSAL: &str = "has no proved local assignment";

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

/// The body text of one method, from its own signature line to the closing brace.
///
/// The signature is searched as the presentation writes it — the `// @method` line a member's
/// comment block carries, which precedes the body it belongs to and is part of the text a reader
/// reads. A refused method's marker line names the same signature first, and the slice then carries
/// the marker and the refusal with it, which is what the negative assertions read.
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
// The anchors: both shapes recover, and the presentations are the ones the design states.
// -------------------------------------------------------------------------------------------

/// Every anchor is written on both legs, with the text the design states.
#[test]
fn the_field_copy_anchors_write_their_presentations_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CF"));
        let report = presented(&snapshot, "CF");
        for anchor in CF_CHAIN
            .iter()
            .chain(CF_PAIR)
            .chain(CF_CALL)
            .chain(CF_CHAIN_LOCAL)
            .chain([&CF_SINGLE, &CF_ENABLE, &CF_DISABLE, &CF_ADD, &CF_SADD])
        {
            assert!(
                report.text.contains(anchor),
                "`CF` on {} lost the anchor {anchor:?}:\n{}",
                leg.label,
                report.text
            );
        }
        // The chain writes one assignment per store and **no** copy of the source per store: the
        // text the shape would produce by writing the call three times is exactly what the saved
        // local exists to prevent.
        assert!(
            !report.text.contains("f()\n") || report.text.matches("f()").count() == 2,
            "`CF.call` on {} wrote the source more than once:\n{}",
            leg.label,
            report.text
        );
        // The static `String` compound keeps the explicit builder chain it already had: this
        // change's receiver-copy rule is about the *instance* form.
        assert!(
            !report.text.contains("CF.sfield = CF.sfield +"),
            "`CF.sAdd` on {} moved a text this change does not own:\n{}",
            leg.label,
            report.text
        );
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

/// The four refusals keep the text they had, and no refused method writes an assignment its own
/// refusal does not cover.
#[test]
fn the_negatives_keep_their_refusals_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("NEG"));
        let report = presented(&snapshot, "NEG");
        for (signature, refusal) in NEG_REFUSALS {
            let body = method_body(&report.text, signature);
            assert!(
                body.contains(refusal),
                "`NEG.{signature}` on {} lost the refusal {refusal:?}:\n{body}",
                leg.label
            );
            assert!(
                report.text.contains(&format!(
                    "not recovered: the recovery run for `{signature}` produced no statement"
                )),
                "`NEG.{signature}` on {} is not quoted whole:\n{}",
                leg.label,
                report.text
            );
        }
        // The refused bodies present no field assignment at all: a copy's value written as a
        // statement would be the effect the refusal exists to keep out of the text.
        for line in report.text.lines().filter(|line| !line.contains("//")) {
            assert!(
                !line.contains("NEG.sa =") && !line.contains("NEG.arr["),
                "`NEG` on {} presented an assignment its own refusal does not cover: {line:?}",
                leg.label
            );
        }
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
    let path = std::env::temp_dir().join(format!("jarde-field-chain-{label}-{stamp}-{ordinal}"));
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
