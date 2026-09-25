//! P3 2.11: `try (Res r = …) { … } catch (E e) { … }` — a resource header with a user `catch`.
//!
//! `read([B)I` is the commonest resource idiom there is: `try (InputStream in = new
//! ByteArrayInputStream(data)) { return in.read(); } catch (IOException e) { return -1; }`. javac
//! writes **four** exception-table rows for it, and only two of them are the user's `catch`:
//!
//! ```text
//!   from    to  target type
//!      9    14      20   Class java/lang/Throwable     <- the compiler's cleanup of the header
//!     21    25      28   Class java/lang/Throwable     <- ... and the close's own self-protection
//!      0    18      36   Class java/io/IOException     <- the source's `catch`, split in two
//!     20    36      36   Class java/io/IOException     <- ... because the code between cannot raise
//! ```
//!
//! **What refused the member.** Two things, and the first one is not the row union:
//!
//! * `crates/jarde-java/src/guard.rs`'s `close_handler` read one handler shape only —
//!   `astore p; aload r; ifnull L` with the close in the other successor — because `javac` guards
//!   the close with a null test whenever the resource's own initialisation **could** be null. This
//!   resource cannot: `new ByteArrayInputStream(data)` is non-null by construction, so the compiler
//!   writes the test nowhere and the handler is `astore p; aload r; invokevirtual close; goto L`.
//!   The proof stopped there (`jre_guard_handler`, BCI 20) before any row question was asked.
//! * `catches()` then asked for the statement's rows to begin at **one** instruction, and the
//!   block holds the union of two — the compiler's row begins at BCI 9, the user's at BCI 0 — so
//!   neither the guarded rule nor the typed `catch` read the shape, and the member was quoted whole.
//!
//! **What this change reads.** The unchecked close on both paths (`close_of_level` and
//! `normal_close`); the row partition — once the guarded rule **claimed** the block, the rows that
//! reach a handler it proved are the compiler's own and are no clause, so the block's clauses are
//! the rows that reach *other* handlers; and `twr`'s reading of the enclosing rows as the clauses of
//! a `try` the statement sits inside, which is the same accommodation `region.rs`'s `try_level`
//! makes at the region level.
//!
//! **What still stands, and why.** `javac` writes no branch at all for this statement: with no null
//! test there is no target, so the whole method body is **one** canonical block
//! `[0, 3, 4, 5, 8, 9, 10, 13, 14, 15, 18, 19]` — the header, the body, the close **and** the
//! `iload_2; ireturn` the `return in.read()` became once the value was kept in a local. A walk
//! continues at a *block*, and after the close the run is inside the statement's own block, so the
//! `return local2;` has nowhere to be written from and claiming the block would leave it out — the
//! silence P3-R7 exists to undo. The statement is therefore refused with that as its reason
//! (`jre_guard_continuation`), never presented with its tail dropped and never degraded into a
//! `catch` that would drop the resource. Writing the tail needs one more mechanism, the one P3 2.6
//! already has for `synchronized` (a `returns` on the shape, written **inside** the braces by
//! `build.rs`); the acceptance for that state is the second test below, marked `#[ignore]` until it
//! lands.
//!
//! The sample is `tests/fixtures/p3-twr-catch/v8/Combo.class` (see its `README.md` for the command,
//! the 550 bytes and the SHA-256).

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 550 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-twr-catch/v8/Combo.class");

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
fn the_unchecked_close_is_proved_and_the_statement_keeps_its_refusal_over_its_tail() {
    // The member's own report states what the proof concluded. What it must **not** say any more is
    // `jre_guard_handler`: this sample's handler is the `astore p; aload r; invokevirtual close;
    // goto L` shape the compiler writes for a resource its own initialisation proves non-null, and
    // reading it is the half of P3 2.11 the samples with a nullable resource never exercised.
    //
    // What stands is the tail. `javac` writes no branch for this statement, so the header, the body,
    // the close and the `iload_2; ireturn` of the source's `return in.read()` are **one** block: a
    // walk continues at a block, and after the close the run is inside the statement's own one.
    // Claiming the block would leave the `return` out of the artifact with nothing naming it, so the
    // rule refuses — and the reason it states is the tail, not the handler.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Combo");
    let run = run_of(&report, "read");
    assert!(
        run.fallbacks.contains(&"jre_guard_continuation"),
        "the refusal states the tail the walk cannot reach: {:?}",
        run.fallbacks
    );
    assert!(
        !run.fallbacks.contains(&"jre_guard_handler"),
        "the handler javac writes for a `new`-initialised resource is the one this rule proves: {:?}",
        run.fallbacks
    );
    let text = text_of(&report, "read");
    // The compiler's cleanup is the header's own: the synthetic `Throwable` handler is **never** a
    // user clause, and neither `addSuppressed` nor a `close()` call is written as source. The member
    // is refused whole rather than presented as a `try` without its resource.
    assert!(
        !text.contains("addSuppressed"),
        "the suppression belongs to the compiler's cleanup:\n{text}"
    );
    assert!(
        !text.contains("catch (java.lang.Throwable"),
        "the row that names `Throwable` is the header's own, not a clause the source wrote:\n{text}"
    );
    assert!(
        !text.contains("try ("),
        "a `try`-with-resources this build cannot present is never spelled as a bare `try`:\n{text}"
    );
    assert_eq!(
        run.content,
        RecoveryContent::ExplanationOnly,
        "the refusal is stated, and no statement of the member is presented under it:\n{text}"
    );
}

