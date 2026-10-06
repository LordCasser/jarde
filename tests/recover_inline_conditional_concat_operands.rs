//! `recover-inline-conditional-concat-operands` in one frozen target: the **inline conditional
//! value as a `+` chain operand** (`"" + a + (x == y) + b`) on **both** compiler legs (javac 23.0.1
//! `--release 8` and real javac 8, Corretto 1.8.0_432, from the same sources).
//!
//! javac lowers a comparison used as a chain operand to a branch whose two arms push the `0`/`1`
//! the join's stack Phi merges — so the chain's `toString` stands **in another block**, and
//! `concat@1`'s ordinary same-block walk refused the whole chain with `jre_concat_split`
//! ("the concatenation that starts at BCI 11 ends in the `toString` at BCI 94, which is in another
//! block"). The change adds one bounded certificate: when the blocks in between are **exactly** one
//! proved conditional materialization — two arms that each push one constant and nothing else, one
//! join — the chain is owned end to end and written in source form, with the comparison inlined at
//! the `append` argument position through the boolean channel that already spells it
//! (`recover-ref-eq-boolean-argument`'s presentation).
//!
//! The fixtures are this change's own `ICC` (the reference-equality form: a nested class, a
//! qualified-`this` return and `==` against the outer instance — the frozen patrol anchor `NI`),
//! `ICB` (the same shape on a class of its own, with a `getfield` operand in the middle of the
//! chain) and `ICQ` (the minimal form with no local reads at all — the frozen patrol probe `CMP`);
//! the control `ICM` is the same chain **without** the branch (the frozen probe `NMA`), whose text
//! is pinned byte-for-byte because a chain with no cut is not this change's shape; and `ICN` holds
//! the four shapes the certificate must refuse, one method each: an arm that calls, an arm that
//! stores, a nested double comparison and a chain whose branch is covered by an exception table.
//!
//! The tests pin the presented texts, keep every refusal verbatim, and the ignored replay strips
//! the presentations the way the patrols' own stripped sources were made (comment lines dropped),
//! compiles each anchor with the installed `javac --release 8` and, when a real javac 8 is present,
//! with that one too, runs both under `-Xverify:all` and compares every answer with the fixture's
//! own class files.

use jarde::class_source::{ClassSourceOutcome, ClassSourceReport};
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

