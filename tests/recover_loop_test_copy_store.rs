//! `recover-loop-test-copy-store`: the loop test's copy-and-store — the assignment javac writes
//! when the value a loop's own test consumes is one the source assigned
//! (`while ((c = fr.read()) != -1)`) — presents as the in-place assignment expression, at the
//! store's own operand position, inside the guard body the resource-guard certificate presents.
//!
//! The anchor is the io-wrapping patrol's own `IO` (`openspec/evidence/java-syntax-2026-10-05/
//! io-wrapping-patrol/fixture/io.jar`): `readAll` was the io slice's registered boundary, and it
//! renders whole now — the same text its `try { … } finally { fr.close(); }` sibling `countLines`
//! has had since that slice, with the loop's test written as `while ((local3 = local1.read()) !=
//! -1)`. That anchor, its driver and the guard certificate's own control live in
//! `tests/recover_io_resource_finally.rs`, which this change updates; the fixtures here are this
//! slice's own probe and controls:
//!
//! * `Probe.readAll` is the **same form outside a guard body**: the copy family's position alone,
//!   with no certificate in between. It presents on both compiler legs and from a jar, byte for
//!   byte the same text;
//! * `Probe.guardPlain` is the control the guard-body negative is read against: the same guard, the
//!   same loop and an `if` inside the protected body, with **no** dance in the `if` — it presents,
//!   so the refusal beside it is the dance's and not the guard body's;
//! * `ProbeControls.parameterTarget` is the target-kind control: the loop test assigns to a
//!   **parameter**, whose declaration is the signature and not a body statement — the in-place
//!   expression is the copy family's local form, so the shape keeps the region layer's refusal;
//! * `ProbeControls.guardIfFirst` now preserves the first read as a same-block saved copy,
//!   stores its result before the condition inside `try`, and retains the loop-test assignment.
//!   This is not a widened protected non-loop assignment-expression rule. Parameter-target and
//!   multi-consumer refusals remain independent controls on both physical compiler legs.
//! * `MultiCopy` is `Probe` with the loop test's `iconst_m1` patched into a second `dup` (the same
//!   one byte): the copy the store takes now has a second consumer and the test reads a copy of a
//!   copy. It stays refused, so the admission is the two-consumer identity and not "any store in a
//!   test".
//!
//! The tests pin current presented texts and retained complete refusals on both frozen legs.
//! The new whole-source gates below never edit generated Java. The historical ignored
//! replay strips the presentations the way the patrols' own stripped sources were made (comment
//! lines dropped), compiles each with the installed `javac --release 8` and, when a real javac 8 is
//! present, with that one too, and runs the probe's own driver — reading a real file, to EOF —
//! under `-Xverify:all` beside the fixture's own class.

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

/// One compiler leg: the label a failure names and the class files it produced.
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

const PROBE_V8: &[u8] = include_bytes!("fixtures/recover-loop-test-copy-store/v8/Probe.class");
const PROBE_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-loop-test-copy-store/v8-javac8/Probe.class");
const CONTROLS_V8: &[u8] =
    include_bytes!("fixtures/recover-loop-test-copy-store/v8/ProbeControls.class");
const CONTROLS_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-loop-test-copy-store/v8-javac8/ProbeControls.class");
const MULTI_COPY_V8: &[u8] =
    include_bytes!("fixtures/recover-loop-test-copy-store/v8/MultiCopy.class");
const MULTI_COPY_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-loop-test-copy-store/v8-javac8/MultiCopy.class");

/// The real javac 8 of this repository's fixture protocol (Corretto 1.8.0_432): the second leg's
/// compiler and JVM, named by path exactly as the fixture README states it.
const CORRETTO_HOME: &str = "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home";

/// The fixture's own input file, written where a run reads it.
const DATA: &str = "hello\nworld\n";

/// What the fixture's own `Probe` answers (its driver reads `data.txt` to EOF): the `readAll` text
/// with the newlines shown as `|`, then `guardPlain`'s sum of the file's bytes plus one per byte.
const PROBE_BEHAVIOR: &str = "hello|world|/1105";

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
        source_target: jarde::SourceTarget::JAVA_8,
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
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
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

/// The probe's whole text, pinned (the same on both compiler legs and from the jar).
const PROBE_TEXT: &str = "// jarde: presentation of `Probe` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic class Probe extends java.lang.Object {\n    public Probe() {\n        // @method <init>()V\n        // @declaration a constructor of `Probe`, member flags 0x0001\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static java.lang.String readAll(java.lang.String arg0) throws java.io.IOException {\n        // @method readAll(Ljava/lang/String;)Ljava/lang/String;\n        // @declaration a static method of `Probe`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.io.FileReader local1;\n        java.lang.StringBuilder local2;\n        local1 = new java.io.FileReader(arg0);\n        local2 = new java.lang.StringBuilder();\n        int local3;\n        while ((local3 = local1.read()) != -1) {\n            local2.append((char) local3);\n        }\n        local1.close();\n        return local2.toString();\n    }\n\n    static int guardPlain(java.lang.String arg0) throws java.io.IOException {\n        // @method guardPlain(Ljava/lang/String;)I\n        // @declaration a static method of `Probe`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.io.FileReader local1;\n        int local2;\n        local1 = new java.io.FileReader(arg0);\n        local2 = 0;\n        try {\n            local2 = local1.read();\n            if (local2 > 0) {\n                local2 = local2 + 1;\n            }\n            int local3;\n            while ((local3 = local1.read()) != -1) {\n                local2 = local2 + local3;\n            }\n        } finally {\n            local1.close();\n        }\n        return local2;\n    }\n\n    public static void main(java.lang.String[] arg0) throws java.lang.Exception {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `Probe`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println(readAll(\"data.txt\").replace((java.lang.CharSequence) \"\\n\", (java.lang.CharSequence) \"|\") + \"/\" + guardPlain(\"data.txt\"));\n        return;\n    }\n}\n";

