//! `recover-boxed-number-widening` in one frozen target: the `java.lang.Number` rows of the six
//! boxed numeric classes inside `platform_interface_argument_widens`.
//!
//! The change states one closed row family: `Byte`, `Short`, `Integer`, `Long`, `Float` and
//! `Double` under `java.lang.Number`, the whole `java.lang` direct set the change's reflective
//! universe check found. The anchors are the patrol's own shapes, frozen on **both** compiler legs
//! (javac 23.0.1 `--release 8` and real javac 8, Corretto 1.8.0_432, from the same sources):
//!
//! * `C8` — the boxed-number patrol's fixture, byte-identical to the patrol's own source: the
//!   patrol's `main` refusal at BCI 63 (`Integer` at the erased `java.lang.Number` parameter of
//!   `larger(3, 7)`) is gone and the whole class presents, with `useWitness`/`loopBuilder` as the
//!   zero-regression controls the patrol recorded as healthy;
//! * `BN` — the six-row family the change pins: `larger(...)` is called with each of the six boxed
//!   types at the erased `Number` parameter, `pickSeq("x", "yy")` is the `String -> CharSequence`
//!   variant the sister implementer table already answers, `same(Integer.valueOf(9))` is the
//!   same-name control that introduces no cast, and `withParam` presents a boxed **parameter**
//!   (not a literal) at the same slot;
//! * `BNX` — the remaining negative: `java.math.BigDecimal` now has the separately pinned direct
//!   `Number` row, while `java.util.concurrent.atomic.AtomicInteger` remains outside the closed
//!   rows and keeps its reference-conversion refusal. The whole `main` still falls back atomically
//!   when that latter call is refused.
//!
//! A `Boolean -> Number` argument position is not a source a javac accepts (a `Boolean` is not
//! convertible to `Number`), so that negative is pinned where it can be stated: the unit test
//! `the_boxed_number_rows_reach_exactly_their_pairs` beside the table.
//!
//! The ignored replay strips the presentations the way the patrol's own stripped source was made
//! (comment lines dropped), compiles each anchor with the installed `javac --release 8` and, when
//! a real javac 8 is present, with that one too, runs both under `-Xverify:all` and compares every
//! answer with the fixture's own class files.

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
// The fixtures: the patrol's own shape and the change's variants, compiled by both javac legs.
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names and the class files it produced.
struct Leg {
    label: &'static str,
    files: &'static [(&'static str, &'static [u8])],
}

