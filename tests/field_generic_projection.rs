use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const STANDALONE: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/field-generic-signatures/fixture/StandaloneFieldBoundary.java"
);
const RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/field-generic-signatures/fixture/StandaloneFieldRunner.java"
);
const REFLECTION: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/field-generic-signatures/fixture/StandaloneFieldReflectionRunner.java"
);
const CONFLICT: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/field-generic-signatures/fixture/FieldSignatureConflict.java"
);
const CONFLICT_RUNNER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/field-generic-signatures/fixture/FieldSignatureConflictRunner.java"
);
const TYPE_USE_FIELD: &str = "package fieldsig;\nimport java.lang.annotation.*;\nimport java.util.List;\n@Target(ElementType.TYPE_USE) @Retention(RetentionPolicy.RUNTIME) @interface FieldType {}\npublic final class TypeUseField { public List<@FieldType String> values; }\n";

struct TempDir(PathBuf);

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

impl TempDir {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock is after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-field-generic-{}-{stamp}-{}",
            std::process::id(),
            NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("create temporary test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 20,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn run(program: &str, args: &[&str]) -> std::process::Output {
    Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("run {program}: {error}"))
}

fn compile_java(source: &Path, output: &Path, classpath: Option<&Path>) {
    fs::create_dir_all(output).expect("create javac output directory");
    let mut command = Command::new("javac");
    command.args(["--release", "8", "-Xlint:-options", "-d"]);
    command.arg(output);
    if let Some(classpath) = classpath {
        command.arg("-cp").arg(classpath);
    }
    command.arg(source);
    let result = command.output().expect("run javac");
    assert!(
        result.status.success(),
        "javac failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn class_source(
    bytes: Vec<u8>,
    class_name: &str,
    evidence: RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> ClassSourceReport {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes), budget)
        .expect("class snapshot opens");
    let request = class_request(&snapshot, class_name);
    match Engine::new()
        .class_source_with_evidence(slice::from_ref(&snapshot), &request, &evidence, budget)
        .expect("class-source request succeeds")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source selection: {other:?}"),
    }
}

fn class_request(snapshot: &ArtifactSnapshot, class_name: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class_name),
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
    }
}

fn class_file(classes: &Path, name: &str) -> Vec<u8> {
    fs::read(classes.join(format!("fieldsig/{name}.class"))).expect("read javac class")
}

