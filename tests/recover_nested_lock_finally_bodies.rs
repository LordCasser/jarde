//! `recover-nested-lock-finally-bodies`: the nested-lock statement and the interruptible
//! acquisition are presented as the one `try`/`finally` their source wrote.
//!
//! The anchor is the multi-lock patrol's own frozen artifact
//! (`openspec/evidence/java-syntax-2026-10-08/multi-lock-patrol/fixture/ml.jar`): its
//! `nestedLocks` takes two locks before the protected range and its `finally` releases them in the
//! reverse order, and its `interruptibly` acquires with a call that **may throw**. Both refused
//! whole before this slice, on the finally-copy four-piece proof (BCI 41 and BCI 27).
//!
//! Two sibling admissions of the lock-guard certificate present them, and neither widens the other
//! proofs:
//!
//! * **A — the multi-statement finally body**: both release copies are the same sequence of one or
//!   two three-instruction groups, each group's receiver is SSA-identical to one of the pre-line
//!   acquisitions, and the groups pair with the acquisitions in **reverse** order — the
//!   nested-lock statement's own invariant;
//! * **B — the throwing acquisition outside the rows**: an acquisition call that may throw
//!   (`lockInterruptibly()`) is admissible exactly when no row of the table covers it, because
//!   then it must complete before the protected range begins: a throw from it never enters and the
//!   release must not run, which is what the bytecode itself states. The declaration's `throws`
//!   clause is presented as it stands.
//!
//! What this file pins:
//!
//! * the whole `ML` class renders on every leg (the patrol's jar, and the fixture's two compiler
//!   legs: javac 23.0.1 `--release 8 -g:none` and the real javac 8), with the two shapes written
//!   as their source wrote them and no refusal of the family left;
//! * `MLOrder` — the same two shapes over this fixture's own recording lock — renders whole on
//!   both legs, and its presented text **compiles** on both compilers and answers what the
//!   original answers under `-Xverify:all`: the release order (`b` before `a`), the `finally` on an
//!   exception raised inside the protected range, no release after a failed `lockInterruptibly`,
//!   and the anchor's own `2`;
//! * the negatives — a release sequence that is not the acquisition order reversed, a release of a
//!   lock the statement never acquired, an acquisition the row itself covers — keep their refusals
//!   **verbatim**, on both legs;
//! * the registered boundaries — a `try`/`finally` inside the guarded range, three locks, and a
//!   body whose branch moves the release copy into a block of its own (the canonical graph fuses
//!   the trailing `return` into it) — keep their refusals **verbatim**;
//! * the patrol's `multiAwait` stays byte-identical: it is this slice's zero-regression control,
//!   refused before and after with the patrol's own recorded text.

use jarde::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

/// The patrol's frozen anchor: javac 23.0.1, `--release 8`, with line numbers.
const JAR: &[u8] =
    include_bytes!("../openspec/evidence/java-syntax-2026-10-08/multi-lock-patrol/fixture/ml.jar");
/// The same source on this fixture's own legs (see the fixture's `README.md`).
const ML_V8: &[u8] = include_bytes!("fixtures/recover-nested-lock-finally-bodies/v8/ML.class");
const ML_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-nested-lock-finally-bodies/v8-javac8/ML.class");
const ORDER_V8: &[u8] =
    include_bytes!("fixtures/recover-nested-lock-finally-bodies/v8/MLOrder.class");
const ORDER_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-nested-lock-finally-bodies/v8-javac8/MLOrder.class");
const ORDER_LOCK_V8: &[u8] =
    include_bytes!("fixtures/recover-nested-lock-finally-bodies/v8/Order.class");
const ORDER_LOCK_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-nested-lock-finally-bodies/v8-javac8/Order.class");
const NEGATIVES_V8: &[u8] =
    include_bytes!("fixtures/recover-nested-lock-finally-bodies/v8/MLNegatives.class");
const NEGATIVES_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-nested-lock-finally-bodies/v8-javac8/MLNegatives.class");
const PROBE_V8: &[u8] =
    include_bytes!("fixtures/recover-nested-lock-finally-bodies/v8/MLProbe.class");
const PROBE_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-nested-lock-finally-bodies/v8-javac8/MLProbe.class");

/// The two shapes' own texts, as the certificate writes them: the acquisition run before the
/// statement, the protected body between the braces, and the releases in the clause's order.
const NESTED_LOCKS: &str = "        this.a.lock();\n        this.b.lock();\n        try {\n            this.count += 1;\n        } finally {\n            this.b.unlock();\n            this.a.unlock();\n        }\n        return;\n";
const INTERRUPTIBLY: &str = "        this.a.lockInterruptibly();\n        try {\n            this.count += 1;\n        } finally {\n            this.a.unlock();\n        }\n        return;\n";

