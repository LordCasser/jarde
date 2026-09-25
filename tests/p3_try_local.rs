//! P3 2.8: `try (r) { return r.read(); }` — the Java 9 resource the compiler copies into a local.
//!
//! `try (r)` names an expression rather than declaring a variable, and javac keeps its value in a
//! local of its own before the protected range (`aload r; astore copy`), which the body then reads.
//! The `twr@1` rule read the instruction before a row's protected range for the resource's own
//! **initialisation**, and the question it asked of that store — does what it holds come from a
//! `new` or an invocation? — has no true answer here: `aload r; astore copy` is neither, so the row
//! was no header at all, and the member was read as the `try`/`catch` its own table states. The copy
//! became a statement before the `try` (`java.io.Reader local1 = arg0;`), the protected range was
//! quoted, and the compiler's cleanup — the `close` and the `addSuppressed` — became the body of a
//! `catch (java.lang.Throwable local2)` the source never wrote, while the `return` the normal path
//! ends in was dropped with the block that holds it.
//!
//! The copy is a value of its own kind: the store's own statement is **one `Load` of another local**
//! (`crates/jarde-java/src/guard.rs`'s `copied_local`), the slot it writes is not the one it read,
//! and the header `crates/jarde-java/src/build.rs`'s `resource_declaration` already reads such a
//! range with is `try (java.io.Reader local1 = arg0)`. Claiming the row needs one more link from the
//! normal path: javac writes no `goto` when the body returns, so the close is the last instruction
//! of its own block, the continuation is the very next instruction, and the close's block has that
//! instruction as its only successor (`normal_close`). The handler is proved exactly as for every
//! other resource, and none of it is written: the row is a header, not a clause. And the row is
//! claimed only where the whole proof succeeds — the same store before an ordinary `catch` stays the
//! `catch` its own table names, which is what keeps `tests/p3_typed_catch.rs`'s texts as they are.
//!
//! What the text below pins over `tests/fixtures/p3-try-local/v9/Held.class` (see its `README.md` for
//! the command, the version and the digest):
//!
//! * the header declares the **copy** the compiler made — `try (java.io.Reader local1 = arg0)`: the
//!   type the frames give the copy, the name of the slot the store fills, and the value the store
//!   read — and the copy is written nowhere else;
//! * the body holds the read's own statement between the header and the brace that closes it, read
//!   once, and the `return` the normal path ends in is written after the statement: the body is the
//!   row's own range and the join is the instruction that follows the close;
//! * nothing of the compiler's cleanup is written — no `close`, no `addSuppressed`, no `catch` — and
//!   the member's content plane is what a proved statement produces.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 9 -g:none` (see the fixture's README for the
/// command, the 480 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-try-local/v9/Held.class");

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
                // The sample is a Java 9 class — `try (r)` is Java 9 syntax and its class file states
                // version 53 — so the run is asked for the release the header itself states.
                java_release: 9,
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

