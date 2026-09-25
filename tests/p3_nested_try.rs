//! P3 2.5: two protected ranges that begin at one instruction and **nest** are a `try` inside a
//! `try`, not a refusal.
//!
//! `crates/jarde-java/src/guard.rs`'s `catches()` reads the named rows that begin in one block. It
//! used to answer `None` for every block whose rows did not declare **one** `(start, end)` — the
//! rows that "merely begin at the same instruction with different ranges are a nesting this statement
//! cannot state". `javac --release 8` writes a nested `try` exactly that way: the inner statement's
//! row and the outer one's begin at the same instruction, the inner range is narrower, and the inner
//! handler entry lies inside the wider range, because the wider range protects the whole inner
//! statement — its handler included. The refusal then quoted the member whole, with the outer
//! handler, the inner handler and the code after both tries left as *uncovered blocks*.
//!
//! The nesting is read off the table: the narrower range is the inner `try`, its handler entry lies
//! inside the wider range, and the wider row's handler is the outer clause — the wider statement's
//! body **is** the inner `try` (the levels are built from the inside out; `Region::Try`'s body is a
//! `Region` already, so no region kind was added). Rows that share one range keep today's clauses,
//! multi-catch included, and ranges that cross — neither containing the other — are still no `try`
//! at all (`region.rs`'s `crossing_records` refuses such a body before the walk starts).
//!
//! The text below pins the shape over the class bytes committed in `tests/fixtures/p3-nested-try/`
//! (see its `README.md` for the command, the version and the digest):
//! `Nest.nest(Ljava/lang/Runnable;)I` states `[0, 6) → 9 IllegalArgumentException` and
//! `[0, 11) → 15 RuntimeException`, and what the text has to be is the inner `try` inside the outer
//! one, each clause holding its handler's own `return`, and the two clauses **not** siblings of one
//! `try`. P3 2.7 is the inner body: both rows protect the body's own call, and each row's handler is
//! one of the clauses the nesting already writes around it, so `arg0.run();` is written where the
//! bytecode states it instead of the block being quoted.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 361 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-nested-try/v8/Nest.class");
const CATCH_ESCAPE: &[u8] = include_bytes!("fixtures/p3-catch-binding-escape/v8/CatchEscape.class");

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

/// Where one substring sits after one position of the same text.
fn after(text: &str, needle: &str, from: usize) -> usize {
    from + at(&text[from..], needle)
}

