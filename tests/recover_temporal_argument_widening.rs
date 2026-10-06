//! `recover-temporal-argument-widening` in one frozen target: the `java.time` Temporal family's
//! rows and the batch-2 `java.util.concurrent.CompletableFuture` rows.
//!
//! The change states two closed row families inside `platform_interface_argument_widens` — the
//! seven `java.time` named types under `java.time.temporal.Temporal` and
//! `java.time.temporal.TemporalAccessor` (the second read through the first's own header), and
//! `CompletableFuture` under the two interfaces its own header declares. The anchors are the
//! patrols' own shapes, frozen on **both** compiler legs (javac 23.0.1 `--release 8` and real
//! javac 8, Corretto 1.8.0_432, from the same sources):
//!
//! * `JT` — the java-time-temporal patrol's fixture: `fmt()`'s `DateTimeFormatter.format(dt)` and
//!   `LocalDateTime.parse(f.format(dt), f)` slots write
//!   `(java.time.temporal.TemporalAccessor)`, `spans()`'s `Duration.between` and
//!   `ChronoUnit.MONTHS.between` slots write `(java.time.temporal.Temporal)`, and the already
//!   recovering `basic()` (factories, chaining, `plusDays(long)`) is the zero-regression control;
//! * `DT` — the patrol's direct-argument discriminator: `direct()` writes the
//!   `TemporalAccessor` cast, `parse()` (the `CharSequence` slot the earlier implementer tables
//!   answer) is the second control;
//! * `CF` — the completable-future patrol's fixture: `combined()`'s `thenCombine` writes
//!   `(java.util.concurrent.CompletionStage)`, `chain()`/`recover()` (already recovering) are the
//!   regression controls.
//!
//! The change's own negative stands beside them: `TWX` presents `java.time.MonthDay` at a
//! `TemporalAccessor` slot and `java.time.Year` at a `Temporal` slot — both declare the interface
//! in their own headers, neither is one of the fourteen rows the change pins, so both keep the
//! reference-conversion refusal verbatim.
//!
//! The ignored replay strips the presentations the way the patrols' own stripped sources were made
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
// The fixtures: the patrols' own shapes, compiled by both javac legs.
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
        "JT.class",
        include_bytes!("fixtures/recover-temporal-argument-widening/v8/JT.class"),
    ),
    (
        "DT.class",
        include_bytes!("fixtures/recover-temporal-argument-widening/v8/DT.class"),
    ),
    (
        "CF.class",
        include_bytes!("fixtures/recover-temporal-argument-widening/v8/CF.class"),
    ),
    (
        "TWX.class",
        include_bytes!("fixtures/recover-temporal-argument-widening/v8/TWX.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "JT.class",
        include_bytes!("fixtures/recover-temporal-argument-widening/v8-javac8/JT.class"),
    ),
    (
        "DT.class",
        include_bytes!("fixtures/recover-temporal-argument-widening/v8-javac8/DT.class"),
    ),
    (
        "CF.class",
        include_bytes!("fixtures/recover-temporal-argument-widening/v8-javac8/CF.class"),
    ),
    (
        "TWX.class",
        include_bytes!("fixtures/recover-temporal-argument-widening/v8-javac8/TWX.class"),
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

/// `JT.basic`: the patrol's already recovering chain — the zero-regression control the change
/// must leave byte-identical (factory, `plusDays(long)`, `withDayOfMonth`, `atTime`).
const JT_BASIC: &str = "    static java.lang.String basic() {\n        // @method basic()Ljava/lang/String;\n        // @declaration a static method of `JT`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.time.LocalDate local0 = java.time.LocalDate.of(2026, 10, 6);\n        java.time.LocalDate local1 = local0.plusDays(10L).plusMonths(1L).withDayOfMonth(1);\n        java.time.LocalDateTime local2 = local1.atTime(9, 30);\n        return \"\" + local0 + \"/\" + local1 + \"/\" + local2.toLocalTime();\n    }\n";

/// `JT.fmt`: the change's main anchor — both `DateTimeFormatter.format(LocalDateTime)` sites
/// (`f.format(dt)` written twice) now carry the `TemporalAccessor` cast, and the
/// `LocalDateTime.parse(CharSequence, DateTimeFormatter)` call between them keeps the earlier
/// implementer table's `CharSequence` render.
const JT_FMT: &str = "    static java.lang.String fmt() {\n        // @method fmt()Ljava/lang/String;\n        // @declaration a static method of `JT`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.time.format.DateTimeFormatter local0 = java.time.format.DateTimeFormatter.ofPattern(\"yyyy/MM/dd HH:mm\");\n        java.time.LocalDateTime local1 = java.time.LocalDateTime.of(2026, 1, 2, 3, 4);\n        java.time.LocalDateTime local2 = java.time.LocalDateTime.parse((java.lang.CharSequence) local0.format((java.time.temporal.TemporalAccessor) local1), local0);\n        return local0.format((java.time.temporal.TemporalAccessor) local1) + \" -> \" + local2.getHour() + \":\" + local2.getMinute();\n    }\n";

/// `JT.spans`: `Duration.between(LocalDateTime, LocalDateTime)` and
/// `ChronoUnit.MONTHS.between(LocalDate, LocalDate)` — the `Temporal` slots (`Period.between`
/// takes two `LocalDate` parameters and needs no cast).
const JT_SPANS: &str = "    static long spans() {\n        // @method spans()J\n        // @declaration a static method of `JT`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.time.LocalDate local0 = java.time.LocalDate.of(2026, 1, 1);\n        java.time.LocalDate local1 = java.time.LocalDate.of(2026, 3, 15);\n        java.time.Duration local2 = java.time.Duration.between((java.time.temporal.Temporal) local0.atStartOfDay(), (java.time.temporal.Temporal) local1.atStartOfDay());\n        java.time.Period local3 = java.time.Period.between(local0, local1);\n        return local2.toDays() + java.time.temporal.ChronoUnit.MONTHS.between((java.time.temporal.Temporal) local0, (java.time.temporal.Temporal) local1) * 100L + (long) local3.getDays();\n    }\n";

/// `JT.main`: the replay's entry point, `f.format(dt)` covered through `fmt()` and `spans()`.
const JT_MAIN: &str = "    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `JT`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.String) basic());\n        java.lang.System.out.println((java.lang.String) fmt());\n        java.lang.System.out.println(spans());\n        return;\n    }\n";

/// `DT.direct`: the patrol's direct-argument discriminator — a `LocalDateTime` value (not an
/// array read) at the formatter's `TemporalAccessor` slot.
const DT_DIRECT: &str = "    static java.lang.String direct() {\n        // @method direct()Ljava/lang/String;\n        // @declaration a static method of `DT`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.time.format.DateTimeFormatter local0 = java.time.format.DateTimeFormatter.ofPattern(\"HH:mm\");\n        java.time.LocalDateTime local1 = java.time.LocalDateTime.of(2026, 1, 2, 3, 4);\n        return local0.format((java.time.temporal.TemporalAccessor) local1);\n    }\n";

/// `DT.parse`: the second direction's control — the `CharSequence` slot the earlier implementer
/// table answers, unchanged by this change.
const DT_PARSE: &str = "    static java.lang.String parse() {\n        // @method parse()Ljava/lang/String;\n        // @declaration a static method of `DT`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return java.time.LocalTime.parse((java.lang.CharSequence) \"03:04\", (java.time.format.DateTimeFormatter) java.time.format.DateTimeFormatter.ofPattern(\"HH:mm\")).toString();\n    }\n";

/// `DT.main`: the replay's entry point.
const DT_MAIN: &str = "    public static void main(java.lang.String[] arg0) {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `DT`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println(direct() + \"/\" + parse());\n        return;\n    }\n";

/// `CF.chain`: the patrol's already recovering synchronous chain — a zero-regression control.
const CF_CHAIN: &str = "    static java.lang.String chain() throws java.lang.Exception {\n        // @method chain()Ljava/lang/String;\n        // @declaration a static method of `CF`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.util.concurrent.CompletableFuture local0 = java.util.concurrent.CompletableFuture.supplyAsync((java.util.function.Supplier) (() -> CF.lambda$chain$0()));\n        java.lang.String local1 = (java.lang.String) local0.thenApply((java.util.function.Function) ((java.lang.Object p0) -> CF.lambda$chain$1((java.lang.String) p0))).thenApply((java.util.function.Function) ((java.lang.Object p0_) -> ((java.lang.String) p0_).toUpperCase())).get();\n        return local1;\n    }\n";

/// `CF.recover`: the `exceptionally` chain — the second zero-regression control.
const CF_RECOVER: &str = "    static java.lang.String recover() throws java.lang.Exception {\n        // @method recover()Ljava/lang/String;\n        // @declaration a static method of `CF`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.util.concurrent.CompletableFuture local0 = java.util.concurrent.CompletableFuture.supplyAsync((java.util.function.Supplier) (() -> CF.lambda$recover$2()));\n        return (java.lang.String) local0.exceptionally((java.util.function.Function) ((java.lang.Object p0) -> CF.lambda$recover$3((java.lang.Throwable) p0))).get();\n    }\n";

/// `CF.combined`: the batch-2 anchor — `thenCombine`'s `CompletionStage` parameter now carries
/// the cast the `CompletableFuture` row proves.
const CF_COMBINED: &str = "    static java.lang.String combined() throws java.lang.Exception {\n        // @method combined()Ljava/lang/String;\n        // @declaration a static method of `CF`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.util.concurrent.CompletableFuture local0 = java.util.concurrent.CompletableFuture.completedFuture((java.lang.Object) java.lang.Integer.valueOf(2));\n        java.util.concurrent.CompletableFuture local1 = java.util.concurrent.CompletableFuture.supplyAsync((java.util.function.Supplier) (() -> CF.lambda$combined$4()));\n        return ((java.lang.Integer) local0.thenCombine((java.util.concurrent.CompletionStage) local1, (java.util.function.BiFunction) ((java.lang.Object p0, java.lang.Object p1) -> CF.lambda$combined$5((java.lang.Integer) p0, (java.lang.Integer) p1))).get()).toString();\n    }\n";

/// `CF.main`: the replay's entry point and the third zero-regression control.
const CF_MAIN: &str = "    public static void main(java.lang.String[] arg0) throws java.lang.Exception {\n        // @method main([Ljava/lang/String;)V\n        // @declaration a static method of `CF`, member flags 0x0009\n        // recovered from bytecode; presentation is not claimed to compile\n        java.lang.System.out.println((java.lang.String) chain());\n        java.lang.System.out.println((java.lang.String) recover());\n        java.lang.System.out.println((java.lang.String) combined());\n        return;\n    }\n";

/// `TWX.month`: this change's own negative — `java.time.MonthDay`'s header declares
/// `TemporalAccessor` verbatim, but the closed fourteen rows do not carry it, so the pair keeps
/// the refusal rather than widening from the header's existence.
const TWX_MONTH: &str = "    static java.lang.String month() {\n        // jarde: not recovered: the recovery run for `month()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method month()Ljava/lang/String;\n        // @declaration a static method of `TWX`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 13 10 2 7\n        // the parameter 0 of the invocation at BCI 10 is declared `java.time.temporal.TemporalAccessor` presents `java.time.MonthDay` but the invocation requires `java.time.temporal.TemporalAccessor` and this layer has no safe reference conversion evidence\n    }\n";

/// `TWX.year`: the same negative at the `Temporal` slot (`java.time.Year`'s header declares it;
/// no row names it).
const TWX_YEAR: &str = "    static long year() {\n        // jarde: not recovered: the recovery run for `year()J` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method year()J\n        // @declaration a static method of `TWX`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 18 15 0 6 12\n        // the parameter 0 of the invocation at BCI 15 is declared `java.time.temporal.Temporal` presents `java.time.Year` but the invocation requires `java.time.temporal.Temporal` and this layer has no safe reference conversion evidence\n    }\n";

/// The refusal text every negative below states: the one reference-conversion sentence this layer
/// writes, unchanged by this change.
const REFUSAL: &str = "but the invocation requires";

// -------------------------------------------------------------------------------------------
// The anchors.
// -------------------------------------------------------------------------------------------

/// The `java.time` Temporal family: the `TemporalAccessor` slots of `JT.fmt`/`DT.direct` and the
/// `Temporal` slots of `JT.spans`, with `JT.basic` (already recovering) untouched and no
/// refusal left in either class.
#[test]
fn the_temporal_family_positions_are_presented() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("JT", &[]));
        let report = class_source_of(&snapshot, "JT");
        assert_eq!(text_of(&report, "basic"), JT_BASIC, "{}", leg.label);
        assert_eq!(text_of(&report, "fmt"), JT_FMT, "{}", leg.label);
        assert_eq!(text_of(&report, "spans"), JT_SPANS, "{}", leg.label);
        assert_eq!(text_of(&report, "main"), JT_MAIN, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`JT` keeps no reference-conversion refusal on `{}`:\n{}",
            leg.label,
            report.text
        );

        let snapshot = open(&leg.fixture("DT", &[]));
        let report = class_source_of(&snapshot, "DT");
        assert_eq!(text_of(&report, "direct"), DT_DIRECT, "{}", leg.label);
        assert_eq!(text_of(&report, "parse"), DT_PARSE, "{}", leg.label);
        assert_eq!(text_of(&report, "main"), DT_MAIN, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`DT` keeps no reference-conversion refusal on `{}`:\n{}",
            leg.label,
            report.text
        );
    }
}

