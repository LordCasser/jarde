//! `recover-io-resource-finally`: the resource guard across a `finally` — the io-wrapping patrol's
//! three-layer stream chain — presents as the one `try { … } finally { r.close(); }` its source
//! wrote.
//!
//! The anchor is the patrol's own frozen artifact
//! (`openspec/evidence/java-syntax-2026-10-05/io-wrapping-patrol/fixture/io.jar`): `countLines`
//! builds `new BufferedReader(new InputStreamReader(new FileInputStream(path), "UTF-8"))`, reads
//! it in a `while ((line = r.readLine()) != null)` loop, saves the count and closes the handle in
//! the `finally`. Its two exception-table rows both reach the handler — the protected range and
//! the row javac writes over the handler's own binding store — and the resource local's one
//! definition stands before the range, so the row-set certificate proves that folding the two
//! close copies into one `finally` closes the object the body read, on both paths, exactly once.
//! The three-layer chain's argument positions need the two `java.io` rows of
//! `platform_reference_argument_widens` (transcribed in
//! `openspec/evidence/java-syntax-2026-10-05/widening-row-sources/`), and the outermost `new` has
//! no single place to write unless `new@1`'s nesting scan reaches the chain's own depth.
//!
//! What this file pins:
//!
//! * `countLines` renders whole — no refusal of the family, one `finally`, one `close()` — on the
//!   patrol's jar and on both compiler legs (javac 23.0.1 `--release 8 -g:none` and the real
//!   javac 8), byte for byte the same text;
//! * `readAll` (the `FileReader` char loop) keeps its refusal **verbatim**: it is the copy
//!   family's registered boundary — its loop test's copy-and-store dance has an observable target,
//!   which that family's purity criterion refuses — and this change neither widens that criterion
//!   nor hides the member. The whole-class text therefore does not compile while that boundary
//!   stands, and the mid-read leg's behavior is exercised through the certificate's own shape over
//!   a **caller-owned** stream (`IOMidRead.countRemaining`), whose close is observable;
//! * the negatives — two nested `finally`s over two resources, and a cleanup call that returns a
//!   value — keep their refusals verbatim on both legs;
//! * the `new@1` depth boundary this change moves: three layers present as one expression, four
//!   keep the outermost refusal;
//! * the ignored replay strips the presentation the way the patrol's own text is stripped, compiles
//!   it with **both** compilers (`javac --release 8` and Corretto 1.8.0_432's own `javac`, whose
//!   path the test asserts before it is used), and compares the drivers' answers with the fixture's
//!   own classes under `-Xverify:all`: the anchor's own class answers `2/hello|world|` on both
//!   legs, and the mid-read driver answers the normal count, the exception's own message and the
//!   close it observed — `normal=3` / `caught=read 3 failed closed=true` — identically on both
//!   sides.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// The patrol's frozen anchor: javac 23.0.1, `--release 8`, with line numbers.
const JAR: &[u8] =
    include_bytes!("../openspec/evidence/java-syntax-2026-10-05/io-wrapping-patrol/fixture/io.jar");
/// The same source on this fixture's own legs (see the fixture's `README.md`).
const IO_V8: &[u8] = include_bytes!("fixtures/recover-io-resource-finally/v8/IO.class");
const IO_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-io-resource-finally/v8-javac8/IO.class");
/// The certificate's own shape over a caller-owned stream: the mid-read leg's method.
const MID_V8: &[u8] = include_bytes!("fixtures/recover-io-resource-finally/v8/IOMidRead.class");
const MID_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-io-resource-finally/v8-javac8/IOMidRead.class");
/// The two negatives.
const NEGATIVES_V8: &[u8] =
    include_bytes!("fixtures/recover-io-resource-finally/v8/IONegatives.class");
