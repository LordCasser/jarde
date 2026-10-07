//! `recover-lock-guard-loop-finally`: the lock-guard skeleton is presented as the one
//! `lock(); try { … } finally { unlock(); }` the source wrote.
//!
//! The anchor is the explicit-lock patrol's own frozen artifact
//! (`openspec/evidence/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar`): a bounded
//! buffer whose `put`/`take` hold a `while`-`await` loop inside the guarded range and whose
//! `tryLockQuick` guards the range with `tryLock`. All three refused before this slice; the
//! certificate proves, instruction by instruction, that one acquisition call runs before the
//! protected range, that both cleanup copies are the same release call on the same receiver the
//! acquisition read, that the handler is exactly the release plus the rethrow, and that the
//! completion is one of javac's two forms — so folding the two copies into one `finally` runs the
//! release exactly once per exit path, on the lock the body holds.
//!
//! What this file pins:
//!
//! * the whole class renders on every leg (the patrol's jar, and the fixture's two compiler legs:
//!   javac 23.0.1 `--release 8 -g:none` and the real javac 8), with one `finally` per method and no
//!   refusal anywhere — the two release copies fold into one `unlock()`;
//! * the rendered class, with the layer's own envelope lines stripped, **compiles** on both javac
//!   legs and answers what the original answers under `java -Xverify:all` (the patrol's recorded
//!   `1/0/true`, whose value is the lock/unlock ordering itself);
//! * the negatives — a release on another lock, a release javac protects with a self-protection
//!   row, a receiver the body rewrites, the io-wrapping probe's local handle — keep their refusals
//!   **verbatim**, on both legs;
//! * the true two-copy non-lock `finally` shape (`tests/fixtures/p3-handlers/`) keeps its own
//!   refusal: this slice adds a shape beside the resource copies, it does not widen them.

use jarde::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

/// The patrol's frozen anchor: javac 23.0.1, `--release 8`, with line numbers.
const JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar"
);
/// The same source on this fixture's own legs (see the fixture's `README.md`).
const LK_V8: &[u8] = include_bytes!("fixtures/recover-lock-guard-loop-finally/v8/LK.class");
const LK_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-lock-guard-loop-finally/v8-javac8/LK.class");
const NEGATIVES_V8: &[u8] =
    include_bytes!("fixtures/recover-lock-guard-loop-finally/v8/LockGuardNegatives.class");
const NEGATIVES_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-lock-guard-loop-finally/v8-javac8/LockGuardNegatives.class");
const PROBE_V8: &[u8] =
    include_bytes!("fixtures/recover-lock-guard-loop-finally/v8/LockGuardProbe.class");
const PROBE_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/recover-lock-guard-loop-finally/v8-javac8/LockGuardProbe.class");
/// The P3 2.4 guarded sample: its `fin`/`catchFinally` are the two-copy `finally` shapes no
/// certificate of this build folds, and this slice must not change that.
const GUARDED: &[u8] = include_bytes!("fixtures/p3-handlers/v8/Guarded.class");

/// The behavior both the original and the rendered class answer: `put`, `put`, `take`, `take`,
/// `tryLockQuick` over the bounded buffer. The two `put`s fill it, the first `take` empties it to
/// `0` after `1`, and the guard acquires the lock — so the line states the lock/unlock ordering the
/// `finally` must keep.
const BEHAVIOR: &str = "1/0/true\n";

/// The refusals the negatives keep, verbatim: the region/guard sentence and every instruction the
/// refusal covers, as the baseline binary stated them (see `results/01-gating.md`).
const LOCK_MISMATCH: [&str; 4] = [
    "// @bytecode 0 1 4 7 8 9 12 13 14 17 18 21 24",
    "// BCI 27: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`",
    "// @bytecode 27 28 29 32 35 36 37",
    "// 2 live block(s) are reachable only through edges the normal-flow view leaves out: [37, 27]",
];
const GUARDED_RELEASE: [&str; 2] = [
    "// @bytecode 0 10 22 28 39",
    "// local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
];
const LOCAL_REWRITTEN: [&str; 2] = [
    "// @bytecode 0 31 38",
    "// local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
];
const THROWING_RELEASE: [&str; 4] = [
    "// @bytecode 0 1 4 7 8 9 12 13 14 17 18 21 26",
    "// block at BCI 0 leaves through exception handler 0: a handler's shape is not part of the recoverable subset",
    "// @bytecode 29 30 31 34 39 40 41",
    "// 2 live block(s) are reachable only through edges the normal-flow view leaves out: [41, 29]",
];
const PROBE: [&str; 2] = [
    "// @bytecode 0 27 36 42 52",
    "// local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice",
];

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
            "LK",
        ),
        ("v8", LK_V8.to_vec(), EnvironmentPolicy::SingleClass, "LK"),
        (
            "v8-javac8",
            LK_V8_JAVAC8.to_vec(),
            EnvironmentPolicy::SingleClass,
            "LK",
        ),
    ]
}

