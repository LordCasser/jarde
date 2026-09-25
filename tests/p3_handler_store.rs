//! P3 2.14: a `catch` clause's binding store is not a resource header.
//!
//! The first instruction of a handler is the store that binds the clause's parameter, and the value
//! it writes is the reference the handler was **entered** with — the exception an exception edge
//! hands it. `crates/jarde-java/src/guard.rs` read the store before a protected range as the
//! `try`-with-resources rule's own question ("is a resource initialisation here?") and answered
//! `true` wherever it could not read the store's statement inside the block: that is the answer for
//! a statement the proof cannot read, and it refused the whole clause body as a header
//! (`jre_guard_resource_init`). A binding store is the one case where the value *is* readable and is
//! no initialisation at all, so a row whose range begins after one is a clause's own body and is
//! left to the `try`/`catch` reading.
//!
//! `tests/fixtures/p3-handler-store/` holds the case with **several** edges into one handler: the
//! outer row's range covers a branch, so the handler entry is fed from three blocks, and the
//! reference the clause binds is the handler block's own entry value for the stack slot the JVM
//! hands the exception in rather than one `caught` definition. The sibling
//! `tests/fixtures/p3-stated-rows/` fixture's `nested(I)I` is the single-edge half of the same rule
//! and is pinned in `tests/p3_stated_rows.rs`.
//!
//! What the region **walk** writes inside those ranges is not this rule's, and it is not pinned
//! here: a stated row protects a block the walk may not write as a statement of the body, and the
//! walk then quotes the block and names it (P3 2.15, another change's reading — the two tests below
//! assert the rule, not that accounting). What is pinned is what the two clauses are and what the
//! member is **not**: no `jre_guard_*` diagnostic at all, no instruction outside the graph's
//! account, and no answer that explains the member instead of presenting it.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 331 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-handler-store/v8/HandlerStore.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

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
        other => panic!("one committed sample answers one definition, got {other:?}"),
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

/// The diagnostic codes of one member's own run, in the order they were reported.
fn codes_of<'a>(report: &'a ClassSourceReport, name: &str) -> Vec<&'a str> {
    run_of(report, name)
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

/// Where one substring sits in one member's text, stated as the panic a missing one deserves.
fn at(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("the text presents `{needle}`:\n{text}"))
}

#[test]
fn a_handlers_own_binding_store_is_not_a_resource_header() {
    // `two(I)I` is a `try`/`catch` whose `catch` body is itself a `try`/`catch`. The outer row
    // `[0, 15) → 18` covers the test block `[0, 4)` and both arms `[4, 8)` and `[11, 15)`, so the
    // handler entry at BCI 18 is entered from three exception edges; its first instruction
    // `astore_1` binds the clause's parameter, and the inner row `[19, 21) → 24` begins right after
    // that store. The store's own statement could not be read inside the block (its run ends at the
    // handler's entry), so the guarded rule read it as a resource header and refused the whole
    // clause body: the text was the outer clause holding
    // `// BCI 18: the resource's own initialisation is not one statement of this block whose value
    // lands in a slot` and no inner statement at all.
    //
    // The value that store writes is the reference the handler was entered with — not a `new`, not
    // an invocation — so the row is a clause's own body: the inner statement is written, with its
    // own clause and body, and no `try`-with-resources header is read into the clause.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "HandlerStore");
    let text = text_of(&report, "two");
    let outer_clause = at(text, "} catch (java.lang.RuntimeException local1) {");
    let inner_open = at(text, "            try {");
    let inner_clause = at(
        text,
        "} catch (java.lang.IllegalArgumentException local2) {",
    );
    let inner_body = at(text, "arg0 = -2;");
    let returned = at(text, "return arg0;");
    assert!(
        outer_clause < inner_open && inner_open < inner_clause && inner_clause < inner_body,
        "the outer clause's body is the inner statement, written with its own clause and body:\n\
         {text}"
    );
    assert!(
        inner_body < returned,
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
        "one clause per row that names a `catch` type, in exception-table order:\n{text}"
    );
    let codes = codes_of(&report, "two");
    assert!(
        !codes.contains(&"jre_guard_resource_init"),
        "the store before the inner range binds the clause's parameter, so it is no resource \
         initialisation and no header is proved or refused from it: {codes:?}\n{text}"
    );
    assert_eq!(
        run_of(&report, "two").content,
        RecoveryContent::ContainsStatements,
        "the member is presented, not explained only:\n{text}"
    );
}

#[test]
fn no_instruction_of_the_member_is_outside_the_graphs_account() {
    // What the two rows state is *presented*, and the body is not quoted for the defect P3 2.9
    // fixed: every instruction of the member sits in a block the walk reached. What is left over is
    // the region walk's own accounting, which this build states rather than hides — a `try` body is
    // written only where the walk may write it, so a block a stated row protects without covering
    // its start, or one reached only through an edge the normal flow leaves out, is quoted and the
    // live block is named (`jre_region_exception_edge`, `jre_region_uncovered_blocks`). Both belong
    // to P3 2.15 and move with it; this test pins only that no instruction of this method is outside
    // the graph's account and that the member is never answered with an explanation alone.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "HandlerStore");
    let text = text_of(&report, "two");
    assert!(
        !text.contains("jre_region_unaccounted_instruction"),
        "defect guard: no instruction of this body is outside the graph's account:\n{text}"
    );
    assert!(
        !text.contains("jarde: not recovered"),
        "the member is presented, not explained:\n{text}"
    );
    let codes = codes_of(&report, "two");
    assert!(
        !codes.iter().any(|code| code.starts_with("jre_guard_")),
        "no guarded rule claims or refuses anything of this method — its rows are clauses, not \
         resource headers: {codes:?}\n{text}"
    );
}
