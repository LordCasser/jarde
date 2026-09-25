//! P3: a row of the exception table that names a `catch` type is a `catch`, not a resource header.
//!
//! `javac --release 8 -g:none` writes a plain `try`/`catch` as a protected range plus one row per
//! clause. When that range begins at the **first instruction of the method** there is no instruction
//! before it, so nothing the range's start could be the end of — it is no `try (…)` header, and the
//! `twr@1` rule's `resources()` refused the shape as `jre_guard_resource_init` instead: the arm was
//! quoted under that code and the handler was quoted as an uncovered block. The classification is
//! what this pins: a row no instruction precedes is not a resource header, the walk states its own
//! reason for the exception edge, and the named rows become the clauses of a `try` whose protected
//! range and handler bodies are recovered as ordinary regions.
//!
//! The other half of the same classification is the instruction that *does* precede the range: an
//! ordinary local assignment (`int x = 1;`) fills no resource either, and `initialisedBeforeTry` is
//! that shape — the statement before `x = 1` is what keeps the row's own reading to the store's own
//! statement, and the assignments are written before the `try` while the clause holds the handler's
//! `return x`. A store filled by `new Res()` or by a call keeps its verdict: a `try`-with-resources
//! whose close order cannot be proved stays a refusal and is never spelled as a user `catch`
//! (`tests/p3_guard.rs`'s swallowed-initialisation case and the guard suite pin that side).
//!
//! The texts below pin every member over the class bytes committed in
//! `tests/fixtures/p3-typed-catch/` (see its `README.md` for the command, the version and the
//! digest): one named clause (`namedCatch`), a range preceded by an ordinary assignment
//! (`initialisedBeforeTry`), two rows with one range and two handler entries, which are two clauses
//! in table order (`twoCatches`), one clause for two rows that share a handler (`multiCatch`), and
//! the control that must not change — an `any` row, which is neither a `catch` nor a `finally`
//! (`finallyIncrements`).
//!
//! `throw new` inside the `try` is still quoted ([`init.rs`]'s `new@1` is another change's work), so
//! nothing here requires the `throw` to be spelled. What is required is the structure around it: the
//! `if` and the `return` of the `try` block are inside the `try`, every clause names the row's own
//! class, and the clause bodies hold the statements their handlers run.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 540 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-typed-catch/v8/TypedCatch.class");

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
fn a_named_row_with_nothing_before_it_is_a_catch_not_a_resource_header() {
    // `namedCatch(I)I`'s row is `[0, 13) → 14 IllegalArgumentException`, and BCI 0 is the method's
    // own first instruction: there is no instruction before the protected range, so the row is no
    // resource header. Before this change the member was quoted under `jre_guard_resource_init`
    // ("the resource's own initialisation is not one statement …") with BCI 14 as an uncovered block;
    // the `try` header was never the shape.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "TypedCatch");
    let text = text_of(&report, "namedCatch");
    let opened = at(text, "try {");
    let branch = at(text, "if (arg0 < 0) {");
    // The `return n` of the source's `try` block, inside the `try` and not only after it.
    let returned = at(text, "return arg0;");
    let clause = at(
        text,
        "} catch (java.lang.IllegalArgumentException local1) {",
    );
    let caught = at(text, "return -1;");
    assert!(
        opened < branch && branch < returned && returned < clause && clause < caught,
        "the `try` holds the branch and the return, and the clause follows it with the handler's \
         own return:\n{text}"
    );
    assert!(
        !text.contains("jre_guard_resource_init"),
        "a row nothing precedes is not a resource header:\n{text}"
    );
    assert!(
        !text.contains("try ("),
        "the statement has no resource header to write:\n{text}"
    );
    assert!(
        !text.contains("Throwable"),
        "the clause names the row's own class, never a superclass:\n{text}"
    );
    assert!(
        !text.contains("jarde: not recovered"),
        "the member is presented, not explained:\n{text}"
    );
    // A text with statements in it is `contains_statements`, and the explanation-only answer this
    // shape used to get is not.
    assert_eq!(
        run_of(&report, "namedCatch").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}

#[test]
fn a_store_before_the_protected_range_is_not_a_resource_header() {
    // `initialisedBeforeTry(I)I` runs `int y = 2; int x = 1;` before its protected range, so the
    // range begins at BCI 4 and the instruction immediately before it is `istore_2` — the store of
    // `x`, whose own statement stops where `y`'s begins. What the store writes is no resource's
    // value (`1` is pushed by a constant and nothing in that statement is a `new` or an invocation),
    // so the row is no header: the assignment is written **before** the `try` and the clause holds
    // the handler's `return x`.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "TypedCatch");
    let text = text_of(&report, "initialisedBeforeTry");
    let assigned = at(text, "local2 = 1;");
    let opened = at(text, "try {");
    let clause = at(
        text,
        "} catch (java.lang.IllegalArgumentException local3) {",
    );
    let caught = at(text, "return local2;");
    assert!(
        assigned < opened && opened < clause && clause < caught,
        "the assignment is written before the `try`, and the clause holds the handler's own return \
         of the local that holds `1`:\n{text}"
    );
    assert!(
        !text.contains("jre_guard_resource_init"),
        "a store no `new` and no invocation produced is not a resource header:\n{text}"
    );
    assert_eq!(
        run_of(&report, "initialisedBeforeTry").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}

#[test]
fn two_rows_with_two_handlers_are_two_clauses_in_table_order() {
    // `twoCatches(I)I` declares `[0, 27) → 28 IllegalArgumentException` and
    // `[0, 27) → 31 IllegalStateException`: one protected range, two handler entries, and the table's
    // **order** is the priority the JVM dispatches in — so the clauses are written in that order,
    // each with its own body.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "TypedCatch");
    let text = text_of(&report, "twoCatches");
    let opened = at(text, "try {");
    let first = at(
        text,
        "} catch (java.lang.IllegalArgumentException local1) {",
    );
    let first_body = at(text, "return -1;");
    let second = at(text, "} catch (java.lang.IllegalStateException local1) {");
    let second_body = at(text, "return -2;");
    assert!(
        opened < first && first < first_body && first_body < second && second < second_body,
        "both clauses are written, in exception-table order, each with its handler's body:\n{text}"
    );
    assert_eq!(
        text.matches("catch (").count(),
        2,
        "one clause per handler entry, and no more:\n{text}"
    );
    assert_eq!(
        run_of(&report, "twoCatches").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}

#[test]
fn two_rows_that_share_one_handler_are_one_multi_catch_clause() {
    // `multiCatch(I)I` declares `[0, 27) → 28 IllegalArgumentException` and
    // `[0, 27) → 28 IllegalStateException`: one protected range, **one handler entry**, two classes.
    // That is the multi-catch the compiler writes `catch (A | B n)`: one clause, in table order, and
    // the handler's body runs once — reading the rows as two clauses would walk and write that body
    // twice, which is a count the bytecode does not have.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "TypedCatch");
    let text = text_of(&report, "multiCatch");
    let opened = at(text, "try {");
    let clause = at(
        text,
        "} catch (java.lang.IllegalArgumentException | java.lang.IllegalStateException local1) {",
    );
    let body = at(text, "return -3;");
    assert!(
        opened < clause && clause < body,
        "the one clause follows the protected range and holds the handler's body:\n{text}"
    );
    assert_eq!(
        text.matches("catch (").count(),
        1,
        "the two rows reach one handler entry, so they are one clause:\n{text}"
    );
    assert_eq!(
        text.matches("return -3;").count(),
        1,
        "and its body is walked, and written, once:\n{text}"
    );
    assert_eq!(
        run_of(&report, "multiCatch").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}

#[test]
fn a_catch_type_of_zero_becomes_neither_a_catch_nor_a_finally() {
    // `finallyIncrements(I)I`'s row is `[2, 4) → 11 any`: `catch_type == 0` is the catch-all a
    // `finally` copy is written under, and this build does not distinguish the two — it writes
    // neither clause for it, whatever the graph says about the row.
    //
    // P3 2.9, on the same member: the range holds `iload_0; istore_1` and covers no throwing
    // instruction, and the table states the row anyway — so the graph builds the record's edge from
    // the block its range intersects, and the copy at BCI 11 is a node the run **accounts for**
    // instead of four instructions that no block covered and no dead-node list named. The walk has no
    // clause to write around that edge (a catch-all row names no `catch` type), and 2.9 read that as
    // a region leaving through it: the whole body was quoted under `jre_region_exception_edge` and
    // the block the edge reaches was quoted beside it. What the change does **not** do is invent a
    // clause: neither `catch` nor `finally` appears, and the unaccounted-instruction quote the old
    // graph produced for a body like this one is gone.
    //
    // P3 2.15, on the same member: an edge **no instruction of the block can take** is not a way out
    // of it. Nothing in `[2, 4)` can raise, so no run enters the copy at BCI 11, and the four
    // statements the bytes hold are this method's normal flow: they are written again — the body the
    // run presented before 2.9 — and the copy the row reaches is named by the uncovered-blocks quote
    // like any other live block no statement reached. The pin below is the pre-2.9 presentation
    // restored, with 2.9's account of the handler kept as the `// @bytecode 11` quote.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "TypedCatch");
    let text = text_of(&report, "finallyIncrements");
    // The body's own statements, in the order the block runs them. Before 2.15 the whole member was
    // quoted, so not one of these four was written.
    let declared = text
        .find("int local1 = 0;")
        .unwrap_or_else(|| panic!("the body's first statement is written:\n{text}"));
    let assigned = text
        .find("local1 = arg0;")
        .unwrap_or_else(|| panic!("the `try` body's own statement is written:\n{text}"));
    let incremented = text.find("local1 = local1 + 1;").unwrap_or_else(|| {
        panic!("the copy the compiler inlined after the range is written:\n{text}")
    });
    let returned = text
        .find("return local1;")
        .unwrap_or_else(|| panic!("the method's own return is written:\n{text}"));
    assert!(
        declared < assigned && assigned < incremented && incremented < returned,
        "the statements keep the order the block runs them in:\n{text}"
    );
    assert!(
        !text.contains("catch"),
        "an `any` row is not a clause:\n{text}"
    );
    // No `finally` clause: the member's own name contains the word, so what is checked is that no
    // `finally` *keyword* is written (`finally` is spelled in Java only as the clause that follows a
    // `try`, and the envelope's own comment lines are not statements), and then that the statements
    // themselves hold neither word.
    assert!(
        !text.contains("finally {"),
        "and it is not written as a `finally` clause either:\n{text}"
    );
    let run = run_of(&report, "finallyIncrements");
    assert!(
        !run.fallbacks
            .contains(&"jre_region_unaccounted_instruction"),
        "every instruction of this body is inside a block the graph holds: {:?}\n{text}",
        run.fallbacks
    );
    assert!(
        !run.fallbacks.contains(&"jre_region_exception_edge"),
        "the edge the catch-all row states is one no instruction of the block can take, so nothing \
         here leaves through it: {:?}\n{text}",
        run.fallbacks
    );
    assert!(
        run.fallbacks.contains(&"jre_region_uncovered_blocks"),
        "the copy the row reaches is a live block no statement of this body reaches: {:?}\n{text}",
        run.fallbacks
    );
    let quoted = |bci: u32| {
        text.lines()
            .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
            .flat_map(|bcis| bcis.split_whitespace())
            .filter_map(|bci| bci.parse::<u32>().ok())
            .any(|cited| cited == bci)
    };
    assert!(
        quoted(11),
        "the block at BCI 11 — the copy nothing can enter — is named by the run's own quote: {text}"
    );
    for bci in [0, 18] {
        assert!(
            !quoted(bci),
            "the block at BCI {bci} is written as a statement, not quoted: {text}"
        );
    }
    assert_eq!(
        run.content,
        RecoveryContent::ContainsStatements,
        "no clause of this member is written, and the body's own statements are:\n{text}"
    );
}
