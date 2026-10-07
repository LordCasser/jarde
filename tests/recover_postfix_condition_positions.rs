//! `recover-postfix-condition-positions` in one frozen target: the postfix old-value **condition
//! positions** on **both** compiler legs (javac 23.0.1 `--release 8` and real javac 8, Corretto
//! 1.8.0_432, from the same sources).
//!
//! javac reads an incremented local *before* it updates it whenever the value is consumed, and a
//! loop's or a branch's test is one of those consumers: `while (xs[i++] != 0 && …)` is
//! `iload; iinc; iaload; if…`, and the old value the load took is read once, after the update, by
//! the test's own value expression. The change presents that value where the bytecode read it — the
//! test writes the postfix expression (`arg0[local1++]`) and the increment is absorbed into it
//! instead of quoting the whole method (`local 1 crosses a quoted fallback region`), exactly the way
//! `recover-postfix-old-value-snapshot` presents the value positions.
//!
//! The slice's own bound is stated: **one** postfix position per condition, at one end of a
//! short-circuit chain. A second variable's position and the middle of a chain are recorded and
//! left to a later slice, so a chain that would present them keeps the refusal it had
//! (`jre_region_chain_position_bound`).
//!
//! The fixtures are the postfix-condition patrol's own frozen trio (`CP7`: the do-while scan, the
//! while compound and the if short-circuit; its source is copied byte for byte and its `v8` class
//! file is byte-identical to the patrol's jar entry, checked by SHA in the fixture README), this
//! slice's `CN` (the two negatives and the compound-chain control) and `CC` (the three positions
//! again, each answering the count its increment reached, plus the control, so the do-while scan's
//! exact iteration count is compared and not only its printed answer). The A-phase traps are read
//! from the A-phase fixture's own `NG.class`, the way `recover-dup-store-conditional` reads CF-06's
//! control from the fixture that owns it.
//!
//! The tests pin the presented texts, keep every negative's refusal verbatim, keep the control
//! byte-identical, and the ignored replay strips the presentations the way the patrols' own stripped
//! sources were made (comment lines dropped), compiles each anchor with the installed
//! `javac --release 8` and, when a real javac 8 is present, with that one too, runs both under
//! `-Xverify:all` and compares every answer with the fixture's own class files.

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

