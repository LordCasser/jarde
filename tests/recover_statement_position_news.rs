//! `recover-statement-position-news` in one frozen target: the **statement position** of a
//! construction — `new X(args);`, a `new` whose finished instance nothing consumes — and the
//! refusals that must not move beside it.
//!
//! The patrol (`openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/`) found the
//! boundary of `refuse-unconsumed-construction-invokes` too wide on the shapes with no order
//! hazard: `B5.main`'s three constructions and `B6`'s `argless`/`withArg` refused whole, so the
//! constructor side effects a reader strips the comments to find were silently dropped. This change
//! reads the statement position as the existing new@1 construction proof's own consumer side: the
//! one reader of the finished instance is the category-1 `pop` that discards it, and every argument
//! is a value the `new` expression writes in place — a constant, a direct local or parameter read,
//! or a construction this same proof completed.
//!
//! What must **not** move is pinned here too: the frozen counterexample of
//! `refuse-unconsumed-construction-invokes` (`VoidBetween.make`, and this change's hand-built
//! statement-position twin `SPC.run`) keeps `jre_new_interleaved_effect` naming BCI 4, because the
//! `Invoke ∉ argument_dependencies` branch runs **before** the statement-position reader is
//! consulted; `chained` (`new B6(new B6(1).n)`) keeps its refusal as a registered boundary; and the
//! negatives whose arguments carry an effect of their own (a real invocation, a field read, an
//! arithmetic chain) keep the refusal text they had.
//!
//! The fixtures are the patrol's own frozen `B5`/`B6` (read from their committed place) and this
//! change's `SP`/`SB`/`SPN` on both compiler legs plus the assembled `SPC` (see the fixture
//! README). The ignored replay strips the presentations the way the patrols' own stripped sources
//! were made (comment lines dropped), compiles each anchor with the installed `javac --release 8`
//! and, when a real javac 8 is present, with that one too, runs both under `-Xverify:all` and
//! compares every answer with the fixture's own class files — and asserts that the classes which
//! keep a refused body do **not** compile.

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

const STORE: u16 = 0;

// -------------------------------------------------------------------------------------------
// The patrol's frozen anchors and this change's fixtures.
// -------------------------------------------------------------------------------------------

/// The patrol's frozen composite: `main` builds `B5`, `B5(7)` and `Sub` in three statement
/// positions whose constructor side effects are `println`s.
const PATROL_B5: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/fixture/B5.class"
);
const PATROL_B5_SUB: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/fixture/B5$Sub.class"
);
/// The patrol's four-shape discriminator.
const PATROL_B6: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/fixture/B6.class"
);
/// The composite's own answer, which the recovered text must print.
const PATROL_ORIG_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/fixture/orig.out"
);

/// The frozen counterexample of `refuse-unconsumed-construction-invokes`: `new Target; dup;
/// Side.effect()V; iconst_1; Target.<init>(I)V; areturn`. Its refusal is the interleaved branch's,
/// and it stays verbatim.
const VOID_BETWEEN: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-25/ordinary-new-void-effect/VoidBetween.class"
);

/// This change's statement-position twin of that counterexample (assembled; see the fixture README).
const SPC: &[u8] = include_bytes!("fixtures/recover-statement-position-news/SPC.class");

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
}

/// javac 23.0.1, `javac --release 8 -g -nowarn -d v8 SP.java SB.java SPN.java`.
const V8_FILES: &[(&str, &[u8])] = &[
    (
        "SP.class",
        include_bytes!("fixtures/recover-statement-position-news/v8/SP.class"),
    ),
    (
        "SP$Inner.class",
        include_bytes!("fixtures/recover-statement-position-news/v8/SP$Inner.class"),
    ),
    (
        "SB.class",
        include_bytes!("fixtures/recover-statement-position-news/v8/SB.class"),
    ),
    (
        "SPN.class",
        include_bytes!("fixtures/recover-statement-position-news/v8/SPN.class"),
    ),
];

