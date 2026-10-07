//! `preserve-local-scope-across-exception-regions` 2.1/2.2/2.4/2.5: the refusal closure of the
//! declaration plan, the boundary it leaves, the stops that never publish a half body, and the
//! audit of the crossings that stay refused.
//!
//! The declaration planner refuses a local whose definition–use slice crosses a region the run
//! quotes whole ([`crates/jarde-java/src/build.rs`]'s `DeclarationPlacement::Incomplete`). What
//! 2.1 pins is that the refusal is *complete*: the quote names every block of the dependent slice —
//! the definition, the handler, the join or transfer and the consumer outside the region — the
//! source map anchors exactly those blocks, and no statement that would read a name declared in a
//! quoted region is written. The fixtures are
//! `tests/fixtures/preserve-local-scope-refusals/` (see its README for the sources, both compiler
//! legs, the digests and the recorded behavior) and the documented escape control beside them.
//!
//! 2.2 is the boundary the same closure must keep: an independent member is still presented whole,
//! and a member whose normal-path statements are written keeps only the blocks it cannot claim
//! quoted. 2.4 is the stop contract at the points this layer charges: the analysis charges of the
//! scope plan and its validation (anchored at the local's own accesses) and the output charge of
//! the refusal text — every stop answers `Stopped`/`NotProduced` with no text and no segment table,
//! never a half-written `try`.
//!
//! 2.5 is the audit of the crossings that stay refused: `p3_try_local`'s mutated resource copy and
//! `p3_typed_catch`'s catch-all row. The test below traces, per fixture, whether the quoted
//! fallback is on the refused local's own slice (it is: the close's read sits inside the quoted
//! region) or belongs to a different local identity (it does: the handler copy's slot is its own
//! lifetime, and the member is presented with only the unreachable block quoted).

use jarde::*;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const REFUSALS_V8: &[u8] =
    include_bytes!("fixtures/preserve-local-scope-refusals/v8/ScopeRefusals.class");
const REFUSALS_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/preserve-local-scope-refusals/v8-javac8/ScopeRefusals.class");
const ESCAPE_V8: &[u8] =
    include_bytes!("fixtures/preserve-local-scope-refusals/v8/ScopeRefusalsEscape.class");
const ESCAPE_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/preserve-local-scope-refusals/v8-javac8/ScopeRefusalsEscape.class");
/// The control `patch-escape.py` derives from the `v8` class: the handler's entry store and load
/// address the method local's slot, so the caught value is read after the clause.
const ESCAPE_DERIVED: &[u8] =
    include_bytes!("fixtures/preserve-local-scope-refusals/escaped/ScopeRefusalsEscape.class");

/// The 2.5 audit's two inputs, read exactly as their own test files read them.
const TRY_LOCAL_V9: &[u8] = include_bytes!("fixtures/p3-try-local/v9/Held.class");
const TYPED_CATCH_V8: &[u8] = include_bytes!("fixtures/p3-typed-catch/v8/TypedCatch.class");

/// The 2.3 comparison's inputs.
const PLAN_V8: &[u8] = include_bytes!("fixtures/preserve-local-scope-plan/v8/ScopePlan.class");
const PLAN_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/preserve-local-scope-plan/v8-javac8/ScopePlan.class");
const PLAN_JADX: &str = include_str!("fixtures/preserve-local-scope-plan/jadx/ScopePlan.java");
const PLAN_DRIVER: &str = include_str!("fixtures/preserve-local-scope-plan/ScopePlanDriver.java");
const LOOP_V8: &[u8] =
    include_bytes!("fixtures/p3-loop-try-handler-entry/v8/LoopTryHandlerEntryArgs.class");
const LOOP_JADX: &str =
    include_str!("fixtures/p3-loop-try-handler-entry/LoopTryHandlerEntryArgs.jadx.java.txt");
const LOOP_RUNNER: &str = include_str!("fixtures/p3-loop-try-handler-entry/Runner.java");

/// The sentence every fallback-crossing local keeps, verbatim.
const CROSSES_FALLBACK: &str = "local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice";
/// The sentence an incomplete exception edge keeps, verbatim.
const CROSSES_PROTECTED: &str = "local 1 crosses a protected region, but SSA does not prove that every path to its reads reaches a presented write";
/// The sentence the escaped shared handler keeps, verbatim.
const ESCAPES_CLAUSE: &str = "local 1 escapes catch parameter scope at region [0, 1]; its catch header cannot declare a method-visible local";
/// The real javac 8 of this repository's fixture protocol (Corretto 1.8.0_432).
const CORRETTO_HOME: &str = "/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home";

/// One refused member's closure, as the class file's own bytes state it.
///
/// `blocks` are the half-open instruction spans of the quoted blocks, read from the fixture's own
/// `javap` output (recorded in the fixture README); they are what turns "the quote names these
/// blocks" into "the quote covers the definition, the handler, the join and the consumer".
struct Closure {
    member: &'static str,
    sentence: &'static str,
    quote: &'static str,
    fallbacks: &'static [&'static str],
    /// `(region bci, block bcis, fallback code)` per region record, in the report's own order.
    regions: &'static [(u32, &'static [u32], Option<&'static str>)],
    /// `(block bci, first instruction bci, one past the last)` per quoted block.
    blocks: &'static [(u32, u32, u32)],
    /// The slice's own instructions: the definition, the handler entry, the join/transfer and the
    /// consumer outside the region.
    slice: &'static [(&'static str, u32)],
}

