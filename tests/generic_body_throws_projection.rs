use jarde::*;
use std::{
    fs,
    ops::Deref,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const BODY_THROWS: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/body-generic-throws/fixture/BodyThrows.java"
);
const CALLER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/body-generic-throws/fixture/BodyThrowsCaller.java"
);
const REFLECT: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/body-generic-throws/fixture/BodyThrowsReflect.java"
);

struct Scratch(std::path::PathBuf);

impl Deref for Scratch {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_dir(label: &str) -> Scratch {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "jarde-body-generic-throws-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    Scratch(dir)
}

fn compile(dir: &Path, debug: &str, sources: &[(&str, &str)]) {
    let mut command = Command::new("javac");
    command
        .args(["--release", "8", "-Xlint:-options", debug, "-d"])
        .arg(dir);
    for (name, source) in sources {
        let path = dir.join(format!("{name}.java"));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, source).unwrap();
        command.arg(path);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn request(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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

fn class_source_with_budget(
    bytes: Vec<u8>,
    name: &str,
    evidence: Option<&RecoveryEvidenceRequest>,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .unwrap();
    match evidence {
        Some(evidence) => engine.class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request(&snapshot, name),
            evidence,
            budget,
        ),
        None => engine.class_source(
            std::slice::from_ref(&snapshot),
            &request(&snapshot, name),
            budget,
        ),
    }
    .unwrap()
}

fn class_source(bytes: Vec<u8>, name: &str) -> ClassSourceReport {
    match class_source_with_budget(bytes, name, None, &mut task_budget(&[]).unwrap()) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source result: {other:?}"),
    }
}

fn class_bytes(label: &str, debug: &str, name: &str, source: &str) -> (Scratch, Vec<u8>) {
    let dir = temp_dir(label);
    compile(&dir, debug, &[(name, source)]);
    let package = source
        .lines()
        .find_map(|line| line.trim().strip_prefix("package "))
        .map(|package| package.trim_end_matches(';').replace('.', "/"));
    let class_path = package
        .map(|package| dir.join(package).join(format!("{name}.class")))
        .unwrap_or_else(|| dir.join(format!("{name}.class")));
    let bytes = fs::read(class_path).unwrap();
    (dir, bytes)
}

#[test]
fn empty_body_class_throws_projection_compiles_and_runs_for_both_debug_variants() {
    for (label, debug) in [("debug", "-g"), ("no-debug", "-g:none")] {
        let (_original, bytes) = class_bytes(label, debug, "BodyThrows", BODY_THROWS);
        let report = class_source(bytes.clone(), "bodythrows/BodyThrows");
        assert!(
            report.text.contains("public void run() throws E"),
            "{}",
            report.text
        );
        let run = report
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"run")
            .expect("physical run member is retained");
        assert!(
            run.markers
                .iter()
                .any(|marker| marker.contains("empty-void proof")),
            "{run:?}"
        );
        assert!(
            !report
                .text
                .contains("generic Signature projection refused for `run()V`"),
            "{}",
            report.text
        );

        let all = class_source_with_budget(
            bytes,
            "bodythrows/BodyThrows",
            Some(&RecoveryEvidenceRequest::all()),
            &mut task_budget(&[]).unwrap(),
        );
        let OperationOutcome::Performed(all) = all else {
            panic!("unexpected all-evidence outcome: {all:?}");
        };
        assert_eq!(report.text, all.text);

        let rebuilt = temp_dir(&format!("rebuilt-{label}"));
        let source_path = rebuilt.join("bodythrows/BodyThrows.java");
        fs::create_dir_all(source_path.parent().unwrap()).unwrap();
        fs::write(&source_path, &report.text).unwrap();
        let reflect_path = rebuilt.join("bodythrows/BodyThrowsReflect.java");
        fs::write(&reflect_path, REFLECT).unwrap();
        let caller_path = rebuilt.join("bodythrows/BodyThrowsCaller.java");
        fs::write(&caller_path, CALLER).unwrap();
        let compile = Command::new("javac")
            .args(["--release", "8", "-Xlint:-options", "-d"])
            .arg(&*rebuilt)
            .arg(&source_path)
            .arg(&reflect_path)
            .arg(&caller_path)
            .output()
            .unwrap();
        assert!(
            compile.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&compile.stderr),
            report.text
        );
        let run = Command::new("java")
            .args(["-Xverify:all", "-cp"])
            .arg(&*rebuilt)
            .arg("bodythrows.BodyThrowsCaller")
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(String::from_utf8(run.stdout).unwrap(), "throws=E\n");
    }
}

#[test]
fn body_effects_and_object_method_names_keep_physical_throws() {
    let effects = r#"
public class BodyThrowsEffect<E extends Exception> {
    static int state;
    public void run() throws E { state++; }
}
"#;
    let object_name = r#"
public class BodyThrowsFinalize<E extends Exception> {
    protected void finalize() throws E { }
}
"#;
    for (label, name, source, target) in [
        ("effect", "BodyThrowsEffect", effects, "BodyThrowsEffect"),
        (
            "object-name",
            "BodyThrowsFinalize",
            object_name,
            "BodyThrowsFinalize",
        ),
    ] {
        let (_dir, bytes) = class_bytes(label, "-g:none", name, source);
        let report = class_source(bytes, target);
        let method_name = if name == "BodyThrowsEffect" {
            "run()V"
        } else {
            "finalize()V"
        };
        assert!(
            report.text.contains("throws java.lang.Exception"),
            "{}",
            report.text
        );
        assert!(!report.text.contains("throws E"), "{}", report.text);
        assert!(
            report.text.contains(&format!(
                "generic Signature projection refused for `{method_name}`"
            )),
            "{}",
            report.text
        );
    }
}

#[test]
fn body_budget_stop_and_cancellation_do_not_publish_a_generic_throws_clause() {
    let (_dir, bytes) = class_bytes("budget", "-g:none", "BodyThrows", BODY_THROWS);
    let mut limits = task_limits(&[]).unwrap();
    limits.method_bodies = 1;
    let outcome = class_source_with_budget(
        bytes.clone(),
        "bodythrows/BodyThrows",
        None,
        &mut Budget::new(limits),
    );
    let report = match outcome {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Incomplete(selection) => {
            panic!("unexpected selection stop: {selection:?}")
        }
        OperationOutcome::Ambiguous(_) => panic!("frozen class must bind uniquely"),
    };
    assert!(!report.text.contains("run() throws E"), "{}", report.text);
    assert!(report.text.contains("run() throws java.lang.Exception"));
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::MethodBodies
            },
            ..
        }
    ));

    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .unwrap();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let outcome = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request(&snapshot, "bodythrows/BodyThrows"),
            &mut Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation),
        )
        .unwrap();
    match outcome {
        OperationOutcome::Incomplete(selection) => {
            assert!(matches!(
                selection.execution,
                ExecutionReport::Cancelled { .. }
            ));
        }
        OperationOutcome::Performed(report) => {
            assert!(!report.text.contains("run() throws E"), "{}", report.text);
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
        }
        OperationOutcome::Ambiguous(_) => panic!("frozen class must bind uniquely"),
    }
}
