//! Acceptance tests of change `recover-fold-context-projection-preservation`: a root whose
//! first-pass text carries a lambda companion projection (the omitted helper and the inlined
//! body the `lambda$…` channel staged) still folds its direct static member family, and the
//! folded unit keeps both presentations at once — the nested member declaration and the lambda
//! inline — beside the same folded-family behavior the static fold already guaranteed.
//!
//! The four anchors:
//!
//! 1. the Y1 shape — a single static interface child in a lambda root — folds, recompiles under
//!    `javac --release 8`, verifies under `-Xverify:all`, and prints exactly what the original
//!    class files print (`hi!`/`45`/`[b, aa]`/`8`);
//! 2. the class-child twin and the array-returning and enum-constant projection forms fold the
//!    same way, so the retention carries the channels the first pass staged, not one favored
//!    shape;
//! 3. the fold's second round does not degrade: the folded unit, recompiled and read again,
//!    still folds with the lambda inline in place — never the physical companion back;
//! 4. a root without projections serializes exactly as before (no new report key) and folds as
//!    it did, so the change is invisible outside the projection-carrying family.

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
            "jarde-lambda-root-fold-{label}-{}-{nonce}-{}",
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
        .lines()
        .find_map(|line| {
            line.strip_prefix("public class ")
                .or_else(|| line.strip_prefix("public enum "))
                .or_else(|| line.strip_prefix("class "))
                .or_else(|| line.strip_prefix("enum "))
                .and_then(|rest| rest.split_whitespace().next())
                .map(|name| format!("{name}.java"))
        })
        .expect("the source states a top-level type");
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
    let engine = Engine::new();
    let mut budget =
        Budget::new(jarde::facade::task_limits(&[]).expect("the task defaults are bounded"));
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
/// rewritten text — the same convention the static-fold acceptance uses.
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

/// The Y1 shape of the patrol's frozen fixture: one static interface child, four lambda sites
/// (an inlineable string function, method references, stream lambdas and a capturing one).
const Y1L_SOURCE: &str = r#"import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.function.Function;
import java.util.function.Supplier;
public class Y1L {
    interface StrFn { String apply(String s); }
    static String viaLambda(String s) {
        StrFn f = x -> x + "!";
        return f.apply(s);
    }
    static int viaMethodRef() {
        Supplier<Integer> sup = () -> 42;
        Function<String, Integer> len = String::length;
        return sup.get() + len.apply("hey");
    }
    static List<String> viaStream() {
        List<String> xs = new ArrayList<String>(Arrays.asList("b", "aa", "ccc"));
        xs.removeIf(x -> x.length() > 2);
        xs.sort((a, b) -> a.length() - b.length());
        return xs;
    }
    static int captureLambda(int n) {
        Function<Integer, Integer> add = v -> v + n;
        return add.apply(5);
    }
    public static void main(String[] a) {
        System.out.println(viaLambda("hi"));
        System.out.println(viaMethodRef());
        System.out.println(viaStream());
        System.out.println(captureLambda(3));
    }
}
"#;

/// The class-child twin: the same lambda content, a static `class` member instead.
const Y1LM_SOURCE: &str = r#"import java.util.function.Function;
public class Y1LM {
    static class Solo { int v() { return 7; } }
    static String viaLambda(String s) {
        Function<String, String> f = x -> x + "!";
        return f.apply(s);
    }
    static int useSolo() { return new Solo().v(); }
    public static void main(String[] a) {
        System.out.println(viaLambda("hi"));
        System.out.println(useSolo());
    }
}
"#;

/// The array-returning form: the projection's inlined body constructs the array.
const ARRL_SOURCE: &str = r#"public class ARRL {
    interface Sup { int[] make(int n); }
    static int[] viaArray(int n) {
        Sup s = int[]::new;
        return s.make(n);
    }
    public static void main(String[] a) {
        System.out.println(java.util.Arrays.toString(viaArray(3)));
    }
}
"#;

/// The enum-constant form: the root's own enum projection (the constants list, the omitted
/// implicit members) beside the folded static child.
const ENL_SOURCE: &str = r#"public enum ENL {
    A, B;
    static class Helper { int v() { return 7; } }
    int use() { return new Helper().v(); }
    public static void main(String[] a) {
        System.out.println(A.use());
        System.out.println(B.use());
    }
}
"#;