/// Current whole text: parameter refused, sequential protected copy presented, main retained.
const CONTROLS_TEXT: &str = "// jarde: presentation of `ProbeControls` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic class ProbeControls extends java.lang.Object {\n    public ProbeControls() {\n        // @method <init>()V\n        // @declaration a constructor of `ProbeControls`, member flags 0x0001\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static int parameterTarget(int arg0, java.lang.String arg1) throws java.io.IOException {\n        // jarde: not recovered: the recovery run for `parameterTarget(ILjava/lang/String;)I` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method parameterTarget(ILjava/lang/String;)I\n        // @declaration a static method of `ProbeControls`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 0 11 21 28\n        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice\n    }\n\n    static int guardIfFirst(java.lang.String arg0) throws java.io.IOException {\n        // @method guardIfFirst(Ljava/lang/String;)I\n        // @declaration a static method of `ProbeControls`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.io.FileReader local1;\n        int local2;\n        local1 = new java.io.FileReader(arg0);\n        local2 = 0;\n        try {\n            int saved0 = local1.read();\n            local2 = saved0;\n            if (saved0 > 0) {\n                local2 = local2 + 1;\n            }\n            int local3;\n            while ((local3 = local1.read()) != -1) {\n                local2 = local2 + local3;\n            }\n        } finally {\n            local1.close();\n        }\n        return local2;\n    }\n\n    public static void main(java.lang.String[] arg0) throws java.lang.Exception {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `ProbeControls`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println(\"\" + parameterTarget(0, \"data.txt\") + \"/\" + guardIfFirst(\"data.txt\"));\n        return;\n    }\n}\n";

/// The refusal `MultiCopy.readAll` keeps, verbatim: the copy has a second consumer.
const MULTI_COPY_REFUSAL: [&str; 6] = [
    "        // jarde: not recovered: the recovery run for `readAll(Ljava/lang/String;)Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below",
    "        // @method readAll(Ljava/lang/String;)Ljava/lang/String;",
    "        // @declaration a static method of `Probe`, member flags 0x0008",
    "        // recovered from bytecode; presentation is not claimed to compile",
    "        // @bytecode 0 17 27 37",
    "        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
];

/// The parameter-target control's refusal, verbatim (the loop's test is refused by the region
/// layer, at the dance's own BCI).
const PARAMETER_TARGET_REFUSAL: [&str; 2] = [
    "// @bytecode 0 11 21 28",
    "// local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
];

/// Sequential copy evaluation is separate from the protected assignment-expression gate.

