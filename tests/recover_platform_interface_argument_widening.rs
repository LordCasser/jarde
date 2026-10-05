//! `recover-platform-interface-argument-widening`: the argument whose presented type is a class of
//! the analyzed snapshot and whose required type is a platform interface the snapshot does not
//! hold, but which the presented type's own class-file header lists verbatim.
//!
//! The critical anchor is the comparator-anon patrol's `CP.byAnon`
//! (`openspec/evidence/java-syntax-2026-10-05/comparator-anon-patrol/`):
//!
//! ```text
//! Collections.sort(c, new Comparator<User>(){ public int compare(User a, User b){ ... } });
//! ```
//!
//! The argument's presented type is the companion `CP$1`, whose own header states
//! `implements java/util/Comparator`; the required type is the JDK interface, which the snapshot
//! does not hold. The hierarchy walk read every name it reached **including the target**, so the
//! walk stopped at the JDK interface and refused the whole call: the sorted call vanished from the
//! text (the patrol measured the isolated strip printing `[30, 10, 20]` where the class prints
//! `[10, 20, 30]`). The same shape with a **top-level named** implementer (`AH`'s `AC`, no `$` in
//! the argument's type) refused identically, which is what ruled the companion spelling out as the
//! cause.
//!
//! What this change fixes is the walk's order: the target is proved by the **name** the class file
//! one step earlier states in its own superclass/interfaces array, so reaching that name proves the
//! relation without ever reading the target's header. Every class before it on the chain is still
//! read and proved from this snapshot alone, no classpath is consulted, and a target no snapshot
//! header names is never reached — it keeps its refusal verbatim.
//!
//! What the presented texts pin, on **both** compiler legs (javac 23.0.1 `--release 8` and real
//! javac 8, the same sources both times):
//!
//! * `CP.byAnon` writes the whole call — `java.util.Collections.sort((java.util.List) local1,
//!   (java.util.Comparator) new CP$1());` — with no reference-conversion refusal anywhere in the
//!   class text;
//! * `AH.byTop`, the top-level named implementer, writes the same call with `new AC()`;
//! * `IS.byAnon` (the patrol's own isolated age-ordering shape, `[10, 20, 30]`) writes the same
//!   call over the raw `IS$1`;
//! * the **own-interface** positions of `AN` — return, argument, static-field initializer and local
//!   — render exactly as they did before this change: a same-run interface is not this change's
//!   subject;
//! * `AW.viaNamedPlatform` proves the platform interface a snapshot class header names
//!   (`java.lang.Runnable`), while `AW.viaAbsent` keeps the refusal verbatim where the target
//!   (`java.lang.Throwable`) appears in no snapshot header at all — the walk states the relation it
//!   read and guesses no classpath.
//!
//! The ignored replay (`cargo test -- --ignored`) strips the presentations the way the patrol's own
//! `isolated-rendered-wrong.java` was made — comment lines dropped — compiles `CP`, `AH`/`AC` and
//! `IS` with the installed `javac --release 8` **and**, when a real javac 8 is present, with that
//! one too, and runs both under `-Xverify:all`: each answers exactly what the fixture's own class
//! files answer (`IS` answers `[10, 20, 30]`, the value the pre-change strip lost).

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
    /// One fixture family's container: its own class file and its companions'.
    fn fixture(&self, class: &str) -> Vec<u8> {
        self.fixture_with(class, &[])
    }

    /// The same container plus the extra top-level classes a presentation reads: `AH`'s argument is
    /// the separately declared `AC`, which the class file names and the container must hold.
    fn fixture_with(&self, class: &str, extra: &[&str]) -> Vec<u8> {
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

    /// The class files of one fixture family, in container order.
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
        for (name, bytes) in self.files {
            let name: &str = name;
            if extra.iter().any(|other| name == format!("{other}.class")) {
                files.push((name.to_owned(), *bytes));
            }
        }
        files
    }
}

