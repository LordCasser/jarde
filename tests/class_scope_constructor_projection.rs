//! Whole-class acceptance of class-scope generic constructor parameters.
//!
//! The input side is the independently frozen javac 8/javac 23 class file, never its JAR or
//! original classes on a candidate class path. The external reflection Driver checks
//! `TypeVariable.getGenericDeclaration()` identity as well as observable field values, so a
//! constructor variable named `T` cannot masquerade as the class variable named `T`.

use jarde::*;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const DRIVER_SOURCE: &str = include_str!(
    "../openspec/changes/recover-class-scope-constructor-parameters/evidence/ConstructorDriver.java"
);
macro_rules! frozen {
    ($class:literal, $jdk:literal, $debug:literal) => {
        FrozenClass {
            class: $class,
            jdk: $jdk,
            debug: $debug,
            bytes: include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/",
                "openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen/",
                $jdk,
                "/",
                $debug,
                "/",
                $class,
                "/",
                "classes/",
                $class,
                ".class"
            )),
            original_output: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/",
                "openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen/",
                $jdk,
                "/",
                $debug,
                "/",
                $class,
                "/reflection-run.stdout"
            )),
        }
    };
}

macro_rules! four_legs {
    ($class:literal) => {
        [
            frozen!($class, "corretto8", "debug"),
            frozen!($class, "corretto8", "nodebug"),
            frozen!($class, "openjdk23", "debug"),
            frozen!($class, "openjdk23", "nodebug"),
        ]
    };
}

#[derive(Clone, Copy)]
struct FrozenClass {
    class: &'static str,
    jdk: &'static str,
    debug: &'static str,
    bytes: &'static [u8],
    original_output: &'static str,
}

const POSITIVES: [[FrozenClass; 4]; 9] = [
    four_legs!("Hold"),
    four_legs!("BoundHold"),
    four_legs!("ArrayHold"),
    four_legs!("WideHold"),
    four_legs!("WideStoredHold"),
    four_legs!("RepeatedHold"),
    four_legs!("MultiHold"),
    four_legs!("UnusedHold"),
    four_legs!("RawNewHold"),
];

const PARTIAL_PEER_NEW: [FrozenClass; 4] = four_legs!("PeerNewHold");

const CALL_CONSUMER_POSITIVES: [[FrozenClass; 4]; 2] =
    [four_legs!("CallHold"), four_legs!("ExceptionHold")];

const REFUSALS: [[FrozenClass; 4]; 7] = [
    four_legs!("RewrittenHold"),
    four_legs!("PhiHold"),
    four_legs!("ThisDelegateHold"),
    four_legs!("ParentCtorHold"),
    four_legs!("ObjectHold"),
    four_legs!("ErasedCastHold"),
    four_legs!("ShadowHold"),
];

const CROSS_HOLD: [FrozenClass; 2] = [
    frozen!("CrossHold", "corretto8", "nodebug"),
    frozen!("CrossHold", "openjdk23", "nodebug"),
];

static NEXT_TEMP_DIR: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-class-scope-ctor-{label}-{}-{nonce}-{}",
            std::process::id(),
            NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("a private test directory is created");
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

fn limits() -> Limits {
    task_limits(&[]).expect("the task defaults are bounded")
}

fn request(snapshot: &ArtifactSnapshot, class: &str) -> ClassSourceRequest {
    ClassSourceRequest {
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
    }
}

fn class_source_outcome(
    frozen: FrozenClass,
    evidence: &RecoveryEvidenceRequest,
    run_budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    let snapshot = Engine::new()
        .open(
            ArtifactInput::bytes(frozen.bytes.to_vec()),
            &mut Budget::new(limits()),
        )
        .expect("the frozen standalone class opens");
    Engine::new()
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request(&snapshot, frozen.class),
            evidence,
            run_budget,
        )
        .expect("the frozen class answers a class-source request")
}

