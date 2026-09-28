//! P3 2.8: `try (r) { return r.read(); }` — the Java 9 resource the compiler copies into a local.
//!
//! The committed class proves a resource header from `aload r; astore copy`, its normal close,
//! and its cleanup handler. The recovered read and return both run inside the `try` body; Java
//! closes the resource before the return completes. The compiler's close and suppression calls
//! are not written as source statements.
//!
//! A one-byte handler mutation breaks that proof. Its copy crosses a quoted fallback region, so
//! the recovery conservatively preserves the bytecode boundary and explains why it cannot write
//! a lexically valid statement for the copy. It must not claim a resource header for that input.

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
fn a_copy_whose_proof_fails_does_not_claim_a_resource_header() {
    // The same sample with one byte changed: the handler's null test at BCI 19 — the `aload_1` at 18
    // and the `ifnull 35` this case turns round — is no longer the `JumpIfNull` the close proof
    // reads, so `twr` fails on the row. The changed handler and the copy crossing its quoted
    // fallback prevent a sound lexical definition-use slice. The result must not claim a resource
    // header or silently omit the copy; it retains the exact boundary as explanation-only text.
    let sample = open(&patched(
        SAMPLE,
        &[0x2b, 0xc6, 0x00, 0x10],
        &[0x2b, 0xc7, 0x00, 0x10],
    ));
    let report = class_source_of(&sample, "Held");
    let text = text_of(&report, "use");
    assert!(text.contains("@bytecode 0 11 15 17 22 29 35"), "{text}");
    assert!(
        text.contains("local 1 crosses a quoted fallback region"),
        "{text}"
    );
    assert!(
        !text.contains("try ("),
        "the row is no header where the close proof it needs is the byte this case changed:\n{text}"
    );
    // The guard does not claim a failed resource initialization; the region walk gives the
    // precise exception-edge and uncovered-block reasons for quoting the member.
    let run = run_of(&report, "use");
    assert!(
        !run.fallbacks
            .iter()
            .any(|code| code.starts_with("jre_guard_")),
        "the failed attempt does not refuse the member as a resource: {:?}\n{text}",
        run.fallbacks
    );
    assert_eq!(
        run.fallbacks,
        vec!["jre_region_exception_edge", "jre_region_uncovered_blocks"],
        "{text}"
    );
    assert_eq!(run.content, RecoveryContent::ExplanationOnly, "{text}");
}

#[test]
fn a_java_nine_resource_is_declared_from_the_copy_the_compiler_made() {
    // `use(Ljava/io/Reader;)I` is `aload_0; astore_1; aload_0; invokevirtual read; istore_2;
    // aload_1; ifnull 15; aload_1; invokevirtual close; 15: iload_2; ireturn`, with the row
    // `[2, 7) → 17 Throwable` and the close's own row `[22, 26) → 29`. The store before the range
    // reads `arg0` into `local1`, and that store is the resource: the header declares the copy, the
    // body reads it, and the close javac performs is written by the `try` the header states — not by
    // a clause, and not as a call. A return inside the resource body closes it before returning.
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
        header < read && read < returned && returned < closing,
        "the copy is declared in the header, and the read and return run inside its body:\n{text}"
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
    // The proved resource statement owns the compiler's handler and the normal close, so no
    // fallback is needed for those blocks.
    assert!(run_of(&report, "use").fallbacks.is_empty(), "{text}");
    assert_eq!(
        run_of(&report, "use").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}