/// The real javac 8 leg (Corretto 1.8.0_432, `javac -g -nowarn -d v8-javac8 …`).
const V8_JAVAC8_FILES: &[(&str, &[u8])] = &[
    (
        "SP.class",
        include_bytes!("fixtures/recover-statement-position-news/v8-javac8/SP.class"),
    ),
    (
        "SP$Inner.class",
        include_bytes!("fixtures/recover-statement-position-news/v8-javac8/SP$Inner.class"),
    ),
    (
        "SB.class",
        include_bytes!("fixtures/recover-statement-position-news/v8-javac8/SB.class"),
    ),
    (
        "SPN.class",
        include_bytes!("fixtures/recover-statement-position-news/v8-javac8/SPN.class"),
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
// The presented texts.
// -------------------------------------------------------------------------------------------

/// `B5.main` — the patrol's composite: three statement-position constructions, in order. The
/// member class is folded into this same presentation (`static class Sub extends B5`), so the
/// reference is written in the source nesting the fold declares.
const ANCHOR_B5_MAIN: &[&str] = &[
    "        new B5();",
    "        new B5(7);",
    "        new Sub();",
];

/// `B6.argless`/`B6.withArg` — the minimal discriminator's two recovering shapes.
const ANCHOR_B6_ARGLESS: &str = "        new B6();";
const ANCHOR_B6_WITH_ARG: &str = "        new B6(7);";

/// The statements `SP` writes on both legs, per method.
const ANCHOR_SP_METHODS: &[(&str, &[&str])] = &[
    ("argless()", &["        new SP();"]),
    ("withArg()", &["        new SP(7);"]),
    ("fromArg(int x)", &["        new SP(x);"]),
    ("nestedArgument()", &["        new SP(new SP(1));"]),
    ("consumed()", &["        int r = new SP(3).n;"]),
    (
        "mixed()",
        &[
            "        new SP(4);",
            "        int r = new SP(5).n;",
            "        new SP(6);",
            "        java.lang.System.out.println(new SP(8).n);",
            "        new Inner();",
        ],
    ),
];

/// The statements `SB` writes on both legs: the patrol's `B6` shapes without the registered
/// `chained` boundary.
const ANCHOR_SB_METHODS: &[(&str, &[&str])] = &[
    ("argless()", &["        new SB();"]),
    ("withArg()", &["        new SB(7);"]),
    ("consumed()", &["        int r = new SB(3).n;"]),
    (
        "main(java.lang.String[] args)",
        &["        java.lang.System.out.println(new SB(9).n);"],
    ),
];

/// The consumed-position shapes that must not move: the patrol's `B6.consumed`/`main` and the
/// fixture's own controls.
const CONSUMED_METHODS: &[(&str, &str, &str)] = &[
    ("B6", "consumed()V", "        int local0 = new B6(3).n;"),
    (
        "B6",
        "main([Ljava/lang/String;)V",
        "        java.lang.System.out.println(new B6(9).n);",
    ),
    ("SP", "consumed()", "        int r = new SP(3).n;"),
    (
        "SPN",
        "main(java.lang.String[] args)",
        "        java.lang.System.out.println(SPN.holder.n);",
    ),
];

/// The refusal `SPN` keeps per method: the code and the two phrases its message states.
const SPN_REFUSALS: &[(&str, &str, &[&str])] = &[
    (
        "callArgument()",
        "jre_new_shape",
        &[
            "the instance the allocation at BCI 0 builds is read only by instructions this build quotes (BCIs 10)",
        ],
    ),
    (
        "fieldArgument()",
        "jre_new_interleaved_effect",
        &[
            "the instruction at BCI 4 is an Field",
            "between the allocation's copy and its constructor call",
        ],
    ),
    (
        "arithmeticArgument(int x)",
        "jre_new_shape",
        &[
            "the instance the allocation at BCI 0 builds is read only by instructions this build quotes (BCIs 10)",
        ],
    ),
    (
        "chained()",
        "jre_new_shape",
        &[
            "the construction at BCI 4 completes inside the construction at BCI 0, and its value is not one of the arguments of the constructor call at BCI 15",
        ],
    ),
];

/// The registered `chained` boundary of the patrol's own `B6`, verbatim.
const B6_CHAINED_REFUSAL: &[&str] = &[
    "the construction at BCI 4 completes inside the construction at BCI 0, and its value is not one of the arguments of the constructor call at BCI 15",
];

/// The message the interleaved branch states, on both the frozen counterexample and its
/// statement-position twin: the branch runs before the reader is consulted.
const INTERLEAVED_REFUSAL: &str =
    "the invocation at BCI 4 is not a value dependency of the constructor's physical arguments";

/// The one diagnostic the statement-position refusal states, which no recovering member may carry.
const UNCONSUMED_REFUSAL: &str = "read only by instructions this build quotes";

/// The methods that must present every instruction they have, per class.
const NO_QUOTE_METHODS: &[(&str, &str)] = &[
    ("B5", "main([Ljava/lang/String;)V"),
    ("B6", "argless()V"),
    ("B6", "withArg()V"),
    ("B6", "consumed()V"),
    ("B6", "main([Ljava/lang/String;)V"),
    ("SP", "argless()"),
    ("SP", "withArg()"),
    ("SP", "fromArg(int x)"),
    ("SP", "nestedArgument()"),
    ("SP", "consumed()"),
    ("SP", "mixed()"),
    ("SP", "main(java.lang.String[] args)"),
    ("SB", "argless()"),
    ("SB", "withArg()"),
    ("SB", "consumed()"),
    ("SB", "main(java.lang.String[] args)"),
];

// -------------------------------------------------------------------------------------------
// The class-source surface.
// -------------------------------------------------------------------------------------------

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
            // The construction records are this rule's own detail plane: the site verdicts the
            // text is read beside are materialized only under `RuleDetails`.
            &RecoveryEvidenceRequest::essential()
                .with_kind(RecoveryEvidenceKind::SourceMap)
                .with_kind(RecoveryEvidenceKind::RuleDetails),
            &mut budget(),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one committed sample answers one definition: {other:?}"),
    }
}