fn report(frozen: FrozenClass, evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    match class_source_outcome(frozen, evidence, &mut Budget::new(limits())) {
        OperationOutcome::Performed(report) => report,
        other => panic!(
            "{} ({}/{}) answers one class-source request: {other:?}",
            frozen.class, frozen.jdk, frozen.debug
        ),
    }
}

fn assert_source_header(text: &str, class: &str) {
    assert!(
        !text.trim().is_empty(),
        "{class} must have nonempty class source"
    );
    assert!(
        text.starts_with(&format!("// jarde: presentation of `{class}`")),
        "{class} must carry its class-source header:\n{text}"
    );
    assert!(
        text.contains(&format!("class {class}")),
        "{class} must carry its declared class:\n{text}"
    );
}

fn rows<'a>(output: &'a str, prefix: &str) -> Vec<&'a str> {
    output
        .lines()
        .filter(|line| line.starts_with(prefix))
        .collect()
}

fn assert_full_runtime_match(actual: &str, expected: &str, label: &str) {
    let actual_behavior = rows(actual, "BEHAVIOR|");
    let expected_behavior = rows(expected, "BEHAVIOR|");
    assert!(
        !expected_behavior.is_empty(),
        "{label} has frozen behavior rows"
    );
    assert_eq!(actual_behavior, expected_behavior, "{label} behavior");

    let actual_reflection = rows(actual, "REFLECT|");
    let expected_reflection = rows(expected, "REFLECT|");
    assert!(
        !expected_reflection.is_empty(),
        "{label} has frozen reflection rows"
    );
    assert_eq!(
        actual_reflection, expected_reflection,
        "{label} generic reflection"
    );
}

