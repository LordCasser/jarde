//! `recover-short-circuit-local-branch-reads` in one frozen target: the **branch condition
//! position** of a proved short-circuit boolean local, on **both** compiler legs (javac 23.0.1
//! `--release 8` and real javac 8, Corretto 1.8.0_432, from the same sources).
//!
//! `proves_boolean_local_store` admits a read of the chain's stored local only where that read is
//! an explicit Boolean consumer. javac lowers `b ? x : y`, the statement `if (b)` and the loop
//! `while (b)` to an `ifeq`/`ifne` on the local's own load, and that condition position was the
//! missing arm: the whole chain refused (`the short-circuit chain from BCI 4 through 8 reaches a
//! shared value consumer at BCI 16`) and the local stayed `int`. The arm this change adds names
//! the two zero-tests by identity, so the load keeps the position it already has — the branch is
//! its consumer, and nothing is reordered.
//!
//! The patrol anchor is `OP2.condAssignOld` (`boolean b = (x += 1) > 0 && x > 0; return b ? x :
//! -1;`, the census re-run's largest single residual point). Its class's own `main` is a
//! *registered residual* of another family — the concatenation chain's saved producers — and this
//! slice leaves it refused; the replay below therefore substitutes the frozen source's own `main`
//! body for the refused-body marker, so the line it measures is the recovered methods' answers
//! (including `condAssignOld(0)`), not a fabricated body.
//!
//! The change's own fixtures are `BranchReads` (the ternary, the `if` statement and the
//! mid-chain read, which the arm also reaches) and `BranchReadNegatives` (the boundaries it must
//! not cross: the **loop** condition position, whose read sits in the loop's own region path and
//! is refused by the same gate's cross-region criterion *before* the whitelist, and the
//! catch-crossing form, refused before the gate at region ownership). Two hand-patched controls
//! pin the arm's width: the same class with `ifeq` at BCI 18 patched to `iflt` stays refused,
//! with it patched to `ifne` it presents the inverted sense as `if (!b)`.
//!
//! ```text
//! default suite:  cargo test --test recover_short_circuit_local_branch_reads --locked
//! replay:         cargo test --test recover_short_circuit_local_branch_reads --locked -- --ignored
//! ```

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

/// The compression every fixture entry uses: a stored entry, so the archive's bytes are the class
/// files' own.
const STORE: u16 = 0;

// -------------------------------------------------------------------------------------------
// The patrol anchor.
// -------------------------------------------------------------------------------------------

/// The operator-remainder patrol's frozen `OP2` (`javac --release 8`, default debug info).
const OP2_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-05/operator-remainder-patrol/fixture/op2.jar"
);
/// The same class's source, frozen beside the jar. The replay reads its `main` body from here —
/// never from a transcription — and asserts that the body it reads is the one the line measures.
const OP2_SOURCE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-10-05/operator-remainder-patrol/fixture/OP2.java"
);
/// The driver the replay substitutes for the refused-body marker: `OP2`'s own `main` body, verbatim.
const OP2_DRIVER: &str = r#"System.out.println(""+shl(3)+"/"+shr(-8)+"/"+ushr(-8)+"/"+lshl(2L)+"/"+nanf()+"/"+pinf()+"/"+ninf()+"/"+(pzero()==0.0f)+"/"+condAssign(0)+"/"+condAssignOld(0));"#;
/// What the frozen class's own `main` prints, measured by running it (JDK 23.0.1,
/// `java -Xverify:all -cp <the jar's OP2.class> OP2`).
const OP2_GOLDEN: &str = "12/-4/2147483644/16/NaN/Infinity/-Infinity/true/true/1";
/// The diagnostic this change exists to remove from the anchor.
const OP2_REFUSAL: &str =
    "the short-circuit chain from BCI 4 through 8 reaches a shared value consumer at BCI 16";

// -------------------------------------------------------------------------------------------
// The change's own fixtures and the two compiler legs.
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names and the class files it produced.
struct Leg {
    label: &'static str,
    files: &'static [(&'static str, &'static [u8])],
}

impl Leg {
    /// The one class's bytes, asserted present under the name the leg committed.
    fn class(&self, name: &str) -> &'static [u8] {
        self.files
            .iter()
            .find(|(file, _)| *file == name)
            .unwrap_or_else(|| panic!("{}: the fixture `{name}` is committed", self.label))
            .1
    }

    /// One class's own container, so the request reads it the way a jar's entry is read.
    fn fixture(&self, name: &str) -> Vec<u8> {
        let entry = format!("{name}.class");
        zip_of(&[(entry.as_bytes(), self.class(&entry))])
    }
}

