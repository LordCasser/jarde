//! P3: a branch whose two successors converge on a **forward join** is an `if`, not a loop.
//!
//! `javac --release 8 -g:none` writes `if (a > 0 && b > 0) return 1; return 0;` as two forward
//! branches onto one block: `iload_0; ifle <join>; iload_1; ifle <join>; iconst_1; ireturn; <join>`.
//! The shared block is the outer branch's join and is **not** its immediate post-dominator, because
//! the `return 1` path leaves the method before passing through it — so the one-armed reading (a
//! successor that *is* the post-dominator) never fired. Both successors were walked as ordinary
//! arms, the inner branch claimed the shared block first, and the outer branch's own arrival there
//! was read as a re-entered block: the join was quoted as the outer `else`'s block under
//! `FallbackReason::Loop` (`jre_region_loop`), and `return 0` was written inside the *inner* `else`
//! instead of after the `if`.
//!
//! The texts below pin the fix over the class bytes committed in `tests/fixtures/p3-forward-join/`
//! (see its `README.md` for the command, the version and the digest): the `&&` shape (`both`), the
//! `||` shape, whose outer branch *transfers* to the join (`either`), and the control the change
//! must not touch — a real loop, whose exit is entered by its own header and by nothing else
//! (`again`, which stays a `while`).

use jarde::*;
use std::slice;

const SAMPLE: &[u8] = include_bytes!("fixtures/p3-forward-join/v8/Join.class");

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

/// The statements of one member's text, with every run of whitespace collapsed into one space: the
/// declaration line and the `//` envelope the writer puts above the statements are dropped, because
/// the statements and their nesting are what these texts are about, and the indentation the writer
/// chose is not.
fn statements(text: &str) -> String {
    let body: String = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let opened = body.find('{').map_or(body.len(), |index| index + 1);
    body[opened..]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// One member's own run, checked to be a **presentation**: it wrote statements, and it refused no
/// region. Both halves are what the defect changed — the shape used to arrive with the join quoted
/// under `FallbackReason::Loop` (`jre_region_loop`).
fn reported_run<'a>(owner: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(owner, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => {
            assert_eq!(
                report.content,
                RecoveryContent::ContainsStatements,
                "`{name}` is presented with statements in it:\n{}",
                text_of(owner, name)
            );
            assert!(
                report.fallbacks.is_empty(),
                "no region of `{name}` is refused, got {:?}:\n{}",
                report.fallbacks,
                text_of(owner, name)
            );
            report
        }
        other => panic!("`{name}` was read by its own run, got {other:?}"),
    }
}

#[test]
fn the_shared_successor_of_an_and_is_the_outer_ifs_join() {
    // `both(II)I` is `iload_0; ifle 10; iload_1; ifle 10; iconst_1; ireturn; 10: iconst_0; ireturn`:
    // block 10 is the taken target of both branches and is not the branch's post-dominator (the
    // `return 1` path never reaches it), so the outer `if` is one-armed around the inner one and
    // `return 0` follows it exactly once.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Join");
    reported_run(&report, "both");
    let raw = text_of(&report, "both");
    let text = statements(raw);
    assert_eq!(
        text, "if (arg0 > 0) { if (arg1 > 0) { return 1; } } return 0; }",
        "the `&&` shape is two `if`s with one `return 0` after them:\n{raw}"
    );
    assert_eq!(
        text.matches("return 1;").count(),
        1,
        "the arm that returns is written once:\n{raw}"
    );
    assert_eq!(
        text.matches("return 0;").count(),
        1,
        "the shared block's `return 0` is written once, after the `if`:\n{raw}"
    );
    assert!(
        !raw.contains("&&"),
        "the two branches are written as two `if`s, not as an `&&`:\n{raw}"
    );
    assert!(
        !raw.contains("else"),
        "the outer `if` has no `else` — its taken successor is the join:\n{raw}"
    );
}

#[test]
fn the_shared_successor_of_an_or_is_the_same_join() {
    // `either(II)I` is `iload_0; ifgt 8; iload_1; ifle 10; iconst_1; ireturn; 10: iconst_0;
    // ireturn`: block 8 is the outer branch's **taken** target and the inner branch's
    // **fall-through** successor, so the same join is read from the other side — the inner `if`
    // keeps its (empty) then arm, the `return 0` is written in the `else`, and `return 1` follows
    // the outer `if` once. No condition is inverted to fill the empty arm.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Join");
    reported_run(&report, "either");
    let raw = text_of(&report, "either");
    let text = statements(raw);
    assert_eq!(
        text, "if (arg0 <= 0) { if (arg1 > 0) { } else { return 0; } } return 1; }",
        "the `||` shape is two `if`s with one `return 1` after them:\n{raw}"
    );
    assert_eq!(
        text.matches("return 1;").count(),
        1,
        "the arm that returns is written once, after the outer `if`:\n{raw}"
    );
    assert_eq!(
        text.matches("return 0;").count(),
        1,
        "the shared block's `return 0` is written once:\n{raw}"
    );
    assert!(
        !raw.contains("||"),
        "the two branches are written as two `if`s, not as an `||`:\n{raw}"
    );
}

#[test]
fn a_real_loop_is_still_a_while() {
    // `again(I)I` is `iload_0; ifle 11; iload_0; iconst_1; isub; istore_0; goto 0; 11: iload_0;
    // ireturn`: the loop's exit is entered by its header's own branch and by nothing else — a
    // successor whose only predecessor is the branch node is no forward join — and the header is
    // read as a loop before any branch arm is examined. The control must not change.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Join");
    reported_run(&report, "again");
    let raw = text_of(&report, "again");
    let text = statements(raw);
    assert_eq!(
        text, "while (arg0 > 0) { arg0 = arg0 - 1; } return arg0; }",
        "the loop is a `while`, with its test written inside the statement:\n{raw}"
    );
}
