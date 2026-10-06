//! `recover-loop-else-if-early-returns` in one frozen target: the binary-search patrol's `BS`/`CB`
//! anchors, its `CB2` exit-consuming twin, this change's own all-recovering anchor `LR2`, and the
//! `LB` negatives — on **both** compiler legs (javac 23.0.1 `--release 8` and real javac 8,
//! Corretto 1.8.0_432, from the same sources).
//!
//! `BS.bsearch` and `CB.loopElseIfRet` are the shape the patrol refused whole-method: a `while`
//! loop whose body is an `else if` ladder with an early-return arm. The ladder's arms regroup on the
//! loop's own latch block, and because the returning arm never passes it the branch has no
//! post-dominator — so the walk kept the loop header as both arms' boundary, the first arm claimed
//! the latch and the second re-entered it, and `jre_region_ownership_overlap` quoted the method
//! ("canonical block at BCI 56 … has more than one owner in the completed Region tree"). The change
//! states the ladder's join from the arms' own routes ([`region::Walker::ladder_join`]), so the
//! ladder is one `if`/`else if` tree whose early-return arm is a terminating leaf.
//!
//! The patrol's three controls stand beside it byte for byte (`loopElseIfNoRet`, `loopIfElseRet`,
//! `noLoopElseIfRet`), and the shapes the change states as out of its scope keep their refusal
//! verbatim: `LB.doubleLadder` and `LB.tryLadder` keep the same canonical-overlap sentence,
//! `LB.switchInArm` (a switch inside a ladder arm) and `LB.firstArmRet` (the early return in the
//! ladder's *first* arm) keep their cross-quote sentence. `LB.forLadder` and `LB.switchLadder` are
//! recorded as what the reading actually reaches: the ladder's own arms regroup on the latch in
//! both, so both recover, and their texts are pinned here.
//!
//! The ignored replay strips the presentations the way the patrol's own stripped sources were made
//! (comment lines dropped), compiles each anchor with the installed `javac --release 8` and, when a
//! real javac 8 is present, with that one too, runs both under `-Xverify:all` and compares every
//! answer with the fixture's own class files. `LR2` — this change's own anchor, the same bsearch
//! shape over an array — is replayed as a **whole class**, which compiles because every member of
//! it recovers; the patrol's `BS`/`CB`/`CB2`/`LB` keep a refused `main` (the reserved
//! `jarde_refused_body()` symbol a refused `void` body writes), so their replay compiles the
//! recovered **method's** text in a unit with the fixture's own call sequence and compares the
//! answer with the fixture class's own run.

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
// The fixtures: the patrol's anchors and controls, this change's anchor, and its negatives.
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names and the class files it produced.
struct Leg {
    label: &'static str,
    files: &'static [(&'static str, &'static [u8])],
}