const SAVED: Closure = Closure {
    member: "savedAcrossFinally",
    sentence: CROSSES_FALLBACK,
    quote: "// @bytecode 0 17 22",
    fallbacks: &["jre_region_exception_edge", "jre_region_uncovered_blocks"],
    regions: &[
        (0, &[0], Some("jre_region_exception_edge")),
        (22, &[22, 17], Some("jre_region_uncovered_blocks")),
    ],
    // `istore_1` at 1 and the normal copy's `iload_1` at 12; the handler copy's read at 18; the
    // consumer `iload_0` at 22.
    blocks: &[(0, 0, 16), (17, 17, 21), (22, 22, 23)],
    slice: &[
        ("definition", 1),
        ("normal read", 12),
        ("handler read", 18),
        ("consumer", 22),
    ],
};

const COMPUTED: Closure = Closure {
    member: "handlerComputed",
    sentence: CROSSES_FALLBACK,
    quote: "// @bytecode 0 8 16 20 22 23",
    fallbacks: &["jre_region_uncovered_blocks"],
    regions: &[
        (0, &[0, 8, 16, 20], None),
        (23, &[23], None),
        (22, &[22], Some("jre_region_uncovered_blocks")),
    ],
    // The definition `istore_1` at 4; the handler entry `astore_2` at 8, `getMessage` at 10 and the
    // test at 13; the conditional's arms at 16 and 20; the join's `istore_1` at 22 — the uncovered
    // block; the consumer `iload_1` at 23.
    blocks: &[
        (0, 0, 7),
        (8, 8, 15),
        (16, 16, 19),
        (20, 20, 21),
        (22, 22, 23),
        (23, 23, 24),
    ],
    slice: &[
        ("definition", 4),
        ("handler", 8),
        ("arm", 16),
        ("arm", 20),
        ("join", 22),
        ("consumer", 23),
    ],
};

const NESTED: Closure = Closure {
    member: "nestedHandler",
    sentence: CROSSES_PROTECTED,
    quote: "// @bytecode 0 10 13 16 20",
    fallbacks: &[],
    regions: &[(0, &[0, 10, 16], None), (13, &[13, 20], None)],
    // The definition `istore_1` at 1; the inner handler entry `astore_2` at 10; the transfer
    // `goto 20` at 13; the outer handler entry `astore_2` at 16; the consumer `iload_1` at 20.
    blocks: &[
        (0, 0, 9),
        (10, 10, 12),
        (13, 13, 15),
        (16, 16, 19),
        (20, 20, 21),
    ],
    slice: &[
        ("definition", 1),
        ("inner handler", 10),
        ("transfer", 13),
        ("outer handler", 16),
        ("consumer", 20),
    ],
};

const SHARED: Closure = Closure {
    member: "sharedHandler",
    sentence: CROSSES_PROTECTED,
    quote: "// @bytecode 0 11 14",
    fallbacks: &[],
    regions: &[(0, &[0, 11], None), (14, &[14], None)],
    // The definition `astore_1` at 1 and the transfer `goto 14` at 8; the handler entry at 11; the
    // consumer `aload_1` at 14.
    blocks: &[(0, 0, 10), (11, 11, 13), (14, 14, 15)],
    slice: &[
        ("definition", 1),
        ("transfer", 8),
        ("handler", 11),
        ("consumer", 14),
    ],
};

const ESCAPED: Closure = Closure {
    member: "sharedHandler",
    sentence: ESCAPES_CLAUSE,
    quote: "// @bytecode 0 11 14",
    fallbacks: &[],
    regions: &[(0, &[0, 11], None), (14, &[14], None)],
    blocks: &[(0, 0, 10), (11, 11, 13), (14, 14, 15)],
    slice: &[
        ("definition", 1),
        ("transfer", 8),
        ("handler", 11),
        ("consumer", 14),
    ],
};

fn budget() -> Budget {
    Budget::new(task_limits(&[]).expect("the task defaults are a bounded budget"))
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of one committed sample, under the entry point the CLI calls.
fn class_source_of(snapshot: &ArtifactSnapshot, class: &str, release: u16) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: release,
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
            &RecoveryEvidenceRequest::essential()
                .with_kind(RecoveryEvidenceKind::SourceMap)
                .with_kind(RecoveryEvidenceKind::RegionDetails),
            &mut budget(),
        )
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

/// The member of one class, by the raw name its class file declares.
fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

