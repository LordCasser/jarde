//! `preserve-local-scope-across-exception-regions` tasks 1.2/1.3: the declaration plan's three
//! answers per local, pinned at the entry point the CLI calls.
//!
//! The declaration planner reads one local's whole definition–use slice and answers one of three
//! ways ([`crates/jarde-java/src/build.rs`]'s `DeclarationPlacement`):
//!
//! * **lexical owner** — every access sits in one region, so the write that fills the local carries
//!   the declaration there and nothing outside that region reads it;
//! * **liftable** — the accesses span regions, and a declaration written at their common owner is in
//!   scope for all of them, with the writes the builder will really present and the join's definite
//!   assignment proved by SSA;
//! * **incomplete** — a quoted fallback region or an unclaimed block is on the slice, or the write
//!   evidence cannot be closed, so the dependent slice is refused whole and the artifact keeps the
//!   refusal sentence and every instruction it covers.
//!
//! `tests/fixtures/preserve-local-scope-plan/` holds the frozen input (see its `README.md` for the
//! two compiler legs, the digests and the recorded behavior): `ScopePlan` is the positive class,
//! where a catch-only local, a `try`/`catch` join, an `if` join and two nested-`try` joins all
//! recover, and `ScopePlanCrossing` is the negative class, where the resource-across-finally and
//! flat-`finally` shapes keep the third answer. The two legs are compiled from one source by javac
//! 23.0.1 `--release 8 -g:none` and by the real javac 8, and both are read here: the classification
//! is a statement about control flow, not about one compiler's lowering.
//!
//! This slice changes no recovery code. It pins the classification the earlier tasks built (1.1's
//! two shapes, 1.4's handler bindings, 1.5's computed write), the origins each lifted declaration
//! covers, and the refusals that must not move — the zero-regression surface the re-sliced
//! region-layer work (the two patrols' `exception-handler shape` / `finally-copy merge` gates, which
//! are *not* the declaration planner's) will be measured against.

use jarde::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const V8: &[u8] = include_bytes!("fixtures/preserve-local-scope-plan/v8/ScopePlan.class");
const V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/preserve-local-scope-plan/v8-javac8/ScopePlan.class");
const CROSSING_V8: &[u8] =
    include_bytes!("fixtures/preserve-local-scope-plan/v8/ScopePlanCrossing.class");
const CROSSING_V8_JAVAC8: &[u8] =
    include_bytes!("fixtures/preserve-local-scope-plan/v8-javac8/ScopePlanCrossing.class");