impl Leg {
    /// The class files of one fixture family, in container order.
    fn family(&self, class: &str) -> Vec<(String, &'static [u8])> {
        let own = format!("{class}.class");
        let nested = format!("{class}$");
        let files: Vec<(String, &'static [u8])> = self
            .files
            .iter()
            .filter(|(name, _)| {
                let name: &str = name;
                name == own || name.starts_with(&nested)
            })
            .map(|(name, bytes)| ((*name).to_owned(), *bytes))
            .collect();
        assert_eq!(
            files.first().map(|(name, _)| name.as_str()),
            Some(own.as_str()),
            "the fixture family of `{class}` is committed"
        );
        files
    }

    /// One fixture family's container.
    fn fixture(&self, class: &str) -> Vec<u8> {
        let own = format!("{class}.class");
        let nested = format!("{class}$");
        let entries: Vec<(&[u8], &[u8])> = self
            .files
            .iter()
            .filter(|(name, _)| {
                let name: &str = name;
                name == own || name.starts_with(&nested)
            })
            .map(|(name, bytes)| (name.as_bytes(), *bytes))
            .collect();
        assert_eq!(
            entries.first().map(|(name, _)| *name),
            Some(own.as_bytes()),
            "the fixture family of `{class}` is committed"
        );
        zip_of(&entries)
    }
}

/// javac 23.0.1, `javac --release 8 -Xlint:-options -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "BS.class",
        include_bytes!("fixtures/recover-loop-else-if-early-returns/v8/BS.class"),
    ),
    (
        "CB.class",
        include_bytes!("fixtures/recover-loop-else-if-early-returns/v8/CB.class"),
    ),
    (
        "CB2.class",
        include_bytes!("fixtures/recover-loop-else-if-early-returns/v8/CB2.class"),
    ),
    (
        "LB.class",
        include_bytes!("fixtures/recover-loop-else-if-early-returns/v8/LB.class"),
    ),
    (
        "LR2.class",
        include_bytes!("fixtures/recover-loop-else-if-early-returns/v8/LR2.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "BS.class",
        include_bytes!("fixtures/recover-loop-else-if-early-returns/v8-javac8/BS.class"),
    ),
    (
        "CB.class",
        include_bytes!("fixtures/recover-loop-else-if-early-returns/v8-javac8/CB.class"),
    ),
    (
        "CB2.class",
        include_bytes!("fixtures/recover-loop-else-if-early-returns/v8-javac8/CB2.class"),
    ),
    (
        "LB.class",
        include_bytes!("fixtures/recover-loop-else-if-early-returns/v8-javac8/LB.class"),
    ),
    (
        "LR2.class",
        include_bytes!("fixtures/recover-loop-else-if-early-returns/v8-javac8/LR2.class"),
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

// -------------------------------------------------------------------------------------------
// The class-source surface.
// -------------------------------------------------------------------------------------------

/// The ordinary request plus the one optional category the member fold reads.
fn evidence() -> RecoveryEvidenceRequest {
    RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap)
}

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
            &evidence(),
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition: {other:?}"),
    }
}

/// One class's presentation, with jarde's own self-header asserted **before** anything is counted
/// in it: a render of nothing is not a render, and counting refusals in one would be a false zero.
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

/// One member's own record in the assembled source.
fn method_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// One member's text, for the assertions that compare a whole presentation.
fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &method_of(report, name).text
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

// -------------------------------------------------------------------------------------------
// The presented texts, pinned whole (the p3 surface tests' own convention).
// -------------------------------------------------------------------------------------------

/// The refusal sentence the two canonical-overlap negatives keep, verbatim.
const OVERLAP: &str =
    "has more than one owner in the completed Region tree; the whole method is quoted";

/// The refusal sentence the two cross-quote negatives keep, verbatim.
const CROSS_QUOTE: &str = "local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice";

/// `BS.bsearch`: the patrol's anchor, recovered as one `if`/`else if` tree whose early-return arm is
/// a terminating leaf. Both legs answer this text byte for byte.
const BS_BSEARCH: &str = "    static int bsearch(int[] arg0, int arg1) {\n        // @method bsearch([II)I\n        // @declaration a static method of `BS`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local2;\n        int local3;\n        local2 = 0;\n        local3 = arg0.length - 1;\n        while (local2 <= local3) {\n            int local4 = local2 + local3 >>> 1;\n            int local5 = arg0[local4];\n            if (local5 < arg1) {\n                local2 = local4 + 1;\n            } else if (local5 > arg1) {\n                local3 = local4 - 1;\n    } else {\n                return local4;\n    }\n        }\n        return -(local2 + 1);\n    }\n";

/// `CB.loopElseIfRet`: the patrol's second anchor — the same shape with a fixed exit value.
const CB_LOOP_ELSE_IF_RET: &str = "    static int loopElseIfRet(int[] arg0, int arg1) {\n        // @method loopElseIfRet([II)I\n        // @declaration a static method of `CB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local2;\n        int local3;\n        local2 = 0;\n        local3 = arg0.length - 1;\n        while (local2 <= local3) {\n            int local4 = local2 + local3 >>> 1;\n            int local5 = arg0[local4];\n            if (local5 < arg1) {\n                local2 = local4 + 1;\n            } else if (local5 > arg1) {\n                local3 = local4 - 1;\n    } else {\n                return local4;\n    }\n        }\n        return -1;\n    }\n";

/// The three controls of `CB`, pinned byte for byte: the change's zero-regression surface.
const CB_LOOP_ELSE_IF_NO_RET: &str = "    static int loopElseIfNoRet(int[] arg0) {\n        // @method loopElseIfNoRet([I)I\n        // @declaration a static method of `CB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local1;\n        int local2;\n        local1 = 0;\n        local2 = 0;\n        while (local1 < arg0.length) {\n            int local3 = arg0[local1];\n            if (local3 < 0) {\n                local1 = local1 + 1;\n            } else if (local3 > 0) {\n                local2 = local2 + local3;\n                local1 = local1 + 1;\n    } else {\n                local1 = local1 + 1;\n    }\n        }\n        return local2;\n    }\n";