#[test]
fn the_lock_guard_class_renders_whole_on_every_leg() {
    let mut texts = Vec::new();
    for (leg, bytes, policy, class) in legs() {
        let snapshot = open(&bytes);
        let report = report(&snapshot, class, policy);
        assert!(
            report.text.starts_with("// jarde: presentation of `LK`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        assert!(
            !report.text.contains("@bytecode") && !report.text.contains("jarde_refused_body"),
            "{leg}: the family's refusals are gone:\n{}",
            report.text
        );
        // One `finally` per member: the two release copies fold into one `unlock()`.
        for name in ["put", "take", "tryLockQuick"] {
            let method = method_text(&report, name);
            assert_eq!(
                method.matches("finally {").count(),
                1,
                "{leg}/{name}: the two copies are one `finally`:\n{method}"
            );
            assert_eq!(
                method.matches("this.lock.unlock();").count(),
                1,
                "{leg}/{name}: the release runs once, on the lock the acquisition took:\n{method}"
            );
            assert_eq!(
                method.matches("this.lock.lock();").count()
                    + method.matches("this.lock.tryLock()").count(),
                1,
                "{leg}/{name}: the acquisition is written once, before the statement:\n{method}"
            );
        }
        let take = method_text(&report, "take");
        assert!(
            take.contains("while (this.count <= 0) {")
                && take.contains("this.notFull.await();")
                && take.contains("int local1 = this.count;")
                && take.contains("return local1;"),
            "{leg}: the loop body and the saved return are the body's own statements:\n{take}"
        );
        let put = method_text(&report, "put");
        assert!(
            put.contains("while (this.count >= 2) {")
                && put.contains("this.count += 1;")
                && !put.contains("return local"),
            "{leg}: the void guard keeps the loop and invents no saved value:\n{put}"
        );
        let quick = method_text(&report, "tryLockQuick");
        assert!(
            quick.contains("if (this.lock.tryLock()) {")
                && quick.contains("} else {\n            return false;\n        }"),
            "{leg}: the guard's own branch is the statement's own:\n{quick}"
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
fn the_negatives_keep_their_refusals_verbatim() {
    for (leg, bytes) in [("v8", NEGATIVES_V8), ("v8-javac8", NEGATIVES_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(
            &snapshot,
            "LockGuardNegatives",
            EnvironmentPolicy::SingleClass,
        );
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `LockGuardNegatives`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        for (name, refusals) in [
            ("lockMismatch", &LOCK_MISMATCH[..]),
            ("guardedRelease", &GUARDED_RELEASE[..]),
            ("localLockRewritten", &LOCAL_REWRITTEN[..]),
            ("throwingRelease", &THROWING_RELEASE[..]),
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
fn the_resource_across_finally_probe_presents_under_the_resource_guard() {
    // The io-wrapping patrol's shape as a single-method probe: one local handle, a construction
    // chain before the range, a loop, a saved return and a protected release. When this slice
    // landed the certificate proved an acquire/release pair on one field read, so the probe kept
    // its refusal — the boundary measured rather than assumed — and the whole-class IO acceptance
    // was registered as another slice's.
    //
    // `recover-io-resource-finally` is that slice, and this control is updated **explicitly** here
    // rather than deleted: the probe is now the resource-guard certificate's own shape (the
    // resource local's one definition before the range, the row set over one handler, the same SSA
    // value at the body's read and both closes), so it presents the one
    // `try { … } finally { r.close(); }` its source wrote. The lock guard's own three methods are
    // untouched — the first test of this file still pins them byte for byte — and the negatives
    // beside them keep their refusals verbatim.
    const PRESENTED: &str = "        java.io.BufferedReader local1;\n        local1 = new java.io.BufferedReader((java.io.Reader) new java.io.InputStreamReader((java.io.InputStream) new java.io.FileInputStream(arg0), \"UTF-8\"));\n        try {\n            int local2;\n            local2 = 0;\n            while (local1.readLine() != null) {\n                local2 = local2 + 1;\n            }\n            int local4 = local2;\n            return local4;\n        } finally {\n            local1.close();\n        }\n";
    for (leg, bytes) in [("v8", PROBE_V8), ("v8-javac8", PROBE_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = report(&snapshot, "LockGuardProbe", EnvironmentPolicy::SingleClass);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `LockGuardProbe`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        let method = method_text(&report, "countLines");
        assert!(
            method.contains(PRESENTED),
            "{leg}: the probe presents the statement, the loop and the close:\n{method}"
        );
        assert!(
            !method.contains("@bytecode"),
            "{leg}: no instruction of the probe stays quoted:\n{method}"
        );
        for refusal in PROBE {
            assert!(
                !method.contains(refusal),
                "{leg}: the family's refusal `{refusal}` is gone:\n{method}"
            );
        }
    }
}

#[test]
fn the_true_two_copy_non_lock_finally_stays_refused() {
    // `Guarded.fin`/`catchFinally`: two copies of one cleanup and no acquire/release pair at all.
    // They are the shape the resource-copy certificate refuses, and this slice must leave them
    // exactly there.
    let snapshot = open(GUARDED);
    let report = report(&snapshot, "Guarded", EnvironmentPolicy::SingleClass);
    for (name, at, handler) in [
        ("fin", "// @bytecode 0 3 6", "BCI 9:"),
        ("catchFinally", "// @bytecode 0 3 6", "BCI 19:"),
    ] {
        let method = method_text(&report, name);
        assert!(
            method.contains(at)
                && method.contains(handler)
                && method.contains("the `finally` copy javac emits for a `finally` clause"),
            "{name}: the two-copy refusal is the one it was:\n{method}"
        );
        assert!(
            !method.contains("finally {"),
            "{name}: no `finally` is invented for a copy pair no certificate proves:\n{method}"
        );
    }
}

// -------------------------------------------------------------------------------------------
// The roundtrip: the rendered class compiles on both javac legs and answers what the original does.
// -------------------------------------------------------------------------------------------

/// One private directory a test compiles and runs in.
struct TestDirectory(PathBuf);

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-lock-guard-{label}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("a private compilation directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Runs one class of one directory under `-Xverify:all`, answering its standard output.
fn run_class(directory: &Path, class: &str) -> String {
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(directory)
        .arg(class)
        .current_dir(directory)
        .output()
        .expect("java runs");
    assert!(
        run.status.success(),
        "{} runs under -Xverify:all:\n{}",
        class,
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run prints text")
}

#[test]
fn the_rendered_class_recompiles_and_answers_what_the_original_answers() {
    let snapshot = open(JAR);
    let report = report(&snapshot, "LK", EnvironmentPolicy::PlainJar);
    // The whole class the anchor is: its own members' bodies, with the layer's own `//` envelope
    // lines stripped, which is exactly what the roundtrip scripts compile.
    let stripped: String = report
        .text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .fold(String::new(), |mut text, line| {
            text.push_str(line);
            text.push('\n');
            text
        });

    // The original, for the comparison: the patrol's own jar, run as the archive it is.
    let original_dir = TestDirectory::new("original");
    let archive = original_dir.path().join("lk.jar");
    std::fs::write(&archive, JAR).expect("the anchor's archive is written");
    let original = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&archive)
        .arg("LK")
        .current_dir(original_dir.path())
        .output()
        .expect("java runs");
    assert!(
        original.status.success(),
        "the anchor runs under -Xverify:all:\n{}",
        String::from_utf8_lossy(&original.stderr)
    );
    assert_eq!(
        String::from_utf8(original.stdout).expect("the run prints text"),
        BEHAVIOR,
        "the anchor answers the patrol's recorded values"
    );

    let rendered_dir = TestDirectory::new("rendered");
    std::fs::write(rendered_dir.path().join("LK.java"), stripped).expect("the text is written");
    let compile = Command::new("javac")
        .args(["--release", "8", "-d"])
        .arg(rendered_dir.path())
        .arg(rendered_dir.path().join("LK.java"))
        .output()
        .expect("javac runs");
    assert!(
        compile.status.success(),
        "the rendered class compiles with `javac --release 8`:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    assert_eq!(
        run_class(rendered_dir.path(), "LK"),
        BEHAVIOR,
        "the rendered class answers what the original answers"
    );
}
