//! P3 1.1/1.2: a proved prefix survives a local gap — the walk's own failure paths keep the
//! statements they already proved, and every live block stays named exactly once.
//!
//! # What the defect is
//!
//! `region.rs`'s `region_at_inner` builds one `Fallback` out of `prefix + current block` at every
//! `FallbackReason` return site and answers `next = None`. Two things follow:
//!
//! * the prefix — blocks the walk *proved* on its way to the gap — is quoted instead of written, so
//!   its statements leave the artifact even though nothing about them was in doubt;
//! * the method-level loop stops at the gap, and the walk's own `covers` check can drop a body it
//!   had claimed without anyone naming those blocks: a live block then belongs to **no** region,
//!   which is the one state the design's exactly-once invariant ("every live block is either a
//!   structured region or a fallback quote, never both and never neither") forbids.
//!
//! The class bytes committed in `tests/fixtures/p3-prefix-survival/` (see its `README.md` for the
//! command, the version, the 336 bytes and the SHA-256) state one method per half of that:
//!
//! * `guarded` is the case the entry point of this task named. `javac --release 8 -g:none` fuses
//!   `int x = 5;` with the branch that follows it into **one** canonical block (`[0, 6)`: the
//!   `iconst_5; istore_1` is the block's lead and `aload_0; instanceof; ifeq 13` its terminal), so
//!   the walk's `prefix` is empty here and the store is *already* written as a statement today —
//!   by `build.rs`'s `test_effects`, before the branch's condition is read. What this member's gap
//!   actually is is **build-side**: `instanceof` is `Operation::Other` (2c.9's work), so
//!   `Region::If`'s `test_expr` cannot render the condition and the whole region is quoted, which
//!   is why the quote it writes names the failing block's own start (BCI 0) beside the arms. This
//!   member therefore pins what is true of it — the store statement is in the text, the artifact
//!   is `contains_statements` and `fallback`, and the quote names the branch and both arms — and
//!   not the `Straight` + `Fallback` pair that only a walk-level failure produces (`loopThenBreak`
//!   below is that case).
//! * `loopThenBreak` is the walk-level case and the failing half of 1.1: the loop's shape is not
//!   one this subset proves (`break` leaves through a block outside the loop's own set), so
//!   `header_tested_loop`'s post-condition fails and the body it walked — block `[6, 15)`, which
//!   holds `x = x + n` and the `if (n == 5)` test — is dropped from the region list while its
//!   blocks stay claimed. Before 1.2 the text names the header, the break's `goto` block and the
//!   latch, and never names the body at all.
//!
//! A probe of every committed fixture in every test target (`region_at`'s failure paths printing
//! when a fallback is built from a non-empty prefix, and the walk's claimed set compared against
//! the blocks the released regions hold) found **no** method that hits either half today. Both
//! facts are why `loopThenBreak`, not `guarded`, is the red run this file pins.
//!
//! The third case was to be 1.1's other half: the graph-does-not-cover case must keep its
//! whole-method quote. `HistoricalControlFlow.finallyPath` (ECJ 4.6.1, v52) is not that body — the
//! `jsr`/`ret` subroutine is v45's, and this member's decode is inside two nodes of its graph, so
//! P3-R7's `jre_region_unaccounted_instruction` quote never fired for it. What kept it quoted was
//! P3 2.9's range-stated exception edge over a record no `may_throw` instruction of the body covers,
//! and P3 2.15 stops reading such an edge as a way out of its block: the body's statements are
//! written and the dead copy is named by the uncovered-blocks quote. The guard rail itself still
//! needs a body that really has the graph-hole shape (the v45 `finallyPath`, whose three dead nodes
//! `unaccounted_instructions` accounts for); re-pointing this case at one is the P3 1.1 owner's
//! follow-up, and the case below pins the measured 2.15 presentation until then.
//!
//! # 2026-09-26 supersession
//!
//! Both members above now present **whole**, and their pins were updated to that: `guarded`'s
//! condition renders since `recover-instanceof-expressions` (2c.9) made `instanceof` a value
//! expression, and `loopThenBreak`'s body walk succeeds since the loop-transfer work
//! (`present-proved-java-structure` 3.1's committed subset, pinned by `p3_loop_transfers`) reads a
//! `goto` leaving the loop's block set as the `break` it states. The walk-level half of 1.1/1.2
//! this file was created to pin — a proved prefix surviving a *walk* gap — has no committed fixture
//! left that exercises it; the exactly-once guard that stays green here is the dead-copy naming of
//! the third case, and re-pointing the walk-level pin at a real failure is the same owner
//! follow-up.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 336 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-prefix-survival/v8/Before.class");

