//! `recover-crossing-array-read-values`: the write-value whitelist a local that crosses a
//! protected region must satisfy accepts a **same-block array element read** whose array operand
//! is a local load and whose index is one of the accepted trees, beside the literals, loads,
//! static calls and additions it already named. The read is evaluated in place, so restoring the
//! store's statement re-runs exactly the subscript the bytecode ran — the same inline discipline
//! the other members state (one basic block, one use, no other instruction between the first
//! producer and the store).
//!
//! The committed inputs are the patrol's frozen `F1.class`/`F2.class` and this slice's
//! `Cf10Values.class`/`Cf10Refused.class`/`Cf10Nested.class` (javac 23.0.1 `--release 8`), all
//! SHA-256 in the evidence results.
//!
//! Assertions, per behavior:
//!
//! * `F2.stepTwoWithQuote` — the patrol's fixed refusal — recovers whole and presents the
//!   accumulator exactly as the unprotected control does, and recompiles and runs like the
//!   original under `java -Xverify:all`;
//! * `F1` — the unprotected control — stays byte-for-byte the patrol's recorded recovery;
//! * the variant family recovers whole and runs like the original: an `int` accumulator with the
//!   call in its tree, a `double[]` accumulator, a reference-array accumulator, two writes inside
//!   one catch, and one crossing local carried across two protected regions;
//! * the refused shapes keep the pre-change refusal byte-for-byte: the element value shared with
//!   a second store (`pick = sum = data[i]`), a write tree whose leaf is a join of two blocks
//!   (the ternary's phi), and an embedded assignment between the tree's producers
//!   (`sum + (k = 2) + data[i]`);
//! * a budget run out of analysis steps states no recovery for the variant class — the same
//!   no-partial-text atomicity the recovery entry point states everywhere else;
//! * the genuinely nested try shapes stay refused exactly as before the change — the refusal is
//!   the region layer's own (`not reducible` / `quoted fallback region`), recorded here as this
//!   slice's boundary, not widened by the whitelist.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const F1: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/fixture/F1.class"
);
/// The patrol's recorded `F1` presentation: the byte-for-byte control this slice must not move.
const F1_VERBATIM: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/results/F1.jarde.java"
);
const F2: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/fixture/F2.class"
);
const VALUES: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/crossing-array-read-values/fixture/Cf10Values.class"
);
const REFUSED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/crossing-array-read-values/fixture/Cf10Refused.class"
);
/// The pre-change recovery of the refused class, captured at the baseline commit.
const REFUSED_BASELINE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/crossing-array-read-values/results/Cf10Refused.before.java"
);
const NESTED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/crossing-array-read-values/fixture/Cf10Nested.class"
);
/// The pre-change recovery of the nested-try boundary class, captured at the baseline commit.
const NESTED_BASELINE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-10-01/cf10-crossing-array-read-patrol/crossing-array-read-values/results/Cf10Nested.before.java"
);

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