/// The refusal the patrol recorded for `multiAwait`, verbatim: this slice's zero-regression
/// control (`results/jarde-ML.txt` of the patrol, reproduced on all three legs).
const MULTI_AWAIT: [&str; 2] = [
    "// @bytecode 0 15 22 31 40 46 70",
    "// local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
];

/// The negatives' refusals, verbatim: the region/guard sentence and every instruction each refusal
/// covers, as the baseline binary stated them (see `results/03-gating.out`).
const RELEASE_ORDER: [&str; 4] = [
    "// @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28 31 32 35 38",
    "// BCI 41: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`",
    "// @bytecode 41 42 43 46 49 50 53 56 57 58",
    "// 2 live block(s) are reachable only through edges the normal-flow view leaves out: [58, 41]",
];
const UNMATCHED_RELEASE: [&str; 4] = RELEASE_ORDER;
const ACQUISITION_IN_RANGE: [&str; 4] = [
    "// @bytecode 0 1 4 7 8 9 12 13 14 17 18 21 24",
    "// BCI 27: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`",
    "// @bytecode 27 28 29 32 35 36 37",
    "// 2 live block(s) are reachable only through edges the normal-flow view leaves out: [37, 27]",
];

/// The boundaries' refusals, verbatim: a `try`/`finally` inside the guarded range states two rows
/// over two handlers, and three locks exceed the clause's two-statement bound.
const NESTED_TRY: [&str; 4] = [
    "// @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28 31",
    "// BCI 54: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`",
    "// @bytecode 34 35 36 39 42 43 44 45 48 51 54 55 56 59 62 63 64",
    "// 3 live block(s) are reachable only through edges the normal-flow view leaves out: [44, 34, 54]",
];
const THREE_LOCKS: [&str; 4] = [
    "// @bytecode 0 1 4 7 8 11 14 15 18 21 22 23 26 27 28 31 32 35 38 39 42 45 46 49 52",
    "// BCI 55: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`",
    "// @bytecode 55 56 57 60 63 64 67 70 71 74 77 78 79",
    "// 2 live block(s) are reachable only through edges the normal-flow view leaves out: [79, 55]",
];
/// The branching body's boundary: the branch moves the release copy into a block of its own, and
/// the canonical graph fuses the method's trailing `return` into it, so the void completion's
/// transfer has no successor block to state.
const BRANCHING_BODY: [&str; 4] = [
    "// @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25",
    "// BCI 55: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`",
    "// @bytecode 28 31 32 34 37 38 39 42 45 46 49 52 55 56 57 60 63 64 67 70 71 72",
    "// 3 live block(s) are reachable only through edges the normal-flow view leaves out: [28, 38, 55]",
];

/// What `MLOrderDriver` prints against the fixture's own class and against the presented text: the
/// release order (`b` before `a`), the `finally` on the exceptional path, no release after the
/// interrupted acquisition, the interruptible completion, and the anchor's own value.
const ORDER_BEHAVIOR: &str = "normal count=1 log=a.lock b.lock b.unlock a.unlock\ncaught=body failed count=1 log=a.lock b.lock b.unlock a.unlock\ninterrupted=a interrupted count=0 log=\ninterruptible count=1 log=a.lockInterruptibly a.unlock\nanchor=2";

/// The patrol's own recorded behavior of `ML.main`.
const ANCHOR_BEHAVIOR: &str = "2";

/// The real javac 8 of this repository's fixture protocol (Corretto 1.8.0_432).
const CORRETTO_HOME: &str = "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home";

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens")
}

fn report(
    snapshot: &ArtifactSnapshot,
    class: &str,
    policy: EnvironmentPolicy,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match Engine::new()
        .class_source(slice::from_ref(snapshot), &request, &mut budget())
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed class has one class-source result, got {other:?}"),
    }
}