const CB_LOOP_IF_ELSE_RET: &str = "    static int loopIfElseRet(int[] arg0, int arg1) {\n        // @method loopIfElseRet([II)I\n        // @declaration a static method of `CB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local2;\n        local2 = 0;\n        while (local2 < arg0.length) {\n            if (arg0[local2] == arg1) {\n                return local2;\n            } else {\n                local2 = local2 + 2;\n            }\n        }\n        return -1;\n    }\n";

const CB_NO_LOOP_ELSE_IF_RET: &str = "    static int noLoopElseIfRet(int arg0) {\n        // @method noLoopElseIfRet(I)I\n        // @declaration a static method of `CB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        if (arg0 < 0) {\n            return -1;\n        } else if (arg0 > 100) {\n            return 100;\n    } else {\n            return arg0;\n    }\n    }\n";

/// `CB2.exitVal`: the same ladder shape whose exit consumes the loop variable — the patrol's third
/// recovery claim, refused at HEAD and recovered by this change.
const CB2_EXIT_VAL: &str = "    static int exitVal(int[] arg0, int arg1) {\n        // @method exitVal([II)I\n        // @declaration a static method of `CB2`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local2;\n        int local3;\n        local2 = 0;\n        local3 = arg0.length - 1;\n        while (local2 <= local3) {\n            int local4 = local2 + local3 >>> 1;\n            int local5 = arg0[local4];\n            if (local5 < arg1) {\n                local2 = local4 + 1;\n            } else if (local5 > arg1) {\n                local3 = local4 - 1;\n    } else {\n                return local4;\n    }\n        }\n        return -(local2 + 1);\n    }\n";

/// `CB2.exitVal2`: the same exit consumption without a ladder — unchanged, pinned as a control.
const CB2_EXIT_VAL2: &str = "    static int exitVal2(int[] arg0) {\n        // @method exitVal2([I)I\n        // @declaration a static method of `CB2`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local1;\n        local1 = 0;\n        while (local1 < arg0.length) {\n            if (arg0[local1] < 0) {\n                return local1;\n            } else {\n                local1 = local1 + 1;\n            }\n        }\n        return -(local1 + 1);\n    }\n";

/// `LR2.bsearch`: this change's own anchor — the patrol's bsearch over an array, in a class whose
/// every member recovers, so its whole-class presentation compiles as one unit.
const LR2_BSEARCH: &str = "    static int bsearch(int[] arg0, int arg1) {\n        // @method bsearch([II)I\n        // @declaration a static method of `LR2`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local2;\n        int local3;\n        local2 = 0;\n        local3 = arg0.length - 1;\n        while (local2 <= local3) {\n            int local4 = local2 + local3 >>> 1;\n            int local5 = arg0[local4];\n            if (local5 < arg1) {\n                local2 = local4 + 1;\n            } else if (local5 > arg1) {\n                local3 = local4 - 1;\n    } else {\n                return local4;\n    }\n        }\n        return -(local2 + 1);\n    }\n";

/// The two MVP-out shapes, pinned whole: the same canonical-overlap refusal the patrol recorded.
const LB_DOUBLE_LADDER: &str = "    static int doubleLadder(int[] arg0, int arg1) {\n        // jarde: not recovered: the recovery run for `doubleLadder([II)I` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method doubleLadder([II)I\n        // @declaration a static method of `LB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 0 1 2 3 4 5 8 9 10 11 12 13 14 17 18 19 20 21 24 25 26 29 30 31 32 33 36 37 38 41 42 43 44 45 46 47 50 51\n        // canonical block at BCI 47 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted\n    }\n";

const LB_TRY_LADDER: &str = "    static int tryLadder(int[] arg0, int arg1) {\n        // jarde: not recovered: the recovery run for `tryLadder([II)I` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method tryLadder([II)I\n        // @declaration a static method of `LB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 0 1 2 3 4 5 6 7 8 9 12 13 14 15 16 17 19 20 22 23 25 27 28 31 33 34 35 36 39 41 42 45 47 48 49 50 53 55 56 59 61 63 64 67 68\n        // canonical block at BCI 56 on jsr path [] has more than one owner in the completed Region tree; the whole method is quoted\n    }\n";

