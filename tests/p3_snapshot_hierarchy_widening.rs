//! `recover-snapshot-hierarchy-widening`: the invocation argument whose presented type and
//! required type are both physical classes of the analyzed snapshot, and whose safe upcast the
//! presented type's own class-file header chain (superclass + interfaces, transitively, bounded)
//! states.
//!
//! The committed inputs are the patrol's frozen `fam.jar` (`I1` family, the slice's anchor), this
//! slice's `hier-fam.jar` (`H1` variant family) and `hier-neg.jar` (`H2` negative family), plus
//! `Helper.class` — the runtime-only superclass that makes the snapshot-internal unrelated pair a
//! verifier-valid negative (javac 23.0.1 `--release 8 -g:none`, SHA-256 in the evidence README).
//!
//! * the **text**: every recovered argument spells the required type (`(I1$Greet) new I1$En()`),
//!   so the recompiled source cannot retarget the call to another overload;
//! * the **boundary**: a chain broken inside the snapshot and a platform target keep their
//!   refusal verbatim, the array/Object/same-name/platform answers run unchanged before this
//!   proof, and the `final` class to `Object` keeps the existing Object answer;
//! * the **runtime**: the frozen originals and the recovered families print the same lines under
//!   `java -Xverify:all`, the `I1` family's fourth path (`hello:v`) included.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FAM_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/user-hierarchy-widening-patrol/fixture/fam.jar"
);
const HIER_FAM_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/user-hierarchy-widening-patrol/fixture/hier/hier-fam.jar"
);
const HIER_NEG_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/user-hierarchy-widening-patrol/fixture/hier/hier-neg.jar"
);
const HIER_DEPTH_JAR: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/user-hierarchy-widening-patrol/fixture/hier/hier-depth.jar"
);
const HELPER_CLASS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-10-02/user-hierarchy-widening-patrol/fixture/hier/Helper.class"
);

/// The one run of the original `I1.main` under `java -Xverify:all` (`fixture/orig.out`).
const I1_EXPECTED_RUN: &str = "hi:d\nhello:d\nstatic\nhello:v\n";

/// The one run of the original `H1.main`: one labelled line per variant position.
const H1_EXPECTED_RUN: &str = "\
hi:two
hi:multi
tag:mtag
hi:anon
hi:sub
lead:hi:x
call:mid
obj
";

/// The one run of the original `H2.main`; the negative family's own runtime is valid even where
/// its recovery must stay refused.
const H2_EXPECTED_RUN: &str = "t:helper\nsinkT:m\n";

/// The one run of the original `H3.main`: both positions are true widenings, the eighth edge and
/// the ninth; only the first is inside the walk's bound.
const H3_EXPECTED_RUN: &str = "d:l0\nd:l0\n";

/// Every class the two jars hold: the recovered presentation is per class, and a family
/// recompiles as the set of its binary names.
const I1_FAMILY: [&str; 5] = ["I1", "I1$1", "I1$En", "I1$Greet", "I1$Greet$1"];
const H1_FAMILY: [&str; 11] = [
    "H1",
    "H1$1",
    "H1$Caller",
    "H1$Fin",
    "H1$Greet",
    "H1$Mid",
    "H1$Multi",
    "H1$Other",
    "H1$Sub",
    "H1$TwoLevel",
    "H1$ViaSub",
];

fn opened(jar: &[u8], class: &str) -> (artifact::ArtifactSnapshot, ClassSourceRequest) {
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(jar.to_vec()), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    (snapshot, request)
}

fn recover(jar: &[u8], class: &str) -> ClassSourceReport {
    let mut budget = task_budget(&[]).unwrap();
    let (snapshot, request) = opened(jar, class);
    match Engine::new().class_source_with_evidence(
        slice::from_ref(&snapshot),
        &request,
        &RecoveryEvidenceRequest::essential(),
        &mut budget,
    ) {
        Ok(OperationOutcome::Performed(report)) => report,
        Ok(other) => panic!("complete class source expected: {other:?}"),
        Err(error) => panic!("a legal class-source request is answered: {error}"),
    }
}

/// The recovered presentation of one whole family, each class in its own file, compiled together
/// under the binary names the presentations spell.
fn recovered_family(dir: &Path, jar: &[u8], family: &[&str]) {
    for class in family {
        let report = recover(jar, class);
        fs::write(dir.join(format!("{class}.java")), &report.text).unwrap();
    }
}

fn scratch(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "jarde-p3-snapshot-hierarchy-{name}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create comparison directory");
    root
}

