//! Acceptance tests of change `recover-inner-class-static-mixed-folding`: a family that mixes
//! direct static member rows with one non-static candidate folds its static subset into the
//! enclosing class's own source unit, while the non-static member keeps the separated
//! presentation it always had — neither member blocks the other.
//!
//! The anchors the proposal fixes:
//!
//! 1. the patrol's frozen N1 family: `static class Stat` folds with source-spelled references
//!    (`new Stat()`), every reference to the non-folded `Inner` keeps the pool spelling, and the
//!    one body the recovery layer never proved (`use`'s qualified `outer.new Inner(9)`) is
//!    carried verbatim — the registered slice-two gap, byte-identical to the pre-change
//!    separated unit's;
//! 2. the three frozen variants — two statics beside one non-static, one beside one, and the
//!    pure non-static negative — recompile with their separated sibling units under
//!    `javac --release 8`, verify under `-Xverify:all`, and print exactly what the original
//!    class files print;
//! 3. the boundaries: a pure non-static family never folds, and budget or cancellation stops
//!    the whole mixed fold atomically.

use jarde::class_source::{
    ClassSourceMemberFamily, ClassSourceMemberProjection, ClassSourceReport,
};
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-mixed-fold-{label}-{}-{nonce}-{}",
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

fn jar_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in entries {
            let (mut entry, config) = zip
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(bytes).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        zip.finish().unwrap();
    }
    output.into_inner()
}

/// One family compiled from its source exactly the way the patrol's fixtures were: `--release 8`
/// with `-g:none`, one jar entry per produced class file.
fn compile_family(label: &str, source: &str) -> (Vec<u8>, Vec<(String, Vec<u8>)>) {
    let temp = TestDirectory::new(label);
    let file = source
        .split_once("class ")
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .map(|name| format!("{name}.java"))
        .expect("the source states a class");
    std::fs::write(temp.path().join(&file), source).expect("the family source is written");
    let compiled = Command::new("javac")
        .args(["--release", "8", "-g:none"])
        .arg(&file)
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let mut owned = Vec::new();
    let mut directories = vec![temp.path().to_owned()];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(&directory).expect("the directory reads") {
            let entry = entry.expect("the directory entry reads");
            if entry.file_type().unwrap().is_dir() {
                directories.push(entry.path());
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("class") {
                continue;
            }
            let name = path
                .strip_prefix(temp.path())
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let bytes = std::fs::read(&path).expect("the class file reads");
            owned.push((name, bytes));
        }
    }
    let entries = owned
        .iter()
        .map(|(name, bytes)| (name.as_bytes() as &[u8], bytes.as_slice()))
        .collect::<Vec<_>>();
    (jar_of(&entries), owned)
}

/// The class-source report of one named class, under the source-map evidence every fold's
/// provenance anchors need.
fn source_of(jar: &[u8], class: &str) -> ClassSourceReport {
    source_with(jar, class, task_limits())
}

fn task_limits() -> Limits {
    jarde::facade::task_limits(&[]).expect("the task defaults are bounded")
}

fn source_with(jar: &[u8], class: &str, limits: Limits) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = Budget::new(limits);
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.to_vec()), &mut budget)
        .expect("the family jar opens");
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
    let evidence = RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap);
    match engine.class_source_with_evidence(
        std::slice::from_ref(&snapshot),
        &request,
        &evidence,
        &mut budget,
    ) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("one family answers one class-source request: {other:?}"),
    }
}

/// Compile one family's recovered units as one `--release 8` set — the folded root beside the
/// separated sibling units a mixed family still states (a pool-spelled `X$Inner` reference
/// resolves against the sibling unit's own top-level declaration, never the binary classpath) —
/// then run the recompiled set itself under `-Xverify:all` and return what the run printed: the
/// runtime classpath puts the recompiled classes first, so the run executes the recovered text's
/// own binary and not the original class the compile used for symbol resolution.
fn recompile_family_set_and_run(
    label: &str,
    units: &[(&str, &str)],
    main_class: &str,
    jar: &[u8],
) -> String {
    let temp = TestDirectory::new(label);
    let mut files = Vec::new();
    for (name, text) in units {
        let file = temp.path().join(format!("{name}.java"));
        std::fs::write(&file, text).expect("the recovered unit is written");
        files.push(file);
    }
    let deps = temp.path().join("deps");
    std::fs::create_dir_all(&deps).expect("the dependency directory is created");
    std::fs::write(deps.join("family.jar"), jar).expect("the family jar is written");
    let compiled = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-cp")
        .arg(deps.join("family.jar"))
        .args(&files)
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    if !compiled.status.success() {
        // Show the errors beside the offending units before failing the assertion.
        for (name, text) in units {
            eprintln!("--- {name}.java ---\n{text}");
        }
    }
    assert!(
        compiled.status.success(),
        "the family set {} recompiles under --release 8:\n{}",
        main_class,
        String::from_utf8_lossy(&compiled.stderr)
    );
    let run = Command::new("java")
        .args([
            "-Xverify:all",
            "-cp",
            &format!(".:{}", deps.join("family.jar").display()),
        ])
        .arg(main_class)
        .current_dir(temp.path())
        .output()
        .expect("java runs");
    assert!(
        run.status.success(),
        "the recompiled {} verifies and runs:\n{}",
        main_class,
        String::from_utf8_lossy(&run.stderr)
    );
    String::from_utf8(run.stdout).expect("the run prints text")
}

