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
//! * `ProbeControls.guardIfFirst` is the guard-body violation control: an `if`-position dance with
//!   an observable target inside the protected range. The assignment would be written in place
//!   there too, but the position is not the loop's own test, so the movement rule keeps the
//!   refusal it has always had and the method stays quoted;
//! * `MultiCopy` is `Probe` with the loop test's `iconst_m1` patched into a second `dup` (the same
//!   one byte): the copy the store takes now has a second consumer and the test reads a copy of a
//!   copy. It stays refused, so the admission is the two-consumer identity and not "any store in a
//!   test".
//!
//! The tests pin the presented texts, keep every refusal verbatim on both legs, and the ignored
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

/// The probe's whole text, pinned (the same on both compiler legs and from the jar).
const PROBE_TEXT: &str = "// jarde: presentation of `Probe` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic class Probe extends java.lang.Object {\n    public Probe() {\n        // @method <init>()V\n        // @declaration a constructor of `Probe`, member flags 0x0001\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static java.lang.String readAll(java.lang.String arg0) throws java.io.IOException {\n        // @method readAll(Ljava/lang/String;)Ljava/lang/String;\n        // @declaration a static method of `Probe`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.io.FileReader local1;\n        java.lang.StringBuilder local2;\n        local1 = new java.io.FileReader(arg0);\n        local2 = new java.lang.StringBuilder();\n        int local3;\n        while ((local3 = local1.read()) != -1) {\n            local2.append((char) local3);\n        }\n        local1.close();\n        return local2.toString();\n    }\n\n    static int guardPlain(java.lang.String arg0) throws java.io.IOException {\n        // @method guardPlain(Ljava/lang/String;)I\n        // @declaration a static method of `Probe`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.io.FileReader local1;\n        int local2;\n        local1 = new java.io.FileReader(arg0);\n        local2 = 0;\n        try {\n            local2 = local1.read();\n            if (local2 > 0) {\n                local2 = local2 + 1;\n            }\n            int local3;\n            while ((local3 = local1.read()) != -1) {\n                local2 = local2 + local3;\n            }\n        } finally {\n            local1.close();\n        }\n        return local2;\n    }\n\n    public static void main(java.lang.String[] arg0) throws java.lang.Exception {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `Probe`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println(readAll(\"data.txt\").replace((java.lang.CharSequence) \"\\n\", (java.lang.CharSequence) \"|\") + \"/\" + guardPlain(\"data.txt\"));\n        return;\n    }\n}\n";

/// The controls' whole text, pinned: both members quoted, the main presented.
const CONTROLS_TEXT: &str = "// jarde: presentation of `ProbeControls` from the class file's own declaration and one recovery run per member.\n// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.\npublic class ProbeControls extends java.lang.Object {\n    public ProbeControls() {\n        // @method <init>()V\n        // @declaration a constructor of `ProbeControls`, member flags 0x0001\n        // recovered from bytecode; presentation is not claimed to compile\n        super();\n        return;\n    }\n\n    static int parameterTarget(int arg0, java.lang.String arg1) throws java.io.IOException {\n        // jarde: not recovered: the recovery run for `parameterTarget(ILjava/lang/String;)I` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method parameterTarget(ILjava/lang/String;)I\n        // @declaration a static method of `ProbeControls`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 0 11 21 28\n        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice\n    }\n\n    static int guardIfFirst(java.lang.String arg0) throws java.io.IOException {\n        // jarde: not recovered: the recovery run for `guardIfFirst(Ljava/lang/String;)I` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method guardIfFirst(Ljava/lang/String;)I\n        // @declaration a static method of `ProbeControls`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 0 3 4 5 8 9 10 11 12 15 16 17 20 21 22 23 24 25 28 29 30 31 34 35 36 37 38 41 42 45 48 50 51 54 56 57 58\n        // the local assignment condition was not completely proved\n    }\n\n    public static void main(java.lang.String[] arg0) throws java.lang.Exception {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `ProbeControls`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println(\"\" + parameterTarget(0, \"data.txt\") + \"/\" + guardIfFirst(\"data.txt\"));\n        return;\n    }\n}\n";

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

/// The guard-body violation control's refusal, verbatim.
const GUARD_IF_REFUSAL: &str = "// the local assignment condition was not completely proved";

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

#[test]
fn the_controls_keep_their_refusals_on_both_legs() {
    for leg in LEGS {
        let bytes = if leg.real_javac8 {
            CONTROLS_V8_JAVAC8
        } else {
            CONTROLS_V8
        };
        let snapshot = open(bytes);
        let report = report(&snapshot, "ProbeControls", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `ProbeControls`"),
            "{}: the render states its own header:\n{}",
            leg.label,
            report.text
        );
        assert_eq!(
            report.text, CONTROLS_TEXT,
            "{}: both controls keep their refusals and the driver presents",
            leg.label
        );
        let parameter = method_text(&report, "parameterTarget");
        for refusal in PARAMETER_TARGET_REFUSAL {
            assert!(
                parameter.contains(refusal),
                "{}: the parameter target keeps `{refusal}` verbatim:\n{parameter}",
                leg.label
            );
        }
        assert!(
            !parameter.contains("while ((c ="),
            "{}: the parameter target's loop is not presented:\n{parameter}",
            leg.label
        );
        let guard_if = method_text(&report, "guardIfFirst");
        assert!(
            guard_if.contains(GUARD_IF_REFUSAL) && guard_if.contains("@bytecode"),
            "{}: the guard body's if-position dance is refused whole:\n{guard_if}",
            leg.label
        );
        assert!(
            !guard_if.contains("total = fr.read()"),
            "{}: the if-position assignment is not written:\n{guard_if}",
            leg.label
        );
    }
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
