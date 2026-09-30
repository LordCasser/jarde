//! `recover-labeled-loop-tail-coverage`: a labeled loop's trailing statement segment admits one
//! discarded call — a non-`void` invocation whose result the last instruction of its own block, a
//! `pop`, drops whole — because the proved `for` header owns the loop update that follows, the
//! tail's block ends at that `pop`, and the presentation's discard plan (P3 2c.31) must read it
//! there. The slice also spells every loop label as a source-style name (`loop`, then `loop2`,
//! …) instead of the synthesized `jarde_loop_{bci}`.
//!
//! The committed inputs are the patrol's frozen `L2.class` and this slice's `T1/T2/T3.class`
//! (javac 23.0.1 `--release 8 -g:none`), all SHA-256 in the evidence READMEs. Assertions, per
//! behavior:
//!
//! * the **text**: `L2.contWithTail` presents the whole tail — the `for` header, the labeled
//!   `continue`, and the trailing chain statement — with no `@bytecode` quote; the break-label,
//!   double-label and mixed-tail variants keep theirs; the unlabeled control (`L3`) keeps its
//!   exact text, which the rename must not touch;
//! * the **label**: one labeled loop spells `loop`; two nested labeled loops spell `loop` and
//!   `loop2` in first-claim write order (the inner loop's `continue` is written first);
//! * the **compile**: the recovered classes recompile under `javac --release 8` as presented;
//! * the **runtime**: the original frozen bytes and the recovered classes print the same lines
//!   under `java -Xverify:all` (`00,10,` for `L2` is the patrol's frozen behavior);
//! * the **rename**: `L1`'s text differs from the frozen pre-change baseline in the label
//!   spelling alone.

use jarde::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const L1: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/labeled-loop-patrol/fixture/L1.class"
);
const L2: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/labeled-loop-patrol/fixture/L2.class"
);
const L3: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/labeled-loop-patrol/fixture/L3.class"
);
const L3_BASELINE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-10-01/labeled-loop-patrol/fixture/L3.jarde.java"
);
const L1_BASELINE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-10-01/labeled-loop-patrol/fixture/L1.jarde.java"
);
const T1: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/labeled-loop-patrol/tail-coverage/original/T1.class"
);
const T2: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/labeled-loop-patrol/tail-coverage/original/T2.class"
);
const T3: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-01/labeled-loop-patrol/tail-coverage/original/T3.class"
);

/// The run of the original frozen classes' `main` — the text every recovered leg printed.
const EXPECTED_RUNS: &[(&[u8], &str, &str)] = &[
    (L2, "L2", "00,10,\n"),
    (T1, "T1", "00,10,11,12,13,T1;\n"),
    (T2, "T2", "00,01,02,W0;10,20,O0.00,01,00,01,\n"),
    (T3, "T3", "<1>E1.<2>E2.\n"),
];

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

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "jarde-labeled-loop-tail-{tag}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock runs")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("the scratch directory creates");
    dir
}

fn javac(dir: &Path, name: &str, text: &str) {
    std::fs::write(dir.join(format!("{name}.java")), text).expect("the scratch source writes");
    let compile = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-cp")
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .arg(dir.join(format!("{name}.java")))
        .output()
        .expect("the installed JDK provides javac");
    assert!(
        compile.status.success(),
        "javac accepted the recovered source:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );
}

fn java(dir: &Path, name: &str) -> String {
    let run = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg(name)
        .output()
        .expect("the installed JDK provides java");
    assert!(
        run.status.success(),
        "`java -Xverify:all {name}` ran clean:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run's output is text")
}

#[test]
fn the_continue_label_tail_presents_its_trailing_call_without_a_quote() {
    let text = class_text(L2, "L2");
    assert!(
        !text.contains("@bytecode"),
        "the labeled loop's whole tail is owned, no quote remains:\n{text}"
    );
    assert!(
        text.contains(
            "loop: for (local1 = 0; local1 < 2; local1 = local1 + 1) {\n            local2 = 0;"
        ),
        "the labeled loop's header and body open as before, under the new label:\n{text}"
    );
    assert!(
        text.contains("continue loop;"),
        "the non-local continue names the loop by its source-style label:\n{text}"
    );
    assert!(
        text.contains("local0.append('T').append(local1).append(';');\n        }\n        return local0.toString();"),
        "the tail chain statement is the last statement of the loop body:\n{text}"
    );
}

#[test]
fn the_break_label_tail_and_the_mixed_tail_keep_their_statements() {
    let break_tail = class_text(T1, "T1");
    assert!(
        !break_tail.contains("@bytecode")
            && break_tail.contains("continue loop;")
            && break_tail.contains("break loop;")
            && break_tail.contains("local1.append('T').append(local2).append(';');\n        }"),
        "the break-label variant owns its tail and spells both transfers:\n{break_tail}"
    );
    let mixed_tail = class_text(T3, "T3");
    let touch = mixed_tail
        .find("touch(local1, local2);")
        .expect("the tail's void call keeps its statement");
    let chain = mixed_tail
        .find("local1.append('E').append(local2).append('.');")
        .expect("the tail's chained call is owned");
    assert!(
        !mixed_tail.contains("@bytecode") && touch < chain,
        "the mixed tail presents void then chained call in bytecode order:\n{mixed_tail}"
    );
}

#[test]
fn two_nested_labeled_loops_spell_loop_and_loop2() {
    let text = class_text(T2, "T2");
    assert!(
        !text.contains("@bytecode"),
        "both labeled tails are owned:\n{text}"
    );
    let outer = text
        .find("loop2: for (local2 = 0; local2 < arg0; local2 = local2 + 1) {")
        .expect("the outer labeled loop spells loop2");
    let inner = text
        .find("loop: for (local3 = 0; local3 < arg0; local3 = local3 + 1) {")
        .expect("the inner labeled loop spells loop");
    assert!(outer < inner, "the outer loop encloses the inner:\n{text}");
    let inner_continue = text
        .find("continue loop;")
        .expect("the inner continue names the inner loop");
    let outer_continue = text
        .find("continue loop2;")
        .expect("the outer continue names the outer loop");
    assert!(
        inner < inner_continue && inner_continue < outer_continue,
        "the labels follow first-claim write order, the inner loop's continue first:\n{text}"
    );
    assert!(
        text.contains("local1.append('W').append(local3).append(';');")
            && text.contains("local1.append('O').append(local2).append('.');"),
        "both labeled bodies keep their trailing chain statements:\n{text}"
    );
}

#[test]
fn the_unlabeled_control_keeps_its_exact_text_and_l1_changes_only_the_label() {
    let plain = class_text(L3, "L3");
    assert_eq!(
        plain, L3_BASELINE,
        "the unlabeled control's text is untouched by this slice"
    );
    let labeled = class_text(L1, "L1");
    let renamed = L1_BASELINE.replace("jarde_loop_10", "loop");
    assert_eq!(
        labeled, renamed,
        "the rename is the one intended change to L1's frozen text"
    );
}

#[test]
fn the_recovered_classes_compile_and_run_the_original_paths() {
    for (bytes, name, expected) in EXPECTED_RUNS {
        let dir = scratch_dir(name);
        javac(&dir, name, &class_text(bytes, name));
        assert_eq!(
            java(&dir, name),
            *expected,
            "the recovered {name} runs the original class's own lines"
        );
    }
}
