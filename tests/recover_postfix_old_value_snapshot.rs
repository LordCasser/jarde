//! `recover-postfix-old-value-snapshot` in one frozen target: the postfix old-value **consumer
//! positions** on **both** compiler legs (javac 23.0.1 `--release 8` and real javac 8, Corretto
//! 1.8.0_432, from the same sources).
//!
//! javac reads an incremented local or field *before* it updates it whenever the value is consumed,
//! and the old value is an SSA value of its own whose consumer reads it after the update ran. The
//! change presents that value where it is read — `x++` — and absorbs the update instruction into
//! that expression, instead of refusing the read (`int j = i++;` was "the value at BCI N is the
//! value local 0 held at BCI M") or quoting the dance (`elems[size++] = t` was "the dependency
//! chain from BCI 1 to final consumer 16 is not bounded").
//!
//! The fixtures are the patrols' own frozen anchors — `CM`/`CM2` from the postfix-old-value patrol,
//! `AD`/`GA` from the array-store soundness patrol (the `AD`/`CM`/`GA` class files are byte-identical
//! to the patrols' jars, checked by SHA in the fixture README) — plus this change's `PT` (the static
//! field as an array index inside a ternary arm), `SR` (the array store's own right side and its
//! local index write), `IX` (the local and static-field index positions) and `NG` (the negatives:
//! the two self-assignment traps, the field self-assignment, the multi-consumer form and a Phase-B
//! condition position).
//!
//! The tests pin the presented texts whole, keep every negative's refusal verbatim, keep the three
//! healthy shapes byte-identical, and the ignored replay strips the presentations the way the
//! patrols' own stripped sources were made (comment lines dropped), compiles each anchor with the
//! installed `javac --release 8` and, when a real javac 8 is present, with that one too, runs both
//! under `-Xverify:all` and compares every answer with the fixture's own class files.

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