impl Leg {
    /// The class files of one fixture family, in container order: the class itself and its own
    /// companions, then any separately declared class the presentation reads.
    fn family(&self, class: &str, extra: &[&str]) -> Vec<(String, &'static [u8])> {
        let own = format!("{class}.class");
        let nested = format!("{class}$");
        let mut files: Vec<(String, &'static [u8])> = Vec::new();
        for (name, bytes) in self.files {
            let name: &str = name;
            if name == own || name.starts_with(&nested) {
                files.push((name.to_owned(), *bytes));
            }
        }
        assert_eq!(
            files.first().map(|(name, _)| name.as_str()),
            Some(own.as_str()),
            "the fixture family of `{class}` is committed"
        );
        for (name, bytes) in self.files {
            let name: &str = name;
            if extra.iter().any(|other| name == format!("{other}.class")) {
                files.push((name.to_owned(), *bytes));
            }
        }
        files
    }

    /// One fixture family's container.
    fn fixture(&self, class: &str, extra: &[&str]) -> Vec<u8> {
        let own = format!("{class}.class");
        let nested = format!("{class}$");
        let mut entries: Vec<(&[u8], &[u8])> = Vec::new();
        for (name, bytes) in self.files {
            let name: &str = name;
            if name == own || name.starts_with(&nested) {
                entries.push((name.as_bytes(), *bytes));
            }
        }
        assert_eq!(
            entries.first().map(|(name, _)| *name),
            Some(own.as_bytes()),
            "the fixture family of `{class}` is committed"
        );
        for (name, bytes) in self.files {
            let name: &str = name;
            if extra.iter().any(|other| name == format!("{other}.class")) {
                entries.push((name.as_bytes(), *bytes));
            }
        }
        zip_of(&entries)
    }
}

/// javac 23.0.1, `javac --release 8 -Xlint:-options -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "C8.class",
        include_bytes!("fixtures/recover-boxed-number-widening/v8/C8.class"),
    ),
    (
        "BN.class",
        include_bytes!("fixtures/recover-boxed-number-widening/v8/BN.class"),
    ),
    (
        "BNX.class",
        include_bytes!("fixtures/recover-boxed-number-widening/v8/BNX.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "C8.class",
        include_bytes!("fixtures/recover-boxed-number-widening/v8-javac8/C8.class"),
    ),
    (
        "BN.class",
        include_bytes!("fixtures/recover-boxed-number-widening/v8-javac8/BN.class"),
    ),
    (
        "BNX.class",
        include_bytes!("fixtures/recover-boxed-number-widening/v8-javac8/BNX.class"),
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

/// The ordinary request plus the one optional category the member fold reads: the presented texts
/// below are the ones this layer writes for a fold that can state its call-site anchors, the same
/// request the platform-interface family's own target pins.
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

/// `C8.main`: the patrol's anchor — the `larger(3, 7)` statement the patrol recorded as refused at
/// BCI 63 now presents, with `Integer -> java.lang.Number` written at both parameter positions.
const C8_MAIN: &str = "    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `C8`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.String) new C8().useWitness());\n        java.lang.System.out.println(\"\" + boxedTern(true) + \":\" + boxedTern(false));\n        java.lang.System.out.println((java.lang.Object) larger((java.lang.Number) java.lang.Integer.valueOf(3), (java.lang.Number) java.lang.Integer.valueOf(7)));\n        java.lang.System.out.println((java.lang.String) loopBuilder(9));\n        return;\n    }\n";

/// `C8.useWitness`: the patrol's explicit-witness control — unchanged.
const C8_USEWITNESS: &str = "    java.lang.String useWitness() {\n        // @method useWitness()Ljava/lang/String;\n        // @declaration an instance method of `C8`, member flags 0x0000\n        // recovered from bytecode; presentation is not claimed to compile\n        return (java.lang.String) this.pick((java.lang.Object) \"x\", (java.lang.Object) \"y\");\n    }\n";

/// `C8.loopBuilder`: the patrol's loop-carried builder control — unchanged.
const C8_LOOPBUILDER: &str = "    static java.lang.String loopBuilder(int arg0) {\n        // @method loopBuilder(I)Ljava/lang/String;\n        // @declaration a static method of `C8`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.StringBuilder local1;\n        int local2;\n        local1 = new java.lang.StringBuilder();\n        for (local2 = 0; local2 < arg0; local2 = local2 + 1) {\n            local1.append(local2 % 2 == 0 ? \"e\" : \"o\");\n            if (local1.length() > 6) {\n                break;\n            }\n        }\n        return local1.toString();\n    }\n";

/// `C8.larger`: the generic method's erased presentation — the body is the patrol's own, the
/// signature refusal is the fold's own marker and not a reference-conversion refusal.
const C8_LARGER: &str = "    // jarde: generic Signature projection refused for `larger(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return\n    static java.lang.Number larger(java.lang.Number arg0, java.lang.Number arg1) {\n        // @method larger(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Number;\n        // @declaration a static method of `C8`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0.doubleValue() >= arg1.doubleValue() ? arg0 : arg1;\n    }\n";

/// `BN.main`: one call site per row of the boxed-number family — `Integer`, `Long`, `Double`,
/// `Float`, `Short` and `Byte` each at the erased `java.lang.Number` parameter — beside the
/// `String -> CharSequence` variant the sister implementer table answers, the same-name control
/// and the boxed-parameter call.
const BN_MAIN: &str = "    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `BN`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.Object) larger((java.lang.Number) java.lang.Integer.valueOf(3), (java.lang.Number) java.lang.Integer.valueOf(7)));\n        java.lang.System.out.println((java.lang.Object) larger((java.lang.Number) java.lang.Long.valueOf(3L), (java.lang.Number) java.lang.Long.valueOf(7L)));\n        java.lang.System.out.println((java.lang.Object) larger((java.lang.Number) java.lang.Double.valueOf(0x1.c000000000000p1d), (java.lang.Number) java.lang.Double.valueOf(0x1.e000000000000p2d)));\n        java.lang.System.out.println((java.lang.Object) larger((java.lang.Number) java.lang.Float.valueOf(0x1.c00000p1f), (java.lang.Number) java.lang.Float.valueOf(0x1.e00000p2f)));\n        java.lang.System.out.println((java.lang.Object) larger((java.lang.Number) java.lang.Short.valueOf((short) 3), (java.lang.Number) java.lang.Short.valueOf((short) 7)));\n        java.lang.System.out.println((java.lang.Object) larger((java.lang.Number) java.lang.Byte.valueOf((byte) 3), (java.lang.Number) java.lang.Byte.valueOf((byte) 7)));\n        java.lang.System.out.println((java.lang.String) pickSeq((java.lang.CharSequence) \"x\", (java.lang.CharSequence) \"yy\"));\n        java.lang.System.out.println((java.lang.Object) same((java.lang.Number) java.lang.Integer.valueOf(9)));\n        java.lang.System.out.println((java.lang.Object) withParam((java.lang.Integer) java.lang.Integer.valueOf(4)));\n        return;\n    }\n";

/// `BN.withParam`: a boxed **parameter** at the erased `Number` slot — the widening does not
/// depend on a literal, and the second argument position presents too.
const BN_WITHPARAM: &str = "    static java.lang.Number withParam(java.lang.Integer arg0) {\n        // @method withParam(Ljava/lang/Integer;)Ljava/lang/Number;\n        // @declaration a static method of `BN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return larger((java.lang.Number) arg0, (java.lang.Number) java.lang.Integer.valueOf(0));\n    }\n";

/// `BN.same`: the same-name control — a `java.lang.Number` value at a `java.lang.Number` slot
/// keeps its render and introduces no cast.
const BN_SAME: &str = "    static java.lang.Number same(java.lang.Number arg0) {\n        // @method same(Ljava/lang/Number;)Ljava/lang/Number;\n        // @declaration a static method of `BN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0;\n    }\n";

/// `BN.pickSeq`: the `String -> CharSequence` variant — the sister implementer table's row, not
/// this change's, so the render the earlier slice wrote stays byte-identical.
const BN_PICKSEQ: &str = "    // jarde: generic Signature projection refused for `pickSeq(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/lang/CharSequence;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return\n    static java.lang.CharSequence pickSeq(java.lang.CharSequence arg0, java.lang.CharSequence arg1) {\n        // @method pickSeq(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/lang/CharSequence;\n        // @declaration a static method of `BN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0.length() >= arg1.length() ? arg0 : arg1;\n    }\n";

/// `BN.larger`: the generic method's erased presentation, the same fold marker `C8.larger` carries.
const BN_LARGER: &str = "    // jarde: generic Signature projection refused for `larger(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Number;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return\n    static java.lang.Number larger(java.lang.Number arg0, java.lang.Number arg1) {\n        // @method larger(Ljava/lang/Number;Ljava/lang/Number;)Ljava/lang/Number;\n        // @declaration a static method of `BN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return arg0.doubleValue() >= arg1.doubleValue() ? arg0 : arg1;\n    }\n";

/// `BNX.main`: BigDecimal no longer refuses at BCI 21; AtomicInteger at BCI 46 still refuses.
/// The whole body falls back because the refused AtomicInteger call blocks partial publication.
const BNX_MAIN: &str = "    public static void main(java.lang.String[] arg0) {\n        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `BNX`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 49 27 46 0 3 6 7 9 12 15 16 18 21 24 30 33 34 35 38 41 42 43 52\n        // the parameter 0 of the invocation at BCI 46 is declared `java.lang.Number` presents `java.util.concurrent.atomic.AtomicInteger` but the invocation requires `java.lang.Number` and this layer has no safe reference conversion evidence\n        jarde_refused_body();\n    }\n";

/// The refusal text every negative below states: the one reference-conversion sentence this layer
/// writes, unchanged by this change.
const REFUSAL: &str = "but the invocation requires";

// -------------------------------------------------------------------------------------------
// The anchors.
// -------------------------------------------------------------------------------------------

/// The patrol's anchor: `C8.main` presents whole, the class keeps no reference-conversion
/// refusal, and the patrol's healthy controls are byte-identical.
#[test]
fn the_patrol_anchor_presents_its_boxed_argument() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("C8", &[]));
        let report = class_source_of(&snapshot, "C8");
        assert_eq!(text_of(&report, "main"), C8_MAIN, "{}", leg.label);
        assert_eq!(
            text_of(&report, "useWitness"),
            C8_USEWITNESS,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "loopBuilder"),
            C8_LOOPBUILDER,
            "{}",
            leg.label
        );
        assert_eq!(text_of(&report, "larger"), C8_LARGER, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`C8` keeps no reference-conversion refusal on `{}`:\n{}",
            leg.label,
            report.text
        );
    }
}

