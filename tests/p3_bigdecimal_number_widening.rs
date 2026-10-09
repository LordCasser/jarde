//! Main-only BigDecimal to Number widening against the two frozen original Main classfiles.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::fs;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};

const STORE: u16 = 0;
static NEXT: AtomicU64 = AtomicU64::new(0);
const EXPECTED_STDOUT: &[u8] =
    include_bytes!("fixtures/bigdecimal-number-widening/oracle/original-main.stdout");
const EXPECTED_STDERR: &[u8] =
    include_bytes!("fixtures/bigdecimal-number-widening/oracle/original-main.stderr");
const NUMBER_ARGUMENT_STDOUT: &[u8] =
    include_bytes!("fixtures/bigdecimal-number-widening/oracle/original-number-argument.stdout");
const NUMBER_ARGUMENT_STDERR: &[u8] =
    include_bytes!("fixtures/bigdecimal-number-widening/oracle/original-number-argument.stderr");

const LEGS: &[(&str, &[u8])] = &[
    (
        "javac8",
        include_bytes!("fixtures/bigdecimal-number-widening/javac8/Main.class"),
    ),
    (
        "javac23",
        include_bytes!("fixtures/bigdecimal-number-widening/javac23/Main.class"),
    ),
];

const NUMBER_ARGUMENT_LEGS: &[(&str, &[u8])] = &[
    (
        "javac8",
        include_bytes!("fixtures/bigdecimal-number-widening/javac8/NumberArgument.class"),
    ),
    (
        "javac23",
        include_bytes!("fixtures/bigdecimal-number-widening/javac23/NumberArgument.class"),
    ),
];

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-bigdecimal-number-{}-{label}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("private fixture scratch directory is created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn archive(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in files {
            let (mut entry, config) = writer
                .new_file(EntryPath::verbatim(name.as_bytes().to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("class entry starts");
            let mut stream = config.wrap(&mut entry);
            stream.write_all(bytes).expect("class entry is written");
            let (_, descriptor) = stream.finish().expect("class entry closes");
            entry.finish(descriptor).expect("class entry finishes");
        }
        writer.finish().expect("fixture archive closes");
    }
    output.into_inner()
}

fn class_source(snapshot: &artifact::ArtifactSnapshot, class: &str) -> ClassSourceReport {
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
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut task_budget(&[]).expect("task defaults are bounded"),
        )
        .expect("valid Main class-source request answers")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("{class} class source did not complete: {other:?}"),
    }
}

fn recovered_body<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let member = report
        .methods
        .iter()
        .find(|member| member.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("class declares {name}"));
    match &member.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("{name} has no complete recovery record: {other:?}"),
    }
}

fn javac_tool() -> PathBuf {
    std::env::var_os("JARDE_JAVAC23")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("javac"))
}

fn java_tool() -> PathBuf {
    std::env::var_os("JARDE_JAVA23")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("java"))
}

fn compile_source(dir: &Path, source_name: &str) -> Output {
    let empty = dir.join("empty-classpath-sourcepath");
    let classes = dir.join("classes");
    fs::create_dir_all(&empty).expect("empty source/class path exists");
    fs::create_dir_all(&classes).expect("isolated output classes exist");
    Command::new(javac_tool())
        .args([
            "-source",
            "8",
            "-target",
            "8",
            "-g:none",
            "-Xlint:-options",
            "-classpath",
        ])
        .env_remove("JAVA_TOOL_OPTIONS")
        .env_remove("_JAVA_OPTIONS")
        .env_remove("JDK_JAVA_OPTIONS")
        .env_remove("CLASSPATH")
        .arg(&empty)
        .arg("-sourcepath")
        .arg(&empty)
        .arg("-d")
        .arg(&classes)
        .arg(dir.join(source_name))
        .current_dir(dir)
        .output()
        .expect("javac is available for Java 8 source compilation")
}

fn run_verified(classes: &Path, main_class: &str) -> Output {
    Command::new(java_tool())
        .arg("-Xverify:all")
        .env_remove("JAVA_TOOL_OPTIONS")
        .env_remove("_JAVA_OPTIONS")
        .env_remove("JDK_JAVA_OPTIONS")
        .env_remove("CLASSPATH")
        .arg("-cp")
        .arg(classes)
        .arg(main_class)
        .output()
        .expect("java is available for verifier execution")
}

fn main_report(main_class: &[u8]) -> ClassSourceReport {
    let input = archive(&[("Main.class", main_class)]);
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(input), &mut task_budget(&[]).unwrap())
        .expect("one Main-only snapshot opens");
    class_source(&snapshot, "Main")
}

fn number_argument_report(class: &[u8]) -> ClassSourceReport {
    let input = archive(&[("NumberArgument.class", class)]);
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(input), &mut task_budget(&[]).unwrap())
        .expect("one NumberArgument-only snapshot opens");
    class_source(&snapshot, "NumberArgument")
}

