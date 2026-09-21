//! P3: a parameter's **slot** decides the name its declaration and its body agree on, and an array
//! never takes its element's width.
//!
//! JVMS 2.6.1 gives a `long` or a `double` two local slots and every other type one, and an array
//! of either is a reference — `[J` fills one slot exactly like `[I`. The class-source presentation
//! read the array's slot width off its **element**, so every parameter after an array of a wide
//! primitive was placed one slot too far: the signature of
//!
//! ```text
//! static int afterArray(long[] arg0, int arg2) { return arg1; }
//! ```
//!
//! declared `arg2` where the body read `arg1`, and `javac --release 8` refuses the text
//! (`cannot find symbol: variable arg1`). The `-g:none` sample this file drives
//! ([`SAMPLE`], `tests/fixtures/p3-parameter-slots/`) states the shapes that do it and the controls
//! that must not move: a `long[]`, a `long[][]` and a `double[][]` before an `int`/`long`, two wide
//! parameters around an array, two arrays whose elements are wide, an instance method whose receiver
//! is slot 0, and the `int[]` control.
//!
//! # What is pinned, and where
//!
//! * **the text**: every member's declaration and the statement the body writes are asserted
//!   exactly ([`the_declaration_and_the_body_name_one_slot_one_way`]), so the name in the signature
//!   and the name in the statement below it are the same reading of one slot;
//! * **the behaviour**: the presentation's own text is compiled with `javac --release 8 -g:none` and
//!   run beside the committed class under one shared driver
//!   ([`the_presented_class_answers_what_the_committed_class_does`]), which is the comparison a
//!   wrong name cannot survive — the presented text would not compile at all;
//! * **the entry point**: both tests drive [`Engine::class_source`] — the presentation that has the
//!   declaration in it. The `Engine::recover_method` entry (one body, no declaration) is covered by
//!   the same sample's row in `tests/p3_execution_comparison.rs`.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 605 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-parameter-slots/v8/SlotTypes.class");

/// Every member of the sample that declares a body, so a renamed fixture fails here instead of
/// covering less than this file claims.
const DECLARED: [(&[u8], &[u8]); 10] = [
    (b"<init>", b"()V"),
    (b"afterArray", b"([JI)I"),
    (b"afterNested", b"([[JI)I"),
    (b"afterGrid", b"([[DJ)J"),
    (b"betweenWide", b"(J[JI)I"),
    (b"arraysOnly", b"([J[DJ)J"),
    (b"echoArray", b"([J)[J"),
    (b"control", b"([II)I"),
    (b"instanceAfterArray", b"([JI)I"),
    (b"instanceWide", b"(J[JI)I"),
];

/// One member's declaration and the statement its body writes, both exactly as the presentation
/// spells them: the parameter the descriptor places after an array is named by the slot the body's
/// own numbering reads.
const AGREEMENT: [(&str, &str, &str); 9] = [
    (
        "afterArray",
        "public static int afterArray(long[] arg0, int arg1)",
        "return arg1;",
    ),
    (
        "afterNested",
        "public static int afterNested(long[][] arg0, int arg1)",
        "return arg1;",
    ),
    (
        "afterGrid",
        "public static long afterGrid(double[][] arg0, long arg1)",
        "return arg1;",
    ),
    (
        "betweenWide",
        "public static int betweenWide(long arg0, long[] arg2, int arg3)",
        "return arg3;",
    ),
    (
        "arraysOnly",
        "public static long arraysOnly(long[] arg0, double[] arg1, long arg2)",
        "return arg2;",
    ),
    (
        "echoArray",
        "public static long[] echoArray(long[] arg0)",
        "return arg0;",
    ),
    (
        "control",
        "public static int control(int[] arg0, int arg1)",
        "return arg1;",
    ),
    (
        "instanceAfterArray",
        "public int instanceAfterArray(long[] arg1, int arg2)",
        "return arg2;",
    ),
    (
        "instanceWide",
        "public int instanceWide(long arg1, long[] arg3, int arg4)",
        "return arg4;",
    ),
];

/// The driver both sides of the comparison are run with: one line per member, each printing the
/// value the member answers for arguments whose own text states which input answered.
const PROBE: &str = r#"import java.util.Arrays;

public class Probe {
    public static void main(String[] args) {
        SlotTypes instance = new SlotTypes();
        System.out.println("afterArray=" + SlotTypes.afterArray(new long[] {1L, 2L}, 42));
        System.out.println("afterNested=" + SlotTypes.afterNested(new long[][] {{1L}}, 43));
        System.out.println("afterGrid=" + SlotTypes.afterGrid(new double[][] {{1.5}}, 44L));
        System.out.println("betweenWide=" + SlotTypes.betweenWide(7L, new long[] {1L}, 45));
        System.out.println("arraysOnly=" + SlotTypes.arraysOnly(new long[] {1L}, new double[] {1.5}, 46L));
        System.out.println("echoArray=" + Arrays.toString(SlotTypes.echoArray(new long[] {47L})));
        System.out.println("control=" + SlotTypes.control(new int[] {1}, 48));
        System.out.println("instanceAfterArray=" + instance.instanceAfterArray(new long[] {1L}, 49));
        System.out.println("instanceWide=" + instance.instanceWide(7L, new long[] {1L}, 50));
        System.out.println("offset=" + instance.offset);
    }
}
"#;

/// The lines [`PROBE`] prints for the committed sample's own bytecode: the answers the presented
/// class must answer too, measured from the sample rather than assumed.
const PROBE_LINES: [&str; 10] = [
    "afterArray=42",
    "afterNested=43",
    "afterGrid=44",
    "betweenWide=45",
    "arraysOnly=46",
    "echoArray=[47]",
    "control=48",
    "instanceAfterArray=49",
    "instanceWide=50",
    "offset=3",
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// The environment every request in this file is made under (the P3 test files' shape).
fn environment(snapshot: &ArtifactSnapshot) -> EnvironmentRequest {
    EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: PhysicalScope::SnapshotAll,
        policy: EnvironmentPolicy::SingleClass,
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".to_owned()),
    }
}