/// One class's presentation, with jarde's own self-header asserted **before** anything is counted
/// in it: a render of nothing is not a render.
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

/// One class's presentation out of one jar built from the committed class files of one leg.
fn presented_family(leg: &Leg, name: &str) -> ClassSourceReport {
    let family = leg.family(name);
    let entries: Vec<(&[u8], &[u8])> = family
        .iter()
        .map(|(name, bytes)| (name.as_bytes(), *bytes))
        .collect();
    let snapshot = open(&zip_of(&entries));
    presented(&snapshot, name)
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

/// Every bytecode index one presentation quotes.
fn quoted_bcis(text: &str) -> Vec<u32> {
    let mut quoted = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("// @bytecode ") else {
            continue;
        };
        for token in rest.split_whitespace() {
            if let Ok(bci) = token.parse::<u32>() {
                quoted.push(bci);
            }
        }
    }
    quoted
}

/// The body text of one method, from its own signature line to the closing brace.
fn method_body<'a>(text: &'a str, signature: &str) -> &'a str {
    let start = text
        .find(signature)
        .unwrap_or_else(|| panic!("the presentation states `{signature}`:\n{text}"));
    let rest = &text[start..];
    let end = rest
        .find("\n    }")
        .unwrap_or_else(|| panic!("the method `{signature}` closes:\n{text}"));
    &rest[..end]
}

/// The construction records one member's run states.
fn news<'a>(report: &'a ClassSourceReport, name: &str) -> &'a [jarde_java::init::NewRecord] {
    let member = report
        .methods
        .iter()
        .find(|member| member.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("missing method `{name}`"));
    match &member.outcome {
        ClassSourceOutcome::Recovered { report, .. } => &report.news,
        other => panic!("method `{name}` was not recovered: {other:?}"),
    }
}