/// One byte sequence of the sample replaced by another, with the count of sites rewritten.
fn patched(bytes: &[u8], needle: &[u8], replacement: &[u8]) -> Vec<u8> {
    assert_eq!(needle.len(), replacement.len(), "a patch keeps every BCI");
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0usize;
    let mut sites = 0usize;
    while at < bytes.len() {
        if bytes[at..].starts_with(needle) {
            out.extend_from_slice(replacement);
            at += needle.len();
            sites += 1;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    assert!(sites > 0, "the sample holds the sequence this case patches");
    out
}

#[test]
fn a_copy_whose_proof_fails_leaves_the_row_the_catch_its_table_names() {
    // The same sample with one byte changed: the handler's null test at BCI 19 — the `aload_1` at 18
    // and the `ifnull 35` this case turns round — is no longer the `JumpIfNull` the close proof
    // reads, so `twr` fails on the row. What the row is *then* is the half of the rule no header can
    // state on its own: the store before it holds a local's value, not a `new` and not an
    // invocation, so the row is no initialisation this rule may refuse the member over — the failed
    // attempt leaves it exactly where it was, and the walk reads it as the `catch` its own table
    // names. A store that holds a `new` or an invocation keeps the refusal beside it
    // (`tests/p3_guard.rs` pins that side), and here the row is no header:
    // `java.io.Reader local1 = arg0;` before the `try`, and the compiler's own cleanup as the
    // clause's body.
    let sample = open(&patched(
        SAMPLE,
        &[0x2b, 0xc6, 0x00, 0x10],
        &[0x2b, 0xc7, 0x00, 0x10],
    ));
    let report = class_source_of(&sample, "Held");
    let text = text_of(&report, "use");
    let copy = at(text, "java.io.Reader local1 = arg0;");
    let opened = at(text, "try {");
    let clause = at(text, "} catch (java.lang.Throwable local2) {");
    assert!(
        copy < opened && opened < clause,
        "the copy is written before the `try` the row is a clause of:\n{text}"
    );
    assert!(
        !text.contains("try ("),
        "the row is no header where the close proof it needs is the byte this case changed:\n{text}"
    );
    // The member is not refused over the row either: a fallback the guarded rule itself states
    // (`jre_guard_*`) is exactly what the copy must keep the row out of.
    let run = run_of(&report, "use");
    assert!(
        !run.fallbacks
            .iter()
            .any(|code| code.starts_with("jre_guard_")),
        "the failed attempt leaves the row alone rather than refusing the member over it: {:?}\n{text}",
        run.fallbacks
    );
    assert_eq!(run.content, RecoveryContent::ContainsStatements, "{text}");
}

#[test]
fn a_java_nine_resource_is_declared_from_the_copy_the_compiler_made() {
    // `use(Ljava/io/Reader;)I` is `aload_0; astore_1; aload_0; invokevirtual read; istore_2;
    // aload_1; ifnull 15; aload_1; invokevirtual close; 15: iload_2; ireturn`, with the row
    // `[2, 7) → 17 Throwable` and the close's own row `[22, 26) → 29`. The store before the range
    // reads `arg0` into `local1`, and that store is the resource: the header declares the copy, the
    // body reads it, and the close javac performs is written by the `try` the header states — not by
    // a clause, and not as a call.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Held");
    let text = text_of(&report, "use");
    let header = at(text, "try (java.io.Reader local1 = arg0) {");
    let read = at(text, "int local2 = arg0.read();");
    // The brace that closes the statement: the first one after the body's own statement.
    let closing = read
        + text[read..]
            .find('}')
            .expect("the resource statement is closed");
    let returned = at(text, "return local2;");
    assert!(
        header < read && read < closing && closing < returned,
        "the copy is declared in the header, the body's read runs between the header and the \
         statement's closing brace, and the return the normal path ends in follows it:\n{text}"
    );
    // The copy is the header's declaration and nothing else: it is not also written as the statement
    // before the `try` the member used to be read as.
    assert!(
        !text.contains("local1 = arg0;"),
        "the copy is declared in the header rather than stored again before the statement:\n{text}"
    );
    // The read is the body's own statement, written once, where the bytecode runs it.
    assert_eq!(
        text.matches("arg0.read()").count(),
        1,
        "the body holds one read, and the text writes it once:\n{text}"
    );
    assert_eq!(
        text.matches("try (").count(),
        1,
        "one resource, declared by one header:\n{text}"
    );
    // The compiler's cleanup is claimed by the rule and never written: the closes at BCI 11-12 and
    // 22-23, the `addSuppressed` at 32 and the rethrow at 36 are what the header's own `try`
    // performs for the source, not statements the source holds.
    assert_eq!(
        text.matches("close(").count(),
        0,
        "the close is the compiler's, written by the header and not as a call:\n{text}"
    );
    assert!(
        !text.contains("addSuppressed"),
        "the suppression belongs to the compiler's cleanup, not to the source:\n{text}"
    );
    assert_eq!(
        text.matches("catch (").count(),
        0,
        "the row is a resource header, so no clause is written:\n{text}"
    );
    // A text with statements in it is `contains_statements`; the explanation-only answer this shape
    // used to get — its protected range quoted under a clause and its join dropped — is not.
    //
    // The one thing the walk does not claim is the compiler's own handler, which javac put **after**
    // the join this text writes (the return is at BCI 15 and the handler's blocks are 17, 22, 35 and
    // 29), so it is named as an uncovered block rather than written: the statement's own range ends
    // at the row's `to`, and nothing of the cleanup is a statement of the source.
    assert_eq!(
        run_of(&report, "use").fallbacks,
        vec!["jre_region_uncovered_blocks"],
        "{text}"
    );
    assert_eq!(
        run_of(&report, "use").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}