/// The acceptance P3 2.11 states, which the `returns` increment lands.
///
/// The statement's own text has to end **inside its braces** with the `return` the normal path
/// performs — `{ int local2 = local1.read(); return local2; }` — because the source's `return
/// in.read()` is one statement whose value the compiler keeps in a local while it closes the
/// resource. That is the mechanism P3 2.6 already has for `synchronized (lock) { return …; }` (a
/// `returns` on the shape, written by `build.rs`), and the clause around the statement is the one
/// `catches()` now partitions out of the header's own rows. Nothing here is dropped: the header, the
/// body's read, the return and the clause are all text, and the compiler's cleanup is stated as the
/// uncovered blocks it is.
#[test]
#[ignore = "needs the `returns` increment in build.rs (P3 2.6's mechanism, for the resource header)"]
fn a_resource_statement_and_the_catch_around_it_are_presented_together() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Combo");
    let text = text_of(&report, "read");
    let header = at(
        text,
        "try (java.io.ByteArrayInputStream local1 = new java.io.ByteArrayInputStream(arg0)) {",
    );
    let read = at(text, "int local2 = local1.read();");
    let returned = at(text, "return local2;");
    // The brace that closes the resource statement: the first one after the statement's own read.
    let closing = read
        + text[read..]
            .find('}')
            .expect("the resource statement is closed");
    let clause = at(text, "catch (java.io.IOException");
    let caught = at(text, "return -1;");
    assert!(
        header < read
            && read < returned
            && returned < closing
            && closing < clause
            && clause < caught,
        "the header declares the resource, the body reads it, the return the normal path performs \
         closes the statement, and the clause the source wrote follows it:\n{text}"
    );
    // The compiler's cleanup is claimed by the rule and never written: the closes at BCI 15 and 22,
    // the `addSuppressed` at 31 and the rethrow at 35 are what the header's own `try` performs for
    // the source — and the synthetic handler is not a clause.
    assert!(
        !text.contains("addSuppressed") && !text.contains("catch (java.lang.Throwable"),
        "the cleanup belongs to the header, not to the source:\n{text}"
    );
    assert_eq!(
        text.matches("try (").count(),
        1,
        "one resource, declared by one header:\n{text}"
    );
    assert_eq!(
        text.matches("local1.read()").count(),
        1,
        "the body holds one read of the resource:\n{text}"
    );
    assert_eq!(
        run_of(&report, "read").content,
        RecoveryContent::ContainsStatements,
        "the member's content plane is what a proved statement produces:\n{text}"
    );
}
