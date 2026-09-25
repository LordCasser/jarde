//! `recover-class-literals`: frozen Java 8 class literals, source maps and name-resolution edges.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-class-literals/v8/ClassLiteralProbe.class");
const LOCAL_NAME_FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-class-literals/v8/LocalNameQualifier.class");
const COLLISION_FIXTURE: &[u8] = include_bytes!("fixtures/p3-class-literals/v8/java.class");
const STRING_NAME_FIXTURE: &[u8] = include_bytes!("fixtures/p3-class-literals/v8/String.class");
const PROBE_SOURCE: &str = include_str!("fixtures/p3-class-literals/ClassLiteralProbe.java");
const LOCAL_NAME_SOURCE: &str = include_str!("fixtures/p3-class-literals/LocalNameQualifier.java");
const COLLISION_SOURCE: &str = include_str!("fixtures/p3-class-literals/TypeNameCollision.java");
const STRING_NAME_SOURCE: &str = include_str!("fixtures/p3-class-literals/TypeNameString.java");
const RUNNER_SOURCE: &str = include_str!("fixtures/p3-class-literals/ClassLiteralRunner.java");
const JADX_SOURCE: &str =
    include_str!("../openspec/evidence/java-syntax-2026-09-22/class-literals/jadx.java.txt");
const JARDE_RED_SOURCE: &str =
    include_str!("../openspec/evidence/java-syntax-2026-09-22/class-literals/jarde.java.txt");

const EXPECTED_OUTPUT: &str = "reference:java.lang.String\n\
array:[[Ljava.lang.String;\n\
primitiveArray:[[I\n\
self:true\n\
primitive:int\n\
void:void\n\
argument:java.lang.String:calls=1\n";

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
        class_headers: 10,
        method_bodies: 50,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn budget() -> Budget {
    Budget::new(limits())
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed Java 8 fixture opens as a standalone CLASS")
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

fn class_source(
    snapshot: &ArtifactSnapshot,
    name: &str,
    evidence: RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot, name),
            &evidence,
            budget,
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one committed sample answers one definition, got {} candidate(s)",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one committed sample answers one definition, got an unfinished selection with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

fn method<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in the fixture's method table"))
}