/// The patrol's frozen N1 family, replayed from its committed class bytes (SHA-verified in the
/// evidence directory): the static `Stat` beside the non-static `Inner` that captures `base`.
fn n1_jar() -> Vec<u8> {
    jar_of(&[
        (
            b"N1.class",
            include_bytes!(
                "../openspec/evidence/java-syntax-2026-10-03/inner-class-folding-patrol/fixture/N1.class"
            ),
        ),
        (
            b"N1$Inner.class",
            include_bytes!(
                "../openspec/evidence/java-syntax-2026-10-03/inner-class-folding-patrol/fixture/N1$Inner.class"
            ),
        ),
        (
            b"N1$Stat.class",
            include_bytes!(
                "../openspec/evidence/java-syntax-2026-10-03/inner-class-folding-patrol/fixture/N1$Stat.class"
            ),
        ),
    ])
}

const MV3_SOURCE: &str = r#"public class MV3 {
    private int base = 4;
    static class StatA { int a() { return 5; } }
    static class StatB extends StatA { int b() { return a() + 1; } }
    class Inner {
        int tag;
        Inner(int t) { this.tag = t; }
        int total() { return tag + base; }
    }
    Inner make(int t) { return new Inner(t); }
    public static void main(String[] args) {
        System.out.println(new StatB().b());
        System.out.println(new MV3().make(2).total());
    }
}
"#;

const MV2_SOURCE: &str = r#"public class MV2 {
    static class Stat { int m() { return 11; } }
    class Inner {
        int tag;
        Inner(int t) { this.tag = t; }
        int total() { return tag * 2; }
    }
    public static void main(String[] args) {
        System.out.println(new Stat().m());
        System.out.println(new MV2().new Inner(4).total());
    }
}
"#;

const MV1_SOURCE: &str = r#"public class MV1 {
    class Inner { int v() { return 9; } }
    int run() { return new Inner().v(); }
    public static void main(String[] args) { System.out.println(new MV1().run()); }
}
"#;