/// The whole-class render both legs answer, byte for byte: the positive class's five
/// members and the classification's first two answers, with the layer's own envelope lines.
const SCOPE_PLAN: &str = r##"// jarde: presentation of `ScopePlan` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ScopePlan extends java.lang.Object {
    private ScopePlan() {
        // @method <init>()V
        // @declaration a constructor of `ScopePlan`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int catchOnly(boolean arg0) {
        // @method catchOnly(Z)I
        // @declaration a static method of `ScopePlan`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            if (arg0) {
                throw null;
            } else {
                return 1;
            }
        } catch (java.lang.NullPointerException local1) {
            int local2 = 2;
            return local2;
        }
    }

    static int assignedAcrossTry(boolean arg0) {
        // @method assignedAcrossTry(Z)I
        // @declaration a static method of `ScopePlan`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        try {
            if (arg0) {
                throw null;
            } else {
                local1 = 3;
            }
        } catch (java.lang.NullPointerException local2) {
            local1 = 4;
        }
        return local1;
    }

    static int assignedAcrossIf(boolean arg0) {
        // @method assignedAcrossIf(Z)I
        // @declaration a static method of `ScopePlan`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        if (arg0) {
            local1 = 5;
        } else {
            local1 = 6;
        }
        return local1;
    }

    static int nestedHandlerOnly(java.lang.Runnable arg0) {
        // @method nestedHandlerOnly(Ljava/lang/Runnable;)I
        // @declaration a static method of `ScopePlan`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        try {
            try {
                arg0.run();
            } catch (java.lang.IllegalArgumentException local1) {
                int local2 = 8;
                return local2;
            }
        } catch (java.lang.RuntimeException local1) {
            return 9;
        }
        return 0;
    }

    static int nestedAcross(boolean arg0) {
        // @method nestedAcross(Z)I
        // @declaration a static method of `ScopePlan`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        try {
            try {
                if (arg0) {
                    throw null;
                } else {
                    local1 = 10;
                }
            } catch (java.lang.NullPointerException local2) {
                local1 = 11;
            }
        } catch (java.lang.RuntimeException local2) {
            local1 = 12;
        }
        return local1;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ScopePlan`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.print("" + catchOnly(false) + "," + catchOnly(true) + ",");
        java.lang.System.out.print("" + assignedAcrossTry(false) + "," + assignedAcrossTry(true) + ",");
        java.lang.System.out.print("" + assignedAcrossIf(false) + "," + assignedAcrossIf(true) + ",");
        java.lang.System.out.print((java.lang.String) new java.lang.StringBuilder().append(nestedHandlerOnly((java.lang.Runnable) null)).append(",").toString());
        java.lang.System.out.print("" + nestedAcross(false) + "," + nestedAcross(true) + "\n");
        return;
    }
}
"##;

/// The sentence the third answer states, verbatim: the patrols' recorded refusal, which a local
/// whose slice crosses a quoted fallback region keeps.
const CROSSING: &str = "local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice";

/// The recorded behavior of both legs' own `ScopePlan` under `java -Xverify:all` (the fixture
/// README records the command).
const BEHAVIOR: &str = "1,2,3,4,6,5,9,10,11\n";

fn budget() -> Budget {
    Budget::new(task_limits(&[]).expect("the task defaults are a bounded budget"))
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("a committed fixture opens as a standalone CLASS")
}

/// One class-source presentation of one committed sample, under the entry point the CLI calls,
/// with the source map materialized: the origins are half of what this fixture pins.
fn class_source_of(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceReport {
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
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            &mut budget(),
        )
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

/// The recovery report of one member's own run.
fn run_of<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` was read by its own run, got {other:?}"),
    }
}

/// Where one substring sits in one text, stated as the panic a missing one deserves.
fn at(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("the text presents `{needle}`:\n{text}"))
}

/// Where one substring sits after one position of the same text.
fn after(text: &str, needle: &str, from: usize) -> usize {
    from + at(&text[from..], needle)
}

/// The bytecode index the narrowest segment covering one text offset is anchored at: the origin of
/// the instruction that wrote exactly this text.
fn origin_of(run: &RecoveryReport, offset: usize) -> u32 {
    let mut covering: Vec<_> = run
        .source_map
        .segments()
        .iter()
        .filter(|segment| segment.start() <= offset && offset < segment.end())
        .collect();
    covering.sort_by_key(|segment| segment.len());
    let segment = covering
        .first()
        .unwrap_or_else(|| panic!("a segment covers offset {offset} of:\n{}", run.text));
    segment.origin().primary().bci()
}