#[test]
fn two_rows_that_begin_at_one_instruction_and_nest_are_a_try_inside_a_try() {
    // `nest(Ljava/lang/Runnable;)I`'s table states `[0, 6) → 9 IllegalArgumentException` and
    // `[0, 11) → 15 RuntimeException` (javap: `from 0 to 6 target 9`, `from 0 to 11 target 15`).
    // The two rows begin at BCI 0, the narrower range is the inner `try`, the inner handler entry
    // (BCI 9) sits inside the wider range, and the protected body is `aload_0; invokeinterface
    // java/lang/Runnable.run:()V` at BCI 0 and 1 — a call, not a `throw new`.
    //
    // The nesting is P3 2.5's; P3 2.7 is the inner body. It used to be quoted under
    // `jre_region_exception_edge` — the call at BCI 1 has an exception edge, and the walk read any
    // such edge as a shape the plain flow cannot leave through — although both rows of the table
    // protect that call and each one's handler is a clause written around the block. The text below
    // is the nesting the table states, with the inner body's own statement and the clauses in table
    // order.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Nest");
    let text = text_of(&report, "nest");
    let outer = at(text, "try {");
    let inner = after(text, "try {", outer + "try {".len());
    // The inner `try`'s own body block, as the statement it is: `aload_0; invokeinterface
    // java/lang/Runnable.run:()V` at BCI 0 and 1. Both rows of the table protect that call and each
    // one's handler is a clause of the `try` the block is inside, so the block is written where the
    // bytecode states it (P3 2.7) and not quoted. What the nesting must not do is lose it or move
    // it out of the inner statement.
    let body = at(text, "arg0.run();");
    let inner_clause = at(
        text,
        "} catch (java.lang.IllegalArgumentException local1) {",
    );
    let inner_return = at(text, "return -1;");
    let outer_clause = at(text, "} catch (java.lang.RuntimeException local1) {");
    let outer_return = at(text, "return -2;");
    let rest = at(text, "return 0;");
    assert!(
        outer < inner && inner < body && body < inner_clause && inner_clause < inner_return,
        "the inner `try` and its body are inside the outer `try`, and its own clause follows \
         them:\n{text}"
    );
    assert!(
        inner_return < outer_clause && outer_clause < outer_return,
        "the outer clause follows the inner one, and holds its own handler's return:\n{text}"
    );
    assert!(
        outer_return < rest,
        "the code after both tries is written after the outer clause:\n{text}"
    );
    // The two clauses are **not** siblings of one `try`: the outer clause hangs off the inner
    // statement's close, so exactly one brace — the inner clause's own — stands between the inner
    // handler's return and the outer `catch`, and the brace the outer clause is written after is
    // the one the *outer* `try` opened.
    let between = &text[inner_return + "return -1;".len()..outer_clause];
    assert_eq!(
        between.matches('}').count(),
        1,
        "the inner clause closes before the outer clause opens, and the outer clause is not a \
         sibling of the inner one:\n{text}"
    );
    assert_eq!(
        text.matches("catch (").count(),
        2,
        "one clause per level, and no more:\n{text}"
    );
    assert_eq!(
        text.matches("try {").count(),
        2,
        "the nesting is one inner `try` and one outer one:\n{text}"
    );
    assert!(
        !text.contains("jre_region_uncovered_blocks"),
        "the nesting claims every block of the statement — the old refusal left the handlers and \
         the code after the tries as uncovered blocks:\n{text}"
    );
    assert!(
        !text.contains("@bytecode"),
        "every block of the statement is written: the call the inner body holds is protected by \
         the two named rows, and the handlers those rows name are the clauses already written \
         around it, so the block is no longer quoted:\n{text}"
    );
    assert!(
        !text.contains("throw new"),
        "the body is a call, so no `throw new` is part of this text:\n{text}"
    );
    assert_eq!(
        run_of(&report, "nest").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}

#[test]
fn a_handler_value_read_after_its_clause_is_still_refused() {
    // In the frozen class, `astore_2; aload_2; astore_1; aload_1; areturn` is legal:
    // the catch parameter is in slot 2 and the method local in slot 1. Reusing slot 1 for
    // the handler entry and its load makes the later `aload_1` consume the caught value.
    // The bytecode is verifiable, but spelling both as `local1` would leave a catch
    // parameter declared inside its clause and read after the clause.
    let old = [0x4d, 0x2c, 0x4c, 0x2b, 0xb0];
    let new = [0x4c, 0x2b, 0x4c, 0x2b, 0xb0];
    let positions: Vec<_> = CATCH_ESCAPE
        .windows(old.len())
        .enumerate()
        .filter_map(|(at, bytes)| (bytes == old).then_some(at))
        .collect();
    assert_eq!(
        positions.len(),
        1,
        "the frozen class has one handler sequence"
    );
    let mut bytes = CATCH_ESCAPE.to_vec();
    bytes[positions[0]..positions[0] + old.len()].copy_from_slice(&new);
    let sample = open(&bytes);
    let report = class_source_of(&sample, "CatchEscape");
    let text = text_of(&report, "read");
    assert!(
        text.contains("local 1 escapes catch parameter scope"),
        "the catch value is read outside its clause: {text}"
    );
    assert!(
        text.contains("@bytecode"),
        "the refusal preserves the code: {text}"
    );
    assert!(
        !text.contains("return local1;"),
        "no out-of-scope read: {text}"
    );
}
