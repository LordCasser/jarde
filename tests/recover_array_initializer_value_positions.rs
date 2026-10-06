//! `recover-array-initializer-value-positions` in one frozen target: the initialization dance's
//! value presented at the **consumption positions** the six-position discriminator found refusing
//! (`openspec/evidence/java-syntax-2026-10-05/array-initializer-value-patrol`), on both compiler
//! legs (javac 23.0.1 `--release 8` and real javac 8, Corretto 1.8.0_432, from the same sources).
//!
//! `javac` lowers `new T[]{…}` to a dance — `newarray; dup; <index>; <value>; iastore` — that
//! leaves the array on the stack for its one reader. Four positions were already presented (a local
//! store, a field write, a call's argument, an outer initializer's element) and a **subscript or a
//! length read one step further** was not: the value's reader stands past the instruction that
//! follows the last element store (`…; iastore; iconst_0; iaload`), and an element store into an
//! array the body already had was dropped by the commit walk's blanket "an element store cannot
//! stand alone" skip. Both refusals are the copy family's own
//! (`the copy at BCI N has no proved local assignment`), and both are what this change closes:
//!
//! * the proof reads the value's **own single use** as the reader, and the run between the element
//!   store and it as that reader's other operand, judged by the same `collect_expression_bcis`
//!   classification the initializer's own element values pass through;
//! * the commit walk keeps the child geometry it always kept — an element store into an array this
//!   run **builds** (the parent's own `dup` copy) — and commits a store into an array the body
//!   already had.
//!
//! The fixtures are the patrol's own `MD`/`MD2`/`MD3` (frozen anchors and zero-regression
//! positions, recompiled on both legs) and this change's own `AV` (the admitted positions one step
//! further: a variable index, a computed index, the length receiver, two dances in one expression,
//! and an effectful element beside an effectful index), beside the hand-built `AVN` whose three
//! members state the same dance with one reader, with two readers and with a discarded second
//! consumer — `javac` emits none of the latter two, so they are assembled byte by byte
//! (`build_avn.py`).
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
        let files: Vec<(String, &'static [u8])> = self
            .files
            .iter()
            .filter(|(name, _)| {
                let name: &str = name;
                name == own
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

/// javac 23.0.1, `javac --release 8 -g -nowarn -d v8 MD.java MD2.java MD3.java AV.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "MD.class",
        include_bytes!("fixtures/recover-array-initializer-value-positions/v8/MD.class"),
    ),
    (
        "MD2.class",
        include_bytes!("fixtures/recover-array-initializer-value-positions/v8/MD2.class"),
    ),
    (
        "MD3.class",
        include_bytes!("fixtures/recover-array-initializer-value-positions/v8/MD3.class"),
    ),
    (
        "AV.class",
        include_bytes!("fixtures/recover-array-initializer-value-positions/v8/AV.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432,
/// `javac -g -nowarn -d v8-javac8 MD.java MD2.java MD3.java AV.java`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "MD.class",
        include_bytes!("fixtures/recover-array-initializer-value-positions/v8-javac8/MD.class"),
    ),
    (
        "MD2.class",
        include_bytes!("fixtures/recover-array-initializer-value-positions/v8-javac8/MD2.class"),
    ),
    (
        "MD3.class",
        include_bytes!("fixtures/recover-array-initializer-value-positions/v8-javac8/MD3.class"),
    ),
    (
        "AV.class",
        include_bytes!("fixtures/recover-array-initializer-value-positions/v8-javac8/AV.class"),
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

/// The hand-built negatives: one class, assembled byte by byte by `build_avn.py`.
const AVN: &[u8] = include_bytes!("fixtures/recover-array-initializer-value-positions/AVN.class");

// -------------------------------------------------------------------------------------------
// The presented texts.
// -------------------------------------------------------------------------------------------

/// The two positions the patrol's discriminator found refusing, and the statement each becomes.
const ANCHOR_ELEMENT_STORE: &str = "        saved0[0] = new int[]{7};";
const ANCHOR_IMMEDIATE_INDEX: &str = "        return new int[]{9}[0];";

/// `MD2.retPos` is the same immediate-index position — the patrol's fixture comments it as 返回位,
/// but `return new int[]{4}[0];` lowers to the same `…; iastore; iconst_0; iaload; ireturn`.
const ANCHOR_INDEX_MD2: &str = "        return new int[]{4}[0];";

/// The methods whose text must not move: the four positions that already presented the dance, the
/// bare immediate consumption, and the sawtooth family. Each body is the one the **frozen parent
/// commit** renders, byte for byte (`openspec/changes/recover-array-initializer-value-positions/
/// results/02-gating-experiment.md`).
const UNMOVED: &[(&str, &str, &str)] = &[
    (
        "MD",
        "mkJagged",
        "    static int[][] mkJagged() {
        // @method mkJagged()[[I
        // @declaration a static method of `MD`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[][]{new int[]{8}, new int[]{9, 10}};
    }",
    ),
    (
        "MD2",
        "localPos",
        "    static int localPos() {
        // @method localPos()I
        // @declaration a static method of `MD2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int[] x = new int[]{1, 2};
        return x[0] + x[1];
    }",
    ),
    (
        "MD2",
        "fieldPos",
        "    static int fieldPos() {
        // @method fieldPos()I
        // @declaration a static method of `MD2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        MD2.f = new int[]{3};
        return MD2.f[0];
    }",
    ),
    (
        "MD2",
        "argPos",
        "    static int argPos() {
        // @method argPos()I
        // @declaration a static method of `MD2`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return sum(new int[]{5});
    }",
    ),
    (
        "MD3",
        "bareIdx",
        "    static int bareIdx() {
        // @method bareIdx()I
        // @declaration a static method of `MD3`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[2].length;
    }",
    ),
    (
        "MD3",
        "bareRet",
        "    static int[] bareRet() {
        // @method bareRet()[I
        // @declaration a static method of `MD3`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[3];
    }",
    ),
    (
        "AV",
        "localPos",
        "    static int localPos() {
        // @method localPos()I
        // @declaration a static method of `AV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int[] x = new int[]{1, 2};
        return x[0] + x[1];
    }",
    ),
    (
        "AV",
        "fieldPos",
        "    static int fieldPos() {
        // @method fieldPos()I
        // @declaration a static method of `AV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        AV.f = new int[]{3};
        return AV.f[0];
    }",
    ),
    (
        "AV",
        "argPos",
        "    static int argPos() {
        // @method argPos()I
        // @declaration a static method of `AV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return sum(new int[]{5});
    }",
    ),
    (
        "AV",
        "outerPos",
        "    static int[][] outerPos() {
        // @method outerPos()[[I
        // @declaration a static method of `AV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[][]{new int[]{6}, new int[]{7, 8}};
    }",
    ),
    (
        "AV",
        "bareIdx",
        "    static int bareIdx() {
        // @method bareIdx()I
        // @declaration a static method of `AV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[2].length;
    }",
    ),
    (
        "AV",
        "bareRet",
        "    static int[] bareRet() {
        // @method bareRet()[I
        // @declaration a static method of `AV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[3];
    }",
    ),
    (
        "AV",
        "jagged",
        "    static int[][] jagged() {
        // @method jagged()[[I
        // @declaration a static method of `AV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[][]{new int[]{1}, new int[]{2, 3}, new int[]{4, 5, 6}};
    }",
    ),
    (
        "AV",
        "foreach",
        "    static int foreach() {
        // @method foreach()I
        // @declaration a static method of `AV`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int s;
        int[] local1;
        s = 0;
        local1 = new int[]{1, 2};
        for (int v : local1) {
            s = s + v;
        }
        return s;
    }",
    ),
];

