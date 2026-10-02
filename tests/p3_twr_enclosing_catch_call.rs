//! `recover-enclosing-catch-call-bodies` (CF-17c): the whole-construct `catch` clause of a
//! `try`-with-resources statement admits the one discarded call in its body — a non-`void`
//! invocation whose result the very next instruction of the same block, a `pop`, drops whole —
//! the same 17a criterion the guarded body's own statement subset carries, read at the clause's
//! body-statement enumeration (`guard.rs::enclosing_clause`).
//!
//! The committed inputs are the patrol's frozen `P2.class` and `P3.class` (the anchors the
//! patrol's P2/P3 判别 pinned: the return-form catch body recovered, the discarded-call form
//! refused — one variable), the `-17c` family's `W17c.class` (javac 23.0.1 `--release 8 -g:none`)
//! and `N17c.class` (JDK 23 Class-File API, generator beside the fixtures), all SHA-256 in the
//! evidence README. Assertions, per behavior:
//!
//! * the **text**: the patrol anchors `P3.voidBodyCatch` and `P2.plainCatch` present
//!   `try (…) { … } catch (…) { log.append("E"); }` whole, and the variant family presents every
//!   discarded-call clause shape — the fall-through clause, the injected body throw, the
//!   multi-statement discard-plus-`return` body, two discards, the chained consumption whose own
//!   last call is the discard, the close-thrown path and the suppression path — while the
//!   return-form control (`W17c.normalReturn`) keeps the 17b family's exact text;
//! * the **refusal**: every verifier-valid negative shape keeps the refusal mainline states — the
//!   three hand-lowered discards outside the criterion (`invoke; dup; pop; pop`,
//!   `invoke; dup; astore; pop`, `invoke; checkcast; pop`) and the javac-branching clause body —
//!   all at the `jre_guard_unexplained_row` the whole-construct row held, with no clause written;
//! * the **compile**: the recovered `P2`, `P3` and `W17c` classes recompile under
//!   `javac --release 8` as presented;
//! * the **runtime**: the original frozen bytes and the recovered classes print the same lines
//!   under `java -Xverify:all` — the patrol anchors' normal paths (`t[c]`, the accumulated
//!   `B[c]`/`B[c]b[c]`), the injected `IllegalStateException` path the variant family's own
//!   bodies throw into the clause, the close-thrown path and the suppression path included;
//! * the **budget**: a stopped or cancelled run over a recovered body publishes nothing.

use jarde::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

macro_rules! patrol {
    ($rest:literal) => {
        concat!(
            "../openspec/evidence/java-syntax-2026-10-02/twr-clause-patrol/",
            $rest
        )
    };
}

const P2: &[u8] = include_bytes!(patrol!("fixture/P2.class"));
const P3: &[u8] = include_bytes!(patrol!("fixture/P3.class"));
const W17C: &[u8] = include_bytes!(patrol!("catchcall-17c/original/W17c.class"));
const N17C: &[u8] = include_bytes!(patrol!("catchcall-17c/original/N17c.class"));

/// The runs of the original frozen classes — the text every compileable leg printed. `P3.main`
/// is the patrol's own `t[c]` accumulation (`P3` fully recovers, so the recovered leg runs it
/// whole); `P2.main` accumulates `B[c]` then `B[c]b[c]`; `W17c.main` walks the normal return,
/// the fall-through clause, the body-thrown, the multi-statement discard-plus-return, the two
/// discards, the chained consumption, then — with `closeBoom` set — the close-thrown and
/// suppressed paths, and last the branching clause body and the log.
const P3_RUN: &str = "t[c]\nt[c]t[c]\nt[c]t[c]t[c]f\n";
const W17C_RUN: &str = "\
done
done
done
caught
done
done
done
done
done
log:EEabbodyC[[java.lang.IllegalStateException: close]]B2
";

/// The one-method driver that runs the `P2.plainCatch` anchor itself: `P2.branchNoCatch` (a
/// branch in the TWR **body**) is a refusal mainline states and not this slice's, so `P2`'s
/// compile leg stubs it and drives the anchor directly.
const DRIVER2: &str = "public class Driver2 {\n    public static void main(String[] args) throws Exception {\n        System.out.println(P2.plainCatch());\n    }\n}\n";

/// The run of the original frozen `N17c.main` — the negative shapes' bodies never run (the `try`
/// bodies cannot raise), so the class runs clean; it is the *recovery* that refuses.
const N17C_RUN: &str = "done\ndone\ndone\n";