fn recovered<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &method(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` did not produce a recovery report: {other:?}"),
    }
}

fn assert_literal_source_map(report: &RecoveryReport, bci: u32, cp: u16, literal: &str) {
    let literal_segment = report
        .source_map
        .of_bci(bci)
        .into_iter()
        .find(|segment| {
            segment.text(&report.text).contains(literal)
                && segment.origin().primary().bci() == bci
                && segment.origin().primary().cp() == Some(cp)
        })
        .unwrap_or_else(|| {
            panic!(
                "no source-map segment for `{literal}` carries direct BCI {bci} and CP #{cp}: {:?}",
                report.source_map.of_bci(bci)
            )
        });
    assert!(literal_segment.text(&report.text).contains(".class"));
}

#[test]
fn frozen_fixture_covers_the_original_class_pool_and_red_boundary() {
    assert_eq!(FIXTURE.len(), 851);
    assert!(PROBE_SOURCE.contains("return String.class;"));
    assert!(PROBE_SOURCE.contains("return String[][].class;"));
    assert!(PROBE_SOURCE.contains("return int[][].class;"));
    assert!(PROBE_SOURCE.contains("return ClassLiteralProbe.class;"));
    assert!(PROBE_SOURCE.contains("return touch(String.class);"));
    assert!(RUNNER_SOURCE.contains("ClassLiteralProbe.calls"));
    assert_eq!(JARDE_RED_SOURCE.matches("@bytecode").count(), 11);
    for method in ["reference", "array", "primitiveArray", "self", "argument"] {
        let signature = format!("public static java.lang.Class {method}()");
        let start = JARDE_RED_SOURCE
            .find(&signature)
            .unwrap_or_else(|| panic!("frozen RED report has `{method}`"));
        let rest = &JARDE_RED_SOURCE[start..];
        let end = rest
            .find("\n    public static java.lang.Class ")
            .unwrap_or(rest.len());
        let block = &rest[..end];
        assert!(block.contains("@bytecode"), "RED report omitted `{method}`");
        assert!(
            !block.contains("return "),
            "RED report unexpectedly returned in `{method}`"
        );
    }
    assert!(JARDE_RED_SOURCE.contains("return java.lang.Integer.TYPE;"));
    assert!(JARDE_RED_SOURCE.contains("return java.lang.Void.TYPE;"));
}

#[test]
fn all_class_literals_are_recovered_once_with_their_real_constant_pool_origins() {
    let snapshot = open(FIXTURE);
    let defaults = class_source(
        &snapshot,
        "ClassLiteralProbe",
        RecoveryEvidenceRequest::essential(),
        &mut budget(),
    );
    let all = class_source(
        &snapshot,
        "ClassLiteralProbe",
        RecoveryEvidenceRequest::all(),
        &mut budget(),
    );
    assert_eq!(
        defaults.text, all.text,
        "evidence selection cannot change text"
    );
    assert_eq!(defaults.methods.len(), 9);
    assert_eq!(all.methods.len(), 9);
    for name in [
        "<init>",
        "touch",
        "reference",
        "array",
        "primitiveArray",
        "self",
        "primitive",
        "voidType",
        "argument",
    ] {
        let body = recovered(&all, name);
        assert!(body.produced(), "`{name}` did not produce Java: {body:?}");
        assert!(
            !body.text.contains("@bytecode"),
            "`{name}` still quotes instructions:\n{}",
            body.text
        );
        assert_eq!(
            recovered(&defaults, name).text,
            body.text,
            "default and all requests produce the same `{name}` body"
        );
    }

    for (name, expression, cp) in [
        ("reference", "java.lang.String.class", 13),
        ("array", "java.lang.String[][].class", 15),
        ("primitiveArray", "int[][].class", 17),
        ("self", "ClassLiteralProbe.class", 8),
    ] {
        let body = recovered(&all, name);
        assert!(
            body.text.contains(&format!("return {expression};")),
            "{name}: {}",
            body.text
        );
        assert_literal_source_map(body, 0, cp, expression);
    }
    let argument = recovered(&all, "argument");
    assert!(
        argument
            .text
            .contains("return touch(java.lang.String.class);")
    );
    assert_eq!(argument.text.matches("java.lang.String.class").count(), 1);
    assert_literal_source_map(argument, 0, 13, "java.lang.String.class");
    assert!(
        argument
            .source_map
            .of_bci(2)
            .iter()
            .any(|segment| { segment.text(&argument.text).contains("touch") })
    );

    assert!(
        recovered(&all, "primitive")
            .text
            .contains("java.lang.Integer.TYPE")
    );
    assert!(
        recovered(&all, "voidType")
            .text
            .contains("java.lang.Void.TYPE")
    );
    assert_eq!(all.text.matches("@bytecode").count(), 0);
}

#[test]
fn class_literal_name_resolution_refuses_known_type_collisions_but_keeps_local_names() {
    assert!(LOCAL_NAME_SOURCE.contains("int java = 1;"));
    assert!(LOCAL_NAME_SOURCE.contains("value = java.lang.String.class;"));
    let local = class_source(
        &open(LOCAL_NAME_FIXTURE),
        "LocalNameQualifier",
        RecoveryEvidenceRequest::all(),
        &mut budget(),
    );
    let local_body = recovered(&local, "target");
    assert!(local_body.text.contains("java.lang.String.class"));
    assert!(local_body.text.contains("return local1;"));
    assert!(!local_body.text.contains("@bytecode"));

    assert!(STRING_NAME_SOURCE.contains("class String {"));
    let current_string = class_source(
        &open(STRING_NAME_FIXTURE),
        "String",
        RecoveryEvidenceRequest::all(),
        &mut budget(),
    );
    let string_body = recovered(&current_string, "target");
    assert!(string_body.text.contains("return java.lang.String.class;"));
    assert!(!string_body.text.contains("@bytecode"));

    assert!(COLLISION_SOURCE.contains("class java {"));
    assert!(COLLISION_SOURCE.contains("return String.class;"));
    let collision = class_source(
        &open(COLLISION_FIXTURE),
        "java",
        RecoveryEvidenceRequest::all(),
        &mut budget(),
    );
    let body = recovered(&collision, "target");
    assert!(body.text.contains("@bytecode 2 0"), "{}", body.text);
    assert!(body.source_map.of_bci(0).iter().any(|segment| {
        segment
            .origin()
            .derived()
            .iter()
            .any(|origin| origin.bci() == 0)
            && segment.text(&body.text).contains("@bytecode 2 0")
    }));
    assert!(body.source_map.of_bci(2).iter().any(|segment| {
        segment.origin().primary().bci() == 2 && segment.text(&body.text).contains("@bytecode 2 0")
    }));
    assert!(!body.text.contains("java.lang.String.class"));
}

#[test]
fn a_low_class_source_budget_and_cancellation_follow_the_existing_stop_contract() {
    let snapshot = open(FIXTURE);
    let engine = Engine::new();
    let mut low = Budget::new(Limits {
        class_headers: 0,
        ..limits()
    });
    let stopped = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot, "ClassLiteralProbe"),
            &RecoveryEvidenceRequest::all(),
            &mut low,
        )
        .expect("a bounded class-source request reports its stop");
    assert!(
        matches!(stopped, OperationOutcome::Incomplete(_)),
        "a zero header budget cannot publish the class: {stopped:?}"
    );

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(limits(), token);
    let stopped = engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot, "ClassLiteralProbe"),
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation is an operation outcome, not an input error");
    assert!(
        matches!(stopped, OperationOutcome::Incomplete(_)),
        "a cancelled class-source request publishes no complete result: {stopped:?}"
    );
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-class-literals-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("the Java comparison scratch directory is created");
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

fn write(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}

fn compile_and_run(
    root: &Path,
    label: &str,
    source: &str,
    runner: &str,
    main_class: &str,
) -> String {
    let directory = root.join(label);
    fs::create_dir_all(&directory).expect("the source variant directory is created");
    let class_file = directory.join("ClassLiteralProbe.java");
    let runner_file = directory.join("ClassLiteralRunner.java");
    write(&class_file, source);
    write(&runner_file, runner);
    let compiled = Command::new("javac")
        .args(["-source", "8", "-target", "8", "-g:none", "-d"])
        .arg(&directory)
        .arg(&class_file)
        .arg(&runner_file)
        .output()
        .unwrap_or_else(|error| panic!("start javac for {label}: {error}"));
    assert!(
        compiled.status.success(),
        "{label} whole-class javac failed:\n{}\n{}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );
    if label == "original" {
        fs::write(directory.join("ClassLiteralProbe.class"), FIXTURE)
            .expect("the original-source run uses the committed frozen class");
    }
    let executed = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(&directory)
        .arg(main_class)
        .output()
        .unwrap_or_else(|error| panic!("start java for {label}: {error}"));
    assert!(
        executed.status.success(),
        "{label} whole-class JVM run failed:\n{}\n{}",
        String::from_utf8_lossy(&executed.stdout),
        String::from_utf8_lossy(&executed.stderr)
    );
    String::from_utf8(executed.stdout).expect("the runner prints UTF-8")
}

#[test]
#[ignore = "requires javac and java on PATH"]
fn original_jadx_and_jarde_whole_classes_compile_and_have_equal_runtime_results() {
    let snapshot = open(FIXTURE);
    let report = class_source(
        &snapshot,
        "ClassLiteralProbe",
        RecoveryEvidenceRequest::all(),
        &mut budget(),
    );
    assert_eq!(report.text.matches("@bytecode").count(), 0);

    let scratch = Scratch::new();
    let original = compile_and_run(
        scratch.path(),
        "original",
        PROBE_SOURCE,
        RUNNER_SOURCE,
        "ClassLiteralRunner",
    );

    let jadx_runner = format!("package defpackage;\n{RUNNER_SOURCE}");
    let jadx = compile_and_run(
        scratch.path(),
        "jadx",
        JADX_SOURCE,
        &jadx_runner,
        "defpackage.ClassLiteralRunner",
    );
    let jarde = compile_and_run(
        scratch.path(),
        "jarde",
        &report.text,
        RUNNER_SOURCE,
        "ClassLiteralRunner",
    );

    assert_eq!(original, EXPECTED_OUTPUT);
    assert_eq!(
        jadx, original,
        "the complete JADX source preserves class semantics"
    );
    assert_eq!(
        jarde, original,
        "the complete jarde source preserves class semantics"
    );
}
