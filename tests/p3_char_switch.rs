//! P3 task 2c.12: a `char` selector's `switch` keys are character literals.
//!
//! `switch (c) { case 'a': … }` over a `char` and `switch (n) { case 97: … }` over an `int` are
//! different programs, and both compile a `lookupswitch` whose payload states the key as the number
//! `97`: the class file cannot say whether the source of a `char` switch wrote the character or the
//! number, because `case 97:` over a `char` is legal and means the same character. What the class
//! file does state is the *selector's type* — for a parameter it is the member's own descriptor,
//! the one fact that tells a `char` from an `int` — and the text writes the key as that type: a
//! selector whose text already presents `char` writes `case 'a':`, and every other selector keeps
//! the decimal number the payload states. The type is read off the expression the emitter was
//! given (`Expr::presented`) and never inferred from the key's own value, so an `int` selector whose
//! key happens to be `97` stays decimal.
//!
//! The sample is `tests/fixtures/p3-char-switch/` (javac 23.0.1 `--release 8 -g:none`; the command,
//! the 301 bytes and the SHA-256 are in its README):
//!
//! * `letter(char)` — `iload_0; lookupswitch {97 → 28, 98 → 30, default → 32}`: the selector's text
//!   is the `char` parameter, so its keys are `case 'a':` and `case 'b':`;
//! * `number(int)` — the same shape over an `int` parameter: the key stays `case 97:`. This is the
//!   control that keeps the rule from becoming a value heuristic.

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 301 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-char-switch/v8/Letters.class");

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

#[test]
fn a_char_selector_writes_character_keys_and_an_int_selector_stays_decimal() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Letters");

    // `letter`: the selector is the `char` parameter, so its keys are the characters the payload's
    // numbers are — and the member is a `switch`, not a quote of the instruction that selects.
    let letter = text_of(&report, "letter");
    for literal in ["case 'a':", "case 'b':"] {
        assert!(
            letter.contains(literal),
            "a `char` selector writes `{literal}`:\n{letter}"
        );
    }
    assert!(
        !letter.contains("case 97:") && !letter.contains("case 98:"),
        "no key of a `char` selector is left as the payload's number:\n{letter}"
    );
    assert!(
        !letter.contains("@bytecode") && !letter.contains("refused"),
        "the switch is presented, not quoted:\n{letter}"
    );
    assert_eq!(
        run_of(&report, "letter").content,
        RecoveryContent::ContainsStatements,
        "{letter}"
    );

    // `number`: the one thing that differs is the selector's presented type — the key is the same
    // 97 — and that type is what decides the spelling. Nothing here may become a character literal.
    let number = text_of(&report, "number");
    assert!(
        number.contains("case 97:"),
        "an `int` selector keeps the decimal key:\n{number}"
    );
    assert!(
        !number.contains("case '"),
        "an `int` selector is not a `char` one because its key is 97:\n{number}"
    );
    assert!(
        !number.contains("@bytecode") && !number.contains("refused"),
        "the switch is presented, not quoted:\n{number}"
    );
    assert_eq!(
        run_of(&report, "number").content,
        RecoveryContent::ContainsStatements,
        "{number}"
    );

    // Both members keep the shape the change does not touch: the selector is written once in its own
    // parentheses and the no-match arm is the arm the payload states. (`javac` puts a `return` in
    // every arm here, so no arm of either member carries a `break` — the change writes no label of
    // its own and adds no statement.)
    for text in [letter, number] {
        assert!(
            text.contains("switch (arg0) {"),
            "the selector is written once, in its own parentheses:\n{text}"
        );
        assert!(
            text.contains("default:"),
            "the no-match arm stays the arm the payload states:\n{text}"
        );
    }
}