fn method_text<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}`"))
        .text
}

/// Every leg of the anchor, with the policy its own artifact kind takes.
fn legs() -> Vec<(&'static str, Vec<u8>, EnvironmentPolicy, &'static str)> {
    vec![
        (
            "patrol-jar",
            JAR.to_vec(),
            EnvironmentPolicy::PlainJar,
            "ML",
        ),
        ("v8", ML_V8.to_vec(), EnvironmentPolicy::SingleClass, "ML"),
        (
            "v8-javac8",
            ML_V8_JAVAC8.to_vec(),
            EnvironmentPolicy::SingleClass,
            "ML",
        ),
    ]
}

#[test]
fn the_nested_lock_shapes_present_whole_on_every_leg() {
    let mut texts = Vec::new();
    for (leg, bytes, policy, class) in legs() {
        let snapshot = open(&bytes);
        let report = report(&snapshot, class, policy);
        assert!(
            report.text.starts_with("// jarde: presentation of `ML`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        let nested = method_text(&report, "nestedLocks");
        assert!(
            nested.contains(NESTED_LOCKS),
            "{leg}: the two acquisitions, the protected body and the two releases in reverse order:\n{nested}"
        );
        assert!(
            !nested.contains("@bytecode") && !nested.contains("jarde_refused_body"),
            "{leg}: no instruction of `nestedLocks` stays quoted:\n{nested}"
        );
        let interruptibly = method_text(&report, "interruptibly");
        assert!(
            interruptibly
                .starts_with("    void interruptibly() throws java.lang.InterruptedException {"),
            "{leg}: the declaration's `throws` clause is presented as it stands:\n{interruptibly}"
        );
        assert!(
            interruptibly.contains(INTERRUPTIBLY),
            "{leg}: the throwing acquisition, the body and the release:\n{interruptibly}"
        );
        assert!(
            !interruptibly.contains("@bytecode") && !interruptibly.contains("jarde_refused_body"),
            "{leg}: no instruction of `interruptibly` stays quoted:\n{interruptibly}"
        );
        // The control this slice must not move: the patrol's own recorded refusal, on every leg.
        let multi_await = method_text(&report, "multiAwait");
        for refusal in MULTI_AWAIT {
            assert!(
                multi_await.contains(refusal),
                "{leg}: `multiAwait` keeps the patrol's recorded refusal `{refusal}`:\n{multi_await}"
            );
        }
        texts.push((leg, report.text));
    }
    // The three legs are the same presentation: the shapes are control flow, not one compiler's
    // lowering and not one debug-flag set.
    assert_eq!(
        texts[0].1, texts[1].1,
        "the patrol's jar and the v8 leg agree"
    );
    assert_eq!(texts[1].1, texts[2].1, "the two compiler legs agree");
}

#[test]
fn the_order_leg_presents_the_same_shapes_over_its_recording_lock() {
    for (leg, bytes) in [("v8", ORDER_V8), ("v8-javac8", ORDER_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(&snapshot, "MLOrder", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `MLOrder`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        assert!(
            method_text(&report, "nestedLocks").contains(NESTED_LOCKS),
            "{leg}: `nestedLocks` presents:\n{}",
            method_text(&report, "nestedLocks")
        );
        assert!(
            method_text(&report, "interruptibly").contains(INTERRUPTIBLY),
            "{leg}: `interruptibly` presents:\n{}",
            method_text(&report, "interruptibly")
        );
        let throwing = method_text(&report, "nestedLocksThrowing");
        assert!(
            throwing.contains("            this.count += 1;\n            this.check();\n"),
            "{leg}: the throwing body is the body's own statements:\n{throwing}"
        );
        assert!(
            throwing.contains("            this.b.unlock();\n            this.a.unlock();\n"),
            "{leg}: the exceptional path's clause is the same two releases:\n{throwing}"
        );
        // Every member of the order leg presents: the whole-class text is the one the driver
        // roundtrip compiles (the anchor's own class keeps `multiAwait`'s registered refusal, which
        // is stated in `the_anchors_own_class_answers_its_baseline_and_its_boundary_is_stated`).
        assert!(
            !report.text.contains("@bytecode") && !report.text.contains("jarde_refused_body"),
            "{leg}: no member of `MLOrder` stays quoted:\n{}",
            report.text
        );
    }
}

#[test]
fn the_negatives_keep_their_refusals_verbatim() {
    for (leg, bytes) in [("v8", NEGATIVES_V8), ("v8-javac8", NEGATIVES_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(&snapshot, "MLNegatives", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `MLNegatives`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        for (name, refusals) in [
            ("releaseOrderNotReversed", &RELEASE_ORDER[..]),
            ("unlockWithoutLock", &UNMATCHED_RELEASE[..]),
            ("throwingCallInsideRange", &ACQUISITION_IN_RANGE[..]),
        ] {
            let method = method_text(&report, name);
            assert!(
                !method.contains("finally {") && method.contains("@bytecode"),
                "{leg}/{name}: the shape is refused whole, with no `finally` invented:\n{method}"
            );
            for refusal in refusals {
                assert!(
                    method.contains(refusal),
                    "{leg}/{name}: the refusal keeps `{refusal}` verbatim:\n{method}"
                );
            }
        }
    }
}

#[test]
fn the_registered_boundaries_stay_refused() {
    for (leg, bytes) in [("v8", PROBE_V8), ("v8-javac8", PROBE_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(&snapshot, "MLProbe", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `MLProbe`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        for (name, refusals) in [
            ("nestedTry", &NESTED_TRY[..]),
            ("threeLocks", &THREE_LOCKS[..]),
            ("nestedLocksBranching", &BRANCHING_BODY[..]),
        ] {
            let method = method_text(&report, name);
            assert!(
                !method.contains("finally {") && method.contains("@bytecode"),
                "{leg}/{name}: the boundary is refused whole, with no `finally` invented:\n{method}"
            );
            for refusal in refusals {
                assert!(
                    method.contains(refusal),
                    "{leg}/{name}: the boundary keeps `{refusal}` verbatim:\n{method}"
                );
            }
        }
    }
}

// -------------------------------------------------------------------------------------------
// The drivers: the presented text compiles on both compilers and answers what the class answers.
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names, whether it is the real JDK 8.
struct Leg {
    label: &'static str,
    real_javac8: bool,
}

const LEGS: &[Leg] = &[
    Leg {
        label: "javac 23.0.1 --release 8",
        real_javac8: false,
    },
    Leg {
        label: "Corretto 1.8.0_432 (real javac 8)",
        real_javac8: true,
    },
];

impl Leg {
    /// This leg's compiler: javac 23 targets Java 8 with `--release 8`, the real javac 8 needs no
    /// flag (its default target is 8) and is named by path, which the test asserts before it is
    /// used.
    fn javac(&self) -> Command {
        let mut command = if self.real_javac8 {
            assert!(
                Path::new(CORRETTO_HOME).join("bin/javac").exists(),
                "the real javac 8 of the fixture protocol is installed at {CORRETTO_HOME}"
            );
            Command::new(format!("{CORRETTO_HOME}/bin/javac"))
        } else {
            let mut command = Command::new("javac");
            command.args(["--release", "8", "-Xlint:-options"]);
            command
        };
        command.args(["-J-Duser.language=en", "-J-Duser.country=US"]);
        command
    }

    /// This leg's JVM.
    fn java(&self) -> Command {
        if self.real_javac8 {
            Command::new(format!("{CORRETTO_HOME}/bin/java"))
        } else {
            Command::new("java")
        }
    }

    fn compile(&self, source: &Path, output: &Path, classpath: Option<&Path>) {
        let mut command = self.javac();
        command.args(["-d"]).arg(output);
        if let Some(classpath) = classpath {
            command.arg("-cp").arg(classpath);
        }
        let result = command
            .arg(source)
            .output()
            .expect("the leg's compiler is installed");
        assert!(
            result.status.success(),
            "{}: javac rejected the presented text:\n{}\n{}",
            self.label,
            String::from_utf8_lossy(&result.stderr),
            std::fs::read_to_string(source).expect("the text reads")
        );
    }

    /// Run one class under `-Xverify:all` with this leg's own JVM and answer what it printed.
    fn run(&self, directory: &Path, class: &str) -> String {
        let result = self
            .java()
            .args(["-Xverify:all", "-cp"])
            .arg(directory)
            .arg(class)
            .current_dir(directory)
            .output()
            .expect("the leg's JVM is installed");
        assert!(
            result.status.success(),
            "{}: `{class}` failed under -Xverify:all:\n{}",
            self.label,
            String::from_utf8_lossy(&result.stderr)
        );
        String::from_utf8(result.stdout)
            .expect("the run prints text")
            .trim_end()
            .to_owned()
    }
}

static UNIQUE: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-nested-lock-{label}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("create the replay directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// One presented class text with every `//` comment line dropped — the strip the patrol's own text
/// is stripped by.
fn stripped(text: &str) -> String {
    let mut body: Vec<String> = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::to_owned)
        .collect();
    body.push(String::new());
    body.join("\n")
}

