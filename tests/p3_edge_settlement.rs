//! P3 2.15: how one exception edge settles the block it leaves.
//!
//! Two questions are asked of one exception edge and one block, and both are facts of the same decode:
//!
//! * **can the block take the edge?** The graph states the table's rows two ways (P3 2.9): a record a
//!   `may_throw` instruction of the body covers keeps the edge its sites feed, and a record **no**
//!   such instruction covers is stated by its protected range — one edge from every block the range
//!   intersects. The second kind cannot be taken: nothing in the block can raise, so no run of it
//!   ever enters the handler. Reading that edge as a way *out* of the block quoted the whole
//!   statement instead of writing it, and the four statements of `noThrowFinally(I)I` were four
//!   `// @bytecode` lines — the `try`/`finally` whose body cannot throw, which is the same shape
//!   `TypedCatch.finallyIncrements(I)I` states in `tests/fixtures/p3-typed-catch/`.
//! * **does the row protect this block?** The settlement reads the row's `[start_bci, end_bci)` and
//!   the block's own span as **intersecting** ranges, and requires the handler to match and the row
//!   to name a `catch` type. The canonical graph fuses straight-line code, so a row may begin
//!   *inside* the block it protects — the inner range of `nested(I)I` begins at BCI 8 in the block
//!   `[7, 11)` the outer clause's binding store opened, and `calls(ILjava/lang/Runnable;)I`'s range
//!   begins at BCI 2 in the block the `int x = n;` lead opened. Requiring the range to cover the
//!   block's **start** quoted such a block, and the statement the bytes hold with it, because the
//!   graph's fusion rather than the table was what the test read.
//!
//! What neither question changes is the guard rules of P3 2.4: a block a rule claims or refuses is
//! answered by the rule, whatever an instruction of it may raise. `calls` is also the control for
//! the *live* edge — the call its row covers is what makes the `try` a statement rather than a quote.
//!
//! The texts below pin the shape over the class bytes committed in `tests/fixtures/p3-edge-settlement/`
//! (see its `README.md` for the command, the version, the 656 bytes and the SHA-256).

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 656 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-edge-settlement/v8/Settled.class");

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

/// The instruction starts one member's text quotes, as `// @bytecode …` lines state them.
fn quoted_bcis(text: &str) -> Vec<u32> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|bcis| bcis.split_whitespace())
        .filter_map(|bci| bci.parse::<u32>().ok())
        .collect()
}

