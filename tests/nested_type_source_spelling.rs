//! The nested-name spelling patrol's presentation slice: a class file's `$`-spelled nested type
//! reference is written the way Java source nests it, and the classes that could not recompile
//! before now do.
//!
//! The fixed fixtures are the patrol's own transcriptions
//! (`openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/`): `V1` with its nested
//! enum `V1$Op` (the original finding: the parameter type and the constant qualifier were spelled
//! with the pool's `$`), and the four frozen variants — a cross-class nested reference (`VN1`), a
//! packaged deep chain in both the self-nested and the cross-class form (`VN2`), an anonymous
//! synthetic name that must stay verbatim (`VN3`), and static nested class/interface positions
//! (`VN4`). What this file proves through the public class-source surface:
//!
//! 1. every source-syntax position — the member declaration's parameter and return types, a local
//!    declaration, a `new`, a cast, an `instanceof`, a static call or field qualifier — spells the
//!    nested name the way source nests it (`Op` inside `V1`, `Other.Inner` inside `VN1`,
//!    `p.A.B.C` across packages), while the `// @method` descriptor comments keep the pool's own
//!    spelling;
//! 2. the conversion is **evidence-gated**: `VN3`'s anonymous `VN3$1` keeps its pool spelling in a
//!    `new` position, and a top-level name that merely carries `$` (`Named$Top`, pinned by the
//!    static-member family identity tests) is a member of nothing no rule may split;
//! 3. the whole recovered families recompile under `javac --release 8`, verify under
//!    `-Xverify:all`, and print what the original classes print.

use jarde::budget::Budget;
use jarde::{
    ArtifactInput, ClassNameQuery, ClassRef, ClassSourceReport, ClassSourceRequest, Engine,
    EnvironmentPolicy, EnvironmentRequest, LayoutMode, LoaderId, MultiReleasePolicy,
    OperationOutcome, PhysicalScope, RuntimeProfile, task_budget,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const V1_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture/fam.jar"
);
const V1_CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture/V1.class"
);
const V1_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture/orig.out"
);

const OTHER: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN1/Other.class"
);
const OTHER_INNER: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN1/Other$Inner.class"
);
const VN1_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN1/vn1.jar"
);
const VN1_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN1/VN1.orig.out"
);

const VN2_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN2/vn2.jar"
);

const VN3_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN3/vn3.jar"
);

const VN4_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN4/vn4.jar"
);
const VN4_OUT: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN4/VN4.orig.out"
);
const OTHER_BOX: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN4/Other$Box.class"
);
const OTHER_MARK: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN4/Other$Mark.class"
);
const OTHER_TAGGED: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/nested-name-spelling-patrol/fixture-variants/VN4/Other$Tagged.class"
);

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are bounded")
}

fn source_of(jar: &[u8], class: &str) -> ClassSourceReport {
    source_with(jar, class, EnvironmentPolicy::PlainJar)
}

fn source_with(jar: &[u8], class: &str, policy: EnvironmentPolicy) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(jar.to_vec()), &mut budget())
        .expect("the frozen family JAR opens");
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
            "jarde-nested-spelling-{label}-{}-{nonce}-{}",
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

/// Compiles one recovered text as `--release 8` Java, runs it under `-Xverify:all`, and returns
/// what the run printed. The classpath holds exactly the fixture classes the family's own members
/// need (the original compiled family, never a rewritten text).
fn recompile_and_run(
    label: &str,
    text: &str,
    class_name: &str,
    classpath_files: &[(&str, &[u8])],
) -> String {
    let temp = TempDir::new(label);
    let deps = TempDir::new(&format!("{label}-deps"));
    for (name, bytes) in classpath_files {
        let path = deps.path().join(*name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the dependency directory tree is created");
        }
        fs::write(path, bytes).expect("one dependency class is written");
    }
    fs::write(temp.path().join(format!("{class_name}.java")), text)
        .expect("the recovered text is written");
    let compiled = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-cp")
        .arg(deps.path())
        .arg(format!("{class_name}.java"))
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "the recovered {} recompiles under --release 8:\n{}",
        class_name,
        String::from_utf8_lossy(&compiled.stderr)
    );
    let run = Command::new("java")
        .args([
            "-Xverify:all",
            "-cp",
            &format!("{}:.", deps.path().display()),
        ])
        .arg(class_name)
        .current_dir(temp.path())
        .output()
        .expect("java runs");
    assert!(
        run.status.success(),
        "the recompiled {} verifies and runs:\n{}",
        class_name,
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run prints text")
}

#[test]
fn nested_enum_references_spell_as_source_and_the_family_recompiles() {
    let report = source_of(&V1_JAR, "V1");
    // The patrol's two findings: the parameter type of `enumSwitch` and the constant qualifier in
    // `main`, both in the self-nested simple spelling the source file itself used.
    assert!(
        report
            .text
            .contains("public static int enumSwitch(Op arg0, int arg1)"),
        "the parameter type is the enum's own name inside V1:\n{}",
        report.text
    );
    assert!(
        report
            .text
            .contains(".append(enumSwitch(Op.MUL, 7)).append(\":\").append(enumSwitch(Op.ADD, 7))"),
        "the constant qualifier is the enum's own name inside V1:\n{}",
        report.text
    );
    // The descriptor comments keep the pool's spelling by design.
    assert!(
        report.text.contains("// @method enumSwitch(LV1$Op;I)I"),
        "the envelope comment keeps the pool spelling:\n{}",
        report.text
    );
    let stdout = recompile_and_run("v1", &report.text, "V1", &[]);
    assert_eq!(
        stdout.as_bytes(),
        V1_OUT,
        "the recompiled family prints what the original class printed"
    );
}