/// javac 23.0.1, `javac --release 8 -g -nowarn -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "ICC.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8/ICC.class"),
    ),
    (
        "ICC$Inner.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8/ICC$Inner.class"),
    ),
    (
        "ICB.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8/ICB.class"),
    ),
    (
        "ICQ.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8/ICQ.class"),
    ),
    (
        "ICM.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8/ICM.class"),
    ),
    (
        "ICN.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8/ICN.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -g -nowarn -d v8-javac8 *.java`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "ICC.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8-javac8/ICC.class"),
    ),
    (
        "ICC$Inner.class",
        include_bytes!(
            "fixtures/recover-inline-conditional-concat-operands/v8-javac8/ICC$Inner.class"
        ),
    ),
    (
        "ICB.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8-javac8/ICB.class"),
    ),
    (
        "ICQ.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8-javac8/ICQ.class"),
    ),
    (
        "ICM.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8-javac8/ICM.class"),
    ),
    (
        "ICN.class",
        include_bytes!("fixtures/recover-inline-conditional-concat-operands/v8-javac8/ICN.class"),
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
const CLASSES: &[&str] = &["ICC", "ICB", "ICQ", "ICM", "ICN"];

/// The one diagnostic this change exists to remove from the anchors.
const SPLIT: &str = "jre_concat_split";

/// The ordinary walk's own refusal for a chain whose operand is a field read: the control keeps it.
const INTERLEAVED: &str = "jre_concat_interleaved_effect";

// -------------------------------------------------------------------------------------------
// The presented texts.
// -------------------------------------------------------------------------------------------

/// `ICC.main` — the reference-equality form: the chain is written as a `+` chain and the comparison
/// is inlined at the last `append`'s argument position.
const ICC_CHAIN: &str = "        java.lang.System.out.println(\"\" + n.make(3).v + \"/\" + n.make(2).outerTag() + \"/\" + externalMake(n, 1).outerRef().tag + \"/\" + (externalMake(n, 1).outerRef() == n));";

/// `ICB.main` — the same shape with a `getfield` operand in the middle of the chain.
const ICB_CHAIN: &str = "        java.lang.System.out.println(\"\" + n.add(1) + \"/\" + n.add(2) + \"/\" + n.self().tag + \"/\" + (n.self() == n));";

/// `ICQ.main` — the minimal form: two local reads and one comparison, nothing else.
const ICQ_CHAIN: &str =
    "        java.lang.System.out.println(\"eq=\" + (p == q) + \" tag=\" + p.hashCode());";

/// `ICM.main` — the control: the same chain **without** the branch. Its `getfield` operands make
/// the ordinary walk refuse it, so the fallback writes the builder chain itself; that text is not
/// this change's business and stays byte-for-byte what it was.
const ICM_CONTROL: &str = "        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(\"\").append(n.add(1)).append(\"/\").append(n.add(2)).append(\"/\").append(n.self().tag).append(\"/\").append(n.tag).toString());";

/// The anchors' methods, which must present every instruction they have.
const NO_QUOTE_METHODS: &[(&str, &str)] = &[
    ("ICC", "([Ljava/lang/String;)V"),
    ("ICB", "([Ljava/lang/String;)V"),
    ("ICQ", "([Ljava/lang/String;)V"),
    ("ICM", "([Ljava/lang/String;)V"),
];

/// `ICN`'s four negatives: the method's own name, the descriptor it declares, and the shape that
/// keeps the refusal.
const NEGATIVES: &[(&str, &str, &str)] = &[
    ("armCall", "(Z)V", "the cut is a zero test whose arms call"),
    ("armStore", "(Z)V", "the cut's arms store"),
    (
        "nested",
        "(Ljava/lang/Object;Ljava/lang/Object;Ljava/lang/Object;Ljava/lang/Object;)V",
        "the join block holds a second comparison",
    ),
    (
        "guarded",
        "(Ljava/lang/Object;Ljava/lang/Object;)V",
        "the branch's block carries an exceptional edge",
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
            &RecoveryEvidenceRequest::essential()
                .with_kind(RecoveryEvidenceKind::SourceMap)
                .with_kind(RecoveryEvidenceKind::RuleDetails),
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

/// One member, found by the raw name and descriptor its own table entry states.
fn member<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a jarde::class_source::ClassSourceMethod {
    for method in &report.methods {
        if method.item.name.raw().0 == name.as_bytes()
            && method.item.descriptor.raw().0 == descriptor.as_bytes()
        {
            return method;
        }
    }
    panic!(
        "the presentation holds a member for `{name}{descriptor}`:\n{}",
        report.text
    );
}

/// One member's own recovery report.
fn member_report<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a RecoveryReport {
    if let ClassSourceOutcome::Recovered { report, .. } = &member(report, name, descriptor).outcome
    {
        return report;
    }
    panic!(
        "the presentation holds a recovery run for `{name}{descriptor}`:\n{}",
        report.text
    );
}

/// One member's own text in the assembled source.
fn member_text<'a>(report: &'a ClassSourceReport, name: &str, descriptor: &str) -> &'a str {
    &member(report, name, descriptor).text
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

// -------------------------------------------------------------------------------------------
// The anchors: the chain is presented in source form, and no `jre_concat_split` is left.
// -------------------------------------------------------------------------------------------

/// Every anchor is written on both legs, with the comparison inlined at the `append` argument
/// position, and its method quotes nothing.
#[test]
fn the_inline_conditional_anchors_write_the_source_form_on_both_legs() {
    for leg in LEGS {
        for (class, chain) in [("ICC", ICC_CHAIN), ("ICB", ICB_CHAIN), ("ICQ", ICQ_CHAIN)] {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            assert!(
                report.text.contains(chain),
                "`{class}` on {} lost the source-form chain {chain:?}:\n{}",
                leg.label,
                report.text
            );
            // The raw builder form the fallback writes is what this change replaces: an anchor that
            // still carries it is not a presentation of the chain.
            assert!(
                !report.text.contains("new java.lang.StringBuilder()"),
                "`{class}` on {} still writes the builder chain:\n{}",
                leg.label,
                report.text
            );
            let method = member_report(&report, "main", "([Ljava/lang/String;)V");
            assert!(
                method.concats.iter().any(|concat| concat.presented()),
                "`{class}` on {} presents a chain: {:?}",
                leg.label,
                method.concats
            );
            assert!(
                method.concats.iter().all(|concat| concat
                    .refusal
                    .as_ref()
                    .map(|refusal| refusal.code)
                    != Some(SPLIT)),
                "`{class}` on {} keeps a `{SPLIT}` refusal: {:?}",
                leg.label,
                method.concats
            );
        }
    }
}

/// The recovering methods present every instruction they have: an anchor that still quotes one
/// would be a partial recovery counted as one.
#[test]
fn no_recovering_method_keeps_a_quote() {
    for leg in LEGS {
        for &(class, descriptor) in NO_QUOTE_METHODS {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let body = member_text(&report, "main", descriptor);
            let quoted = quoted_bcis(body);
            assert!(
                quoted.is_empty(),
                "`{class}{descriptor}` on {} still quotes {quoted:?}:\n{body}",
                leg.label
            );
            assert!(
                !body.contains("not recovered"),
                "`{class}{descriptor}` on {} is not a whole recovery:\n{body}",
                leg.label
            );
        }
    }
}

// -------------------------------------------------------------------------------------------
// The control: the same chain without the branch is byte-identical.
// -------------------------------------------------------------------------------------------

/// `ICM` — the chain with no cut: the ordinary walk refuses it at its `getfield` operand and the
/// fallback writes the builder chain itself. That text is pinned exactly, and the refusal is the
/// walk's own, not the split attribution's.
#[test]
fn the_chain_without_a_cut_keeps_its_own_presentation_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("ICM"));
        let report = presented(&snapshot, "ICM");
        assert!(
            report.text.contains(ICM_CONTROL),
            "`ICM` on {} lost its control text {ICM_CONTROL:?}:\n{}",
            leg.label,
            report.text
        );
        let method = member_report(&report, "main", "([Ljava/lang/String;)V");
        assert!(
            method
                .concats
                .iter()
                .all(|concat| concat.refusal.as_ref().map(|refusal| refusal.code) != Some(SPLIT)),
            "`ICM` on {} is a chain with no cut, so the split attribution says nothing about it: {:?}",
            leg.label,
            method.concats
        );
        assert!(
            method
                .concats
                .iter()
                .any(|concat| concat.refusal.as_ref().map(|refusal| refusal.code)
                    == Some(INTERLEAVED)),
            "`ICM` on {} keeps the ordinary walk's own refusal: {:?}",
            leg.label,
            method.concats
        );
    }
}