#[test]
fn the_same_form_loop_presents_on_every_leg() {
    for leg in LEGS {
        let bytes = if leg.real_javac8 {
            PROBE_V8_JAVAC8
        } else {
            PROBE_V8
        };
        let snapshot = open(bytes);
        let report = report(&snapshot, "Probe", EnvironmentPolicy::SingleClass);
        assert!(
            report.text.starts_with("// jarde: presentation of `Probe`"),
            "{}: the render states its own header before anything is counted:\n{}",
            leg.label,
            report.text
        );
        assert_eq!(
            report.text, PROBE_TEXT,
            "{}: the same-form loop presents whole, outside any guard body",
            leg.label
        );
        assert!(
            !report.text.contains("@bytecode") && !report.text.contains("not recovered"),
            "{}: the probe carries no refusal:\n{}",
            leg.label,
            report.text
        );
        assert_eq!(
            method_text(&report, "readAll")
                .matches("(local3 = local1.read()) != -1")
                .count(),
            1,
            "{}: the assignment stands at the test's own operand position, once:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn the_probe_renders_the_same_text_from_a_jar() {
    let jar = zip_of(&[
        (b"Probe.class", PROBE_V8),
        (b"ProbeControls.class", CONTROLS_V8),
    ]);
    let snapshot = open(&jar);
    let report = report(&snapshot, "Probe", EnvironmentPolicy::PlainJar);
    assert!(
        report.text.starts_with("// jarde: presentation of `Probe`"),
        "the jar render states its own header:\n{}",
        report.text
    );
    assert_eq!(
        report.text, PROBE_TEXT,
        "the same class renders the same text from a jar as from a standalone class file"
    );
}

// Each physical leg and each contract is independently visible even when another fails.
fn controls_parameter(bytes: &[u8]) {
    let r = report(
        &open(bytes),
        "ProbeControls",
        EnvironmentPolicy::SingleClass,
    );
    let text = method_text(&r, "parameterTarget");
    assert_refusal_closure(&r, "parameterTarget");
    for refusal in PARAMETER_TARGET_REFUSAL {
        assert!(text.contains(refusal), "{text}");
    }
    assert!(
        text.lines()
            .skip(1)
            .filter(|line| !line.trim().is_empty() && line.trim() != "}")
            .all(|line| line.trim().starts_with("//")),
        "{text}"
    );
}
fn controls_guard(bytes: &[u8]) {
    let r = report(
        &open(bytes),
        "ProbeControls",
        EnvironmentPolicy::SingleClass,
    );
    let text = method_text(&r, "guardIfFirst");
    assert!(
        !text.contains("@bytecode") && !text.contains("not recovered"),
        "{text}"
    );
    let sequence = [
        "try {",
        "int saved0 = local1.read();",
        "local2 = saved0;",
        "if (saved0 > 0)",
        "local2 = local2 + 1;",
        "while ((local3 = local1.read()) != -1)",
        "local2 = local2 + local3;",
        "} finally {",
        "local1.close();",
        "return local2;",
    ];
    let positions: Vec<_> = sequence
        .iter()
        .map(|part| {
            text.find(part)
                .unwrap_or_else(|| panic!("missing {part}: {text}"))
        })
        .collect();
    assert!(positions.windows(2).all(|p| p[0] < p[1]), "{text}");
    assert_eq!(text.matches("local1.read()").count(), 2);
    assert_eq!(text.matches("local1.close()").count(), 1);
    assert!(
        !text.contains("if ((local2 ="),
        "no newly admitted protected assignment expression"
    );
    let method = r
        .methods
        .iter()
        .find(|m| m.item.name.raw().0 == b"guardIfFirst")
        .unwrap();
    if let ClassSourceOutcome::Recovered { report: run, .. } = &method.outcome {
        for segment in run.source_map.segments() {
            assert!(segment.start() <= segment.end() && segment.end() <= run.text.len());
            assert!(
                run.text.is_char_boundary(segment.start())
                    && run.text.is_char_boundary(segment.end())
            );
            for origin in
                std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
            {
                assert_eq!(origin.method(), Some(&method.item.identity));
            }
        }
        for (start, end, primary, derived, span) in [
            (
                302,
                338,
                12,
                &[][..],
                "        int saved0 = local1.read();\n",
            ),
            (355, 361, 16, &[12, 15][..], "saved0"),
            (375, 381, 17, &[12, 15][..], "saved0"),
            (389, 422, 23, &[][..], "            local2 = local2 + 1;\n"),
            (477, 490, 25, &[][..], "local1.read()"),
            (468, 490, 29, &[28][..], "local3 = local1.read()"),
            (467, 497, 29, &[31][..], "(local3 = local1.read()) != -1"),
            (
                501,
                539,
                37,
                &[][..],
                "            local2 = local2 + local3;\n",
            ),
            (573, 587, 42, &[][..], "local1.close()"),
            (595, 614, 58, &[][..], "    return local2;\n"),
        ] {
            let segment = run
                .source_map
                .segments()
                .iter()
                .find(|segment| segment.start() == start && segment.end() == end)
                .expect("exact UTF-8 span exists");
            assert_eq!(&run.text[start..end], span);
            assert_eq!(segment.origin().primary().bci(), primary);
            assert_eq!(
                segment
                    .origin()
                    .derived()
                    .iter()
                    .map(|o| o.bci())
                    .collect::<Vec<_>>(),
                derived
            );
        }
        let guard = run
            .source_map
            .segments()
            .iter()
            .find(|s| s.start() == 292 && s.end() == 595)
            .unwrap();
        assert_eq!(guard.origin().primary().bci(), 11);
        assert_eq!(
            guard
                .origin()
                .derived()
                .iter()
                .map(|o| o.bci())
                .collect::<Vec<_>>(),
            [
                11, 12, 15, 16, 17, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 34, 35, 36, 37, 38, 41,
                42, 45, 48, 50, 51, 54, 56
            ]
        );
        // Cleanup 51 and rethrow 56 are structural origins, not separately emitted statements.
        // The loop backedge 38 and cleanup-continuation GOTO 45 are retained here; no general
        // claim of full instruction-to-token GOTO coverage is made.
    } else {
        panic!("no recovery report");
    }
}
#[test]
fn parameter_refusal_v8() {
    controls_parameter(CONTROLS_V8);
}
#[test]
fn parameter_refusal_javac8() {
    controls_parameter(CONTROLS_V8_JAVAC8);
}
#[test]
fn protected_sequential_copy_v8() {
    controls_guard(CONTROLS_V8);
}
#[test]
fn protected_sequential_copy_javac8() {
    controls_guard(CONTROLS_V8_JAVAC8);
}
#[test]
fn current_whole_snapshot_v8() {
    assert_eq!(
        report(
            &open(CONTROLS_V8),
            "ProbeControls",
            EnvironmentPolicy::SingleClass
        )
        .text,
        CONTROLS_TEXT
    );
}
#[test]
fn current_whole_snapshot_javac8() {
    assert_eq!(
        report(
            &open(CONTROLS_V8_JAVAC8),
            "ProbeControls",
            EnvironmentPolicy::SingleClass
        )
        .text,
        CONTROLS_TEXT
    );
}

#[test]
fn the_multi_consumer_control_keeps_its_refusal_on_both_legs() {
    for leg in LEGS {
        let (patched, plain) = if leg.real_javac8 {
            (MULTI_COPY_V8_JAVAC8, PROBE_V8_JAVAC8)
        } else {
            (MULTI_COPY_V8, PROBE_V8)
        };
        // The pair is the point: one byte apart, one presents and one refuses.
        let plain_report = report(&open(plain), "Probe", EnvironmentPolicy::SingleClass);
        assert!(
            !method_text(&plain_report, "readAll").contains("@bytecode"),
            "{}: the unpatched loop test presents:\n{}",
            leg.label,
            plain_report.text
        );
        let snapshot = open(patched);
        let report = report(&snapshot, "Probe", EnvironmentPolicy::SingleClass);
        assert!(
            report.text.starts_with("// jarde: presentation of `Probe`"),
            "{}: the patched control's render states its own header:\n{}",
            leg.label,
            report.text
        );
        let read_all = method_text(&report, "readAll");
        for refusal in MULTI_COPY_REFUSAL {
            assert!(
                read_all.contains(refusal),
                "{}: the multi-consumer control keeps `{refusal}` verbatim:\n{read_all}",
                leg.label
            );
        }
        assert!(
            !read_all.contains("(local3 = local1.read())"),
            "{}: the two-consumer identity is not claimed for a three-consumer copy:\n{read_all}",
            leg.label
        );
        // The control's other members are the anchor's: the patch moved one byte of one test.
        assert_eq!(
            method_text(&report, "guardPlain"),
            method_text(&plain_report, "guardPlain"),
            "{}: the patch moved the loop test alone",
            leg.label
        );
    }
}

// -------------------------------------------------------------------------------------------
// The replay: the presented text, stripped, compiled and run (needs both JDKs).
// -------------------------------------------------------------------------------------------

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
            "jarde-loop-test-copy-{label}-{}-{nonce}-{sequence}",
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

/// One presented class text with every `//` comment line dropped — the strip the patrols' own text
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

#[test]
#[ignore = "needs both JDKs: it strips the class-source comment lines, compiles the text with \
            `javac --release 8` **and** the real javac 8 and runs the probe's driver against a real \
            file on both sides (see the module doc)"]
fn the_probes_presented_text_answers_what_its_class_answers() {
    for leg in LEGS {
        let temp = TempDir::new("probe");
        let original = temp.path().join("original");
        let presented = temp.path().join("presented");
        fs::create_dir_all(&original).expect("create the original directory");
        fs::create_dir_all(&presented).expect("create the presented directory");
        let bytes = if leg.real_javac8 {
            PROBE_V8_JAVAC8
        } else {
            PROBE_V8
        };
        fs::write(original.join("Probe.class"), bytes).expect("the fixture class is written");
        fs::write(original.join("data.txt"), DATA).expect("the input file is written");
        fs::write(presented.join("data.txt"), DATA).expect("the input file is written");
        let source = presented.join("Probe.java");
        fs::write(
            &source,
            stripped(&report(&open(bytes), "Probe", EnvironmentPolicy::SingleClass).text),
        )
        .expect("write the presented text");
        leg.compile(&source, &presented);
        assert_eq!(
            leg.run(&original, "Probe"),
            PROBE_BEHAVIOR,
            "{}: the fixture's own class answers its baseline",
            leg.label
        );
        assert_eq!(
            leg.run(&presented, "Probe"),
            PROBE_BEHAVIOR,
            "{}: the presented text answers exactly what the class answers, the file read to EOF \
             included",
            leg.label
        );
    }
}

/// One jar of the given entries, as the fixture's own container.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    const STORE: u16 = 0;
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
// Whole-class public Engine contract. Generated Java is never edited.
const CONTRACT_RESOURCE: &str =
    include_str!("fixtures/recover-loop-test-copy-store/contract/ContractResource.java");
const CONTRACT_TARGET: &str =
    include_str!("fixtures/recover-loop-test-copy-store/contract/PlainFinallyCopy.java");
const CONTRACT_DRIVER: &str =
    include_str!("fixtures/recover-loop-test-copy-store/contract/PlainFinallyCopyDriver.java");

fn bounded_command(dir: &Path, label: &str, program: &str, args: &[&str]) -> std::process::Output {
    let output = Command::new("timeout")
        .args(["30s", program])
        .args(args)
        .current_dir(dir)
        .output()
        .expect("bounded command starts");
    fs::write(dir.join(format!("{label}.stdout")), &output.stdout).unwrap();
    fs::write(dir.join(format!("{label}.stderr")), &output.stderr).unwrap();
    fs::write(
        dir.join(format!("{label}.status")),
        format!("{:?}\n", output.status),
    )
    .unwrap();
    output
}
fn runtime_root(label: &str) -> PathBuf {
    let root = std::env::var_os("JARDE_FINALLY_COPY_EVIDENCE")
        .map(PathBuf::from)
        .expect("explicit fresh evidence directory required")
        .join(label);
    fs::create_dir_all(root.parent().unwrap()).unwrap();
    fs::create_dir(&root).expect("never overwrite evidence");
    root
}
#[test]
#[ignore = "installed JDK; independent whole original/recovered classpaths"]
fn complete_programmable_plain_finally_copy_contract() {
    let root = runtime_root("programmable");
    let input = root.join("input");
    fs::create_dir(&input).unwrap();
    for (name, text) in [
        ("ContractResource.java", CONTRACT_RESOURCE),
        ("PlainFinallyCopy.java", CONTRACT_TARGET),
        ("PlainFinallyCopyDriver.java", CONTRACT_DRIVER),
    ] {
        fs::write(input.join(name), text).unwrap();
    }
    let compile = bounded_command(
        &input,
        "javac",
        "javac",
        &[
            "--release",
            "8",
            "-g:none",
            "-proc:none",
            "ContractResource.java",
            "PlainFinallyCopy.java",
            "PlainFinallyCopyDriver.java",
        ],
    );
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let bytes = fs::read(input.join("PlainFinallyCopy.class")).unwrap();
    let r = report(
        &open(&bytes),
        "PlainFinallyCopy",
        EnvironmentPolicy::SingleClass,
    );
    fs::write(root.join("Engine.java"), &r.text).unwrap();
    let original = bounded_command(
        &input,
        "java",
        "java",
        &["-Xverify:all", "-cp", ".", "PlainFinallyCopyDriver"],
    );
    assert!(
        original.status.success(),
        "{}",
        String::from_utf8_lossy(&original.stderr)
    );
    let recovered = root.join("recovered");
    fs::create_dir(&recovered).unwrap();
    for helper in ["ContractResource.class", "PlainFinallyCopyDriver.class"] {
        fs::copy(input.join(helper), recovered.join(helper)).unwrap();
        assert_eq!(
            fs::read(input.join(helper)).unwrap(),
            fs::read(recovered.join(helper)).unwrap()
        );
    }
    fs::write(recovered.join("PlainFinallyCopy.java"), &r.text).unwrap();
    let compile = bounded_command(
        &recovered,
        "javac",
        "javac",
        &[
            "--release",
            "8",
            "-proc:none",
            "-classpath",
            ".",
            "-sourcepath",
            "empty-sourcepath",
            "PlainFinallyCopy.java",
        ],
    );
    assert!(
        !r.text.contains("@bytecode"),
        "complete target refusal:\n{}",
        r.text
    );
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let actual = bounded_command(
        &recovered,
        "java",
        "java",
        &["-Xverify:all", "-cp", ".", "PlainFinallyCopyDriver"],
    );
    assert!(
        actual.status.success(),
        "{}",
        String::from_utf8_lossy(&actual.stderr)
    );
    assert_eq!(actual.stdout, original.stdout);
}

#[test]
#[ignore = "installed JDK; derivative-only evidence, not frozen whole-class recovery"]
fn byte_preserving_guard_derivative_runtime() {
    let root = runtime_root("derivative");
    let driver = r#"import java.nio.file.*;
public class DerivativeDriver {
 public static void main(String[] args) throws Exception {
  byte[][] inputs = {new byte[0], new byte[]{0},new byte[]{65,66},new byte[]{65,0,66}};
  int[] expected = {-1,0,132,132};
  for(int i=0;i<inputs.length;i++) { Path p=Paths.get("input.bin");Files.write(p,inputs[i]);int v=ProbeControls.guardIfFirst(p.toString());if(v!=expected[i])throw new AssertionError(i+":"+v);System.out.println(i+":"+v);Files.delete(p); }
  try {ProbeControls.guardIfFirst(null);throw new AssertionError("null accepted");}catch(NullPointerException expectedFailure){System.out.println("null:NPE");}
  try {ProbeControls.guardIfFirst("absent-input");throw new AssertionError("absent accepted");}catch(java.io.FileNotFoundException expectedFailure){System.out.println("absent:FileNotFoundException");}
 }
}"#;
    for (index, (whole, derivative)) in [
        (
            CONTROLS_V8,
            include_bytes!("fixtures/recover-loop-test-copy-store/contract/v8/ProbeControls.class")
                .as_slice(),
        ),
        (
            CONTROLS_V8_JAVAC8,
            include_bytes!(
                "fixtures/recover-loop-test-copy-store/contract/v8-javac8/ProbeControls.class"
            )
            .as_slice(),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let full = report(
            &open(whole),
            "ProbeControls",
            EnvironmentPolicy::SingleClass,
        );
        let refusal = root.join(format!("leg{index}-frozen-whole"));
        fs::create_dir(&refusal).unwrap();
        fs::write(refusal.join("ProbeControls.java"), &full.text).unwrap();
        let compile = bounded_command(
            &refusal,
            "javac",
            "javac",
            &["--release", "8", "-proc:none", "ProbeControls.java"],
        );
        assert!(
            !compile.status.success(),
            "do not erase the frozen whole-class limitation"
        );
        assert!(String::from_utf8_lossy(&compile.stderr).contains("missing return statement"));
        let r = report(
            &open(derivative),
            "ProbeControls",
            EnvironmentPolicy::SingleClass,
        );
        let mut outputs = Vec::new();
        for (side, recovered) in [("original", false), ("recovered", true)] {
            let d = root.join(format!("leg{index}-{side}"));
            fs::create_dir(&d).unwrap();
            fs::write(d.join("DerivativeDriver.java"), driver).unwrap();
            fs::write(d.join("Engine.java.txt"), &r.text).unwrap();
            let mut args = vec![
                "--release",
                "8",
                "-proc:none",
                "-classpath",
                ".",
                "-sourcepath",
                "empty-sourcepath",
                "DerivativeDriver.java",
            ];
            if recovered {
                fs::write(d.join("ProbeControls.java"), &r.text).unwrap();
                args.push("ProbeControls.java");
            } else {
                fs::write(d.join("ProbeControls.class"), derivative).unwrap();
            }
            let compile = bounded_command(&d, "javac", "javac", &args);
            assert!(
                compile.status.success(),
                "{}",
                String::from_utf8_lossy(&compile.stderr)
            );
            let run = bounded_command(
                &d,
                "java",
                "java",
                &["-Xverify:all", "-cp", ".", "DerivativeDriver"],
            );
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            outputs.push(run.stdout);
        }
        assert_eq!(outputs[0], outputs[1]);
    }
}

#[test]
#[ignore = "installed JDK; same-origin metamorphic combinations, not holdouts"]
fn whole_plain_finally_copy_metamorphic_combinations() {
    let root = runtime_root("variants");
    for (label, class, method, debug, shift, change) in [
        (
            "renamed",
            "RenamedFinallyCopy",
            "compute",
            false,
            false,
            false,
        ),
        ("debug", "PlainFinallyCopy", "run", true, false, false),
        (
            "offset-values",
            "ShiftedFinallyCopy",
            "compute",
            false,
            true,
            true,
        ),
        (
            "combined-debug",
            "CombinedFinallyCopy",
            "changed",
            true,
            true,
            true,
        ),
    ] {
        // Transform only original test source/driver, never generated Engine Java.
        let mut target = CONTRACT_TARGET
            .replace("PlainFinallyCopy", class)
            .replace("int run(", &format!("int {method}("));
        if shift {
            target = target.replace(
                "ContractResource resource =",
                "int padding = 7;\n        ContractResource resource =",
            );
        }
        if change {
            target = target
                .replace("total = 0;", "total = 99;")
                .replace("total + 1;", "total + 7;");
        }
        let mut driver =
            CONTRACT_DRIVER.replace("PlainFinallyCopy.run", &format!("{class}.{method}"));
        if change {
            driver = driver.replace("false,false,132,", "false,false,138,");
        }
        let dir = root.join(label);
        fs::create_dir(&dir).unwrap();
        let original = dir.join("original");
        fs::create_dir(&original).unwrap();
        let source_file = format!("{class}.java");
        let class_file = format!("{class}.class");
        fs::write(original.join(&source_file), &target).unwrap();
        fs::write(original.join("ContractResource.java"), CONTRACT_RESOURCE).unwrap();
        fs::write(original.join("PlainFinallyCopyDriver.java"), &driver).unwrap();
        let compile = bounded_command(
            &original,
            "javac",
            "javac",
            &[
                "--release",
                "8",
                if debug { "-g" } else { "-g:none" },
                "-proc:none",
                &source_file,
                "ContractResource.java",
                "PlainFinallyCopyDriver.java",
            ],
        );
        assert!(
            compile.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let bytes = fs::read(original.join(&class_file)).unwrap();
        let r = report(&open(&bytes), class, EnvironmentPolicy::SingleClass);
        fs::write(dir.join("Engine.java"), &r.text).unwrap();
        if let Some(member) = r.methods.iter().find(|m| m.item.name.raw().0 == b"run")
            && let ClassSourceOutcome::Recovered { report: run, .. } = &member.outcome
        {
            fs::write(
                dir.join("run-report.json"),
                serde_json::to_vec_pretty(run).unwrap(),
            )
            .unwrap();
        }

        let expected = bounded_command(
            &original,
            "java",
            "java",
            &["-Xverify:all", "-cp", ".", "PlainFinallyCopyDriver"],
        );
        assert!(
            expected.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&expected.stderr)
        );
        let recovered = dir.join("recovered");
        fs::create_dir(&recovered).unwrap();
        fs::write(recovered.join(&source_file), &r.text).unwrap();
        for helper in ["ContractResource.class", "PlainFinallyCopyDriver.class"] {
            fs::copy(original.join(helper), recovered.join(helper)).unwrap();
            assert_eq!(
                fs::read(original.join(helper)).unwrap(),
                fs::read(recovered.join(helper)).unwrap()
            );
        }
        let compile = bounded_command(
            &recovered,
            "javac",
            "javac",
            &[
                "--release",
                "8",
                "-proc:none",
                "-classpath",
                ".",
                "-sourcepath",
                "empty-sourcepath",
                &source_file,
            ],
        );
        assert!(
            compile.status.success(),
            "{label}: {}\n{}",
            String::from_utf8_lossy(&compile.stderr),
            r.text
        );
        assert!(
            !r.text.contains("@bytecode"),
            "{label}: full class required"
        );
        let actual = bounded_command(
            &recovered,
            "java",
            "java",
            &["-Xverify:all", "-cp", ".", "PlainFinallyCopyDriver"],
        );
        assert!(
            actual.status.success(),
            "{label}: {}",
            String::from_utf8_lossy(&actual.stderr)
        );
        assert_eq!(actual.stdout, expected.stdout, "{label}");
    }
}

#[test]
#[ignore = "installed JDK; separate whole-class positives and exact complete refusals"]
fn independent_plain_finally_structural_controls() {
    let root = runtime_root("structural-controls");
    let mut outcomes = Vec::new();
    for (class, source) in [
        (
            "ExplicitBodyFinallyCopy",
            include_str!(
                "fixtures/recover-loop-test-copy-store/contract/ExplicitBodyFinallyCopy.java"
            ),
        ),
        (
            "EarlyReturnFinallyCopy",
            include_str!(
                "fixtures/recover-loop-test-copy-store/contract/EarlyReturnFinallyCopy.java"
            ),
        ),
        (
            "BreakFinallyCopy",
            include_str!("fixtures/recover-loop-test-copy-store/contract/BreakFinallyCopy.java"),
        ),
        (
            "ContinueFinallyCopy",
            include_str!("fixtures/recover-loop-test-copy-store/contract/ContinueFinallyCopy.java"),
        ),
        (
            "AliasFinallyCopy",
            include_str!("fixtures/recover-loop-test-copy-store/contract/AliasFinallyCopy.java"),
        ),
    ] {
        let dir = root.join(class);
        fs::create_dir(&dir).unwrap();
        let original = dir.join("original");
        fs::create_dir(&original).unwrap();
        let filename = format!("{class}.java");
        fs::write(original.join(&filename), source).unwrap();
        fs::write(original.join("ContractResource.java"), CONTRACT_RESOURCE).unwrap();
        fs::write(
            original.join("AliasState.java"),
            include_str!("fixtures/recover-loop-test-copy-store/contract/AliasState.java"),
        )
        .unwrap();
        fs::write(
            original.join("ControlDriver.java"),
            include_str!("fixtures/recover-loop-test-copy-store/contract/ControlDriver.java"),
        )
        .unwrap();
        let compile = bounded_command(
            &original,
            "javac",
            "javac",
            &[
                "--release",
                "8",
                "-g:none",
                "-proc:none",
                &filename,
                "ContractResource.java",
                "AliasState.java",
                "ControlDriver.java",
            ],
        );
        assert!(
            compile.status.success(),
            "{class}: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let expected = bounded_command(
            &original,
            "java",
            "java",
            &["-Xverify:all", "-cp", ".", "ControlDriver", class],
        );
        assert!(
            expected.status.success(),
            "{class}: {}",
            String::from_utf8_lossy(&expected.stderr)
        );
        let bytes = fs::read(original.join(format!("{class}.class"))).unwrap();
        let r = report(&open(&bytes), class, EnvironmentPolicy::SingleClass);
        fs::write(dir.join("Engine.java"), &r.text).unwrap();
        if let Some(member) = r.methods.iter().find(|m| m.item.name.raw().0 == b"run")
            && let ClassSourceOutcome::Recovered { report: run, .. } = &member.outcome
        {
            fs::write(
                dir.join("run-report.json"),
                serde_json::to_vec_pretty(run).unwrap(),
            )
            .unwrap();
        }

        let recovered = dir.join("recovered");
        fs::create_dir(&recovered).unwrap();
        fs::write(recovered.join(&filename), &r.text).unwrap();
        for helper in [
            "ContractResource.class",
            "ControlDriver.class",
            "AliasState.class",
        ] {
            fs::copy(original.join(helper), recovered.join(helper)).unwrap();
            assert_eq!(
                fs::read(original.join(helper)).unwrap(),
                fs::read(recovered.join(helper)).unwrap()
            );
        }
        let compile = bounded_command(
            &recovered,
            "javac",
            "javac",
            &[
                "--release",
                "8",
                "-proc:none",
                "-classpath",
                ".",
                "-sourcepath",
                "empty-sourcepath",
                &filename,
            ],
        );
        let expected_source = match class {
            "ExplicitBodyFinallyCopy" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/ExplicitBodyFinallyCopy.java.txt"
            ),
            "EarlyReturnFinallyCopy" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/EarlyReturnFinallyCopy.java.txt"
            ),
            "BreakFinallyCopy" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/BreakFinallyCopy.java.txt"
            ),
            "ContinueFinallyCopy" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/ContinueFinallyCopy.java.txt"
            ),
            "AliasFinallyCopy" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/AliasFinallyCopy.java.txt"
            ),
            _ => unreachable!(),
        };
        assert_eq!(
            r.text, expected_source,
            "{class}: exact whole output, including refusal closure"
        );
        let positive = matches!(class, "ExplicitBodyFinallyCopy" | "ContinueFinallyCopy");
        assert_eq!(
            compile.status.success(),
            positive,
            "{class}: do not silently change admission"
        );
        if positive {
            assert!(!r.text.contains("@bytecode"));
        }
        if compile.status.success() {
            let actual = bounded_command(
                &recovered,
                "java",
                "java",
                &["-Xverify:all", "-cp", ".", "ControlDriver", class],
            );
            assert!(
                actual.status.success(),
                "{class}: {}",
                String::from_utf8_lossy(&actual.stderr)
            );
            assert_eq!(actual.stdout, expected.stdout, "{class}");
            outcomes.push(format!(
                "{class}: whole-class runtime equivalent on 36 bounded cases; quoted={}",
                r.text.contains("@bytecode")
            ));
        } else {
            assert_refusal_closure(&r, "run");
            outcomes.push(format!(
                "{class}: whole-class compile refused; run text={}",
                method_text(&r, "run")
            ));
        }
    }
    fs::write(root.join("outcomes.txt"), outcomes.join("\n")).unwrap();
    assert_eq!(outcomes.len(), 5);
}

