//! The nested-class-literal patrol's proof slice (change `recover-nested-class-literal-values`):
//! a `CONSTANT_Class` name reached by `ldc` whose `/`- and `$`-separated segments are all legal
//! Java identifiers admits its class literal, and every spelling rides the seams that already
//! exist — the proof layer keeps the pool's `$` form (`spell_reference` never rewrites `$`), and
//! the presentation layer's `InnerClasses` row set decides what the text spells, exactly as it
//! does for declarations. Nothing here adds a spelling rule.
//!
//! The fixed fixtures are the patrol's own transcriptions
//! (`openspec/evidence/java-syntax-2026-10-04/nested-class-literal-patrol/`): the frozen `A12`
//! family (the two minimal nested-literal forms), the frozen `N2` family (a three-segment chain,
//! a mid-chain literal and a chained structural-reflection receiver), the frozen `A11` family
//! (reflected annotation reads), the frozen `LC` family (the local-class negative whose
//! digit-leading tail the identifier gate refuses), and the frozen variants — `A10` (the healthy
//! top-level five forms this slice must not disturb), `WC1` (a top-level class whose own name
//! carries `$`, presented pool-spelled on both sides) and `WV1` (the array form riding the same
//! gate). What this file proves through the public class-source surface:
//!
//! 1. every nested or array class literal the subset refused before now recovers with no
//!    `@bytecode` quotation, spelled by the presentation seam the run's own `InnerClasses` rows
//!    drive (folded members in the simple spelling, separated units in the pool's `$` spelling);
//! 2. the recovered texts recompile as the family's units, and the runs print what the original
//!    classes print wherever the presentation preserves source nesting;
//! 3. the negatives hold: `LC`'s local-class literal stays refused, and the top-level five forms
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
fn deep_chains_recover_with_pool_spelling_and_the_family_project_compiles() {
    let report = source_of_jar(N2_JAR, "N2");
    assert!(
        !report.text.contains("@bytecode"),
        "all three deep-chain forms recover without quotation:\n{}",
        report.text
    );
    // The fold claims the root's direct member only; the deeper members stay their own physical
    // units, so the references keep the pool's `$` spelling — exactly the rule the
    // nested-spelling slice states for self-nested names without a fold.
    for expected in [
        "return N2$Outer$Mid$Leaf.class.getSimpleName();",
        "return N2$Outer$Mid.class.getName();",
        "return N2$Outer$Mid$Leaf.class.getEnclosingClass().getSimpleName();",
        "static class Outer extends java.lang.Object {",
    ] {
        assert!(
            report.text.contains(expected),
            "`{expected}` keeps the pool spelling:\n{}",
            report.text
        );
    }
    let mid = source_of_jar(N2_JAR, "N2$Outer$Mid");
    let leaf = source_of_jar(N2_JAR, "N2$Outer$Mid$Leaf");
    // The three recovered units compile as one project: a flat unit named `N2$Outer$Mid` resolves
    // the pool-spelled reference by its own binary name. The structural-reflection forms over the
    // flat units are the fold-depth domain's residual — recorded, not pinned here
    // (results-values/README.md, 遗留 1).
    compile_project(
        "n2",
        &[
            ("N2", &report.text),
            ("N2$Outer$Mid", &mid.text),
            ("N2$Outer$Mid$Leaf", &leaf.text),
        ],
    );
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
fn array_class_literals_of_a_member_admit_and_the_name_forms_run_like_the_original() {
    let report = source_of_jar(WV1_JAR, "WV1");
    assert!(
        !report.text.contains("@bytecode"),
        "the array forms recover without quotation:\n{}",
        report.text
    );
    // The array element rides the same segment gate; the descriptor spelling appends the
    // dimensions, and this family's presentation keeps the pool spelling (the fold's token-tie
    // anchor gap for array class constants is the fold domain's residual —
    // results-values/README.md, 遗留 2).
    for expected in [
        "return WV1$Nested.class.getSimpleName();",
        "return WV1$Nested[].class.getName();",
        "return WV1$Nested[].class.getName().length();",
    ] {
        assert!(
            report.text.contains(expected),
            "`{expected}` is spelled the pool way:\n{}",
            report.text
        );
    }
    let nested = source_of_jar(WV1_JAR, "WV1$Nested");
    // The `getName()` forms are exact under the flat units — binary names survive — as are the
    // dimensions; the `getSimpleName` line of the run is the fold residual and is not pinned.
    let stdout = compile_project(
        "wv1",
        &[("WV1", &report.text), ("WV1$Nested", &nested.text)],
    )
    .run("WV1");
    assert!(
        stdout.ends_with("[LWV1$Nested;\n13\n"),
        "the array-name forms print what the original class printed:\n{stdout}"
    );
}
