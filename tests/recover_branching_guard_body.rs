//! `recover-branching-guard-body`: a lock guard whose **protected body branches** is presented as
//! the one `try`/`finally` its source wrote.
//!
//! The measured line this slice closes is the nested-lock slice's third registered boundary
//! (`MLProbe.nestedLocksBranching`, refused at BCI 55): the branch puts the release copy in a block
//! of its own — the protected range's end is that block's own start — and because that block
//! carries no exception edge of its own, the canonical graph fuses the method's trailing value-less
//! `return` into it, so the void completion's transfer has **no successor block to state**. The
//! same behavior without that layout (`MLOrder.nestedLocksThrowing`, and `LK`'s loop bodies whose
//! release block still carries the range's exception edge) presents before and after this slice.
//!
//! What this file pins:
//!
//! * `BG` — the anchor (`singleIf`, the patrol's boundary shape verbatim, and `ifElse`) — presents
//!   whole on both compiler legs, the branch by the walk's own `if` shapes and the completion by
//!   the fused block's tail;
//! * `BGProbe` — the walk's branch shapes **beyond** the filing's own MVP note (`twoIfs`,
//!   `nestedIf`, `bodyLoop`) — presents whole: measured, not assumed, and flagged as the
//!   over-delivery the filing's note is narrower than;
//! * `BGNegatives` — a fused tail that is not the method's value-less `return` (`tailThrow`'s
//!   allocation, `tailStoredThrow`'s `athrow`), a branch whose arm returns inside the range
//!   (`bodyReturn`), and the multi-way branch (`switchBody`) — keep their refusals; the first three
//!   byte-identically, the fourth with the diagnostic the change moves (recorded here);
//! * `BGOrder` — the same shapes over the fixture's own recording lock — renders whole, and the
//!   ignored replay compiles the stripped text with **both** javac legs and runs the fixture's
//!   driver against the fixture's class and against the recompiled text under `-Xverify:all`:
//!   both answer the same eight lines, the exceptional arm included.

use jarde::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

/// The fixture's classes, both compiler legs (see the fixture's `README.md`).
const BG_V8: &[u8] = include_bytes!("fixtures/recover-branching-guard-body/v8/BG.class");
const BG_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-branching-guard-body/v8-javac8/BG.class");
const ORDER_V8: &[u8] = include_bytes!("fixtures/recover-branching-guard-body/v8/BGOrder.class");
const ORDER_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-branching-guard-body/v8-javac8/BGOrder.class");
const ORDER_LOCK_V8: &[u8] = include_bytes!("fixtures/recover-branching-guard-body/v8/Order.class");
const ORDER_LOCK_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-branching-guard-body/v8-javac8/Order.class");
const NEGATIVES_V8: &[u8] =
    include_bytes!("fixtures/recover-branching-guard-body/v8/BGNegatives.class");
const NEGATIVES_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-branching-guard-body/v8-javac8/BGNegatives.class");
const PROBE_V8: &[u8] = include_bytes!("fixtures/recover-branching-guard-body/v8/BGProbe.class");
const PROBE_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-branching-guard-body/v8-javac8/BGProbe.class");

/// The anchor's own texts, as the certificate writes them: the acquisitions before the statement,
/// the branching protected body between the braces, the releases in the clause's order, and the
/// fused trailing `return` written after the `try`.
const SINGLE_IF: &str = "        this.a.lock();\n        this.b.lock();\n        try {\n            this.count += 1;\n            if (arg1) {\n                throw new java.lang.IllegalStateException(\"body failed\");\n            }\n        } finally {\n            this.b.unlock();\n            this.a.unlock();\n        }\n        return;\n";
const IF_ELSE: &str = "        this.a.lock();\n        this.b.lock();\n        try {\n            if (arg1) {\n                this.count += 1;\n            } else {\n                this.count -= 1;\n            }\n        } finally {\n            this.b.unlock();\n            this.a.unlock();\n        }\n        return;\n";

/// The probe's own texts: the shapes the walk structures beyond the filing's MVP note.
const TWO_IFS: &str = "            this.count += 1;\n            if (arg1) {\n                this.count += 1;\n            }\n            if (arg2) {\n                this.count -= 1;\n            }\n";
const NESTED_IF: &str = "            this.count += 1;\n            if (arg1) {\n                if (arg2) {\n                    this.count += 1;\n                }\n            }\n";
const BODY_LOOP: &str =
    "            while (this.flag) {\n                this.count += 1;\n            }\n";