/// One class-source presentation of one committed sample, under the entry point the CLI calls.
fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
        },
        environment: environment(snapshot),
    };
    match Engine::new()
        .class_source(slice::from_ref(snapshot), &request, &mut budget())
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition, got {other:?}"),
    }
}

/// The member of one class, by the raw name its class file declares.
fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's own text in the assembled source.
fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &member(report, name).text
}

/// A directory the comparison compiles and runs in, removed when it goes out of scope.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos();
        let sequence = AtomicU64::new(0).fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-slots-{label}-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create the comparison directory");
        Self(path)
    }

    fn write(&self, name: &str, bytes: &[u8]) {
        fs::write(self.0.join(name), bytes).expect("write into the comparison directory");
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

/// `javac --release 8 -g:none` over the sources named, with its messages in English, so a refusal
/// recorded here is the same string on every machine.
fn javac(dir: &Path, sources: &[&str]) {
    let output = Command::new("javac")
        .current_dir(dir)
        .args([
            "--release",
            "8",
            "-g:none",
            "-J-Duser.language=en",
            "-J-Duser.country=US",
            "-cp",
            ".",
            "-d",
            ".",
        ])
        .args(sources)
        .output()
        .expect("execute javac from PATH");
    assert!(
        output.status.success(),
        "javac refused {sources:?}:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// `java -cp . Probe`, whose stdout is the trace this file compares.
fn run_probe(dir: &Path) -> String {
    let output = Command::new("java")
        .current_dir(dir)
        .args(["-cp", ".", "Probe"])
        .output()
        .expect("execute java from PATH");
    assert!(
        output.status.success(),
        "the probe failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// One trace's lines, without the trailing newline the last line carries.
fn lines(trace: &str) -> Vec<&str> {
    trace.lines().collect()
}

#[test]
fn every_declared_member_is_covered_by_this_file() {
    let snapshot = open(SAMPLE);
    let report = class_source_of(&snapshot, "SlotTypes");
    assert_eq!(
        report.methods.len(),
        DECLARED.len(),
        "the sample's method table: {:?}",
        report
            .methods
            .iter()
            .map(|method| method.item.name.escaped())
            .collect::<Vec<_>>()
    );
    for (name, descriptor) in DECLARED {
        let method = member(&report, &String::from_utf8_lossy(name));
        assert_eq!(
            method.item.descriptor.raw().0,
            descriptor,
            "`{}` is the member this file classifies",
            String::from_utf8_lossy(name)
        );
    }
    assert!(
        report.text.contains("public int offset;"),
        "the field is presented beside the members:\n{}",
        report.text
    );
}

/// The acceptance the defect is about, at the text level: for every member, the name a parameter
/// has in the signature is the name the statement below it reads.
#[test]
fn the_declaration_and_the_body_name_one_slot_one_way() {
    let snapshot = open(SAMPLE);
    let report = class_source_of(&snapshot, "SlotTypes");
    for (name, declaration, statement) in AGREEMENT {
        let text = text_of(&report, name);
        assert!(
            text.contains(declaration),
            "`{name}` must declare `{declaration}`:\n{text}"
        );
        assert!(
            text.contains(statement),
            "`{name}` must read `{statement}`:\n{text}"
        );
    }
    // The control that must not move: an array whose element is one slot wide placed its successor
    // the same way before this change, and the instance receiver is still slot 0 — written as
    // `this` where the body reads it, and never as a parameter name.
    let constructor = text_of(&report, "<init>");
    assert!(
        constructor.contains("public SlotTypes()"),
        "the receiver is slot 0 of the constructor too:\n{constructor}"
    );
    assert!(
        constructor.contains("this.offset = 3;"),
        "the write through the receiver keeps its place:\n{constructor}"
    );
}

/// The acceptance at the behaviour level: the presentation's own text is compiled by the controlled
/// JDK and run under the same driver as the committed class, and both answer the same lines.
///
/// This is the check the defect cannot survive: a declaration that named `arg2` under a body that
/// read `arg1` is a text `javac` refuses (`cannot find symbol: variable arg1`), so the presentation
/// would not compile at all — and a text that compiled with *both* names shifted would still have to
/// answer what the original answers.
#[test]
#[ignore = "needs a JDK on PATH: it compiles the presentation's own text with `javac --release 8` \
            and runs it beside the committed class, so it is run explicitly \
            (`cargo test --test p3_parameter_slots --locked -- --ignored`)"]
fn the_presented_class_answers_what_the_committed_class_does() {
    let snapshot = open(SAMPLE);
    let report = class_source_of(&snapshot, "SlotTypes");

    // The committed class, driven by the probe.
    let original = TempDir::new("original");
    original.write("SlotTypes.class", SAMPLE);
    original.write("Probe.java", PROBE.as_bytes());
    javac(original.path(), &["Probe.java"]);
    let expected = run_probe(original.path());
    assert_eq!(
        lines(&expected),
        PROBE_LINES,
        "the committed sample answers these lines"
    );

    // The same probe over the presentation, compiled from the text the entry produced.
    let presented = TempDir::new("presented");
    presented.write("SlotTypes.java", report.text.as_bytes());
    presented.write("Probe.java", PROBE.as_bytes());
    javac(presented.path(), &["SlotTypes.java", "Probe.java"]);
    let answered = run_probe(presented.path());
    assert_eq!(
        lines(&answered),
        lines(&expected),
        "the presented class answers what the committed one does"
    );
}