#[test]
fn the_three_answers_are_the_rendered_class() {
    let mut texts = Vec::new();
    for (leg, bytes) in [("v8", V8), ("v8-javac8", V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = class_source_of(&snapshot, "ScopePlan");
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `ScopePlan`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        // Every member of the positive class is read by its own run: the classification's first two
        // answers leave no refusal anywhere in the class, which is what makes the whole text
        // compilable.
        for method in &report.methods {
            let name = String::from_utf8_lossy(&method.item.name.raw().0).into_owned();
            let run = run_of(&report, &name);
            assert!(
                run.produced() && run.content == RecoveryContent::ContainsStatements,
                "{leg}/{name}: the member is presented whole, got {:?}:\n{}",
                run.content,
                run.text
            );
        }
        assert!(
            !report.text.contains("// jarde: not recovered"),
            "{leg}: no member of the positive class is refused:\n{}",
            report.text
        );
        texts.push((leg, report.text.clone()));
    }
    // One text, byte for byte, on both legs: the two compilers lower the source differently and the
    // classification reads the same answers out of both.
    assert_eq!(texts[0].1, texts[1].1, "the two legs present one text");
    assert_eq!(texts[0].1, SCOPE_PLAN, "the positive class is pinned whole");

    // The first answer: the handler's own local stays inside the clause that declares it, and
    // nothing after the clause reads it. Read on the member's own body text, where the clause's
    // closing brace is the scope's own end.
    let body = run_of(&class_source_of(&open(V8), "ScopePlan"), "catchOnly")
        .text
        .clone();
    let clause = at(&body, "} catch (java.lang.NullPointerException local1) {");
    let declared = at(&body, "        int local2 = 2;\n");
    let read = after(&body, "        return local2;\n", declared);
    let clause_end = after(&body, "\n    }", read);
    assert!(
        clause < declared && declared < read && read < clause_end,
        "the catch-only local is declared and read inside its clause:\n{body}"
    );
    assert!(
        !body[clause_end..].contains("local2"),
        "no statement after the catch reads the catch-only local:\n{body}"
    );

    // The second answer: each lifted local is declared above the region its accesses span, and every
    // write in that region is an assignment of the one declaration. Each join is read on its own
    // member's body text, where the declarations stand at the body's own indentation.
    let join_body = run_of(
        &class_source_of(&open(V8), "ScopePlan"),
        "assignedAcrossTry",
    )
    .text
    .clone();
    let join = at(&join_body, "    int local1;\n    try {");
    let normal = after(&join_body, "local1 = 3;", join);
    let handler = after(&join_body, "local1 = 4;", normal);
    let join_read = after(&join_body, "return local1;", handler);
    assert!(
        join < normal && normal < handler && handler < join_read,
        "the try/catch join declares above the `try`, assigns on both paths and reads after:\n{join_body}"
    );
    let arm_body = run_of(&class_source_of(&open(V8), "ScopePlan"), "assignedAcrossIf")
        .text
        .clone();
    let arm = at(&arm_body, "    int local1;\n    if (arg0) {");
    let then_arm = after(&arm_body, "local1 = 5;", arm);
    let else_arm = after(&arm_body, "local1 = 6;", then_arm);
    let arm_read = after(&arm_body, "return local1;", else_arm);
    assert!(
        arm < then_arm && then_arm < else_arm && else_arm < arm_read,
        "the `if` join declares above the branch and assigns in both arms:\n{arm_body}"
    );
    let nested_body = run_of(&class_source_of(&open(V8), "ScopePlan"), "nestedAcross")
        .text
        .clone();
    let outer = at(&nested_body, "    int local1;\n    try {\n        try {");
    let inner_write = after(&nested_body, "local1 = 10;", outer);
    let inner_clause = after(&nested_body, "local1 = 11;", inner_write);
    let outer_clause = after(&nested_body, "local1 = 12;", inner_clause);
    let nested_read = after(&nested_body, "return local1;", outer_clause);
    assert!(
        outer < inner_write
            && inner_write < inner_clause
            && inner_clause < outer_clause
            && outer_clause < nested_read,
        "the nested join covers the inner `try`, the inner clause and the outer clause:\n{nested_body}"
    );
}

#[test]
fn the_lifted_declaration_carries_the_origin_of_every_region_it_covers() {
    for (leg, bytes) in [("v8", V8), ("v8-javac8", V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = class_source_of(&snapshot, "ScopePlan");

        // The `try`/`catch` join: the declaration is anchored at the first write the plan read (the
        // store on the protected path), and each of the two writes and the join's read keeps its own
        // origin. The three are the three regions the one declaration covers.
        let run = run_of(&report, "assignedAcrossTry");
        let text = run.text.as_str();
        let declaration = at(text, "    int local1;\n");
        let normal = at(text, "            local1 = 3;\n");
        let handler = at(text, "        local1 = 4;\n");
        let read = at(text, "    return local1;\n");
        assert_eq!(
            origin_of(run, declaration),
            origin_of(run, normal),
            "{leg}: the lifted declaration is anchored at the write that fills it first:\n{text}"
        );
        assert_eq!(
            (
                origin_of(run, normal),
                origin_of(run, handler),
                origin_of(run, read)
            ),
            (7, 13, 15),
            "{leg}: the protected write, the handler write and the join read are the BCIs the class \
             states for them (javap: `istore_1` at 7, `istore_1` at 13, `iload_1` at 14/15):\n{text}"
        );

        // The `if` join: the declaration is anchored at the then-arm's write, and the else arm and
        // the read after the branch are theirs.
        let run = run_of(&report, "assignedAcrossIf");
        let text = run.text.as_str();
        let declaration = at(text, "    int local1;\n");
        let then_arm = at(text, "        local1 = 5;\n");
        let else_arm = at(text, "        local1 = 6;\n");
        let read = at(text, "    return local1;\n");
        assert_eq!(
            origin_of(run, declaration),
            origin_of(run, then_arm),
            "{leg}: the declaration above the `if` is the then-arm's own write:\n{text}"
        );
        assert_eq!(
            (
                origin_of(run, then_arm),
                origin_of(run, else_arm),
                origin_of(run, read)
            ),
            (5, 11, 13),
            "{leg}: both arms and the read keep their own origins:\n{text}"
        );

        // The nested join: one declaration above the outer `try` covers the inner protected write,
        // the inner clause's write, the outer clause's write and the read after both.
        let run = run_of(&report, "nestedAcross");
        let text = run.text.as_str();
        let declaration = at(text, "    int local1;\n");
        let inner_write = at(text, "                local1 = 10;\n");
        let inner_clause = at(text, "            local1 = 11;\n");
        let outer_clause = at(text, "        local1 = 12;\n");
        let read = at(text, "    return local1;\n");
        assert_eq!(
            origin_of(run, declaration),
            origin_of(run, inner_write),
            "{leg}: the declaration is anchored at the innermost write:\n{text}"
        );
        assert_eq!(
            (
                origin_of(run, inner_write),
                origin_of(run, inner_clause),
                origin_of(run, outer_clause),
                origin_of(run, read)
            ),
            (8, 15, 22, 24),
            "{leg}: every region of the nested join keeps its own origin:\n{text}"
        );

        // The first answer's own origin: the catch-only local is written inside the clause, its
        // declaration is the catch body's own store, and the whole `try` statement — the one node
        // that presents the handler entry the clause is entered with — is what contains it.
        let run = run_of(&report, "catchOnly");
        let text = run.text.as_str();
        let statement = run
            .source_map
            .segments()
            .iter()
            .find(|segment| segment.text(text).starts_with("    try {"))
            .unwrap_or_else(|| panic!("{leg}: the `try` statement is one node:\n{text}"));
        let clause = at(text, "} catch (java.lang.NullPointerException local1) {");
        let declaration = at(text, "        int local2 = 2;\n");
        let read = at(text, "        return local2;\n");
        assert!(
            statement.start() < clause && clause < declaration && declaration < read,
            "{leg}: the catch-only local is written inside the clause:\n{text}"
        );
        assert!(
            read < statement.end(),
            "{leg}: the catch-only local's own read is inside the statement's own node:\n{text}"
        );
        assert!(
            statement
                .origin()
                .derived()
                .iter()
                .any(|origin| origin.bci() == 8),
            "{leg}: the statement's node presents the handler entry (javap: `astore_1` at 8) its \
             clause is entered with:\n{text}"
        );
        assert_eq!(
            origin_of(run, declaration),
            10,
            "{leg}: the declaration is the catch body's own store (javap: `istore_2` at 10):\n{text}"
        );
    }
}

#[test]
fn a_crossing_local_keeps_the_dependent_slice_refused() {
    for (leg, bytes) in [("v8", CROSSING_V8), ("v8-javac8", CROSSING_V8_JAVAC8)] {
        let snapshot = open(bytes);
        let report = class_source_of(&snapshot, "ScopePlanCrossing");
        assert!(
            report
                .text
                .starts_with("// jarde: presentation of `ScopePlanCrossing`"),
            "{leg}: the render states its own header before anything is counted:\n{}",
            report.text
        );
        for (name, quote) in [
            ("flatFinally", "// @bytecode 0 15 28"),
            ("resourceAcrossFinally", "// @bytecode 0 27 36 42 52"),
        ] {
            let run = run_of(&report, name);
            assert_eq!(
                run.content,
                RecoveryContent::ExplanationOnly,
                "{leg}/{name}: the dependent slice is refused whole:\n{}",
                run.text
            );
            assert_eq!(
                run.representation,
                Representation::Mixed,
                "{leg}/{name}: the refusal is the mixed presentation it was:\n{}",
                run.text
            );
            assert!(
                run.text.contains(quote) && run.text.contains(CROSSING),
                "{leg}/{name}: the refusal names every instruction it covers and keeps its own \
                 sentence:\n{}",
                run.text
            );
            assert!(
                !run.text.contains("return local1;") && !run.text.contains("int local1"),
                "{leg}/{name}: no read of the crossing local is written outside its region:\n{}",
                run.text
            );
        }
    }
}

/// One private directory a test compiles and runs in.
struct TestDirectory(PathBuf);

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-local-scope-plan-{label}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("a private compilation directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Runs one class of one directory under `-Xverify:all`, answering its standard output.
fn run_class(directory: &Path, class: &str) -> String {
    let run = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(directory)
        .arg(class)
        .current_dir(directory)
        .output()
        .expect("java runs");
    assert!(
        run.status.success(),
        "{} runs under -Xverify:all:\n{}",
        class,
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run prints text")
}

#[test]
fn the_recovered_class_recompiles_and_answers_what_the_original_answers() {
    // The whole class the positive fixture is: its own members' bodies, with the layer's own
    // `//` envelope lines stripped, which is exactly what the roundtrip scripts compile.
    let snapshot = open(V8);
    let report = class_source_of(&snapshot, "ScopePlan");
    let stripped: String = report
        .text
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| format!("{line}\n"))
        .collect();

    let original = TestDirectory::new("original");
    std::fs::write(original.path().join("ScopePlan.class"), V8).expect("the fixture is written");
    let expected = run_class(original.path(), "ScopePlan");
    assert_eq!(
        expected, BEHAVIOR,
        "the fixture's own class is the baseline"
    );

    let recovered = TestDirectory::new("recovered");
    std::fs::write(recovered.path().join("ScopePlan.java"), &stripped)
        .expect("the recovered text is written");
    let compiled = Command::new("javac")
        .args(["--release", "8", "-Xlint:-options"])
        .arg("ScopePlan.java")
        .current_dir(recovered.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "the recovered class recompiles under `javac --release 8`:\n{}\n{stripped}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert_eq!(
        run_class(recovered.path(), "ScopePlan"),
        expected,
        "the recovered class answers what the class file's own bytes answer"
    );
}

#[test]
fn a_bounded_or_cancelled_run_commits_no_partial_body() {
    let snapshot = open(V8);

    // A body that cannot be written whole is not published half-written: the output charge is the
    // bound the emitter pays, and the class request answers an incomplete outcome with no text.
    let mut output_stopped = task_budget(&[
        BudgetOverride::new("output_bytes", 1).expect("a positive output bound is valid")
    ])
    .expect("the positive output bound is valid");
    let stopped = Engine::new()
        .class_source(
            slice::from_ref(&snapshot),
            &ClassSourceRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal("ScopePlan"),
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
            },
            &mut output_stopped,
        )
        .expect("the bounded request reports an outcome");
    assert!(
        matches!(stopped, OperationOutcome::Incomplete(_)),
        "a body that cannot write source does not publish a complete class: {stopped:?}"
    );

    // The caller's cancellation is the same contract at the same points: no half-written try/catch
    // and no source map are committed.
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &ClassSourceRequest {
                class: ClassRef::Name {
                    class: ClassNameQuery::internal("ScopePlan"),
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
            },
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            &mut cancelled,
        )
        .expect("the cancelled request reports an outcome");
    assert!(
        matches!(stopped, OperationOutcome::Incomplete(_)),
        "a cancelled class request does not publish a complete class: {stopped:?}"
    );
}