/// The no-projection control: no lambda, no projection channel — the fold this family always
/// had, byte for byte, and a report that serializes without the new key.
const PLAIN_SOURCE: &str = "public class PLAIN {\n    interface Marker { }\n    static class Kid extends java.lang.Object { int v() { return 5; } }\n    public static void main(String[] a) { System.out.println(new Kid().v()); }\n}\n";

fn assert_projected_fold(report: &ClassSourceReport) {
    let projected = matches!(
        &report.member_family,
        ClassSourceMemberFamily::PreparedStatic {
            projection: ClassSourceMemberProjection::Projected { .. },
            ..
        }
    ) || matches!(
        &report.member_family,
        ClassSourceMemberFamily::PreparedFold {
            projection: ClassSourceMemberProjection::Projected { .. },
            ..
        }
    );
    assert!(
        projected,
        "the family folds: {:?}\n{}",
        report.member_family, report.text
    );
}

#[test]
fn y1_lambda_root_folds_with_both_presentations_and_reproduces_the_baseline() {
    let (jar, _) = compile_family("y1l", Y1L_SOURCE);
    let report = source_of(&jar, "Y1L");
    assert_projected_fold(&report);
    // The nested declaration the fold states...
    assert!(
        report.text.contains("static interface StrFn {"),
        "the folded unit nests the interface declaration: {}",
        report.text
    );
    // ...beside the first pass's lambda presentation: the companion omitted, the body inlined,
    // and the one reference re-spelled with the fold's own source spelling.
    assert!(
        report
            .text
            .contains("omitted physical lambda helper \"lambda$viaLambda$0\""),
        "the omitted-helper marker is retained: {}",
        report.text
    );
    assert!(
        report
            .text
            .contains("StrFn local1 = (java.lang.String p0) -> p0 + \"!\";"),
        "the inlined companion body and the re-spelled declaration type: {}",
        report.text
    );
    assert!(
        !report.text.contains("Y1L$StrFn"),
        "no pool spelling of the folded member remains: {}",
        report.text
    );
    assert_eq!(
        recompile_and_run("y1l-run", &report.text, "Y1L", &jar),
        "hi!\n45\n[b, aa]\n8\n",
        "the folded unit reproduces the original class file's behavior"
    );
}

#[test]
fn folded_lambda_root_second_round_does_not_degrade() {
    let (jar, _) = compile_family("y1l2", Y1L_SOURCE);
    let first = source_of(&jar, "Y1L");
    assert_projected_fold(&first);
    // The rerun net the instance fold established, over the fold's own product: the folded unit
    // recompiled and read again presents both halves again — never the physical companion back.
    // (The recompiled shape's own recovery limits — javac compiles the explicitly-typed lambda
    // presentation into writes the local-assignment proof refuses — predate this change and
    // degrade the unfolded baseline's second round identically; they are registered with the
    // change's evidence, not pinned here.)
    let second_jar = {
        let temp = TestDirectory::new("y1l2-recompile");
        std::fs::write(temp.path().join("Y1L.java"), &first.text)
            .expect("the folded text is written");
        let compiled = Command::new("javac")
            .args(["--release", "8", "-g:none"])
            .arg("Y1L.java")
            .current_dir(temp.path())
            .output()
            .expect("javac runs");
        assert!(
            compiled.status.success(),
            "{}\ntext was: {}",
            String::from_utf8_lossy(&compiled.stderr),
            first.text
        );
        let mut owned = Vec::new();
        for entry in std::fs::read_dir(temp.path()).expect("the directory reads") {
            let path = entry.expect("the entry reads").path();
            if path.extension().and_then(|e| e.to_str()) == Some("class") {
                owned.push((
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    std::fs::read(&path).unwrap(),
                ));
            }
        }
        let entries = owned
            .iter()
            .map(|(name, bytes)| (name.as_bytes() as &[u8], bytes.as_slice()))
            .collect::<Vec<_>>();
        jar_of(&entries)
    };
    let second = source_of(&second_jar, "Y1L");
    assert_projected_fold(&second);
    assert!(
        second.text.contains("static interface StrFn {"),
        "the second round keeps the nested declaration: {}",
        second.text
    );
    assert!(
        second.text.contains("(java.lang.String p0) -> p0 + \"!\""),
        "the second round keeps the inlined companion body: {}",
        second.text
    );
    assert!(
        !second.text.contains("lambda$viaLambda$0("),
        "the second round never calls or declares the physical companion (its omission marker \
         may name it): {}",
        second.text
    );
}

