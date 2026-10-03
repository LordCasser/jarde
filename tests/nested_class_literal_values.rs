//! The nested-class-literal patrol's proof slice (change `recover-nested-class-literal-values`):
//! a `CONSTANT_Class` name reached by `ldc` whose `/`- and `$`-separated segments are all legal
//! Java identifiers admits its class literal, and every spelling rides the seams that already
//! exist — the proof layer keeps the pool's `$` form (`spell_reference` never rewrites `$`), and
//! the presentation layer's `InnerClasses` row set decides what the text spells, exactly as it
//! does for declarations. One refusal guards the seam's faithful half: a literal of a
//! **really-nested** class (its own row in the declaring class's `InnerClasses` attribute) whose
//! presented text stays in the pool's `$` form is a top-level class in the compiled text, so a
//! structural-reflection read over it — `getSimpleName`, `getEnclosingClass` — would answer from
//! metadata the text does not state; the build refuses such a call instead of publishing a
//! compilable text that behaves differently.
//!
//! The fixed fixtures are the patrol's own transcriptions
//! (`openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/`): the frozen `A12`
//! family (the two minimal nested-literal forms), the frozen `N2` family (a three-segment chain,
//! a mid-chain literal and a chained structural-reflection receiver), the frozen `A11` family
//! (reflected annotation reads), the frozen `LC` family (the local-class negative whose
//! digit-leading tail the identifier gate refuses), and the frozen variants — `A10` (the healthy
//! top-level five forms this slice must not disturb), `WC1` (a top-level class whose own name
//! carries `$`: its pool spelling is exact, so even `getSimpleName` recovers) and `WV1` (the
//! array form riding the same gate, folded and exact). What this file proves through the public
//! class-source surface:
//!
//! 1. every class literal the subset refused before now recovers with no `@bytecode` quotation,
//!    spelled by the presentation seam the run's own `InnerClasses` rows drive (folded members in
//!    the simple spelling, separated units in the pool's `$` spelling);
//! 2. the recovered texts recompile as the family's units, and the runs print what the original
//!    classes print;
//! 3. the refusal holds in both directions the criterion states: a structural read over a
//!    pool-spelled nested literal is refused (the `N2` chain, and the standalone `A12` read whose
//!    flag states the no-fold presentation), while `getName` over the same literal and the
//!    folded member's `getSimpleName` recover;
//! 4. the negatives hold: `LC`'s local-class literal stays refused, and the top-level five forms
//!    (`A10`) are byte-for-byte what the patrol transcribed on the mainline.

use jarde::budget::Budget;
use jarde::{
    ArtifactInput, ClassNameQuery, ClassRef, ClassSourceOutcome, ClassSourceReport,
    ClassSourceRequest, Engine, EnvironmentPolicy, EnvironmentRequest, LayoutMode, LoaderId,
    MultiReleasePolicy, OperationOutcome, PhysicalScope, RuntimeProfile, task_budget,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const A12_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/f12.jar"
);
const A12_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/o4.out"
);
const N2_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/fn2.jar"
);
const A11_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/a11.jar"
);
const A11_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/o3.out"
);
const A9_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/a9.jar"
);
const A9_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/A9.orig.out"
);
const A10_CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/A10.class"
);
const A10_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/o2.out"
);
const LC_CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/LC.class"
);
const WC1_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture-variants/WC1/wc1.jar"
);
const WC1_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture-variants/WC1/WC1.orig.out"
);
const WV1_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture-variants/WV1/wv1.jar"
);
const WV1_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture-variants/WV1/WV1.orig.out"
);
const A12_CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/fixture/A12.class"
);
const RF_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/residual-boundary/fixture/rf.jar"
);
const RF_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/residual-boundary/results/original.out"
);

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn source_of(artifact: &[u8], class: &str, policy: EnvironmentPolicy) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(artifact.to_vec()), &mut budget())
        .expect("the frozen fixture opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match Engine::new().class_source(slice::from_ref(&snapshot), &request, &mut budget()) {
        Ok(OperationOutcome::Performed(report)) => report,
        Ok(OperationOutcome::Ambiguous(candidates)) => panic!(
            "one frozen family answers one definition, got {} candidates",
            candidates.candidates.len()
        ),
        Ok(OperationOutcome::Incomplete(candidates)) => panic!(
            "one frozen family has an incomplete selection with {} candidates",
            candidates.candidates.len()
        ),
        Err(error) => panic!("one frozen family answers one class-source request: {error}"),
    }
}