/// javac 23.0.1, `javac --release 8 -g -nowarn -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "BranchReads.class",
        include_bytes!("fixtures/recover-short-circuit-local-branch-reads/v8/BranchReads.class"),
    ),
    (
        "BranchReadNegatives.class",
        include_bytes!(
            "fixtures/recover-short-circuit-local-branch-reads/v8/BranchReadNegatives.class"
        ),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -g -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "BranchReads.class",
        include_bytes!(
            "fixtures/recover-short-circuit-local-branch-reads/v8-javac8/BranchReads.class"
        ),
    ),
    (
        "BranchReadNegatives.class",
        include_bytes!(
            "fixtures/recover-short-circuit-local-branch-reads/v8-javac8/BranchReadNegatives.class"
        ),
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

/// The two hand-patched controls: `v8/BranchReads.class` with the `ifeq` at BCI 18 — the branch
/// that tests the chain's stored local in `ifStatement` — replaced by a numeric test and by the
/// inverted zero-test. Each file declares the class name `BranchReads`, exactly as the class it
/// was patched from does, so it is read under the name it states.
const CONTROL_NUMERIC_BRANCH: &[u8] = include_bytes!(
    "fixtures/recover-short-circuit-local-branch-reads/controls/NumericBranch.class"
);
const CONTROL_NOT_ZERO_BRANCH: &[u8] = include_bytes!(
    "fixtures/recover-short-circuit-local-branch-reads/controls/NotZeroBranch.class"
);

/// The condition position's own presentation, on both legs.
const TERNARY: &[&str] = &[
    "        x = x + 1;",
    "        boolean b = x > 0 && x > 0;",
    "        return b ? x : -1;",
];
const IF_STATEMENT: &[&str] = &[
    "        x = x + 1;",
    "        boolean b = x > 0 && x > 0;",
    "        if (b) {",
    "            return x;",
    "        } else {",
    "            return -1;",
];
/// The mid-chain read: the stored local is the **middle** test of a second chain, and the arm
/// reaches it too. The nested spelling is the second chain's own region composition, and it
/// evaluates in the source's order.
const MID_CHAIN: &[&str] = &[
    "        x = x + 1;",
    "        boolean b = x > 0 && x > 0;",
    "        return x > 0 && (b && x < 100);",
];

/// The loop condition position's refusal — the same gate's cross-region criterion, reached before
/// the consumer whitelist: the loop's header is a canonical block of its own, so the read's region
/// path (`[1]`) is not the declaration's (`[0]`).
const LOOP_REFUSAL: &str =
    "// the short-circuit chain from BCI 4 through 8 reaches a shared value consumer at BCI 16";
/// The catch-crossing form's refusal, one layer earlier: the region tree gives block BCI 18 two
/// owners, so the whole method is quoted before the gate is ever asked.
const CROSS_CATCH_REFUSAL: &str = "// canonical block at BCI 18 on jsr path [] has more than one owner in the completed Region tree";

/// What `BranchReads`'s own `main` prints, measured by running the committed class (JDK 23.0.1,
/// `java -Xverify:all`).
const BRANCH_READS_GOLDEN: &str = "1\n-1\n1\n-1\ntrue\nfalse\nstatus=0";

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

/// The body of a member the request reports on, by the name the class file states.
fn member_body<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("the class has a member `{name}`"))
        .text
        .as_str()
}

// -------------------------------------------------------------------------------------------
// The patrol anchor, and the residual beside it.
// -------------------------------------------------------------------------------------------

