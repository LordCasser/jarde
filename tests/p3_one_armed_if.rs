//! P3: a two-successor branch whose **one successor is the immediate post-dominator** is an `if`
//! with no `else`, not a refusal.
//!
//! `javac --release 8 -g:none` writes `if (n > 0) x = n;` as `iload_0; ifle <join>; iload_0;
//! istore_1; <join>`: the arm is the **fall-through** successor and the successor the branch
//! transfers to *is* the block after the `if`. The subset models two arms, and the region walk
//! refused this shape as `ArmsDoNotMeet` — an arm was read as a region that *walks into* the join, so
//! a branch whose successor already **is** the join had no arm to walk. The member was then
//! explanation-only: the prefix and the branch were quoted under that reason, the arm's block and the
//! block after the `if` were quoted as uncovered, and no statement of the body was written.
//!
//! The texts below pin all three shapes over the class bytes committed in
//! `tests/fixtures/p3-one-armed-if/` (see its `README.md` for the command, the version and the
//! digest): the taken successor being the join (`oneArmed`), the fall-through successor *not* being
//! the join with an empty arm of its own (`elseOnly` — the two-arm path, where the arm the branch
//! transfers to holds the assignment), and two real arms (`bothArms`, which the change must not
//! touch).

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 8 -g:none` (see the fixture's README for the
/// command, the 331 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-one-armed-if/v8/OneArmed.class");

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
fn a_branch_whose_taken_successor_is_the_join_is_an_if_with_no_else() {
    // `oneArmed(I)I` is `iconst_1; istore_1; iload_0; ifle 8; iload_0; istore_1; iload_1; ireturn`:
    // the branch transfers to the block after the `if`, so its arm is the fall-through one and the
    // statement has no `else`. Before this change the member was explanation-only — "the arms of the
    // branch in block 0 do not meet" with blocks 6 and 8 quoted as uncovered.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "OneArmed");
    let text = text_of(&report, "oneArmed");
    let assignment = at(text, "local1 = 1;");
    let if_at = at(text, "if (arg0 > 0) {");
    let arm = at(text, "local1 = arg0;");
    let returned = at(text, "return local1;");
    assert!(
        assignment < if_at && if_at < arm && arm < returned,
        "the statements keep their order — assignment before the `if`, the arm inside it, the \
         return after it:\n{text}"
    );
    assert!(
        !text.contains("else"),
        "a one-armed branch is written without an `else`:\n{text}"
    );
    assert!(
        !text.contains("jarde: not recovered"),
        "the member is presented, not explained:\n{text}"
    );
    // What the member holds is read off the run's own plane: a text with statements in it is
    // `contains_statements`, and the explanation-only answer this shape used to get is not.
    assert_eq!(
        run_of(&report, "oneArmed").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}

#[test]
fn the_arm_the_branch_transfers_to_keeps_its_else() {
    // `elseOnly(I)I` states the opposite arm: `ifle 9; goto 11; iload_0; istore_1; …`. The **empty**
    // then arm is the block that jumps to the join, and the assignment is in the arm the branch
    // transfers to — which is not the join, so this is the two-arm path and the `else` stays.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "OneArmed");
    let text = text_of(&report, "elseOnly");
    let if_at = at(text, "if (arg0 > 0) {");
    // The `else` **keyword**, not the `elseOnly` the member's own declaration spells.
    let else_at = at(text, "} else {");
    let arm = at(text, "= arg0");
    assert!(
        if_at < else_at && else_at < arm,
        "the assignment is inside the `else` arm:\n{text}"
    );
    assert_eq!(
        run_of(&report, "elseOnly").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}

#[test]
fn two_real_arms_are_still_two_arms() {
    // The control: `bothArms(I)I` assigns on both sides of the branch (`ifle 9; iconst_2; …; goto 13;
    // iconst_3; …`), and neither successor is the join. Nothing about that shape changed.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "OneArmed");
    let text = text_of(&report, "bothArms");
    let then = at(text, "local1 = 2;");
    let else_at = at(text, "} else {");
    let other = at(text, "local1 = 3;");
    assert!(
        then < else_at && else_at < other,
        "both arms keep their own body:\n{text}"
    );
    assert_eq!(
        run_of(&report, "bothArms").content,
        RecoveryContent::ContainsStatements,
        "{text}"
    );
}