/// javac 23.0.1, `javac --release 8 -Xlint:-options -nowarn -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "CP7.class",
        include_bytes!("fixtures/recover-postfix-condition-positions/v8/CP7.class"),
    ),
    (
        "CN.class",
        include_bytes!("fixtures/recover-postfix-condition-positions/v8/CN.class"),
    ),
    (
        "CC.class",
        include_bytes!("fixtures/recover-postfix-condition-positions/v8/CC.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -nowarn -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "CP7.class",
        include_bytes!("fixtures/recover-postfix-condition-positions/v8-javac8/CP7.class"),
    ),
    (
        "CN.class",
        include_bytes!("fixtures/recover-postfix-condition-positions/v8-javac8/CN.class"),
    ),
    (
        "CC.class",
        include_bytes!("fixtures/recover-postfix-condition-positions/v8-javac8/CC.class"),
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
const CLASSES: &[&str] = &["CP7", "CN", "CC"];

/// The A-phase traps: the fixture that owns them (`recover-postfix-old-value-snapshot`'s `NG`),
/// SHA-pinned in that fixture's README, is read from there rather than copied here.
const A_PHASE_TRAP_FILES: &[(&str, &[u8])] = &[(
    "NG.class",
    include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8/NG.class"),
)];
const A_PHASE_TRAP_FILES_JAVAC8: &[(&str, &[u8])] = &[(
    "NG.class",
    include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8-javac8/NG.class"),
)];

// -------------------------------------------------------------------------------------------
// The presented texts, pinned whole (the p3 surface tests' own convention).
// -------------------------------------------------------------------------------------------

/// `CP7.scan` — the do-while scan: the postfix expression is written in the loop's own condition
/// and the absorbed increment writes no statement of its own.
const CP7_SCAN: &[&str] = &[
    "        do {",
    "            local2 = arg0[local1];",
    "        } while (arg0[local1++] != 0 && local1 < arg0.length);",
];

/// `CP7.find` — the while compound: the postfix expression is written in the second test, and the
/// first test still reads the slot where it holds the pre-update value.
const CP7_FIND: &[&str] = &[
    "        while (local2 < arg0.length && arg0[local2++] != arg1) {",
    "        }",
    "        return local2 - 1;",
];

/// `CP7.cond` — the if short-circuit: the two-exit return writes the condition as one expression,
/// and the local's own initialisation is written in front of it.
const CP7_COND: &[&str] = &[
    "        int local1 = 0;",
    "        return arg0[local1++] > 0 ? local1 < arg0.length ? true : false : false;",
];

/// `CC.scan`/`CC.find`/`CC.cond` — the same three positions answering their increment's count.
const CC_SCAN: &[&str] = &[
    "        } while (arg0[local1++] != 0 && local1 < arg0.length);",
    "        return local1;",
];
const CC_FIND: &[&str] = &[
    "        while (local2 < arg0.length && arg0[local2++] != arg1) {",
    "        }",
    "        return local2;",
];
const CC_COND: &[&str] = &[
    "        if (arg0[local1++] > 0) {",
    "            if (local1 < arg0.length) {",
    "                return local1;",
    "        return -local1;",
];

/// `CN.chain` — the compound do-while chain the region layer already presented, with no postfix
/// position in it: its text must stay byte-identical.
const CN_CHAIN: &[&str] = &[
    "        do {",
    "            local2 = local2 + 1;",
    "            arg0 = arg0 - 1;",
    "            arg1 = arg1 - 1;",
    "        } while (arg0 > 0 && arg1 > 0);",
    "        return local2;",
];

/// The refusals the recovering members must not keep: a member that still quotes its own
/// instructions would be a partial recovery counted as one.
const NO_QUOTE_METHODS: &[(&str, &str)] = &[
    ("CP7", "scan([I)I"),
    ("CP7", "find([II)I"),
    ("CP7", "cond([I)Z"),
    ("CN", "chain(II)I"),
    ("CN", "main([Ljava/lang/String;)V"),
    ("CC", "scan([I)I"),
    ("CC", "find([II)I"),
    ("CC", "cond([I)I"),
    ("CC", "chain(II)I"),
    ("CC", "main([Ljava/lang/String;)V"),
];

/// The two negatives this slice states, verbatim: a second variable's position in one condition
/// and a position in the middle of a short-circuit chain. Each keeps its own refusal.
const CN_NEGATIVES: &[(&str, &str)] = &[
    (
        "twoVariables",
        "// local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
    ),
    (
        "midChain",
        "// local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
    ),
];

/// The region the fence refuses the negatives under: the slice's own bound, stated by the rule that
/// declined the chain.
const CHAIN_POSITION_BOUND: &str = "jre_region_chain_position_bound";

/// The A-phase traps, verbatim (the texts `recover_postfix_old_value_snapshot` pins for its own
/// fixture): the two local self-assignment traps, the field self-assignment and the multi-consumer
/// form. None of them may move for a condition position to be presented.
const A_PHASE_TRAPS: &[(&str, &str)] = &[
    (
        "postSelf",
        "// the value at BCI 6 is the value local 0 held at BCI 2, and the slot does not hold it at BCI 6",
    ),
    (
        "postSelfDec",
        "// the value at BCI 6 is the value local 0 held at BCI 2, and the slot does not hold it at BCI 6",
    ),
    (
        "fieldSelf",
        "// the old-value update ending at BCI 19 has no complete same-target, single-consumer, same-handler and evaluation-order proof",
    ),
    (
        "compoundSelf",
        "// the value at BCI 10 is the value local 1 held at BCI 2, and the slot does not hold it at BCI 10",
    ),
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
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::RegionDetails),
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

/// The region codes one member's own run reported.
fn region_codes(report: &ClassSourceReport, method: &str) -> Vec<String> {
    for member in &report.methods {
        let ClassSourceOutcome::Recovered { report: run, .. } = &member.outcome else {
            continue;
        };
        if run.method == method {
            return run
                .regions
                .iter()
                .filter_map(|region| region.code.map(str::to_owned))
                .collect();
        }
    }
    panic!("the presentation holds the member `{method}`");
}

// -------------------------------------------------------------------------------------------
// The anchors: every condition position writes the postfix expression.
// -------------------------------------------------------------------------------------------

/// Every condition position writes the postfix expression, on both legs, and no recovering member
/// keeps a quote: a position that still quoted one would be a partial recovery counted as one.
#[test]
fn the_condition_positions_write_the_postfix_expression_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CP7"));
        let report = presented(&snapshot, "CP7");
        for anchor in CP7_SCAN.iter().chain(CP7_FIND).chain(CP7_COND) {
            assert!(
                report.text.contains(anchor),
                "`CP7` on {} lost the condition anchor {anchor:?}:\n{}",
                leg.label,
                report.text
            );
        }
        let snapshot = open(&leg.fixture("CC"));
        let report = presented(&snapshot, "CC");
        for anchor in CC_SCAN.iter().chain(CC_FIND).chain(CC_COND) {
            assert!(
                report.text.contains(anchor),
                "`CC` on {} lost the condition anchor {anchor:?}:\n{}",
                leg.label,
                report.text
            );
        }
    }
}