/// The batch-2 rows: `thenCombine`'s `CompletionStage` parameter now carries the cast, and the
/// patrol's already recovering chain and `exceptionally` member are the regression controls.
#[test]
fn the_completion_stage_position_is_presented() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CF", &[]));
        let report = class_source_of(&snapshot, "CF");
        assert_eq!(text_of(&report, "chain"), CF_CHAIN, "{}", leg.label);
        assert_eq!(text_of(&report, "recover"), CF_RECOVER, "{}", leg.label);
        assert_eq!(text_of(&report, "combined"), CF_COMBINED, "{}", leg.label);
        assert_eq!(text_of(&report, "main"), CF_MAIN, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`CF` keeps no reference-conversion refusal on `{}`:\n{}",
            leg.label,
            report.text
        );
    }
}

/// The change's own negative: a `java.time` type whose header declares the interface but which no
/// row names keeps both refusals verbatim, and nothing else in the class is refused.
#[test]
fn the_types_outside_the_closed_rows_still_refuse() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("TWX", &[]));
        let report = class_source_of(&snapshot, "TWX");
        assert_eq!(text_of(&report, "month"), TWX_MONTH, "{}", leg.label);
        assert_eq!(text_of(&report, "year"), TWX_YEAR, "{}", leg.label);
        assert_eq!(
            report.text.matches(REFUSAL).count(),
            2,
            "only the two out-of-row pairs stay refused on `{}`:\n{}",
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
            "jarde-temporal-argument-widening-{label}-{}-{nonce}-{sequence}",
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
        class: "JT",
        extra: &[],
        presented: &["JT"],
        answer: "2026-10-06/2026-11-01/09:30\n2026/01/02 03:04 -> 3:4\n287\n",
    },
    Replay {
        class: "DT",
        extra: &[],
        presented: &["DT"],
        answer: "03:04/03:04\n",
    },
    Replay {
        class: "CF",
        extra: &[],
        presented: &["CF"],
        answer: "DATA!\nfallback:java.lang.IllegalStateException: x\n6\n",
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