/// javac 23.0.1, `javac --release 8 -Xlint:-options -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "CM.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8/CM.class"),
    ),
    (
        "CM2.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8/CM2.class"),
    ),
    (
        "AD.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8/AD.class"),
    ),
    (
        "GA.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8/GA.class"),
    ),
    (
        "GA$Cfg.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8/GA$Cfg.class"),
    ),
    (
        "PT.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8/PT.class"),
    ),
    (
        "SR.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8/SR.class"),
    ),
    (
        "IX.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8/IX.class"),
    ),
    (
        "NG.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8/NG.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "CM.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8-javac8/CM.class"),
    ),
    (
        "CM2.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8-javac8/CM2.class"),
    ),
    (
        "AD.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8-javac8/AD.class"),
    ),
    (
        "GA.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8-javac8/GA.class"),
    ),
    (
        "GA$Cfg.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8-javac8/GA$Cfg.class"),
    ),
    (
        "PT.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8-javac8/PT.class"),
    ),
    (
        "SR.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8-javac8/SR.class"),
    ),
    (
        "IX.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8-javac8/IX.class"),
    ),
    (
        "NG.class",
        include_bytes!("fixtures/recover-postfix-old-value-snapshot/v8-javac8/NG.class"),
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
const CLASSES: &[&str] = &["CM", "CM2", "AD", "GA", "PT", "SR", "IX", "NG"];

// -------------------------------------------------------------------------------------------
// The presented texts, pinned whole (the p3 surface tests' own convention).
// -------------------------------------------------------------------------------------------

/// `CM.incDec` — the four-shape discriminator the postfix patrol froze: the two *value-consuming*
/// forms recover as the postfix expression, the prefix forms stay their own statements.
const CM_INCDEC: &[&str] = &[
    "        int local1 = local0++;",
    "        int local3 = local0--;",
    "        return local0 + local1 + local2 + local3 + local4;",
];

/// `CM.compound`/`compoundInExpr`/`compoundField`/`incField` — the healthy shapes the patrol
/// recorded as already recovered: compound chains and in-expression compounds, static field
/// compounds and the field prefix/postfix pair. They must stay byte-identical.
const CM_HEALTHY: &[&str] = &[
    "        local0 = local0 + 5;",
    "        local0 = local0 - 3;",
    "        local0 = local0 * 2;",
    "        local0 = local0 / 4;",
    "        local0 = local0 % 4;",
    "        int local1 = local0;",
    "        CM.a = CM.a + 3;",
    "        CM.b = CM.b - 1;",
    "        CM.a = CM.a + 1;",
];

/// `CM2.immUse`/`postfixExpr` — the patrol's two refused local shapes, now the postfix expression.
const CM2_POSTFIX: &[&str] = &[
    "        int local1 = local0++;",
    "        return local0++ + 10;",
];

/// `CM2.crossStmt`/`prefix` — the split-statement and prefix controls, byte-identical.
const CM2_HEALTHY: &[&str] = &[
    "        local0 = local0 + 1;",
    "        int local1 = local0;",
    "        return local1;",
    "        return local0 + 10;",
];

/// `AD.add` — the flagship: the array reference survives the field `putfield` dance and the whole
/// store is one statement. The `null/null vs x/y` compilable-wrong face closes here.
const AD_ADD: &str = "        this.elems[this.size++] = arg1;";

/// `GA.add` — the same shape over the generic collection's own `Object[]` field.
const GA_ADD: &str = "        this.elems[this.size++] = arg1;";

/// `PT.read` — the static field's postfix as an array index inside a ternary arm (the shape the
/// conditional-arm patrol recorded as "the conditional arm contains an independent instruction").
const PT_READ: &str = "        return PT.pos < PT.src.length ? PT.src[PT.pos++] : null;";

/// `SR.arrSelf`/`backWrite` — the array store's own right side is the old value, and the local
/// postfix as the store's index (`a[i] = i++`, the postfix-self-assign patrol's behavior trap).
const SR_SHAPES: &[&str] = &[
    "        local1[local0] = local0++;",
    "        return local1[1] * 100 + local0;",
    "        local1[local0--] = local1[0] + 100;",
    "        return local1[2] * 1000 + local0;",
];

/// `IX` — the four index positions: the local postfix as an array index (write and read) and the
/// static field's postfix as an array index (write and read).
const IX_SHAPES: &[&str] = &[
    "        local1[local0++] = 10;",
    "        return local1[local0--];",
    "        IX.arr[IX.idx++] = 20;",
    "        return IX.arr[IX.idx--];",
];

/// Every recovering anchor of the Phase-A matrix, class by class.
const PHASE_A: &[(&str, &[&str])] = &[
    ("CM", CM_INCDEC),
    ("CM2", CM2_POSTFIX),
    ("AD", &[AD_ADD]),
    ("GA", &[GA_ADD]),
    ("PT", &[PT_READ]),
    ("SR", SR_SHAPES),
    ("IX", IX_SHAPES),
];

/// The negatives, verbatim: the two local self-assignment traps, the field self-assignment, the
/// multi-consumer form and a Phase-B condition position. Each keeps its own refusal.
const NG_NEGATIVES: &[(&str, &str)] = &[
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
    (
        "condShape",
        "// local 0 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
    ),
];

/// The refusals the recovering classes must not keep: a phase-A method that still quotes its own
/// instructions would be a partial recovery counted as one.
const NO_QUOTE_METHODS: &[(&str, &str)] = &[
    ("CM", "incDec()I"),
    ("CM2", "immUse()I"),
    ("CM2", "postfixExpr()I"),
    ("AD", "add(Ljava/lang/Object;)V"),
    ("GA", "add(Ljava/lang/Object;)V"),
    ("PT", "read()Ljava/lang/String;"),
    ("SR", "arrSelf()I"),
    ("SR", "backWrite()I"),
    ("IX", "localWrite()V"),
    ("IX", "localRead()I"),
    ("IX", "fieldWrite()V"),
    ("IX", "fieldRead()I"),
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
// The Phase-A matrix: every anchor recovers, and no recovering method keeps a quote.
// -------------------------------------------------------------------------------------------

/// Every Phase-A position writes the postfix expression, on both legs.
#[test]
fn the_phase_a_positions_write_the_postfix_expression_on_both_legs() {
    for leg in LEGS {
        for &(class, anchors) in PHASE_A {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            for anchor in anchors {
                assert!(
                    report.text.contains(anchor),
                    "`{class}` on {} lost the postfix anchor {anchor:?}:\n{}",
                    leg.label,
                    report.text
                );
            }
        }
    }
}

/// The recovering methods present every instruction they have: a Phase-A anchor that still quotes
/// one would be a partial recovery counted as one.
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

/// The three healthy shapes the patrol recorded — prefix, split-statement and compound — stay
/// byte-identical, and so do the two controls the recovering classes carry.
#[test]
fn the_healthy_shapes_stay_byte_identical_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CM"));
        let report = presented(&snapshot, "CM");
        for line in CM_HEALTHY {
            assert!(
                report.text.contains(line),
                "`CM` on {} lost the healthy shape {line:?}:\n{}",
                leg.label,
                report.text
            );
        }
        assert!(
            !report.text.contains("local0++ +") && !report.text.contains("local0-- +"),
            "`CM` on {} turned a compound into a postfix expression:\n{}",
            leg.label,
            report.text
        );

        let snapshot = open(&leg.fixture("CM2"));
        let report = presented(&snapshot, "CM2");
        for line in CM2_HEALTHY {
            assert!(
                report.text.contains(line),
                "`CM2` on {} lost the healthy shape {line:?}:\n{}",
                leg.label,
                report.text
            );
        }
        assert!(
            quoted_bcis(&method_body(&report.text, "crossStmt()I")).is_empty()
                && quoted_bcis(&method_body(&report.text, "prefix()I")).is_empty(),
            "`CM2`'s split-statement and prefix controls present every instruction on {}:\n{}",
            leg.label,
            report.text
        );
    }
}

