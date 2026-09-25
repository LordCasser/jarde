//! Acceptance fixture for verifier-valid Java 8 `boolean[]` initializer bytecode.

use jarde::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-boolean-array-initializers/v8/BoolInit.class");
const RUNNER: &str = include_str!("fixtures/p3-boolean-array-initializers/BoolInitRunner.java");
const EXPECTED: &str = include_str!("fixtures/p3-boolean-array-initializers/expected.txt");
static NEXT: AtomicU64 = AtomicU64::new(0);

fn budget() -> Budget {
    Budget::new(Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    })
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after the epoch")
            .as_nanos();
        let seq = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "jarde-boolean-array-init-{}-{nonce}-{seq}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated Java output directory");
        Self(path)
    }

    fn write(&self, name: &str, contents: &str) {
        fs::write(self.0.join(name), contents).expect("write temporary Java source");
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn java8_compile(dir: &TempDir, classpath: Option<&str>, files: &[&str]) {
    let mut command = Command::new("javac");
    command.args(["--release", "8", "-g:none", "-d"]);
    command.arg(&dir.0);
    if let Some(classpath) = classpath {
        command.arg("-classpath").arg(classpath);
    }
    for file in files {
        command.arg(dir.0.join(file));
    }
    let output = command.output().expect("javac is available");
    assert!(
        output.status.success(),
        "javac refused the source:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn execute(dir: &TempDir) -> String {
    let output = Command::new("java")
        .args(["-Xverify:all", "-classpath"])
        .arg(&dir.0)
        .arg("BoolInitRunner")
        .output()
        .expect("java is available");
    assert!(
        output.status.success(),
        "verified JVM execution failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("runner output is UTF-8")
}

#[test]
#[ignore = "needs a JDK on PATH to compile the complete recovered class with Java 8"]
fn complete_recovered_class_recompiles_and_matches_the_verified_fixture() {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(FIXTURE.to_vec()), &mut budget())
        .expect("fixture is a standalone Java 8 class");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("BoolInit"),
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
    let report = match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("legal class-source request")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("class-source did not complete: {other:?}"),
    };
    assert!(
        !report.text.contains("@bytecode"),
        "quoted bytecode remains"
    );
    assert!(
        !report.text.contains("not recovered"),
        "unrecovered member remains"
    );
    assert!(
        report.text.contains("return new boolean[]{"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("% 2 != 0"),
        "raw integer elements have no low-bit conversion: {}",
        report.text
    );

    let values = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"values")
        .expect("values() is declared");
    let ClassSourceOutcome::Recovered { report: body, .. } = &values.outcome else {
        panic!("values() was not recovered: {:?}", values.outcome);
    };
    for bci in [
        0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    ] {
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "values() omitted bytecode source {bci}: {:?}",
            body.source_map.segments()
        );
    }
    for store_bci in [6, 10, 14, 18, 22] {
        assert!(
            !body.source_map.of_bci(store_bci).is_empty(),
            "values() omitted bytecode source {store_bci}: {:?}",
            body.source_map.segments()
        );
    }
    let mut low_bit_spans = Vec::new();
    for store_bci in [14, 18, 22] {
        let source = body.source_map.of_bci(store_bci);
        let conversion = source
            .iter()
            .find(|segment| {
                let text = &body.text[segment.start()..segment.end()];
                text.contains("% 2 != 0") && !text.starts_with("new boolean[]{")
            })
            .unwrap_or_else(|| panic!("store {store_bci} has no element-specific conversion span"));
        low_bit_spans.push((conversion.start(), conversion.end()));
    }
    low_bit_spans.sort_unstable();
    low_bit_spans.dedup();
    assert_eq!(
        low_bit_spans.len(),
        3,
        "the 2, 3, and -1 conversions need distinct source spans"
    );

    let original = TempDir::new();
    fs::write(original.0.join("BoolInit.class"), FIXTURE).expect("copy frozen class");
    original.write("BoolInitRunner.java", RUNNER);
    java8_compile(&original, None, &["BoolInitRunner.java"]);
    let original_output = execute(&original);
    assert_eq!(
        original_output, EXPECTED,
        "frozen patch-class oracle drifted"
    );

    let generated = TempDir::new();
    generated.write("BoolInit.java", &report.text);
    generated.write("BoolInitRunner.java", RUNNER);
    java8_compile(&generated, None, &["BoolInit.java", "BoolInitRunner.java"]);
    assert_eq!(
        execute(&generated),
        original_output,
        "recovered full class differs from verifier-valid input"
    );
}