/// `OP2.condAssignOld` recovers at the branch condition position, and the class's own `main` —
/// another family's registered residual — keeps the refusal this slice does not touch.
#[test]
fn the_patrol_anchor_recovers_at_the_branch_condition_position() {
    let snapshot = open(OP2_JAR);
    let report = presented(&snapshot, "OP2");
    let body = method_body(&report.text, "static int condAssignOld(int arg0)");
    for line in [
        "        arg0 = arg0 + 1;",
        "        boolean local1 = arg0 > 0 && arg0 > 0;",
        "        return local1 ? arg0 : -1;",
    ] {
        assert!(body.contains(line), "the anchor lost {line:?}:\n{body}");
    }
    assert!(
        !body.contains("@bytecode") && !body.contains("not recovered"),
        "the anchor is quoted, not recovered:\n{body}"
    );
    assert!(
        !report.text.contains(OP2_REFUSAL),
        "the anchor's refusal is still in the class:\n{}",
        report.text
    );
    // The two neighbours the whitelist already admitted, unchanged by this change: the direct
    // `ireturn` of a chain and the chain stored and consumed by a concatenation.
    let cond_assign = method_body(&report.text, "static boolean condAssign(int arg0)");
    assert!(
        cond_assign.contains("return arg0 + 1 > 0;"),
        "the direct-return neighbour moved:\n{cond_assign}"
    );
    // `main` is the concatenation chain's own residual: refused before and after this change, and
    // its refused body is stated rather than silently dropped.
    let main = method_body(
        &report.text,
        "public static void main(java.lang.String[] arg0)",
    );
    assert!(
        main.contains("not recovered") && main.contains("jarde_refused_body();"),
        "the registered residual beside the anchor moved:\n{main}"
    );
    assert!(
        main.contains("the saved producer at BCI 0 has 3 consumers"),
        "the residual's own diagnosis moved:\n{main}"
    );
}

// -------------------------------------------------------------------------------------------
// The condition positions the arm admits.
// -------------------------------------------------------------------------------------------

/// The ternary, the `if` statement and the mid-chain read present on both legs, and every one of
/// them presents every instruction it has.
#[test]
fn the_branch_condition_positions_recover_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BranchReads"));
        let report = presented(&snapshot, "BranchReads");
        for (name, expected) in [
            ("ternaryRead", TERNARY),
            ("ifStatement", IF_STATEMENT),
            ("midChain", MID_CHAIN),
        ] {
            let body = member_body(&report, name);
            for line in expected {
                assert!(
                    body.contains(line),
                    "{}: `{name}` lost {line:?}:\n{body}",
                    leg.label
                );
            }
            assert!(
                !body.contains("@bytecode") && !body.contains("not recovered"),
                "{}: `{name}` is quoted, not recovered:\n{body}",
                leg.label
            );
            assert!(
                !body.contains("int b") && !body.contains("!= 0"),
                "{}: `{name}` still spells the local as an int:\n{body}",
                leg.label
            );
        }
        // The fixture's own driver is a plain statement sequence: it recovers, which is what makes
        // the replay below a whole-class round trip.
        let main = member_body(&report, "main");
        assert!(
            !main.contains("@bytecode") && !main.contains("not recovered"),
            "{}: the fixture's own `main` is refused:\n{main}",
            leg.label
        );
    }
}

// -------------------------------------------------------------------------------------------
// The boundaries the arm does not cross.
// -------------------------------------------------------------------------------------------

/// The loop condition position keeps the cross-region refusal, and the catch-crossing form keeps
/// the ownership refusal — both verbatim, on both legs.
#[test]
fn the_loop_and_crossing_positions_keep_their_refusals_on_both_legs() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BranchReadNegatives"));
        let report = presented(&snapshot, "BranchReadNegatives");
        let loop_body = member_body(&report, "loopCondition");
        assert!(
            loop_body.contains(LOOP_REFUSAL),
            "{}: the loop condition position lost its refusal:\n{loop_body}",
            leg.label
        );
        assert!(
            loop_body.contains("while (b != 0) {"),
            "{}: the loop is no longer presented beside the quoted chain:\n{loop_body}",
            leg.label
        );
        assert!(
            !loop_body.contains("boolean b ="),
            "{}: the loop condition position was admitted past the region criterion:\n{loop_body}",
            leg.label
        );
        let crossing = member_body(&report, "crossCatch");
        assert!(
            crossing.contains(CROSS_CATCH_REFUSAL) && crossing.contains("not recovered"),
            "{}: the catch-crossing form lost its refusal:\n{crossing}",
            leg.label
        );
    }
}

