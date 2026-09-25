//! P3 2.6: `synchronized (this) { return n; }` — a `return` inside the braces.
//!
//! `monitor()` in `crates/jarde-java/src/guard.rs` reads the instruction after the normal
//! `monitorexit` and required it to be the `goto` javac writes when the run continues after the
//! statement. When the body returns, javac writes no `goto`: the value the body left on the stack is
//! returned straight through the exit, and the exception table's row ends *at* that `return`. The
//! rule refused the whole member as `jre_guard_monitor`, and the handler — whose range starts after
//! the return — became an uncovered block.
//!
//! What the presented text must pin:
//!
//! * the `return` is written **inside** the braces, after the header and before the statement's
//!   closing brace — not moved after the statement, where it would run after the exit;
//! * the value it names is the body's own read, written where the value is returned: `this.n`
//!   occurs once, so the read is not repeated and the `getfield` is not quoted beside the `return`;
//! * nothing is invented for that value — no local is declared, and no `+=` or second read appears;
//! * the member's content plane is `ContainsStatements`, which is what a proved statement produces.
//!
//! The control in the other direction is `tests/fixtures/p3-handlers/v8/Guarded.class`'s `sync` and
//! `syncThrows` — the `goto` shape — whose texts `tests/p3_guard.rs` pins unchanged.

use jarde::*;
use std::collections::BTreeSet;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 285 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-sync-return/v8/Locked.class");
const EFFECTS: &[u8] =
    include_bytes!("fixtures/preserve-guarded-return-expression/v8/GuardReturnEffects.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of one committed sample, under the entry point the CLI calls.
///
/// Every evidence category is asked for, so the member's own report carries the segment table the
/// anchors below are read from; the text and the content plane are the same ones the plain
/// `Engine::class_source` presents.
fn class_source_of(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceReport {
    class_source_with_evidence(snapshot, name, &RecoveryEvidenceRequest::all())
}

fn class_source_with_evidence(
    snapshot: &ArtifactSnapshot,
    name: &str,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
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
        .class_source_with_evidence(slice::from_ref(snapshot), &request, evidence, &mut budget())
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

/// One member's own record in the assembled source.
fn method_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the sample's method table"))
}

#[test]
fn a_return_inside_a_synchronized_block_is_written_between_its_braces() {
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Locked");
    let locked = method_of(&report, "locked");
    let text = &locked.text;
    // The whole text is pinned, so the statement is where it is written and nowhere else.
    assert_eq!(
        text,
        "    int locked() {\n        // @method locked()I\n        // @declaration an instance method of `Locked`, member flags 0x0000\n        // recovered from bytecode; presentation is not claimed to compile\n        synchronized (this) {\n            return this.n;\n        }\n    }\n",
        "the member's own text"
    );
    assert!(
        text.contains("synchronized (this)"),
        "the monitor shape is presented as the statement it is:\n{text}"
    );
    assert!(
        text.contains("return this.n;"),
        "the body's own read is returned by name, once:\n{text}"
    );
    // The `return` belongs to the body: it is written after the header and before the statement's
    // own closing brace — the value the body left on the stack is returned across the exit, so the
    // `return` runs while the monitor is still held.
    let header = text
        .find("synchronized (this) {")
        .expect("the statement's header is written");
    let returns = text.find("return this.n;").expect("the return is written");
    let closing = returns
        + text[returns..]
            .find("\n        }")
            .expect("the statement's own closing brace follows the return");
    assert!(
        header < returns && returns < closing,
        "the return sits between the header and the statement's closing brace:\n{text}"
    );
    // The read is the one the bytecode performed: it is written once, at the return, and the
    // `getfield` beside it is claimed rather than quoted — a second mention would be a second read.
    assert_eq!(
        text.matches("this.n").count(),
        1,
        "the field is read once and written where its value is returned:\n{text}"
    );
    // Nothing is invented for the value: no local, and no second read or compound assignment — the
    // text pinned above is the whole of what the member presents.
    assert!(
        !text.contains("local"),
        "no local is declared for the returned value:\n{text}"
    );
    // The method is presented, not quoted, and its own content plane says so.
    assert!(
        locked.markers.is_empty(),
        "the member carries no marker: {:?}\n{text}",
        locked.markers
    );
    let ClassSourceOutcome::Recovered { report, .. } = &locked.outcome else {
        panic!("the member's own run produced a body: {:?}", locked.outcome);
    };
    assert_eq!(
        report.content,
        RecoveryContent::ContainsStatements,
        "a proved statement is statements, not an explanation:\n{text}"
    );
    // The instructions the proof read are anchors of the statement's own text, and the `return` is
    // one of them: which instruction the returned value came from is answerable from the artifact.
    let anchored: BTreeSet<u32> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis())
        .collect();
    for bci in [3, 5, 8, 9, 10, 11, 13] {
        assert!(
            anchored.contains(&bci),
            "BCI {bci} is an anchor of the statement: {anchored:?}"
        );
    }
}

#[test]
fn essential_and_all_evidence_keep_the_same_member_body() {
    let sample = open(SAMPLE);
    let all = class_source_of(&sample, "Locked");
    let essential =
        class_source_with_evidence(&sample, "Locked", &RecoveryEvidenceRequest::essential());
    assert_eq!(
        method_of(&all, "locked").text,
        method_of(&essential, "locked").text,
        "optional evidence does not select another body or presentation"
    );
}

#[test]
fn independent_effects_and_unproved_nested_monitors_do_not_receive_the_exit_exemption() {
    let report = class_source_of(&open(EFFECTS), "GuardReturnEffects");
    let effect = method_of(&report, "effect");
    assert!(
        effect.text.contains("@bytecode 0 21 24 28"),
        "the value, call and monitor exits stay represented by their fallback anchors:\n{}",
        effect.text
    );
    assert!(
        !effect.text.contains("synchronized (this)"),
        "an independent call inside the body cannot be discarded by the return exemption:\n{}",
        effect.text
    );

    let nested = method_of(&report, "nested");
    assert!(
        nested.text.contains("@bytecode 0 28 31 38 45"),
        "a nested monitor not proved by the single-monitor rule remains anchored:\n{}",
        nested.text
    );
    assert!(
        !nested.text.contains("synchronized (this)"),
        "an unproved nested monitor cannot borrow another Guard's exit identity:\n{}",
        nested.text
    );
}
