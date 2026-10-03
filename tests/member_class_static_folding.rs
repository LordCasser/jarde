//! Acceptance tests of change `recover-member-class-static-folding`: the direct static
//! member family — any number of classes and interfaces — folds into the enclosing class's
//! own source unit, and every reference to a folded member inside that unit is re-spelled with
//! the source nesting the declaration states.
//!
//! The three-way anchors the proposal fixes:
//!
//! 1. the whole folded families recompile under `javac --release 8`, verify under
//!    `-Xverify:all`, and print exactly what the original class files print (`hi`/`ok`, `7`);
//! 2. the separated presentations — every child class, the cross-class presenter, a jar that
//!    does not carry a child definition — keep the flattened `$`-named units they had before;
//! 3. the boundary the slice registers: grandchildren stay outside the fold (a reference to one
//!    inside the fold scope keeps the pool spelling), and a non-static member child never
//!    enters it.

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
            "jarde-static-fold-{label}-{}-{nonce}-{}",
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
/// with `-g:none`, one jar entry per produced class file, in the writer's own order.
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

/// The class-source report of one named class, under the source-map evidence every family fold's
/// provenance anchors need (the same selection the nested-enum fold established).
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

/// Compiles one recovered text as `--release 8` Java, runs it under `-Xverify:all`, and returns
/// what the run printed. The classpath holds exactly the original compiled family, never a
/// rewritten text — the same convention the nested-enum and nested-spelling acceptance uses.
fn recompile_and_run(label: &str, text: &str, class_name: &str, jar: &[u8]) -> String {
    let temp = TestDirectory::new(label);
    std::fs::write(temp.path().join(format!("{class_name}.java")), text)
        .expect("the recovered text is written");
    let deps = temp.path().join("deps");
    std::fs::create_dir_all(&deps).expect("the dependency directory is created");
    std::fs::write(deps.join("family.jar"), jar).expect("the family jar is written");
    let compiled = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-cp")
        .arg(deps.join("family.jar"))
        .arg(format!("{class_name}.java"))
        .current_dir(temp.path())
        .output()
        .expect("javac runs");
    assert!(
        compiled.status.success(),
        "the folded {} recompiles under --release 8:\n{}",
        class_name,
        String::from_utf8_lossy(&compiled.stderr)
    );
    let run = Command::new("java")
        .args([
            "-Xverify:all",
            "-cp",
            &format!("{}:.", deps.join("family.jar").display()),
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

const M1_SOURCE: &str = r#"public class M1 {
    static class Base { void hi() { System.out.println("hi"); } }
    interface Ctrl { }
    static class Deep extends Base implements Ctrl { }
    static class Err extends Exception { }
    static class Inner { static class Leaf extends Base { } }
    static Deep deepField;
    Base baseField;
    void work() throws Err, RuntimeException { }
    static Deep make() throws Err { return new Deep(); }
    public static void main(String[] a) throws Err {
        M1 m = new M1();
        m.baseField = new Deep();
        m.baseField.hi();
        m.work();
        System.out.println("ok");
    }
}
class M1User extends M1.Base implements M1.Ctrl {
    public static void run() { new M1User().hi(); }
}
"#;

const M2_SOURCE: &str = "public class M2 {\n    static class Solo { int v() { return 7; } }\n    public static void main(String[] a) { System.out.println(new Solo().v()); }\n}\n";

#[test]
fn multi_child_family_folds_five_declarations_and_reproduces_the_baseline() {
    let (jar, _) = compile_family("m1", M1_SOURCE);
    let report = source_of(&jar, "M1");
    // The five direct static members are nested declarations of one source unit, in the root's
    // own InnerClasses order, and the grandchild stays out (registered boundary).
    for declaration in [
        "static class Deep extends Base implements Ctrl {",
        "static class Base extends java.lang.Object {",
        "static class Inner extends java.lang.Object {",
        "static class Err extends java.lang.Exception {",
        "static interface Ctrl {",
    ] {
        assert!(report.text.contains(declaration), "{}", report.text);
    }
    assert!(!report.text.contains("class Leaf"), "{}", report.text);
    // Every reference position inside the fold scope takes the source spelling: fields, throws,
    // declarations, and bodies.
    assert!(
        report.text.contains("static Deep deepField;"),
        "{}",
        report.text
    );
    assert!(report.text.contains("Base baseField;"), "{}", report.text);
    assert!(
        report
            .text
            .contains("void work() throws Err, java.lang.RuntimeException"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("static Deep make() throws Err {"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("return new Deep();"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains(".baseField = new Deep();"),
        "{}",
        report.text
    );
    // No code line keeps the pool `$` spelling: the markers retain physical descriptors by
    // design, and every source-syntax position inside the fold scope states the nesting.
    for line in report.text.lines() {
        if !line.trim_start().starts_with("//") {
            assert!(!line.contains("M1$"), "{line}");
        }
    }
    let ClassSourceMemberFamily::PreparedStatic {
        members,
        projection: ClassSourceMemberProjection::Projected { derived },
    } = &report.member_family
    else {
        panic!("the five-row family folds: {:?}", report.member_family)
    };
    assert_eq!(members.len(), 5);
    // Each child keeps its own physical report beside the fold: the nested declaration is a
    // presentation of the root, never a mutation of a child.
    for member in members {
        assert!(member.child.text.contains("M1$"), "{}", member.child.text);
    }
    assert!(
        derived.iter().any(|entry| entry.kind
            == jarde::class_source::MemberFamilyDerivedKind::MemberClassDeclaration)
    );
    assert!(
        derived.iter().any(|entry| entry.kind
            == jarde::class_source::MemberFamilyDerivedKind::MemberConstructorName)
    );
    assert_eq!(
        recompile_and_run("m1", &report.text, "M1", &jar),
        "hi\nok\n"
    );
}

#[test]
fn single_child_family_folds_when_the_narrow_certificate_refuses() {
    let (jar, _) = compile_family("m2", M2_SOURCE);
    let report = source_of(&jar, "M2");
    assert!(report.text.contains("static class Solo"), "{}", report.text);
    assert!(report.text.contains("new Solo().v()"), "{}", report.text);
    assert!(report.text.contains("int v()"), "{}", report.text);
    assert!(matches!(
        &report.member_family,
        ClassSourceMemberFamily::PreparedStatic {
            members,
            projection: ClassSourceMemberProjection::Projected { .. },
        } if members.len() == 1
    ));
    assert_eq!(recompile_and_run("m2", &report.text, "M2", &jar), "7\n");
}

#[test]
fn interface_child_and_sibling_inheritance_fold_with_source_spellings() {
    let interface_family = "class VIface {\n    interface Mark { int TAG = 5; }\n    static class Tagged implements Mark { int tag() { return TAG; } }\n    public static void main(String[] a) { System.out.println(new Tagged().tag()); }\n}\n";
    let (jar, _) = compile_family("iface", interface_family);
    let report = source_of(&jar, "VIface");
    assert!(
        report
            .text
            .contains("static class Tagged extends java.lang.Object implements Mark {"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("static interface Mark {"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("new Tagged().tag()"),
        "{}",
        report.text
    );
    assert_eq!(
        recompile_and_run("iface", &report.text, "VIface", &jar),
        "5\n"
    );

    let sibling_family = "class VSib {\n    static class Base { int id() { return 3; } }\n    static class Err extends Exception { }\n    static class Kid extends Base { }\n    static Kid kid;\n    static Base make() throws Err { return new Kid(); }\n    public static void main(String[] a) throws Err { System.out.println(make().id()); }\n}\n";
    let (jar, _) = compile_family("sib", sibling_family);
    let report = source_of(&jar, "VSib");
    assert!(
        report.text.contains("static class Kid extends Base {"),
        "{}",
        report.text
    );
    assert!(report.text.contains("static Kid kid;"), "{}", report.text);
    assert!(
        report.text.contains("static Base make() throws Err {"),
        "{}",
        report.text
    );
    assert!(report.text.contains("return new Kid();"), "{}", report.text);
    assert_eq!(recompile_and_run("sib", &report.text, "VSib", &jar), "3\n");
}

#[test]
fn grandchild_stays_outside_the_fold_and_keeps_its_own_presentation() {
    let family = "class VGrand {\n    static class Base { int b() { return 6; } }\n    static class Inner { static class Leaf extends Base { } }\n    public static void main(String[] a) { System.out.println(new Base().b()); }\n}\n";
    let (jar, _) = compile_family("grand", family);
    let report = source_of(&jar, "VGrand");
    assert!(
        report
            .text
            .contains("static class Base extends java.lang.Object {"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("static class Inner extends java.lang.Object {"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("Leaf"), "{}", report.text);
    // The grandchild's own single-class presentation keeps the flattened pool spelling it
    // always had — the fold neither embeds nor re-spells it.
    let leaf = source_of(&jar, "VGrand$Inner$Leaf");
    assert!(
        leaf.text
            .starts_with("// jarde: presentation of `VGrand$Inner$Leaf`"),
        "{}",
        leaf.text
    );
    assert!(
        leaf.text
            .contains("class VGrand$Inner$Leaf extends VGrand$Base {"),
        "{}",
        leaf.text
    );
    assert_eq!(
        recompile_and_run("grand", &report.text, "VGrand", &jar),
        "6\n"
    );
}

#[test]
fn non_static_member_child_never_enters_the_static_fold() {
    let family = "class VInner {\n    class Member { int v() { return 4; } }\n    int run() { return new Member().v(); }\n    public static void main(String[] a) { System.out.println(new VInner().run()); }\n}\n";
    let (jar, _) = compile_family("inner", family);
    let report = source_of(&jar, "VInner");
    // A non-static member child belongs to the capture channel's own family; the static fold
    // does not admit it and the root keeps its separated presentation.
    assert!(!report.text.contains("class Member {"), "{}", report.text);
    assert!(!matches!(
        &report.member_family,
        ClassSourceMemberFamily::PreparedStatic { .. }
    ));
    let member = source_of(&jar, "VInner$Member");
    assert!(
        member.text.contains("class VInner$Member"),
        "{}",
        member.text
    );
}

#[test]
fn cross_class_presenter_and_missing_child_keep_the_flattened_presentations() {
    let (jar, _) = compile_family("user", M1_SOURCE);
    // The cross-class presenter is a top-level unit of its own: its header keeps the pool
    // spelling, byte for byte the pre-change presentation.
    let user = source_of(&jar, "M1User");
    assert!(
        user.text
            .contains("class M1User extends M1$Base implements M1$Ctrl {"),
        "{}",
        user.text
    );
    assert!(!matches!(
        &user.member_family,
        ClassSourceMemberFamily::PreparedStatic { .. }
    ));
    // A jar without the child definitions folds nothing: no definition, no fold — the root's
    // separated text stays exactly what it was.
    let (_, owned) = compile_family("missing", M1_SOURCE);
    let root_only = owned
        .iter()
        .find(|(name, _)| name == "M1.class")
        .map(|(_, bytes)| bytes.as_slice())
        .expect("the root class file is compiled");
    let missing = source_of(&jar_of(&[(b"M1.class", root_only)]), "M1");
    assert!(!missing.text.contains("static class"), "{}", missing.text);
    assert!(matches!(
        &missing.member_family,
        ClassSourceMemberFamily::RefusedPair { .. }
    ));
}

/// Change `recover-single-static-interface-fold`: the token anchor's direct-coverage matcher reads
/// an interface method reference's owner exactly as it already read a method or field reference's
/// owner, so a family whose only external use is a static interface call folds — and the negative
/// whose covering segment names the class nowhere at all still refuses.
#[test]
fn interface_call_owner_anchors_the_fold_and_an_unnamed_segment_still_refuses() {
    // `M.sv()` compiles to `invokestatic` on an `InterfaceMethodRef` whose owner is `WCallI$M`:
    // before this change the anchor matcher read that owner position as nothing, so the token
    // `WCallI$M` had no proved class reference and the whole fold was refused. Its class-child
    // twin `WCallC` folds on the identical shape through an ordinary `MethodRef` owner.
    let interface_family = "public class WCallI {\n    interface M { static int sv() { return 8; } int v(); }\n    public static void main(String[] a) { System.out.println(M.sv()); }\n}\n";
    let (jar, _) = compile_family("wcalli", interface_family);
    let report = source_of(&jar, "WCallI");
    assert!(
        report.text.contains("static interface M {"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("System.out.println(M.sv());"),
        "{}",
        report.text
    );
    // No code line keeps the pool `$` spelling: the markers retain physical descriptors by
    // design, and every source-syntax position inside the fold scope states the nesting.
    for line in report.text.lines() {
        if !line.trim_start().starts_with("//") {
            assert!(!line.contains("WCallI$"), "{line}");
        }
    }
    assert!(matches!(
        &report.member_family,
        ClassSourceMemberFamily::PreparedStatic {
            projection: ClassSourceMemberProjection::Projected { .. },
            ..
        }
    ));
    assert_eq!(
        recompile_and_run("wcalli", &report.text, "WCallI", &jar),
        "8\n"
    );

    // The negative: a family member's type is introduced through a local variable holding a
    // value this run never produced, so the covering segment names no class at all — not through
    // any owner, a `Class` entry or a producer. The direct-coverage matcher's own requirement is
    // unchanged by this change, so the fold still refuses.
    let unnamed = "public class NAnchor {\n    interface M { static int sv() { return 8; } }\n    static M pick(M candidate) { M local = candidate; return local; }\n    public static void main(String[] a) { System.out.println(M.sv() + (pick(null) == null ? 1 : 0)); }\n}\n";
    let (jar, _) = compile_family("nanchor", unnamed);
    let negative = source_of(&jar, "NAnchor");
    assert!(
        !negative.text.contains("static interface M"),
        "{}",
        negative.text
    );
    match &negative.member_family {
        ClassSourceMemberFamily::Prepared {
            projection: ClassSourceMemberProjection::Refused { reason },
            ..
        } => assert_eq!(reason, "capture proof is incomplete", "{}", negative.text),
        other => panic!("the unanchored family keeps its refusal: {other:?}"),
    }
}

#[test]
fn budget_and_cancellation_stop_the_whole_fold_atomically() {
    let (jar, _) = compile_family("budget", M2_SOURCE);
    let full = source_of(&jar, "M2");
    let mut low = task_limits();
    low.output_bytes = full.usage.output_bytes.saturating_sub(1);
    let stopped = source_with(&jar, "M2", low);
    assert!(
        !stopped.text.contains("static class Solo"),
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
            class: ClassNameQuery::internal("M2"),
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
                !report.text.contains("static class Solo"),
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
