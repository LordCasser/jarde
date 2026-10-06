//! `recover-conditional-rhs-field-compound` in one frozen target: the field compound assignment
//! whose right-hand side is one **proved conditional materialisation**, on the frozen anchor
//! (`bi.jar` — the `boolean-loop-earlyret` patrol's own input) and on **both** compiler legs
//! (javac 23.0.1 `--release 8` and real javac 8, Corretto 1.8.0_432, from the same sources).
//!
//! `this.ok &= x > 0` lowers the comparison to a branch whose two arms push the `0`/`1` the join's
//! stack Phi merges: the field read stands in the copy's own block and the `putfield` in the block
//! the arms hand their value to. The ordinary same-block receiver-copy proof reads the copy's
//! consumers in one block and refuses that write, which is why the whole class used to be quoted
//! and its stripped text answered `false/false/false/true` where the original answered
//! `false/false/false/false`. The change states the branch's region and hands it to
//! `prove_conditional_value` — the `recover-conditional-values` two-arm proof, read **only** — so
//! the compound is written as the source form `this.ok = this.ok & (x > 0)`, inside a surviving loop
//! exactly as outside one.
//!
//! The fixtures are this change's own `RC` (the loop anchor, the same statement with no loop, the
//! `|=` operator and the integral field whose sibling operand is an `int`) and `RCN` (the three
//! refusals: a materialisation whose arm calls, two materialisations in one right-hand side, and one
//! covered by an exception table), beside the frozen `bi.jar` itself, which is the anchor input.
//!
//! The tests pin the presented texts, keep every refusal verbatim, and the ignored replay strips the
//! presentations the way the patrols' own stripped sources were made (comment lines dropped),
//! compiles each anchor with the installed `javac --release 8` and, when a real javac 8 is present,
//! with that one too, runs both under `-Xverify:all` and compares every answer with the fixture's
//! own class files.

use jarde::class_source::ClassSourceReport;
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const STORE: u16 = 0;

// -------------------------------------------------------------------------------------------
// The fixtures and the two compiler legs.
// -------------------------------------------------------------------------------------------

/// The frozen anchor input: the patrol's own `bi.jar`, byte for byte.
const FROZEN_JAR: &[u8] = include_bytes!("fixtures/recover-conditional-rhs-field-compound/bi.jar");

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