#[test]
fn a_row_whose_range_holds_no_throwing_instruction_is_still_the_bodys_statement() {
    // `increments(I)I`'s row is `[0, 4) → 7 RuntimeException`, and the range holds `iload_0;
    // iconst_1; iadd; istore_0`: the opcode table calls none of those instructions throwing, so
    // nothing in the block can enter the handler and the edge the table states is one the walk
    // cannot leave through. The statement is the method's normal flow and is written as one —
    // `steps(I)I` of `tests/fixtures/p3-stated-rows/` is the same shape on its own fixture, and this
    // member is this fixture's control for it.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Settled");
    let text = text_of(&report, "increments");
    let open_try = at(text, "try {");
    let incremented = at(text, "arg0 = arg0 + 1;");
    let clause = at(text, "} catch (java.lang.RuntimeException local1) {");
    let caught = at(text, "arg0 = -1;");
    let returned = at(text, "return arg0;");
    assert!(
        open_try < incremented && incremented < clause,
        "the statement's body is the increment the row's range holds:\n{text}"
    );
    assert!(
        clause < caught && caught < returned,
        "the clause the row names follows it, and the method's own return follows both:\n{text}"
    );
    assert!(
        !text.contains("@bytecode"),
        "every instruction of this body is accounted for by a statement:\n{text}"
    );
    assert_eq!(
        run_of(&report, "increments").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}

#[test]
fn a_nested_statement_whose_range_begins_inside_its_block_is_written() {
    // `nested(I)I` is a `try`/`catch` whose clause body is itself a `try`/`catch`. The inner row is
    // `[8, 11) → 14 IllegalArgumentException`, and BCI 8 is **inside** the block `[7, 11)` that the
    // outer clause's binding store (`astore_1` at BCI 7) opened — the canonical graph fuses the
    // clause's own entry with the inner statement's first instructions. The row protects that block
    // by the table's own statement, so the inner `try` is written where its range begins and the
    // block is not quoted for a range that begins after its start.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Settled");
    let text = text_of(&report, "nested");
    let outer_open = at(text, "try {");
    let outer_body = at(text, "arg0 = arg0 + 1;");
    let outer_clause = at(text, "} catch (java.lang.RuntimeException local1) {");
    let inner_open = at(text, "\n            try {");
    let inner_body = at(text, "arg0 = -2;");
    let inner_clause = at(
        text,
        "} catch (java.lang.IllegalArgumentException local2) {",
    );
    let inner_caught = at(text, "arg0 = -3;");
    let returned = at(text, "return arg0;");
    assert!(
        outer_open < outer_body && outer_body < outer_clause,
        "the outer protected range is presented, and its clause follows it:\n{text}"
    );
    assert!(
        outer_clause < inner_open && inner_open < inner_body && inner_body < inner_clause,
        "the outer clause's body is the inner statement, written where its range begins:\n{text}"
    );
    assert!(
        inner_clause < inner_caught && inner_caught < returned,
        "the inner clause holds its own statement, and the method's return follows both:\n{text}"
    );
    assert!(
        !text.contains("@bytecode"),
        "no block of this body is quoted: both ranges begin in the blocks that hold them:\n{text}"
    );
    let run = run_of(&report, "nested");
    assert!(
        !run.fallbacks.contains(&"jre_region_exception_edge"),
        "neither row's edge holds a statement that the bytes state:\n{text}"
    );
    assert_eq!(run.content, RecoveryContent::ContainsStatements, "{text}");
}

#[test]
fn a_row_nothing_in_its_range_can_throw_does_not_quote_the_block() {
    // `noThrowFinally(I)I`'s row is `[2, 4) → 11 any` — the catch-all a `finally` copy is written
    // under — and its range holds `iload_0; istore_1`. Nothing in the block can raise, so the copy
    // at BCI 11 is code no run enters; the four statements of the body are the method's normal flow
    // and are written, and the handler the table names stays named by the uncovered-blocks quote
    // (`TypedCatch.finallyIncrements(I)I` is the same shape in `tests/fixtures/p3-typed-catch/`,
    // and its own pin is changed by the same rule).
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Settled");
    let text = text_of(&report, "noThrowFinally");
    let declared = at(text, "int local1 = 0;");
    let assigned = at(text, "local1 = arg0;");
    let incremented = at(text, "local1 = local1 + 1;");
    let returned = at(text, "return local1;");
    assert!(
        declared < assigned && assigned < incremented && incremented < returned,
        "the body's statements are written in the order the block runs them:\n{text}"
    );
    // A catch-all row names no `catch` type, so no clause is written for it — in this build or any
    // other — and the word `finally` is spelled in Java only as the clause that would follow a `try`.
    assert!(
        !text.contains("catch"),
        "an `any` row is not a clause:\n{text}"
    );
    assert!(
        !text.contains("finally {"),
        "and it is not written as a `finally` clause either:\n{text}"
    );
    let quoted = quoted_bcis(text);
    assert_eq!(
        quoted,
        vec![11, 12, 13, 14, 15, 16, 17],
        "every instruction of the copy the row reaches is named by the quote, and no statement \
         is:\n{text}"
    );
    let run = run_of(&report, "noThrowFinally");
    assert!(
        run.fallbacks.contains(&"jre_region_uncovered_blocks"),
        "the block the record reaches is a live block no statement reached: {:?}\n{text}",
        run.fallbacks
    );
    assert!(
        !run.fallbacks.contains(&"jre_region_exception_edge"),
        "an edge no instruction of the block can take is not a way out of it: {:?}\n{text}",
        run.fallbacks
    );
    assert_eq!(run.content, RecoveryContent::ContainsStatements, "{text}");
}

#[test]
fn a_range_that_begins_inside_a_block_whose_instruction_throws_settles_it() {
    // `calls(ILjava/lang/Runnable;)I` is the live half of the mid-block question: the row
    // `[2, 8) → 11 IllegalArgumentException` covers the `invokeinterface run` at BCI 3, so the edge
    // is one the block really takes, and BCI 2 is inside the block `[0, 8)` that the `int x = n;`
    // lead opened. The lead is written before the statement and the call is the `try`'s body: the
    // row protects the block even though it does not cover the block's start.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Settled");
    let text = text_of(&report, "calls");
    let lead = at(text, "local2 = arg0;");
    let open_try = at(text, "try {");
    let call = at(text, "arg1.run();");
    let clause = at(
        text,
        "} catch (java.lang.IllegalArgumentException local3) {",
    );
    let caught = at(text, "local2 = -4;");
    let returned = at(text, "return local2;");
    assert!(
        lead < open_try && open_try < call && call < clause,
        "the lead is written before the statement, and the call is its body:\n{text}"
    );
    assert!(
        clause < caught && caught < returned,
        "the clause holds its own statement, and the return follows both:\n{text}"
    );
    assert!(
        !text.contains("@bytecode"),
        "the block the row protects is a statement, not a quote:\n{text}"
    );
    let run = run_of(&report, "calls");
    assert!(
        !run.fallbacks.contains(&"jre_region_exception_edge"),
        "the row's edge is settled by the clause the statement is inside: {:?}\n{text}",
        run.fallbacks
    );
    assert_eq!(run.content, RecoveryContent::ContainsStatements, "{text}");
}