// -------------------------------------------------------------------------------------------
// The negatives keep their refusals.
// -------------------------------------------------------------------------------------------

/// The self-assignment traps, the multi-consumer form and the Phase-B condition position keep
/// exactly the refusals they had: this slice's Non-Goal.
#[test]
fn the_negatives_keep_their_refusals_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("NG"));
        let report = presented(&snapshot, "NG");
        for (method, refusal) in NG_NEGATIVES {
            assert!(
                report.text.contains(refusal),
                "`NG.{method}` on {} lost the refusal {refusal:?}:\n{}",
                leg.label,
                report.text
            );
            let body = method_body(&report.text, method);
            assert!(
                body.contains("not recovered"),
                "`NG.{method}` on {} is not quoted whole:\n{body}",
                leg.label
            );
        }
        // No negative writes a postfix expression, and none of the increments they refuse is
        // presented as its own statement either — the whole method is quoted.
        for line in report.text.lines().filter(|line| !line.contains("//")) {
            assert!(
                !line.contains("++") && !line.contains("--"),
                "`NG` on {} presented a postfix expression the Non-Goal refuses: {line:?}",
                leg.label
            );
        }
        // `i = i++`/`i = i--` still carry the store-back form's own refusal: the value read back
        // into the slot it was read from is not claimed.
        assert!(
            report
                .text
                .matches("the value at BCI 6 is the value local 0 held at BCI 2")
                .count()
                >= 2,
            "`NG` on {} lost one of the two local self-assignment refusals:\n{}",
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
    let path =
        std::env::temp_dir().join(format!("jarde-postfix-snapshot-{label}-{stamp}-{ordinal}"));
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
            // `GA`'s `main` names its nested interface through the pool spelling (`GA$Cfg`), which
            // is a pre-existing presentation of that member and not this slice's business: its
            // anchor is verified through the isolated `add` probe in the change's results, and the
            // whole-class replay covers the other seven.
            if class == "GA" {
                continue;
            }
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let text = stripped(&report);
            // `NG` is the negatives' own class: every body it holds is quoted, so its stripped text
            // is the *safe* form the soundness invariant asks for — a method whose body a reader
            // cannot compile, never one that compiles and behaves differently. The refusal texts
            // are pinned by the tests above; the replay's job for it is exactly that it does not
            // compile.
            if class == "NG" {
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