fn compile_driver(driver_dir: &Path) {
    fs::create_dir_all(driver_dir).expect("the external Driver directory is created");
    let driver = driver_dir.join("ConstructorDriver.java");
    fs::write(&driver, DRIVER_SOURCE).expect("the external Driver source is written");
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(driver_dir)
        .arg(&driver)
        .output()
        .expect("javac compiles the external reflection Driver");
    assert!(
        output.status.success(),
        "javac rejected ConstructorDriver:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn javac_source(source: &Path, classes: &Path, driver: &Path) -> Output {
    fs::create_dir_all(classes).expect("the candidate class directory is created");
    let class_path = std::env::join_paths([driver.as_os_str()])
        .expect("the external Driver path is a valid class path");
    Command::new("javac")
        .args(["--release", "8", "-g:none", "-classpath"])
        .arg(class_path)
        .arg("-d")
        .arg(classes)
        .arg(source)
        .output()
        .expect("javac compiles the complete candidate class")
}

fn run_candidate(root: &Path, driver: &Path, class: &str, text: &str, label: &str) -> String {
    let directory = root.join(label);
    fs::create_dir_all(&directory).expect("the isolated candidate directory is created");
    let source = directory.join(format!("{class}.java"));
    let classes = directory.join("classes");
    fs::write(&source, text).expect("the complete class source is written");
    let compiled = javac_source(&source, &classes, driver);
    assert!(
        compiled.status.success(),
        "javac --release 8 rejected {class} ({label}):\n{}\n{text}",
        String::from_utf8_lossy(&compiled.stderr)
    );

    // The original class/JAR is deliberately absent: only the newly compiled class and the
    // independent reflection Driver can satisfy this run.
    let class_path = std::env::join_paths([classes.as_os_str(), driver.as_os_str()])
        .expect("the generated class and external Driver form a valid class path");
    let output = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(class_path)
        .arg("ConstructorDriver")
        .arg(class)
        .output()
        .expect("the candidate class runs under full JVM verification");
    assert!(
        output.status.success(),
        "java -Xverify:all failed for {class} ({label}):\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the reflection Driver emits UTF-8")
}

fn assert_constructor_refused(report: &ClassSourceReport, class: &str, needs_refusal_marker: bool) {
    let constructors = report
        .methods
        .iter()
        .filter(|method| method.item.name.raw().0 == b"<init>")
        .collect::<Vec<_>>();
    assert!(
        !constructors.is_empty(),
        "{class} keeps its physical constructor"
    );
    for method in constructors {
        let declaration = method
            .declaration
            .as_deref()
            .unwrap_or_else(|| panic!("{class} constructor keeps its physical spelling"));
        assert!(
            declaration.starts_with(&format!("public {class}(")),
            "{class} constructor keeps its access and physical declaration: {declaration}"
        );
        assert!(
            declaration.contains("java.lang.Object"),
            "{class} keeps its erased Object parameter: {declaration}"
        );
        assert!(
            !declaration.contains("(T ") && !declaration.contains("<T>"),
            "{class} must not invent a constructor T binder: {declaration}"
        );
        if needs_refusal_marker {
            assert!(
                method.markers.iter().any(|marker| {
                    marker.contains("generic Signature projection refused")
                        || marker.contains("generic_constructor_source_unproved")
                        || marker.contains("jvm_signature_scope_unproved")
                }),
                "{class} retains the constructor Signature refusal: {:?}",
                method.markers
            );
        }
    }
}

#[test]
fn class_scope_constructor_positives_recompile_and_preserve_behavior_and_binders() {
    let scratch = Scratch::new("positives");
    let driver = scratch.path().join("driver");
    compile_driver(&driver);

    for frozen in POSITIVES.iter().flatten().copied() {
        let essential = report(frozen, &RecoveryEvidenceRequest::essential());
        let all = report(frozen, &RecoveryEvidenceRequest::all());
        assert_eq!(
            essential.text, all.text,
            "{} ({}/{})",
            frozen.class, frozen.jdk, frozen.debug
        );
        assert_source_header(&all.text, frozen.class);
        assert!(
            !all.fields.is_empty(),
            "{} retains its fields",
            frozen.class
        );

        let label = format!("{}-{}-{}", frozen.class, frozen.jdk, frozen.debug);
        let actual = run_candidate(scratch.path(), &driver, frozen.class, &all.text, &label);
        assert_full_runtime_match(&actual, frozen.original_output, &label);
        let formal_counts = rows(&actual, "REFLECT|ctor[")
            .into_iter()
            .filter(|line| line.contains(".formalCount="))
            .collect::<Vec<_>>();
        assert!(
            !formal_counts.is_empty(),
            "{} reflects its constructor formals",
            frozen.class
        );
        assert!(
            formal_counts
                .iter()
                .all(|line| line.contains("typeVariableCount=0")),
            "{} preserves zero constructor-formal type variables:\n{actual}",
            frozen.class
        );
    }
}

#[test]
fn same_class_call_consumers_restore_class_binders_in_both_evidence_flavors() {
    let scratch = Scratch::new("call-consumer-positives");
    let driver = scratch.path().join("driver");
    compile_driver(&driver);

    for frozen in CALL_CONSUMER_POSITIVES.iter().flatten().copied() {
        let essential = report(frozen, &RecoveryEvidenceRequest::essential());
        let all = report(frozen, &RecoveryEvidenceRequest::all());
        assert_eq!(
            essential.text, all.text,
            "{} ({}/{}) essential/all source remains stable",
            frozen.class, frozen.jdk, frozen.debug
        );
        assert_source_header(&essential.text, frozen.class);

        let method_name = match frozen.class {
            "CallHold" => "identity",
            "ExceptionHold" => "maybe",
            _ => unreachable!("the call-consumer fixture list is explicit"),
        };
        assert!(
            essential.text.contains(&format!(" T {method_name}(T ")),
            "{} restores both the callee parameter and return to the class T: {}",
            frozen.class,
            essential.text
        );
        for evidence in [(&essential, "essential"), (&all, "all")] {
            assert!(
                evidence
                    .0
                    .fields
                    .iter()
                    .any(|field| field.item.name.raw().0 == b"v"),
                "{} reports the physical v field for the {}/{} evidence flavor",
                frozen.class,
                frozen.jdk,
                evidence.1
            );
            let label = format!(
                "{}-{}-{}-{}",
                frozen.class, frozen.jdk, frozen.debug, evidence.1
            );
            let actual = run_candidate(
                scratch.path(),
                &driver,
                frozen.class,
                &evidence.0.text,
                &label,
            );
            assert_eq!(
                rows(&actual, "BEHAVIOR|"),
                rows(frozen.original_output, "BEHAVIOR|"),
                "{label} preserves the original constructor behavior"
            );
            assert_eq!(
                rows(&actual, "REFLECT|ctor["),
                rows(frozen.original_output, "REFLECT|ctor["),
                "{label} preserves the class-bound constructor parameter"
            );
            assert_eq!(
                rows(&actual, "REFLECT|class"),
                rows(frozen.original_output, "REFLECT|class"),
                "{label} preserves the class type-variable declaration"
            );
            assert!(
                rows(&actual, "REFLECT|ctor[")
                    .iter()
                    .any(|line| line.contains(".param[0]=T;binder=class")),
                "{label} constructor T is owned by the class, not a constructor binder"
            );
        }
    }
}

#[test]
fn peer_new_restores_only_the_two_argument_constructor_binder() {
    let scratch = Scratch::new("peer-new-partial");
    let driver = scratch.path().join("driver");
    compile_driver(&driver);

    for frozen in PARTIAL_PEER_NEW {
        let all = report(frozen, &RecoveryEvidenceRequest::all());
        assert_eq!(
            report(frozen, &RecoveryEvidenceRequest::essential()).text,
            all.text,
            "PeerNewHold essential/all source agree for {}/{}",
            frozen.jdk,
            frozen.debug
        );
        assert_source_header(&all.text, frozen.class);
        assert!(all.text.contains("public java.lang.Object v;"));

        let label = format!("PeerNewHold-{}-{}", frozen.jdk, frozen.debug);
        let actual = run_candidate(scratch.path(), &driver, frozen.class, &all.text, &label);
        assert_eq!(
            rows(&actual, "BEHAVIOR|"),
            rows(frozen.original_output, "BEHAVIOR|"),
            "{label} behavior"
        );
        let constructor_rows = rows(&actual, "REFLECT|ctor[");
        let one = constructor_rows
            .iter()
            .find(|line| line.contains(".formalCount=1;"))
            .expect("PeerNewHold retains its one-argument constructor");
        let two = constructor_rows
            .iter()
            .find(|line| line.contains(".formalCount=2;"))
            .expect("PeerNewHold retains its two-argument constructor");
        let one_prefix = one.split(".formalCount=").next().unwrap();
        let two_prefix = two.split(".formalCount=").next().unwrap();
        let original_constructors = rows(frozen.original_output, "REFLECT|ctor[");
        let original_one = format!("{one_prefix}.param[0]=T;binder=class");
        let original_two = format!("{two_prefix}.param[0]=T;binder=class");
        assert!(original_constructors.contains(&original_one.as_str()));
        assert!(original_constructors.contains(&original_two.as_str()));
        assert_eq!(
            rows(frozen.original_output, "REFLECT|field["),
            vec!["REFLECT|field[v]=T;binder=class"],
            "the original field shares the class binder but has an unsafe Object writer"
        );
        let erased_one = format!("{one_prefix}.param[0]=java.lang.Object;binder=non-variable");
        let class_bound_two = format!("{two_prefix}.param[0]=T;binder=class");
        let boolean_two = format!("{two_prefix}.param[1]=boolean;binder=non-variable");
        assert!(
            constructor_rows.contains(&erased_one.as_str()),
            "the allocating constructor stays erased: {actual}"
        );
        assert!(
            constructor_rows.contains(&class_bound_two.as_str()),
            "the directly storing overload uses the class binder: {actual}"
        );
        assert!(constructor_rows.contains(&boolean_two.as_str()));
        assert_eq!(
            rows(&actual, "REFLECT|field["),
            vec!["REFLECT|field[v]=java.lang.Object;binder=non-variable"],
            "the writer from the one-argument constructor keeps the field erased"
        );
        assert_ne!(
            rows(&actual, "REFLECT|"),
            rows(frozen.original_output, "REFLECT|"),
            "partial constructor recovery is not full reflection equality"
        );
    }
}

#[test]
fn cross_hold_restores_u_constructor_binder_and_keeps_t_field_erased() {
    let scratch = Scratch::new("cross");
    let driver = scratch.path().join("driver");
    compile_driver(&driver);

    for frozen in CROSS_HOLD {
        let all = report(frozen, &RecoveryEvidenceRequest::all());
        assert_source_header(&all.text, frozen.class);
        assert!(
            all.text.contains("public java.lang.Object v;"),
            "CrossHold keeps the T field erased based on the published U constructor parameter:\n{}",
            all.text
        );
        let constructor = all
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"<init>")
            .and_then(|method| method.declaration.as_deref())
            .expect("CrossHold keeps its constructor");
        assert!(
            constructor.contains("(U "),
            "CrossHold publishes the class-scope U constructor parameter: {constructor}"
        );

        let label = format!("CrossHold-{}-{}", frozen.jdk, frozen.debug);
        let actual = run_candidate(scratch.path(), &driver, frozen.class, &all.text, &label);
        assert_eq!(
            rows(&actual, "BEHAVIOR|"),
            rows(frozen.original_output, "BEHAVIOR|")
        );
        assert_eq!(
            rows(&actual, "REFLECT|ctor["),
            rows(frozen.original_output, "REFLECT|ctor[")
        );
        assert_eq!(
            rows(&actual, "REFLECT|class"),
            rows(frozen.original_output, "REFLECT|class")
        );
        let field_rows = rows(&actual, "REFLECT|field[");
        assert_eq!(
            field_rows,
            vec!["REFLECT|field[v]=java.lang.Object;binder=non-variable"]
        );
        assert_ne!(field_rows, rows(frozen.original_output, "REFLECT|field["));
    }
}

