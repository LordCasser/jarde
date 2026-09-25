//! P3 2.9: the exception table is the artifact's own statement of what a row protects.
//!
//! `javac --release 8` protects the instructions a `try` body runs without asking whether any of
//! them can raise: `try { n = n + 1; } catch (RuntimeException e)` is `iload_0; iconst_1; iadd;
//! istore_0` under a row whose range holds no throwing instruction, and the same holds for the
//! `try { n = n * 2; }` of `steps`. The graph used to build an `Exception` edge only at a
//! `may_throw` instruction site, and the canonical handler rows were read out of those sites, so a
//! row with no site was invisible to the graph and the `catch` clause it declares was not written.
//!
//! The rule now is the table's own statement, read once per record: a record a `may_throw`
//! instruction of the body covers keeps the edges its sites feed, exactly as before, and a record
//! **no** such instruction covers is stated by its protected range — one `Exception` edge from
//! every block that range intersects, with the handler entry a leader either way. The handler rows'
//! `protected` is read out of those edges — one source of truth for what the graph says is
//! protected. `throw_sites` stays the runtime fact: only real `may_throw` instructions are listed
//! and a site-less range invents no entry.
//!
//! `steps` is the case the rule is about, and it is pinned whole: two sequential `try`/`catch`
//! statements, neither of whose ranges holds a throwing instruction, both presented — where before
//! the second handler's bytes were in no canonical block, `jre_region_unaccounted_instruction`
//! fired, and the whole body was quoted as `// @bytecode 0 10 21 17 18 20`.
//!
//! `nested` is the case whose **row is stated and whose clause was not written**: the inner `try`'s
//! range `[8, 10)` begins inside the block that holds the outer clause's own binding store
//! (`astore_1` at BCI 7, the handler entry), and the guarded-region rule that reads the store before
//! a protected range (`jarde_java::guard::resources`) refused the block as a resource header because
//! that store's statement cannot be read inside the block. P3 2.14 reads that store as what it is —
//! a `catch` clause's binding, whose value is the reference the handler was entered with — so the
//! row is a clause's own body and the inner statement is written: both `try`s and both `catch (`s of
//! this table are in the text, where the defect produced `arg0 = arg0 + 1; return arg0;` with no
//! marker at all. What the region **walk** writes inside the inner range — whether the block at
//! BCI 7, which holds that binding store and which the inner row `[8, 10)` does not start with, is
//! written as `arg0 = -1;` or quoted — is P3 2.15's accounting, another change's reading, and this
//! file does not pin it: what it pins is that both clauses are stated and that the member is
//! presented.
//!
//! The texts below pin the shape over the class bytes committed in
//! `tests/fixtures/p3-stated-rows/` (see its `README.md` for the command, the version and the
//! digest).

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 443 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-stated-rows/v8/Stated.class");

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
fn a_row_whose_range_holds_no_throwing_instruction_is_still_a_catch() {
    // `steps(I)I` is two sequential `try`/`catch` statements whose rows are
    // `[0, 4) → 7 RuntimeException` and `[10, 14) → 17 IllegalStateException`. The first range holds
    // `iload_0; iconst_1; iadd; istore_0` and the second `iload_0; iconst_2; imul; istore_0`: the
    // opcode table calls none of those instructions throwing, and the table states the rows anyway.
    // The second row's handler is what the defect lost first: its bytes (BCI 17, 18, 20) were in no
    // canonical block and in no dead-node list, so the whole body was quoted as
    // `// @bytecode 0 10 21 17 18 20` under `jre_region_unaccounted_instruction`.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Stated");
    let text = text_of(&report, "steps");
    // Both statements, in the order the source wrote them, each clause naming the row's own type.
    let first_open = at(text, "try {");
    let first_clause = at(text, "} catch (java.lang.RuntimeException local1) {");
    let first_body = at(text, "arg0 = -1;");
    let second_clause = at(text, "} catch (java.lang.IllegalStateException local1) {");
    let second_body = at(text, "arg0 = -2;");
    let returned = at(text, "return arg0;");
    assert!(
        first_open < first_clause && first_clause < first_body,
        "the first `try` holds the increment and its clause holds `-1`:\n{text}"
    );
    assert!(
        first_body < second_clause && second_clause < second_body,
        "the second statement follows the first, and its own clause names \
         `IllegalStateException`:\n{text}"
    );
    assert!(
        second_body < returned,
        "the method's own `return arg0;` follows both statements:\n{text}"
    );
    assert_eq!(
        text.matches("try {").count(),
        2,
        "both protected ranges are presented, and neither statement twice:\n{text}"
    );
    assert_eq!(
        text.matches("catch (").count(),
        2,
        "one clause per row, in exception-table order:\n{text}"
    );
    // Each `try` body holds its own statement and nothing else: the increment is inside the first,
    // the multiply inside the second — the two ranges' instructions are not merged into one run.
    let incremented = at(text, "arg0 = arg0 + 1;");
    let doubled = at(text, "arg0 = arg0 * 2;");
    assert!(
        first_open < incremented && incremented < first_clause,
        "`arg0 = arg0 + 1;` is the first statement's body:\n{text}"
    );
    assert!(
        first_body < doubled && doubled < second_clause,
        "`arg0 = arg0 * 2;` is the second statement's body:\n{text}"
    );
    // The member is presented, not explained: no quote of the body and no explanation-only answer.
    assert!(
        !text.contains("@bytecode"),
        "every instruction of this body is accounted for by a block:\n{text}"
    );
    assert!(
        !text.contains("jarde: not recovered"),
        "the member is presented, not explained:\n{text}"
    );
    assert_eq!(
        run_of(&report, "steps").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}

#[test]
fn a_nested_catch_is_stated_instead_of_silently_dropped() {
    // `nested(I)I` is a `try`/`catch` whose `catch` body is itself a `try`/`catch`: the outer row is
    // `[0, 4) → 7 RuntimeException` and the inner one `[8, 10) → 13 IllegalArgumentException`, and
    // neither range holds a throwing instruction. Both rows used to be invisible to the graph, whose
    // only node was the fused normal path `[0, 19)`: the text was `arg0 = arg0 + 1; return arg0;`
    // and the bytecode's two `catch` clauses had **no marker at all** — the presentation claimed a
    // program that is not this body's.
    //
    // With the rule the rows are edges and the outer clause is written; with P3 2.14 the inner
    // statement is written too. Its range begins inside the block that holds the outer clause's own
    // binding store (`astore_1` at BCI 7), and that store is what the guarded rule reads before it
    // asks whether a resource header is there: its value traces to `Definition::Caught`, so it is
    // the clause's own parameter and no resource's initialisation, and the row is left to the
    // `try`/`catch` reading. Before that reading the store failed
    // `initialises_resource`'s conservative fallback (its statement cannot be grown inside the
    // block) and the whole clause body was refused as `jre_guard_resource_init`.
    //
    // What the region **walk** writes inside the inner range is not part of this pin: the block at
    // BCI 7 holds the binding store and the inner row `[8, 10)` does not start with it, so the walk
    // may write `arg0 = -1;` there or quote the block where `arg0 = -1;` runs, and that accounting
    // is P3 2.15's. The outer range's own statement (`arg0 = arg0 + 1;`) is written either way, and
    // so is the inner clause's body (`arg0 = -2;`, outside the inner range) — the clause levels are
    // what this test is about.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Stated");
    let text = text_of(&report, "nested");
    let outer_clause = at(text, "} catch (java.lang.RuntimeException local1) {");
    let outer_open = at(text, "try {");
    let added = at(text, "arg0 = arg0 + 1;");
    let inner_open = at(text, "\n            try {");
    let inner_clause = at(
        text,
        "} catch (java.lang.IllegalArgumentException local2) {",
    );
    let inner_body = at(text, "arg0 = -2;");
    let returned = at(text, "return arg0;");
    assert!(
        outer_open < added && added < outer_clause,
        "the outer protected range is presented, and the clause follows it:\n{text}"
    );
    assert!(
        outer_clause < inner_open && inner_open < inner_clause && inner_clause < inner_body,
        "the outer clause's body is the inner statement: its `try` and its own clause are written \
         inside the outer clause, and `-2` is the inner clause's body:\n{text}"
    );
    assert!(
        inner_body < returned,
        "the method's own return follows both statements:\n{text}"
    );
    // The binding store itself is written nowhere: the inner statement stands where its own range
    // begins, and the store the range follows is the outer clause's parameter, which the clause
    // header already declares (`local1`). The clause levels are therefore the whole of what this
    // test asserts, and no guarded rule speaks about this method any more: the store is a binding,
    // so no `try`-with-resources header is proved or refused from it.
    let run = run_of(&report, "nested");
    let codes: Vec<&str> = run
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect();
    assert!(
        !codes.iter().any(|code| code.starts_with("jre_guard_")),
        "no guarded rule claims or refuses anything of this method: {codes:?}\n{text}"
    );
    assert!(
        !text.contains("jre_region_unaccounted_instruction"),
        "defect guard: no instruction of this body is outside the graph's account any more:\n{text}"
    );
    assert_eq!(
        run.content,
        RecoveryContent::ContainsStatements,
        "the member is presented, not explained only:\n{text}"
    );
}
