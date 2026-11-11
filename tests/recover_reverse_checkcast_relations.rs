//! Reconstructed tests, not historical validation receipts. No source rewriting is permitted.
use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::{
    fs,
    io::{Cursor, Write},
    path::Path,
    process::Command,
    slice,
};
#[path = "support/reverse_cast_fixture.rs"]
mod fixture;
fn jar(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    {
        let mut zip = ZipArchiveWriter::new(&mut out);
        for (name, bytes) in entries {
            let (mut entry, config) = zip
                .new_file(EntryPath::verbatim(name.as_bytes().to_vec()))
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
    out.into_inner()
}
fn report(entries: &[(String, Vec<u8>)], name: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let mut budget = task_budget(&[]).unwrap();
    let snapshot = engine
        .open(ArtifactInput::bytes(jar(entries)), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        source_target: SourceTarget::JAVA_8,
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
            loader: LoaderId("app".into()),
        },
    };
    match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
        .unwrap()
    {
        OperationOutcome::Performed(r) => r,
        other => panic!("{other:?}"),
    }
}
fn cases() -> [(&'static str, &'static str, &'static str, bool); 3] {
    [
        ("CastTag", "java/lang/annotation/Annotation", "Tag", true),
        (
            "RenamedCast",
            "java/lang/annotation/Annotation",
            "RenamedMarker",
            true,
        ),
        ("InterfaceCast", "RootFace", "LeafFace", false),
    ]
}
fn entries(
    name: &str,
    parent: &str,
    child: &str,
    annotation: bool,
    wide: bool,
) -> Vec<(String, Vec<u8>)> {
    let mut e = vec![
        (
            format!("{name}.class"),
            fixture::subject(name, parent, child, wide),
        ),
        (
            format!("{child}.class"),
            fixture::interface(child, Some(parent), annotation),
        ),
    ];
    if !annotation {
        e.push((
            format!("{parent}.class"),
            fixture::interface(parent, None, false),
        ));
    }
    e
}
#[test]
fn selected_reverse_edges_remove_only_unnecessary_object_bridges() {
    for ((name, parent, child, annotation), wide) in cases()
        .into_iter()
        .flat_map(|case| [false, true].map(|wide| (case, wide)))
    {
        let e = entries(name, parent, child, annotation, wide);
        let r = report(&e, name);
        assert!(!r.text.contains("@bytecode"), "{}", r.text);
        assert!(r.text.contains(&format!("({child}) arg0")), "{}", r.text);
        assert!(
            !r.text.contains("(Object)") && !r.text.contains("(java.lang.Object)"),
            "{}",
            r.text
        );
        let method = r
            .methods
            .iter()
            .find(|m| m.item.name.raw().0 == b"run")
            .unwrap();
        let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
            panic!("missing run recovery")
        };
        assert_eq!(body.quality, jarde_jvm::ir::Quality::Structured);
        assert_eq!(body.representation, jarde_jvm::ir::Representation::Java);
        let cast_bci = fixture::cast_bci(wide);
        for bci in [0, cast_bci, cast_bci + 3] {
            assert!(
                !body.source_map.of_bci(bci).is_empty(),
                "wide={wide} missing physical BCI {bci}: {}",
                body.text
            );
        }
        let absent: Vec<_> = e
            .into_iter()
            .filter(|(n, _)| n != &format!("{child}.class"))
            .collect();
        let missing = report(&absent, name);
        assert!(
            missing.text.contains("(Object)") || missing.text.contains("(java.lang.Object)"),
            "missing target must not acquire a proved narrowing edge: {}",
            missing.text
        );
    }
}
fn success(command: &mut Command) -> String {
    command
        .env_remove("JAVA_TOOL_OPTIONS")
        .env_remove("_JAVA_OPTIONS")
        .env_remove("JDK_JAVA_OPTIONS")
        .env_remove("CLASSPATH");
    let output = command.output().expect("required JDK command starts");
    assert!(
        output.status.success(),
        "{command:?}\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
fn run(dir: &Path) -> String {
    success(
        Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(dir)
            .arg("CastRunner"),
    )
}
struct Scratch(std::path::PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
#[ignore = "requires JDK; run explicitly with --ignored --exact"]
fn complete_unmodified_cast_classes_match_original_null_identity_and_cce() {
    let root = std::env::temp_dir().join(format!(
        "jarde-reverse-cast-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let scratch = Scratch(root);
    let root = &scratch.0;
    fs::create_dir_all(root).unwrap();
    for ((name, parent, child, annotation), wide) in cases()
        .into_iter()
        .flat_map(|case| [false, true].map(|wide| (case, wide)))
    {
        let e = entries(name, parent, child, annotation, wide);
        let recovered = report(&e, name);
        let original = root.join(format!("{name}-{wide}")).join("original");
        let rebuilt = root.join(format!("{name}-{wide}")).join("recovered");
        fs::create_dir_all(&original).unwrap();
        fs::create_dir_all(&rebuilt).unwrap();
        for (file, bytes) in &e {
            fs::write(original.join(file), bytes).unwrap();
            if file != &format!("{name}.class") {
                fs::write(rebuilt.join(file), bytes).unwrap();
            }
        }
        let parent = parent.replace('/', ".");
        let runner = format!(
            r#"public class CastRunner {{
 public static void main(String[] args) {{
  if ({name}.run(null) != null) throw new AssertionError("null");
  Object good = java.lang.reflect.Proxy.newProxyInstance(CastRunner.class.getClassLoader(), new Class<?>[] {{{child}.class}}, (p,m,a) -> null);
  if ({name}.run(({parent}) good) != good) throw new AssertionError("identity");
  Object bad = java.lang.reflect.Proxy.newProxyInstance(CastRunner.class.getClassLoader(), new Class<?>[] {{{parent}.class}}, (p,m,a) -> null);
  try {{ {name}.run(({parent}) bad); throw new AssertionError("missing CCE"); }} catch (ClassCastException expected) {{}}
  System.out.println("null/identity/CCE");
 }}
}}"#
        );
        for dir in [&original, &rebuilt] {
            fs::write(dir.join("CastRunner.java"), &runner).unwrap();
        }
        fs::write(rebuilt.join(format!("{name}.java")), &recovered.text).unwrap();
        success(
            Command::new("javac")
                .args(["--release", "8", "-cp"])
                .arg(&original)
                .arg("-sourcepath")
                .arg(&original)
                .arg("-d")
                .arg(&original)
                .arg(original.join("CastRunner.java")),
        );
        success(
            Command::new("javac")
                .args(["--release", "8", "-cp"])
                .arg(&rebuilt)
                .arg("-sourcepath")
                .arg(&rebuilt)
                .arg("-d")
                .arg(&rebuilt)
                .arg(rebuilt.join(format!("{name}.java")))
                .arg(rebuilt.join("CastRunner.java")),
        );
        let expected = run(&original);
        assert_eq!(expected, "null/identity/CCE\n");
        assert_eq!(run(&rebuilt), expected);
    }
}