#[test]
fn this_delegate_keeps_physical_signature_and_recompiles_with_original_behavior() {
    let scratch = Scratch::new("this-delegate");
    let driver = scratch.path().join("driver");
    compile_driver(&driver);

    for frozen in four_legs!("ThisDelegateHold") {
        let all = report(frozen, &RecoveryEvidenceRequest::all());
        assert_source_header(&all.text, frozen.class);
        assert_constructor_refused(&all, frozen.class, true);

        let label = format!("ThisDelegateHold-{}-{}", frozen.jdk, frozen.debug);
        let actual = run_candidate(scratch.path(), &driver, frozen.class, &all.text, &label);
        assert_eq!(
            rows(&actual, "BEHAVIOR|"),
            rows(frozen.original_output, "BEHAVIOR|"),
            "{label} preserves behavior after compiling the whole class"
        );
        assert_ne!(
            rows(&actual, "REFLECT|ctor["),
            rows(frozen.original_output, "REFLECT|ctor["),
            "{label} keeps the constructor's physical erased parameters"
        );
    }
}

#[test]
fn unproved_constructor_shapes_keep_physical_parameters_and_refusals() {
    for frozen in REFUSALS.iter().flatten().copied() {
        let all = report(frozen, &RecoveryEvidenceRequest::all());
        assert_source_header(&all.text, frozen.class);
        assert!(
            all.fields
                .iter()
                .any(|field| field.item.name.raw().0 == b"v"),
            "{} retains the physical v field",
            frozen.class
        );
        let signature_exists = !matches!(frozen.class, "ObjectHold" | "ErasedCastHold");
        assert_constructor_refused(&all, frozen.class, signature_exists);

        let original_constructor_parameters = rows(frozen.original_output, "REFLECT|ctor[")
            .into_iter()
            .filter(|line| line.contains(".param["))
            .collect::<Vec<_>>();
        assert!(
            !original_constructor_parameters.is_empty(),
            "{} has frozen physical constructor-parameter evidence",
            frozen.class
        );
        if frozen.class == "ShadowHold" {
            assert!(
                original_constructor_parameters
                    .iter()
                    .any(|line| line.contains("=T;binder=constructor")),
                "ShadowHold's constructor T belongs to the constructor declaration: {:?}",
                original_constructor_parameters
            );
            assert!(
                rows(frozen.original_output, "REFLECT|field[")
                    .iter()
                    .any(|line| line.contains("=T;binder=class")),
                "ShadowHold's field T belongs to the class declaration"
            );
        }
        if matches!(frozen.class, "ObjectHold" | "ErasedCastHold") {
            assert!(
                original_constructor_parameters
                    .iter()
                    .all(|line| line.contains("java.lang.Object;binder=non-variable")),
                "{} has no constructor Signature from which T could be invented: {:?}",
                frozen.class,
                original_constructor_parameters
            );
        }
    }
}