/// The one diagnostic this change exists to remove from the anchors.
const COPY_REFUSAL: &str = "has no proved local assignment";

/// The hand-built negatives: the bodies the frozen parent commit renders, byte for byte.
const AVN_TWO_READERS: &str = "    public static int twoReaders() {\n        // jarde: not recovered: the recovery run for `twoReaders()I` stopped (jre_ir_table_missing); the analysis of that member did not complete (ir_frame_inconsistent)\n    }";
const AVN_DISCARDED: &str = "    public static int discarded() {\n        // jarde: not recovered: the recovery run for `discarded()I` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method discarded()I\n        // @declaration a static method of `AVN`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 1\n        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text\n        // @bytecode 3 1\n        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds\n        // @bytecode 7 1\n        // the copy at BCI 3 has no proved local assignment\n        // @bytecode 8 1\n        // the instruction at BCI 8 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds\n        // @bytecode 9\n        // the instruction at BCI 9 is not part of the provable subset\n        // @bytecode 12 11 1\n        // the copy at BCI 8 has no proved local assignment\n    }";

/// The answers the fixtures' own classes print, which the recovered text must print too.
const FROZEN_ANSWERS: &[(&str, &str)] = &[
    ("MD", "21/9/9/10"),
    ("MD2", "3/3/4/5"),
    ("MD3", "2/9/3"),
    ("AV", "7/9/3/3/1/8/2/3/6/3/9/8/10/17/8/9/3/true/2/1/2"),
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

/// The body text of one method, from its own signature line to the closing brace.
///
/// The signature is looked for on a **declaration** line (one that ends in `{` and is not a
/// comment): the `// @method …` line above it states the same descriptor, and a body read from
/// there would start inside the comment.
fn method_body<'a>(text: &'a str, signature: &str) -> &'a str {
    let mut offset = 0;
    let mut start = None;
    for line in text.split_inclusive('\n') {
        if !line.trim_start().starts_with("//")
            && line.trim_end().ends_with('{')
            && line.contains(&format!("{signature}("))
        {
            start = Some(offset);
            break;
        }
        offset += line.len();
    }
    let start = start.unwrap_or_else(|| panic!("the presentation states `{signature}`:\n{text}"));
    let rest = &text[start..];
    let end = rest
        .find("\n    }")
        .unwrap_or_else(|| panic!("the method `{signature}` closes:\n{text}"));
    &rest[..end + "\n    }".len()]
}

