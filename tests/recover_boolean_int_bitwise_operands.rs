//! `recover-boolean-int-bitwise-operands` in one frozen target: the **int-ified boolean operand of
//! a bitwise expression** — the `0`/`1` a `!` or a condition materialises, and the `int` counter a
//! `boolean r ^= x` accumulation lowers to — on the patrol's own anchor and on **both** compiler
//! legs (javac 23.0.1 `--release 8` and real javac 8, Corretto 1.8.0_432, from the same sources).
//!
//! `a & !b` lowers `!b` to `iload_1; ifne 9; iconst_1; goto 10; 9: iconst_0; 10: iand`, and
//! `r ^= x` lowers the `boolean` accumulator to an `int` local the frames cannot tell from an `int`.
//! Both used to refuse — "the bitwise operator `^`/`&` … has operands presented as `int` and
//! `boolean`, which no Java integral or boolean bitwise expression accepts" — and the class the
//! patrol filed answered `true/5/false/…` from its stripped text where the original answered
//! `true/5/true/…` (`mix`'s quoted loop body dropped the accumulation). The change reads an int
//! operand back as the `boolean` it is when its **whole production chain** is boolean bitwise
//! context and **every consumption** is a position a `boolean` is read in, and leaves every other
//! shape refused.
//!
//! The fixtures are the patrol's own `BW` (the acceptance anchor: its two shapes and the healthy
//! same-type shapes that must not move), this change's `BWR` (the read-back's four admitted shapes)
//! and `BWN` (the five refusals). The tests pin the presented texts, keep every refusal verbatim,
//! and the ignored replay strips the presentations the way the patrols' own stripped sources were
//! made (comment lines dropped), compiles each anchor with the installed `javac --release 8` and,
//! when a real javac 8 is present, with that one too, runs both under `-Xverify:all` and compares
//! every answer with the fixture's own class files.

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

/// The patrol's own frozen class, byte for byte: the answer the recovered text must reproduce.
const PATROL_CLASS: &[u8] =
    include_bytes!("fixtures/recover-boolean-int-bitwise-operands/patrol-BW.class");

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