#[test]
fn class_child_and_array_and_enum_projection_roots_fold_too() {
    let (jar, _) = compile_family("y1lm", Y1LM_SOURCE);
    let report = source_of(&jar, "Y1LM");
    assert_projected_fold(&report);
    assert!(
        report.text.contains("static class Solo extends"),
        "the class child folds beside the lambda root: {}",
        report.text
    );
    assert!(
        report.text.contains("-> (java.lang.String) p0 + \"!\""),
        "the lambda inline is retained (the erased Function site types its parameter): {}",
        report.text
    );
    assert_eq!(
        recompile_and_run("y1lm-run", &report.text, "Y1LM", &jar),
        "hi!\n7\n",
        "the class-child twin reproduces the original class file's behavior"
    );

    let (jar, _) = compile_family("arrl", ARRL_SOURCE);
    let report = source_of(&jar, "ARRL");
    assert_projected_fold(&report);
    assert!(
        report.text.contains("static interface Sup {"),
        "the array-returning root folds: {}",
        report.text
    );
    assert!(
        report
            .text
            .contains("Sup local1 = (int p0) -> new int[p0];"),
        "the inlined array-constructor body and the re-spelled type: {}",
        report.text
    );
    assert_eq!(
        recompile_and_run("arrl-run", &report.text, "ARRL", &jar),
        "[0, 0, 0]\n",
        "the array-returning fold reproduces the original class file's behavior"
    );

    let (jar, _) = compile_family("enl", ENL_SOURCE);
    let report = source_of(&jar, "ENL");
    assert_projected_fold(&report);
    assert!(
        report.text.contains("static class Helper extends"),
        "the enum root folds its static child: {}",
        report.text
    );
    assert!(
        report.text.contains("    A,\n    B;\n"),
        "the enum-constant projection is retained: {}",
        report.text
    );
    assert!(
        !report.text.contains("$VALUES"),
        "the implicit enum members stay omitted: {}",
        report.text
    );
    assert_eq!(
        recompile_and_run("enl-run", &report.text, "ENL", &jar),
        "7\n7\n",
        "the enum fold reproduces the original class file's behavior"
    );
}

#[test]
fn projection_free_root_serializes_and_folds_exactly_as_before() {
    let (jar, _) = compile_family("plain", PLAIN_SOURCE);
    let report = source_of(&jar, "PLAIN");
    assert_projected_fold(&report);
    // The retention is absent for a root the first pass did not project: the serialized report
    // keeps the exact shape it had before this change introduced the field.
    assert!(
        report.projection_inputs.is_empty(),
        "a projection-free root retains nothing"
    );
    let document = serde_json::to_value(&report).expect("the report serializes");
    assert!(
        document.get("projection_inputs").is_none(),
        "the serialized report carries no new key for a projection-free root"
    );
    assert_eq!(
        recompile_and_run("plain-run", &report.text, "PLAIN", &jar),
        "5\n",
        "the projection-free fold reproduces the original class file's behavior"
    );

    // A projected root does carry its retention, and the retained member text names the
    // projection's own marker — the fact the fold re-projects from.
    let (jar, _) = compile_family("plain2", Y1L_SOURCE);
    let report = source_of(&jar, "Y1L");
    let document = serde_json::to_value(&report).expect("the report serializes");
    let retained = document
        .get("projection_inputs")
        .expect("a projected root retains its inputs");
    assert!(
        retained
            .get("omitted_methods")
            .and_then(|value| value.as_array())
            .is_some_and(|values| !values.is_empty()),
        "the omitted companions are retained: {retained}"
    );
    assert!(
        retained
            .get("member_texts")
            .and_then(|value| value.as_array())
            .is_some_and(
                |values| values
                    .iter()
                    .any(
                        |entry| entry.get("emission").is_some_and(|emission| emission
                            .get("segments")
                            .and_then(|segments| segments.as_array())
                            .is_some_and(|segments| !segments.is_empty()))
                    )
            ),
        "the re-spellable emission with its anchor table is retained: {retained}"
    );
}