/// The one refusal one member's records state, and the record's own code.
fn refusal<'a>(records: &'a [jarde_java::init::NewRecord], head: u32) -> (&'static str, &'a str) {
    let record = records
        .iter()
        .find(|record| record.head == head)
        .unwrap_or_else(|| panic!("no construction candidate at BCI {head}: {records:?}"));
    assert!(
        !record.presented(),
        "the candidate at BCI {head} is presented: {record:?}"
    );
    let refusal = record
        .refusal
        .as_ref()
        .unwrap_or_else(|| panic!("the refused candidate at BCI {head} states no refusal"));
    (refusal.code, refusal.message.as_str())
}

/// Whether one member's records present every candidate they read.
fn all_presented(records: &[jarde_java::init::NewRecord]) -> bool {
    !records.is_empty() && records.iter().all(jarde_java::init::NewRecord::presented)
}

// -------------------------------------------------------------------------------------------
// The anchors: the patrol's frozen classes recover, and the boundaries stay refused.
// -------------------------------------------------------------------------------------------

/// The patrol's composite renders whole: `B5.main`'s three statement-position constructions are
/// written, and no bytecode of the class is quoted.
#[test]
fn the_frozen_composite_renders_whole() {
    let snapshot = open(&zip_of(&[
        (b"B5.class", PATROL_B5),
        (b"B5$Sub.class", PATROL_B5_SUB),
    ]));
    let report = presented(&snapshot, "B5");
    let quoted = quoted_bcis(&report.text);
    assert!(
        quoted.is_empty(),
        "the frozen `B5` still quotes {quoted:?}:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("not recovered"),
        "the frozen `B5` is not a whole recovery:\n{}",
        report.text
    );
    assert!(
        !report.text.contains(UNCONSUMED_REFUSAL),
        "the frozen `B5` still carries the statement-position refusal:\n{}",
        report.text
    );
    let main = method_body(&report.text, "main([Ljava/lang/String;)V");
    for line in ANCHOR_B5_MAIN {
        assert!(
            main.contains(line),
            "the frozen `B5.main` lost {line:?}:\n{main}"
        );
    }
    let records = news(&report, "main");
    assert!(
        all_presented(records),
        "the frozen `B5.main` did not present every construction: {records:?}"
    );
}

/// The patrol's discriminator: `argless` and `withArg` recover, `chained` keeps its refusal, and
/// the consumed shapes do not move.
#[test]
fn the_frozen_discriminator_recovers_its_two_shapes() {
    let snapshot = open(&zip_of(&[(b"B6.class", PATROL_B6)]));
    let report = presented(&snapshot, "B6");
    assert!(
        method_body(&report.text, "argless()V").contains(ANCHOR_B6_ARGLESS),
        "the frozen `B6.argless` lost its statement:\n{}",
        report.text
    );
    assert!(
        method_body(&report.text, "withArg()V").contains(ANCHOR_B6_WITH_ARG),
        "the frozen `B6.withArg` lost its statement:\n{}",
        report.text
    );
    assert!(
        all_presented(news(&report, "argless")),
        "`B6.argless` did not present its construction: {:?}",
        news(&report, "argless")
    );
    assert!(
        all_presented(news(&report, "withArg")),
        "`B6.withArg` did not present its construction: {:?}",
        news(&report, "withArg")
    );
    // The registered boundary: `chained` keeps the refusal it had, verbatim.
    let (code, message) = refusal(news(&report, "chained"), 0);
    assert_eq!(code, "jre_new_shape");
    for phrase in B6_CHAINED_REFUSAL {
        assert!(message.contains(phrase), "`B6.chained` moved: {message}");
    }
    let chained = method_body(&report.text, "chained()V");
    assert!(
        chained.contains("jarde_refused_body();"),
        "the frozen `B6.chained` is presented as recovered:\n{chained}"
    );
    // The consumed shapes keep the text they had.
    for (_, signature, expected) in CONSUMED_METHODS
        .iter()
        .filter(|(class, _, _)| *class == "B6")
    {
        let body = method_body(&report.text, signature);
        assert!(
            body.contains(expected),
            "`B6.{signature}` moved: expected {expected:?}:\n{body}"
        );
    }
    assert!(
        all_presented(news(&report, "consumed")),
        "`B6.consumed` did not present its construction: {:?}",
        news(&report, "consumed")
    );
    assert!(
        all_presented(news(&report, "main")),
        "`B6.main` did not present its construction: {:?}",
        news(&report, "main")
    );
}

