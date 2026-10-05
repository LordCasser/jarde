//! `preserve-monitor-exit-evaluation-order`: the `return` a nested `synchronized` body ends in.
//!
//! `NL`'s `probe` is the discriminant the sync-return-timing patrol froze
//! (`openspec/evidence/java-syntax-2026-10-05/sync-return-timing-patrol/`):
//!
//! ```text
//! static java.lang.String probe(java.lang.Object o) {
//!     synchronized (LOCK) {
//!         synchronized (NL.class) { return "n" + o; }
//!     }
//! }
//! ```
//!
//! The concatenation's evaluation (BCIs 11–27) precedes the inner `monitorexit` (BCI 31); the value
//! is returned across both exits. A presentation that writes the inner block **empty** and the
//! `return` after it evaluates the concatenation with the inner lock already released: the text
//! still compiles, and `Box.toString`'s `Thread.holdsLock(NL.class)` prints `nN` where the class
//! prints `nY` — the violation this change closes, with `nY` as its anchor.
//!
//! What the presented text pins, on **both** compiler legs (javac 23.0.1 `--release 8` and real
//! javac 8, the same source both times):
//!
//! * the `return` is written **inside** the inner braces, after the inner header and before the
//!   inner statement's own closing brace, so the evaluation the `return` consumes stays before the
//!   inner `monitorexit` the pair proves;
//! * the inner block is not empty, and no local or second read is invented for the value;
//! * `SR`'s single-monitor members (`retInside` inside the braces, `localAcross` across them) and
//!   its refused `voidBody` render exactly as they did before the inner-return reading existed:
//!   the change moves the `return` of a **proved** inner pair and nothing else;
//! * `GuardReturnEffects`'s proven nested pair — whose two other members keep their own refusal —
//!   moves its `return` into the inner braces for the same reason, and its `effect` member is
//!   untouched.
//!
//! The replay that needs a JDK (`cargo test -- --ignored`) strips the comment lines from the
//! class-source text, compiles it with `javac --release 8`, and runs it with `-Xverify:all`: the
//! fixture's own class prints `nY`, and the text must print `nY` too. The pre-change text prints
//! `nN`, which is what makes the anchor discriminating (the change's `verification.md` records that
//! run).

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
// The fixtures: the patrol's own sources, compiled by both javac legs.
// -------------------------------------------------------------------------------------------

/// `NL` — javac 23.0.1, `--release 8` (the patrol's own bytes: the same source, the same command).
const NL: &[u8] = include_bytes!("fixtures/preserve-monitor-exit-evaluation-order/v8/NL.class");
/// `NL$Box` — the companion the discriminant's `toString` reads.
const NL_BOX: &[u8] =
    include_bytes!("fixtures/preserve-monitor-exit-evaluation-order/v8/NL$Box.class");
/// `SR` — the same leg: `retInside`, `localAcross`, `voidBody` and `nestedLock`.
const SR: &[u8] = include_bytes!("fixtures/preserve-monitor-exit-evaluation-order/v8/SR.class");

/// The real javac 8 leg (Corretto 1.8.0_432): the same sources, the layout the patrol's own
/// `javap` reading was taken from.
const NL_JAVAC8: &[u8] =
    include_bytes!("fixtures/preserve-monitor-exit-evaluation-order/v8-javac8/NL.class");
const NL_BOX_JAVAC8: &[u8] =
    include_bytes!("fixtures/preserve-monitor-exit-evaluation-order/v8-javac8/NL$Box.class");
const SR_JAVAC8: &[u8] =
    include_bytes!("fixtures/preserve-monitor-exit-evaluation-order/v8-javac8/SR.class");

/// The committed nested-pair boundary: `nested`'s returned value is a body local, `effect` keeps the
/// bytecode quote its independent call forces (the fixture of `preserve-guarded-return-expression`).
const GUARD_RETURN_EFFECTS: &[u8] =
    include_bytes!("fixtures/preserve-guarded-return-expression/v8/GuardReturnEffects.class");

/// One compiler leg: the label a failure names, and the bytes that leg produced.
struct Leg {
    label: &'static str,
    nl: &'static [u8],
    nl_box: &'static [u8],
    sr: &'static [u8],
}