const NEGATIVES_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-io-resource-finally/v8-javac8/IONegatives.class");
/// The `new@1` depth boundary: `threeLayer` presents, `fourLayer` refuses.
const DEPTH_V8: &[u8] = include_bytes!("fixtures/recover-io-resource-finally/v8/NestedDepth.class");
const DEPTH_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-io-resource-finally/v8-javac8/NestedDepth.class");

/// The real javac 8 of this repository's fixture protocol (Corretto 1.8.0_432): the second leg's
/// compiler and JVM, named by path exactly as the fixture README states it.
const CORRETTO_HOME: &str = "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home";

/// The fixture's own input file, written where a run reads it.
const DATA: &str = "hello\nworld\n";

/// What the anchor's own class answers under `java -Xverify:all` (two lines, joined by `|`).
const IO_BEHAVIOR: &str = "2/hello|world|";

/// What the mid-read driver answers: the normal count, then the exception's own message and the
/// close the failing reader observed.
const MID_BEHAVIOR: &str = "normal=3\ncaught=read 3 failed closed=true";

/// The refusal `readAll` keeps, verbatim: the copy family's registered boundary.
const READ_ALL_REFUSAL: [&str; 3] = [
    "// @bytecode 0 17 27 37 44 53",
    "// local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
    "readAll(Ljava/lang/String;)Ljava/lang/String;` produced no statement",
];

/// The refusal `NestedDepth.fourLayer` keeps: the depth boundary one layer past the chain.
const FOUR_LAYER_REFUSAL: [&str; 2] = [
    "// @bytecode 0",
    "// the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds",
];

/// The negatives' refusals, verbatim (both legs state the same two sentences).
const TWO_NESTED_REFUSAL: [&str; 2] = [
    "// @bytecode 0 21 29 35 50 59",
    "// local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
];
const CLOSE_RETURNS_REFUSAL: [&str; 2] = [
    "// @bytecode 0 11 19 25 34",
    "// local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
];

/// `countLines`, whole: the one `try { … } finally { r.close(); }` its source wrote, with the
/// resource local declared above the statement and the three-layer chain's argument positions
/// written with the two `java.io` widening rows' own casts.
const COUNT_LINES: &str = "    static int countLines(java.lang.String arg0) throws java.io.IOException {\n        // @method countLines(Ljava/lang/String;)I\n        // @declaration a static method of `IO`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.io.BufferedReader local1;\n        local1 = new java.io.BufferedReader((java.io.Reader) new java.io.InputStreamReader((java.io.InputStream) new java.io.FileInputStream(arg0), \"UTF-8\"));\n        try {\n            int local2;\n            local2 = 0;\n            while (local1.readLine() != null) {\n                local2 = local2 + 1;\n            }\n            int local4 = local2;\n            return local4;\n        } finally {\n            local1.close();\n        }\n    }\n";

/// `countRemaining`, whole: the same certificate over a caller-owned stream.
const COUNT_REMAINING: &str = "    static int countRemaining(java.io.BufferedReader arg0) throws java.io.IOException {\n        // @method countRemaining(Ljava/io/BufferedReader;)I\n        // @declaration a static method of `IOMidRead`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.io.BufferedReader local1;\n        local1 = arg0;\n        try {\n            int local2;\n            local2 = 0;\n            while (local1.readLine() != null) {\n                local2 = local2 + 1;\n            }\n            int local4 = local2;\n            return local4;\n        } finally {\n            local1.close();\n        }\n    }\n";

/// `NestedDepth.threeLayer`, whole: the three-layer chain as one `new` expression.
const THREE_LAYER: &str = "    public static java.lang.String threeLayer() {\n        // @method threeLayer()Ljava/lang/String;\n        // @declaration a static method of `NestedDepth`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        return new NestedDepth$First(new NestedDepth$Second(new NestedDepth$Third(\"t\"))).inner.inner.s;\n    }\n";

