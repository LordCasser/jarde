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
//! The active run proves the resource initializer and refuses the close handler at BCI 20 with
//! `jre_guard_handler`. The bytecode handler is `astore p; aload r; invokevirtual close; goto L`,
//! but Guard does not establish that shape here, so it never reaches the enclosing user-catch rows
//! or normal return tail. Site preplanning advances the refusal from initializer analysis to this
//! handler; it does not change the handler bytecode. The old `jre_guard_continuation` expectation was
//! stale because that later proof stage is not reached. The active test requires no partial TWR, no
//! disguised catch, and an explanation retaining physical block-entry BCIs. The ignored test below
//! remains a future acceptance state and makes no claim about current recovery.
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
fn the_unchecked_close_handler_is_refused_without_partial_twr() {
    // The member's own report states where the proof stops: initializer accepted, handler refused.
    //
    // What stands is the tail. `javac` writes no branch for this statement, so the header, the body,
    // the close and the `iload_2; ireturn` of the source's `return in.read()` are **one** block: a
    // walk continues at a block, and after the close the run is inside the statement's own one.
    // Claiming the block would leave the `return` out of the artifact with nothing naming it, so the
    // rule refuses. The handler refusal prevents Guard from reaching the continuation question.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Combo");
    let run = run_of(&report, "read");
    assert!(
        run.fallbacks.contains(&"jre_guard_handler"),
        "the unchecked close handler remains explicitly refused: {:?}; {:?}",
        run.fallbacks,
        run.diagnostics
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
    assert!(
        !text.contains("catch (java.io.IOException")
            && !text.contains("catch (java.lang.Throwable"),
        "neither the user's clause nor compiler cleanup is disguised as a recovered catch:\n{text}"
    );
    assert_eq!(
        run.content,
        RecoveryContent::ExplanationOnly,
        "the refusal is stated, and no statement of the member is presented under it:\n{text}"
    );
    assert!(
        text.contains("@bytecode 0 20 28 34 36"),
        "the explanation retains physical block-entry BCIs:\n{text}"
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
