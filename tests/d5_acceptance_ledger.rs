//! D5 task 7.4: the per-item ledger for the change's twelve acceptance rows (D01–D12).
//!
//! This file is the ledger itself — its module documentation is the record, and the one test below
//! is the mechanical gate that keeps the record's pointers honest (every path it names has to exist
//! in this repository, and every row D01–D12 has to be present).
//!
//! # How to read a row
//!
//! Each of the twelve rows states four things, and never mixes them:
//!
//! * **positive**: the gate that has to pass for the row's claim;
//! * **negative**: the counter-example or the mutation that has to fail it — `—` where this round
//!   has no *mutation* and the row's negative half is one of the recorded counter-examples instead
//!   (the change's own D0/D1 logs name each of those), so that "no mutation here" is never mistaken
//!   for "no counter-example here";
//! * **evidence**: the command that was really run and where its raw output is recorded;
//! * **conclusion**: exactly one of the three kinds below.
//!
//! | conclusion | means |
//! | --- | --- |
//! | **契约通过** | the run *states* what the contract says: the same text, the same planes, a named state, a refused cursor, an attached evidence set. Nothing about cost. |
//! | **工作量下降** | a *counted* figure says the run really did less: a construction counter, a decoded-byte dimension, an entry that was never read, a residency that stayed inside its bound. Measured against a recorded baseline or against the full selection in the same run. |
//! | **时间收益未证实** | no wall-clock, throughput or first-result claim is made anywhere in this ledger. Task **7.3** owns that protocol (independent interleaved samples, first-result and full-sequence readings, RSS). The rows below mark this explicitly wherever a reader might be tempted to read a smaller count as a faster run. |
//!
//! # What this round added, and what it only re-ran
//!
//! Two targets are new in this round: `tests/d5_semantic_comparison.rs` (task 7.1) and
//! `tests/d5_process_release.rs` (task 7.2). Every other pointer below is an existing gate that this
//! round **ran** — the workspace run of the evidence section covers all of them — and the ledger says
//! which is which, because "an existing gate still passes" and "a new gate now holds this row" are
//! different facts.
//!
//! # The evidence of this round
//!
//! All of the following was run on this worktree, in the debug profile unless stated otherwise. The
//! figures are the commands' own summaries, copied without rounding; the *counts* are the record, and
//! a repeated run states its own durations (which is why no duration is read as a result anywhere in
//! this ledger).
//!
//! ```text
//! $ cargo test --test d5_semantic_comparison --all-features --locked
//! test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.24s
//!   determinism: 237 (member, selection) pairs, each run twice, identical
//!   three-way comparison: 79 member runs state one answer; 158 of them publish read details
//!   projection: 79 member runs compared against the full delivery
//!
//! $ cargo test --test d5_semantic_comparison --all-features --locked -- --ignored --nocapture
//! test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.10s
//!   Scope/simple: essential == full (241 bytes of class file), both print `5`
//!   Scope/simple: essential == full (241 bytes of class file), both print `5`
//!   BooleanContexts/flag: essential == full (232 bytes of class file), both print `true`
//!   BooleanContexts/answer: essential == full (234 bytes of class file), both print `1`
//!   ConcatConversion/booleanLiteral: essential == full (272 bytes of class file), both print `true!`
//!   ConcatConversion/nullPart: essential == full (470 bytes of class file), both print `null!`
//!   RefusedCast/leftRead: essential == full (291 bytes of class file), both print `7`
//!   RefusedCast/chainCast: both selections refuse, with javac's own sentence: D57.java:10: 错误: 缺少返回语句
//!
//! $ cargo test --test d5_process_release --all-features --locked -- --ignored
//! test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.36s
//!
//! $ cargo test --release --test d5_process_release --all-features --locked -- --ignored
//! test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s
//!   D5-CHILD depth additions=8      outcome=produced quality=structured text_bytes=199 ir_items=217  body_decodes=1 recovery_runs=1 work=11 work_after_drop=0
//!   D5-CHILD depth additions=1024   outcome=produced quality=fallback   text_bytes=241 ir_items=24601 body_decodes=1 recovery_runs=1 work=11 work_after_drop=0
//!   D5-CHILD depth additions=131072 outcome=produced quality=fallback   text_bytes=245 ir_items=3145753 body_decodes=1 recovery_runs=1 work=11 work_after_drop=0
//!   D5-CHILD depth-stop additions=131072 stop=report:jre_ir_table_missing,budget_exceeded_ir_items:dimension=IrItems work=12 work_after_drop=0
//!   D5-CHILD stage-stops attributed=7 facts_strong_count=1 ok=true
//!   D5-CHILD abandon-sequence enumerate=0 first=11 second=22 discarded=22 work=22 work_after_drop=0 ok=true
//!   D5-CHILD cache-and-slow-consumer stored=1 refused_capacity=1 hits=1 entries_after_clear=0 methods_delivered=18 buffered_weight_high_water=10253 buffered_weight_limit=33554432 final_delivered=true ok=true
//!   D5-CHILD parent child=<each of the four> exit=0 markers=<all present>
//!
//! $ cargo test --test p3_execution_comparison --locked -- --ignored
//! test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 46.01s
//!
//! $ cargo test --workspace --all-targets --all-features --locked
//! 108 targets: 1578 passed; 0 failed; 17 ignored
//!   tests/d5_semantic_comparison.rs: ok. 4 passed; 0 failed; 1 ignored
//!   tests/d5_process_release.rs:    ok. 0 passed; 0 failed; 5 ignored
//!   tests/d5_acceptance_ledger.rs:  ok. 1 passed; 0 failed; 0 ignored
//!
//! $ cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
//! Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.35s   (0 warnings)
//! ```
//!
//! Two runs of that workspace command were made in this round: the first was green (107 targets,
//! 1577 passed — it did not yet carry this file), and the second was green with the figures above. One
//! run in between failed a single assertion in `crates/jarde-cli/tests/export_cli.rs`
//! (`a_counted_dimension_override_is_accepted_and_stops_the_run_at_that_dimension`: the CLI's failure
//! document carried no `error.code`, where the run should have stopped unfinished on `ir_items=1`).
//! That target passes 11/11 alone and 3/3 on a repeated single-case run, and the same flake is already
//! recorded in `openspec/changes/add-demand-driven-core-results/verification.md` (D3' round) as an
//! existing one — it is named here so a reader who meets it does not read it as this round's
//! regression, and it is **not** worked around by relaxing the assertion.
//!
//! The D0–D4 figures this ledger refers to (the frozen baselines, the four D0 mutations, and the
//! per-stage records) are in `openspec/changes/add-demand-driven-core-results/verification.md`; they
//! are **not** re-measured here, and a row that leans on one says so.
//!
//! # The two mutation gates this round leaves red-on-removal
//!
//! Both mutations were applied to the production sources, run, and then reverted; the worktree's
//! final state carries **no** change to `crates/**` or `src/**` (`git diff --stat crates src` is
//! empty), and both gates were green again after the revert.
//!
//! ## M-a — "the detail category is closed but the whole table is still built"
//!
//! The one edit: in `crates/jarde-java/src/report.rs` the region-records arm was changed from "build
//! only when the category was selected" to "build always, then clear the vector when it was not
//! selected". The payload and the status list are therefore identical to the correct run — which is
//! exactly why the *construction-site* counter (`crates/jarde-java/src/demand_counts.rs`) is the only
//! thing that can see it, and why this row's gate is an in-crate one.
//!
//! ```text
//! $ cargo test -p jarde-java --lib --locked the_essential_selection_builds_only_what_it_asked_for
//! test evidence::tests::the_essential_selection_builds_only_what_it_asked_for ... FAILED
//! panicked at crates/jarde-java/src/evidence.rs:1047:9:
//! assertion `left == right` failed: the ordinary recovery constructs no owning record of any optional category
//!   left: Built { source_map: 0, region_details: 1, rule_details: 0, name_details: 0 }
//!  right: Built { source_map: 0, region_details: 0, rule_details: 0, name_details: 0 }
//! test result: FAILED. 0 passed; 1 failed; 0 ignored; 80 filtered out
//! ```
//!
//! ## M-b — "the continuation re-scans the unit by the published match ordinal"
//!
//! The one edit: in `crates/jarde-query/src/xref/code.rs`'s `open`, a resumed `QueryPosition::Method`
//! was still *verified* but no longer *honoured* — the walk started at the unit's first member again,
//! so the published item ordinal became the only thing that skipped what an earlier page had already
//! published (the shape the D4 stage removed).
//!
//! ```text
//! $ cargo test --test p1_query_demand --all-features --locked
//! test a_page_that_ends_at_a_member_boundary_does_not_decode_it_again ... FAILED
//! panicked at tests/p1_query_demand.rs:581:5:
//! assertion `left == right` failed: the continuation decoded the later members, the filler and the
//! nested entry's class — and not the dense member again
//!   left: 193
//!  right: 104
//! test the_continuation_splices_back_to_the_unpaged_scan ... FAILED
//! panicked at tests/p1_query_demand.rs:477:9:
//! assertion `left == right` failed: page 17 carries the next items of the unpaged scan, in order
//! test result: FAILED. 4 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
//! ```
//!
//! Two different kinds of failure from one removal, which is the point of recording both: the
//! continuation paid for the member it had already published (193 decoded bytes against the 104 the
//! run needs), **and** it published that member's items a second time.
//!
//! # The twelve rows
//!
//! | ID | positive gate (this round ran it) | negative / mutation | conclusion |
//! | --- | --- | --- | --- |
//! | D01 | `tests/d0_demand_counts.rs::a_member_only_read_decodes_no_body_and_builds_no_record` (member-only read: 0 body decodes, 0 owning records); the structural-reference half is the `QueryResolution::NotRequested` plane of `tests/p1_xref_metadata.rs` and `tests/p1_xref_code.rs` | D0's M2 ("build the whole unit, then filter") turns the same gate red: `body_decodes` 1→9, `method_bodies` 1→9, `owned_records` 7→89 (recorded in `openspec/changes/add-demand-driven-core-results/verification.md` §7.3) | **工作量下降** |
//! | D02 | `tests/d2_prepared_handoff.rs::a_view_decodes_every_requested_body_against_one_preparation` and `::both_binding_paths_prepare_the_read_they_selected_once`; `tests/p3_accessor_edges.rs::the_driver_and_its_same_class_callees_share_one_read_and_one_preparation` (`class_materializations == 1`, `class_preparations == 1`) | D0's M1 (a second materialization) is recorded with its raw output in `openspec/changes/add-demand-driven-core-results/verification.md` §7.3; D2's own counter-examples M3–M6 are named in the change's `openspec/changes/add-demand-driven-core-results/tasks.md` rows 3.1–3.4 with their gates (`class_headers` 2→1, a preparation per body → 3≠1, a store-answered container counted as held) — this ledger does not claim their raw output was kept | **工作量下降** |
//! | D03 | `tests/d5_semantic_comparison.rs::every_selection_presents_the_same_answer` (79 member runs: text, `representation/quality/syntax_status/content`, outcome, fallbacks and every diagnostic identical under essential / local / all) and `::a_repeated_request_states_the_same_report` (237 pairs, field-for-field) | no mutation in this round: the counter-example is the D0 free-function baseline (§7.2: every optional table built for every request), and the in-crate gate below (M-a) is the one that fails when the gate is removed | **契约通过** |
//! | D03 (JDK) | `tests/d5_semantic_comparison.rs::the_essential_and_the_full_selection_compile_the_same_program` — eight listed members (seven compile and run, one is the sample's quoted refusal): the essential and the full text compile to **byte-identical** class files and print the sample's own value, and the refusal stays the same refusal | no mutation: a selection that changed the text, dropped a rule's output or added a refusal compiles a different program or refuses a member the P3 3.3 tables call whole | **契约通过**; no time claim |
//! | D04 | `crates/jarde-java/src/evidence.rs::the_essential_selection_builds_only_what_it_asked_for` (construction-site counts of four categories) and `tests/d5_semantic_comparison.rs::the_local_delivery_is_the_full_one_projected_onto_its_range` (79 runs: ordered subsequence per category, exact region projection, anchor-set and quoted-text projection) | **M-a** above ("closed but still built") turns the construction-site gate red with `region_details: 1` against `0` | **工作量下降** |
//! | D05 | `tests/d1_evidence_selection.rs::{a_legal_empty_range_is_a_complete_empty_selection, a_selection_this_entry_cannot_answer_is_refused_with_its_own_code, a_range_over_a_body_with_no_code_is_refused_as_a_missing_table}`; the query-plane states are `tests/p1_query_planes.rs::a_full_page_a_cancellation_and_a_budget_stop_are_three_states` and `::a_container_the_page_never_reaches_is_unknown_and_not_empty` | the D0 counter-examples are the recorded ones (a legal empty selection answered as an error, a missing `Code` answered with an empty range); the recorded raw outputs are in verification §6 | **契约通过** |
//! | D06 | `tests/d1_evidence_selection.rs::a_selected_category_that_stopped_states_its_prefix` (the committed text is untouched by a refusal in the evidence phase); `tests/d5_process_release.rs::child_a_stop_at_every_stage_releases_what_it_held` (7 counted dimensions each stop the run, attributed to their own dimension, with `work_after_drop=0`) | no mutation in this round: the recorded counter-example is the D0 baseline where the phase had no stop at all; the child gate's own red-on-removal property is the abstention of the attribution assertion | **契约通过** + **工作量下降** (the stop is where the budget says, and nothing after it is charged) |
//! | D07 | `tests/d3_artifact_binding.rs::{the_evidence_is_rebuilt_after_every_temporary_of_the_first_request_is_dropped, a_text_that_is_not_this_artifact_is_a_mismatch_and_never_a_missing_answer, the_binding_holds_no_payload_of_the_run}` | the recorded integration counter-example (cancelling the prepared handover) is kept by `tests/d2_prepared_handoff.rs` and `tests/bulk_recovery_serial.rs` | **契约通过**; the rebuild is *charged again* by design, so no saving is claimed |
//! | D08 | `tests/p1_query_demand.rs::{a_small_page_stops_before_the_work_behind_it, the_continuation_splices_back_to_the_unpaged_scan, a_page_that_ends_at_a_member_boundary_does_not_decode_it_again}`; `tests/d0_demand_counts.rs::a_page_limited_query_publishes_one_item_and_decodes_the_whole_first_unit` (`unit_page_usage.code_bytes < unit_full_usage.code_bytes`) | **M-b** above turns two of them red: `code_bytes` 193 against 104 on the continuation, and repeated items on the spliced pages | **工作量下降** (counted bytes and entries) + **契约通过** (pages splice exactly) |
//! | D09 | `tests/p1_query_planes.rs::a_cursor_is_refused_unless_it_describes_the_binding_that_issued_it` (tampered target/scope/relation/position → `query_cursor_mismatch`); `tests/p1_query_positions.rs::{mixed_consumers_share_one_cursor_and_one_item_order, a_page_inside_the_metadata_step_names_it_and_replays_only_its_prefix}` | the recorded counter-example is the pre-D4 cursor (no position, no identity); no mutation in this round | **契约通过** |
//! | D10 | `tests/d3_artifact_binding.rs::{two_definitions_of_one_name_are_two_artifacts, one_class_stored_twice_is_two_artifacts, a_duplicated_member_record_binds_nothing}`; `tests/d1_evidence_selection.rs::the_driver_range_selects_the_records_it_intersects` (a callee's own BCI is not a driver position); the origin family is the same source with and without its debug tables in `tests/d5_semantic_comparison.rs` | no mutation in this round: the counter-examples are the four inputs the D3' collision matrix already carries (`Mismatched{Method}`/`Mismatched{Text}` and the zero-attachment rows) | **契约通过** |
//! | D11 | `tests/d5_process_release.rs::{child_the_abandon_sequence_leaves_no_work_behind, child_the_cache_and_a_slow_consumer_stay_bounded}` — enumerate (`work=0`), switch method (`first=11 second=22`), discard (`work_after_drop=0`), the facts handle back to `facts_strong_count=1`, a one-entry store `stored=1 refused_capacity=1 hits=1`, `clear()` → `entries_after_clear=0`, and a slow consumer delivered all 18 records inside a 33554432-byte window (`high_water=10253`) | no mutation in this round: the recorded counter-example is D2's M6 (a store-answered container registered as held), and the row's own negative half is that every one of those figures is asserted, not printed | **契约通过** + **工作量下降** (residency inside its own bound) |
//! | D12 | **not covered by this ledger.** Task 7.3 owns it: the frozen workloads, the interleaved samples, first-result and full-sequence CPU/wall-clock, RSS and retention-weight readings | — | **时间收益未证实** — this ledger makes no wall-clock, throughput or first-result claim, and none of the counts above may be read as one |
//!
//! # What this ledger does *not* cover
//!
//! * **Time.** Nothing here is a timing measurement (see D12): the counts are construction counts,
//!   decoded bytes, entries read and residency bounds. Task 7.3's protocol is the only place a
//!   duration or a rate may be stated.
//! * **The D0–D4 raw outputs.** The figures quoted above for those stages come from
//!   `openspec/changes/add-demand-driven-core-results/verification.md`, which records the commands
//!   and outputs of the rounds that produced them; this round re-ran the *gates*, not the profilers.
//! * **`ReadDetails` in the projection row.** `tests/d5_semantic_comparison.rs` compares the four
//!   categories this layer materializes; the fifth is published by the entry that performed the read,
//!   and the file states that boundary itself.
//! * **A mutation for every row.** Six of the twelve rows have no mutation in this round (§the two
//!   above plus the four whose counter-examples are recorded as inputs); the rows say so rather than
//!   implying a mutation that does not exist.
//! * **The JDK half of D03 outside the seven listed members.** Members with parameters are executed
//!   by `tests/p3_execution_comparison.rs`, whose wrapper builder reads each parameter's own facts;
//!   this round's JDK gate holds the selection fixed over the members whose declaration it can spell
//!   without them.