/// The `IO` class's whole text, pinned: `countLines` presented, `readAll` the registered boundary,
/// `main` the patrol's own driver.
const IO_TEXT: &str = "// jarde: presentation of `IO` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic class IO extends java.lang.Object {\n    public IO() {\n        // @method <init>()V\n        // @declaration a constructor of `IO`, member flags 0x0001\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static int countLines(java.lang.String arg0) throws java.io.IOException {\n        // @method countLines(Ljava/lang/String;)I\n        // @declaration a static method of `IO`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.io.BufferedReader local1;\n        local1 = new java.io.BufferedReader((java.io.Reader) new java.io.InputStreamReader((java.io.InputStream) new java.io.FileInputStream(arg0), \"UTF-8\"));\n        try {\n            int local2;\n            local2 = 0;\n            while (local1.readLine() != null) {\n                local2 = local2 + 1;\n            }\n            int local4 = local2;\n            return local4;\n        } finally {\n            local1.close();\n        }\n    }\n\n    static java.lang.String readAll(java.lang.String arg0) throws java.io.IOException {\n        // jarde: not recovered: the recovery run for `readAll(Ljava/lang/String;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method readAll(Ljava/lang/String;)Ljava/lang/String;\n        // @declaration a static method of `IO`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 0 17 27 37 44 53\n        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice\n    }\n\n    public static void main(java.lang.String[] arg0) throws java.lang.Exception {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `IO`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println(\"\" + countLines(\"data.txt\") + \"/\" + readAll(\"data.txt\").replace((java.lang.CharSequence) \"\\n\", (java.lang.CharSequence) \"|\"));\n        return;\n    }\n}\n";

/// The `IOMidRead` class's whole text, pinned.
const MID_TEXT: &str = "// jarde: presentation of `IOMidRead` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic final class IOMidRead extends java.lang.Object {\n    private IOMidRead() {\n        // @method <init>()V\n        // @declaration a constructor of `IOMidRead`, member flags 0x0002\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static int countRemaining(java.io.BufferedReader arg0) throws java.io.IOException {\n        // @method countRemaining(Ljava/io/BufferedReader;)I\n        // @declaration a static method of `IOMidRead`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.io.BufferedReader local1;\n        local1 = arg0;\n        try {\n            int local2;\n            local2 = 0;\n            while (local1.readLine() != null) {\n                local2 = local2 + 1;\n            }\n            int local4 = local2;\n            return local4;\n        } finally {\n            local1.close();\n        }\n    }\n}\n";

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

/// The anchor's three legs, with the policy each artifact kind takes.
fn anchor_legs() -> Vec<(&'static str, Vec<u8>, EnvironmentPolicy)> {
    vec![
        ("patrol-jar", JAR.to_vec(), EnvironmentPolicy::PlainJar),
        ("v8", IO_V8.to_vec(), EnvironmentPolicy::SingleClass),
        (
            "v8-javac8",
            IO_V8_JAVAC8.to_vec(),
            EnvironmentPolicy::SingleClass,
        ),
    ]
}

