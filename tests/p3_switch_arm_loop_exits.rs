//! `recover-switch-arm-loop-exits`: a switch arm whose terminal edge reaches the enclosing
//! loop's latch or header test is presented as an arm-local `continue;` (the existing
//! `Region::LoopContinue` leaf), and is excluded from the switch's join candidates. When no
//! in-loop block is met by two arms, the join is the loop's own continue target and every arm
//! falls out of the switch. A switch that still cannot be built degrades to fallbacks whose
//! block sets are disjoint and whose first diagnosis names the real first failure.
//!
//! The committed inputs are the patrol's frozen `W1.class`/`W2.class` and this slice's
//! `Cf13Exits.class` (javac 23.0.1 `--release 8`), all SHA-256 in the evidence READMEs.
//! Assertions, per behavior:
//!
//! * `W1.mix` (continue + join statement) and `W2.contNoJoin` (join == latch) recover whole;
//!   `mix` presents the arm-local `continue;` and the post-switch `+= 10`; `contNoJoin` presents
//!   no `continue;` — its goto-latch edges are the arms' natural fall-out;
//! * `W2.noCont` — the no-continue control — stays byte-for-byte the baseline recovery
//!   (`results/W2.baseline.java`, captured at the pre-change baseline);
//! * the variant class pins the boundaries: goto-latch with no source continue stays fall-out,
//!   two continue arms, a whole-arm `default: continue;`, an arm break through the loop's own
//!   break channel, a while-form continue to the header test, a labeled continue one loop up
//!   (refusal channel unchanged), and a continue inside a string switch — the whole class
//!   recompiles under `javac --release 8` and every run matches the original under
//!   `java -Xverify:all`;
//! * a switch that still refuses (`SwitchLoopAdjacent.nested`, the boundary class' cross-case,
//!   extra-entry and two-join methods) keeps its quote, and the fallbacks list leads with the
//!   switch's own refusal — the loop shape and the uncovered blocks follow, never
//!   `jre_region_ownership_overlap`.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const W1: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/cf13-switch-continue-patrol/fixture/W1.class"
);
const W2: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/cf13-switch-continue-patrol/fixture/W2.class"
);
const W2_BASELINE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-10-01/cf13-switch-continue-patrol/arm-loop-exits/results/W2.baseline.java"
);
const VARIANTS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/cf13-switch-continue-patrol/arm-loop-exits/fixture/Cf13Exits.class"
);
const ADJACENT: &[u8] = include_bytes!("fixtures/p3-switch-loop-exits/v8/SwitchLoopAdjacent.class");
const BOUNDARIES: &[u8] =
    include_bytes!("fixtures/p3-switch-loop-exits/v8/SwitchLoopLocalBoundaries.class");

fn budget() -> Budget {
    task_budget(&[]).expect("bounded default budget")
}

fn opened(bytes: &[u8], class: &str) -> (Engine, ArtifactSnapshot, ClassSourceRequest) {
    let engine = Engine::new();
    let mut budget = budget();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen Java 8 class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
    (engine, snapshot, request)
}

fn class_text(bytes: &[u8], class: &str) -> ClassSourceReport {
    let (engine, snapshot, request) = opened(bytes, class);
    match engine
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("the frozen class answers the class-source request")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen class produced a complete answer: {other:?}"),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("{name} exists"))
}

fn recovered_method_text(report: &ClassSourceReport, name: &str) -> String {
    let method = method(report, name);
    match &method.outcome {
        ClassSourceOutcome::Recovered { .. } => method.text.clone(),
        other => panic!("{name} must recover whole: {other:?}"),
    }
}

/// The method text between the `noCont` declaration and the next member, extracted from a whole
/// class recovery by the same brace walk javac's parser would do.
fn method_segment(class_text: &str, name: &str) -> String {
    let declaration = class_text
        .find(&format!("static int {name}("))
        .unwrap_or_else(|| panic!("{name} declared in the class text"));
    let body = class_text[declaration..].find('{').expect("body opens");
    let mut depth = 0;
    let mut end = body;
    for (offset, character) in class_text[declaration + body..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = body + offset;
                    break;
                }
            }
            _ => {}
        }
    }
    class_text[declaration + body..=declaration + end].to_owned()
}

static UNIQUE: AtomicU64 = AtomicU64::new(0);

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "jarde-switch-arm-loop-exits-{tag}-{}-{}",
        std::process::id(),
        UNIQUE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&dir).expect("the scratch directory creates");
    dir
}

