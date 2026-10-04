//! The shared-latch nested-loop family (CF-11, `recover-shared-latch-nested-loops`): the outer
//! `continue` edge and the nested loop's exit route onto the same back-edge destination, the
//! branch join is re-elected to the continuing arm's own statements, and each edge carries its
//! own semantics — the `continue` as a loop-continue region, the nested loop's exit through its
//! ordinary header-test exit. The refusal boundary stays where an exit leaves the loop past its
//! back edge, and the shapes the reading does not classify stay byte-identical.
//!
//! Frozen fixtures live in `tests/fixtures/shared-latch-nested-loops/` (provenance and SHA-256
//! records in its README). `S5.class` and `S3.class` are byte-identical copies of the patrol's
//! fixtures, so the pre-change refusals recorded there are the before side of every assertion
//! that asserts recovery here.

use jarde::*;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const S5: &[u8] = include_bytes!("fixtures/shared-latch-nested-loops/S5.class");
const S3: &[u8] = include_bytes!("fixtures/shared-latch-nested-loops/S3.class");
const THREE_LEVEL: &[u8] = include_bytes!("fixtures/shared-latch-nested-loops/ThreeLevel.class");
const EXIT_DIVERGES: &[u8] =
    include_bytes!("fixtures/shared-latch-nested-loops/ExitDiverges.class");
const LABELED_BREAK_OUTER: &[u8] =
    include_bytes!("fixtures/shared-latch-nested-loops/LabeledBreakOuter.class");
const OVERLAP: &[u8] = include_bytes!("fixtures/shared-latch-nested-loops/Overlap.class");

/// The recovered double loop spells the `continue` and the counted inner header clause; the
/// labeled twin recovers the same way.
const RECOVERED_INNER_FOR: &str = "for (local3 = 0; local3 < local2; local3 = local3 + 1) {";

/// `outerContinueOnly` and `innerOnly` never reach the shared-latch reading (the single-continue
/// shape keeps its post-dominator join through the elided-goto channel, and the no-jump shape
/// has no branch to re-elect). Their rendered text is pinned here exactly as the pre-change
/// build wrote it: any difference is the reading over-reaching, not progress.
const UNTOUCHED_SINGLE_CONTINUE: &str = "\
    public static int outerContinueOnly(int arg0) {
        // @method outerContinueOnly(I)I
        // @declaration a static method of `S5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            if (local2 % 2 == 0) {
            } else {
                local1 = local1 + local2;
            }
        }
        return local1;
    }
";

const UNTOUCHED_NO_JUMP: &str = "\
    public static int innerOnly(int arg0) {
        // @method innerOnly(I)I
        // @declaration a static method of `S5`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        int local2;
        local1 = 0;
        int local3;
        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {
            for (local3 = 0; local3 < local2; local3 = local3 + 1) {
                local1 = local1 + local3;
            }
        }
        return local1;
    }
";

fn class_source(class: &[u8], name: &str) -> String {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).expect("task defaults provide a bounded budget");
    let snapshot = engine
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("the compiled Java 8 class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match engine
        .class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
        .expect("the class-source request is valid")
    {
        OperationOutcome::Performed(report) => report.text,
        OperationOutcome::Ambiguous(candidates) => {
            panic!("expected one {name}, found {}", candidates.candidates.len())
        }
        OperationOutcome::Incomplete(candidates) => panic!(
            "{name} selection stopped with {} candidates",
            candidates.candidates.len()
        ),
    }
}

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let unique = NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("{label}-{unique}"));
        std::fs::create_dir_all(&path).expect("the scratch directory is created");
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The recovered text recompiles under `javac --release 8`, verifies under `-Xverify:all`, and
/// prints what the original class prints. The same run the acceptance evidence performs with a
/// real javac 8, narrowed here to what guards the frozen fixture.
fn recompile_and_run(label: &str, text: &str, class_name: &str) -> String {
    let temp = TestDirectory::new(label);
    std::fs::write(temp.path().join(format!("{class_name}.java")), text)
        .expect("the recovered text is written");
    let compiled = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg(format!("{class_name}.java"))
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "the recovered {class_name} recompiles under --release 8:\n{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp", "."])
        .arg(class_name)
        .current_dir(temp.path())
        .output()
        .expect("java runs");
    assert!(
        run.status.success(),
        "the recompiled {class_name} verifies and runs:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run prints text")
}