/// The statement position itself: `SP`'s five shapes on both legs, and its consumed controls.
#[test]
fn the_statement_position_shapes_recover_on_both_legs() {
    for leg in LEGS {
        let report = presented_family(leg, "SP");
        let quoted = quoted_bcis(&report.text);
        assert!(
            quoted.is_empty(),
            "`SP` on {} still quotes {quoted:?}:\n{}",
            leg.label,
            report.text
        );
        for (signature, anchors) in ANCHOR_SP_METHODS {
            let body = method_body(&report.text, signature);
            for anchor in *anchors {
                assert!(
                    body.contains(anchor),
                    "`SP.{signature}` on {} lost {anchor:?}:\n{body}",
                    leg.label
                );
            }
            let name = signature.split('(').next().expect("a signature has a name");
            assert!(
                all_presented(news(&report, name)),
                "`SP.{signature}` on {} did not present every construction: {:?}",
                leg.label,
                news(&report, name)
            );
        }
    }
}

/// `SB` — the patrol's `B6` shapes without the registered boundary — renders whole on both legs.
#[test]
fn the_discriminator_shapes_render_whole_without_the_boundary() {
    for leg in LEGS {
        let report = presented_family(leg, "SB");
        let quoted = quoted_bcis(&report.text);
        assert!(
            quoted.is_empty(),
            "`SB` on {} still quotes {quoted:?}:\n{}",
            leg.label,
            report.text
        );
        assert!(
            !report.text.contains(UNCONSUMED_REFUSAL),
            "`SB` on {} still carries the statement-position refusal:\n{}",
            leg.label,
            report.text
        );
        for (signature, anchors) in ANCHOR_SB_METHODS {
            let body = method_body(&report.text, signature);
            for anchor in *anchors {
                assert!(
                    body.contains(anchor),
                    "`SB.{signature}` on {} lost {anchor:?}:\n{body}",
                    leg.label
                );
            }
        }
    }
}

// -------------------------------------------------------------------------------------------
// The boundaries: the CST counterexample's ordering, and the negatives.
// -------------------------------------------------------------------------------------------

/// The statement position's own counterexample: the interleaved call is refused **before** the
/// reader is consulted, so `SPC.run` — whose argument is a constant and whose reader is the `pop`
/// the statement position accepts — keeps the interleaved code and BCI.
#[test]
fn the_statement_position_counterexample_keeps_the_interleaved_refusal() {
    let snapshot = open(&zip_of(&[(b"SPC.class", SPC)]));
    let report = presented(&snapshot, "SPC");
    let (code, message) = refusal(news(&report, "run"), 0);
    assert_eq!(code, "jre_new_interleaved_effect");
    assert!(
        message.contains(INTERLEAVED_REFUSAL) && message.contains("at BCI 8"),
        "the statement-position counterexample's refusal moved: {message}"
    );
    assert!(
        !message.contains(UNCONSUMED_REFUSAL),
        "the statement-position reader answered before the interleaved branch: {message}"
    );
    assert!(
        method_body(&report.text, "run()V").contains("jarde_refused_body();"),
        "the statement-position counterexample is presented as recovered:\n{}",
        report.text
    );
    // The control the ordering is read against: the same statement position **without** the
    // interleaved call is accepted (`SP.withArg` is `new; dup; bipush 7; <init>; pop`).
    let control = presented_family(&LEGS[0], "SP");
    assert!(
        all_presented(news(&control, "withArg")),
        "the statement-position control did not recover: {:?}",
        news(&control, "withArg")
    );
}