fn source_of_jar(artifact: &[u8], class: &str) -> ClassSourceReport {
    source_of(artifact, class, EnvironmentPolicy::PlainJar)
}

fn source_of_class(artifact: &[u8], class: &str) -> ClassSourceReport {
    source_of(artifact, class, EnvironmentPolicy::SingleClass)
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-nested-literal-{label}-{}-{nonce}-{}",
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("a private compilation directory is created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// One recovered family written to a private directory and compiled as one `--release 8`
/// compilation: one unit per `(name, text)`. The directories live as long as the value, so `run`
/// reads what `compile` wrote.
fn compile_project(label: &str, units: &[(&str, &str)]) -> FamilyCompilation {
    let temp = TempDir::new(label);
    let deps = TempDir::new(&format!("{label}-deps"));
    for (name, text) in units {
        fs::write(temp.path().join(format!("{name}.java")), text)
            .expect("the recovered text is written");
    }
    let sources: Vec<String> = units
        .iter()
        .map(|(name, _)| format!("{}.java", name))
        .collect();
    let compiled = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-cp")
        .arg(deps.path())
        .args(&sources)
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "the recovered {label} family recompiles under --release 8:\n{}\n{}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );
    FamilyCompilation { temp, deps }
}

struct FamilyCompilation {
    temp: TempDir,
    deps: TempDir,
}

impl FamilyCompilation {
    /// Runs one compiled class under `-Xverify:all` and returns what the run printed.
    fn run(self, main_class: &str) -> String {
        let run = Command::new("java")
            .args([
                "-Xverify:all",
                "-cp",
                &format!("{}:.", self.deps.path().display()),
            ])
            .arg(main_class)
            .current_dir(self.temp.path())
            .output()
            .expect("java runs");
        assert!(
            run.status.success(),
            "the recompiled {main_class} verifies and runs:\n{}",
            String::from_utf8_lossy(&run.stderr)
        );
        String::from_utf8(run.stdout).expect("the run prints text")
    }
}

#[test]
fn nested_class_literals_recover_and_the_folded_family_runs_like_the_original() {
    let report = source_of_jar(A12_JAR, "A12");
    assert!(
        !report.text.contains("@bytecode"),
        "both minimal nested-literal forms recover without quotation:\n{}",
        report.text
    );
    // The fold declares `Nested` in this text, and the literal rides the same row set: the
    // simple spelling both sides construct.
    for expected in [
        "return A12.class.getSimpleName();",
        "return Nested.class.getSimpleName();",
        "return Nested.class.getName();",
        "static class Nested extends java.lang.Object {",
    ] {
        assert!(
            report.text.contains(expected),
            "`{expected}` is spelled the folded way:\n{}",
            report.text
        );
    }
    let stdout = compile_project("a12", &[("A12", &report.text)]).run("A12");
    assert_eq!(
        stdout.as_bytes(),
        A12_OUT,
        "the recompiled family prints what the original class printed"
    );
}

#[test]
fn reflected_annotation_reads_recover_and_the_family_project_runs_like_the_original() {
    let report = source_of_jar(A11_JAR, "A11");
    assert!(
        !report.text.contains("@bytecode"),
        "all three reflected annotation reads recover without quotation:\n{}",
        report.text
    );
    for expected in [
        // The folded member's literal spells simply; the annotation member keeps its own unit
        // and its pool spelling — one row set drives both sides of each.
        "Marked.class.getAnnotation(A11$Tag.class)",
        "((A11$Tag) local0).value()",
        "Marked.class.isAnnotationPresent(A11$Tag.class)",
        "@A11$Tag(value = \"real\")",
        "static class Marked extends java.lang.Object {",
    ] {
        assert!(
            report.text.contains(expected),
            "`{expected}` is spelled the consistent way:\n{}",
            report.text
        );
    }
    let tag = source_of_jar(A11_JAR, "A11$Tag");
    assert!(
        tag.text.contains("@java.lang.annotation.Retention"),
        "the annotation unit keeps its runtime retention:\n{}",
        tag.text
    );
    let stdout =
        compile_project("a11", &[("A11", &report.text), ("A11$Tag", &tag.text)]).run("A11");
    assert_eq!(
        stdout.as_bytes(),
        A11_OUT,
        "the recompiled family prints what the original class printed"
    );
}

#[test]
fn reflected_compound_annotation_reads_recover_and_run_like_the_original() {
    let report = source_of_jar(A9_JAR, "A9");
    assert!(
        !report.text.contains("@bytecode"),
        "the compound reflection entry point recovers without quotation:\n{}",
        report.text
    );
    for expected in [
        "Target.class.getAnnotation(A9$Meta.class)",
        "Target.class.isAnnotationPresent(java.lang.Deprecated.class)",
        "DefTarget.class.getAnnotation(A9$Meta.class)",
        "@A9$Meta(value = \"m\", n = 7, tags = {\"a\", \"b\"})",
        "static class Target extends java.lang.Object {",
        "static class DefTarget extends java.lang.Object {",
    ] {
        assert!(
            report.text.contains(expected),
            "`{expected}` is spelled the consistent way:\n{}",
            report.text
        );
    }
    let meta = source_of_jar(A9_JAR, "A9$Meta");
    let stdout = compile_project("a9", &[("A9", &report.text), ("A9$Meta", &meta.text)]).run("A9");
    assert_eq!(
        stdout.as_bytes(),
        A9_OUT,
        "the recompiled family prints what the original class printed"
    );
}

#[test]
fn deep_chain_structural_reads_refuse_but_the_name_form_recovers() {
    let report = source_of_jar(N2_JAR, "N2");
    // The single-level fold spells only the root's direct member; the deeper members stay their
    // own physical units in the pool's `$` spelling. `getName` over a pool-spelled name is exact
    // (the binary name survives), and it is the one form of the chain that recovers.
    assert!(
        report.text.contains("return N2$Outer$Mid.class.getName();"),
        "the name form recovers with the pool spelling:\n{}",
        report.text
    );
    // `getSimpleName` and `getEnclosingClass` over a pool-spelled literal would answer from
    // nesting metadata the text does not state — the refusal is the run's own explanation, and
    // the method publishes no statement that could compile and differ.
    assert!(
        !report.text.contains("getSimpleName()"),
        "no structural read over a pool-spelled literal is published:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("getEnclosingClass()"),
        "no enclosing-class read over a pool-spelled literal is published:\n{}",
        report.text
    );
    for name in ["multiLevel", "recvChain"] {
        let member = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == name.as_bytes())
            .unwrap_or_else(|| panic!("the fixture declares `{name}`"));
        let ClassSourceOutcome::Recovered { report: body, .. } = &member.outcome else {
            panic!(
                "the refusal is a run's own explanation: {:?}",
                member.outcome
            )
        };
        assert!(
            body.text.contains("@bytecode")
                && body
                    .text
                    .contains("which this text spells in the pool's form"),
            "`{name}` quotes the pool-spelling refusal:\n{}",
            body.text
        );
    }
}

#[test]
fn local_class_literals_stay_out_of_the_provable_subset() {
    let report = source_of_class(LC_CLASS, "LC");
    // The local class's binary name `LC$1Local` carries a digit-leading tail: no identifier, no
    // literal, no recovery — the run explains itself instead of publishing a statement.
    let local = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"m")
        .expect("the fixture declares `m`");
    let ClassSourceOutcome::Recovered { report: body, .. } = &local.outcome else {
        panic!(
            "the refusal is a run's own explanation: {:?}",
            local.outcome
        )
    };
    assert!(
        body.text.contains("@bytecode 0")
            && body
                .text
                .contains("the instruction at BCI 0 is not part of the provable subset"),
        "the refusal names the unprovable literal instruction:\n{}",
        body.text
    );
    // The anonymous-family member beside it keeps its own recovery: the refusal is exactly
    // scoped to the digit-leading name.
    assert!(
        report.text.contains("getClass()"),
        "the anonymous getClass form beside the negative is untouched:\n{}",
        report.text
    );
}