fn assert_output_budget_stop(execution: &ExecutionReport, cap: u64) {
    assert!(
        matches!(
            execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::OutputBytes
                },
                ..
            }
        ),
        "the one-byte-short output limit is reported as a stop: {execution:?}"
    );
    let usage = match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    };
    assert!(
        usage.output_bytes <= cap,
        "reported output usage stays within its cap"
    );
}

fn stopped_for_output_budget(outcome: OperationOutcome<ClassSourceReport>, cap: u64) {
    match outcome {
        OperationOutcome::Performed(report) => {
            assert_output_budget_stop(&report.execution, cap);
            assert!(report.usage.output_bytes <= cap);
        }
        OperationOutcome::Incomplete(candidates) => {
            assert_output_budget_stop(&candidates.execution, cap);
        }
        OperationOutcome::Ambiguous(candidates) => {
            panic!("one frozen class is not ambiguous: {candidates:?}")
        }
    }
}

#[test]
fn positive_constructor_output_budget_and_early_cancellation_stop_cleanly() {
    let frozen = frozen!("Hold", "corretto8", "debug");
    let complete = report(frozen, &RecoveryEvidenceRequest::all());
    let cap = complete
        .usage
        .output_bytes
        .checked_sub(1)
        .expect("the complete class-source output has a positive size");
    let output_limit = BudgetOverride::new("output_bytes", cap)
        .expect("output_bytes is an overridable budget dimension");
    let bounded_limits = task_limits(&[output_limit]).expect("the one-byte-short budget is valid");
    let bounded = class_source_outcome(
        frozen,
        &RecoveryEvidenceRequest::all(),
        &mut Budget::new(bounded_limits),
    );
    stopped_for_output_budget(bounded, cap);

    let snapshot = Engine::new()
        .open(
            ArtifactInput::bytes(frozen.bytes.to_vec()),
            &mut Budget::new(limits()),
        )
        .expect("the frozen class opens before the cancellation is signalled");
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = Engine::new()
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request(&snapshot, frozen.class),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::with_cancellation_token(limits(), token),
        )
        .expect("the cancelled class-source request returns its stop state");
    let execution = match cancelled {
        OperationOutcome::Performed(report) => report.execution,
        OperationOutcome::Incomplete(candidates) => candidates.execution,
        OperationOutcome::Ambiguous(candidates) => {
            panic!("one frozen class is not ambiguous: {candidates:?}")
        }
    };
    assert!(
        matches!(&execution, ExecutionReport::Cancelled { .. }),
        "early cancellation remains visible in the execution report: {execution:?}"
    );
}