// -------------------------------------------------------------------------------------------
// The anchors: both refusing positions recover, on both legs.
// -------------------------------------------------------------------------------------------

/// The two positions the patrol found refusing recover whole: no quote anywhere in the class, and
/// the statement the design states.
#[test]
fn the_two_refusing_positions_recover_on_both_legs() {
    for leg in LEGS {
        for class in ["MD", "MD2", "MD3"] {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let quoted = quoted_bcis(&report.text);
            assert!(
                quoted.is_empty(),
                "`{class}` on {} still quotes {quoted:?}:\n{}",
                leg.label,
                report.text
            );
            assert!(
                !report.text.contains("not recovered"),
                "`{class}` on {} is not a whole recovery:\n{}",
                leg.label,
                report.text
            );
            assert!(
                !report.text.contains(COPY_REFUSAL),
                "`{class}` on {} still carries the copy family's refusal:\n{}",
                leg.label,
                report.text
            );
        }
        let snapshot = open(&leg.fixture("MD"));
        let report = presented(&snapshot, "MD");
        assert!(
            report.text.contains(ANCHOR_ELEMENT_STORE),
            "`MD` on {} lost the element-store anchor {ANCHOR_ELEMENT_STORE:?}:\n{}",
            leg.label,
            report.text
        );
        let snapshot = open(&leg.fixture("MD3"));
        let report = presented(&snapshot, "MD3");
        assert!(
            report.text.contains(ANCHOR_IMMEDIATE_INDEX),
            "`MD3` on {} lost the immediate-index anchor {ANCHOR_IMMEDIATE_INDEX:?}:\n{}",
            leg.label,
            report.text
        );
        let snapshot = open(&leg.fixture("MD2"));
        let report = presented(&snapshot, "MD2");
        assert!(
            report.text.contains(ANCHOR_INDEX_MD2),
            "`MD2` on {} lost the same position's anchor {ANCHOR_INDEX_MD2:?}:\n{}",
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
        let snapshot = open(&leg.fixture("AV"));
        let report = presented(&snapshot, "AV");
        let quoted = quoted_bcis(&report.text);
        assert!(
            quoted.is_empty(),
            "`AV` on {} still quotes {quoted:?}:\n{}",
            leg.label,
            report.text
        );
        for signature in [
            "elemStore",
            "immIdx",
            "immLen",
            "immIdxVar",
            "immIdxExpr",
            "immIdxSum",
            "twoIdx",
            "twoStores",
            "nestedIdx",
            "condIdx",
            "immIdxInCall",
            "order",
        ] {
            let body = method_body(&report.text, signature);
            assert!(
                !body.contains(COPY_REFUSAL),
                "`AV.{signature}` on {} still carries the copy family's refusal:\n{body}",
                leg.label
            );
        }
    }
}

// -------------------------------------------------------------------------------------------
// Zero regression: the positions that already presented the dance keep their text.
// -------------------------------------------------------------------------------------------

/// Every position that already presented the dance — the local store, the field write, the
/// argument, the outer initializer's element, the bare immediate consumption and the sawtooth
/// family — is byte for byte the text the frozen parent commit renders.
#[test]
fn the_presenting_positions_keep_their_text_byte_for_byte() {
    for leg in LEGS {
        for (class, signature, expected) in UNMOVED {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let body = method_body(&report.text, signature);
            assert_eq!(
                body, *expected,
                "`{class}.{signature}` on {} moved:\n{body}",
                leg.label
            );
        }
    }
}

// -------------------------------------------------------------------------------------------
// The negatives keep their refusals.
// -------------------------------------------------------------------------------------------

/// The hand-built members state the same dance with one reader, with two readers and with a
/// discarded second consumer. The first is the builder's own self-test — the bytes really are the
/// anchor's shape — and the other two keep the text they had before the change, byte for byte.
#[test]
fn the_hand_built_negatives_keep_their_refusals() {
    let snapshot = open(&zip_of(&[(b"AVN.class", AVN)]));
    let report = presented(&snapshot, "AVN");
    assert!(
        report.text.contains(ANCHOR_IMMEDIATE_INDEX),
        "the hand-built `single()` is not the anchor's shape:\n{}",
        report.text
    );
    assert_eq!(
        method_body(&report.text, "twoReaders"),
        AVN_TWO_READERS,
        "the two-reader dance moved:\n{}",
        report.text
    );
    assert_eq!(
        method_body(&report.text, "discarded"),
        AVN_DISCARDED,
        "the discarded second consumer moved:\n{}",
        report.text
    );
    // The refused members name the instructions they no longer present: a dance neither rendered
    // nor quoted would be an effect the text silently drops.
    let discarded = method_body(&report.text, "discarded");
    for bci in [1, 3, 7, 8, 9, 12] {
        assert!(
            discarded.contains(&format!("// @bytecode {bci}")),
            "the refused dance leaves BCI {bci} unaccounted for:\n{discarded}"
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
    let path = std::env::temp_dir().join(format!("jarde-array-value-{label}-{stamp}-{ordinal}"));
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

/// Run one committed fixture under `-Xverify:all`, answering its standard output.
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
    for leg in LEGS {
        for (class, answer) in FROZEN_ANSWERS {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let text = stripped(&report);
            assert!(
                !text.contains("@bytecode") && !text.contains("not recovered"),
                "`{class}` on {} still holds a quote:\n{text}",
                leg.label
            );
            let want_dir = scratch(&format!("original-{class}"));
            for (name, bytes) in leg.family(class) {
                fs::write(want_dir.join(&name), bytes).expect("the fixture class is written");
            }
            let want = run_class("/usr/bin/java", &want_dir, class);
            assert!(
                want.starts_with(answer),
                "the fixture's own `{class}` answers {answer:?}:\n{want}"
            );
            replay_one(leg, class, &text, &want);
        }
    }
    // The hand-built negatives are quoted whole: their stripped text must not compile — a method
    // whose body a reader cannot compile is the safe form, never one that compiles and behaves
    // differently.
    let snapshot = open(&zip_of(&[(b"AVN.class", AVN)]));
    let report = presented(&snapshot, "AVN");
    let text = stripped(&report);
    let work = scratch("negative-AVN");
    fs::write(work.join("AVN.java"), &text).expect("the presentation is written");
    let compiled = Command::new("/usr/bin/javac")
        .arg("--release")
        .arg("8")
        .arg("-nowarn")
        .arg("-d")
        .arg(&work)
        .arg(work.join("AVN.java"))
        .output()
        .expect("the compiler runs");
    assert!(
        !compiled.status.success(),
        "`AVN` is quoted whole and its stripped text must not compile:\n{text}"
    );
}