/// The frozen counterexample of `refuse-unconsumed-construction-invokes` keeps its code and BCI.
#[test]
fn the_frozen_counterexample_keeps_the_interleaved_refusal() {
    let snapshot = open(&zip_of(&[(b"VoidBetween.class", VOID_BETWEEN)]));
    let report = presented(&snapshot, "VoidBetween");
    let (code, message) = refusal(news(&report, "make"), 0);
    assert_eq!(code, "jre_new_interleaved_effect");
    assert!(
        message.contains(INTERLEAVED_REFUSAL) && message.contains("at BCI 8"),
        "the frozen counterexample's refusal moved: {message}"
    );
    let make = method_body(&report.text, "make()LTarget;");
    assert!(
        make.contains("Side.effect()"),
        "the frozen counterexample's call left the text:\n{make}"
    );
}

/// The negatives keep the refusal text they had: an invocation, a field read and an arithmetic
/// chain at an argument position, plus the registered `chained`.
#[test]
fn the_negatives_keep_their_refusals_on_both_legs() {
    for leg in LEGS {
        let report = presented_family(leg, "SPN");
        for (signature, code, phrases) in SPN_REFUSALS {
            let name = signature.split('(').next().expect("a signature has a name");
            let (refusal_code, message) = refusal(news(&report, name), 0);
            assert_eq!(refusal_code, *code, "`SPN.{signature}` on {}", leg.label);
            for phrase in *phrases {
                assert!(
                    message.contains(phrase),
                    "`SPN.{signature}` on {} moved: {message}",
                    leg.label
                );
            }
            let body = method_body(&report.text, signature);
            assert!(
                body.contains("jarde_refused_body();"),
                "`SPN.{signature}` on {} is presented as recovered:\n{body}",
                leg.label
            );
        }
        // The consumed-position control: the static initializer's own construction is presented.
        let held = news(&report, "<clinit>");
        assert!(
            all_presented(held),
            "`SPN.<clinit>` on {} lost its construction: {held:?}",
            leg.label
        );
        for (class, signature, expected) in CONSUMED_METHODS.iter().filter(|(c, _, _)| *c == "SPN")
        {
            let body = method_body(&report.text, signature);
            assert!(
                body.contains(expected),
                "`{class}.{signature}` on {} moved: expected {expected:?}:\n{body}",
                leg.label
            );
        }
    }
}

/// Every recovering method presents every instruction it has: an anchor that still quotes one
/// would be a partial recovery counted as one.
#[test]
fn no_recovering_method_keeps_a_quote() {
    for leg in LEGS {
        for class in ["SP", "SB"] {
            let report = presented_family(leg, class);
            for &(owner, signature) in NO_QUOTE_METHODS.iter().filter(|(owner, _)| *owner == class)
            {
                let body = method_body(&report.text, signature);
                let quoted = quoted_bcis(body);
                assert!(
                    quoted.is_empty(),
                    "`{owner}.{signature}` on {} still quotes {quoted:?}:\n{body}",
                    leg.label
                );
                assert!(
                    !body.contains("not recovered"),
                    "`{owner}.{signature}` on {} is not a whole recovery:\n{body}",
                    leg.label
                );
                assert!(
                    !body.contains(UNCONSUMED_REFUSAL),
                    "`{owner}.{signature}` on {} still carries the statement-position refusal:\n{body}",
                    leg.label
                );
            }
        }
    }
    // The patrol's own two classes, on the frozen bytes they were committed as.
    let b5 = presented(
        &open(&zip_of(&[
            (b"B5.class", PATROL_B5),
            (b"B5$Sub.class", PATROL_B5_SUB),
        ])),
        "B5",
    );
    let b6 = presented(&open(&zip_of(&[(b"B6.class", PATROL_B6)])), "B6");
    for &(owner, signature) in NO_QUOTE_METHODS
        .iter()
        .filter(|(owner, _)| *owner == "B5" || *owner == "B6")
    {
        let report = if owner == "B5" { &b5 } else { &b6 };
        let body = method_body(&report.text, signature);
        let quoted = quoted_bcis(body);
        assert!(
            quoted.is_empty(),
            "`{owner}.{signature}` still quotes {quoted:?}:\n{body}"
        );
        assert!(
            !body.contains("not recovered"),
            "`{owner}.{signature}` is not a whole recovery:\n{body}"
        );
    }
}