/// javac 23.0.1, `javac --release 8 -Xlint:-options -d v8 *.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "AC.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AC.class"),
    ),
    (
        "AH.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AH.class"),
    ),
    (
        "AN.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AN.class"),
    ),
    (
        "AN$1.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AN$1.class"),
    ),
    (
        "AN$2.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AN$2.class"),
    ),
    (
        "AN$3.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AN$3.class"),
    ),
    (
        "AN$4.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AN$4.class"),
    ),
    (
        "AN$Op.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AN$Op.class"),
    ),
    (
        "AW.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AW.class"),
    ),
    (
        "AW$MyErr.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AW$MyErr.class"),
    ),
    (
        "AW$Work.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/AW$Work.class"),
    ),
    (
        "CP.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/CP.class"),
    ),
    (
        "CP$1.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/CP$1.class"),
    ),
    (
        "CP$User.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/CP$User.class"),
    ),
    (
        "IS.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/IS.class"),
    ),
    (
        "IS$1.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/IS$1.class"),
    ),
    (
        "IS$User.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8/IS$User.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -d v8-javac8 *.java`: no `--release`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "AC.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8-javac8/AC.class"),
    ),
    (
        "AH.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8-javac8/AH.class"),
    ),
    (
        "AN.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8-javac8/AN.class"),
    ),
    (
        "AN$1.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/AN$1.class"
        ),
    ),
    (
        "AN$2.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/AN$2.class"
        ),
    ),
    (
        "AN$3.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/AN$3.class"
        ),
    ),
    (
        "AN$4.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/AN$4.class"
        ),
    ),
    (
        "AN$Op.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/AN$Op.class"
        ),
    ),
    (
        "AW.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8-javac8/AW.class"),
    ),
    (
        "AW$MyErr.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/AW$MyErr.class"
        ),
    ),
    (
        "AW$Work.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/AW$Work.class"
        ),
    ),
    (
        "CP.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8-javac8/CP.class"),
    ),
    (
        "CP$1.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/CP$1.class"
        ),
    ),
    (
        "CP$User.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/CP$User.class"
        ),
    ),
    (
        "IS.class",
        include_bytes!("fixtures/recover-platform-interface-argument-widening/v8-javac8/IS.class"),
    ),
    (
        "IS$1.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/IS$1.class"
        ),
    ),
    (
        "IS$User.class",
        include_bytes!(
            "fixtures/recover-platform-interface-argument-widening/v8-javac8/IS$User.class"
        ),
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
/// below are the ones this layer writes for a fold that can state its call-site anchors, which is
/// the presentation the class literal and member-fold slices already pin under the same request.
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

/// `CP.byAnon`: the critical anchor — the anonymous `Comparator` argument's whole call, and the
/// generic Signature note the sorter's own `List<User>` return keeps (a class-level debt this change
/// does not touch).
const CP_BY_ANON: &str = "    // jarde: generic Signature projection refused for `byAnon(Ljava/util/List;)Ljava/util/List;`: unsupported (generic_source_shape_unproved): class name has no unambiguous Java source spelling\n    static java.util.List byAnon(java.util.List arg0) {\n        // @method byAnon(Ljava/util/List;)Ljava/util/List;\n        // @declaration a static method of `CP`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.util.ArrayList local1 = new java.util.ArrayList((java.util.Collection) arg0);\n        java.util.Collections.sort((java.util.List) local1, (java.util.Comparator) new CP$1());\n        return local1;\n    }\n";

/// `AH.byTop` — the same call whose argument type carries no `$`: `AC` is a top-level class of the
/// same container, and the walk proves its header's own interface list.
const AH_BY_TOP: &str = "    // jarde: generic Signature projection refused for `byTop(Ljava/util/List;)Ljava/util/List;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types\n    static java.util.List byTop(java.util.List arg0) {\n        // @method byTop(Ljava/util/List;)Ljava/util/List;\n        // @declaration a static method of `AH`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.util.ArrayList local1 = new java.util.ArrayList((java.util.Collection) arg0);\n        java.util.Collections.sort((java.util.List) local1, (java.util.Comparator) new AC());\n        return local1;\n    }\n";

/// `IS.byAnon` — the patrol's isolated age-ordering shape, raw `Comparator` and all: the strip of
/// this text is the replay that answers `[10, 20, 30]` where the pre-change strip lost the sort.
const IS_BY_ANON: &str = "    static java.util.List byAnon(java.util.List arg0) {\n        // @method byAnon(Ljava/util/List;)Ljava/util/List;\n        // @declaration a static method of `IS`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        java.util.ArrayList local1 = new java.util.ArrayList((java.util.Collection) arg0);\n        java.util.Collections.sort((java.util.List) local1, (java.util.Comparator) new IS$1());\n        return local1;\n    }\n";

/// `AN.returned`: the return position of the own-interface family — unchanged by this change.
const AN_RETURNED: &str = "    static AN$Op returned() {\n        // @method returned()LAN$Op;\n        // @declaration a static method of `AN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return new AN$1();\n    }\n";

/// `AN.asArg`: the argument position of the own-interface family, with the cast the same-run
/// interface already had.
const AN_AS_ARG: &str = "    static int asArg(int arg0) {\n        // @method asArg(I)I\n        // @declaration a static method of `AN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        return run((AN$Op) new AN$2(), arg0);\n    }\n";

/// `AN.<clinit>`: the static-field initializer position.
const AN_CLINIT: &str = "    static {\n        // @method <clinit>()V\n        // @declaration a static initializer of `AN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        AN.field = new AN$3();\n    }\n";

/// `AN.localVar`: the local-variable position.
const AN_LOCAL_VAR: &str = "    static int localVar() {\n        // @method localVar()I\n        // @declaration a static method of `AN`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        AN$4 local0 = new AN$4();\n        return local0.apply(5);\n    }\n";

/// `AW.viaNamedPlatform`: the platform interface a snapshot class header **names** — `AW$Work`
/// states `implements java.lang.Runnable` and the call presents.
const AW_VIA_NAMED_PLATFORM: &str = "    static void viaNamedPlatform() {\n        // @method viaNamedPlatform()V\n        // @declaration a static method of `AW`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        runRunnable((java.lang.Runnable) new AW$Work());\n        return;\n    }\n";

/// `AW.viaAbsent`: the target appears in no snapshot header at all (`AW$MyErr extends
/// java.lang.Exception`, and the platform chain up to `java.lang.Throwable` is not in this
/// snapshot), so the refusal stands verbatim — no classpath is consulted and nothing is guessed.
const AW_VIA_ABSENT: &str = "    static void viaAbsent() {\n        // jarde: not recovered: the recovery run for `viaAbsent()V` produced no statement (explanation only); the artifact's own comment lines are below\n        // @method viaAbsent()V\n        // @declaration a static method of `AW`, member flags 0x0008\n        // recovered from bytecode; presentation is not claimed to compile\n        // @bytecode 7 0 3 4 10\n        // the parameter 0 of the invocation at BCI 7 is declared `java.lang.Throwable` presents `AW$MyErr` but the invocation requires `java.lang.Throwable` and this layer has no safe reference conversion evidence\n        jarde_refused_body();\n    }\n";

/// The refusal text both fixture legs write for the nameless platform target.
const AW_ABSENT_REFUSAL: &str = "presents `AW$MyErr` but the invocation requires `java.lang.Throwable` and this layer has no safe reference conversion evidence";

// -------------------------------------------------------------------------------------------
// The anchors.
// -------------------------------------------------------------------------------------------

#[test]
fn the_anonymous_comparator_argument_is_presented() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("CP"));
        let report = class_source_of(&snapshot, "CP");
        assert_eq!(text_of(&report, "byAnon"), CP_BY_ANON, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`{}` keeps no reference-conversion refusal:\n{}",
            leg.label,
            report.text
        );
        assert!(
            !report.text.contains("@bytecode 17"),
            "BCI 17 is a presented statement on `{}`:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn the_top_level_implementer_argument_is_presented() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture_with("AH", &["AC"]));
        let report = class_source_of(&snapshot, "AH");
        assert_eq!(text_of(&report, "byTop"), AH_BY_TOP, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`{}` keeps no reference-conversion refusal:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn the_isolated_ordering_program_presents_its_sort() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("IS"));
        let report = class_source_of(&snapshot, "IS");
        assert_eq!(text_of(&report, "byAnon"), IS_BY_ANON, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`{}` keeps no reference-conversion refusal:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn the_own_interface_positions_do_not_move() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("AN"));
        let report = class_source_of(&snapshot, "AN");
        assert_eq!(text_of(&report, "returned"), AN_RETURNED, "{}", leg.label);
        assert_eq!(text_of(&report, "asArg"), AN_AS_ARG, "{}", leg.label);
        assert_eq!(text_of(&report, "<clinit>"), AN_CLINIT, "{}", leg.label);
        assert_eq!(text_of(&report, "localVar"), AN_LOCAL_VAR, "{}", leg.label);
        assert!(
            !report
                .text
                .contains("no safe reference conversion evidence"),
            "`{}` keeps no reference-conversion refusal:\n{}",
            leg.label,
            report.text
        );
    }
}

