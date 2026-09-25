//! P3 task 2c.19: `append(C)` is a character operand of a string `+`.
//!
//! `concat@1` presents a verified `StringBuilder`/`StringBuffer` chain as one `+` expression, and
//! `append` is overloaded: only the overloads whose text under `+` is the same text may be written
//! away (`keeps_its_conversion`). `append(char)` used to be refused on the reading that `+` on two
//! int-shaped operands adds numbers — which is true of a **binary** `+` between two primitives, and
//! not of a concatenation chain: the emitter already writes `"" +` when the first part is not a
//! `String` (`crate::emit`'s `Concat` arm), so `"" + c` is the one character the overload wrote and
//! the conversion is kept. `append(char[])`, which `+` would write as the array's own `toString`,
//! and `append(CharSequence)`, which `append` writes character by character, are still refused — the
//! predicate's own unit test in `crates/jarde-java/src/concat.rs` pins those.
//!
//! The sample is `tests/fixtures/p3-concat-char/` (javac 23.0.1 `--release 8 -g:none`; the command,
//! the 453 bytes and the SHA-256 are in its README):
//!
//! * `letter(char)` — `"x" + c`: `append(String)` (`ldc "x"`) and then `append(C)`. The first part is
//!   already a `String`, so the text needs no decoration and the character is concatenated:
//!   `return "x" + arg0;`;
//! * `only(char)` — `"" + c`: javac lowers this to `append("")` and then `append(C)`, so the empty
//!   string is a **part of its own** rather than the decoration the emitter writes for a first part
//!   needing conversion. The text keeps it (`return "" + arg0;`) and the character is never the bare
//!   `arg0` a lost conversion would leave.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 453 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-concat-char/v8/Letters.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of the committed sample, under the entry point the CLI calls.
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
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
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

/// The parameter type of every `append` of the one chain the member's body builds, in call order:
/// the pool fact the rule reads the overload from, as the record publishes it.
fn append_parameters(report: &RecoveryReport) -> Vec<String> {
    let chain = report
        .concats
        .first()
        .unwrap_or_else(|| panic!("the member's chain is recorded:\n{}", report.text));
    chain
        .appends
        .iter()
        .map(|append| append.parameter.clone())
        .collect()
}

#[test]
fn a_character_part_is_a_string_operand_of_the_chain() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Letters");

    // `letter`: the first part is already a `String`, so the text gains no empty string and the
    // `append(C)` that follows is the character it wrote — not the `int` the slot's shape suggests.
    let letter = text_of(&report, "letter");
    assert!(
        letter.contains("\"x\" + arg0"),
        "the character is concatenated after the `String` part:\n{letter}"
    );
    assert!(
        !letter.contains("append"),
        "the chain is written as `+`, not as the calls it made:\n{letter}"
    );
    assert!(
        !letter.contains("@bytecode") && !letter.contains("refused"),
        "the chain is presented, not quoted:\n{letter}"
    );
    assert_eq!(
        append_parameters(run_of(&report, "letter")),
        vec!["java.lang.String".to_string(), "char".to_string()],
        "the second `append` really names `C`: that is the overload this task accepts"
    );

    // `only`: `"" + c` is `append("")` then `append(C)`. The empty string is a part of the chain, so
    // the text keeps it — the character is never the bare value, which would be an `int` addition or
    // a return of the wrong type.
    let only = text_of(&report, "only");
    assert!(
        only.contains("\"\" + arg0"),
        "the chain keeps its own empty-string part and concatenates the character:\n{only}"
    );
    assert!(
        !only.contains("return arg0;"),
        "the conversion is not lost to a bare value:\n{only}"
    );
    assert_eq!(
        append_parameters(run_of(&report, "only")),
        vec!["java.lang.String".to_string(), "char".to_string()],
        "the empty string is a `String` part javac emitted, not decoration this layer added"
    );

    for (name, text) in [("letter", letter), ("only", only)] {
        assert_eq!(
            run_of(&report, name).content,
            RecoveryContent::ContainsStatements,
            "{name}:\n{text}"
        );
    }
}
