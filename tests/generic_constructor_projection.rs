use jarde::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

const CLASS: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/generic-constructor-signatures/fixture/GenericConstructor.java"
);
const CALLER: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/generic-constructor-signatures/fixture/GenericConstructorCaller.java"
);
const REFLECT: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/generic-constructor-signatures/fixture/GenericConstructorReflect.java"
);
const TYPE_CHECK: &str = include_str!(
    "../openspec/evidence/java-syntax-2026-09-24/generic-constructor-signatures/fixture/GenericConstructorTypeCheck.java"
);

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-generic-constructor-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
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

fn javac(args: &[&str]) -> Output {
    Command::new("javac").args(args).output().unwrap()
}

fn java(classpath: &Path, name: &str) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(classpath)
        .arg(name)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn write_fixture(dir: &Path, name: &str, source: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, source).unwrap();
    path
}

fn class_report(
    bytes: Vec<u8>,
    evidence: RecoveryEvidenceRequest,
    limits: Option<Limits>,
) -> ClassSourceReport {
    match class_outcome(bytes, evidence, limits) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source result: {other:?}"),
    }
}

fn class_outcome(
    bytes: Vec<u8>,
    evidence: RecoveryEvidenceRequest,
    limits: Option<Limits>,
) -> OperationOutcome<ClassSourceReport> {
    let engine = Engine::new();
    let mut budget = Budget::new(limits.unwrap_or(task_limits(&[]).unwrap()));
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("genericctor/GenericConstructor"),
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
    engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            &evidence,
            &mut budget,
        )
        .unwrap()
}

fn class_bytes(debug: &str, dir: &Path) -> Vec<u8> {
    let class = write_fixture(dir, "GenericConstructor.java", CLASS);
    let caller = write_fixture(dir, "GenericConstructorCaller.java", CALLER);
    let output = javac(&[
        "--release",
        "8",
        "-Xlint:-options",
        debug,
        "-d",
        dir.to_str().unwrap(),
        class.to_str().unwrap(),
        caller.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read(dir.join("genericctor/GenericConstructor.class")).unwrap()
}

#[test]
fn generic_constructor_projects_atomically_for_debug_and_no_debug_classes() {
    for (label, debug) in [("debug", "-g"), ("no-debug", "-g:none")] {
        let original = Scratch::new(&format!("original-{label}"));
        let bytes = class_bytes(debug, original.path());
        let original_caller = java(original.path(), "genericctor.GenericConstructorCaller");
        assert!(original_caller.contains("types=T"), "{original_caller}");
        assert!(original_caller.contains("parameter=T"), "{original_caller}");
        let invalid_source = write_fixture(
            original.path(),
            "GenericConstructorTypeCheck.java",
            TYPE_CHECK,
        );
        let invalid_dir = original.path().join("invalid-original");
        fs::create_dir(&invalid_dir).unwrap();
        let invalid = javac(&[
            "--release",
            "8",
            "-Xlint:-options",
            "-cp",
            original.path().to_str().unwrap(),
            "-d",
            invalid_dir.to_str().unwrap(),
            invalid_source.to_str().unwrap(),
        ]);
        assert!(
            !invalid.status.success(),
            "invalid caller compiled against original"
        );

        let essential = class_report(bytes.clone(), RecoveryEvidenceRequest::essential(), None);
        let all = class_report(bytes.clone(), RecoveryEvidenceRequest::all(), None);
        assert_eq!(essential.text, all.text);
        let declaration = essential
            .text
            .lines()
            .find(|line| line.contains("GenericConstructor(T "))
            .expect(&essential.text);
        assert!(
            declaration.contains("<T extends java.lang.Number>"),
            "{declaration}"
        );
        let constructor = essential
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"<init>")
            .unwrap();
        assert!(
            constructor
                .markers
                .iter()
                .any(|marker| marker.contains("empty-constructor proof")),
            "{:?}",
            constructor.markers
        );
        assert!(
            !constructor
                .markers
                .iter()
                .any(|marker| marker.contains("parameter-return proof")),
            "{:?}",
            constructor.markers
        );

        let projected = Scratch::new(&format!("projected-{label}"));
        let source = write_fixture(projected.path(), "GenericConstructor.java", &essential.text);
        let reflect = write_fixture(projected.path(), "GenericConstructorReflect.java", REFLECT);
        let caller = write_fixture(projected.path(), "GenericConstructorCaller.java", CALLER);
        let rebuild = javac(&[
            "--release",
            "8",
            "-Xlint:-options",
            "-d",
            projected.path().to_str().unwrap(),
            source.to_str().unwrap(),
            reflect.to_str().unwrap(),
            caller.to_str().unwrap(),
        ]);
        assert!(
            rebuild.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&rebuild.stderr),
            essential.text
        );
        assert_eq!(
            java(projected.path(), "genericctor.GenericConstructorReflect").trim(),
            "types=1\nparameter=T"
        );
        let caller_output = java(projected.path(), "genericctor.GenericConstructorCaller");
        assert!(caller_output.contains("types=T"), "{caller_output}");
        assert!(caller_output.contains("parameter=T"), "{caller_output}");
        let invalid_output = projected.path().join("invalid");
        fs::create_dir(&invalid_output).unwrap();
        let invalid = javac(&[
            "--release",
            "8",
            "-Xlint:-options",
            "-cp",
            projected.path().to_str().unwrap(),
            "-d",
            invalid_output.to_str().unwrap(),
            invalid_source.to_str().unwrap(),
        ]);
        assert!(
            !invalid.status.success(),
            "invalid caller compiled against projection"
        );
    }
}