/// The two shapes this change's reading does not reach, pinned whole with their cross-quote refusal.
const LB_SWITCH_IN_ARM: &str = "    static int switchInArm(int[] arg0, int arg1) {\n        // jarde: not recovered: the recovery run for `switchInArm([II)I` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method switchInArm([II)I\n        // @declaration a static method of `LB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 0 4 10 21 28 34 56 63 67 74 76 79\n        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice\n    }\n";

const LB_FIRST_ARM_RET: &str = "    static int firstArmRet(int[] arg0, int arg1) {\n        // jarde: not recovered: the recovery run for `firstArmRet([II)I` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method firstArmRet([II)I\n        // @declaration a static method of `LB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 0 7 12 31 34 40 48 53 56\n        // local 2 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice\n    }\n";

/// The two shapes the reading does reach beside the anchor, pinned whole as well: a `for` header's
/// ladder (the update block is the join) and a switch that is the ladder's *sibling* — the reading
/// never examines sibling statements, so both recover.
const LB_FOR_LADDER: &str = "    static int forLadder(int[] arg0, int arg1) {\n        // @method forLadder([II)I\n        // @declaration a static method of `LB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local2;\n        local2 = 0;\n        while (local2 < arg0.length) {\n            int local3 = arg0[local2];\n            if (local3 < arg1) {\n                local2 = local2 + 1;\n            } else if (local3 > arg1) {\n                local2 = local2 + 2;\n    } else {\n                return local2;\n    }\n            local2 = local2 + 1;\n        }\n        return -1;\n    }\n";

const LB_SWITCH_LADDER: &str = "    static int switchLadder(int[] arg0, int arg1) {\n        // @method switchLadder([II)I\n        // @declaration a static method of `LB`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        int local2;\n        int local3;\n        local2 = 0;\n        local3 = 0;\n        int local4;\n        while (local2 < arg0.length) {\n            local4 = arg0[local2];\n            switch (local4) {\n                case 0:\n                    local3 = local3 + 1;\n                    break;\n                default:\n                    local3 = local3 - 1;\n                    break;\n            }\n            if (local4 < arg1) {\n                local2 = local2 + 1;\n            } else if (local4 > arg1) {\n                local2 = local2 + 2;\n    } else {\n                return local3;\n    }\n        }\n        return local3;\n    }\n";

// -------------------------------------------------------------------------------------------
// The anchors and the controls.
// -------------------------------------------------------------------------------------------

/// The patrol's two anchors recover whole on both legs: the ladder is one `if`/`else if` tree, the
/// canonical-overlap sentence is gone from the class, and no `@bytecode` quote is left in the
/// method. The patrol's own recorded BCI (56) is the join this change stops the arms at.
#[test]
fn the_ladder_recovers_where_the_patrol_recorded_the_refusal() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BS"));
        let report = presented(&snapshot, "BS");
        assert_eq!(text_of(&report, "bsearch"), BS_BSEARCH, "{}", leg.label);
        assert_eq!(
            report.text.matches(OVERLAP).count(),
            0,
            "`BS` keeps no ownership overlap on {}:\n{}",
            leg.label,
            report.text
        );
        assert!(
            !text_of(&report, "bsearch").contains("@bytecode"),
            "`BS.bsearch` is presented whole on {}:\n{}",
            leg.label,
            text_of(&report, "bsearch")
        );
        // The members beside it are untouched: the patrol's own twoPtr defect stays where it was.
        assert!(
            text_of(&report, "twoPtr").contains("@bytecode 24 17 23"),
            "`BS.twoPtr` keeps the patrol's recorded quote on {}:\n{}",
            leg.label,
            text_of(&report, "twoPtr")
        );

        let snapshot = open(&leg.fixture("CB"));
        let report = presented(&snapshot, "CB");
        assert_eq!(
            text_of(&report, "loopElseIfRet"),
            CB_LOOP_ELSE_IF_RET,
            "{}",
            leg.label
        );
        assert_eq!(
            report.text.matches(OVERLAP).count(),
            0,
            "`CB` keeps no ownership overlap on {}:\n{}",
            leg.label,
            report.text
        );
        assert!(
            !text_of(&report, "loopElseIfRet").contains("@bytecode"),
            "`CB.loopElseIfRet` is presented whole on {}",
            leg.label
        );
    }
}