/// The committed ECJ 4.6.1 v52 sample: its `finallyPath(I)I` holds the `jsr`/`ret` era's
/// subroutine, which the region pass refuses whole.
const HISTORICAL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of one committed sample, under the entry point the CLI calls.
fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
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

/// The recovery report of one member's own run.
fn run_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` was read by its own run, got {other:?}"),
    }
}

/// Where one substring sits in one member's text, stated as the panic a missing one deserves.
fn at(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("the text presents `{needle}`:\n{text}"))
}

#[test]
fn a_store_inside_the_failing_block_is_still_a_statement() {
    // `guarded(Ljava/lang/Object;)I` is
    //
    // ```text
    // 0: iconst_5      6: ifeq 13
    // 1: istore_1      9: iload_1  10: iconst_1  11: iadd  12: ireturn
    // 2: aload_0      13: iload_1  14: ireturn
    // 3: instanceof
    // ```
    //
    // so the canonical graph is three blocks — `[0, 6)` (the store's lead **and** the branch),
    // `[9, 13)` (`return x + 1`) and `[13, 15)` (`return x`) — and the region walk presents the
    // branch as `if@1`. The member was pinned when `instanceof` (BCI 3) was still
    // `Operation::Other`: the condition could not render, the region was quoted, and this test
    // held the store statement beside that quote. Since `recover-instanceof-expressions` (2c.9)
    // made `instanceof` a value expression the condition renders, so the whole member presents:
    // the fused block's lead is still written as the statement in front of the branch (by
    // `test_effects`, the block's own lead — the fusion the module comment above describes), and
    // nothing is quoted any more.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Before");
    let text = text_of(&report, "guarded");
    at(text, "int local1 = 5;");
    at(text, "arg0 instanceof java.lang.String");
    at(text, "return local1 + 1;");
    at(text, "return local1;");
    assert!(
        !text.contains("@bytecode"),
        "the member presents whole, with no refused region left to quote:\n{text}"
    );
    assert_eq!(
        run_of(&report, "guarded").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
    assert_eq!(
        run_of(&report, "guarded").quality,
        Quality::Structured,
        "{text}"
    );
}