/// The method text between the declaration and the next member, extracted from a whole class
/// recovery by the same brace walk javac's parser would do.
fn method_segment(class_text: &str, signature: &str) -> String {
    let declaration = class_text
        .find(signature)
        .unwrap_or_else(|| panic!("{signature} declared in the class text"));
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
        "jarde-crossing-array-read-values-{tag}-{}-{}",
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
fn f2_recovers_and_the_unprotected_control_stays_verbatim() {
    // The control first: the unprotected `i += 2` recovery is one byte of the patrol's record.
    let control = class_text(F1, "F1");
    assert_eq!(
        control.text, F1_VERBATIM,
        "the unprotected control must not move by one character"
    );
    // The patrol's fixed refusal now recovers whole, in the control's own presentation.
    let report = class_text(F2, "F2");
    assert!(
        !report.text.contains("@bytecode"),
        "F2 recovers whole:\n{}",
        report.text
    );
    let sum = recovered_method_text(&report, "stepTwoWithQuote");
    assert!(
        sum.contains("while (local2 < arg0.length)")
            && sum.contains("local1 = local1 + arg0[local2];"),
        "the loop accumulates the element read:\n{sum}"
    );
    assert!(
        sum.contains("try {\n                local1 = local1 + risky(arg0[local2]);")
            && sum.contains("local1 = local1 - 1;"),
        "the protected arm keeps its call and the catch its decrement:\n{sum}"
    );
    let body = match &method(&report, "stepTwoWithQuote").outcome {
        ClassSourceOutcome::Recovered { report: body, .. } => body,
        other => panic!("stepTwoWithQuote recovered: {other:?}"),
    };
    assert!(
        body.fallbacks.is_empty(),
        "the recovery carries no fallback: {:?}",
        body.fallbacks
    );
    // The original class prints 14 (the loop enters the catch on `data[2] == 3`); the recovered
    // presentation recompiles and prints the same.
    assert_eq!(
        compile_and_run("f2", "F2", &report.text),
        "14\n",
        "the original class prints 14"
    );
}

#[test]
fn the_variant_family_recovers_and_runs_like_the_original() {
    let report = class_text(VALUES, "Cf10Values");
    assert!(
        !report.text.contains("@bytecode"),
        "the variant class recovers whole:\n{}",
        report.text
    );
    // The `int` accumulator is the F2 shape inside the class: call in the tree, `iinc` catch.
    let int_sum = recovered_method_text(&report, "intSum");
    assert!(
        int_sum.contains("sum = sum + data[i];") && int_sum.contains("sum = sum - 1;"),
        "the int accumulator keeps the accepted tree:\n{int_sum}"
    );
    // The `double[]` accumulator: the numeric store family presents like the int one.
    let double_sum = recovered_method_text(&report, "doubleSum");
    assert!(
        double_sum.contains("double sum;")
            && double_sum.contains("sum = sum + data[i];")
            && double_sum.contains(
                "catch (java.lang.IllegalStateException e) {\n                sum = data[1];"
            ),
        "the double accumulator presents its element reads:\n{double_sum}"
    );
    // The reference-array accumulator: `aaload` through the array's own element type.
    let last_ref = recovered_method_text(&report, "lastRef");
    assert!(
        last_ref.contains("java.lang.String cur;")
            && last_ref.contains("cur = parts[i];")
            && last_ref.contains("cur = parts[0];"),
        "the reference accumulator presents its element reads:\n{last_ref}"
    );
    // Two writes inside one catch: both stores present, in order.
    let catch_writes = recovered_method_text(&report, "catchMultiWrite");
    assert!(
        catch_writes.contains("sum = data[1];") && catch_writes.contains("sum = sum + data[2];"),
        "the catch's two writes present in order:\n{catch_writes}"
    );
    // One local carried across two protected regions in one loop body.
    let two_tries = recovered_method_text(&report, "twoTries");
    assert!(
        two_tries.matches("try {").count() == 2
            && two_tries.contains("noise2(sum);")
            && two_tries.contains("sum = sum - 1;")
            && two_tries.contains("sum = data[1];"),
        "both protected regions present around the one local:\n{two_tries}"
    );
    for name in [
        "intSum",
        "doubleSum",
        "lastRef",
        "catchMultiWrite",
        "twoTries",
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
    // The original prints one line per method, with every catch entered by its own input.
    assert_eq!(
        compile_and_run("values", "Cf10Values", &report.text),
        "14\n8.0\nf\n15\n12\n",
        "the original variant class prints 14/8.0/f/15/12"
    );
}

#[test]
fn the_refused_shapes_keep_their_refusals() {
    let report = class_text(REFUSED, "Cf10Refused");
    // The element value shared with a second store, the cross-block join leaf, and the embedded
    // assignment between producers: three refusals, unchanged from the baseline capture.
    for name in ["shared", "crossBlock", "intervalEffect"] {
        let member = method(&report, name);
        assert!(
            member.text.contains("@bytecode"),
            "{name} keeps its quote:\n{}",
            member.text
        );
        let before = method_segment(REFUSED_BASELINE, &format!("static int {name}("));
        let after = method_segment(&report.text, &format!("static int {name}("));
        assert_eq!(before, after, "{name} must not change by one character");
    }
    assert_eq!(
        report.text.matches("crosses a ").count(),
        3,
        "each refusal names its crossing-local walk or quoted region:\n{}",
        report.text
    );
}

#[test]
fn the_nested_try_boundary_stays_where_the_region_layer_left_it() {
    let report = class_text(NESTED, "Cf10Nested");
    for (signature, refusal) in [
        (
            "static int inLoop(",
            "the graph is not reducible over 4 block(s)",
        ),
        ("static int aroundLoop(", "crosses a quoted fallback region"),
    ] {
        let member_text = method_segment(&report.text, signature);
        assert!(
            member_text.contains("@bytecode") && member_text.contains(refusal),
            "the nested shape keeps the region layer's own refusal ({refusal}):\n{member_text}"
        );
        let before = method_segment(NESTED_BASELINE, signature);
        assert_eq!(
            before, member_text,
            "the nested boundary must not change by one character"
        );
    }
}

#[test]
fn a_budget_starved_variant_run_states_no_recovery() {
    let engine = Engine::new();
    let mut budget = task_budget(&[BudgetOverride::AnalysisSteps { limit: 64 }])
        .expect("a bounded budget builds");
    let snapshot = engine
        .open(ArtifactInput::bytes(VALUES.to_vec()), &mut budget)
        .expect("the frozen class opens under a starved budget too");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("Cf10Values"),
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
    let mut starved = task_budget(&[BudgetOverride::AnalysisSteps { limit: 64 }])
        .expect("a bounded budget builds");
    // Whatever answer the starved run gives — an error, an incomplete search or a performed run —
    // no crossing method may present the body its proof would have needed. (`<init>` is two
    // instructions and may legitimately finish inside the same budget.)
    let answer = engine.class_source(slice::from_ref(&snapshot), &request, &mut starved);
    let performed = match answer {
        Ok(OperationOutcome::Performed(report)) => Some(report),
        Ok(_) => None,
        Err(_) => None,
    };
    if let Some(report) = performed {
        for method in &report.methods {
            let name = String::from_utf8_lossy(&method.item.name.raw().0).into_owned();
            if !matches!(
                name.as_str(),
                "intSum" | "doubleSum" | "lastRef" | "catchMultiWrite" | "twoTries"
            ) {
                continue;
            }
            if let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome {
                assert!(
                    !matches!(body.execution, jarde::ExecutionReport::Complete { .. }),
                    "{name} claimed a complete presentation the budget could not have proven:\n{}",
                    method.text
                );
            }
        }
    }
}