/// The three controls are byte for byte what they were before the change — the patrol's own
/// recorded texts.
#[test]
fn the_three_controls_are_byte_identical() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CB"));
        let report = presented(&snapshot, "CB");
        assert_eq!(
            text_of(&report, "loopElseIfNoRet"),
            CB_LOOP_ELSE_IF_NO_RET,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "loopIfElseRet"),
            CB_LOOP_IF_ELSE_RET,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "noLoopElseIfRet"),
            CB_NO_LOOP_ELSE_IF_RET,
            "{}",
            leg.label
        );

        let snapshot = open(&leg.fixture("CB2"));
        let report = presented(&snapshot, "CB2");
        assert_eq!(text_of(&report, "exitVal"), CB2_EXIT_VAL, "{}", leg.label);
        assert_eq!(text_of(&report, "exitVal2"), CB2_EXIT_VAL2, "{}", leg.label);
    }
}

/// The MVP-out shapes keep their refusal **verbatim**, and the two shapes the reading does not
/// reach keep theirs: a double ladder and an exception table crossing the ladder still own one
/// block twice, a switch inside a ladder arm and an early return in the ladder's *first* arm still
/// leave a quoted fallback behind the loop variable.
#[test]
fn the_shapes_outside_the_single_level_ladder_keep_their_refusal() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("LB"));
        let report = presented(&snapshot, "LB");
        assert_eq!(
            text_of(&report, "doubleLadder"),
            LB_DOUBLE_LADDER,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "tryLadder"),
            LB_TRY_LADDER,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "switchInArm"),
            LB_SWITCH_IN_ARM,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "firstArmRet"),
            LB_FIRST_ARM_RET,
            "{}",
            leg.label
        );
        assert_eq!(
            report.text.matches(OVERLAP).count(),
            2,
            "only the two overlap negatives stay refused on {}:\n{}",
            leg.label,
            report.text
        );
        assert_eq!(
            report.text.matches(CROSS_QUOTE).count(),
            2,
            "only the two cross-quote negatives stay refused on {}:\n{}",
            leg.label,
            report.text
        );
        // The reading's actual reach, pinned: a `for` header's ladder and a switch that is the
        // ladder's sibling both regroup on the latch and recover.
        assert_eq!(
            text_of(&report, "forLadder"),
            LB_FOR_LADDER,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "switchLadder"),
            LB_SWITCH_LADDER,
            "{}",
            leg.label
        );
    }
}

/// This change's own anchor class recovers whole: every member is presented, and the whole-class
/// presentation is the one the replay compiles as a unit.
#[test]
fn the_all_recovering_anchor_class_is_presented_whole() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("LR2"));
        let report = presented(&snapshot, "LR2");
        assert_eq!(text_of(&report, "bsearch"), LR2_BSEARCH, "{}", leg.label);
        assert!(
            !report.text.contains("@bytecode"),
            "`LR2` is presented whole on {}:\n{}",
            leg.label,
            report.text
        );
        assert!(
            !report.text.contains("jarde: not recovered"),
            "`LR2` keeps no refused member on {}:\n{}",
            leg.label,
            report.text
        );
    }
}

// -------------------------------------------------------------------------------------------
// The replay: the presented text, stripped, compiled and run (needs a JDK on PATH).
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
            "jarde-loop-else-if-early-returns-{label}-{}-{nonce}-{sequence}",
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

/// One class text with every `//` comment line dropped — the strip the patrols' own stripped
/// sources were made by.
fn comment_lines_dropped(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::to_owned)
        .collect()
}

/// The fixture's own class files, written where a JVM can load them.
fn original_classes(leg: &Leg, class: &str, directory: &Path) {
    for (name, bytes) in leg.family(class) {
        fs::write(directory.join(name), bytes).expect("the fixture class is written");
    }
}

/// The real javac 8 the frozen `v8-javac8` leg was compiled by, when this machine holds it.
fn javac8() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("JARDE_JAVAC8") {
        return Some(PathBuf::from(path));
    }
    let default = PathBuf::from(
        "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac",
    );
    default.is_file().then_some(default)
}