/// The six-row family: every boxed numeric class at the erased `Number` parameter, the
/// `CharSequence` variant, the same-name control and the boxed-parameter call, with no refusal
/// left in the class.
#[test]
fn the_boxed_number_positions_are_presented() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BN", &[]));
        let report = class_source_of(&snapshot, "BN");
        assert_eq!(text_of(&report, "main"), BN_MAIN, "{}", leg.label);
        assert_eq!(text_of(&report, "withParam"), BN_WITHPARAM, "{}", leg.label);
        assert_eq!(text_of(&report, "same"), BN_SAME, "{}", leg.label);
        assert_eq!(text_of(&report, "pickSeq"), BN_PICKSEQ, "{}", leg.label);
        assert_eq!(text_of(&report, "larger"), BN_LARGER, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`BN` keeps no reference-conversion refusal on `{}`:\n{}",
            leg.label,
            report.text
        );
    }
}

/// BigDecimal no longer refuses at BCI 21; AtomicInteger at BCI 46 still refuses. The complete
/// `main` remains a fallback because one unproved call prevents publishing a partial body.
#[test]
fn atomic_number_subclass_outside_the_closed_rows_still_refuses() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("BNX", &[]));
        let report = class_source_of(&snapshot, "BNX");
        assert_eq!(text_of(&report, "main"), BNX_MAIN, "{}", leg.label);
        assert_eq!(
            report.text.matches(REFUSAL).count(),
            1,
            "only the AtomicInteger pair stays refused on `{}`:\n{}",
            leg.label,
            report.text
        );
        assert!(
            text_of(&report, "main").contains("at BCI 46 is declared `java.lang.Number` presents `java.util.concurrent.atomic.AtomicInteger`"),
            "the AtomicInteger refusal remains anchored: {}",
            text_of(&report, "main")
        );
        assert!(
            !text_of(&report, "main").contains("at BCI 21"),
            "BigDecimal at BCI 21 no longer carries a refusal: {}",
            text_of(&report, "main")
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
            "jarde-boxed-number-widening-{label}-{}-{nonce}-{sequence}",
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

/// One class text with every `//` comment line dropped — the strip the patrol's own stripped
/// source was made by.
fn comment_lines_dropped(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::to_owned)
        .collect()
}