#[test]
fn a_loop_whose_body_leaves_early_presents_its_break_and_exit() {
    // `loopThenBreak(I)I` is
    //
    // ```text
    //  0: iconst_0   6: iload_1   7: iload_0   8: iadd   9: istore_1
    //  1: istore_1  10: iload_0  11: iconst_5  12: if_icmpne 18
    //  2: iload_0   15: goto 25
    //  3: ifle 25   18: iload_0  19: iconst_1  20: isub  21: istore_0  22: goto 2
    //               25: iload_1  26: ireturn
    // ```
    //
    // and the canonical graph is `[0, 2)`, `[2, 6)` (the header), `[6, 15)` (the body: the store
    // `x = x + n` and the `if (n == 5)` test), `[15, 18)` (the `break`'s `goto`) and `[18, 25)`
    // (the latch). This member was the walk-level red run of 1.1/1.2: the body's walk dropped the
    // successor outside the loop's own block set (`15`), the loop's post-condition failed and the
    // body it had claimed was named nowhere. The loop-transfer work
    // (`present-proved-java-structure` 3.1's committed subset, pinned by `p3_loop_transfers`) now
    // reads that `goto` as the `break` it states, so the member presents whole: the store, the
    // loop, the `break` arm and the latch's decrement are all statements, and every live block is
    // claimed by the structure rather than named by a quote — the exactly-once invariant this file
    // exists for, satisfied with no refusal at all.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Before");
    let text = text_of(&report, "loopThenBreak");
    at(text, "int local1;");
    at(text, "local1 = 0;");
    at(text, "while (arg0 > 0)");
    at(text, "local1 = local1 + arg0;");
    at(text, "if (arg0 == 5)");
    at(text, "break;");
    at(text, "arg0 = arg0 - 1;");
    at(text, "return local1;");
    assert!(
        !text.contains("@bytecode"),
        "the member presents whole, with no refused region left to quote:\n{text}"
    );
    assert_eq!(
        run_of(&report, "loopThenBreak").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
    assert_eq!(
        run_of(&report, "loopThenBreak").quality,
        Quality::Structured,
        "{text}"
    );
}
#[test]
fn a_dead_exception_edge_does_not_quote_the_block_the_bytes_state() {
    // This case was named "a body the graph does not cover stays a whole method quote", and its
    // premise no longer holds — for either half of it.
    //
    // `HistoricalControlFlow.finallyPath(I)I` (ECJ 4.6.1, class-file version **52**) does not hold
    // the `jsr`/`ret` subroutine the old comment named: that is the v45 body of the same fixture.
    // Its one row is `[0, 4) → 9 any`, a `finally` copy's catch-all over `iload_1; iconst_1; iadd;
    // istore_3`, and the graph holds nodes for both the body (`[0, 9)`) and the copy (`[9, …)`) —
    // every instruction the decode reads is inside one of them, so P3-R7's
    // `jre_region_unaccounted_instruction` whole-body quote never fired for this member. What kept
    // it quoted was P3 2.9's range-stated exception edge: a row **no** `may_throw` instruction of
    // the body covers is stated by its protected range, the walk read that edge as a way out of
    // block `[0, 9)`, and the whole method became quotes.
    //
    // P3 2.15 reads the same edge honestly: nothing in `[0, 4)` can raise, so no run of the block
    // enters the copy at BCI 9 and the edge is not a way out of it. The three statements the bytes
    // hold are the method's normal flow and are written — `int local3 = arg1 + 1;` is `[0, 4)`'s
    // own `iload_1; iconst_1; iadd; istore_3` — and the copy is named by the uncovered-blocks quote
    // like any other live block no statement reached.
    //
    // The guard rail 1.1/1.2 stated here — a body with instructions the graph does not account for
    // must stay a whole-method quote — is **not** exercised by this member any more (the graph does
    // account for it). Re-pointing it at a body that really has the graph-hole shape (the v45
    // `finallyPath`, whose three dead nodes `unaccounted_instructions` names) is a follow-up of the
    // P3 1.1 owner's.
    let sample = open(HISTORICAL);
    let report = class_source_of(&sample, "HistoricalControlFlow");
    let text = text_of(&report, "finallyPath");
    let added = text
        .find("int local3 = arg1 + 1;")
        .unwrap_or_else(|| panic!("the row's own statement is written:\n{text}"));
    let incremented = text.find("arg1 = arg1 + 2;").unwrap_or_else(|| {
        panic!("the copy the compiler inlined after the range is written:\n{text}")
    });
    let returned = text
        .find("return local3;")
        .unwrap_or_else(|| panic!("the method's own return is written:\n{text}"));
    assert!(
        added < incremented && incremented < returned,
        "the statements keep the order the blocks run them in:\n{text}"
    );
    assert!(
        text.contains("// @bytecode 9"),
        "the copy the row reaches is named by the run's own quote:\n{text}"
    );
    let run = run_of(&report, "finallyPath");
    assert!(
        run.fallbacks.contains(&"jre_region_uncovered_blocks"),
        "the block the record reaches is a live block no statement reached: {:?}\n{text}",
        run.fallbacks
    );
    assert!(
        !run.fallbacks
            .contains(&"jre_region_unaccounted_instruction"),
        "every instruction this member's decode reads is inside a node of the graph: {:?}\n{text}",
        run.fallbacks
    );
    assert_eq!(
        run.content,
        RecoveryContent::ContainsStatements,
        "the body's own statements are written, and the copy is quoted beside them:\n{text}"
    );
    assert_eq!(run.quality, Quality::Fallback, "{text}");
    assert_eq!(
        run.representation,
        Representation::Mixed,
        "the artifact still carries the reasons and the bytecode it quotes:\n{text}"
    );
}