#[test]
fn standalone_field_signatures_recompile_and_match_reflection_and_caller_types() {
    let temp = TempDir::new();
    let source_dir = temp.path().join("source/fieldsig");
    let original_classes = temp.path().join("original");
    let generated_classes = temp.path().join("generated");
    fs::create_dir_all(&source_dir).unwrap();
    let standalone = source_dir.join("StandaloneFieldBoundary.java");
    let runner = source_dir.join("StandaloneFieldRunner.java");
    let reflection = source_dir.join("StandaloneFieldReflectionRunner.java");
    fs::write(&standalone, STANDALONE).unwrap();
    fs::write(&runner, RUNNER).unwrap();
    fs::write(&reflection, REFLECTION).unwrap();
    compile_java(&standalone, &original_classes, None);
    compile_java(&runner, &original_classes, Some(&original_classes));
    compile_java(&reflection, &original_classes, Some(&original_classes));

    let original = class_file(&original_classes, "StandaloneFieldBoundary");
    let mut essential_budget = Budget::new(limits());
    let essential = class_source(
        original.clone(),
        "fieldsig/StandaloneFieldBoundary",
        RecoveryEvidenceRequest::essential(),
        &mut essential_budget,
    );
    let mut all_budget = Budget::new(limits());
    let all = class_source(
        original,
        "fieldsig/StandaloneFieldBoundary",
        RecoveryEvidenceRequest::all(),
        &mut all_budget,
    );
    assert_eq!(
        essential.text, all.text,
        "essential/all changed Java source"
    );

    for (name, ty) in [
        ("names", "java.util.List<java.lang.String>"),
        ("numbers", "java.util.List<? extends java.lang.Number>"),
        ("current", "T"),
        ("vector", "T[]"),
        ("sink", "java.util.List<? super T>"),
    ] {
        let field = essential
            .fields
            .iter()
            .find(|field| field.item.name.raw().0 == name.as_bytes())
            .unwrap_or_else(|| panic!("missing field {name}"));
        assert!(field.declaration.as_deref().unwrap().contains(ty));
        assert!(
            field
                .markers
                .iter()
                .any(|marker| marker.contains("projected after descriptor erasure"))
        );
    }

    let generated_source = temp
        .path()
        .join("generated-src/fieldsig/StandaloneFieldBoundary.java");
    fs::create_dir_all(generated_source.parent().unwrap()).unwrap();
    fs::write(&generated_source, &essential.text).unwrap();
    compile_java(&generated_source, &generated_classes, None);
    compile_java(&runner, &generated_classes, Some(&generated_classes));
    compile_java(&reflection, &generated_classes, Some(&generated_classes));

    let mut java_outputs = Vec::new();
    for (classes, main) in [
        (&original_classes, "fieldsig.StandaloneFieldRunner"),
        (&generated_classes, "fieldsig.StandaloneFieldRunner"),
        (
            &original_classes,
            "fieldsig.StandaloneFieldReflectionRunner",
        ),
        (
            &generated_classes,
            "fieldsig.StandaloneFieldReflectionRunner",
        ),
    ] {
        let output = run(
            "java",
            &["-Xverify:all", "-cp", classes.to_str().unwrap(), main],
        );
        assert!(
            output.status.success(),
            "java failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        java_outputs.push(String::from_utf8(output.stdout).unwrap());
    }
    assert_eq!(
        java_outputs[0], java_outputs[1],
        "generic caller behavior changed"
    );
    assert_eq!(java_outputs[2], java_outputs[3], "field reflection changed");
}

#[test]
fn field_signature_refusals_keep_descriptor_for_mismatch_unbound_and_body_use() {
    let temp = TempDir::new();
    let source_dir = temp.path().join("source/fieldsig");
    let classes = temp.path().join("classes");
    fs::create_dir_all(&source_dir).unwrap();
    let standalone = source_dir.join("StandaloneFieldBoundary.java");
    fs::write(&standalone, STANDALONE).unwrap();
    compile_java(&standalone, &classes, None);
    let mismatch = temp.path().join("mismatch.class");
    let result = Command::new("python3")
        .args([
            concat!(env!("CARGO_MANIFEST_DIR"), "/openspec/evidence/java-syntax-2026-09-24/field-generic-signatures/mutate_field_signature.py"),
            classes.join("fieldsig/StandaloneFieldBoundary.class").to_str().unwrap(),
            mismatch.to_str().unwrap(),
            "names",
            "Ljava/util/List<Ljava/lang/String;>;",
            "Ljava/lang/String;",
        ])
        .output()
        .expect("run signature mutator");
    assert!(result.status.success());
    let mut mismatch_budget = Budget::new(limits());
    let mismatch_report = class_source(
        fs::read(mismatch).unwrap(),
        "fieldsig/StandaloneFieldBoundary",
        RecoveryEvidenceRequest::all(),
        &mut mismatch_budget,
    );
    let names = mismatch_report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"names")
        .unwrap();
    assert!(
        names
            .declaration
            .as_deref()
            .unwrap()
            .contains("java.util.List names")
    );
    assert!(
        names
            .markers
            .iter()
            .any(|marker| marker.contains("erasure_mismatch"))
    );

    let unbound = temp.path().join("unbound.class");
    let result = Command::new("python3")
        .args([
            concat!(env!("CARGO_MANIFEST_DIR"), "/openspec/evidence/java-syntax-2026-09-24/field-generic-signatures/mutate_field_signature.py"),
            classes.join("fieldsig/StandaloneFieldBoundary.class").to_str().unwrap(),
            unbound.to_str().unwrap(),
            "current",
            "TT;",
            "TV;",
        ])
        .output()
        .expect("run signature mutator");
    assert!(result.status.success());
    let mut unbound_budget = Budget::new(limits());
    let unbound_report = class_source(
        fs::read(unbound).unwrap(),
        "fieldsig/StandaloneFieldBoundary",
        RecoveryEvidenceRequest::all(),
        &mut unbound_budget,
    );
    let current = unbound_report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"current")
        .unwrap();
    assert!(
        current
            .declaration
            .as_deref()
            .unwrap()
            .contains("java.lang.Object current")
    );
    assert!(
        current
            .markers
            .iter()
            .any(|marker| marker.contains("scope_unproved"))
    );

    let conflict_source = source_dir.join("FieldSignatureConflict.java");
    fs::write(&conflict_source, CONFLICT).unwrap();
    let conflict_classes = temp.path().join("conflict");
    compile_java(&conflict_source, &conflict_classes, None);
    let conflict_original = conflict_classes.join("fieldsig/FieldSignatureConflict.class");
    let conflict_mutated = temp.path().join("FieldSignatureConflict.class");
    let result = Command::new("python3")
        .args([
            concat!(env!("CARGO_MANIFEST_DIR"), "/openspec/evidence/java-syntax-2026-09-24/field-generic-signatures/mutate_field_signature.py"),
            conflict_original.to_str().unwrap(),
            conflict_mutated.to_str().unwrap(),
            "items",
            "Ljava/util/List<Ljava/lang/Object;>;",
            "Ljava/util/List<Ljava/lang/String;>;",
        ])
        .output()
        .expect("mutate conflicting field signature");
    assert!(result.status.success());
    let mut conflict_budget = Budget::new(limits());
    let conflict_report = class_source(
        fs::read(conflict_mutated).unwrap(),
        "fieldsig/FieldSignatureConflict",
        RecoveryEvidenceRequest::all(),
        &mut conflict_budget,
    );
    let items = conflict_report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"items")
        .unwrap();
    assert!(
        items
            .declaration
            .as_deref()
            .unwrap()
            .contains("java.util.List items")
    );
    assert!(
        items
            .markers
            .iter()
            .any(|marker| marker.contains("same-class Fieldref"))
    );

    let conflict_generated_source = temp
        .path()
        .join("conflict-generated-src/fieldsig/FieldSignatureConflict.java");
    fs::create_dir_all(conflict_generated_source.parent().unwrap()).unwrap();
    fs::write(&conflict_generated_source, &conflict_report.text).unwrap();
    let conflict_generated_classes = temp.path().join("conflict-generated");
    compile_java(
        &conflict_generated_source,
        &conflict_generated_classes,
        None,
    );
    let conflict_runner = source_dir.join("FieldSignatureConflictRunner.java");
    fs::write(&conflict_runner, CONFLICT_RUNNER).unwrap();
    compile_java(
        &conflict_runner,
        &conflict_generated_classes,
        Some(&conflict_generated_classes),
    );
    let conflict_run = run(
        "java",
        &[
            "-Xverify:all",
            "-cp",
            conflict_generated_classes.to_str().unwrap(),
            "fieldsig.FieldSignatureConflictRunner",
        ],
    );
    assert!(
        conflict_run.status.success(),
        "fallback source failed at runtime: {}",
        String::from_utf8_lossy(&conflict_run.stderr)
    );
}

