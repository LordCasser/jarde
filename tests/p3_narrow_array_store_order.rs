//! End-to-end order checks for real recovered `bastore`/`castore`/`sastore` expressions.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const HERE: &str = "tests/fixtures/p3-narrow-array-store-order-e2e";
const EFFECTS: &str =
    include_str!("fixtures/p3-narrow-array-store-order-e2e/NarrowArrayStoreOrderEffects.java");
const RUNNER: &str =
    include_str!("fixtures/p3-narrow-array-store-order-e2e/NarrowArrayStoreOrderRunner.java");

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
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
    }
}

fn run(command: &mut Command, context: &str) -> Output {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{context}: {error}"));
    assert!(
        output.status.success(),
        "{context} failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn compile_java(output: &Path, classpath: Option<&Path>, sources: &[&Path]) {
    fs::create_dir_all(output).expect("create Java output directory");
    let mut command = Command::new("javac");
    command
        .args(["--release", "8", "-g:none", "-d"])
        .arg(output);
    if let Some(classpath) = classpath {
        command.arg("-cp").arg(classpath);
    }
    command.args(sources);
    run(&mut command, "javac --release 8");
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is after the epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "jarde-narrow-array-store-order-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create private Java fixture directory");
        Self(root)
    }

    fn child(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(&path).expect("create scratch child");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn member<'a>(
    report: &'a ClassSourceReport,
    name: &[u8],
    descriptor: &[u8],
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name && method.item.descriptor.raw().0 == descriptor
        })
        .unwrap_or_else(|| {
            panic!(
                "missing method {}{}",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(descriptor)
            )
        })
}

fn recovery<'a>(
    report: &'a ClassSourceReport,
    name: &[u8],
    descriptor: &[u8],
) -> &'a RecoveryReport {
    match &member(report, name, descriptor).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!(
            "method {}{} was not recovered: {other:?}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(descriptor)
        ),
    }
}

fn class_source(bytes: &[u8]) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut Budget::new(limits()),
        )
        .expect("patched Java 8 fixture opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("NarrowArrayStoreOrder"),
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
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .expect("class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one standalone class has a non-performed outcome: {other:?}"),
    }
}

#[test]
#[ignore = "requires JDK and compares actual recovered B/C/S store ordering against verified patched classes"]
fn recovered_bcs_array_index_value_order_matches_the_patched_jvm() {
    let scratch = Scratch::new();
    let fixture = Path::new(HERE);
    let source_path = fixture.join("NarrowArrayStoreOrder.java");
    let effects_path = fixture.join("NarrowArrayStoreOrderEffects.java");
    let runner_path = fixture.join("NarrowArrayStoreOrderRunner.java");
    let source_classes = scratch.child("source-classes");
    compile_java(&source_classes, None, &[&source_path, &effects_path]);

    let patched_class = source_classes.join("NarrowArrayStoreOrder.class");
    let patcher = fixture.join("patch_order_stores.py");
    let patched_output = scratch.0.join("patched.class");
    let mut patch = Command::new("python3");
    patch.arg(patcher).arg(&patched_class).arg(&patched_output);
    run(&mut patch, "patch verifier-valid B/C/S stores");
    let patched_bytes = fs::read(&patched_output).expect("read patched fixture class");
    Engine::new()
        .open(
            ArtifactInput::bytes(patched_bytes.clone()),
            &mut Budget::new(limits()),
        )
        .expect("patched fixture parses");
    fs::copy(&patched_output, &patched_class).expect("install patched class for verification");

    let patched_runner = scratch.child("patched-runner");
    compile_java(&patched_runner, Some(&source_classes), &[&runner_path]);
    let patched_cp = format!("{}:{}", patched_runner.display(), source_classes.display());
    let mut patched_java = Command::new("java");
    patched_java.args([
        "-Xverify:all",
        "-cp",
        &patched_cp,
        "NarrowArrayStoreOrderRunner",
    ]);
    let patched_trace = run(&mut patched_java, "verified patched JVM trace").stdout;
    assert_eq!(String::from_utf8_lossy(&patched_trace).lines().count(), 24);

    let report = class_source(&patched_bytes);
    assert_eq!(report.methods.len(), 4);
    assert!(
        !report.text.contains("@bytecode"),
        "full recovered class contains a refusal: {}",
        report.text
    );
    let methods = [
        (b"storeByte".as_slice(), b"([BIIZZZ)V".as_slice(), "(byte)"),
        (b"storeChar", b"([CIIZZZ)V", "(char)"),
        (b"storeShort", b"([SIIZZZ)V", "(short)"),
    ];
    for (name, descriptor, cast) in methods {
        let method = member(&report, name, descriptor);
        let body = recovery(&report, name, descriptor);
        assert!(body.produced(), "{}", body.text);
        assert_eq!(body.representation, Representation::Java, "{}", method.text);
        assert_eq!(body.quality, Quality::Structured, "{}", method.text);
        assert!(!method.text.contains("@bytecode"), "{}", method.text);
        assert!(
            method.text.contains(cast),
            "store cast absent: {}",
            method.text
        );
        let array_producer = match name {
            b"storeByte" => "NarrowArrayStoreOrderEffects.byteArray",
            b"storeChar" => "NarrowArrayStoreOrderEffects.charArray",
            _ => "NarrowArrayStoreOrderEffects.shortArray",
        };
        for producer in [
            array_producer,
            "NarrowArrayStoreOrderEffects.index",
            "NarrowArrayStoreOrderEffects.value",
        ] {
            assert_eq!(
                method.text.matches(producer).count(),
                1,
                "producer missing or duplicated: {}",
                method.text
            );
        }
        for bci in [7, 13, 19, 22] {
            assert!(
                !body.text_of_bci(bci).is_empty(),
                "operand/store BCI {bci} lost: {}",
                body.text
            );
        }
        assert!(
            !body.source_map.direct_of_bci(22).is_empty(),
            "actual store BCI lacks a direct source anchor: {}",
            body.text
        );
    }

    let recovered_class = scratch.child("recovered");
    fs::write(
        recovered_class.join("NarrowArrayStoreOrder.java"),
        &report.text,
    )
    .expect("write complete recovered class");
    fs::write(
        recovered_class.join("NarrowArrayStoreOrderEffects.java"),
        EFFECTS,
    )
    .expect("write helper source");
    fs::write(
        recovered_class.join("NarrowArrayStoreOrderRunner.java"),
        RUNNER,
    )
    .expect("write runner source");
    compile_java(
        &recovered_class,
        None,
        &[
            &recovered_class.join("NarrowArrayStoreOrder.java"),
            &recovered_class.join("NarrowArrayStoreOrderEffects.java"),
            &recovered_class.join("NarrowArrayStoreOrderRunner.java"),
        ],
    );
    let mut recovered_java = Command::new("java");
    recovered_java.args([
        "-Xverify:all",
        "-cp",
        &recovered_class.to_string_lossy(),
        "NarrowArrayStoreOrderRunner",
    ]);
    let recovered_trace = run(&mut recovered_java, "verified recovered JVM trace").stdout;
    assert_eq!(
        recovered_trace, patched_trace,
        "recovered complete-class trace differs from patched input"
    );

    eprintln!(
        "matched trace rows: {}",
        String::from_utf8_lossy(&patched_trace).lines().count()
    );
}
