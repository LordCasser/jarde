//! The catch parameter's reads present the type the clause declares, not the slot's decision.
//!
//! `try (S1 r = new S1()) { touch(r); } catch (IllegalStateException e) { return tag(e); }` has
//! javac write the catch parameter into the **resource's own slot 0**: the binding `astore_0` at
//! BCI 38 overwrites the resource's store at BCI 7, and the slot stays one variable the plan
//! decided once — from its first write, the resource's `S1`. The clause header spells the row's
//! own `IllegalStateException` either way, so before the value-attribution rule a read of the
//! parameter inside the handler presented `S1` and the invocation of `tag` was refused with `no
//! safe reference conversion evidence` — the handler body was quoted while the header stood. The
//! read of a clause's own binding presents the type the handler was entered with (the row's class,
//! the entry reference's own type), so the invocation recovers; the no-reuse control's decision
//! already stated the row's class and its text is unchanged.
//!
//! The sample is `tests/fixtures/p3-caught-value-argument-typing/v8/S1.class` (see its `README.md`
//! for the command, the 1221 bytes and the SHA-256).

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g` (see the fixture's README for the command,
/// the 1221 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-caught-value-argument-typing/v8/S1.class");

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

/// One member's own text in the assembled source.
fn text_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a str {
    &report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
        .text
}

#[test]
fn the_slot_reusing_handlers_argument_presents_the_rows_own_type() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "S1");
    let text = text_of(&report, "twrHelper");
    // The clause header already stated the row's own type before this rule; the handler body now
    // states the invocation with the parameter the header declares.
    assert!(
        text.contains("catch (java.lang.IllegalStateException local0)"),
        "the clause header spells the row's own type:\n{text}"
    );
    assert!(
        text.contains("return tag(local0);"),
        "the handler's invocation of `tag` is presented with the parameter:\n{text}"
    );
    assert!(
        !text.contains("no safe reference conversion evidence"),
        "the slot's own decision never reaches the binding's argument again:\n{text}"
    );
}

#[test]
fn the_no_reuse_controls_text_is_unchanged() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "S1");
    let text = text_of(&report, "plainHelper");
    // The control's parameter owns its slot alone: the plan's decision and the binding's type are
    // the same answer, so the presentation states the invocation exactly as it always has.
    assert!(
        text.contains("try {"),
        "the control has no resource header to state:\n{text}"
    );
    assert!(
        text.contains("return tag(local0);"),
        "the control's invocation is presented as it always was:\n{text}"
    );
    assert!(
        !text.contains("// @bytecode"),
        "the control carries no quoted fallback:\n{text}"
    );
}