/// The recovery report of one member's own run.
fn run_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` was read by its own run, got {other:?}"),
    }
}

/// The bytecode indices one refusal quotes, in the order its own line states them.
fn quoted_bcis(text: &str) -> BTreeSet<u32> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|bcis| bcis.split_whitespace())
        .filter_map(|bci| bci.parse::<u32>().ok())
        .collect()
}

/// Every bytecode index the source map's origins mention, primary and derived.
fn origin_bcis(run: &RecoveryReport) -> BTreeSet<u32> {
    let mut bcis = BTreeSet::new();
    for segment in run.source_map.segments() {
        bcis.insert(segment.origin().primary().bci());
        for derived in segment.origin().derived() {
            bcis.insert(derived.bci());
        }
    }
    bcis
}

/// The lines of one member's text that hold a statement: everything that is neither an envelope
/// comment, nor blank, nor the body's own braces.
fn statement_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && trimmed != "{" && trimmed != "}"
        })
        .collect()
}

/// Where one substring sits in one text, stated as the panic a missing one deserves.
fn at(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("the text presents `{needle}`:\n{text}"))
}

/// The bytecode index the narrowest segment covering one text offset is anchored at.
fn origin_of(run: &RecoveryReport, offset: usize) -> u32 {
    let mut covering: Vec<_> = run
        .source_map
        .segments()
        .iter()
        .filter(|segment| segment.start() <= offset && offset < segment.end())
        .collect();
    covering.sort_by_key(|segment| segment.len());
    let segment = covering
        .first()
        .unwrap_or_else(|| panic!("a segment covers offset {offset} of:\n{}", run.text));
    segment.origin().primary().bci()
}

/// The five member texts of one class, by member name.
fn texts_of(report: &ClassSourceReport) -> Vec<(String, String)> {
    report
        .methods
        .iter()
        .map(|method| {
            let name = String::from_utf8_lossy(&method.item.name.raw().0).into_owned();
            let text = match &method.outcome {
                ClassSourceOutcome::Recovered { report, .. } => report.text.clone(),
                ClassSourceOutcome::NoBody
                | ClassSourceOutcome::Unspelled
                | ClassSourceOutcome::Refused { .. } => String::new(),
            };
            (name, text)
        })
        .collect()
}

#[test]
fn the_refused_slices_keep_the_instructions_their_closure_covers() {
    // Both compiler legs and the derived control: the closure is a statement about control flow,
    // so the two legs must answer the same quotes, the same region records and the same origins.
    for (leg, bytes) in [("v8", REFUSALS_V8), ("v8-javac8", REFUSALS_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = class_source_of(&snapshot, "ScopeRefusals", 8);
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `ScopeRefusals`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        for closure in [&SAVED, &COMPUTED, &NESTED] {
            let run = run_of(&report, closure.member);
            let text = run.text.as_str();
            assert_eq!(
                run.content,
                RecoveryContent::ExplanationOnly,
                "{leg}/{}: the dependent slice is refused whole:\n{text}",
                closure.member
            );
            assert!(
                text.contains(closure.sentence) && text.contains(closure.quote),
                "{leg}/{}: the refusal keeps its own sentence and quote:\n{text}",
                closure.member
            );
            // The quote is the whole closure and nothing else: the blocks that hold the
            // definition, the handler, the join/transfer and the consumer.
            assert_eq!(
                quoted_bcis(text),
                closure
                    .blocks
                    .iter()
                    .map(|(bci, _, _)| *bci)
                    .collect::<BTreeSet<_>>(),
                "{leg}/{}: the refusal quotes exactly the dependent slice's blocks:\n{text}",
                closure.member
            );
            for (label, bci) in closure.slice {
                let covered = closure
                    .blocks
                    .iter()
                    .any(|(_, start, end)| (*start..*end).contains(bci));
                assert!(
                    covered,
                    "{leg}/{}: the {label} at BCI {bci} is not inside any quoted block \
                     {:#?}:\n{text}",
                    closure.member, closure.blocks
                );
            }
            // The run's own region records name the blocks and the fallback code behind the quote.
            let regions: Vec<(u32, Vec<u32>, Option<&str>)> = run
                .regions
                .iter()
                .map(|region| (region.bci, region.blocks.clone(), region.code))
                .collect();
            let expected: Vec<(u32, Vec<u32>, Option<&str>)> = closure
                .regions
                .iter()
                .map(|(bci, blocks, code)| (*bci, blocks.to_vec(), *code))
                .collect();
            assert_eq!(
                regions, expected,
                "{leg}/{}: the region records state the quoted blocks and the fallback \
                 codes:\n{text}",
                closure.member
            );
            assert_eq!(
                run.fallbacks, closure.fallbacks,
                "{leg}/{}: the run states the fallback codes it used:\n{text}",
                closure.member
            );
            // The source map anchors exactly the same blocks: the refused text's origins are the
            // closure's own, so a reader can locate every quoted instruction.
            assert_eq!(
                origin_bcis(run),
                closure
                    .blocks
                    .iter()
                    .map(|(bci, _, _)| *bci)
                    .collect::<BTreeSet<_>>(),
                "{leg}/{}: the refused text's origins are the quoted blocks:\n{text}",
                closure.member
            );
            // No statement is written: no read of a name declared inside a quoted region, and no
            // half `try` around one.
            assert!(
                statement_lines(text).is_empty(),
                "{leg}/{}: a refused member writes no statement:\n{text}",
                closure.member
            );
            assert!(
                !text.contains("local1")
                    || !statement_lines(text)
                        .iter()
                        .any(|line| line.contains("local1")),
                "{leg}/{}: no out-of-scope read of the refused local is written:\n{text}",
                closure.member
            );
        }
    }

    // The escape control: the same closure, with the sentence that states the binding's own
    // reason. The two unmutated legs keep the incomplete-edge answer; the derived class — whose
    // caught value is read after the clause — keeps the escape answer.
    for (leg, bytes, closure) in [
        ("v8", ESCAPE_V8, &SHARED),
        ("v8-javac8", ESCAPE_V8_JAVAC8, &SHARED),
        ("escaped", ESCAPE_DERIVED, &ESCAPED),
    ] {
        let snapshot = open(bytes);
        let report = class_source_of(&snapshot, "ScopeRefusalsEscape", 8);
        let run = run_of(&report, "sharedHandler");
        let text = run.text.as_str();
        assert!(
            text.contains(closure.sentence) && text.contains(closure.quote),
            "{leg}: the shared handler keeps its own sentence and quote:\n{text}"
        );
        assert_eq!(
            quoted_bcis(text),
            closure
                .blocks
                .iter()
                .map(|(bci, _, _)| *bci)
                .collect::<BTreeSet<_>>(),
            "{leg}: the shared handler's refusal quotes the whole slice:\n{text}"
        );
        for (label, bci) in closure.slice {
            let covered = closure
                .blocks
                .iter()
                .any(|(_, start, end)| (*start..*end).contains(bci));
            assert!(
                covered,
                "{leg}: the shared handler's {label} at BCI {bci} is inside a quoted block:\n{text}"
            );
        }
        assert_eq!(
            origin_bcis(run),
            closure
                .blocks
                .iter()
                .map(|(bci, _, _)| *bci)
                .collect::<BTreeSet<_>>(),
            "{leg}: the shared handler's origins are the quoted blocks:\n{text}"
        );
        assert!(
            statement_lines(text).is_empty(),
            "{leg}: the shared handler's refusal writes no statement:\n{text}"
        );
    }
}

#[test]
fn the_boundary_keeps_the_independent_members_and_quotes_only_the_dependent_slice() {
    for (leg, bytes) in [("v8", REFUSALS_V8), ("v8-javac8", REFUSALS_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = class_source_of(&snapshot, "ScopeRefusals", 8);

        // The independent sibling: no local of this member crosses a quoted region, so it is
        // presented whole — the refusal beside it does not leak into a member that does not
        // depend on the refused local.
        let run = run_of(&report, "siblingKept");
        let text = run.text.as_str();
        assert_eq!(
            run.content,
            RecoveryContent::ContainsStatements,
            "{leg}/siblingKept: the independent sibling is presented whole:\n{text}"
        );
        assert!(run.fallbacks.is_empty(), "{leg}/siblingKept:\n{text}");
        assert!(!text.contains("@bytecode"), "{leg}/siblingKept:\n{text}");
        let declaration = at(text, "int local1;");
        let first = at(text, "local1 = arg0;");
        let protected = at(text, "local1 = local1 + 1;");
        let clause = at(text, "catch (java.lang.RuntimeException local2)");
        let handler = at(text, "local1 = -1;");
        let read = at(text, "return local1;");
        assert!(
            declaration < first
                && first < protected
                && protected < clause
                && clause < handler
                && handler < read,
            "{leg}/siblingKept: the lifted declaration covers both writes and the join:\n{text}"
        );

        // The member-level boundary: the normal path's statements are written and the handler copy
        // the walk cannot claim stays one quoted block. The quoted slice is refused whole — its
        // quote names its own blocks and its own fallback code — while every statement that does
        // not depend on it is kept, and the source map locates both sides.
        let run = run_of(&report, "quotedSliceKept");
        let text = run.text.as_str();
        assert_eq!(
            run.content,
            RecoveryContent::ContainsStatements,
            "{leg}/quotedSliceKept: the normal path is presented:\n{text}"
        );
        assert_eq!(
            run.quality,
            Quality::Fallback,
            "{leg}/quotedSliceKept:\n{text}"
        );
        assert_eq!(
            run.fallbacks,
            vec!["jre_region_uncovered_blocks"],
            "{leg}/quotedSliceKept: the dependent slice states its own code:\n{text}"
        );
        assert_eq!(
            quoted_bcis(text),
            [19, 20, 21, 22, 23, 24, 25].into_iter().collect(),
            "{leg}/quotedSliceKept: the quote names the blocks the walk could not claim:\n{text}"
        );
        assert!(
            text.contains("1 live block(s) are reachable only through edges the normal-flow view leaves out: [19]"),
            "{leg}/quotedSliceKept: the quote states why the block is refused:\n{text}"
        );
        for statement in [
            "int local1;",
            "local1 = arg0;",
            "if (arg0 > 0) {",
            "int local2 = arg0;",
            "local1 = local1 + local2;",
            "local2 = local2 - 1;",
            "return local1;",
        ] {
            assert!(
                text.contains(statement),
                "{leg}/quotedSliceKept: the independent statements are kept (`{statement}`):\n{text}"
            );
        }
        // The source map locates the presented statements at their own instructions and the quoted
        // block at the copy's: the two sides are separately readable.
        assert_eq!(
            origin_of(run, at(text, "local1 = local1 + local2;")),
            11,
            "{leg}/quotedSliceKept:\n{text}"
        );
        assert_eq!(
            origin_of(run, at(text, "local2 = local2 - 1;")),
            15,
            "{leg}/quotedSliceKept:\n{text}"
        );
        let quoted_segment = run
            .source_map
            .segments()
            .iter()
            .find(|segment| segment.text(text).contains("@bytecode 19"))
            .unwrap_or_else(|| {
                panic!("{leg}/quotedSliceKept: the quote has its own segment:\n{text}")
            });
        assert_eq!(
            quoted_segment.origin().primary().bci(),
            19,
            "{leg}/quotedSliceKept:\n{text}"
        );

        // The class text carries both answers: the refused members keep their sentences and the
        // independent members their statements, in one presentation.
        for sentence in [CROSSES_FALLBACK, CROSSES_PROTECTED] {
            assert!(
                report.text.contains(sentence),
                "{leg}: the class text keeps the refusal `{sentence}`"
            );
        }
        assert!(
            report.text.contains("return local1;") && report.text.contains("local1 = -1;"),
            "{leg}: the class text keeps the independent members' statements:\n{}",
            report.text
        );
    }
}

/// One member's own state in one class-source answer: whether it produced text, and its outcome.
fn member_states(
    snapshot: &ArtifactSnapshot,
    request: &ClassSourceRequest,
    limit: u64,
) -> Vec<(String, bool, RecoveryOutcome)> {
    let mut limited = Budget::new(
        task_limits(&[BudgetOverride::new("analysis_steps", limit).expect("a positive bound")])
            .expect("the bounded task limits"),
    );
    match Engine::new()
        .class_source(slice::from_ref(snapshot), &request, &mut limited)
        .expect("a bounded class-source request is answered")
    {
        OperationOutcome::Performed(report) => report
            .methods
            .iter()
            .map(|method| {
                let name = String::from_utf8_lossy(&method.item.name.raw().0).into_owned();
                match &method.outcome {
                    ClassSourceOutcome::Recovered { report, .. } => {
                        (name, !report.text.is_empty(), report.outcome.clone())
                    }
                    _ => (
                        name,
                        false,
                        RecoveryOutcome::Stopped(StopReason::Cancelled { at: None }),
                    ),
                }
            })
            .collect(),
        other => panic!("the sweep answers one class, got {other:?}"),
    }
}

/// One class-source request for one committed sample.
fn class_request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
    }
}

#[test]
fn the_scope_planning_and_validation_stops_commit_no_partial_body() {
    // The scope plan's own charges — the local accesses it reads, decides and validates — are
    // anchored at the member's own instructions, so a sweep of the run's `analysis_steps` bound
    // stops inside the plan: the boundary below is the last stop before each member completes, and
    // its position is the member's first local access. Every stop answers the same contract: the
    // run is `Stopped`, its content is `not_produced`, and no text or segment table is committed.
    let snapshot = open(PLAN_V8);
    let request = class_request(&snapshot, "ScopePlan");
    let reference = class_source_of(&snapshot, "ScopePlan", 8);
    let total = reference.usage.analysis_steps;
    assert!(
        total > 0,
        "the positive fixture's run charges the analysis dimension"
    );

    // `(member, first access bci)` — the anchor the plan's last charge names before the member
    // completes, read from the member's own bytecode: `catchOnly`'s handler binding at 9,
    // `assignedAcrossTry`'s protected write at 7, `assignedAcrossIf`'s then-arm write at 5,
    // `nestedHandlerOnly`'s inner body at 19, `nestedAcross`'s innermost write at 8.
    let directed = [
        ("catchOnly", 9u32),
        ("assignedAcrossTry", 7),
        ("assignedAcrossIf", 5),
        ("nestedHandlerOnly", 19),
        ("nestedAcross", 8),
    ];
    for (name, anchor) in directed {
        let mut first_complete = None;
        for limit in 1..=total {
            let states = member_states(&snapshot, &request, limit);
            let Some(state) = states.iter().find(|(member, _, _)| member == name) else {
                continue;
            };
            if state.1 {
                first_complete = Some(limit);
                break;
            }
        }
        let complete = first_complete.unwrap_or_else(|| panic!("{name} completes within {total}"));
        assert!(
            complete > 1,
            "{name}: the member is charged more than one step"
        );
        let states = member_states(&snapshot, &request, complete - 1);
        let (_, produced, outcome) = states
            .iter()
            .find(|(member, _, _)| member == name)
            .expect("the member is in the class report");
        assert!(!produced, "{name}: the last stop commits no text");
        match outcome {
            RecoveryOutcome::Stopped(StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                at: Some(at),
                ..
            }) => assert_eq!(
                *at, anchor,
                "{name}: the plan's last charge before the member completes is anchored at its \
                 first local access"
            ),
            other => panic!("{name}: expected a stop in the scope plan, got {other:?}"),
        }
        // The stop's own report: no half body and no segment table.
        let mut limited = Budget::new(
            task_limits(&[BudgetOverride::new("analysis_steps", complete - 1).expect("a bound")])
                .expect("the bounded task limits"),
        );
        let stopped = match Engine::new()
            .class_source(slice::from_ref(&snapshot), &request, &mut limited)
            .expect("a bounded class-source request is answered")
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("{name}: the class is still presented, got {other:?}"),
        };
        let stopped_member = run_of(&stopped, name);
        assert_eq!(
            stopped_member.content,
            RecoveryContent::NotProduced,
            "{name}: a stop states no content"
        );
        assert_eq!(stopped_member.text, "", "{name}: a stop hands out no text");
        assert_eq!(
            stopped_member.source_map.len(),
            0,
            "{name}: a stop hands out no segment table"
        );
        assert!(
            stopped_member.regions.is_empty() && stopped_member.fallbacks.is_empty(),
            "{name}: a stop claims no region"
        );
        // At the boundary the member is presented whole: the same text the ample budget answers.
        let states = member_states(&snapshot, &request, complete);
        assert!(
            states
                .iter()
                .find(|(member, _, _)| member == name)
                .expect("member")
                .1,
            "{name}: the member completes at the boundary"
        );
        let completed = class_source_of(&snapshot, "ScopePlan", 8);
        let _ = completed;
    }

    // The whole sweep, both legs: every answer is the reference text or a stop — never a member
    // presented in part — and any stop makes the class execution non-complete.
    for (leg, bytes) in [("v8", PLAN_V8), ("v8-javac8", PLAN_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let request = class_request(&snapshot, "ScopePlan");
        let reference = class_source_of(&snapshot, "ScopePlan", 8);
        let reference_texts = texts_of(&reference);
        for limit in 1..=reference.usage.analysis_steps {
            let mut limited = Budget::new(
                task_limits(&[BudgetOverride::new("analysis_steps", limit).expect("a bound")])
                    .expect("the bounded task limits"),
            );
            let report = match Engine::new()
                .class_source(slice::from_ref(&snapshot), &request, &mut limited)
                .expect("a bounded class-source request is answered")
            {
                OperationOutcome::Performed(report) => report,
                other => panic!("{leg}: limit {limit}: the class is presented, got {other:?}"),
            };
            let mut stopped = 0usize;
            for (member, (name, text)) in report.methods.iter().zip(&reference_texts) {
                let ClassSourceOutcome::Recovered { report, .. } = &member.outcome else {
                    continue;
                };
                if report.text.is_empty() {
                    stopped += 1;
                    assert_eq!(
                        report.content,
                        RecoveryContent::NotProduced,
                        "{leg}/{name}: limit {limit}: a stopped member states no content"
                    );
                    assert_eq!(
                        report.source_map.len(),
                        0,
                        "{leg}/{name}: limit {limit}: a stopped member hands out no segment table"
                    );
                } else {
                    assert_eq!(
                        &report.text, text,
                        "{leg}/{name}: limit {limit}: a presented member is the whole member"
                    );
                }
            }
            if stopped > 0 {
                assert!(
                    !matches!(report.execution, ExecutionReport::Complete { .. }),
                    "{leg}: limit {limit}: a stopped member makes the class execution non-complete"
                );
            }
        }
    }

    // The refusal closure's own output charge: the refused members' text (the quote lines) is the
    // emitter's write, so a bound below it stops the run — with no partial quote committed.
    let snapshot = open(REFUSALS_V8);
    let request = class_request(&snapshot, "ScopeRefusals");
    let reference = class_source_of(&snapshot, "ScopeRefusals", 8);
    let reference_texts = texts_of(&reference);
    let total = reference.usage.output_bytes;
    assert!(total > 0, "the refusals fixture writes text");
    let mut saw_output_stop = false;
    for limit in 1..=total {
        let mut limited = Budget::new(
            task_limits(&[BudgetOverride::new("output_bytes", limit).expect("a bound")])
                .expect("the bounded task limits"),
        );
        let report = match Engine::new()
            .class_source(slice::from_ref(&snapshot), &request, &mut limited)
            .expect("a bounded class-source request is answered")
        {
            OperationOutcome::Performed(report) => report,
            // A bound below the class envelope's own write stops the request before any member is
            // presented: no member text exists to be partial.
            OperationOutcome::Incomplete(selection) => {
                assert!(
                    matches!(
                        selection.execution,
                        ExecutionReport::Partial {
                            reason: TerminationReason::BudgetExceeded {
                                dimension: BudgetDimension::OutputBytes
                            },
                            ..
                        }
                    ),
                    "limit {limit}: the request stops on its own output charge"
                );
                continue;
            }
            other => panic!("limit {limit}: the class is presented, got {other:?}"),
        };
        for (member, (name, text)) in report.methods.iter().zip(&reference_texts) {
            let ClassSourceOutcome::Recovered { report, .. } = &member.outcome else {
                continue;
            };
            if report.text.is_empty() {
                if let RecoveryOutcome::Stopped(StopReason::Budget {
                    dimension: CountedBudgetDimension::OutputBytes,
                    ..
                }) = &report.outcome
                {
                    saw_output_stop = true;
                }
                assert_eq!(
                    report.content,
                    RecoveryContent::NotProduced,
                    "{name}/{limit}"
                );
                assert_eq!(report.source_map.len(), 0, "{name}/{limit}");
            } else {
                assert_eq!(
                    &report.text, text,
                    "{name}: limit {limit}: a refused member's quote is committed whole or not at all"
                );
            }
        }
    }
    assert!(
        saw_output_stop,
        "the sweep below the output charge reaches the refusal closure's own write"
    );

    // The caller's cancellation is the same contract at the same points: no half `try` and no
    // segment table are committed.
    for (leg, bytes, class) in [
        ("v8", REFUSALS_V8, "ScopeRefusals"),
        ("v8", PLAN_V8, "ScopePlan"),
    ] {
        let snapshot = open(bytes);
        let request = class_request(&snapshot, class);
        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(
            task_limits(&[]).expect("the bounded task limits"),
            token,
        );
        let outcome = Engine::new()
            .class_source(slice::from_ref(&snapshot), &request, &mut cancelled)
            .expect("a cancelled request reports an outcome");
        match outcome {
            OperationOutcome::Incomplete(selection) => assert!(
                matches!(selection.execution, ExecutionReport::Cancelled { .. }),
                "{leg}/{class}: the stop states the cancellation"
            ),
            OperationOutcome::Performed(report) => {
                assert!(matches!(
                    report.execution,
                    ExecutionReport::Cancelled { .. }
                ));
                for member in &report.methods {
                    if let ClassSourceOutcome::Recovered { report, .. } = &member.outcome {
                        assert!(
                            report.text.is_empty(),
                            "{leg}/{class}: a cancelled run commits no member text"
                        );
                    }
                }
            }
            OperationOutcome::Ambiguous(_) => panic!("a fixed class identity is unambiguous"),
        }
    }
}

#[test]
fn the_crossing_fallbacks_belong_to_the_slices_they_refuse() {
    // 2.5's audit, per fixture. The question is ownership: when a refusal says a local crosses a
    // quoted fallback, is the fallback on that local's own definition–use slice, or is the refusal
    // wider than the slice it names?

    // `p3_try_local`'s mutated resource copy: the one-byte handler change (`ifnull` → `ifnonnull`)
    // fails the close proof, so the handler's close blocks stay quoted. The copy's own read is the
    // `aload_1` of the close at BCI 11 — inside the quoted block [11, 15) — and its definition
    // (`astore_1` at 1) and the consumer after the range (BCI 22) are in the other quoted block
    // [0, 17). The fallback is therefore on the copy's slice, and the whole refusal is the slice's
    // own closure; nothing here may be narrowed without presenting the close the proof refused.
    let needle = [0x2b, 0xc6, 0x00, 0x10];
    let replacement = [0x2b, 0xc7, 0x00, 0x10];
    let sites: Vec<_> = TRY_LOCAL_V9
        .windows(needle.len())
        .enumerate()
        .filter_map(|(at, bytes)| (bytes == needle).then_some(at))
        .collect();
    assert_eq!(sites.len(), 1, "the frozen class has one handler test");
    let mut mutated = TRY_LOCAL_V9.to_vec();
    mutated[sites[0]..sites[0] + needle.len()].copy_from_slice(&replacement);
    let snapshot = open(&mutated);
    let report = class_source_of(&snapshot, "Held", 9);
    let run = run_of(&report, "use");
    let text = run.text.as_str();
    assert!(text.contains(CROSSES_FALLBACK), "{text}");
    assert_eq!(
        quoted_bcis(text),
        [0, 11, 15, 17, 22, 29, 35].into_iter().collect(),
        "{text}"
    );
    assert_eq!(
        run.fallbacks,
        vec!["jre_region_exception_edge", "jre_region_uncovered_blocks"],
        "{text}"
    );
    // The definition, the close's read and the consumer are all inside the quoted blocks.
    for (label, bci, block) in [
        ("definition", 1u32, (0u32, 11u32)),
        ("close read", 11, (11, 15)),
        ("consumer", 22, (22, 29)),
    ] {
        assert!(
            (block.0..block.1).contains(&bci),
            "the {label} at BCI {bci} sits in the quoted block {block:?}:\n{text}"
        );
    }
    assert_eq!(
        run.regions
            .iter()
            .map(|region| (region.bci, region.blocks.clone(), region.code))
            .collect::<Vec<_>>(),
        vec![
            (0, vec![0, 17, 22, 29], Some("jre_region_exception_edge")),
            (11, vec![11, 15, 35], Some("jre_region_uncovered_blocks")),
        ],
        "{text}"
    );

    // The positive copy on the same fixture: no fallback is left, so the same local's slice is
    // presented whole as the resource header. The audit's contrast case: the refusal above is the
    // mutation's own, not a standing property of the shape.
    let snapshot = open(TRY_LOCAL_V9);
    let report = class_source_of(&snapshot, "Held", 9);
    let run = run_of(&report, "use");
    assert!(
        run.fallbacks.is_empty() && run.content == RecoveryContent::ContainsStatements,
        "the unmutated copy presents whole:\n{}",
        run.text
    );
    assert!(
        run.text.contains("try (java.io.Reader local1 = arg0)"),
        "{}",
        run.text
    );

    // `p3_typed_catch`'s catch-all row: the member is presented and only the unreachable handler
    // copy is quoted. The quoted block's slot-1 accesses (`iload_1` at 12, `istore_1` at 15) are
    // the handler copy's own lifetime — the member would refuse under the fallback rule if they
    // belonged to the presented local — so this fallback is *not* on the presented slice, and the
    // closure is already narrowed to the block.
    let snapshot = open(TYPED_CATCH_V8);
    let report = class_source_of(&snapshot, "TypedCatch", 8);
    let run = run_of(&report, "finallyIncrements");
    let text = run.text.as_str();
    assert_eq!(
        run.content,
        RecoveryContent::ContainsStatements,
        "the member is presented:\n{text}"
    );
    assert!(
        !text.contains("crosses a quoted fallback region")
            && !text.contains("crosses a protected region"),
        "no whole-member refusal is left:\n{text}"
    );
    assert_eq!(run.fallbacks, vec!["jre_region_uncovered_blocks"], "{text}");
    assert_eq!(
        quoted_bcis(text),
        [11, 12, 13, 14, 15, 16, 17].into_iter().collect(),
        "{text}"
    );
    // The presented local's own instructions are outside the quoted block: the declaration at 1,
    // the protected store at 3, the normal copy's read at 4 and store at 7, and the join read at 18
    // are all located by their own segments.
    for (needle, bci) in [
        ("int local1 = 0;", 1u32),
        ("local1 = arg0;", 3),
        ("local1 = local1 + 1;", 7),
        ("return local1;", 19),
    ] {
        assert_eq!(
            origin_of(run, at(text, needle)),
            bci,
            "`{needle}` keeps its own origin:\n{text}"
        );
    }
    assert!(
        (11..17).contains(&12) && (11..17).contains(&15),
        "the handler copy's slot-1 accesses sit inside the quoted block"
    );
}

// -------------------------------------------------------------------------------------------
// 2.3 — the three-way acceptance (needs both JDKs; run with `--ignored`).
// -------------------------------------------------------------------------------------------

/// One compiler leg: the label a failure names, whether it is the real JDK 8, and its tools.
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

impl Leg {
    fn javac(&self) -> Command {
        if self.real_javac8 {
            assert!(
                Path::new(CORRETTO_HOME).join("bin/javac").exists(),
                "the real javac 8 of the fixture protocol is installed at {CORRETTO_HOME}"
            );
            Command::new(format!("{CORRETTO_HOME}/bin/javac"))
        } else {
            let mut command = Command::new("javac");
            command.args(["--release", "8", "-Xlint:-options"]);
            command
        }
    }

    fn java(&self) -> Command {
        if self.real_javac8 {
            Command::new(format!("{CORRETTO_HOME}/bin/java"))
        } else {
            Command::new("java")
        }
    }

    fn compile(&self, directory: &Path, sources: &[&str]) {
        let mut command = self.javac();
        command
            .args(["-g:none", "-cp"])
            .arg(directory)
            .args(["-d"])
            .arg(directory);
        for source in sources {
            command.arg(directory.join(source));
        }
        let result = command.output().expect("the leg's compiler is installed");
        assert!(
            result.status.success(),
            "{}: javac rejected {sources:?}:\n{}",
            self.label,
            String::from_utf8_lossy(&result.stderr)
        );
    }

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

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

/// One private directory a test compiles and runs in.
struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-local-scope-refusals-{label}-{}-{nonce}-{}",
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

/// The presented text with every `//` comment line dropped — the strip the fixture's own
/// roundtrip and the patrols' texts are stripped by.
fn stripped(text: &str) -> String {
    let mut body: Vec<String> = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(str::to_owned)
        .collect();
    body.push(String::new());
    body.join("\n")
}