/// The order leg's committed classes of one compiler leg, beside the driver's own source.
fn order_files(real_javac8: bool) -> Vec<(&'static str, &'static [u8])> {
    if real_javac8 {
        vec![
            ("MLOrder.class", ORDER_V8_JAVAC8),
            ("Order.class", ORDER_LOCK_V8_JAVAC8),
        ]
    } else {
        vec![("MLOrder.class", ORDER_V8), ("Order.class", ORDER_LOCK_V8)]
    }
}

/// The driver source the order leg is run by (a fixture source, compiled per leg here).
const DRIVER_SOURCE: &str =
    include_str!("fixtures/recover-nested-lock-finally-bodies/MLOrderDriver.java");

#[test]
#[ignore = "needs both JDKs: it strips the class-source comment lines, compiles the text with \
            `javac --release 8` **and** the real javac 8 and runs both under `-Xverify:all` (see \
            the module doc)"]
fn the_order_and_exception_drivers_answer_what_the_original_answers() {
    for leg in LEGS {
        let temp = TempDir::new("order");
        let original = temp.path().join("original");
        let presented = temp.path().join("presented");
        std::fs::create_dir_all(&original).expect("create the original directory");
        std::fs::create_dir_all(&presented).expect("create the presented directory");
        for (name, bytes) in order_files(leg.real_javac8) {
            std::fs::write(original.join(name), bytes).expect("the fixture class is written");
            // The lock the presented class calls is the fixture's own committed class, standing
            // beside the text; the text's own compilation replaces the order leg's class.
            std::fs::write(presented.join(name), bytes).expect("the lock's class is written");
        }
        let source = presented.join("MLOrder.java");
        std::fs::write(
            &source,
            stripped(
                &report(
                    &open(if leg.real_javac8 {
                        ORDER_V8_JAVAC8
                    } else {
                        ORDER_V8
                    }),
                    "MLOrder",
                    EnvironmentPolicy::SingleClass,
                )
                .text,
            ),
        )
        .expect("write the presented text");
        // The presented class references the fixture's own lock: the lock's committed class stands
        // beside it, and the text compiles against it.
        leg.compile(&source, &presented, Some(&original));
        // The driver is compiled beside each side: against the fixture's own class and against the
        // presented one, so both runs exercise the same driver source.
        let driver = temp.path().join("MLOrderDriver.java");
        std::fs::write(&driver, DRIVER_SOURCE).expect("write the driver source");
        for directory in [&original, &presented] {
            leg.compile(&driver, directory, Some(directory));
        }
        assert_eq!(
            leg.run(&original, "MLOrderDriver"),
            ORDER_BEHAVIOR,
            "{}: the fixture's own class answers the release order, the exceptional path and the \
             interrupted acquisition",
            leg.label
        );
        assert_eq!(
            leg.run(&presented, "MLOrderDriver"),
            ORDER_BEHAVIOR,
            "{}: the presented text answers exactly what the class answers — the two releases in \
             the order the source wrote them, the `finally` on the exceptional path, and no \
             release after a failed `lockInterruptibly`",
            leg.label
        );
    }
}