/// javac 23.0.1, `javac --release 8 -g -nowarn -d v8 BI.java RC.java RCN.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "BI.class",
        include_bytes!("fixtures/recover-conditional-rhs-field-compound/v8/BI.class"),
    ),
    (
        "RC.class",
        include_bytes!("fixtures/recover-conditional-rhs-field-compound/v8/RC.class"),
    ),
    (
        "RCN.class",
        include_bytes!("fixtures/recover-conditional-rhs-field-compound/v8/RCN.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432,
/// `javac -g -nowarn -d v8-javac8 BI.java RC.java RCN.java`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "BI.class",
        include_bytes!("fixtures/recover-conditional-rhs-field-compound/v8-javac8/BI.class"),
    ),
    (
        "RC.class",
        include_bytes!("fixtures/recover-conditional-rhs-field-compound/v8-javac8/RC.class"),
    ),
    (
        "RCN.class",
        include_bytes!("fixtures/recover-conditional-rhs-field-compound/v8-javac8/RCN.class"),
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
// The presented texts.
// -------------------------------------------------------------------------------------------

/// The frozen anchor's own statement — the whole point of the change.
const ANCHOR_FROZEN: &str = "        this.ok = this.ok & local5 > 0;";

/// The recompiled legs' statement, with the debug name the fixture's own source declares.
const ANCHOR_LEG: &str = "        this.ok = this.ok & x > 0;";

/// `RC.orEq` — the same materialisation under the other eager boolean operator.
const ANCHOR_OR: &str = "        this.ok = this.ok | x > 0;";

/// `RC.mask` — the integral field: the sibling operand is an `int`, so the arms keep the `? 1 : 0`
/// spelling the position states, and the same materialisation is admitted by the same proof.
const ANCHOR_MASK: &str = "        this.n = this.n & (x > 0 ? 1 : 0);";

/// The refusals `RCN` keeps, verbatim: the copy's own refusal, the region pass's own refusal of a
/// materialisation whose arm is not a constant, and the quoted BCIs of each shape the certificate
/// did not admit.
const NEG_REFUSALS: &[(&str, &[&str])] = &[
    (
        "armCall(Z)V",
        &[
            "// the two values joined at BCI 16 do not have a conditional Java type this run can prove",
            "// @bytecode 17 16 2",
            "// the copy at BCI 1 has no proved local assignment",
            "jarde_refused_body();",
        ],
    ),
    (
        "nested(II)V",
        &[
            "// @bytecode 25 24 2 23",
            "// the copy at BCI 1 has no proved local assignment",
            "jarde_refused_body();",
        ],
    ),
    (
        "inTry(I)V",
        &[
            "// the instruction at BCI 1 belongs to no shape this run verified",
            "// @bytecode 14 15 18",
            "// 1 live block(s) are reachable only through edges the normal-flow view leaves out: [14]",
        ],
    ),
];

/// The recovering methods that must present every instruction they have.
const NO_QUOTE_METHODS: &[(&str, &str)] = &[
    ("BI", "earlyRet([I)Z"),
    ("BI", "andEq(Z)V"),
    ("BI", "orEq(Z)V"),
    ("BI", "andUse(Z)Z"),
    ("BI", "statAnd(ZZ)Z"),
    ("BI", "main([Ljava/lang/String;)V"),
    ("RC", "earlyRet([I)Z"),
    ("RC", "plain(I)V"),
    ("RC", "orEq(I)V"),
    ("RC", "mask(I)V"),
    ("RC", "main([Ljava/lang/String;)V"),
];

/// The one diagnostic this change exists to remove from the anchors.
const COPY_REFUSAL: &str = "has no proved local assignment";

/// The anchor's own answer, which the frozen class prints and the recovered text must print too.
const FROZEN_ANSWER: &str = "false/false/false/false";

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

/// One class entry of the **frozen** jar, read back out of the committed bytes: the anchor input is
/// the patrol's own container, so the original class the replay compares against is the very one the
/// archive holds.
fn frozen_class(jar: &[u8], path: &[u8]) -> Vec<u8> {
    let archive = rawzip::ZipArchive::from_slice(jar).expect("the frozen fixture is a ZIP");
    let mut entries = archive.entries();
    while let Some(header) = entries.next_entry().expect("the entry header reads") {
        if header.file_path().as_ref() == path {
            let entry = archive
                .get_entry(header.wayfinder())
                .expect("the entry is addressable");
            let decoder = flate2::bufread::DeflateDecoder::new(entry.data());
            let mut reader = entry.verifying_reader(decoder);
            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .expect("the frozen class reads");
            return bytes;
        }
    }
    panic!("the frozen jar holds `{}`", String::from_utf8_lossy(path));
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
// The anchors: the frozen class recovers whole, and both legs write the source form.
// -------------------------------------------------------------------------------------------

/// The frozen anchor is a **whole** recovery: no quoted bytecode anywhere in the class, and the
/// compound assignment the patrol's critical line named is written as the source form.
#[test]
fn the_frozen_anchor_recovers_whole_class() {
    let snapshot = open(FROZEN_JAR);
    let report = presented(&snapshot, "BI");
    let quoted = quoted_bcis(&report.text);
    assert!(
        quoted.is_empty(),
        "the frozen `BI` still quotes {quoted:?}:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("not recovered"),
        "the frozen `BI` is not a whole recovery:\n{}",
        report.text
    );
    assert!(
        report.text.contains(ANCHOR_FROZEN),
        "the frozen `BI` lost the anchor {ANCHOR_FROZEN:?}:\n{}",
        report.text
    );
    assert!(
        !report.text.contains(COPY_REFUSAL),
        "the frozen `BI` still carries the copy family's refusal:\n{}",
        report.text
    );
}

/// Every anchor is written on both legs, with the text the design states.
#[test]
fn the_conditional_rhs_anchors_write_their_presentations_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BI"));
        let report = presented(&snapshot, "BI");
        assert!(
            report.text.contains(ANCHOR_LEG),
            "`BI` on {} lost the anchor {ANCHOR_LEG:?}:\n{}",
            leg.label,
            report.text
        );
        let snapshot = open(&leg.fixture("RC"));
        let report = presented(&snapshot, "RC");
        for anchor in [ANCHOR_LEG, ANCHOR_OR, ANCHOR_MASK] {
            assert!(
                report.text.contains(anchor),
                "`RC` on {} lost the anchor {anchor:?}:\n{}",
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
        for class in ["BI", "RC"] {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            for &(owner, signature) in NO_QUOTE_METHODS.iter().filter(|(owner, _)| *owner == class)
            {
                let body = method_body(&report.text, signature);
                let quoted = quoted_bcis(body);
                assert!(
                    quoted.is_empty(),
                    "`{owner}.{signature}` on {} still quotes {quoted:?}:\n{body}",
                    leg.label
                );
                assert!(
                    !body.contains("not recovered"),
                    "`{owner}.{signature}` on {} is not a whole recovery:\n{body}",
                    leg.label
                );
                assert!(
                    !body.contains(COPY_REFUSAL),
                    "`{owner}.{signature}` on {} still carries the copy family's refusal:\n{body}",
                    leg.label
                );
            }
        }
    }
}

// -------------------------------------------------------------------------------------------
// The negatives keep their refusals.
// -------------------------------------------------------------------------------------------

/// The three refusals keep the text they had, and no refused body writes a field assignment its own
/// refusal does not cover.
#[test]
fn the_negatives_keep_their_refusals_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("RCN"));
        let report = presented(&snapshot, "RCN");
        for (signature, refusals) in NEG_REFUSALS {
            let body = method_body(&report.text, signature);
            for refusal in *refusals {
                assert!(
                    body.contains(refusal),
                    "`RCN.{signature}` on {} lost the refusal {refusal:?}:\n{body}",
                    leg.label
                );
            }
        }
        // The refused bodies present no field assignment at all: a copy's value written as a
        // statement would be the effect the refusal exists to keep out of the text.
        for line in report.text.lines().filter(|line| !line.contains("//")) {
            assert!(
                !line.contains("this.ok = this.ok") && !line.contains("this.n = this.n"),
                "`RCN` on {} presented an assignment its own refusal does not cover: {line:?}",
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
    let path =
        std::env::temp_dir().join(format!("jarde-conditional-rhs-{label}-{stamp}-{ordinal}"));
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

/// One class of one presentation, compiled on both compilers and compared with the fixture's own
/// answer.
fn replay_one(leg: &Leg, class: &str, text: &str, want: &str) {
    let javac8 =
        Path::new("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac");
    let java8 =
        Path::new("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java");
    let work = scratch(&format!("recovered-{class}"));
    fs::write(work.join(format!("{class}.java")), text).expect("the presentation is written");
    let got = compile_and_run("/usr/bin/javac", "/usr/bin/java", true, &work, class);
    assert_eq!(
        got, want,
        "`{class}` on {} answers differently after the round trip:\n{text}",
        leg.label
    );
    if javac8.is_file() {
        let work8 = scratch(&format!("recovered-8-{class}"));
        fs::write(work8.join(format!("{class}.java")), text).expect("the presentation is written");
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

#[test]
#[ignore = "compiles and runs the recovered text; needs the installed JDK (and javac 8 for the second leg)"]
fn the_recovered_text_compiles_and_runs_identically_on_both_legs() {
    // The frozen anchor: the class the patrol's own archive holds is the answer it must reproduce.
    let snapshot = open(FROZEN_JAR);
    let report = presented(&snapshot, "BI");
    let text = stripped(&report);
    assert!(
        !text.contains("jarde_refused_body"),
        "the frozen `BI` still holds a refused body marker:\n{text}"
    );
    let want_dir = scratch("frozen-original");
    fs::write(
        want_dir.join("BI.class"),
        frozen_class(FROZEN_JAR, b"BI.class"),
    )
    .expect("the frozen class is written");
    let want = run_class("/usr/bin/java", &want_dir, "BI");
    assert!(
        want.starts_with(FROZEN_ANSWER),
        "the frozen `BI` answers {FROZEN_ANSWER:?}:\n{want}"
    );
    let frozen_leg = Leg {
        label: "frozen bi.jar",
        files: &[("BI.class", FROZEN_JAR)],
    };
    replay_one(&frozen_leg, "BI", &text, &want);

    for leg in LEGS {
        for class in ["BI", "RC"] {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let text = stripped(&report);
            assert!(
                !text.contains("jarde_refused_body"),
                "`{class}` on {} still holds a refused body marker:\n{text}",
                leg.label
            );
            let want_dir = original_dir(leg.label, leg, class);
            let want = run_class("/usr/bin/java", &want_dir, class);
            replay_one(leg, class, &text, &want);
        }
        // `RCN` is the negatives' own class: its bodies are quoted, so its stripped text is the
        // *safe* form the soundness invariant asks for — a method whose body a reader cannot
        // compile, never one that compiles and behaves differently. The refusal texts are pinned by
        // the tests above; the replay's job for it is exactly that it does not compile.
        let snapshot = open(&leg.fixture("RCN"));
        let report = presented(&snapshot, "RCN");
        let text = stripped(&report);
        let work = scratch(&format!(
            "negative-RCN-{}",
            leg.label.replace([' ', '.', '('], "_")
        ));
        fs::write(work.join("RCN.java"), &text).expect("the presentation is written");
        let compiled = Command::new("/usr/bin/javac")
            .arg("--release")
            .arg("8")
            .arg("-nowarn")
            .arg("-d")
            .arg(&work)
            .arg(work.join("RCN.java"))
            .output()
            .expect("the compiler runs");
        assert!(
            !compiled.status.success(),
            "`RCN` on {} is quoted whole and its stripped text must not compile:\n{text}",
            leg.label
        );
    }
}