/// What the `ScopePlan` driver prints on every side: the return values of the normal and
/// exceptional inputs and the escaping `Error`'s own type and message.
const PLAN_TRACE: &str = "catchOnly false=1\n\
catchOnly true=2\n\
assignedAcrossTry false=3\n\
assignedAcrossTry true=4\n\
assignedAcrossIf false=6\n\
assignedAcrossIf true=5\n\
nestedHandlerOnly null=9\n\
nestedAcross false=10\n\
nestedAcross true=11\n\
nestedHandlerOnly error=java.lang.AssertionError:boom";

/// What the 1.5 shape's runner prints: the computed write on the normal path and the handler's
/// write on the caught one.
const LOOP_TRACE: &str = "normal=6\ncaught=0";

#[test]
#[ignore = "needs both JDKs: it compiles the original, the frozen JADX render and the recovered \
            text with `javac --release 8` **and** the real javac 8 and runs every side under \
            `java -Xverify:all` (see the module doc)"]
fn the_legal_shapes_answer_what_the_original_and_jadx_answer() {
    for leg in LEGS {
        // `ScopePlan` (the change's positive fixture): the class's own bytes, the frozen JADX
        // render and the recovered text, each beside the same driver source.
        let temp = TestDirectory::new("plan");
        let original = temp.path().join("original");
        let jadx = temp.path().join("jadx");
        let jarde = temp.path().join("jarde");
        for directory in [&original, &jadx, &jarde] {
            std::fs::create_dir_all(directory).expect("create the comparison directory");
        }
        std::fs::write(original.join("ScopePlan.class"), PLAN_V8).expect("write the frozen class");
        std::fs::write(original.join("ScopePlanDriver.java"), PLAN_DRIVER)
            .expect("write the driver");
        leg.compile(&original, &["ScopePlanDriver.java"]);
        std::fs::write(jadx.join("ScopePlan.java"), PLAN_JADX).expect("write the JADX render");
        std::fs::write(
            jadx.join("ScopePlanDriver.java"),
            format!("package defpackage;\n{PLAN_DRIVER}"),
        )
        .expect("write the JADX driver");
        leg.compile(&jadx, &["ScopePlan.java", "ScopePlanDriver.java"]);
        let report = class_source_of(&open(PLAN_V8), "ScopePlan", 8);
        std::fs::write(jarde.join("ScopePlan.java"), stripped(&report.text))
            .expect("write the recovered text");
        std::fs::write(jarde.join("ScopePlanDriver.java"), PLAN_DRIVER).expect("write the driver");
        leg.compile(&jarde, &["ScopePlan.java", "ScopePlanDriver.java"]);

        assert_eq!(
            leg.run(&original, "ScopePlanDriver"),
            PLAN_TRACE,
            "{}: the fixture's own class answers the trace",
            leg.label
        );
        assert_eq!(
            leg.run(&jadx, "defpackage.ScopePlanDriver"),
            PLAN_TRACE,
            "{}: the frozen JADX render answers exactly what the class answers",
            leg.label
        );
        assert_eq!(
            leg.run(&jarde, "ScopePlanDriver"),
            PLAN_TRACE,
            "{}: the recovered text answers exactly what the class answers, the exceptional input \
             and the escaping `Error` included",
            leg.label
        );

        // The 1.5 shape (`p3-loop-try-handler-entry`): the computed try update and the handler
        // write, on the frozen class, its frozen JADX render and the recovered text.
        let temp = TestDirectory::new("loop");
        let original = temp.path().join("original");
        let jadx = temp.path().join("jadx");
        let jarde = temp.path().join("jarde");
        for directory in [&original, &jadx, &jarde] {
            std::fs::create_dir_all(directory).expect("create the comparison directory");
        }
        std::fs::write(original.join("LoopTryHandlerEntryArgs.class"), LOOP_V8)
            .expect("write the frozen class");
        std::fs::write(original.join("Runner.java"), LOOP_RUNNER).expect("write the runner");
        leg.compile(&original, &["Runner.java"]);
        std::fs::write(jadx.join("LoopTryHandlerEntryArgs.java"), LOOP_JADX)
            .expect("write the JADX render");
        std::fs::write(jadx.join("Runner.java"), LOOP_RUNNER).expect("write the runner");
        leg.compile(&jadx, &["LoopTryHandlerEntryArgs.java", "Runner.java"]);
        let report = class_source_of(&open(LOOP_V8), "LoopTryHandlerEntryArgs", 8);
        std::fs::write(
            jarde.join("LoopTryHandlerEntryArgs.java"),
            stripped(&report.text),
        )
        .expect("write the recovered text");
        std::fs::write(jarde.join("Runner.java"), LOOP_RUNNER).expect("write the runner");
        leg.compile(&jarde, &["LoopTryHandlerEntryArgs.java", "Runner.java"]);

        assert_eq!(
            leg.run(&original, "Runner"),
            LOOP_TRACE,
            "{}: the frozen 1.5 class answers its recorded trace",
            leg.label
        );
        assert_eq!(
            leg.run(&jadx, "Runner"),
            LOOP_TRACE,
            "{}: the frozen JADX render answers the same trace",
            leg.label
        );
        assert_eq!(
            leg.run(&jarde, "Runner"),
            LOOP_TRACE,
            "{}: the recovered text answers the same trace, the caught path included",
            leg.label
        );

        // The refused shapes are never counted as passing: the recovered text of a class with a
        // refused member does not compile, and the refusal markers are what its acceptance states.
        let temp = TestDirectory::new("refused");
        let refused = temp.path().join("refused");
        std::fs::create_dir_all(&refused).expect("create the refusal directory");
        let report = class_source_of(&open(REFUSALS_V8), "ScopeRefusals", 8);
        assert!(report.text.contains(CROSSES_FALLBACK) && report.text.contains(CROSSES_PROTECTED));
        std::fs::write(refused.join("ScopeRefusals.java"), stripped(&report.text))
            .expect("write the refused text");
        let result = leg
            .javac()
            .args(["-g:none", "-d"])
            .arg(&refused)
            .arg(refused.join("ScopeRefusals.java"))
            .output()
            .expect("the leg's compiler is installed");
        assert!(
            !result.status.success(),
            "{}: a class with a refused member stays uncompilable, and is never counted as a \
             passing Java source:\n{}",
            leg.label,
            std::fs::read_to_string(refused.join("ScopeRefusals.java")).expect("the text reads")
        );
    }
}