/// Compiles one recovered class with `javac --release 8` and runs it under `java -Xverify:all`,
/// returning the run's exact stdout.
fn compile_and_run(tag: &str, class: &str, text: &str) -> String {
    let dir = scratch_dir(tag);
    let source = dir.join(format!("{class}.java"));
    fs::write(&source, text).expect("the recovered source writes");
    let compile = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-Xlint:-options")
        .arg("-d")
        .arg(&dir)
        .arg(&source)
        .output()
        .expect("the installed JDK provides javac");
    assert!(
        compile.status.success(),
        "{class} must compile as presented: {}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(&dir)
        .arg(class)
        .output()
        .expect("the installed JDK provides java");
    assert!(
        run.status.success(),
        "{class} must run: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run prints UTF-8")
}

#[test]
fn switch_arm_continue_and_the_switch_join_present_together() {
    let report = class_text(W1, "W1");
    assert!(
        !report.text.contains("@bytecode"),
        "W1 recovers whole:\n{}",
        report.text
    );
    let mix = recovered_method_text(&report, "mix");
    assert!(mix.contains("switch (local2 % 3)"), "{mix}");
    assert!(
        mix.contains("if (local2 > 4) {\n                        continue;\n                    } else {\n                        local1 = local1 + 3;\n                    }"),
        "the default arm ends in its own continue:\n{mix}"
    );
    assert_eq!(
        mix.matches("local1 = local1 + 10;").count(),
        1,
        "the switch join runs once per iteration:\n{mix}"
    );
    let body = match &method(&report, "mix").outcome {
        ClassSourceOutcome::Recovered { report: body, .. } => body,
        other => panic!("mix recovered: {other:?}"),
    };
    for (bci, expected) in [
        (12, "switch (local2 % 3)"),
        (57, "continue;"),
        (60, "local1 = local1 + 3;"),
        (63, "local1 = local1 + 10;"),
        (66, "local2 = local2 + 1"),
    ] {
        assert!(
            body.source_map
                .text_of_bci(&body.text, bci)
                .iter()
                .any(|piece| piece.contains(expected)),
            "BCI {bci} must own {expected:?}: {:?}",
            body.source_map.of_bci(bci)
        );
    }
    assert_eq!(
        compile_and_run("w1", "W1", &report.text),
        "70\n36\n",
        "the original class prints 70/36"
    );
}

#[test]
fn join_equal_to_the_latch_recovers_and_the_control_stays_verbatim() {
    let report = class_text(W2, "W2");
    assert!(
        !report.text.contains("@bytecode"),
        "W2 recovers whole:\n{}",
        report.text
    );
    let no_join = recovered_method_text(&report, "contNoJoin");
    assert!(
        !no_join.contains("continue;"),
        "every goto latch is a natural fall-out here, not a continue:\n{no_join}"
    );
    assert!(
        no_join.contains("if (local2 > 4) {") && no_join.contains("local1 = local1 + 3;"),
        "the default arm keeps its conditional shape:\n{no_join}"
    );
    // The no-continue control must be byte-for-byte the pre-change recovery.
    let before = method_segment(W2_BASELINE, "noCont");
    let after = method_segment(&report.text, "noCont");
    assert_eq!(before, after, "noCont must not change by one character");
    assert_eq!(
        compile_and_run("w2", "W2", &report.text),
        "72\n9\n",
        "the original class prints 72/9"
    );
}

#[test]
fn the_variant_family_recovers_and_runs_like_the_original() {
    let report = class_text(VARIANTS, "Cf13Exits");
    assert!(
        !report.text.contains("@bytecode"),
        "the variant class recovers whole:\n{}",
        report.text
    );
    // goto latch with no source continue: the strict negative stays a natural fall-out.
    let no_continue = recovered_method_text(&report, "noJoinNoContinue");
    assert!(
        !no_continue.contains("continue;"),
        "no goto latch may be read as a continue the source never had:\n{no_continue}"
    );
    // Two arms each end in `if (…) continue;` and still complete at the shared join.
    let two_arms = recovered_method_text(&report, "twoContinueArms");
    assert_eq!(
        two_arms.matches("continue;").count(),
        2,
        "both continue arms present:\n{two_arms}"
    );
    assert!(
        two_arms.contains("local1 = local1 + 10;"),
        "the switch join survives both continues:\n{two_arms}"
    );
    // The whole default arm is one continue statement.
    let whole = recovered_method_text(&report, "defaultWholeContinue");
    assert_eq!(
        whole.matches("continue;").count(),
        1,
        "an arm that is only a continue presents exactly that:\n{whole}"
    );
    assert!(
        whole.contains("default:") && whole.contains("local1 = local1 + 10;"),
        "the default continue keeps the switch join:\n{whole}"
    );
    // An arm leaving through the loop's break channel keeps the existing presentation.
    let breaks = recovered_method_text(&report, "armBreaksLoop");
    assert!(
        breaks.contains("break loop;") && !breaks.contains("continue;"),
        "the loop break channel is unchanged:\n{breaks}"
    );
    // A while-form continue targets the header test.
    let while_form = recovered_method_text(&report, "whileFormContinue");
    assert!(
        while_form.contains("continue;"),
        "the while-form continue presents:\n{while_form}"
    );
    // A labeled continue one loop up keeps its pre-change presentation (behaviorally the
    // loop's break channel here); it must not become a bare continue.
    let outer = recovered_method_text(&report, "outerLabeledContinue");
    assert!(
        outer.contains("break loop;") && !outer.contains("continue;"),
        "the non-direct loop exit is unchanged:\n{outer}"
    );
    // A continue inside a string switch presents like the integer switch now does.
    let string_switch = recovered_method_text(&report, "stringSwitchContinue");
    assert!(
        string_switch.contains("switch (local5)") && string_switch.contains("continue;"),
        "the string switch arm continue presents:\n{string_switch}"
    );
    for name in [
        "noJoinNoContinue",
        "twoContinueArms",
        "defaultWholeContinue",
        "armBreaksLoop",
        "whileFormContinue",
        "outerLabeledContinue",
        "stringSwitchContinue",
    ] {
        let member = method(&report, name);
        match &member.outcome {
            ClassSourceOutcome::Recovered { report: body, .. } => assert!(
                body.fallbacks.is_empty(),
                "{name} carries no fallback: {:?}",
                body.fallbacks
            ),
            other => panic!("{name} recovers: {other:?}"),
        }
    }
    assert_eq!(
        compile_and_run("variants", "Cf13Exits", &report.text),
        "13\n87\n57\n72\n83\n96\n47\n",
        "the original variant class prints 13/87/57/72/83/96/47"
    );
}

#[test]
fn a_still_refused_switch_names_its_own_failure_first() {
    // The nested negative keeps its quote, and the fallbacks lead with the switch's own
    // refusal — the loop shape and the uncovered blocks follow. The pre-change tree reported
    // `jre_region_ownership_overlap` because the refusal quotes claimed block 9 twice.
    let report = class_text(ADJACENT, "SwitchLoopAdjacent");
    let nested = method(&report, "nested");
    assert!(
        nested.text.contains("@bytecode") && !nested.text.contains("switch ("),
        "the nested switch still refuses:\n{}",
        nested.text
    );
    match &nested.outcome {
        ClassSourceOutcome::Recovered { report: body, .. } => {
            assert_eq!(
                body.fallbacks,
                vec![
                    "jre_region_switch_shape",
                    "jre_region_loop_shape",
                    "jre_region_uncovered_blocks",
                ],
                "the degraded walk names the switch refusal first"
            );
        }
        other => panic!("the method outcome is a recovery report: {other:?}"),
    }
    // The boundary class keeps its three refusals — none of them misreported as an ownership
    // overlap, and each naming a structural refusal of its own walk first — and its ordinary
    // tail still recovers without a continue.
    let boundaries = class_text(BOUNDARIES, "SwitchLoopLocalBoundaries");
    for name in ["crossCase", "extraEntry", "multipleJoins"] {
        let member = method(&boundaries, name);
        assert!(
            member.text.contains("@bytecode"),
            "{name} keeps its quote:\n{}",
            member.text
        );
        match &member.outcome {
            ClassSourceOutcome::Recovered { report: body, .. } => {
                assert!(
                    !body.fallbacks.contains(&"jre_region_ownership_overlap"),
                    "{name} must not report an ownership overlap: {:?}",
                    body.fallbacks
                );
                assert!(
                    !body.fallbacks.is_empty(),
                    "{name} names the refusal its walk hit first"
                );
            }
            other => panic!("{name} outcome is a recovery report: {other:?}"),
        }
    }
    let ordinary = recovered_method_text(&boundaries, "ordinaryTail");
    assert!(
        ordinary.contains("switch (") && !ordinary.contains("continue;"),
        "a normal loop tail is still not a continue:\n{ordinary}"
    );
}