#[test]
fn cross_class_nested_references_spell_dotted_in_every_position() {
    let report = source_of(&VN1_JAR, "VN1");
    for expected in [
        "static Other.Inner box = new Other.Inner();",
        "static int use(Other.Inner arg0, int arg1)",
        "Other.Inner local2 = new Other.Inner();",
        "instanceof Other.Inner;",
        "((Other.Inner) local2).id(arg1)",
        "int local5 = Other.Inner.twice(arg1);",
    ] {
        assert!(
            report.text.contains(expected),
            "`{expected}` is spelled the source way:\n{}",
            report.text
        );
    }
    assert!(
        !report
            .text
            .replace("// @method use(LOther$Inner;I)I", "")
            .contains("Other$Inner"),
        "no source-syntax position keeps the pool spelling:\n{}",
        report.text
    );
    let stdout = recompile_and_run(
        "vn1",
        &report.text,
        "VN1",
        &[("Other.class", OTHER), ("Other$Inner.class", OTHER_INNER)],
    );
    assert_eq!(
        stdout.as_bytes(),
        VN1_OUT,
        "the recompiled cross-class family prints what the original class printed"
    );
}

#[test]
fn packaged_deep_chains_spell_dotted_and_self_nested_keeps_the_pool_name() {
    let report = source_of(&VN2_JAR, "p/VN2");
    // A cross-class chain keeps its package and dots every `$`.
    assert!(
        report
            .text
            .contains("static int deep(p.A.B.C arg0, int arg1)"),
        "the cross-class chain is fully dotted:\n{}",
        report.text
    );
    assert!(
        report.text.contains("return p.A.B.C.three(arg1);"),
        "the cross-class qualifier is fully dotted:\n{}",
        report.text
    );
    assert!(
        report.text.contains("deep(new p.A.B.C(), 4)"),
        "the cross-class creation is fully dotted:\n{}",
        report.text
    );
    // The self-nested chain keeps the pool spelling: this text does not fold `Mid`/`Leaf` into
    // itself, so the member is presented as its own physical unit under exactly this name and a
    // project built from those units resolves it. Only a fold that declares the member here may
    // spell it as source nesting.
    assert!(
        report
            .text
            .contains("static int own(p.VN2$Mid$Leaf arg0, int arg1)"),
        "the self-nested chain keeps the pool spelling:\n{}",
        report.text
    );
    assert!(
        report.text.contains("return p.VN2$Mid$Leaf.leaf(arg1);"),
        "the self-nested qualifier keeps the pool spelling:\n{}",
        report.text
    );
}

#[test]
fn synthetic_anonymous_names_stay_verbatim() {
    let report = source_of(&VN3_JAR, "VN3");
    // The interface is a member of VN3 itself and this text folds no declaration for it, so it
    // keeps the pool spelling like every self-nested reference without a fold; the anonymous
    // capture class keeps the pool spelling the separated presentation writes for its own class.
    assert!(
        report.text.contains("static VN3$Op capture(int arg0)"),
        "the self-nested interface return type keeps the pool spelling:\n{}",
        report.text
    );
    assert!(
        report.text.contains("return new VN3$1(arg0);"),
        "the anonymous synthetic name keeps the pool spelling:\n{}",
        report.text
    );
    assert!(
        report.text.contains("// @method capture(I)LVN3$Op;"),
        "the envelope comment keeps the pool spelling:\n{}",
        report.text
    );
}

#[test]
fn static_nested_class_and_interface_positions_spell_dotted() {
    let report = source_of(&VN4_JAR, "VN4");
    // The static nested class and interface of another top-level class, in every type position
    // this subset writes: a parameter, a local declaration, a `new`, a cast and an `instanceof`.
    for expected in [
        "static int area(Other.Box arg0)",
        "return arg0 instanceof Other.Mark;",
        "Other.Tagged local1 = new Other.Tagged();",
        "area((Other.Box) local1)",
        "((Other.Mark) local1).tag()",
    ] {
        assert!(
            report.text.contains(expected),
            "`{expected}` is spelled the source way:\n{}",
            report.text
        );
    }
    let stdout = recompile_and_run(
        "vn4",
        &report.text,
        "VN4",
        &[
            ("Other.class", OTHER),
            ("Other$Inner.class", OTHER_INNER),
            ("Other$Box.class", OTHER_BOX),
            ("Other$Mark.class", OTHER_MARK),
            ("Other$Tagged.class", OTHER_TAGGED),
        ],
    );
    assert_eq!(
        stdout.as_bytes(),
        VN4_OUT,
        "the recompiled static-nested family prints what the original class printed"
    );
}

#[test]
fn a_refused_fold_keeps_the_pool_spelling_of_the_member() {
    // The single-class policy opens `V1.class` alone: the nested enum definition cannot be
    // resolved from the environment, the fold is refused, and no declaration of `Op` exists in
    // the text. The self-nested references then keep the pool spelling — exactly the rule that
    // keeps a flat family of physical member units resolvable — while the same jar input with
    // the child present folds and spells `Op` (the test above).
    let report = source_with(&V1_CLASS, "V1", EnvironmentPolicy::SingleClass);
    assert!(
        report
            .text
            .contains("public static int enumSwitch(V1$Op arg0, int arg1)"),
        "a refused fold keeps the pool parameter type:\n{}",
        report.text
    );
    assert!(
        report.text.contains("enumSwitch(V1$Op.MUL, 7)"),
        "a refused fold keeps the pool qualifier:\n{}",
        report.text
    );
    assert!(
        !report.text.contains("enum Op"),
        "no enum declaration is rendered without the child definition:\n{}",
        report.text
    );
}