/// The recovering members present every instruction they have.
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
        }
    }
}

/// The compound do-while chain the region layer already presented — no postfix position in it —
/// stays byte-identical: the chain rules the condition position lands in must not move it.
#[test]
fn the_control_chain_stays_byte_identical_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CN"));
        let report = presented(&snapshot, "CN");
        let body = method_body(&report.text, "chain(II)I");
        for line in CN_CHAIN {
            assert!(
                body.contains(line),
                "`CN.chain` on {} lost the control line {line:?}:\n{body}",
                leg.label
            );
        }
        assert!(
            !report.text.contains("local2++") && !report.text.contains("++local2"),
            "`CN.chain` on {} turned the compound into a postfix expression:\n{}",
            leg.label,
            report.text
        );
    }
}

// -------------------------------------------------------------------------------------------
// The negatives keep their refusals.
// -------------------------------------------------------------------------------------------

/// The two negatives this slice states keep their refusals, and the region they are refused under
/// is the slice's own bound. The A-phase traps keep theirs verbatim: none of them moved for a
/// condition position to be presented.
#[test]
fn the_negatives_keep_their_refusals_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CN"));
        let report = presented(&snapshot, "CN");
        for (method, refusal) in CN_NEGATIVES {
            assert!(
                report.text.contains(refusal),
                "`CN.{method}` on {} lost the refusal {refusal:?}:\n{}",
                leg.label,
                report.text
            );
            let body = method_body(&report.text, method);
            assert!(
                body.contains("not recovered"),
                "`CN.{method}` on {} is not quoted whole:\n{body}",
                leg.label
            );
            assert!(
                !body.contains("++") && !body.contains("--"),
                "`CN.{method}` on {} presented a postfix expression the bound refuses:\n{body}",
                leg.label
            );
        }
        // The fence states *why* the chain was refused, and it is the slice's own bound.
        assert!(
            region_codes(&report, "twoVariables([I[I)I").contains(&CHAIN_POSITION_BOUND.to_owned())
                && region_codes(&report, "midChain([I)I")
                    .contains(&CHAIN_POSITION_BOUND.to_owned()),
            "`CN` on {} does not state the chain position bound:\n{}",
            leg.label,
            report.text
        );

        // The A-phase traps, read from the fixture that owns them.
        let files: &[(&str, &[u8])] = if leg.label.starts_with("javac 23") {
            A_PHASE_TRAP_FILES
        } else {
            A_PHASE_TRAP_FILES_JAVAC8
        };
        let entries: Vec<(&[u8], &[u8])> = files
            .iter()
            .map(|(name, bytes)| (name.as_bytes(), *bytes))
            .collect();
        let traps = open(&zip_of(&entries));
        let report = presented(&traps, "NG");
        for (method, refusal) in A_PHASE_TRAPS {
            assert!(
                report.text.contains(refusal),
                "the A-phase trap `NG.{method}` on {} lost the refusal {refusal:?}:\n{}",
                leg.label,
                report.text
            );
            let body = method_body(&report.text, method);
            assert!(
                body.contains("not recovered"),
                "the A-phase trap `NG.{method}` on {} is not quoted whole:\n{body}",
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
        std::env::temp_dir().join(format!("jarde-postfix-condition-{label}-{stamp}-{ordinal}"));
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
            // `CN` is the negatives' class: its two refused bodies are quoted whole, so its
            // stripped text is the *safe* form the soundness invariant asks for — a class a reader
            // cannot compile, never one that compiles and behaves differently. The refusals are
            // pinned by the tests above; the replay's job for it is exactly that it does not
            // compile.
            if class == "CN" {
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