use std::collections::BTreeSet;
use std::path::Path;

/// The ledger's own text, read from this file: the pointers it states are the record, and a record
/// whose pointers rot is a record nobody can check.
const LEDGER: &str = include_str!("d5_acceptance_ledger.rs");

/// Every backticked token of the ledger that names a path of this repository.
///
/// The extraction is deliberately literal — a token inside backticks whose part before any `::` ends
/// in `.rs` is a path, and nothing else is read as one — so that a pointer which stops naming a file
/// fails here instead of being noticed by a reader a year from now. The `::` cut is what makes the
/// ledger's own style (`path::test_name`) checkable: the file half is checked, the test half is the
/// reader's.
fn named_paths() -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    let mut rest = LEDGER;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else {
            break;
        };
        let token = after[..close].split("::").next().unwrap_or_default();
        if token.ends_with(".rs") && token.contains('/') && !token.contains('*') {
            paths.insert(token.to_string());
        }
        rest = &after[close + 1..];
    }
    paths
}

/// Every pointer this ledger states is a file of this repository, and every row is present.
///
/// The row half is the completeness check the task asks for in mechanical form: a ledger that quietly
/// lost D07 would pass every other reading of it, and this one fails.
#[test]
fn the_ledger_names_only_records_this_repository_has() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let paths = named_paths();
    assert!(
        paths.len() >= 12,
        "the ledger names its records: only {} path(s) found",
        paths.len()
    );
    for path in &paths {
        assert!(
            root.join(path).is_file(),
            "the ledger names `{path}`, which is not a file of this repository"
        );
    }
    // The two targets this round added are named by the ledger itself: a ledger that described them
    // without naming them would not be a pointer a reader could follow.
    for required in [
        "tests/d5_semantic_comparison.rs",
        "tests/d5_process_release.rs",
        "crates/jarde-java/src/evidence.rs",
        "crates/jarde-query/src/xref/code.rs",
    ] {
        assert!(paths.contains(required), "the ledger names `{required}`");
    }
    for row in 1..=12 {
        let id = format!("| D{row:02} |");
        assert!(
            LEDGER.contains(&id),
            "the ledger carries the `D{row:02}` row"
        );
    }
    // And the three conclusions are three words, not a spectrum: each row states one of them.
    for conclusion in ["契约通过", "工作量下降", "时间收益未证实"] {
        assert!(
            LEDGER.contains(conclusion),
            "the ledger states the `{conclusion}` conclusion"
        );
    }
    println!(
        "ledger: {} pointer(s), 12 rows, three conclusions",
        paths.len()
    );
}