const EFFECTFUL_FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-boolean-array-initializers/v8/BoolInitEffectful.class");
const EFFECTFUL_RUNNER: &str =
    include_str!("fixtures/p3-boolean-array-initializers/BoolInitEffectfulRunner.java");
const EFFECTFUL_EXPECTED: &str =
    include_str!("fixtures/p3-boolean-array-initializers/expected-effectful.txt");

#[test]
#[ignore = "needs a JDK on PATH to compile and execute the effectful complete-class comparison"]
fn effectful_initializer_recovery_preserves_order_and_partial_exception_trace() {
    let snapshot = Engine::new()
        .open(
            ArtifactInput::bytes(EFFECTFUL_FIXTURE.to_vec()),
            &mut budget(),
        )
        .expect("effectful fixture is a standalone Java 8 class");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("BoolInitEffectful"),
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
    let report = match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("legal effectful class-source request")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("effectful class-source did not complete: {other:?}"),
    };
    assert!(
        !report.text.contains("@bytecode"),
        "quoted bytecode remains: {}",
        report.text
    );
    assert!(
        !report.text.contains("not recovered"),
        "unrecovered member remains: {}",
        report.text
    );
    assert!(
        report.text.contains("return new boolean[]{"),
        "{}",
        report.text
    );
    assert!(report.text.contains("% 2 != 0"), "{}", report.text);
    let values = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"values")
        .expect("effectful values() is declared");
    let ClassSourceOutcome::Recovered { report: body, .. } = &values.outcome else {
        panic!("effectful values() was not recovered: {:?}", values.outcome);
    };
    let mut effectful_spans = Vec::new();
    for store_bci in [10, 18, 26] {
        let source = body.source_map.of_bci(store_bci);
        assert!(
            !source.is_empty(),
            "effectful element omitted its store source {store_bci}: {:?}",
            body.source_map.segments()
        );
        let conversion = source
            .iter()
            .find(|segment| {
                let text = &body.text[segment.start()..segment.end()];
                text.contains("% 2 != 0") && !text.starts_with("new boolean[]{")
            })
            .unwrap_or_else(|| {
                panic!("effectful store {store_bci} has no element conversion span")
            });
        effectful_spans.push((conversion.start(), conversion.end()));
    }
    effectful_spans.sort_unstable();
    effectful_spans.dedup();
    assert_eq!(
        effectful_spans.len(),
        3,
        "effectful store conversions have distinct spans"
    );

    let original = TempDir::new();
    fs::write(
        original.0.join("BoolInitEffectful.class"),
        EFFECTFUL_FIXTURE,
    )
    .expect("copy frozen effectful class");
    original.write("BoolInitEffectfulRunner.java", EFFECTFUL_RUNNER);
    java8_compile(
        &original,
        Some(&original.0.display().to_string()),
        &["BoolInitEffectfulRunner.java"],
    );
    let original_output = execute_effectful(&original);
    assert_eq!(
        original_output, EFFECTFUL_EXPECTED,
        "frozen effectful JVM oracle drifted"
    );

    let generated = TempDir::new();
    generated.write("BoolInitEffectful.java", &report.text);
    generated.write("BoolInitEffectfulRunner.java", EFFECTFUL_RUNNER);
    java8_compile(
        &generated,
        None,
        &["BoolInitEffectful.java", "BoolInitEffectfulRunner.java"],
    );
    assert_eq!(
        execute_effectful(&generated),
        original_output,
        "effectful generated class changed order, call count, or partial exception trace"
    );
}

fn execute_effectful(dir: &TempDir) -> String {
    let output = Command::new("java")
        .args(["-Xverify:all", "-classpath"])
        .arg(&dir.0)
        .arg("BoolInitEffectfulRunner")
        .output()
        .expect("java is available");
    assert!(
        output.status.success(),
        "effectful verified JVM execution failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("effectful runner output is UTF-8")
}