#[test]
#[ignore = "installed JDK; separate whole-class positives and exact complete refusals"]
fn resource_cleanup_boundaries_are_independent() {
    let root = runtime_root("boundaries");
    let mut outcomes = Vec::new();
    for (class, source) in [
        (
            "BoundaryOtherReceiver",
            include_str!(
                "fixtures/recover-loop-test-copy-store/contract/BoundaryOtherReceiver.java"
            ),
        ),
        (
            "BoundaryReassignedReceiver",
            include_str!(
                "fixtures/recover-loop-test-copy-store/contract/BoundaryReassignedReceiver.java"
            ),
        ),
        (
            "BoundaryProtectedCleanup",
            include_str!(
                "fixtures/recover-loop-test-copy-store/contract/BoundaryProtectedCleanup.java"
            ),
        ),
        (
            "BoundaryChangedRethrow",
            include_str!(
                "fixtures/recover-loop-test-copy-store/contract/BoundaryChangedRethrow.java"
            ),
        ),
        (
            "BoundarySkippedNormalCleanup",
            include_str!(
                "fixtures/recover-loop-test-copy-store/contract/BoundarySkippedNormalCleanup.java"
            ),
        ),
        (
            "BoundaryChangedCloseTarget",
            include_str!(
                "fixtures/recover-loop-test-copy-store/contract/BoundaryChangedCloseTarget.java"
            ),
        ),
    ] {
        let dir = root.join(class);
        fs::create_dir(&dir).unwrap();
        let original = dir.join("original");
        fs::create_dir(&original).unwrap();
        let filename = format!("{class}.java");
        fs::write(original.join(&filename), source).unwrap();
        fs::write(original.join("ContractResource.java"), CONTRACT_RESOURCE).unwrap();
        fs::write(
            original.join("AliasState.java"),
            include_str!("fixtures/recover-loop-test-copy-store/contract/AliasState.java"),
        )
        .unwrap();
        fs::write(
            original.join("BoundaryResource.java"),
            include_str!("fixtures/recover-loop-test-copy-store/contract/BoundaryResource.java"),
        )
        .unwrap();
        fs::write(
            original.join("BoundaryControlDriver.java"),
            include_str!(
                "fixtures/recover-loop-test-copy-store/contract/BoundaryControlDriver.java"
            ),
        )
        .unwrap();
        let compile = bounded_command(
            &original,
            "javac",
            "javac",
            &[
                "--release",
                "8",
                "-g:none",
                "-proc:none",
                &filename,
                "ContractResource.java",
                "BoundaryResource.java",
                "AliasState.java",
                "BoundaryControlDriver.java",
            ],
        );
        assert!(
            compile.status.success(),
            "{class}: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let expected = bounded_command(
            &original,
            "java",
            "java",
            &["-Xverify:all", "-cp", ".", "BoundaryControlDriver", class],
        );
        assert!(
            expected.status.success(),
            "{class}: {}",
            String::from_utf8_lossy(&expected.stderr)
        );
        let bytes = fs::read(original.join(format!("{class}.class"))).unwrap();
        let r = report(&open(&bytes), class, EnvironmentPolicy::SingleClass);
        fs::write(dir.join("Engine.java"), &r.text).unwrap();
        if let Some(member) = r.methods.iter().find(|m| m.item.name.raw().0 == b"run")
            && let ClassSourceOutcome::Recovered { report: run, .. } = &member.outcome
        {
            fs::write(
                dir.join("run-report.json"),
                serde_json::to_vec_pretty(run).unwrap(),
            )
            .unwrap();
        }

        let recovered = dir.join("recovered");
        fs::create_dir(&recovered).unwrap();
        fs::write(recovered.join(&filename), &r.text).unwrap();
        for helper in [
            "ContractResource.class",
            "BoundaryResource.class",
            "BoundaryControlDriver.class",
            "AliasState.class",
        ] {
            fs::copy(original.join(helper), recovered.join(helper)).unwrap();
            assert_eq!(
                fs::read(original.join(helper)).unwrap(),
                fs::read(recovered.join(helper)).unwrap()
            );
        }
        let compile = bounded_command(
            &recovered,
            "javac",
            "javac",
            &[
                "--release",
                "8",
                "-proc:none",
                "-classpath",
                ".",
                "-sourcepath",
                "empty-sourcepath",
                &filename,
            ],
        );
        assert!(
            !method_text(&r, "run").contains("finally {"),
            "{class}: must not invent resource-finally cleanup"
        );
        let expected_source = match class {
            "BoundaryChangedCloseTarget" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/BoundaryChangedCloseTarget.java.txt"
            ),
            "BoundaryChangedRethrow" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/BoundaryChangedRethrow.java.txt"
            ),
            "BoundaryOtherReceiver" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/BoundaryOtherReceiver.java.txt"
            ),
            "BoundaryProtectedCleanup" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/BoundaryProtectedCleanup.java.txt"
            ),
            "BoundaryReassignedReceiver" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/BoundaryReassignedReceiver.java.txt"
            ),
            "BoundarySkippedNormalCleanup" => include_str!(
                "fixtures/recover-loop-test-copy-store/contract/expected/BoundarySkippedNormalCleanup.java.txt"
            ),
            _ => unreachable!(),
        };
        assert_eq!(r.text, expected_source, "{class}: exact complete refusal");
        assert!(
            !compile.status.success(),
            "{class}: do not change admission"
        );
        assert!(String::from_utf8_lossy(&compile.stderr).contains("missing return statement"));
        assert_refusal_closure(&r, "run");
        if let ClassSourceOutcome::Recovered { report: run, .. } = &r
            .methods
            .iter()
            .find(|m| m.item.name.raw().0 == b"run")
            .unwrap()
            .outcome
        {
            assert_eq!(
                run.fallbacks,
                ["jre_region_exception_edge", "jre_region_uncovered_blocks"]
            );
        }
        outcomes.push(format!("{class}: original passed 36 independent expected records; recovered complete refusal; no recovered execution"));
    }
    fs::write(root.join("outcomes.txt"), outcomes.join("\n")).unwrap();
    assert_eq!(outcomes.len(), 6);
}