fn compile(dir: &Path) {
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-Xlint:-options")
        .arg("-cp")
        .arg(dir)
        .arg("-d")
        .arg(dir)
        .args(
            fs::read_dir(dir)
                .unwrap()
                .filter_map(|entry| {
                    let path = entry.unwrap().path();
                    (path
                        .extension()
                        .is_some_and(|extension| extension == "java"))
                    .then_some(path)
                })
                .collect::<Vec<_>>(),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run(classpath: &str, main_class: &str) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(classpath)
        .arg(main_class)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn i1_family_recovers_the_interface_argument_with_the_required_spelling() {
    let report = recover(FAM_JAR, "I1");
    let text = &report.text;
    assert!(
        !text.contains("no safe reference conversion evidence"),
        "the interface argument refusal is closed:\n{text}"
    );
    assert!(
        !text.contains("@bytecode"),
        "the whole class presents:\n{text}"
    );
    assert!(
        text.contains("viaInterface((I1$Greet) new I1$En(), \"v\")"),
        "main keeps the pool's selected parameter type:\n{text}"
    );
}

#[test]
fn i1_family_recompiles_and_runs_every_path() {
    let original_dir = scratch("i1-original");
    let jar_path = original_dir.join("fam.jar");
    fs::write(&jar_path, FAM_JAR).unwrap();
    let original_run = run(jar_path.to_str().unwrap(), "I1");

    let recovered_dir = scratch("i1-recovered");
    recovered_family(&recovered_dir, FAM_JAR, &I1_FAMILY);
    compile(&recovered_dir);
    let recovered_run = run(recovered_dir.to_str().unwrap(), "I1");

    assert_eq!(original_run, I1_EXPECTED_RUN, "{original_run}");
    assert_eq!(recovered_run, original_run, "the recovered family diverges");
}

#[test]
fn hier_variants_each_cast_with_the_required_spelling() {
    let report = recover(HIER_FAM_JAR, "H1");
    let text = &report.text;
    assert!(
        !text.contains("no safe reference conversion evidence"),
        "no variant may refuse:\n{text}"
    );
    assert!(
        !text.contains("@bytecode"),
        "the whole class presents:\n{text}"
    );
    // The two-level class chain, the multi-implementation interface list (both directions), the
    // anonymous implementation, the super-interface hop, a widening past a leading `String`
    // parameter and an instance call's receiver-offset position.
    for statement in [
        "via((H1$Greet) new H1$TwoLevel(), \"two\")",
        "via((H1$Greet) new H1$Multi(), \"multi\")",
        "viaOther((H1$Other) new H1$Multi())",
        "via((H1$Greet) new H1$1(), \"anon\")",
        "via((H1$Greet) new H1$ViaSub(), \"sub\")",
        "lead(\"lead\", (H1$Greet) new H1$Multi())",
        "new H1$Caller().call((H1$Greet) new H1$TwoLevel())",
    ] {
        assert!(
            text.contains(statement),
            "main misses `{statement}`:\n{text}"
        );
    }
    // The `final` class to `Object` keeps the dispatch's unconditional Object answer, exactly as
    // before this slice: no snapshot walk decides it.
    assert!(
        text.contains("viaObject((java.lang.Object) new H1$Fin())"),
        "{text}"
    );
}

#[test]
fn hier_variants_recompile_and_run_every_path() {
    let original_dir = scratch("hier-original");
    let jar_path = original_dir.join("hier-fam.jar");
    fs::write(&jar_path, HIER_FAM_JAR).unwrap();
    let original_run = run(jar_path.to_str().unwrap(), "H1");

    let recovered_dir = scratch("hier-recovered");
    recovered_family(&recovered_dir, HIER_FAM_JAR, &H1_FAMILY);
    compile(&recovered_dir);
    let recovered_run = run(recovered_dir.to_str().unwrap(), "H1");

    assert_eq!(original_run, H1_EXPECTED_RUN, "{original_run}");
    assert_eq!(recovered_run, original_run, "the recovered family diverges");
}

#[test]
fn negatives_keep_the_refusal_and_the_original_runtime() {
    let report = recover(HIER_NEG_JAR, "H2");
    let text = &report.text;
    assert_eq!(
        text.matches("no safe reference conversion evidence")
            .count(),
        2,
        "the two table-out calls stay refused:\n{text}"
    );
    // The snapshot-internal unrelated pair: `H2$Ext` and `H2$Target` are both physical classes of
    // this snapshot, and the only chain between them runs through `Helper`, which the snapshot
    // does not hold. The refusal is this slice's boundary, not a missing table row.
    assert!(
        text.contains(
            "presents `H2$Ext` but the invocation requires `H2$Target` and this layer has no safe reference conversion evidence"
        ),
        "{text}"
    );
    // The single-sided platform target: the user exception class is in the snapshot, the
    // `java.lang.Throwable` requirement is not, and no classpath is consulted.
    assert!(
        text.contains(
            "presents `H2$MyErr` but the invocation requires `java.lang.Throwable` and this layer has no safe reference conversion evidence"
        ),
        "{text}"
    );

    // The negative family's own runtime is valid: `Helper` on the classpath makes the original
    // verify and print under `-Xverify:all`, exactly as it always has.
    let original_dir = scratch("hier-neg-original");
    let jar_path = original_dir.join("hier-neg.jar");
    fs::write(&jar_path, HIER_NEG_JAR).unwrap();
    fs::write(original_dir.join("Helper.class"), HELPER_CLASS).unwrap();
    let original_run = run(
        &format!(
            "{}:{}",
            jar_path.to_str().unwrap(),
            original_dir.to_str().unwrap()
        ),
        "H2",
    );
    assert_eq!(original_run, H2_EXPECTED_RUN, "{original_run}");
}

#[test]
fn the_walk_proves_the_eighth_edge_and_refuses_the_ninth() {
    let report = recover(HIER_DEPTH_JAR, "H3");
    let text = &report.text;
    // `L7` reaches `Sig` in eight chain edges (`L7`..`L0` plus `Sig`): the last proved depth.
    assert!(
        text.contains("via((H3$Sig) new H3$L7())"),
        "the cap's own edge recovers:\n{text}"
    );
    // `L8` needs a ninth edge: beyond the bound, the position keeps its refusal exactly as it
    // stood before this slice — a true widening the walk declines on depth, not a false pair.
    assert_eq!(
        text.matches("no safe reference conversion evidence")
            .count(),
        1,
        "only the ninth edge stays refused:\n{text}"
    );
    assert!(
        text.contains(
            "presents `H3$L8` but the invocation requires `H3$Sig` and this layer has no safe reference conversion evidence"
        ),
        "{text}"
    );

    let original_dir = scratch("hier-depth-original");
    let jar_path = original_dir.join("hier-depth.jar");
    fs::write(&jar_path, HIER_DEPTH_JAR).unwrap();
    let original_run = run(jar_path.to_str().unwrap(), "H3");
    assert_eq!(original_run, H3_EXPECTED_RUN, "{original_run}");
}