// -------------------------------------------------------------------------------------------
// The replay: both compiler legs, real execution, the fixtures' own answers.
// -------------------------------------------------------------------------------------------

/// A scratch directory under the target tree.
fn scratch(label: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let ordinal = NEXT.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "jarde-statement-position-news-{label}-{stamp}-{ordinal}"
    ));
    fs::create_dir_all(&path).expect("the scratch directory is created");
    path
}

/// The comment-stripped text one class's presentation becomes.
fn stripped(report: &ClassSourceReport) -> String {
    report
        .text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Run one class under `-Xverify:all`, answering its standard output and exit status.
fn run_class(runner: &str, dir: &Path, class: &str) -> String {
    let run = Command::new(runner)
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(class)
        .output()
        .expect("the JVM runs");
    format!(
        "{}\nstatus={}",
        String::from_utf8_lossy(&run.stdout).trim_end(),
        run.status.code().unwrap_or(-1)
    )
}

/// Compile one stripped presentation with one compiler and run it under `-Xverify:all`.
///
/// The presentation folds the member classes it presents into the same text (a static nested class
/// is declared inside it, and every reference is written in that nesting), so the compile needs no
/// dependency classpath: the class is one unit.
fn compile_and_run(compiler: &str, runner: &str, release: bool, dir: &Path, class: &str) -> String {
    let source = dir.join(format!("{class}.java"));
    let mut command = Command::new(compiler);
    if release {
        command.arg("--release").arg("8");
    }
    let output = command
        .arg("-nowarn")
        .arg("-d")
        .arg(dir)
        .arg(&source)
        .output()
        .expect("the compiler runs");
    assert!(
        output.status.success(),
        "{class}: javac failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = Command::new(runner)
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(class)
        .output()
        .expect("the JVM runs");
    format!(
        "{}\nstatus={}",
        String::from_utf8_lossy(&run.stdout).trim_end(),
        run.status.code().unwrap_or(-1)
    )
}

/// One class of one presentation, compiled on both compilers and compared with the fixture's own
/// answer.
fn replay_one(label: &str, class: &str, text: &str, want: &str) {
    let javac8 =
        Path::new("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac");
    let java8 =
        Path::new("/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java");
    let work = scratch(&format!("recovered-{class}"));
    fs::write(work.join(format!("{class}.java")), text).expect("the presentation is written");
    let got = compile_and_run("/usr/bin/javac", "/usr/bin/java", true, &work, class);
    assert_eq!(
        got, want,
        "`{class}` on {label} answers differently after the round trip:\n{text}"
    );
    if javac8.is_file() {
        let work8 = scratch(&format!("recovered-8-{class}"));
        fs::write(work8.join(format!("{class}.java")), text).expect("the presentation is written");
        let got8 = compile_and_run(
            javac8.to_str().expect("the path is UTF-8"),
            java8.to_str().expect("the path is UTF-8"),
            false,
            &work8,
            class,
        );
        assert_eq!(
            got8, want,
            "`{class}` on {label} answers differently under javac 8:\n{text}"
        );
    }
}

/// One class whose body a refusal keeps quoted: its stripped text must **not** compile.
fn assert_refused_text_does_not_compile(label: &str, class: &str, text: &str) {
    let work = scratch(&format!("refused-{class}"));
    fs::write(work.join(format!("{class}.java")), text).expect("the presentation is written");
    let compiled = Command::new("/usr/bin/javac")
        .arg("--release")
        .arg("8")
        .arg("-nowarn")
        .arg("-d")
        .arg(&work)
        .arg(work.join(format!("{class}.java")))
        .output()
        .expect("the compiler runs");
    assert!(
        !compiled.status.success(),
        "`{class}` on {label} keeps a refused body and its stripped text must not compile:\n{text}"
    );
}

#[test]
#[ignore = "compiles and runs the recovered text; needs the installed JDK (and javac 8 for the second leg)"]
fn the_recovered_text_compiles_and_runs_identically_on_both_legs() {
    // The patrol's composite: the whole class, against the answer the original class files print.
    let b5 = presented(
        &open(&zip_of(&[
            (b"B5.class", PATROL_B5),
            (b"B5$Sub.class", PATROL_B5_SUB),
        ])),
        "B5",
    );
    let text = stripped(&b5);
    assert!(
        !text.contains("jarde_refused_body"),
        "the frozen `B5` still holds a refused body marker:\n{text}"
    );
    let want_dir = scratch("original-B5");
    fs::write(want_dir.join("B5.class"), PATROL_B5).expect("the fixture class is written");
    fs::write(want_dir.join("B5$Sub.class"), PATROL_B5_SUB).expect("the fixture class is written");
    let want = run_class("/usr/bin/java", &want_dir, "B5");
    assert_eq!(
        want,
        format!(
            "{}\nstatus=0",
            String::from_utf8_lossy(PATROL_ORIG_OUT).trim_end()
        ),
        "the frozen `B5` prints its committed answer"
    );
    replay_one("the patrol's frozen B5", "B5", &text, &want);

    // The change's own classes, on both legs: the whole class, against its own class files.
    for leg in LEGS {
        for class in ["SP", "SB"] {
            let report = presented_family(leg, class);
            let text = stripped(&report);
            assert!(
                !text.contains("jarde_refused_body"),
                "`{class}` on {} still holds a refused body marker:\n{text}",
                leg.label
            );
            let want_dir = scratch(&format!("original-{class}"));
            for (name, bytes) in leg.family(class) {
                fs::write(want_dir.join(name), bytes).expect("the fixture class is written");
            }
            let want = run_class("/usr/bin/java", &want_dir, class);
            replay_one(leg.label, class, &text, &want);
        }
        // The two classes a refusal keeps incomplete: their text must not compile, which is the
        // safe form the soundness invariant asks for — never a method that compiles and behaves
        // differently. `B6.chained` is the registered boundary; `SPN`'s four negatives are the
        // argument-effect refusals.
        for class in ["B6", "SPN"] {
            let report = if class == "B6" {
                presented(&open(&zip_of(&[(b"B6.class", PATROL_B6)])), class)
            } else {
                presented_family(leg, class)
            };
            let text = stripped(&report);
            assert_refused_text_does_not_compile(leg.label, class, &text);
        }
    }

    // The statement-position counterexample: the original class answers `CST` under
    // `-Xverify:all`, and its recovered text is quoted, so it must not compile either.
    let work = scratch("counterexample");
    let support = work.join("out");
    fs::create_dir_all(&support).expect("the support directory is created");
    fs::write(support.join("SPC.class"), SPC).expect("the assembled class is written");
    for (name, source) in [
        (
            "Side.java",
            include_str!("fixtures/recover-statement-position-news/spc/Side.java"),
        ),
        (
            "Target.java",
            include_str!("fixtures/recover-statement-position-news/spc/Target.java"),
        ),
        (
            "Trace.java",
            include_str!("fixtures/recover-statement-position-news/spc/Trace.java"),
        ),
        (
            "SPCRunner.java",
            include_str!("fixtures/recover-statement-position-news/spc/SPCRunner.java"),
        ),
    ] {
        fs::write(work.join(name), source).expect("the support source is written");
    }
    let compiled = Command::new("/usr/bin/javac")
        .arg("--release")
        .arg("8")
        .arg("-nowarn")
        .arg("-cp")
        .arg(&support)
        .arg("-d")
        .arg(&support)
        .arg(work.join("Side.java"))
        .arg(work.join("Target.java"))
        .arg(work.join("Trace.java"))
        .arg(work.join("SPCRunner.java"))
        .output()
        .expect("the compiler runs");
    assert!(
        compiled.status.success(),
        "the counterexample's support family compiles: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let original = run_class("/usr/bin/java", &support, "SPCRunner");
    assert_eq!(
        original, "CST\nstatus=0",
        "the original statement-position counterexample answers its class-initialization order"
    );
    let spc = presented(&open(&zip_of(&[(b"SPC.class", SPC)])), "SPC");
    assert_refused_text_does_not_compile("the assembled SPC", "SPC", &stripped(&spc));
}