#[test]
fn consuming_or_delegating_constructor_keeps_its_physical_parameter() {
    for (name, source) in [
        (
            "ConstructorUses",
            r#"public class ConstructorUses {
                private Number value;
                public <T extends Number> ConstructorUses(T value) { this.value = value; }
            }"#,
        ),
        (
            "ConstructorDelegates",
            r#"public class ConstructorDelegates {
                public <T extends Number> ConstructorDelegates(T value) { this(value, 0); }
                private ConstructorDelegates(Number value, int ignored) { }
            }"#,
        ),
    ] {
        let dir = Scratch::new(name);
        let java = write_fixture(dir.path(), &format!("{name}.java"), source);
        let build = javac(&[
            "--release",
            "8",
            "-Xlint:-options",
            "-g:none",
            "-d",
            dir.path().to_str().unwrap(),
            java.to_str().unwrap(),
        ]);
        assert!(
            build.status.success(),
            "{}",
            String::from_utf8_lossy(&build.stderr)
        );
        let bytes = fs::read(dir.path().join(format!("{name}.class"))).unwrap();
        let report = report_for(&bytes, name);
        assert!(
            !report.text.contains("<T extends java.lang.Number>"),
            "{}",
            report.text
        );
        assert!(
            report
                .text
                .contains(&format!("{name}(java.lang.Number arg1)")),
            "{}",
            report.text
        );
        assert!(
            report.text.contains("generic Signature projection refused"),
            "{}",
            report.text
        );
    }
}

fn report_for(bytes: &[u8], name: &str) -> ClassSourceReport {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = ClassSourceRequest {
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
    };
    match engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source result: {other:?}"),
    }
}

#[test]
fn same_class_constructor_binding_and_adjacent_generic_method_are_independent() {
    let dir = Scratch::new("same-class-binding");
    let java = write_fixture(
        dir.path(),
        "GenericConstructorBinding.java",
        r#"public class GenericConstructorBinding {
            public <T extends Number> GenericConstructorBinding(T value) { }
            public static GenericConstructorBinding of(Number value) {
                return new GenericConstructorBinding(value);
            }
            public static <U extends Number> U identity(U value) { return value; }
        }"#,
    );
    let build = javac(&[
        "--release",
        "8",
        "-Xlint:-options",
        "-g:none",
        "-d",
        dir.path().to_str().unwrap(),
        java.to_str().unwrap(),
    ]);
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let bytes = fs::read(dir.path().join("GenericConstructorBinding.class")).unwrap();
    let report = report_for(&bytes, "GenericConstructorBinding");
    assert!(
        !report.text.contains("<T extends java.lang.Number>"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("GenericConstructorBinding(java.lang.Number arg1)"),
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("<U extends java.lang.Number> U identity(U arg0)"),
        "{}",
        report.text
    );
}

#[test]
fn body_budget_and_cancellation_never_publish_a_partial_constructor_header() {
    let original = Scratch::new("stops");
    let bytes = class_bytes("-g:none", original.path());
    let baseline = class_report(bytes.clone(), RecoveryEvidenceRequest::essential(), None);
    let mut limits = task_limits(&[]).unwrap();
    limits.output_bytes = baseline.usage.output_bytes.saturating_sub(1);
    let budget = class_outcome(
        bytes.clone(),
        RecoveryEvidenceRequest::essential(),
        Some(limits),
    );
    match budget {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded { .. },
                ..
            }
        )),
        OperationOutcome::Performed(report) => {
            assert!(
                !report.text.contains("<T extends java.lang.Number>"),
                "budget stop published a generic constructor: {}",
                report.text
            );
            assert!(
                report
                    .text
                    .contains("GenericConstructor(java.lang.Number arg1)"),
                "physical constructor declaration missing: {}",
                report.text
            );
            assert!(matches!(
                report.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: BudgetDimension::OutputBytes
                    },
                    ..
                }
            ));
        }
        OperationOutcome::Ambiguous(_) => panic!("one frozen class must bind uniquely"),
    }

    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes), &mut task_budget(&[]).unwrap())
        .unwrap();
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("genericctor/GenericConstructor"),
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
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut cancelled = Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation);
    match engine
        .class_source(std::slice::from_ref(&snapshot), &request, &mut cancelled)
        .unwrap()
    {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => assert!(
            !report.text.contains("<T extends java.lang.Number>"),
            "{}",
            report.text
        ),
        OperationOutcome::Ambiguous(_) => panic!("one frozen class must bind uniquely"),
    }
}