/// javac 23.0.1, `javac --release 8 -g -nowarn -d v8 BW.java BWR.java BWN.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "BW.class",
        include_bytes!("fixtures/recover-boolean-int-bitwise-operands/v8/BW.class"),
    ),
    (
        "BWR.class",
        include_bytes!("fixtures/recover-boolean-int-bitwise-operands/v8/BWR.class"),
    ),
    (
        "BWN.class",
        include_bytes!("fixtures/recover-boolean-int-bitwise-operands/v8/BWN.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432,
/// `javac -g -nowarn -d v8-javac8 BW.java BWR.java BWN.java`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "BW.class",
        include_bytes!("fixtures/recover-boolean-int-bitwise-operands/v8-javac8/BW.class"),
    ),
    (
        "BWR.class",
        include_bytes!("fixtures/recover-boolean-int-bitwise-operands/v8-javac8/BWR.class"),
    ),
    (
        "BWN.class",
        include_bytes!("fixtures/recover-boolean-int-bitwise-operands/v8-javac8/BWN.class"),
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

/// `BW.mix` — the `boolean` accumulate counter, read back as the boolean it is. The frozen patrol
/// class carries no debug names (the locals are the synthetic ones); the recompiled legs carry the
/// source's own (`r`, `x`).
const ANCHOR_MIX_FROZEN: &[&str] = &[
    "        boolean local1;",
    "        local1 = false;",
    "            local1 = local1 ^ local5;",
    "        return local1;",
];

/// The same anchor on the two recompiled legs.
const ANCHOR_MIX_LEG: &[&str] = &[
    "        boolean r;",
    "        r = false;",
    "            r = r ^ x;",
    "        return r;",
];

/// `BW.andNot` — the materialised `!b` read back beside the `Z` parameter it is combined with.
const ANCHOR_AND_NOT_FROZEN: &str = "        return arg0 & !arg1;";

/// The same anchor on the two recompiled legs.
const ANCHOR_AND_NOT_LEG: &str = "        return a & !b;";

/// `BWR.accXor` — the counter read by another bitwise operator and by the `Z` return.
const ANCHOR_ACC_XOR: &[&str] = &[
    "        boolean r;",
    "        r = false;",
    "            r = r ^ x;",
    "        return r ^ c;",
];

/// `BWR.andNotAnd` — the materialisation read by a bitwise operator whose own value the `Z` return
/// reads.
const ANCHOR_AND_NOT_AND: &str = "        return a & !b & c;";

/// `BWR.orNot` and `BWR.notOr` — the other eager boolean operator, on either side.
const ANCHOR_OR_NOT: &str = "        return a | !b;";
const ANCHOR_NOT_OR: &str = "        return !a | b;";

/// The healthy **same-type** shapes the patrol recorded, in the frozen class's own spelling: every
/// one of them keeps the presentation the patrol's own recorded render has, byte for byte.
const FROZEN_SAME_TYPE_METHODS: &[(&str, &str)] = &[
    ("xor(ZZ)Z", "        return arg0 ^ arg1;"),
    ("ixor(II)I", "        return arg0 ^ arg1;"),
    ("shConst(I)I", "        return arg0 << 33;"),
    ("shFold()I", "        return -2147483648;"),
    (
        "bits(I)I",
        "        int local1;\n        local1 = 0;\n        while (arg0 != 0) {\n            arg0 = arg0 & arg0 - 1;\n            local1 = local1 + 1;\n        }\n        return local1;",
    ),
];

/// The same shapes on the two recompiled legs, with the source's own debug names.
const LEG_SAME_TYPE_METHODS: &[(&str, &str)] = &[
    ("xor(ZZ)Z", "        return a ^ b;"),
    ("ixor(II)I", "        return a ^ b;"),
    ("shConst(I)I", "        return x << 33;"),
    ("shFold()I", "        return -2147483648;"),
    (
        "bits(I)I",
        "        int n;\n        n = 0;\n        while (x != 0) {\n            x = x & x - 1;\n            n = n + 1;\n        }\n        return n;",
    ),
];

/// The refusals `BWN` keeps, verbatim, on both legs.
const NEG_REFUSALS: &[(&str, &[&str])] = &[
    (
        "andNotInt(ZZ)I",
        &[
            "// the bitwise operator `&` at BCI 10 has operands presented as `boolean` and `int`, which no Java integral or boolean bitwise expression accepts",
            "// @bytecode 10 14 18 11",
        ],
    ),
    (
        "compareRead([ZZ)Z",
        &[
            "// the bitwise operator `^` at BCI 27 has operands presented as `int` and `boolean`, which no Java integral or boolean bitwise expression accepts",
            "// the branch at BCI 37 compares a value this layer proves boolean with one it does not (`==`), and no Java comparison spells that pair of operands: the text this layer would write is refused by javac (`incomparable types: boolean and int`)",
        ],
    ),
    (
        "passed([Z)V",
        &[
            "// the bitwise operator `^` at BCI 25 has operands presented as `int` and `boolean`, which no Java integral or boolean bitwise expression accepts",
            "// the parameter 0 of the invocation at BCI 37 is declared `boolean` presents `int` and this layer has no proven conversion to `boolean`",
        ],
    ),
];

/// The negatives that keep an **int** presentation unchanged: the materialised `0`/`1` consumed by
/// arithmetic and the `int` counter beside an `int` sibling.
const NEG_PRESENTATIONS: &[(&str, &str)] = &[
    ("plusOne(ZI)I", "        return (b ? 1 : 0) + x;"),
    ("intSibling(IZ)I", "        return r ^ (b ? 1 : 0);"),
];

/// The one diagnostic this change exists to remove from the anchor.
const BITWISE_REFUSAL: &str = "which no Java integral or boolean bitwise expression accepts";

/// The recovering methods that must present every instruction they have.
const NO_QUOTE_METHODS: &[(&str, &str)] = &[
    ("BW", "xor(ZZ)Z"),
    ("BW", "ixor(II)I"),
    ("BW", "mix([Z)Z"),
    ("BW", "shConst(I)I"),
    ("BW", "shFold()I"),
    ("BW", "andNot(ZZ)Z"),
    ("BW", "bits(I)I"),
    ("BW", "main([Ljava/lang/String;)V"),
    ("BWR", "accXor([ZZ)Z"),
    ("BWR", "andNotAnd(ZZZ)Z"),
    ("BWR", "orNot(ZZ)Z"),
    ("BWR", "notOr(ZZ)Z"),
    ("BWR", "main([Ljava/lang/String;)V"),
];

/// The anchor's own answer, which the frozen class prints and the recovered text must print too.
const FROZEN_ANSWER: &str = "true/5/true/2/-2147483648/false/3";

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
// The anchors: the frozen class recovers whole, and both legs write the source forms.
// -------------------------------------------------------------------------------------------

/// The patrol's frozen anchor is a **whole** recovery: no quoted bytecode anywhere in the class, and
/// both of the change's shapes are written as the boolean forms their source had.
#[test]
fn the_frozen_anchor_recovers_whole_class() {
    let snapshot = open(&zip_of(&[(b"BW.class", PATROL_CLASS)]));
    let report = presented(&snapshot, "BW");
    let quoted = quoted_bcis(&report.text);
    assert!(
        quoted.is_empty(),
        "the frozen `BW` still quotes {quoted:?}:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("not recovered"),
        "the frozen `BW` is not a whole recovery:\n{}",
        report.text
    );
    assert!(
        !report.text.contains(BITWISE_REFUSAL),
        "the frozen `BW` still carries the bitwise operand refusal:\n{}",
        report.text
    );
    let mix = method_body(&report.text, "mix([Z)Z");
    for line in ANCHOR_MIX_FROZEN {
        assert!(
            mix.contains(line),
            "the frozen `BW.mix` lost {line:?}:\n{mix}"
        );
    }
    let and_not = method_body(&report.text, "andNot(ZZ)Z");
    assert!(
        and_not.contains(ANCHOR_AND_NOT_FROZEN),
        "the frozen `BW.andNot` lost {ANCHOR_AND_NOT_FROZEN:?}:\n{and_not}"
    );
    // The same-type shapes keep the presentation the patrol's own recorded render states.
    for (signature, expected) in FROZEN_SAME_TYPE_METHODS {
        let body = method_body(&report.text, signature);
        assert!(
            body.ends_with(expected),
            "the frozen `BW.{signature}` moved: expected it to end with {expected:?}:\n{body}"
        );
    }
}

/// Every anchor is written on both legs, with the text the design states.
#[test]
fn the_anchors_write_their_presentations_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BW"));
        let report = presented(&snapshot, "BW");
        let mix = method_body(&report.text, "mix([Z)Z");
        for line in ANCHOR_MIX_LEG {
            assert!(
                mix.contains(line),
                "`BW.mix` on {} lost {line:?}:\n{mix}",
                leg.label
            );
        }
        let and_not = method_body(&report.text, "andNot(ZZ)Z");
        assert!(
            and_not.contains(ANCHOR_AND_NOT_LEG),
            "`BW.andNot` on {} lost {ANCHOR_AND_NOT_LEG:?}:\n{and_not}",
            leg.label
        );

        let snapshot = open(&leg.fixture("BWR"));
        let report = presented(&snapshot, "BWR");
        let acc_xor = method_body(&report.text, "accXor([ZZ)Z");
        for line in ANCHOR_ACC_XOR {
            assert!(
                acc_xor.contains(line),
                "`BWR.accXor` on {} lost {line:?}:\n{acc_xor}",
                leg.label
            );
        }
        for (signature, anchor) in [
            ("andNotAnd(ZZZ)Z", ANCHOR_AND_NOT_AND),
            ("orNot(ZZ)Z", ANCHOR_OR_NOT),
            ("notOr(ZZ)Z", ANCHOR_NOT_OR),
        ] {
            let body = method_body(&report.text, signature);
            assert!(
                body.contains(anchor),
                "`BWR.{signature}` on {} lost {anchor:?}:\n{body}",
                leg.label
            );
        }
    }
}