/// The whitelist's new arm is the two zero-tests and nothing else: the same class with the
/// `ifStatement` branch patched to a numeric test keeps the whole refusal, and with it patched to
/// `ifne` it presents the inverted sense.
#[test]
fn the_consumer_whitelist_stays_narrow() {
    let numeric = zip_of(&[(b"BranchReads.class", CONTROL_NUMERIC_BRANCH)]);
    let report = presented(&open(&numeric), "BranchReads");
    let body = member_body(&report, "ifStatement");
    assert!(
        body.contains("not recovered") && body.contains(OP2_REFUSAL),
        "the numeric branch control is no longer refused whole:\n{body}"
    );
    assert!(
        !body.contains("boolean b ="),
        "a numeric test on the loaded value was admitted:\n{body}"
    );
    // The same container's *other* members are untouched by the patch: the arm, not the class.
    assert!(
        member_body(&report, "ternaryRead").contains("return b ? x : -1;"),
        "the numeric branch control's ternary member moved:\n{}",
        report.text
    );

    let not_zero = zip_of(&[(b"BranchReads.class", CONTROL_NOT_ZERO_BRANCH)]);
    let report = presented(&open(&not_zero), "BranchReads");
    let body = member_body(&report, "ifStatement");
    for line in [
        "        boolean b = x > 0 && x > 0;",
        "        if (!b) {",
        "            return x;",
        "        } else {",
        "            return -1;",
    ] {
        assert!(
            body.contains(line),
            "the inverted zero-test control lost {line:?}:\n{body}"
        );
    }
    assert!(
        !body.contains("@bytecode") && !body.contains("not recovered"),
        "the inverted zero-test control is quoted:\n{body}"
    );
}

// -------------------------------------------------------------------------------------------
// The budget: a bounded run never publishes a partial recovery.
// -------------------------------------------------------------------------------------------

#[test]
fn an_exhausted_source_budget_cannot_publish_a_partial_branch_read_recovery() {
    let leg = &LEGS[0];
    let bytes = leg.fixture("BranchReads");
    let snapshot = open(&bytes);
    let complete = presented(&snapshot, "BranchReads");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("BranchReads"),
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
    let mut limits = complete.limits.clone();
    limits.analysis_steps = complete.usage.analysis_steps - 1;
    let mut bounded = Budget::new(limits);
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut bounded,
        )
        .expect("a bounded class-source request is answered");
    match outcome {
        OperationOutcome::Incomplete(candidates) => assert!(matches!(
            candidates.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        )),
        OperationOutcome::Performed(report) => assert!(matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        )),
        other => panic!("unexpected bounded result: {other:?}"),
    }
}

// -------------------------------------------------------------------------------------------
// The replay: both compiler legs, real execution, the fixture's own answers.
// -------------------------------------------------------------------------------------------

/// One private directory a test compiles and runs in.
struct Scratch(PathBuf);

static NEXT_SCRATCH: AtomicU64 = AtomicU64::new(0);