#[test]
fn n1_folds_its_static_stat_beside_the_separated_inner() {
    let jar = n1_jar();
    let report = source_of(&jar, "N1");
    // The static subset folds: one nested declaration in the root's own source unit, and every
    // reference to it inside the fold scope states the source spelling.
    assert!(
        report
            .text
            .contains("static class Stat extends java.lang.Object {"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("new Stat().use(new N1())"),
        "{}",
        report.text
    );
    assert!(report.text.contains("int use(N1 arg1)"), "{}", report.text);
    // No code line keeps the folded member's pool spelling; the marker comments keep the
    // physical descriptors by design.
    for line in report.text.lines() {
        if !line.trim_start().starts_with("//") {
            assert!(!line.contains("N1$Stat"), "{line}");
        }
    }
    // The non-static member never enters the fold: its references keep the pool spelling the
    // separated presentation always stated, and no nested Inner declaration appears.
    assert!(
        report.text.contains("N1$Inner make(int arg1)"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("return new N1$Inner(this, arg1);"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("new N1$Inner(new N1(), 3).total()"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("class Inner"), "{}", report.text);
    // The one body the recovery layer never proved — `use`'s qualified `outer.new Inner(9)` —
    // is carried verbatim into the nested declaration: the registered slice-two gap, exactly
    // the pre-change separated unit's presentation of the same member.
    assert!(
        report
            .text
            .contains("not recovered: the recovery run for `use(LN1;)I`"),
        "{}",
        report.text
    );
    let ClassSourceMemberFamily::PreparedStatic {
        members,
        projection: ClassSourceMemberProjection::Projected { derived },
    } = &report.member_family
    else {
        panic!(
            "the mixed family folds its static subset: {:?}",
            report.member_family
        )
    };
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].relation.simple_name, "Stat");
    assert_eq!(members[0].relation.access_flags, 0x0008);
    assert!(
        derived.iter().any(|entry| entry.kind
            == jarde::class_source::MemberFamilyDerivedKind::MemberClassDeclaration)
    );
    assert!(
        derived.iter().any(|entry| entry.kind
            == jarde::class_source::MemberFamilyDerivedKind::MemberConstructorName)
    );
    // The separated units keep their own flat presentations: the fold neither embeds nor
    // re-spells them.
    let inner = source_of(&jar, "N1$Inner");
    assert!(
        inner
            .text
            .starts_with("// jarde: presentation of `N1$Inner`"),
        "{}",
        inner.text
    );
    assert!(
        inner
            .text
            .contains("class N1$Inner extends java.lang.Object {"),
        "{}",
        inner.text
    );
    let stat = source_of(&jar, "N1$Stat");
    assert!(
        stat.text.starts_with("// jarde: presentation of `N1$Stat`"),
        "{}",
        stat.text
    );
    assert!(
        stat.text
            .contains("class N1$Stat extends java.lang.Object {"),
        "{}",
        stat.text
    );
}

#[test]
fn mixed_variants_recompile_as_family_sets_and_reproduce_the_baseline() {
    // Two static members beside one non-static candidate: both static rows fold — including the
    // sibling inheritance between them — while the Inner unit stays separated and the family
    // set reproduces the original run.
    let (jar, _) = compile_family("mv3", MV3_SOURCE);
    let report = source_of(&jar, "MV3");
    assert!(
        report
            .text
            .contains("static class StatA extends java.lang.Object {"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("static class StatB extends StatA {"),
        "{}",
        report.text
    );
    assert!(report.text.contains("new StatB().b()"), "{}", report.text);
    assert!(
        report.text.contains("return new MV3$Inner(this, arg1);"),
        "{}",
        report.text
    );
    assert!(matches!(
        &report.member_family,
        ClassSourceMemberFamily::PreparedStatic {
            members,
            projection: ClassSourceMemberProjection::Projected { .. },
        } if members.len() == 2
    ));
    let inner = source_of(&jar, "MV3$Inner");
    assert!(
        inner
            .text
            .contains("class MV3$Inner extends java.lang.Object {"),
        "{}",
        inner.text
    );
    assert_eq!(
        recompile_family_set_and_run(
            "mv3",
            &[("MV3", &report.text), ("Inner_unit", &inner.text)],
            "MV3",
            &jar,
        ),
        "6\n6\n"
    );

    // One static member beside one non-static candidate — the N1 shape with fully recovered
    // bodies, so the whole family set recompiles, verifies and runs.
    let (jar, _) = compile_family("mv2", MV2_SOURCE);
    let report = source_of(&jar, "MV2");
    assert!(
        report
            .text
            .contains("static class Stat extends java.lang.Object {"),
        "{}",
        report.text
    );
    assert!(report.text.contains("new Stat().m()"), "{}", report.text);
    assert!(
        report.text.contains("new MV2$Inner(new MV2(), 4).total()"),
        "{}",
        report.text
    );
    assert!(matches!(
        &report.member_family,
        ClassSourceMemberFamily::PreparedStatic {
            members,
            projection: ClassSourceMemberProjection::Projected { .. },
        } if members.len() == 1
    ));
    let inner = source_of(&jar, "MV2$Inner");
    assert_eq!(
        recompile_family_set_and_run(
            "mv2",
            &[("MV2", &report.text), ("Inner_unit", &inner.text)],
            "MV2",
            &jar,
        ),
        "11\n8\n"
    );
}

#[test]
fn pure_non_static_family_never_folds() {
    let (jar, _) = compile_family("mv1", MV1_SOURCE);
    let report = source_of(&jar, "MV1");
    // A family without a static row has no subset to fold: the root keeps the separated
    // presentation and the capture channel's own family state, exactly as before this change.
    assert!(!report.text.contains("static class"), "{}", report.text);
    assert!(!matches!(
        &report.member_family,
        ClassSourceMemberFamily::PreparedStatic { .. }
    ));
    let inner = source_of(&jar, "MV1$Inner");
    assert!(
        inner
            .text
            .contains("class MV1$Inner extends java.lang.Object {"),
        "{}",
        inner.text
    );
    assert_eq!(
        recompile_family_set_and_run(
            "mv1",
            &[("MV1", &report.text), ("Inner_unit", &inner.text)],
            "MV1",
            &jar,
        ),
        "9\n"
    );
}

#[test]
fn budget_and_cancellation_stop_the_mixed_fold_atomically() {
    let (jar, _) = compile_family("mv2-budget", MV2_SOURCE);
    let full = source_of(&jar, "MV2");
    let mut low = task_limits();
    low.output_bytes = full.usage.output_bytes.saturating_sub(1);
    let stopped = source_with(&jar, "MV2", low);
    assert!(
        !stopped.text.contains("static class Stat"),
        "{}",
        stopped.text
    );
    let cancellation = jarde::CancellationToken::new();
    let engine = Engine::new();
    let mut budget = Budget::with_cancellation_token(task_limits(), cancellation.clone());
    let snapshot = engine
        .open(ArtifactInput::bytes(jar.clone()), &mut budget)
        .expect("the family jar opens");
    cancellation.cancel();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("MV2"),
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
    let evidence = RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap);
    match engine.class_source_with_evidence(
        std::slice::from_ref(&snapshot),
        &request,
        &evidence,
        &mut budget,
    ) {
        Ok(OperationOutcome::Performed(report)) => {
            assert!(
                !report.text.contains("static class Stat"),
                "{}",
                report.text
            );
            assert!(!matches!(
                &report.member_family,
                ClassSourceMemberFamily::PreparedStatic {
                    projection: ClassSourceMemberProjection::Projected { .. },
                    ..
                }
            ));
        }
        // A run cancelled before its first member is an `Incomplete` selection: no class was
        // prepared, so no fold was published either — the all-or-nothing answer either way.
        Ok(OperationOutcome::Incomplete(_)) => {}
        other => panic!("a cancelled run still answers: {other:?}"),
    }
}