/// One unrecovered member's text with a throwing stub in its body, the direct-finally slice's
/// compile-leg convention: the recovered class then states the members this run proved and no
/// behaviour for the ones it refused, so the run truncates where the first stub stands.
fn stubbed(text: &str, method: &str) -> String {
    let start = text
        .find(&format!("{method}()"))
        .unwrap_or_else(|| panic!("{method} is presented"));
    let end = text[start..].find("\n    }").expect("the member ends") + start;
    let mut stubbed = String::new();
    stubbed.push_str(&text[..end]);
    stubbed.push_str("\n        throw new java.lang.UnsupportedOperationException();");
    stubbed.push_str(&text[end..]);
    stubbed
}

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn class_text(bytes: &[u8], class: &str) -> String {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen class opens");
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
    match Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("the frozen class answers the class-source request")
    {
        OperationOutcome::Performed(report) => report.text,
        other => panic!("one frozen class produced a complete answer: {other:?}"),
    }
}

fn member_text(class: &str, bytes: &[u8], method: &str) -> String {
    let text = class_text(bytes, class);
    let start = text
        .find(&format!("{method}()"))
        .unwrap_or_else(|| panic!("{class}.{method} is presented"));
    text[start..]
        .split_once("\n    }")
        .map(|(body, _)| body.to_owned())
        .expect("the member's own text ends")
}

/// One member's own recovery report, as the class-source run performed it.
fn member_report(class: &str, bytes: &[u8], method: &str) -> RecoveryReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen class opens");
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
    let report = match Engine::new()
        .class_source(slice::from_ref(&snapshot), &request, &mut budget())
        .expect("the frozen class answers the class-source request")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one frozen class produced a complete answer: {other:?}"),
    };
    let member = report
        .methods
        .iter()
        .find(|member| member.item.name.raw().0 == method.as_bytes())
        .unwrap_or_else(|| panic!("the fixture has {method}()"));
    match &member.outcome {
        ClassSourceOutcome::Recovered { report, .. } => (**report).clone(),
        other => panic!("{method}() has a recovery report: {other:?}"),
    }
}

fn write_source(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).expect("the scratch source writes");
    path
}

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "jarde-twrcatchcall-{tag}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock runs")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("the scratch directory creates");
    dir
}