#[test]
fn the_named_platform_target_is_proved_and_the_absent_one_still_refuses() {
    for leg in LEGS {
        let snapshot = open(&leg.fixture("AW"));
        let report = class_source_of(&snapshot, "AW");
        assert_eq!(
            text_of(&report, "viaNamedPlatform"),
            AW_VIA_NAMED_PLATFORM,
            "{}",
            leg.label
        );
        assert_eq!(
            text_of(&report, "viaAbsent"),
            AW_VIA_ABSENT,
            "{}",
            leg.label
        );
        assert!(
            report.text.contains(AW_ABSENT_REFUSAL),
            "the nameless platform target keeps its refusal verbatim on `{}`:\n{}",
            leg.label,
            report.text
        );
        assert_eq!(
            report
                .text
                .matches("no safe reference conversion evidence")
                .count(),
            1,
            "only the nameless target stays refused on `{}`:\n{}",
            leg.label,
            report.text
        );
        // The member fold claims this class's text and re-spells the presented member-name token at
        // the anchor the fold's own source map states: the nested declaration is published under the
        // simple name while the member text keeps the pool spelling it was recovered with.
        assert!(
            report.text.contains(
                "static class Work extends java.lang.Object implements java.lang.Runnable"
            ),
            "`{}` publishes the member declaration the call names:\n{}",
            leg.label,
            report.text
        );
        assert!(
            report
                .text
                .contains("runRunnable((java.lang.Runnable) new Work());"),
            "`{}` re-spells the call site the header proof presented:\n{}",
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
            "jarde-platform-interface-widening-{label}-{}-{nonce}-{sequence}",
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

/// One class text with every `//` comment line dropped — the strip the patrol's own
/// `isolated-rendered-wrong.java` was made by.
fn comment_lines_dropped(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::to_owned)
        .collect()
}

/// The stripped presentation of one fixture: its own text and every companion's, each a source
/// unit of its own under the binary name its class file states (a companion named `X$Y` is its own
/// class file, which is exactly how the fixture's family compiles).
fn replay_sources(leg: &Leg, class: &str, extra: &[&str]) -> Vec<(String, String)> {
    let snapshot = open(&leg.fixture_with(class, extra));
    leg.family(class, extra)
        .into_iter()
        .map(|(file, _)| {
            let name = file
                .strip_suffix(".class")
                .expect("a class file name ends in `.class`")
                .to_owned();
            let text = class_source_of(&snapshot, &name).text;
            let stripped = comment_lines_dropped(&text).join("\n") + "\n";
            (format!("{name}.java"), stripped)
        })
        .collect()
}

/// The fixture's own class files, written where a JVM can load them.
fn original_classes(leg: &Leg, class: &str, extra: &[&str], directory: &Path) {
    for (name, bytes) in leg.family(class, extra) {
        fs::write(directory.join(name), bytes).expect("the fixture class is written");
    }
}

/// The real javac 8 the frozen `v8-javac8` leg was compiled by, when this machine holds it.
fn javac8() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("JARDE_JAVAC8") {
        return Some(PathBuf::from(path));
    }
    let default = PathBuf::from(
        "/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac",
    );
    default.is_file().then_some(default)
}

/// Compile one fixture's stripped units: the installed javac under `--release 8`, or a real javac 8
/// (whose default target is Java 8, and which has no `--release` flag). Every unit of the family is
/// written into the directory first and compiled in one invocation, exactly as the fixture itself
/// was.
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

/// The stripped text compiled and run by both compiler legs this machine holds, and the fixture's
/// own classes run — one answer per compilation.
fn replay(leg: &Leg, class: &str, extra: &[&str]) -> Vec<(String, String, String)> {
    let sources = replay_sources(leg, class, extra);
    let original_dir = TempDir::new(&format!("{}-original", leg.label.replace(' ', "-")));
    original_classes(leg, class, extra, original_dir.path());
    let original_run = run(original_dir.path(), class);

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
            run(directory.path(), class)
        },
        original_run.clone(),
    ));
    if let Some(javac) = javac8() {
        answers.push((
            format!("{} / compiled by {}", leg.label, javac.display()),
            {
                let directory = TempDir::new(&format!("{}-javac8", leg.label.replace(' ', "-")));
                compile_with(&javac, false, directory.path(), &sources);
                run(directory.path(), class)
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
fn every_stripped_fixture_answers_what_its_class_answers() {
    eprintln!(
        "the real javac 8 leg is {}",
        javac8().map_or_else(
            || "absent on this machine".to_owned(),
            |path| path.display().to_string()
        )
    );
    let expected = [
        (
            "CP",
            &[][..],
            "[bo:30, al:40, al:20]/[al:20, bo:30, al:40]/[a, bb, ccc]\n",
        ),
        ("AH", &["AC"][..], "[a, bb, ccc]\n"),
        ("IS", &[][..], "[10, 20, 30]\n"),
    ];
    for leg in LEGS {
        for (class, extra, text) in expected {
            for (label, answer, original) in replay(leg, class, extra) {
                assert_eq!(original, text, "{label}: the fixture's own run moved");
                assert_eq!(answer, original, "{label}: the stripped text diverges");
            }
        }
    }
    // `AN` compiles as its own family too, and its four own-interface positions still answer the
    // patrol's value.
    for leg in LEGS {
        for (label, answer, original) in replay(leg, "AN", &[]) {
            assert_eq!(original, "11/42/9/25\n", "{label}");
            assert_eq!(answer, original, "{label}: the stripped text diverges");
        }
    }
}
