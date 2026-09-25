//! P3 task 2c.25: a `switch` **expression** is written as one `return` per arm.
//!
//! `return switch (n) { case 1 -> 2; case 2 -> 3; default -> 0; };` compiles to an ordinary
//! `lookupswitch`: each arm pushes one value and jumps to one `ireturn`, and the default arm falls
//! into that same `ireturn` rather than returning on its own. The join block's entry state therefore
//! names a **stack phi** — the value every arm hands it — and `Builder::render_value` refuses every
//! phi like it ("the value at BCI 37 is the entry state of stack depth 0, which no instruction
//! produced"), so each arm was written as an empty `break` and the `ireturn` was quoted. The text
//! that states the shape is one `return` inside each arm — where the value is produced — with the
//! join's own instruction skipped, and `emit.rs` already writes no `break` after a `return`.
//!
//! The rule is all or nothing, and the texts below pin both halves of it: `expr` (three constant
//! arms and a default that falls into the join) and `yielded` (a block arm whose `yield x` is the
//! load of the local it stored, which must keep its `int local1 = arg0 + 1;` and must **not** be
//! folded into `return arg0 + 1`). The control in the other direction is `p3_char_switch`'s
//! `letter`/`number`: their arms end in their own `ireturn`, so no join `return` exists to write and
//! neither text may gain a `return`.
//!
//! The sample is `tests/fixtures/p3-switch-value/` (javac 23.0.1 `--release 21 -g:none`; the
//! command, the 306 bytes and the SHA-256 are in its README).

use jarde::*;
use std::slice;

/// The committed sample: javac 23.0.1, `--release 21 -g:none` (see the fixture's README for the
/// command, the 306 bytes and the SHA-256).
const SAMPLE: &[u8] = include_bytes!("fixtures/p3-switch-value/v21/Joined.class");

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
                java_release: 21,
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

/// One member's own run, checked to be a **presentation**: it wrote statements and refused no
/// region — the join's phi is not rendered, so nothing of either method is quoted bytecode.
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

#[test]
fn a_switch_expressions_join_return_is_written_into_every_arm() {
    // `expr(I)I` is `iload_0; lookupswitch {1 → 28, 2 → 32, default → 36}; 28: iconst_2; goto 37;
    // 32: iconst_3; goto 37; 36: iconst_0; 37: ireturn`: every arm leaves exactly one value and the
    // join's only instruction is the member's own `ireturn`, which the default arm falls into. The
    // `ireturn` is therefore written **inside each arm** — and exactly once, so the method states
    // three `return`s and no `break`, no `@bytecode` and no `return switch`.
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Joined");
    reported_run(&report, "expr");
    let raw = text_of(&report, "expr");
    // The statements are compared whole — the body's own closing brace included, as in
    // `p3_forward_join`'s texts — so a statement the change adds anywhere is a failure, not just a
    // statement it drops.
    assert_eq!(
        statements(raw),
        "switch (arg0) { case 1: return 2; case 2: return 3; default: return 0; } }",
        "each arm ends in the return the join holds:\n{raw}"
    );
    for (arm, value) in [("case 1", "return 2;"), ("case 2", "return 3;")] {
        assert_eq!(
            raw.matches(value).count(),
            1,
            "`{arm}` states `{value}` once:\n{raw}"
        );
    }
    assert_eq!(
        raw.matches("return 0;").count(),
        1,
        "the default arm states the join's `return 0` once:\n{raw}"
    );
    assert_eq!(
        raw.matches("return").count(),
        3,
        "the join's return is written once per arm and nowhere else:\n{raw}"
    );
    assert!(
        !raw.contains("return switch"),
        "a switch expression is written as its arms' returns, not as `return switch`:\n{raw}"
    );
    assert!(
        !raw.contains("break"),
        "no arm ends in a `break`, because the statement before it is a `return`:\n{raw}"
    );
    assert!(
        !raw.contains("@bytecode"),
        "the join is presented, not quoted:\n{raw}"
    );
}

#[test]
fn a_yielding_arm_keeps_the_store_it_already_wrote() {
    // `yielded(I)I` is `iload_0; lookupswitch {1 → 20, default → 28}; 20: iload_0; iconst_1; iadd;
    // istore_1; iload_1; goto 29; 28: iconst_0; 29: ireturn`: the arm's `yield x` leaves the load of
    // `local1`, so its `return` reads the local the arm stored — the statement the arm already wrote
    // is kept, and the value is not folded back into `return arg0 + 1`, which is a program the
    // arm's own instructions do not state (the store would be dropped).
    let sample = open(SAMPLE);
    let report = class_source_of(&sample, "Joined");
    reported_run(&report, "yielded");
    let raw = text_of(&report, "yielded");
    assert_eq!(
        statements(raw),
        "switch (arg0) { case 1: int local1 = arg0 + 1; return local1; default: return 0; } }",
        "the arm's store stays, and the return reads what it stored:\n{raw}"
    );
    assert!(
        raw.contains("int local1 = arg0 + 1;"),
        "the arm's own store is kept:\n{raw}"
    );
    assert!(
        raw.contains("return local1;"),
        "the arm returns the local it stored:\n{raw}"
    );
    assert!(
        !raw.contains("return arg0 + 1"),
        "the store is not folded away into the expression it was computed from:\n{raw}"
    );
    assert_eq!(
        raw.matches("return").count(),
        2,
        "one return per arm, and none at the join:\n{raw}"
    );
    assert!(
        !raw.contains("break") && !raw.contains("@bytecode"),
        "the switch is presented whole:\n{raw}"
    );
}