/// The recovering methods present every instruction they have: an anchor that still quotes one
/// would be a partial recovery counted as one.
#[test]
fn no_recovering_method_keeps_a_quote() {
    for leg in LEGS {
        for class in ["BW", "BWR"] {
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
                    !body.contains(BITWISE_REFUSAL),
                    "`{owner}.{signature}` on {} still carries the bitwise operand refusal:\n{body}",
                    leg.label
                );
            }
        }
    }
}

/// The healthy same-type shapes — `boolean ^ boolean`, `int ^ int`, the over-wide shift, the folded
/// constant shift and the Kernighan loop — keep the presentation they had before the change, byte
/// for byte.
#[test]
fn the_same_type_shapes_are_unchanged_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BW"));
        let report = presented(&snapshot, "BW");
        for (signature, expected) in LEG_SAME_TYPE_METHODS {
            let body = method_body(&report.text, signature);
            assert!(
                body.ends_with(expected),
                "`BW.{signature}` on {} moved: expected it to end with {expected:?}:\n{body}",
                leg.label
            );
        }
    }
}

// -------------------------------------------------------------------------------------------
// The negatives keep their refusals and their int presentations.
// -------------------------------------------------------------------------------------------

/// The refusals keep the text they had, and no refused body writes a statement its own refusal does
/// not cover.
#[test]
fn the_negatives_keep_their_refusals_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BWN"));
        let report = presented(&snapshot, "BWN");
        for (signature, refusals) in NEG_REFUSALS {
            let body = method_body(&report.text, signature);
            for refusal in *refusals {
                assert!(
                    body.contains(refusal),
                    "`BWN.{signature}` on {} lost the refusal {refusal:?}:\n{body}",
                    leg.label
                );
            }
            // The refusal is either the whole method's (the marker names it) or one quoted region
            // of its body: the body's own slice starts inside the marker line, so the marker is
            // read from the whole class text.
            assert!(
                !quoted_bcis(body).is_empty()
                    || report.text.contains(&format!(
                        "the recovery run for `{signature}` produced no statement"
                    )),
                "`BWN.{signature}` on {} is presented as recovered:\n{body}",
                leg.label
            );
        }
        for (signature, expected) in NEG_PRESENTATIONS {
            let body = method_body(&report.text, signature);
            assert!(
                body.ends_with(expected),
                "`BWN.{signature}` on {} moved: expected it to end with {expected:?}:\n{body}",
                leg.label
            );
            assert!(
                !body.contains(BITWISE_REFUSAL),
                "`BWN.{signature}` on {} became a refusal:\n{body}",
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
    let path = std::env::temp_dir().join(format!(
        "jarde-boolean-int-bitwise-{label}-{stamp}-{ordinal}"
    ));
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
    // The frozen anchor: the patrol's own class is the answer the recovered text must reproduce.
    let frozen_leg = Leg {
        label: "the patrol's frozen BW.class",
        files: &[("BW.class", PATROL_CLASS)],
    };
    let snapshot = open(&zip_of(&[(b"BW.class", PATROL_CLASS)]));
    let report = presented(&snapshot, "BW");
    let text = stripped(&report);
    let want_dir = original_dir("frozen", &frozen_leg, "BW");
    let want = run_class("/usr/bin/java", &want_dir, "BW");
    assert!(
        want.starts_with(FROZEN_ANSWER),
        "the frozen `BW` answers {FROZEN_ANSWER:?}:\n{want}"
    );
    replay_one(&frozen_leg, "BW", &text, &want);

    for leg in LEGS {
        for class in ["BW", "BWR"] {
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
        // `BWN` is the negatives' own class: its bodies are quoted, so its stripped text is the
        // *safe* form the soundness invariant asks for — a method whose body a reader cannot
        // compile, never one that compiles and behaves differently. The refusal texts are pinned by
        // the tests above; the replay's job for it is exactly that it does not compile.
        let snapshot = open(&leg.fixture("BWN"));
        let report = presented(&snapshot, "BWN");
        let text = stripped(&report);
        let work = scratch(&format!(
            "negative-BWN-{}",
            leg.label.replace([' ', '.', '('], "_")
        ));
        fs::write(work.join("BWN.java"), &text).expect("the presentation is written");
        let compiled = Command::new("/usr/bin/javac")
            .arg("--release")
            .arg("8")
            .arg("-nowarn")
            .arg("-d")
            .arg(&work)
            .arg(work.join("BWN.java"))
            .output()
            .expect("the compiler runs");
        assert!(
            !compiled.status.success(),
            "`BWN` on {} is quoted whole and its stripped text must not compile:\n{text}",
            leg.label
        );
    }
}