/// The negatives' refusals, verbatim: the region/guard sentence and every instruction each refusal
/// covers, as the **pre-change** binary stated them too (see `results/03-anchors-and-negatives.md`).
const TAIL_THROW: [&str; 4] = [
    "// @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28",
    "// BCI 58: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`",
    "// @bytecode 31 34 35 37 40 41 42 45 48 49 52 55 58 59 60 63 66 67 70 73 74 75 78 79 81 84",
    "// 3 live block(s) are reachable only through edges the normal-flow view leaves out: [31, 41, 58]",
];
const TAIL_STORED_THROW: [&str; 4] = [
    "// @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 28",
    "// BCI 58: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`",
    "// @bytecode 31 34 35 37 40 41 42 45 48 49 52 55 58 59 60 63 66 67 70 73 74 75 76 79",
    "// 3 live block(s) are reachable only through edges the normal-flow view leaves out: [31, 41, 58]",
];
const BODY_RETURN: [&str; 4] = [
    "// @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25",
    "// BCI 60: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`",
    "// @bytecode 28 29 32 35 36 39 42 43 44 47 50 51 54 57 60 61 62 65 68 69 72 75 76 77",
    "// 3 live block(s) are reachable only through edges the normal-flow view leaves out: [28, 43, 60]",
];
/// The multi-way branch's refusal, as this slice leaves it: the certificate now claims the
/// statement (the fused tail is admitted) and the **body walk** refuses the `switch`, so the
/// diagnostic is the guard's own body refusal. The pre-change text — the four-piece cascade
/// (`// BCI 84: …` + `// 4 live block(s) … [44, 57, 84, 67]`) — is recorded in the fixture's
/// `README.md` and in `results/03-anchors-and-negatives.md`; the member stays refused whole either
/// way.
const SWITCH_BODY: [&str; 2] = [
    "// @bytecode 0 1 4 7 8 11 14 15 16 19 20 21 24 25 44 45 46 49 50 51 54 57 58 59 62 63 64 67 68 71 74 75 78 81 84 85 86 89 92 93 96 99 100 101",
    "// the shared catch-all finally has no complete bounded try and catch bodies",
];

/// What `BGOrderDriver` prints against the fixture's own class and against the presented text: the
/// normal path, the branch's throwing arm (both with the release order `b` before `a`), both arms
/// of the `if`/`else`, and all four combinations of the two branches.
const ORDER_BEHAVIOR: &str = "normal count=1 log=a.lock b.lock b.unlock a.unlock\ncaught=body failed count=1 log=a.lock b.lock b.unlock a.unlock\nifElse-taken count=1 log=a.lock b.lock b.unlock a.unlock\nifElse-not-taken count=-1 log=a.lock b.lock b.unlock a.unlock\ntwoIfs-00 count=1 log=a.lock b.lock b.unlock a.unlock\ntwoIfs-01 count=0 log=a.lock b.lock b.unlock a.unlock\ntwoIfs-10 count=2 log=a.lock b.lock b.unlock a.unlock\ntwoIfs-11 count=1 log=a.lock b.lock b.unlock a.unlock";

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

/// One leg of the fixture's own bytes.
fn legs() -> [(&'static str, &'static [u8]); 2] {
    [("v8", BG_V8), ("v8-javac8", BG_V8_JAVAC8)]
}