fn javac(files: &[PathBuf]) {
    let dir = files[0].parent().expect("the scratch directory exists");
    let compile = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-cp")
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .args(files)
        .output()
        .expect("the installed JDK provides javac");
    assert!(
        compile.status.success(),
        "javac accepted the recovered source:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}

fn java(dir: &Path, class: &str) -> String {
    let run = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(class)
        .output()
        .expect("the installed JDK provides java");
    assert!(
        run.status.success(),
        "`java -Xverify:all {class}` ran clean:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run's output is text")
}

#[test]
fn the_patrol_anchors_present_the_discarded_call_clause() {
    // The patrol's P2/P3 判别 pinned this slice's one variable: the return-form catch body
    // recovered, the discarded-call form refused. The criterion the clause's body enumeration
    // now carries flips exactly the refused side.
    let expectations = [
        (
            "P3",
            P3,
            "public static java.lang.String voidBodyCatch",
            "try (P3 local0 = new P3()) {\n            touch(local0);\n        } catch (java.lang.IllegalStateException local0) {\n            P3.log.append(\"E\");\n        }\n        return P3.log.toString();",
        ),
        (
            "P2",
            P2,
            "public static java.lang.String plainCatch",
            "try (P2 local0 = new P2()) {\n            P2.log.append(\"b\");\n        } catch (java.lang.IllegalStateException local0) {\n            P2.log.append(\"E\");\n        }\n        return P2.log.toString();",
        ),
    ];
    for (class, bytes, method, expected) in expectations {
        let text = member_text(class, bytes, method);
        assert!(
            text.contains(expected),
            "{class}.{method} presents the clause with its discarded call:\n{text}"
        );
    }
}

#[test]
fn the_discarded_call_clause_anchor_records_the_handler_entry() {
    // The clause's handler entry stays an anchor of the statement's own text: what the text
    // states is what the table named (P3.voidBodyCatch's handler entry is BCI 38).
    let report = member_report("P3", P3, "voidBodyCatch");
    assert_eq!(report.quality, Quality::Structured, "{report:?}");
    assert!(report.fallbacks.is_empty(), "{:?}", report.fallbacks);
    let anchors: std::collections::BTreeSet<u32> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis())
        .collect();
    assert!(
        anchors.contains(&38),
        "the handler entry at BCI 38 is recorded beside the text: {anchors:?}"
    );
}

#[test]
fn the_variant_family_presents_every_discarded_call_clause_shape() {
    let expectations = [
        // The fall-through clause beside a body that cannot raise: the patrol's P2/P3 shape.
        (
            "callCatch",
            "try (W17c local0 = new W17c()) {\n            touch(local0);\n        } catch (java.lang.IllegalStateException local0) {\n            W17c.log.append(\"E\");\n        }",
        ),
        // The injected body throw really runs the clause on the original class (the E path).
        (
            "bodyThrows",
            "try (W17c local0 = new W17c()) {\n            boom();\n        } catch (java.lang.IllegalStateException local0) {\n            W17c.log.append(\"E\");\n        }",
        ),
        // The multi-statement clause body: the discarded call beside a value `return`.
        (
            "callThenReturn",
            "catch (java.lang.IllegalStateException local0) {\n            W17c.log.append(\"E\");\n            return \"caught\";\n        }",
        ),
        // Two discarded calls in one clause body, in the order the bytecode runs them.
        (
            "twoCalls",
            "catch (java.lang.IllegalStateException local0) {\n            W17c.log.append(\"a\");\n            W17c.log.append(\"b\");\n        }",
        ),
        // The chained consumption whose own last call is the one discard.
        (
            "chainedConsume",
            "catch (java.lang.IllegalStateException local0) {\n            W17c.log.append((java.lang.String) local0.getMessage());\n        }",
        ),
        // The close's exception lands in the named clause directly (no suppression copy).
        (
            "closeThrows",
            "catch (java.lang.IllegalStateException local0) {\n            W17c.log.append(\"C\");\n        }",
        ),
        // The suppression chain preserved, the clause body a concat over the suppressed array.
        (
            "suppressedBoth",
            "catch (java.lang.IllegalStateException local0) {\n            W17c.log.append(\"[\" + java.util.Arrays.toString((java.lang.Object[]) local0.getSuppressed()) + \"]\");\n        }",
        ),
    ];
    for (method, expected) in expectations {
        let text = member_text(
            "W17c",
            W17C,
            &format!("public static java.lang.String {method}"),
        );
        assert!(
            text.contains(expected),
            "{method} presents the recovered clause:\n{text}"
        );
    }
    // The return-form control keeps the 17b family's exact text: the criterion widened the
    // whitelist by one statement, and the shapes that never held a discard do not move.
    let control = member_text("W17c", W17C, "public static java.lang.String normalReturn");
    assert!(
        control.contains(
            "try (W17c local0 = new W17c()) {\n            touch(local0);\n        } catch (java.lang.IllegalStateException local0) {\n            return \"caught\";\n        }"
        ),
        "the return-form control keeps its exact text:\n{control}"
    );
}

#[test]
fn verifier_valid_discards_outside_the_criterion_keep_the_clause_refusal() {
    // Every negative kept the refusal mainline states, at the `jre_guard_unexplained_row` the
    // whole-construct row held when no clause reading took it: a `pop` fed by a `dup`, a stored
    // copy read beside the discard (the consumed form), a `checkcast` between the call and the
    // `pop`, and — javac's own — a clause body that branches.
    let negatives = [
        (
            "N17c",
            N17C,
            "popWrongValue",
            "the `pop` reads the copy, not the call's result",
        ),
        (
            "N17c",
            N17C,
            "popSecondReader",
            "the stored copy reads the value beside the `pop`",
        ),
        (
            "N17c",
            N17C,
            "popAfterCast",
            "a `checkcast` stands between the call and the `pop`",
        ),
        ("W17c", W17C, "branchBody", "the clause body branches"),
    ];
    for (class, bytes, method, why) in negatives {
        let report = member_report(class, bytes, method);
        assert_eq!(
            report.quality,
            Quality::Fallback,
            "{method} ({why}) keeps its refusal, not a recovery"
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|item| item.code == "jre_guard_unexplained_row"),
            "{method} ({why}) keeps the unexplained-row refusal: {:?}",
            report.diagnostics
        );
        assert!(
            !report.text.contains("} catch ("),
            "{method} presents no catch clause:\n{}",
            report.text
        );
    }
}

#[test]
fn the_negative_shapes_themselves_run() {
    // The negative class is verifier-valid bytecode that runs — it is the *recovery* that
    // refuses.
    let dir = scratch_dir("negative");
    std::fs::write(dir.join("N17c.class"), N17C).expect("the frozen class writes");
    assert_eq!(
        java(&dir, "N17c"),
        N17C_RUN,
        "the negative shapes themselves run"
    );
}