impl Scratch {
    fn new(label: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos();
        let ordinal = NEXT_SCRATCH.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("jarde-branch-reads-{label}-{stamp}-{ordinal}"));
        fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
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

/// Run one class under `-Xverify:all`, answering its standard output and exit status.
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

/// Compile one stripped presentation with the named compiler and run it under `-Xverify:all`.
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

/// The two compilers and the two runtimes of the replay, in the order the legs are listed.
fn legs() -> [(&'static str, &'static str, bool); 2] {
    [
        ("/usr/bin/javac", "/usr/bin/java", true),
        (
            "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac",
            "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java",
            false,
        ),
    ]
}

/// The patrol anchor's replay: the recovered class, with the one refused body replaced by the
/// frozen source's own `main`, compiles on both legs and prints the original's line — including
/// `condAssignOld(0)`.
#[test]
#[ignore = "compiles and runs the recovered text; needs the installed JDK (and javac 8 for the second leg)"]
fn the_patrol_anchor_compiles_and_answers_like_the_original() {
    // The driver is the frozen source's own, not a transcription of it.
    assert!(
        OP2_SOURCE.contains(OP2_DRIVER),
        "the frozen source no longer states the driver this test substitutes"
    );
    let snapshot = open(OP2_JAR);
    let report = presented(&snapshot, "OP2");
    let text = stripped(&report);
    assert_eq!(
        text.matches("jarde_refused_body();").count(),
        1,
        "the class has exactly one refused body (its own `main`, the concat family's residual):\n{text}"
    );
    let recovered = text.replace("jarde_refused_body();", OP2_DRIVER);

    let original = Scratch::new("op2-original");
    fs::write(original.path().join("OP2.class"), op2_class()).expect("the frozen class is written");
    let want = run_class("/usr/bin/java", original.path(), "OP2");
    assert_eq!(want, format!("{OP2_GOLDEN}\nstatus=0"));

    for (compiler, runner, release) in legs() {
        if !Path::new(compiler).is_file() {
            continue;
        }
        let work = Scratch::new("op2-recovered");
        fs::write(work.path().join("OP2.java"), &recovered).expect("the presentation is written");
        let got = compile_and_run(compiler, runner, release, work.path(), "OP2");
        assert_eq!(
            got, want,
            "the recovered anchor answers differently after the round trip ({compiler}):\n{recovered}"
        );
    }
}

/// The fixture's replay: the whole class recovers, so its own `main` is what is measured.
#[test]
#[ignore = "compiles and runs the recovered text; needs the installed JDK (and javac 8 for the second leg)"]
fn the_branch_reads_fixture_compiles_and_answers_like_the_original() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BranchReads"));
        let report = presented(&snapshot, "BranchReads");
        let text = stripped(&report);
        assert!(
            !text.contains("jarde_refused_body"),
            "{}: the fixture's text still holds a refused body marker:\n{text}",
            leg.label
        );
        let original = Scratch::new("branch-reads-original");
        fs::write(
            original.path().join("BranchReads.class"),
            leg.class("BranchReads.class"),
        )
        .expect("the fixture class is written");
        let want = run_class("/usr/bin/java", original.path(), "BranchReads");
        assert_eq!(
            want, BRANCH_READS_GOLDEN,
            "{}: the committed fixture's own answer moved",
            leg.label
        );
        for (compiler, runner, release) in legs() {
            if !Path::new(compiler).is_file() {
                continue;
            }
            let work = Scratch::new("branch-reads-recovered");
            fs::write(work.path().join("BranchReads.java"), &text)
                .expect("the presentation is written");
            let got = compile_and_run(compiler, runner, release, work.path(), "BranchReads");
            assert_eq!(
                got, want,
                "{} ({compiler}): the recovered fixture answers differently:\n{text}",
                leg.label
            );
        }
    }
}

/// The boundaries' own text is the *safe* form: it does not compile, so it can never be a
/// presentation that compiles and behaves differently. The controls are legal classes all the
/// same — their patched bytecode verifies — which is what makes them controls and not bytes.
#[test]
#[ignore = "runs the JVM; needs the installed JDK"]
fn the_boundaries_do_not_compile_and_the_controls_verify() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BranchReadNegatives"));
        let report = presented(&snapshot, "BranchReadNegatives");
        let text = stripped(&report);
        let work = Scratch::new("negatives");
        fs::write(work.path().join("BranchReadNegatives.java"), &text)
            .expect("the presentation is written");
        let compiled = Command::new("/usr/bin/javac")
            .arg("--release")
            .arg("8")
            .arg("-nowarn")
            .arg("-d")
            .arg(work.path())
            .arg(work.path().join("BranchReadNegatives.java"))
            .output()
            .expect("the compiler runs");
        assert!(
            !compiled.status.success(),
            "{}: the boundaries' text compiles and must not:\n{text}",
            leg.label
        );
    }
    for (label, bytes) in [
        ("NumericBranch", CONTROL_NUMERIC_BRANCH),
        ("NotZeroBranch", CONTROL_NOT_ZERO_BRANCH),
    ] {
        let work = Scratch::new(label);
        fs::write(work.path().join("BranchReads.class"), bytes).expect("the control is written");
        let run = Command::new("/usr/bin/java")
            .args(["-Xverify:all", "-cp"])
            .arg(work.path())
            .arg("BranchReads")
            .output()
            .expect("the JVM runs");
        assert!(
            run.status.success(),
            "{label}: the patched control does not verify: {}",
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

/// `OP2`'s own class file, read out of the frozen patrol jar: the original the replay compares
/// against is the very class the archive holds, not a recompilation.
fn op2_class() -> Vec<u8> {
    let archive = rawzip::ZipArchive::from_slice(OP2_JAR).expect("the frozen fixture is a ZIP");
    let mut entries = archive.entries();
    while let Some(header) = entries.next_entry().expect("the entry header reads") {
        if header.file_path().as_ref() == b"OP2.class" {
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
    panic!("the frozen jar holds `OP2.class`")
}