fn assert_refusal_closure(r: &ClassSourceReport, name: &str) {
    use std::collections::BTreeSet;
    let member = r
        .methods
        .iter()
        .find(|m| m.item.name.raw().0 == name.as_bytes())
        .unwrap();
    let ClassSourceOutcome::Recovered { report: run, .. } = &member.outcome else {
        panic!("missing run");
    };
    assert_eq!(run.content, RecoveryContent::ExplanationOnly);
    assert!(
        run.text.lines().all(|line| line.trim().is_empty()
            || line.trim() == "{"
            || line.trim() == "}"
            || line.trim().starts_with("//")),
        "no executable fragment: {}",
        run.text
    );
    let quoted: BTreeSet<u32> = run
        .text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|line| line.split_whitespace())
        .map(|value| value.parse().unwrap())
        .collect();
    assert!(!quoted.is_empty());
    let mut origins = BTreeSet::new();
    for segment in run.source_map.segments() {
        assert!(
            run.text.is_char_boundary(segment.start()) && run.text.is_char_boundary(segment.end())
        );
        for origin in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(origin.method(), Some(&member.item.identity));
            origins.insert(origin.bci());
        }
    }
    assert_eq!(origins, quoted, "exact quoted origin closure");
    for region in &run.regions {
        assert!(quoted.contains(&region.bci));
        assert!(region.blocks.iter().all(|bci| quoted.contains(bci)));
    }
}