/// One anchor's replay: which container to open, which units are compiled **from their own
/// presentation**, and what the fixture's own class files answer.
struct Replay {
    class: &'static str,
    extra: &'static [&'static str],
    presented: &'static [&'static str],
    answer: &'static str,
}

const REPLAYS: &[Replay] = &[
    Replay {
        class: "C8",
        extra: &[],
        presented: &["C8"],
        answer: "x\n1:2\n7\neoeoeoe\n",
    },
    Replay {
        class: "BN",
        extra: &[],
        presented: &["BN"],
        answer: "7\n7\n7.5\n7.5\n7\n7\nyy\n9\n4\n",
    },
];

/// The units of one replay: every presented unit stripped.
fn replay_sources(leg: &Leg, replay: &Replay) -> Vec<(String, String)> {
    let snapshot = open(&leg.fixture(replay.class, replay.extra));
    let mut sources: Vec<(String, String)> = Vec::new();
    for name in replay.presented {
        let text = class_source_of(&snapshot, name).text;
        sources.push((
            format!("{name}.java"),
            comment_lines_dropped(&text).join("\n") + "\n",
        ));
    }
    sources
}

/// The fixture's own class files, written where a JVM can load them.
fn original_classes(leg: &Leg, replay: &Replay, directory: &Path) {
    for (name, bytes) in leg.family(replay.class, replay.extra) {
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

/// Compile one replay's stripped units: the installed javac under `--release 8`, or a real javac 8
/// (whose default target is Java 8, and which has no `--release` flag).
fn compile_with(
    javac: &Path,
    release_8: bool,
    directory: &Path,
    sources: &[(String, String)],
) -> Vec<PathBuf> {
    let paths: Vec<PathBuf> = sources
        .iter()
        .map(|(file, text)| {
            let path = directory.join(file);
            fs::write(&path, text).expect("the stripped unit is written");
            path
        })
        .collect();
    let mut command = Command::new(javac);
    if release_8 {
        command.args(["--release", "8"]);
    }
    let output = command
        .arg("-Xlint:-options")
        .arg("-d")
        .arg(directory)
        .args(&paths)
        .output()
        .expect("the named javac runs");
    assert!(
        output.status.success(),
        "javac rejected the presented text:\n{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        paths
            .iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .collect::<Vec<_>>()
            .join("\n")
    );
    paths
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

/// One replay's stripped text compiled and run by both compiler legs this machine holds, and the
/// fixture's own classes run — one answer per compilation.
fn replay(leg: &Leg, replay: &Replay) -> Vec<(String, String, String)> {
    let sources = replay_sources(leg, replay);
    let original_dir = TempDir::new(&format!("{}-original", leg.label.replace(' ', "-")));
    original_classes(leg, replay, original_dir.path());
    let original_run = run(original_dir.path(), replay.class);

    let mut answers = Vec::new();
    let installed = PathBuf::from("javac");
    answers.push((
        format!(
            "{} / compiled by the installed javac --release 8",
            leg.label
        ),
        {
            let directory = TempDir::new(&format!("{}-current", leg.label.replace(' ', "-")));
            compile_with(&installed, true, directory.path(), &sources);
            run(directory.path(), replay.class)
        },
        original_run.clone(),
    ));
    if let Some(javac) = javac8() {
        answers.push((
            format!("{} / compiled by {}", leg.label, javac.display()),
            {
                let directory = TempDir::new(&format!("{}-javac8", leg.label.replace(' ', "-")));
                compile_with(&javac, false, directory.path(), &sources);
                run(directory.path(), replay.class)
            },
            original_run.clone(),
        ));
    }
    answers
}

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
        for anchor in REPLAYS {
            for (label, answer, original) in replay(leg, anchor) {
                assert_eq!(
                    original, anchor.answer,
                    "{label}: the fixture's own run moved"
                );
                assert_eq!(answer, original, "{label}: the stripped text diverges");
            }
        }
    }
}