/// Compile one replay's stripped unit: the installed javac under `--release 8`, or a real javac 8
/// (whose default target is Java 8, and which has no `--release` flag). The unit is written under
/// the class's own name, which is what a public class's declaration requires.
fn compile_with(javac: &Path, release_8: bool, directory: &Path, class: &str, source: &str) {
    let path = directory.join(format!("{class}.java"));
    fs::write(&path, source).expect("the stripped unit is written");
    let mut command = Command::new(javac);
    if release_8 {
        command.args(["--release", "8"]);
    }
    let output = command
        .arg("-Xlint:-options")
        .arg("-d")
        .arg(directory)
        .arg(&path)
        .output()
        .expect("the named javac runs");
    assert!(
        output.status.success(),
        "javac rejected the presented text:\n{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        source
    );
}

fn run(classpath: &Path, main_class: &str) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(classpath)
        .arg(main_class)
        .output()
        .expect("the installed JVM runs");
    assert!(
        output.status.success(),
        "the stripped text did not run:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the fixture prints text")
}

/// The two compilations this machine holds: the installed javac under `--release 8`, and a real
/// javac 8 when one is present.
fn compilers() -> Vec<(String, PathBuf, bool)> {
    let mut compilers = vec![(
        "the installed javac --release 8".to_owned(),
        PathBuf::from("javac"),
        true,
    )];
    if let Some(path) = javac8() {
        compilers.push((
            format!("the real javac 8 at {}", path.display()),
            path,
            false,
        ));
    }
    compilers
}

/// One or more members' recovered texts, as a whole unit: the class declaration the request named,
/// the members' own stripped presentations, and the fixture's own call sequence — the shape a
/// fixture whose `main` is refused by the reserved `jarde_refused_body()` marker can be replayed
/// in.
fn method_unit(class: &str, methods: &[&str], driver: &str) -> String {
    let bodies: Vec<String> = methods
        .iter()
        .map(|method| comment_lines_dropped(method).join("\n"))
        .collect();
    format!(
        "public class {class} {{\n{}\n{driver}}}\n",
        bodies.join("\n")
    )
}

/// The array a driver hands the recovered member: the fixture's own arguments, built without an
/// array initializer so the driver stays plain Java.
const DRIVER_SAMPLE: &str = "    static int[] sample(int... values) {\n        int[] xs = new int[values.length];\n        for (int i = 0; i < values.length; i++) { xs[i] = values[i]; }\n        return xs;\n    }\n";

/// Every stripped anchor answers what its own class answers, on both compiler legs this machine
/// holds: `LR2`'s whole-class presentation compiles as one unit, and each patrol anchor's recovered
/// **method** is compiled beside the fixture's own call sequence and compared with the values the
/// fixture class itself prints.
#[test]
#[ignore = "needs a JDK on PATH: it strips the class-source comment lines, compiles the text with \
            the installed javac --release 8 (and with a real javac 8 when one is present) and runs \
            both, comparing every answer with the fixture's own class files"]