// -------------------------------------------------------------------------------------------
// The negatives: every shape keeps its `jre_concat_split`, verbatim.
// -------------------------------------------------------------------------------------------

/// The four shapes the certificate must not admit keep the split refusal, and their bodies stay
/// quoted whole.
#[test]
fn the_negatives_keep_their_split_refusals_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("ICN"));
        let report = presented(&snapshot, "ICN");
        for &(method, descriptor, why) in NEGATIVES {
            let signature = format!("{method}{descriptor}");
            let body = member_text(&report, method, descriptor);
            assert!(
                body.contains(&format!(
                    "not recovered: the recovery run for `{signature}` produced no statement"
                )),
                "`ICN.{method}` on {} is not quoted whole ({why}):\n{body}",
                leg.label
            );
            let run = member_report(&report, method, descriptor);
            let split: Vec<&str> = run
                .concats
                .iter()
                .filter_map(|concat| concat.refusal.as_ref())
                .filter(|refusal| refusal.code == SPLIT)
                .map(|refusal| refusal.message.as_str())
                .collect();
            assert_eq!(
                split.len(),
                1,
                "`ICN.{method}` on {} keeps exactly one `{SPLIT}` refusal ({why}): {:?}",
                leg.label,
                run.concats
            );
            assert!(
                split[0].contains("which is in another block"),
                "`ICN.{method}` on {} names the cut in the refusal's own words: {}",
                leg.label,
                split[0]
            );
            assert!(
                split[0].contains("the concatenation that starts at BCI"),
                "`ICN.{method}` on {} names the chain's own head: {}",
                leg.label,
                split[0]
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
    let path = std::env::temp_dir().join(format!("jarde-inline-concat-{label}-{stamp}-{ordinal}"));
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

/// The same answer with the JVM's **identity hash** normalized.
///
/// `Object.hashCode()` is the identity hash, and it differs between runs of the *same* class file:
/// the one fixture that prints it (`ICQ`, the frozen patrol probe `CMP`) is therefore compared with
/// that number masked out — the recovered text must print the same shape and the same deterministic
/// parts, and a run whose `tag=` value changed kind is still caught. Every other anchor's answer is
/// compared exactly.
fn without_identity_hash(answer: &str) -> String {
    answer
        .lines()
        .map(|line| match line.split_once("tag=") {
            Some((head, _)) => format!("{head}tag=<identity hash>"),
            None => line.to_owned(),
        })
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
            // `ICN` is the negatives' own class: every body it holds is quoted, so its stripped text
            // is the *safe* form the soundness invariant asks for — a method whose body a reader
            // cannot compile, never one that compiles and behaves differently. The refusal texts are
            // pinned by the test above; the replay's job for it is exactly that it does not compile.
            if class == "ICN" {
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
                without_identity_hash(&got),
                without_identity_hash(&want),
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
                    without_identity_hash(&got8),
                    without_identity_hash(&want),
                    "`{class}` on {} answers differently under javac 8:\n{text}",
                    leg.label
                );
            }
        }
    }
}