#[test]
fn top_level_class_literals_keep_their_five_forms_and_run_like_the_original() {
    let report = source_of_class(A10_CLASS, "A10");
    assert!(
        !report.text.contains("@bytecode"),
        "the top-level five forms stay healthy:\n{}",
        report.text
    );
    for expected in [
        "return nameOf(A10.class);",
        "return A10.class.getName();",
        "return A10.class.getSimpleName() + \"!\";",
        "return A10.class.getName().length();",
        "java.lang.Class local0 = A10.class;",
    ] {
        assert!(
            report.text.contains(expected),
            "`{expected}` keeps its top-level spelling:\n{}",
            report.text
        );
    }
    let stdout = compile_project("a10", &[("A10", &report.text)]).run("A10");
    assert_eq!(
        stdout.as_bytes(),
        A10_OUT,
        "the recompiled family prints what the original class printed"
    );
}

#[test]
fn a_literal_dollar_top_level_name_stays_pool_spelled_and_runs() {
    let report = source_of_jar(WC1_JAR, "WC1$Top");
    assert!(
        !report.text.contains("@bytecode"),
        "the literal-`$` top-level self reference recovers:\n{}",
        report.text
    );
    // No `InnerClasses` row states a nesting relation for a top-level `$` name, so both the
    // declaration and the literal keep the pool spelling — legal source on both sides.
    for expected in [
        "public class WC1$Top extends java.lang.Object {",
        "return WC1$Top.class.getSimpleName();",
    ] {
        assert!(
            report.text.contains(expected),
            "`{expected}` keeps the pool spelling:\n{}",
            report.text
        );
    }
    let stdout = compile_project("wc1", &[("WC1$Top", &report.text)]).run("WC1$Top");
    assert_eq!(
        stdout.as_bytes(),
        WC1_OUT,
        "the recompiled family prints what the original class printed"
    );
}