#[test]
fn shared_latch_double_loop_recovers_and_spells_the_continue() {
    let text = class_source(S5, "S5");
    assert!(
        !text.contains("jarde: not recovered"),
        "every S5 member recovers after the slice:\n{text}"
    );
    let inner = text
        .split("public static int outerContinueInner")
        .nth(1)
        .expect("outerContinueInner is presented");
    assert!(
        inner.contains("continue;"),
        "the continue edge is attributed as the loop continue:\n{text}"
    );
    assert!(
        inner.contains(RECOVERED_INNER_FOR),
        "the nested counted loop recovers with its header clause:\n{text}"
    );
}

#[test]
fn labeled_continue_outer_form_recovers_the_same_way() {
    let text = class_source(S5, "S5");
    let labeled = text
        .split("public static int labeledOuter")
        .nth(1)
        .expect("labeledOuter is presented");
    assert!(
        labeled.contains("continue;") && labeled.contains(RECOVERED_INNER_FOR),
        "the labeled twin recovers with the same double loop and continue:\n{text}"
    );
}

#[test]
fn untouched_single_continue_and_no_jump_shapes_stay_byte_identical() {
    let text = class_source(S5, "S5");
    assert!(
        text.contains(UNTOUCHED_SINGLE_CONTINUE),
        "the single-continue shape renders exactly as before the slice:\n{text}"
    );
    assert!(
        text.contains(UNTOUCHED_NO_JUMP),
        "the no-jump nested shape renders exactly as before the slice:\n{text}"
    );
}

#[test]
fn business_form_with_inner_break_recovers() {
    let text = class_source(S3, "S3");
    assert!(
        !text.contains("jarde: not recovered"),
        "every S3 member recovers after the slice:\n{text}"
    );
    let nested_break = text
        .split("public static java.util.List nestedBreak")
        .nth(1)
        .expect("nestedBreak is presented");
    assert!(
        nested_break.contains("continue;") && nested_break.contains("break;"),
        "the business form spells its continue and its inner break:\n{text}"
    );
    assert!(
        nested_break.contains("while (local5.hasNext())"),
        "the inner iterator loop recovers inside the outer one:\n{text}"
    );
}

#[test]
fn business_form_behavior_matches_the_original_class() {
    let text = class_source(S3, "S3");
    let text = text
        .split("// jarde: presentation of `S3`")
        .nth(1)
        .map(|rest| format!("// jarde: presentation of `S3`{rest}"))
        .expect("the render keeps its header");
    // The render claims no imports; the `java.util` references are fully spelled, so the text
    // compiles as it stands.
    let run = recompile_and_run("cf11-s3", &text, "S3");
    assert_eq!(
        run, "[px:3]\n[px:3, px:-1, px:5]\n[a!, b!]\n",
        "the recompiled S3 prints the original behavior"
    );
}

#[test]
fn three_level_composition_recovers_and_behaves() {
    let text = class_source(THREE_LEVEL, "ThreeLevel");
    assert!(
        !text.contains("jarde: not recovered"),
        "the three-level composition recovers:\n{text}"
    );
    let run = recompile_and_run("cf11-three-level", &text, "ThreeLevel");
    assert_eq!(
        run, "24\n",
        "the recompiled ThreeLevel prints the original behavior"
    );
}

#[test]
fn exits_that_leave_the_loop_past_its_back_edge_stay_refused() {
    let escaped = class_source(LABELED_BREAK_OUTER, "LabeledBreakOuter");
    assert!(
        escaped.contains("jarde: not recovered"),
        "the labeled break outer escape stays refused:\n{escaped}"
    );
    let divergent = class_source(EXIT_DIVERGES, "ExitDiverges");
    assert!(
        divergent.contains("jarde: not recovered"),
        "the inner exit that does not route onto the back edge stays refused:\n{divergent}"
    );
}

#[test]
fn overlap_with_inner_two_jump_body_stays_refused() {
    // Registered residual: the outer reading fires, the inner break+continue body belongs to
    // the double-jump slice's closed domain and stays refused here. This test pins the
    // registration so a silent change in either direction is caught.
    let text = class_source(OVERLAP, "Overlap");
    assert!(
        text.contains("jarde: not recovered"),
        "the overlap form stays refused until the two-jump domain reopens it:\n{text}"
    );
}