#[test]
fn the_branching_guard_body_presents_whole_on_every_leg() {
    let mut texts = Vec::new();
    for (leg, bytes) in legs() {
        let snapshot = open(bytes);
        let report = report(&snapshot, "BG", EnvironmentPolicy::SingleClass);
        assert!(
            report.text.starts_with("// jarde: presentation of `BG`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        let single = method_text(&report, "singleIf");
        assert!(
            single.contains(SINGLE_IF),
            "{leg}: the branch, the protected body and the two releases, with the fused trailing \
             `return` written after the statement:\n{single}"
        );
        let both = method_text(&report, "ifElse");
        assert!(
            both.contains(IF_ELSE),
            "{leg}: both arms of the branch present:\n{both}"
        );
        assert!(
            !report.text.contains("@bytecode") && !report.text.contains("jarde_refused_body"),
            "{leg}: no member of the anchor stays quoted:\n{}",
            report.text
        );
        texts.push((leg, report.text));
    }
    assert_eq!(texts[0].1, texts[1].1, "the two compiler legs agree");
}

#[test]
fn the_probe_shapes_beyond_the_filings_note_present() {
    // The filing's MVP note ("single branch level; multi-branch stays registered") is narrower than
    // what was measured: the walk structures these bodies already, and the fused-tail admission
    // presents them exactly as the source wrote them. They are pinned so the delivered behavior is
    // visible rather than assumed.
    for (leg, bytes) in [("v8", PROBE_V8), ("v8-javac8", PROBE_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(&snapshot, "BGProbe", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `BGProbe`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        for (name, body) in [
            ("twoIfs", TWO_IFS),
            ("nestedIf", NESTED_IF),
            ("bodyLoop", BODY_LOOP),
        ] {
            let method = method_text(&report, name);
            assert!(
                method.contains(body),
                "{leg}: `{name}` is the body's own statements:\n{method}"
            );
            assert!(
                !method.contains("@bytecode") && !method.contains("jarde_refused_body"),
                "{leg}: no instruction of `{name}` stays quoted:\n{method}"
            );
        }
    }
}

#[test]
fn the_negatives_keep_their_refusals() {
    for (leg, bytes) in [("v8", NEGATIVES_V8), ("v8-javac8", NEGATIVES_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(&snapshot, "BGNegatives", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `BGNegatives`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        for (name, refusals) in [
            ("tailThrow", &TAIL_THROW[..]),
            ("tailStoredThrow", &TAIL_STORED_THROW[..]),
            ("bodyReturn", &BODY_RETURN[..]),
            ("switchBody", &SWITCH_BODY[..]),
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
        // The one classified delta, stated rather than hidden: the multi-way branch's diagnostic is
        // the guard's own body refusal now, not the four-piece cascade the fused layout produced.
        let switch = method_text(&report, "switchBody");
        assert!(
            !switch.contains("BCI 84") && !switch.contains("4 live block(s)"),
            "{leg}: `switchBody` states the body refusal this change moves to, not the pre-change \
             cascade:\n{switch}"
        );
    }
}

#[test]
fn the_order_leg_presents_the_same_shapes_over_its_recording_lock() {
    for (leg, bytes) in [("v8", ORDER_V8), ("v8-javac8", ORDER_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(&snapshot, "BGOrder", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `BGOrder`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        for (name, body) in [
            ("singleIf", SINGLE_IF.replace("arg1", "this.fail")),
            ("ifElse", IF_ELSE.to_owned()),
            ("twoIfs", TWO_IFS.to_owned()),
        ] {
            let method = method_text(&report, name);
            assert!(
                method.contains(body.as_str()),
                "{leg}: `{name}` presents:\n{method}"
            );
        }
        // Every member presents, so the whole-class text is the one the driver roundtrip compiles.
        assert!(
            !report.text.contains("@bytecode") && !report.text.contains("jarde_refused_body"),
            "{leg}: no member of the order leg stays quoted:\n{}",
            report.text
        );
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
            "jarde-branching-guard-{label}-{}-{nonce}-{sequence}",
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

/// One presented class text with every `//` comment line dropped.
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
            ("BGOrder.class", ORDER_V8_JAVAC8),
            ("Order.class", ORDER_LOCK_V8_JAVAC8),
        ]
    } else {
        vec![("BGOrder.class", ORDER_V8), ("Order.class", ORDER_LOCK_V8)]
    }
}

/// The driver source the order leg is run by (a fixture source, compiled per leg here).
const DRIVER_SOURCE: &str =
    include_str!("fixtures/recover-branching-guard-body/BGOrderDriver.java");

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
        let source = presented.join("BGOrder.java");
        std::fs::write(
            &source,
            stripped(
                &report(
                    &open(if leg.real_javac8 {
                        ORDER_V8_JAVAC8
                    } else {
                        ORDER_V8
                    }),
                    "BGOrder",
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
        let driver = temp.path().join("BGOrderDriver.java");
        std::fs::write(&driver, DRIVER_SOURCE).expect("write the driver source");
        for directory in [&original, &presented] {
            leg.compile(&driver, directory, Some(directory));
        }
        assert_eq!(
            leg.run(&original, "BGOrderDriver"),
            ORDER_BEHAVIOR,
            "{}: the fixture's own class answers the release order, the throwing arm and every \
             branch combination",
            leg.label
        );
        assert_eq!(
            leg.run(&presented, "BGOrderDriver"),
            ORDER_BEHAVIOR,
            "{}: the presented text answers exactly what the class answers — the branch's own \
             statements, the releases in the clause's order, and the `finally` on the exceptional \
             path",
            leg.label
        );

        // The anchor's own class text compiles as well: both of its members present whole, so the
        // class is a compilable-and-run unit on both legs.
        let anchor = temp.path().join("BG.java");
        std::fs::write(
            &anchor,
            stripped(
                &report(
                    &open(if leg.real_javac8 { BG_V8_JAVAC8 } else { BG_V8 }),
                    "BG",
                    EnvironmentPolicy::SingleClass,
                )
                .text,
            ),
        )
        .expect("write the anchor's presented text");
        leg.compile(&anchor, &presented, None);
    }
}