const LEGS: &[Leg] = &[
    Leg {
        label: "javac 23.0.1 --release 8",
        nl: NL,
        nl_box: NL_BOX,
        sr: SR,
    },
    Leg {
        label: "Corretto 1.8.0_432 (real javac 8)",
        nl: NL_JAVAC8,
        nl_box: NL_BOX_JAVAC8,
        sr: SR_JAVAC8,
    },
];

// -------------------------------------------------------------------------------------------
// The presented texts, pinned whole (the p3 surface tests' own convention).
// -------------------------------------------------------------------------------------------

/// `NL.probe` on both legs: the `return`, with the value it consumes, **inside** the inner braces.
const NL_PROBE: &str = "    static java.lang.String probe(java.lang.Object arg0) {\n        // @method probe(Ljava/lang/Object;)Ljava/lang/String;\n        // @declaration a static method of `NL`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        synchronized (NL.LOCK) {\n            synchronized (NL.class) {\n                return \"n\" + arg0;\n            }\n        }\n    }\n";

/// `NL$Box` as the class-source text carries it: the companion the discriminant's `toString` is.
const NL_BOX_MEMBER: &str = "    static class Box extends java.lang.Object {\n        Box() {\n            // @method <init>()V\n            // @declaration a constructor of `NL$Box`, member flags 0x0000\n            // recovered from bytecode; presentation is not claimed to compile\n            super();\n            return;\n        }\n\n        public java.lang.String toString() {\n            // @method toString()Ljava/lang/String;\n            // @declaration an instance method of `NL$Box`, member flags 0x0001\n            // recovered from bytecode; presentation is not claimed to compile\n            return java.lang.Thread.holdsLock((java.lang.Object) NL.class) ? \"Y\" : \"N\";\n        }\n    }\n";

/// `SR.retInside`: the single-monitor shape the patrol found healthy — unchanged by this change.
const SR_RET_INSIDE: &str = "    static int retInside(int arg0) {\n        // @method retInside(I)I\n        // @declaration a static method of `SR`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        synchronized (SR.LOCK) {\n            return arg0 * 2;\n        }\n    }\n";

/// `SR.localAcross`: a local written inside the braces and read outside them — unchanged.
const SR_LOCAL_ACROSS: &str = "    static int localAcross(int arg0) {\n        // @method localAcross(I)I\n        // @declaration a static method of `SR`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local1;\n        synchronized (SR.LOCK) {\n            local1 = arg0 + 1;\n        }\n        return local1;\n    }\n";

/// `SR.voidBody`: the soundness slice's fail-closed refusal — the reserved marker, unchanged here.
const SR_VOID_BODY: &str = "    static void voidBody(int arg0) {\n        // jarde: not recovered: the recovery run for `voidBody(I)V` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method voidBody(I)V\n        // @declaration a static method of `SR`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 0 10 20 25 30\n        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice\n        jarde_refused_body();\n    }\n";

/// `SR.nestedLock`: the same shape as `NL.probe` over `SR`'s own locks — the second anchor of the
/// change, moved inside the inner braces.
const SR_NESTED_LOCK: &str = "    static java.lang.String nestedLock(int arg0) {\n        // @method nestedLock(I)Ljava/lang/String;\n        // @declaration a static method of `SR`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        synchronized (SR.LOCK) {\n            synchronized (SR.class) {\n                return \"n\" + arg0;\n            }\n        }\n    }\n";

// -------------------------------------------------------------------------------------------
// The request path (the surface tests' shape): one snapshot, one class, the whole scope.
// -------------------------------------------------------------------------------------------

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a ZIP")
}