fn every_stripped_anchor_answers_what_its_class_answers() {
    eprintln!(
        "the real javac 8 leg is {}",
        javac8().map_or_else(
            || "absent on this machine".to_owned(),
            |path| path.display().to_string()
        )
    );
    for leg in LEGS {
        // The fixture's own class, run first: its answer is what every replay is compared with.
        let original_dir = TempDir::new(&format!("{}-original", leg.label.replace(' ', "-")));
        for class in ["BS", "CB", "CB2", "LB", "LR2"] {
            original_classes(leg, class, original_dir.path());
        }
        let answers: Vec<(&str, Vec<String>)> = ["BS", "CB", "CB2", "LB", "LR2"]
            .into_iter()
            .map(|class| {
                let printed = run(original_dir.path(), class);
                (
                    class,
                    printed.trim_end().split('/').map(str::to_owned).collect(),
                )
            })
            .collect();
        let answer_of = |class: &str| -> Vec<String> {
            answers
                .iter()
                .find(|(name, _)| *name == class)
                .map(|(_, fields)| fields.clone())
                .expect("the fixture's own class ran")
        };
        // The fixtures' own runs, pinned: a fixture whose own run moved is a different fixture.
        assert_eq!(
            answer_of("BS"),
            ["2", "-3", "[1, 2, 3]", "102334155"],
            "{}: `BS`'s own run moved",
            leg.label
        );
        assert_eq!(
            answer_of("CB"),
            ["1", "4", "-1", "50"],
            "{}: `CB`'s own run moved",
            leg.label
        );
        assert_eq!(
            answer_of("CB2"),
            ["-3", "1"],
            "{}: `CB2`'s own run moved",
            leg.label
        );
        assert_eq!(
            answer_of("LB"),
            ["1", "1", "0", "0", "1", "-1"],
            "{}: `LB`'s own run moved",
            leg.label
        );
        assert_eq!(
            answer_of("LR2"),
            ["2", "-3"],
            "{}: `LR2`'s own run moved",
            leg.label
        );

        // `LR2`: the whole-class presentation, stripped, is one compilable unit — every member of
        // the class recovers, so the render needs no edit beyond the comment lines.
        let snapshot = open(&leg.fixture("LR2"));
        let report = presented(&snapshot, "LR2");
        let whole = comment_lines_dropped(&report.text).join("\n") + "\n";
        assert!(
            whole.contains("public class LR2"),
            "the strip lost the class declaration on {}",
            leg.label
        );
        for (label, javac, release_8) in compilers() {
            let directory = TempDir::new(&format!(
                "{}-whole-{}",
                leg.label.replace(' ', "-"),
                javac.display()
            ));
            compile_with(&javac, release_8, directory.path(), "LR2", &whole);
            assert_eq!(
                run(directory.path(), "LR2")
                    .trim_end()
                    .split('/')
                    .map(str::to_owned)
                    .collect::<Vec<_>>(),
                answer_of("LR2"),
                "{} / {label}: `LR2`'s whole-class strip diverges",
                leg.label
            );
        }

        // The patrol's anchors: the fixture's `main` is refused (a `void` body that wrote no
        // statement carries the reserved `jarde_refused_body()` marker, so the whole-class strip
        // cannot compile by design). The recovered **method's** text is compiled beside the
        // fixture's own call sequence instead, and compared with the values its class prints.
        let cases: [(&str, &[&str], &str, Vec<String>); 4] = [
            (
                "BS",
                &["bsearch"],
                "    public static void main(String[] a) { System.out.println(\"\" + bsearch(sample(1,3,5,7), 5) + \"/\" + bsearch(sample(1,3), 4)); }\n",
                answer_of("BS")[..2].to_vec(),
            ),
            (
                "CB",
                &["loopElseIfRet"],
                "    public static void main(String[] a) { System.out.println(\"\" + loopElseIfRet(sample(1,3,5), 3)); }\n",
                answer_of("CB")[..1].to_vec(),
            ),
            (
                "CB2",
                &["exitVal"],
                "    public static void main(String[] a) { System.out.println(\"\" + exitVal(sample(1,3,5), 4)); }\n",
                answer_of("CB2")[..1].to_vec(),
            ),
            (
                "LB",
                &["switchLadder", "forLadder"],
                "    public static void main(String[] a) { System.out.println(\"\" + switchLadder(sample(0,1,0), 1) + \"/\" + forLadder(sample(1,3,5), 3)); }\n",
                vec![answer_of("LB")[2].clone(), answer_of("LB")[5].clone()],
            ),
        ];
        for (class, methods, driver, expected) in cases {
            let snapshot = open(&leg.fixture(class));
            let report = presented(&snapshot, class);
            let texts: Vec<&str> = methods
                .iter()
                .map(|method| text_of(&report, method))
                .collect();
            let unit = method_unit(class, &texts, &format!("{DRIVER_SAMPLE}{driver}"));
            for (label, javac, release_8) in compilers() {
                let directory = TempDir::new(&format!(
                    "{}-{class}-{}",
                    leg.label.replace(' ', "-"),
                    methods.join("-")
                ));
                compile_with(&javac, release_8, directory.path(), class, &unit);
                assert_eq!(
                    run(directory.path(), class)
                        .trim_end()
                        .split('/')
                        .map(str::to_owned)
                        .collect::<Vec<_>>(),
                    expected,
                    "{} / {label}: `{class}.{}`'s stripped text diverges",
                    leg.label,
                    methods.join("+")
                );
            }
        }
    }
}