#[test]
fn array_class_literals_of_a_member_admit_and_the_folded_family_runs_like_the_original() {
    let report = source_of_jar(WV1_JAR, "WV1");
    assert!(
        !report.text.contains("@bytecode"),
        "the array forms recover without quotation:\n{}",
        report.text
    );
    // The array element rides the same segment gate; the descriptor spelling appends the
    // dimensions. The fold's token tie reads the array descriptor's element as the class the
    // token names, so the folded text spells every literal the simple way its declaration does —
    // and `getSimpleName` over it answers what the original answers.
    for expected in [
        "return Nested.class.getSimpleName();",
        "return Nested[].class.getName();",
        "return Nested[].class.getName().length();",
        "static class Nested extends java.lang.Object {",
    ] {
        assert!(
            report.text.contains(expected),
            "`{expected}` is spelled the folded way:\n{}",
            report.text
        );
    }
    let stdout = compile_project("wv1", &[("WV1", &report.text)]).run("WV1");
    assert_eq!(
        stdout.as_bytes(),
        WV1_OUT,
        "the recompiled family prints what the original class printed"
    );
}

#[test]
fn a_standalone_read_refuses_structural_reads_over_its_own_pool_spelled_member() {
    // One whole CLASS file: no child resolves, no fold re-spells anything, so the presentation
    // states the pool-spelled-members fact itself. The direct member's literal is then refused in
    // the structural reads — the flag the caller states — while `getName` over the same literal
    // and every top-level literal recover exactly.
    let report = source_of_class(A12_CLASS, "A12");
    assert!(
        report.text.contains("return A12.class.getSimpleName();"),
        "the top-level literal's structural read is exact and recovers:\n{}",
        report.text
    );
    assert!(
        report.text.contains("return A12$Nested.class.getName();"),
        "the name form over the pool-spelled member recovers:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("A12$Nested.class.getSimpleName()"),
        "the structural read over the pool-spelled member is refused:\n{}",
        report.text
    );
    let nested_lit = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"nestedLit")
        .expect("the fixture declares `nestedLit`");
    let ClassSourceOutcome::Recovered { report: body, .. } = &nested_lit.outcome else {
        panic!(
            "the refusal is a run's own explanation: {:?}",
            nested_lit.outcome
        )
    };
    assert!(
        body.text.contains("@bytecode")
            && body
                .text
                .contains("which this text spells in the pool's form"),
        "`nestedLit` quotes the pool-spelling refusal:\n{}",
        body.text
    );
}