/// One class-source presentation over the snapshot's own root container: the policy `--policy
/// plain-jar` declares, which is what makes the companion class a member of the class it belongs to.
fn class_source_of(
    snapshot: &ArtifactSnapshot,
    name: &str,
    policy: EnvironmentPolicy,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one committed sample answers one definition, got {} candidate(s)",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one committed sample answers one definition, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

/// One member's own record in the assembled source.
fn method_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's recovery report, for the planes the text alone cannot state.
fn recovered<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &method_of(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` is a recovered member: {other:?}"),
    }
}

/// The one ZIP the class-source request reads, built from the committed class files: the companion
/// is an entry of the same container, which is what makes it a member of the class it belongs to.
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

fn fixture(leg: &Leg) -> ArtifactSnapshot {
    open(&zip_of(&[
        (b"NL.class", leg.nl),
        (b"NL$Box.class", leg.nl_box),
        (b"SR.class", leg.sr),
    ]))
}

// -------------------------------------------------------------------------------------------
// The anchors.
// -------------------------------------------------------------------------------------------

#[test]
fn the_nested_return_is_written_inside_the_inner_braces() {
    for leg in LEGS {
        let snapshot = fixture(leg);
        let report = class_source_of(&snapshot, "NL", EnvironmentPolicy::PlainJar);
        let probe = method_of(&report, "probe");
        assert_eq!(probe.text, NL_PROBE, "{}: `NL.probe`", leg.label);
        assert!(
            !probe
                .text
                .contains("synchronized (NL.class) {\n            }\n"),
            "{}: the inner block is not left empty with the return moved after it:\n{}",
            leg.label,
            probe.text
        );
        // The statements the proof read are anchors of the text that took their place, the inner
        // exit among them: which instruction the returned value came from stays answerable.
        let anchored: std::collections::BTreeSet<u32> = recovered(&report, "probe")
            .source_map
            .segments()
            .iter()
            .flat_map(|segment| segment.origin().bcis())
            .collect();
        for bci in [5, 10, 11, 31, 33, 34] {
            assert!(
                anchored.contains(&bci),
                "{}: BCI {bci} is an anchor of the statement: {anchored:?}",
                leg.label
            );
        }
        // The companion the discriminant reads is present in the same presentation, and its own
        // reading is unchanged: the whole class text is the member texts plus the envelope.
        assert!(
            report.text.contains(NL_BOX_MEMBER),
            "{}: the companion member is presented unchanged:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn the_single_monitor_members_and_the_refused_void_body_are_unchanged() {
    for leg in LEGS {
        let snapshot = fixture(leg);
        let report = class_source_of(&snapshot, "SR", EnvironmentPolicy::PlainJar);
        assert_eq!(
            method_of(&report, "retInside").text,
            SR_RET_INSIDE,
            "{}: `SR.retInside`",
            leg.label
        );
        assert_eq!(
            method_of(&report, "localAcross").text,
            SR_LOCAL_ACROSS,
            "{}: `SR.localAcross`",
            leg.label
        );
        assert_eq!(
            method_of(&report, "voidBody").text,
            SR_VOID_BODY,
            "{}: `SR.voidBody`",
            leg.label
        );
        assert_eq!(
            method_of(&report, "nestedLock").text,
            SR_NESTED_LOCK,
            "{}: `SR.nestedLock`",
            leg.label
        );
        // The refused void body is an explanation and not a statement: the marker stands in for the
        // body javac would otherwise compile as a silent no-op.
        assert_eq!(
            recovered(&report, "voidBody").content,
            RecoveryContent::ExplanationOnly,
            "{}: the refused body presents no statement",
            leg.label
        );
        assert!(
            !method_of(&report, "voidBody").markers.is_empty(),
            "{}: the refused body carries its marker",
            leg.label
        );
        // The single-monitor members are statements, not explanations: this change moves no
        // single-monitor rendering at all.
        assert_eq!(
            recovered(&report, "retInside").content,
            RecoveryContent::ContainsStatements,
            "{}: `retInside` is a proved statement",
            leg.label
        );
        assert_eq!(
            recovered(&report, "localAcross").content,
            RecoveryContent::ContainsStatements,
            "{}: `localAcross` is a proved statement",
            leg.label
        );
    }
}

#[test]
fn a_proved_nested_pairs_local_return_moves_inside_and_its_sibling_keeps_its_refusal() {
    let snapshot = open(GUARD_RETURN_EFFECTS);
    let report = class_source_of(
        &snapshot,
        "GuardReturnEffects",
        EnvironmentPolicy::SingleClass,
    );
    let nested = method_of(&report, "nested").text.clone();
    let inner_header = "synchronized (GuardReturnEffects.LOCK) {\n";
    let header = nested
        .find(inner_header)
        .expect("the proven inner pair is written");
    let returns = nested
        .find("return local4;")
        .expect("the return is written");
    let closing = returns
        + nested[returns..]
            .find("\n            }")
            .expect("the inner statement's own closing brace follows the return");
    assert!(
        header < returns && returns < closing,
        "the local the return names is read while the inner monitor is held:\n{nested}"
    );
    assert!(
        !nested.contains("@bytecode"),
        "the proven nested pair quotes nothing:\n{nested}"
    );
    // The single-monitor sibling whose body holds an independent call keeps the refusal: the new
    // reading is the nested pair's, and it is not borrowed by the one-monitor shape.
    let effect = method_of(&report, "effect").text.clone();
    assert!(
        effect.contains("@bytecode 0 21 24 28") && !effect.contains("synchronized (this)"),
        "the independent call inside a single monitor is still refused:\n{effect}"
    );
}

// -------------------------------------------------------------------------------------------
// The replay: the presented text, compiled and run (needs a JDK on PATH).
// -------------------------------------------------------------------------------------------

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
            "jarde-monitor-exit-{label}-{}-{nonce}-{sequence}",
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

/// The class-source text as the compile-and-run protocol reads it: every `//` comment line dropped,
/// which is exactly the strip the patrol's own `verify-wrong-NL.java` was made by.
fn stripped(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| format!("{line}\n"))
        .collect()
}

#[test]
#[ignore = "needs a JDK on PATH: it strips the class-source comment lines, compiles the text with \
            `javac --release 8` and runs it with `-Xverify:all` (see the module doc)"]
fn the_rendered_text_runs_with_the_inner_lock_held() {
    let temp = TempDir::new("NL");
    let rendered = temp.path().join("rendered");
    let original = temp.path().join("original");
    fs::create_dir_all(&rendered).expect("create the rendered directory");
    fs::create_dir_all(&original).expect("create the original directory");

    let snapshot = open(&zip_of(&[(b"NL.class", NL), (b"NL$Box.class", NL_BOX)]));
    let source = stripped(&class_source_of(&snapshot, "NL", EnvironmentPolicy::PlainJar).text);
    fs::write(rendered.join("NL.java"), &source).expect("write the presented text");

    let compile = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options", "-d"])
        .arg(&rendered)
        .arg(rendered.join("NL.java"))
        .output()
        .expect("the installed JDK provides javac");
    assert!(
        compile.status.success(),
        "javac rejected the presented text:\n{}\n{source}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&rendered)
        .arg("NL")
        .output()
        .expect("the installed JDK provides java");
    assert!(
        run.status.success(),
        "the presented text's program failed:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "nY\n",
        "the value is evaluated while the inner monitor is held:\n{source}"
    );

    // The same reading from the class the fixture was compiled from: the presented text answers what
    // the bytecode answers, on the artifact's own terms rather than on a remembered string.
    fs::write(original.join("NL.class"), NL).expect("write the fixture class");
    fs::write(original.join("NL$Box.class"), NL_BOX).expect("write the fixture's companion class");
    let own = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&original)
        .arg("NL")
        .output()
        .expect("the installed JDK provides java");
    assert!(own.status.success(), "the fixture's own class runs");
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&own.stdout),
        "the presented text answers what the class answers"
    );

    // `SR`'s text is deliberately not a compilation unit: `voidBody`'s refusal is written as the
    // reserved marker, so a stripped text can never present that body as a silent no-op.
    let sr_snapshot = open(&zip_of(&[(b"SR.class", SR)]));
    let sr_source =
        stripped(&class_source_of(&sr_snapshot, "SR", EnvironmentPolicy::PlainJar).text);
    let sr_dir = temp.path().join("sr");
    fs::create_dir_all(&sr_dir).expect("create the SR directory");
    fs::write(sr_dir.join("SR.java"), &sr_source).expect("write the SR text");
    let refused = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options", "-d"])
        .arg(&sr_dir)
        .arg(sr_dir.join("SR.java"))
        .output()
        .expect("the installed JDK provides javac");
    assert!(
        !refused.status.success(),
        "the stripped SR text must not compile: the refused body's marker is a reserved symbol"
    );
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("jarde_refused_body"),
        "the refusal names the reserved marker:\n{}",
        String::from_utf8_lossy(&refused.stderr)
    );
}