#[test]
fn type_use_annotations_and_stopped_requests_do_not_publish_field_candidates() {
    let temp = TempDir::new();
    let source_dir = temp.path().join("source/fieldsig");
    let classes = temp.path().join("compiled");
    fs::create_dir_all(&source_dir).unwrap();
    let source = source_dir.join("TypeUseField.java");
    fs::write(&source, TYPE_USE_FIELD).unwrap();
    compile_java(&source, &classes, None);
    let bytes = class_file(&classes, "TypeUseField");
    let mut annotation_budget = Budget::new(limits());
    let annotation_report = class_source(
        bytes.clone(),
        "fieldsig/TypeUseField",
        RecoveryEvidenceRequest::all(),
        &mut annotation_budget,
    );
    let values = annotation_report
        .fields
        .iter()
        .find(|field| field.item.name.raw().0 == b"values")
        .unwrap();
    assert!(
        values
            .declaration
            .as_deref()
            .unwrap()
            .contains("java.util.List values")
    );
    assert!(
        values
            .markers
            .iter()
            .any(|marker| marker.contains("type-use annotation"))
    );

    let mut open_budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut open_budget)
        .unwrap();
    let request = class_request(&snapshot, "fieldsig/TypeUseField");
    let mut low_limits = limits();
    low_limits.output_bytes = 0;
    let mut low_budget = Budget::new(low_limits);
    let low = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut low_budget,
        )
        .expect("low output budget is reported as a bounded outcome");
    assert!(matches!(low, OperationOutcome::Incomplete(_)));

    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(), cancellation);
    let result = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation is reported as a bounded outcome");
    assert!(matches!(result, OperationOutcome::Incomplete(_)));
}