#[test]
fn a_refused_fold_reruns_the_structural_reader_and_the_family_stays_loud() {
    // `RF`'s member carries a `<clinit>`, so the fold is refused and the separated presentation
    // keeps the pool spelling. The rerun this road states puts the member back where the
    // class-literal slice found it: the structural read quotes the run's own refusal instead of
    // publishing `RF$Inner.class.getSimpleName()` as a recovery — a compilable text whose
    // `getSimpleName` answers `RF$Inner` where the original answers `Inner`.
    let report = source_of_jar(RF_JAR, "RF");
    assert!(
        !report.text.contains("RF$Inner.class.getSimpleName()"),
        "the structural read over the pool-spelled member is refused:\n{}",
        report.text
    );
    let simple_name = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"simpleName")
        .expect("the fixture declares `simpleName`");
    let ClassSourceOutcome::Recovered { report: body, .. } = &simple_name.outcome else {
        panic!(
            "the refusal is a run's own explanation: {:?}",
            simple_name.outcome
        )
    };
    assert!(
        body.text.contains("@bytecode")
            && body
                .text
                .contains("which this text spells in the pool's form"),
        "`simpleName` quotes the pool-spelling refusal:\n{}",
        body.text
    );
    // The member beside it keeps its own recovery: the fold refusal refused nothing else.
    assert!(
        report.text.contains("RF$Inner.K"),
        "the plain field read beside the refusal is untouched:\n{}",
        report.text
    );
    // And the loud failure is real: the refusal states no return where the source returned, so
    // the family cannot compile into the differently-behaving program.
    let temp = TempDir::new("rf-family");
    fs::write(temp.path().join("RF.java"), &report.text).expect("RF.java is written");
    fs::write(
        temp.path().join("RF$Inner.java"),
        source_of_jar(RF_JAR, "RF$Inner").text,
    )
    .expect("the inner unit is written");
    let compiled = Command::new("javac")
        .arg("--release")
        .arg("8")
        .args(["RF.java", "RF$Inner.java"])
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        !compiled.status.success(),
        "the refused family must not compile into a differently-behaving program:\n{}",
        String::from_utf8_lossy(&compiled.stdout)
    );
    // The member's own unit still presents and compiles beside the refusal.
    let inner = source_of_jar(RF_JAR, "RF$Inner");
    assert!(inner.text.contains("static int init()"));
    compile_project("rf", &[("RF$Inner", &inner.text)]);
    assert_eq!(
        RF_OUT,
        include_bytes!(
            "../openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/residual-boundary/results/original.out"
        ),
        "the frozen original output states `Inner`, `7`"
    );
}