#[test]
fn the_recovered_classes_compile_and_run_the_original_paths() {
    // `P3` recovers whole, so its recovered class runs its own `main` against the original's
    // lines. `P2`'s and `W17c`'s unrecovered members are not this slice's (`P2.branchNoCatch`
    // bodies its TWR in a branch — a refusal mainline states; `W17c.branchBody` is this slice's
    // own negative), so those members stand as throwing stubs and the runs truncate where the
    // stub stands: every recovered member's path prints the original's lines before it. The
    // `P2.plainCatch` path itself is driven directly — the patrol anchor's own run.
    let jarde_dir = scratch_dir("compile");
    write_source(
        &jarde_dir,
        "P2.java",
        &stubbed(
            &class_text(P2, "P2"),
            "public static java.lang.String branchNoCatch",
        ),
    );
    write_source(&jarde_dir, "P3.java", &class_text(P3, "P3"));
    write_source(
        &jarde_dir,
        "W17c.java",
        &stubbed(
            &class_text(W17C, "W17c"),
            "public static java.lang.String branchBody",
        ),
    );
    write_source(&jarde_dir, "Driver2.java", DRIVER2);
    javac(&[
        jarde_dir.join("P2.java"),
        jarde_dir.join("P3.java"),
        jarde_dir.join("W17c.java"),
        jarde_dir.join("Driver2.java"),
    ]);

    let original_dir = scratch_dir("original");
    for (name, bytes) in [("P2.class", P2), ("P3.class", P3), ("W17c.class", W17C)] {
        std::fs::write(original_dir.join(name), bytes).expect("the frozen class writes");
    }
    write_source(&original_dir, "Driver2.java", DRIVER2);
    javac(&[original_dir.join("Driver2.java")]);

    // The patrol anchors: `P3.voidBodyCatch` runs its own class whole; `P2.plainCatch` runs
    // under one driver on both legs and prints the same line.
    assert_eq!(java(&original_dir, "P3"), P3_RUN, "original P3");
    assert_eq!(java(&jarde_dir, "P3"), P3_RUN, "recovered P3");
    assert_eq!(
        java(&original_dir, "Driver2"),
        "b[c]\n",
        "original P2.plainCatch"
    );
    assert_eq!(
        java(&jarde_dir, "Driver2"),
        "b[c]\n",
        "recovered P2.plainCatch"
    );

    // The variant family's own paths on the original class, and the same lines from the
    // recovered class up to the branch-body stub.
    assert_eq!(java(&original_dir, "W17c"), W17C_RUN, "original W17c");
    let truncated = W17C_RUN
        .split_once("done\nlog:")
        .expect("the run separates the branch-body line from the log line");
    let stubbed_run = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(&jarde_dir)
        .arg("W17c")
        .output()
        .expect("the installed JDK provides java");
    assert_eq!(
        String::from_utf8(stubbed_run.stdout).expect("the run's output is text"),
        truncated.0,
        "every recovered member prints the original's lines before the stub"
    );
}

#[test]
fn a_stopped_or_cancelled_run_publishes_no_partial_clause() {
    // A cap that bites before the first member's own document fits refuses the whole answer: a
    // stopped run publishes nothing partial — no clause, no half of one.
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(W17C.to_vec()), &mut budget())
        .expect("the frozen class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("W17c"),
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
    let mut bounded =
        task_budget(&[BudgetOverride::new("output_bytes", 1).expect("a legal output cap")])
            .expect("the task budget accepts the cap");
    match Engine::new().class_source(slice::from_ref(&snapshot), &request, &mut bounded) {
        Ok(OperationOutcome::Performed(report)) => {
            panic!(
                "a one-byte cap cannot publish a presentation: {}",
                report.text
            )
        }
        // The ordinary stopped answer: the operation's own document never fit, so nothing below
        // it was published either.
        Ok(_) => {}
        other => panic!("the bounded run answers: {other:?}"),
    }

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    match Engine::new().class_source(slice::from_ref(&snapshot), &request, &mut cancelled) {
        // A run that completed anyway still publishes no recovered clause.
        Ok(OperationOutcome::Performed(report)) => assert!(
            !report.text.contains("} catch ("),
            "a cancelled run publishes no recovered clause: {}",
            report.text
        ),
        // The ordinary cancelled answer: the search never executed, so nothing was published.
        Ok(_) => {}
        other => panic!("the cancelled run answers: {other:?}"),
    }
}