#[test]
#[ignore = "needs both JDKs: it runs the anchor's own classes on both legs and states why the \
            whole-class text stays uncompilable while `multiAwait`'s registered refusal stands \
            (see the module doc)"]
fn the_anchors_own_class_answers_its_baseline_and_its_boundary_is_stated() {
    for leg in LEGS {
        let temp = TempDir::new("anchor");
        let directory = temp.path().join("original");
        std::fs::create_dir_all(&directory).expect("create the original directory");
        std::fs::write(
            directory.join("ML.class"),
            if leg.real_javac8 { ML_V8_JAVAC8 } else { ML_V8 },
        )
        .expect("the fixture class is written");
        assert_eq!(
            leg.run(&directory, "ML"),
            ANCHOR_BEHAVIOR,
            "{}: the anchor's own class answers its recorded value (the behavior the two guards \
             must keep)",
            leg.label
        );

        // The boundary, stated rather than hidden: `multiAwait`'s refusal leaves the member without
        // a `return`, so the whole-class text does not compile while that criterion stands. A run
        // that compiles it is a change this test wants to hear about — the follow-up slice's own
        // acceptance will flip this assertion with its evidence.
        let source = temp.path().join("ML.java");
        std::fs::write(
            &source,
            stripped(
                &report(
                    &open(if leg.real_javac8 { ML_V8_JAVAC8 } else { ML_V8 }),
                    "ML",
                    EnvironmentPolicy::SingleClass,
                )
                .text,
            ),
        )
        .expect("write the presented text");
        let result = leg
            .javac()
            .args(["-d"])
            .arg(temp.path())
            .arg(&source)
            .output()
            .expect("the leg's compiler is installed");
        assert!(
            !result.status.success(),
            "{}: the whole-class text must stay uncompilable while `multiAwait`'s registered \
             refusal stands:\n{}",
            leg.label,
            std::fs::read_to_string(&source).expect("the text reads")
        );
    }
}