fn assert_bigdecimal_main_structure(report: &ClassSourceReport, leg: &str) {
    let body = recovered_body(report, "main");
    assert_eq!(body.quality, Quality::Structured, "{leg}: {body:?}");
    assert_eq!(body.representation, Representation::Java, "{leg}");
    assert!(!body.text.contains("@bytecode"), "{leg}: {}", body.text);
    assert!(
        report.text.contains("System.out.println("),
        "{leg}: {}",
        report.text
    );
    assert_eq!(body.text.matches("new java.math.BigDecimal(").count(), 1);
    assert_eq!(body.text.matches("new java.lang.Number[]{").count(), 1);
    assert_eq!(body.text.matches(".append(").count(), 3);
    assert_eq!(body.text.matches(".toString()").count(), 1);
    assert!(body.text.contains(".append((java.lang.Object) local1[0])"));
    for bci in [1, 6, 9, 12, 15, 16, 17, 28, 29, 34, 40, 43, 46] {
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "{leg}: missing source for required BCI {bci}: {:?}",
            body.source_map.segments()
        );
    }
}

fn assert_number_argument_structure(report: &ClassSourceReport, leg: &str) {
    let body = recovered_body(report, "main");
    assert_eq!(body.quality, Quality::Structured, "{leg}: {body:?}");
    assert_eq!(body.representation, Representation::Java, "{leg}");
    assert!(!body.text.contains("@bytecode"), "{leg}: {}", body.text);
    assert_eq!(body.text.matches("new java.math.BigDecimal(").count(), 1);
    assert_eq!(body.text.matches("identity(").count(), 1);
    for bci in [0, 3, 6, 7, 9, 12, 15, 18] {
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "{leg}: missing source for NumberArgument main BCI {bci}: {:?}",
            body.source_map.segments()
        );
    }

    let identity = report
        .methods
        .iter()
        .find(|member| member.item.name.raw().0 == b"identity")
        .expect("NumberArgument declares identity");
    assert_eq!(
        identity.item.descriptor.raw().0,
        b"(Ljava/lang/Number;)Ljava/lang/Number;",
        "the invoked method's actual descriptor requires Number"
    );
    let identity_body = recovered_body(report, "identity");
    assert_eq!(
        identity_body.quality,
        Quality::Structured,
        "{leg}: {identity_body:?}"
    );
    assert_eq!(identity_body.representation, Representation::Java, "{leg}");
    assert!(
        !identity_body.text.contains("@bytecode"),
        "{leg}: {}",
        identity_body.text
    );
}

#[test]
fn main_only_bigdecimal_array_is_recovered_from_both_frozen_javac_legs() {
    for (leg, main_class) in LEGS {
        let report = main_report(main_class);
        assert_bigdecimal_main_structure(&report, leg);
    }
}

#[test]
fn bigdecimal_argument_is_recovered_at_the_actual_number_descriptor_call() {
    for (leg, class) in NUMBER_ARGUMENT_LEGS {
        let report = number_argument_report(class);
        assert_number_argument_structure(&report, leg);
    }
}

#[test]
#[ignore = "requires the frozen Java 23 toolchain to compile and verify generated source"]
fn main_only_bigdecimal_generated_source_matches_frozen_original_runtime() {
    for (leg, main_class) in LEGS {
        let report = main_report(main_class);
        assert_bigdecimal_main_structure(&report, leg);
        let generated = Scratch::new(leg);
        fs::write(generated.path().join("Main.java"), &report.text)
            .expect("complete generated Main source is written");
        let compile = compile_source(generated.path(), "Main.java");
        assert!(
            compile.status.success(),
            "{leg} generated Main did not compile:\n{}\n{}",
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = run_verified(&generated.path().join("classes"), "Main");
        assert!(
            run.status.success(),
            "{leg} generated Main verifier run failed: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            run.stdout, EXPECTED_STDOUT,
            "{leg} stdout differs from original"
        );
        assert_eq!(
            run.stderr, EXPECTED_STDERR,
            "{leg} stderr differs from original"
        );
    }
}

#[test]
#[ignore = "requires the frozen Java 23 toolchain to compile and verify generated source"]
fn bigdecimal_number_argument_generated_source_matches_frozen_runtime() {
    for (leg, class) in NUMBER_ARGUMENT_LEGS {
        let report = number_argument_report(class);
        assert_number_argument_structure(&report, leg);
        let generated = Scratch::new(&format!("{leg}-number-argument"));
        fs::write(generated.path().join("NumberArgument.java"), &report.text)
            .expect("complete generated NumberArgument source is written");
        let compile = compile_source(generated.path(), "NumberArgument.java");
        assert!(
            compile.status.success(),
            "{leg} generated NumberArgument did not compile:\n{}\n{}",
            String::from_utf8_lossy(&compile.stdout),
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = run_verified(&generated.path().join("classes"), "NumberArgument");
        assert!(
            run.status.success(),
            "{leg} generated NumberArgument verifier run failed: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            run.stdout, NUMBER_ARGUMENT_STDOUT,
            "{leg} NumberArgument stdout differs from original"
        );
        assert_eq!(
            run.stderr, NUMBER_ARGUMENT_STDERR,
            "{leg} NumberArgument stderr differs from original"
        );
    }
}
