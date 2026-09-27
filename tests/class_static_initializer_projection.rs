//! Ordinary-class static field projection and whole-group refusal boundaries.

use jarde::*;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::slice;

const FIELD_ORDER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-27/em06-field-init/input/em06/FieldOrder.java"
);
const NEGATIVES: &str = r#"
class Missing { static String a; static String b = next(); static String next() { return "b"; } }
class Duplicate { static String a; static { a = next(); a = next(); } static String next() { return "a"; } }
class Extra { static String a; static { tick(); a = next(); } static void tick() {} static String next() { return "a"; } }
class Forward { static String a = Forward.b; static String b = next(); static String next() { return "b"; } }
class Constant { static final int K = 7; static String a = next(); static String next() { return "a"; } }
class Handler { static String a; static { try { a = next(); } catch (RuntimeException ex) { a = "x"; } } static String next() { return "a"; } }
"#;

fn budget() -> Budget {
    task_budget(&[]).expect("bounded default budget")
}

fn compile(directory: &Path, file: &str, source: &str) {
    let path = directory.join(file);
    fs::write(&path, source).expect("write Java source");
    let result = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(directory)
        .arg(&path)
        .output()
        .expect("JDK javac available");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn outcome(
    bytes: &[u8],
    class: &str,
    run_budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("compiled class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
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
    Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            run_budget,
        )
        .expect("one class-source response")
}

fn report(bytes: &[u8], class: &str) -> ClassSourceReport {
    let OperationOutcome::Performed(report) = outcome(bytes, class, &mut budget()) else {
        panic!("compiled class must complete")
    };
    report
}

fn no_projected_fields(report: &ClassSourceReport) {
    assert!(matches!(
        report.initializer_proof,
        ClassSourceInitializerProof::Refused { .. }
    ));
    assert!(report.fields.iter().all(|field| {
        field.item.access_flags & 0x0008 == 0
            || field.item.name.raw().0 == b"K"
            || field
                .declaration
                .as_ref()
                .is_some_and(|text| !text.contains(" = "))
    }));
    assert!(
        report
            .methods
            .iter()
            .any(|method| method.item.identity.name.0 == b"<clinit>")
    );
}

#[test]
fn ordinary_class_without_clinit_has_no_initializer_group() {
    let dir = std::env::temp_dir().join(format!("jarde-em06-absent-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("scratch directory");
    compile(
        &dir,
        "NoInitializer.java",
        "class NoInitializer { static int value; }",
    );
    let bytes = fs::read(dir.join("NoInitializer.class")).expect("compiled class");
    let report = report(&bytes, "NoInitializer");
    assert!(matches!(
        report.initializer_proof,
        ClassSourceInitializerProof::NotApplicable
    ));
    assert!(report.text.contains("static int value;"));
    assert!(!report.text.contains("static {"));
    fs::remove_dir_all(dir).expect("remove scratch directory");
}

#[test]
fn em06_projects_only_the_complete_static_chain() {
    let dir = std::env::temp_dir().join(format!("jarde-em06-positive-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("scratch directory");
    compile(&dir, "FieldOrder.java", FIELD_ORDER);
    let bytes = fs::read(dir.join("em06/FieldOrder.class")).expect("compiled class");
    let report = report(&bytes, "em06/FieldOrder");
    let ClassSourceInitializerProof::Proved { fields } = &report.initializer_proof else {
        panic!("EM-06 static group refused: {:?}", report.initializer_proof)
    };
    assert_eq!(fields.len(), 5);
    assert_eq!(
        report
            .fields
            .iter()
            .map(|field| String::from_utf8_lossy(&field.item.name.raw().0).into_owned())
            .collect::<Vec<_>>(),
        ["trace", "a", "b", "c", "result", "state", "field"]
    );
    let source = &report.text;
    let positions = [" trace = ", " a = ", " b = ", " c = ", " result = "]
        .map(|needle| source.find(needle).expect("static declaration initializer"));
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(!source.contains("static {"));
    assert!(source.contains("field = this.initField()") || source.contains("field = initField()"));
    assert!(!source.contains("int field = "));
    let clinit = report
        .methods
        .iter()
        .find(|method| method.item.identity.name.0 == b"<clinit>")
        .unwrap();
    let ClassSourceOutcome::Recovered { report: body, .. } = &clinit.outcome else {
        panic!("the physical initializer remains recovered")
    };
    assert!(!body.source_map.of_bci(fields[0].write_bci).is_empty());
    assert!(clinit.text.contains("static {"));
    fs::remove_dir_all(dir).expect("remove scratch directory");
}

#[test]
fn incomplete_or_unsafe_static_groups_remain_in_clinit() {
    let dir = std::env::temp_dir().join(format!("jarde-em06-negative-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("scratch directory");
    compile(&dir, "NegativeCases.java", NEGATIVES);
    for class in [
        "Missing",
        "Duplicate",
        "Extra",
        "Forward",
        "Constant",
        "Handler",
    ] {
        let bytes = fs::read(dir.join(format!("{class}.class"))).expect("compiled negative class");
        let report = report(&bytes, class);
        no_projected_fields(&report);
        assert!(report.text.contains("static {"), "{class}: {}", report.text);
    }
    fs::remove_dir_all(dir).expect("remove scratch directory");
}

#[test]
fn output_stop_and_cancellation_do_not_commit_a_prefix() {
    let dir = std::env::temp_dir().join(format!("jarde-em06-stop-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("scratch directory");
    compile(&dir, "FieldOrder.java", FIELD_ORDER);
    let bytes = fs::read(dir.join("em06/FieldOrder.class")).expect("compiled class");
    let complete = report(&bytes, "em06/FieldOrder");
    let mut limits = complete.limits.clone();
    limits.output_bytes = complete.usage.output_bytes.saturating_sub(1);
    let OperationOutcome::Performed(stopped) =
        outcome(&bytes, "em06/FieldOrder", &mut Budget::new(limits))
    else {
        panic!("output stop retains class report")
    };
    assert!(matches!(stopped.execution, ExecutionReport::Partial { .. }));
    assert!(stopped.fields.iter().all(|field| {
        field
            .declaration
            .as_ref()
            .is_none_or(|text| !text.contains(" = "))
    }));
    let mut limits = complete.limits.clone();
    limits.ir_items = complete.usage.ir_items.saturating_sub(1);
    let OperationOutcome::Performed(stopped) =
        outcome(&bytes, "em06/FieldOrder", &mut Budget::new(limits))
    else {
        panic!("proof-stage IR stop retains class report")
    };
    assert!(matches!(stopped.execution, ExecutionReport::Partial { .. }));
    assert!(stopped.fields.iter().all(|field| {
        field
            .declaration
            .as_ref()
            .is_none_or(|text| !text.contains(" = "))
    }));
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = outcome(
        &bytes,
        "em06/FieldOrder",
        &mut Budget::with_cancellation_token(complete.limits, token),
    );
    assert!(matches!(cancelled, OperationOutcome::Incomplete(_)));
    fs::remove_dir_all(dir).expect("remove scratch directory");
}