#[test]
fn the_anchors_guard_presents_whole_on_every_leg() {
    let mut texts = Vec::new();
    for (leg, bytes, policy) in anchor_legs() {
        let snapshot = open(&bytes);
        let report = report(&snapshot, "IO", policy);
        assert!(
            report.text.starts_with("// jarde: presentation of `IO`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        assert_eq!(
            method_text(&report, "countLines"),
            COUNT_LINES,
            "{leg}: the guard is the one `try {{ … }} finally {{ r.close(); }}` its source wrote"
        );
        assert_eq!(
            report.text, IO_TEXT,
            "{leg}: the class renders its guard, its registered boundary and its driver"
        );
        // The family's refusal is gone from the guard: one `finally`, one close, no quote.
        let guard = method_text(&report, "countLines");
        assert_eq!(guard.matches("finally {").count(), 1, "{leg}:\n{guard}");
        assert_eq!(
            guard.matches("local1.close();").count(),
            1,
            "{leg}: the close runs once, on the handle the body read:\n{guard}"
        );
        assert!(
            !guard.contains("@bytecode"),
            "{leg}: no instruction of the guard stays quoted:\n{guard}"
        );
        // The registered boundary stays visible, verbatim, in the member beside it.
        let refused = method_text(&report, "readAll");
        for refusal in READ_ALL_REFUSAL {
            assert!(
                refused.contains(refusal),
                "{leg}/readAll: the registered boundary keeps `{refusal}` verbatim:\n{refused}"
            );
        }
        assert!(
            !refused.contains("finally {"),
            "{leg}/readAll: the copy family's criterion is not widened here:\n{refused}"
        );
        texts.push((leg, report.text));
    }
    // The three legs are the same presentation: the shape is control flow, not one compiler's
    // lowering and not one debug-flag set.
    assert_eq!(
        texts[0].1, texts[1].1,
        "the patrol's jar and the v8 leg agree"
    );
    assert_eq!(texts[1].1, texts[2].1, "the two compiler legs agree");
}

#[test]
fn the_certificate_shape_over_a_caller_owned_stream_presents() {
    for (leg, bytes) in [("v8", MID_V8), ("v8-javac8", MID_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(&snapshot, "IOMidRead", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `IOMidRead`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        assert_eq!(
            report.text, MID_TEXT,
            "{leg}: the same certificate over a caller-owned stream renders the same statement"
        );
        assert_eq!(method_text(&report, "countRemaining"), COUNT_REMAINING);
    }
}

#[test]
fn the_negatives_keep_their_refusals_verbatim() {
    for (leg, bytes) in [("v8", NEGATIVES_V8), ("v8-javac8", NEGATIVES_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(&snapshot, "IONegatives", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `IONegatives`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        for (name, refusals) in [
            ("twoNested", &TWO_NESTED_REFUSAL[..]),
            ("closeReturns", &CLOSE_RETURNS_REFUSAL[..]),
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
fn the_depth_boundary_presents_three_layers_and_refuses_four() {
    for (leg, bytes) in [("v8", DEPTH_V8), ("v8-javac8", DEPTH_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(&snapshot, "NestedDepth", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `NestedDepth`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        assert_eq!(
            method_text(&report, "threeLayer"),
            THREE_LAYER,
            "{leg}: the chain's own depth presents as one expression"
        );
        let four = method_text(&report, "fourLayer");
        assert!(
            four.contains("@bytecode") && !four.contains("new NestedDepth$Fourth"),
            "{leg}: one layer deeper keeps the outermost refusal:\n{four}"
        );
        for refusal in FOUR_LAYER_REFUSAL {
            assert!(
                four.contains(refusal),
                "{leg}/fourLayer: the depth refusal keeps `{refusal}` verbatim:\n{four}"
            );
        }
    }
}

// -------------------------------------------------------------------------------------------
// The replay: the presented text, stripped, compiled and run (needs both JDKs).
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names, whether it is the real JDK 8, and its class files.
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

    fn compile(&self, source: &Path, output: &Path) {
        let result = self
            .javac()
            .args(["-d"])
            .arg(output)
            .arg(source)
            .output()
            .expect("the leg's compiler is installed");
        assert!(
            result.status.success(),
            "{}: javac rejected the presented text:\n{}\n{}",
            self.label,
            String::from_utf8_lossy(&result.stderr),
            fs::read_to_string(source).expect("the text reads")
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
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-io-resource-{label}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create the replay directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
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

/// The fixture's committed class files of one leg.
fn fixture_files(real_javac8: bool) -> Vec<(&'static str, &'static [u8])> {
    if real_javac8 {
        vec![
            ("IO.class", IO_V8_JAVAC8),
            ("IOMidRead.class", MID_V8_JAVAC8),
        ]
    } else {
        vec![("IO.class", IO_V8), ("IOMidRead.class", MID_V8)]
    }
}

/// The driver source the mid-read leg is run by (a fixture source, compiled per leg here).
const DRIVER_SOURCE: &str =
    include_str!("fixtures/recover-io-resource-finally/IOMidReadDriver.java");

#[test]
#[ignore = "needs both JDKs: it strips the class-source comment lines, compiles the text with \
            `javac --release 8` **and** the real javac 8 and runs both under `-Xverify:all` (see \
            the module doc)"]
fn the_mid_read_guard_answers_what_its_class_answers() {
    for leg in LEGS {
        let temp = TempDir::new("mid-read");
        let original = temp.path().join("original");
        let presented = temp.path().join("presented");
        fs::create_dir_all(&original).expect("create the original directory");
        fs::create_dir_all(&presented).expect("create the presented directory");
        for (name, bytes) in fixture_files(leg.real_javac8) {
            fs::write(original.join(name), bytes).expect("the fixture class is written");
        }
        let source = presented.join("IOMidRead.java");
        fs::write(
            &source,
            stripped(
                &report(
                    &open(if leg.real_javac8 {
                        MID_V8_JAVAC8
                    } else {
                        MID_V8
                    }),
                    "IOMidRead",
                    EnvironmentPolicy::SingleClass,
                )
                .text,
            ),
        )
        .expect("write the presented text");
        leg.compile(&source, &presented);
        // The driver is compiled beside each side: against the fixture's own class and against the
        // presented one, so both runs exercise the same driver source.
        let driver = temp.path().join("IOMidReadDriver.java");
        fs::write(&driver, DRIVER_SOURCE).expect("write the driver source");
        for directory in [&original, &presented] {
            let result = leg
                .javac()
                .args(["-cp"])
                .arg(directory)
                .args(["-d"])
                .arg(directory)
                .arg(&driver)
                .output()
                .expect("the leg's compiler is installed");
            assert!(
                result.status.success(),
                "{}: the driver compiles beside {directory:?}:\n{}",
                leg.label,
                String::from_utf8_lossy(&result.stderr)
            );
        }
        assert_eq!(
            leg.run(&original, "IOMidReadDriver"),
            MID_BEHAVIOR,
            "{}: the fixture's own class answers the normal count, the mid-read failure and the \
             close it observed",
            leg.label
        );
        assert_eq!(
            leg.run(&presented, "IOMidReadDriver"),
            MID_BEHAVIOR,
            "{}: the presented text answers exactly what the class answers, the close on the \
             exceptional path included",
            leg.label
        );
    }
}

#[test]
#[ignore = "needs both JDKs: it runs the anchor's own classes on both legs and states why the \
            whole-class text stays uncompilable while `readAll` is the registered boundary (see \
            the module doc)"]
fn the_anchors_own_class_answers_its_baseline_and_its_boundary_is_stated() {
    for leg in LEGS {
        let temp = TempDir::new("anchor");
        let directory = temp.path().join("original");
        fs::create_dir_all(&directory).expect("create the original directory");
        for (name, bytes) in fixture_files(leg.real_javac8) {
            fs::write(directory.join(name), bytes).expect("the fixture class is written");
        }
        fs::write(directory.join("data.txt"), DATA).expect("the input file is written");
        assert_eq!(
            leg.run(&directory, "IO"),
            IO_BEHAVIOR,
            "{}: the anchor's own class answers its baseline (the behavior the guard must keep)",
            leg.label
        );

        // The boundary, stated rather than hidden: `readAll`'s refusal leaves the member without a
        // `return`, so the whole-class text does not compile while that criterion stands. A run
        // that compiles it is a change this test wants to hear about — the follow-up slice's own
        // acceptance will flip this assertion with its evidence.
        let source = temp.path().join("IO.java");
        fs::write(
            &source,
            stripped(
                &report(
                    &open(if leg.real_javac8 { IO_V8_JAVAC8 } else { IO_V8 }),
                    "IO",
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
            "{}: the whole-class text must stay uncompilable while `readAll`'s copy-family \
             boundary stands:\n{}",
            leg.label,
            fs::read_to_string(&source).expect("the text reads")
        );
    }
}
