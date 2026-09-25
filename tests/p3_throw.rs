//! `recover-throw-statements`: the positive Java 8 throw shapes and their source ownership.
//!
//! The committed input is one complete `ThrowProbe.class`.  `ThrowEffects.java` and
//! `ThrowProbeRunner.java` are source-only JDK comparison inputs, so the ignored runtime check
//! recompiles the whole positive class from the recovered output and keeps the original class's
//! behavior as its oracle.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-throw/v8/ThrowProbe.class");
const PROBE_SOURCE: &str = include_str!("fixtures/p3-throw/ThrowProbe.java");
const EFFECTS_SOURCE: &str = include_str!("fixtures/p3-throw/ThrowEffects.java");
const RUNNER_SOURCE: &str = include_str!("fixtures/p3-throw/ThrowProbeRunner.java");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults to a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed throw fixture opens as a standalone CLASS")
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ThrowProbe"),
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

fn class_source_of(snapshot: &ArtifactSnapshot) -> ClassSourceReport {
    let request = request(snapshot);
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("the all-evidence class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone fixture answers one definition, got {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one standalone fixture answers one definition, got an unfinished selection with {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn class_source_without_evidence(snapshot: &ArtifactSnapshot) -> ClassSourceReport {
    let request = request(snapshot);
    match Engine::new()
        .class_source(slice::from_ref(snapshot), &request, &mut budget())
        .expect("the default class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone fixture answers one definition, got {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one standalone fixture answers one definition, got an unfinished selection with {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn member<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name.as_bytes()
                && method.item.descriptor.raw().0 == descriptor.as_bytes()
        })
        .unwrap_or_else(|| panic!("no member `{name}{descriptor}` in ThrowProbe"))
}

fn recovered<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a RecoveryReport {
    match &member(report, name, descriptor).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}{descriptor}` was not recovered: {other:?}"),
    }
}

fn assert_presented(report: &ClassSourceReport, name: &str, descriptor: &str, snippet: &str) {
    let method = member(report, name, descriptor);
    assert!(
        !method.text.contains("@bytecode"),
        "`{name}{descriptor}` still quotes bytecode:\n{}",
        method.text
    );
    assert!(
        method.text.contains(snippet),
        "`{name}{descriptor}` does not contain `{snippet}`:\n{}",
        method.text
    );
    assert_eq!(
        recovered(report, name, descriptor).content,
        RecoveryContent::ContainsStatements,
        "`{name}{descriptor}` is a statement body"
    );
}

fn assert_source_map(report: &ClassSourceReport, name: &str, descriptor: &str, bcis: &[u32]) {
    let method = member(report, name, descriptor);
    let recovery = recovered(report, name, descriptor);
    assert_eq!(
        recovery.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete,
        "all source-map evidence was requested for `{name}{descriptor}`"
    );
    for bci in bcis {
        let segments = recovery.source_map.of_bci(*bci);
        assert!(
            !segments.is_empty(),
            "`{name}{descriptor}` has no source-map segment for BCI {bci}: {}",
            method.text
        );
        assert!(
            segments
                .iter()
                .any(|segment| segment.origin().primary().method() == Some(&method.item.identity)),
            "BCI {bci} of `{name}{descriptor}` has no segment owned by its real member: {:?}",
            segments
        );
    }
}

#[test]
fn fixture_has_the_complete_positive_throw_surface() {
    assert!(PROBE_SOURCE.contains("throw null;"));
    assert!(PROBE_SOURCE.contains("throw problem;"));
    assert!(PROBE_SOURCE.contains("throw new IllegalStateException(\"fresh\");"));
    assert!(PROBE_SOURCE.contains("throw ThrowEffects.problem();"));
    assert!(PROBE_SOURCE.contains("throw (RuntimeException) problem;"));
    assert!(PROBE_SOURCE.contains("if (first)"));
    assert!(PROBE_SOURCE.contains("catch (RuntimeException caught)"));
    assert!(PROBE_SOURCE.contains("throws IOException"));

    let report = class_source_of(&open(FIXTURE));
    assert_eq!(
        report.methods.len(),
        9,
        "constructor plus eight throw methods"
    );
    for (name, descriptor) in [
        ("<init>", "()V"),
        ("nullValue", "()V"),
        ("parameter", "(Ljava/lang/RuntimeException;)V"),
        ("allocation", "()V"),
        ("call", "()V"),
        ("cast", "(Ljava/lang/Object;)V"),
        (
            "conditional",
            "(ZLjava/lang/RuntimeException;Ljava/lang/RuntimeException;)V",
        ),
        (
            "namedCatch",
            "(Ljava/lang/RuntimeException;)Ljava/lang/RuntimeException;",
        ),
        ("checked", "(Ljava/io/IOException;)V"),
    ] {
        assert!(
            matches!(
                member(&report, name, descriptor).item.body,
                MemberBodyEvidence::CodeAttribute { .. }
            ),
            "`{name}{descriptor}` has a Code attribute"
        );
    }
}

#[test]
fn ordinary_throw_shapes_are_presented_with_their_source_types() {
    let report = class_source_of(&open(FIXTURE));
    assert_presented(&report, "nullValue", "()V", "throw null;");
    assert_presented(
        &report,
        "parameter",
        "(Ljava/lang/RuntimeException;)V",
        "throw arg0;",
    );
    assert_presented(
        &report,
        "allocation",
        "()V",
        "throw new java.lang.IllegalStateException(\"fresh\");",
    );
    assert_presented(&report, "call", "()V", "throw ThrowEffects.problem();");
    assert_presented(
        &report,
        "cast",
        "(Ljava/lang/Object;)V",
        "throw (java.lang.RuntimeException) arg0;",
    );

    let conditional = member(
        &report,
        "conditional",
        "(ZLjava/lang/RuntimeException;Ljava/lang/RuntimeException;)V",
    );
    assert!(conditional.text.contains("throw arg1;"), "{conditional:?}");
    assert!(conditional.text.contains("throw arg2;"), "{conditional:?}");
    assert!(!conditional.text.contains("@bytecode"), "{conditional:?}");

    assert_presented(
        &report,
        "namedCatch",
        "(Ljava/lang/RuntimeException;)Ljava/lang/RuntimeException;",
        "throw arg0;",
    );
    let named_catch = member(
        &report,
        "namedCatch",
        "(Ljava/lang/RuntimeException;)Ljava/lang/RuntimeException;",
    );
    assert!(
        named_catch
            .text
            .contains("catch (java.lang.RuntimeException local1)"),
        "{named_catch:?}"
    );
    assert!(
        named_catch.text.contains("return local1;"),
        "{named_catch:?}"
    );

    assert_presented(
        &report,
        "checked",
        "(Ljava/io/IOException;)V",
        "throw arg0;",
    );
    assert!(
        member(&report, "checked", "(Ljava/io/IOException;)V")
            .text
            .contains("throws java.io.IOException"),
        "the checked declaration is retained"
    );
}

#[test]
fn throw_sources_keep_real_bcis_and_member_identity() {
    let report = class_source_of(&open(FIXTURE));
    assert_source_map(&report, "nullValue", "()V", &[1]);
    assert_source_map(
        &report,
        "parameter",
        "(Ljava/lang/RuntimeException;)V",
        &[1],
    );
    assert_source_map(&report, "allocation", "()V", &[0, 3, 4, 6, 9]);
    assert_source_map(&report, "call", "()V", &[0, 3]);
    assert_source_map(&report, "cast", "(Ljava/lang/Object;)V", &[1, 4]);
    assert_source_map(
        &report,
        "conditional",
        "(ZLjava/lang/RuntimeException;Ljava/lang/RuntimeException;)V",
        &[5, 7],
    );
    assert_source_map(
        &report,
        "namedCatch",
        "(Ljava/lang/RuntimeException;)Ljava/lang/RuntimeException;",
        &[1],
    );
    assert_source_map(&report, "checked", "(Ljava/io/IOException;)V", &[1]);
}

#[test]
fn source_map_is_not_built_by_default() {
    let snapshot = open(FIXTURE);
    let report = class_source_without_evidence(&snapshot);
    let complete = class_source_of(&snapshot);
    assert_eq!(
        report.text, complete.text,
        "evidence selection preserves the artifact"
    );
    assert_eq!(report.methods.len(), 9);
    for method in &report.methods {
        let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
            panic!("the positive fixture method has no recovery report: {method:?}");
        };
        assert!(
            report.source_map.is_empty(),
            "default recovery unexpectedly built source-map evidence for `{}`",
            String::from_utf8_lossy(&method.item.name.raw().0)
        );
        assert_eq!(
            report.evidence.state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::NotRequested
        );
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("jarde-p3-throw-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).expect("create the JDK comparison directory");
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

fn write_sources(dir: &Path, probe: &str) {
    fs::write(dir.join("ThrowProbe.java"), probe).expect("write the probe source");
    fs::write(dir.join("ThrowEffects.java"), EFFECTS_SOURCE).expect("write the helper source");
    fs::write(dir.join("ThrowProbeRunner.java"), RUNNER_SOURCE).expect("write the runner source");
}

fn javac(dir: &Path) {
    let output = std::process::Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir)
        .args([
            "ThrowEffects.java",
            "ThrowProbe.java",
            "ThrowProbeRunner.java",
        ])
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "compiling ThrowProbe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = std::process::Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir)
        .arg("ThrowProbeRunner")
        .current_dir(dir)
        .output()
        .expect("execute the throw fixture runner");
    assert!(
        output.status.success(),
        "the throw fixture runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the throw fixture runner writes UTF-8")
}

#[test]
#[ignore = "requires JDK: compile Engine::class_source output and execute a temporary runner"]
fn recovered_throw_probe_matches_original_runtime() {
    let report = class_source_of(&open(FIXTURE));
    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered = scratch.path().join("recovered");
    fs::create_dir_all(&original).expect("create original comparison directory");
    fs::create_dir_all(&recovered).expect("create recovered comparison directory");

    write_sources(&original, PROBE_SOURCE);
    javac(&original);
    // Execute the exact bytes consumed by Engine, even when the local javac version changes.
    fs::write(original.join("ThrowProbe.class"), FIXTURE)
        .expect("install the frozen original class");
    let original_output = run_runner(&original);

    write_sources(&recovered, &report.text);
    javac(&recovered);
    let recovered_output = run_runner(&recovered);

    for marker in [
        "null=java.lang.NullPointerException:false:0",
        "parameter=java.lang.IllegalArgumentException:true:0",
        "parameterNull=java.lang.NullPointerException:false:0",
        "allocation=java.lang.IllegalStateException:false:0",
        "call=java.lang.IllegalArgumentException:true:1",
        "callFailure=java.lang.IllegalStateException:true:1",
        "cast=java.lang.IllegalArgumentException:true:0",
        "castBad=java.lang.ClassCastException:false:0",
        "castNull=java.lang.NullPointerException:false:0",
        "first=java.lang.IllegalArgumentException:true:0",
        "second=java.lang.IllegalStateException:true:0",
        "checked=java.io.IOException:true:0",
        "caught=true",
    ] {
        assert!(
            original_output.contains(marker),
            "the original runner covers `{marker}`:\n{original_output}"
        );
    }
    assert_eq!(
        recovered_output, original_output,
        "the complete recovered class preserves throw identity, order and calls"
    );
}

// ---------------------------------------------------------------------------
// Throw consumption boundaries: exact complete Code-attribute patches over the frozen positive
// class. Each patch is a source-only JVM input variant; no permanent boundary class is added.
// ---------------------------------------------------------------------------

fn boundary_request(snapshot: &ArtifactSnapshot, class_name: &str) -> ClassSourceRequest {
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

fn boundary_class_source(bytes: &[u8], class_name: &str) -> ClassSourceReport {
    let snapshot = open(bytes);
    let request = boundary_request(&snapshot, class_name);
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("the boundary class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one boundary input answers one definition, got {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one boundary input answers one definition, got an unfinished selection with {} candidates",
            candidates.candidates.len()
        ),
    }
}

fn boundary_member<'a>(
    report: &'a ClassSourceReport,
    class_name: &str,
    name: &str,
    descriptor: &str,
) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name.as_bytes()
                && method.item.descriptor.raw().0 == descriptor.as_bytes()
        })
        .unwrap_or_else(|| panic!("no member `{name}{descriptor}` in {class_name}"))
}

fn boundary_recovered<'a>(
    report: &'a ClassSourceReport,
    class_name: &str,
    name: &str,
    descriptor: &str,
) -> &'a RecoveryReport {
    match &boundary_member(report, class_name, name, descriptor).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}{descriptor}` in {class_name} was not recovered: {other:?}"),
    }
}

fn quoted_bcis(text: &str) -> Vec<u32> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|list| list.split_whitespace())
        .map(|bci| bci.parse().expect("a quoted BCI is an integer"))
        .collect()
}

fn assert_boundary_sources(
    report: &ClassSourceReport,
    class_name: &str,
    name: &str,
    descriptor: &str,
    bcis: &[u32],
) {
    let member = boundary_member(report, class_name, name, descriptor);
    let recovery = boundary_recovered(report, class_name, name, descriptor);
    assert_eq!(
        recovery.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete,
        "all source-map evidence was requested for `{class_name}.{name}`"
    );
    for bci in bcis {
        let segments = recovery.source_map.of_bci(*bci);
        assert!(
            !segments.is_empty(),
            "`{class_name}.{name}` has no source-map segment for BCI {bci}: {}",
            member.text
        );
        assert!(
            segments
                .iter()
                .any(|segment| segment.origin().primary().method() == Some(&member.item.identity)),
            "BCI {bci} of `{class_name}.{name}` has no segment owned by its real member: {:?}",
            segments
        );
    }
}

/// A legal boundary may be presented or partly quoted while the producer/consumer proof is being
/// extended. Presented prefixes need real source coverage; a quoted producer/consumer must retain
/// only the BCIs that the refusal actually defers.
fn assert_boundary_value_or_quote(
    report: &ClassSourceReport,
    class_name: &str,
    name: &str,
    descriptor: &str,
    required_quote: &[u32],
    effect: Option<&str>,
    forbidden_throw: &[&str],
) {
    let text = &boundary_member(report, class_name, name, descriptor).text;
    if text.contains("@bytecode") {
        let recovery = boundary_recovered(report, class_name, name, descriptor);
        assert!(
            recovery.produced(),
            "quoted `{class_name}.{name}` did not produce a refusal artifact: {text}"
        );
        assert_eq!(
            recovery.representation,
            Representation::Mixed,
            "quoted `{class_name}.{name}` is not marked as mixed: {text}"
        );
        assert_eq!(
            recovery.quality,
            Quality::Fallback,
            "quoted `{class_name}.{name}` is not marked as fallback: {text}"
        );
        assert_eq!(
            recovery.syntax_status,
            SyntaxStatus::NotJava,
            "quoted `{class_name}.{name}` claims Java syntax: {text}"
        );
        let quoted = quoted_bcis(text);
        for bci in required_quote {
            assert!(
                quoted.contains(bci),
                "quoted `{class_name}.{name}` lost BCI {bci}: {text}"
            );
            assert!(
                !recovery.text_of_bci(*bci).is_empty(),
                "quoted `{class_name}.{name}` lost its member source at BCI {bci}: {text}"
            );
        }
    } else {
        assert!(
            text.contains("throw "),
            "presented `{class_name}.{name}` has no throw statement: {text}"
        );
        if let Some(effect) = effect {
            assert_eq!(
                text.matches(effect).count(),
                1,
                "presented `{class_name}.{name}` evaluates its producer/effect once: {text}"
            );
        }
    }
    for forbidden in forbidden_throw {
        assert!(
            !text.contains(forbidden),
            "`{class_name}.{name}` uses the wrong throw value `{forbidden}`: {text}"
        );
    }
}

/// Applies one complete method_info replacement. This is deliberately a fixture helper, not a
/// class-file parser: the frozen method header and complete Code attribute identify one method,
/// while the replacement adjusts the attribute and code lengths for the legal variant.
fn patch_complete_method(bytes: &[u8], original: &[u8], patched: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len());
    let mut cursor = 0usize;
    let mut matches = 0usize;
    while cursor < bytes.len() {
        if bytes[cursor..].starts_with(original) {
            output.extend_from_slice(patched);
            cursor += original.len();
            matches += 1;
        } else {
            output.push(bytes[cursor]);
            cursor += 1;
        }
    }
    assert_eq!(matches, 1, "the frozen class has one matching method_info");
    output
}

fn stale_parameter_patch() -> Vec<u8> {
    patch_complete_method(
        FIXTURE,
        &[
            0x00, 0x09, 0x00, 0x1a, 0x00, 0x1b, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, 0x0e,
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x2a, 0xbf, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            0x00, 0x09, 0x00, 0x1a, 0x00, 0x1b, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, 0x10,
            0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x2a, 0x01, 0x4b, 0xbf, 0x00, 0x00,
            0x00, 0x00,
        ],
    )
}

fn call_duplicate_patch() -> Vec<u8> {
    patch_complete_method(
        FIXTURE,
        &[
            0x00, 0x09, 0x00, 0x1d, 0x00, 0x06, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, 0x10,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0xb8, 0x00, 0x0e, 0xbf, 0x00, 0x00,
            0x00, 0x00,
        ],
        &[
            0x00, 0x09, 0x00, 0x1d, 0x00, 0x06, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, 0x12,
            0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0xb8, 0x00, 0x0e, 0x59, 0x57, 0xbf,
            0x00, 0x00, 0x00, 0x00,
        ],
    )
}

fn lower_parameter_patch() -> Vec<u8> {
    patch_complete_method(
        FIXTURE,
        &[
            0x00, 0x09, 0x00, 0x1a, 0x00, 0x1b, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, 0x0e,
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x2a, 0xbf, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            0x00, 0x09, 0x00, 0x1a, 0x00, 0x1b, 0x00, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, 0x12,
            0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0xb8, 0x00, 0x0e, 0x00, 0x2a, 0xbf,
            0x00, 0x00, 0x00, 0x00,
        ],
    )
}

#[test]
fn stale_parameter_throw_keeps_the_old_stack_value_or_quotes_its_boundary() {
    let patched = stale_parameter_patch();
    let report = boundary_class_source(&patched, "ThrowProbe");
    assert_boundary_sources(
        &report,
        "ThrowProbe",
        "parameter",
        "(Ljava/lang/RuntimeException;)V",
        &[0, 1, 2, 3],
    );
    assert_boundary_value_or_quote(
        &report,
        "ThrowProbe",
        "parameter",
        "(Ljava/lang/RuntimeException;)V",
        &[0, 3],
        None,
        &["throw null;", "throw arg1;", "throw replacement;"],
    );
    let text = &boundary_member(
        &report,
        "ThrowProbe",
        "parameter",
        "(Ljava/lang/RuntimeException;)V",
    )
    .text;
    if text.contains("arg0 = null;") {
        assert!(
            !text.contains("throw arg0;"),
            "a rewritten local must not be used as the old stack throw value: {text}"
        );
    }
}

#[test]
fn duplicated_call_result_is_called_once_or_fully_referenced() {
    let patched = call_duplicate_patch();
    let report = boundary_class_source(&patched, "ThrowProbe");
    assert_boundary_sources(&report, "ThrowProbe", "call", "()V", &[0, 3, 4, 5]);
    assert_boundary_value_or_quote(
        &report,
        "ThrowProbe",
        "call",
        "()V",
        &[3],
        Some("problem()"),
        &[],
    );
}

#[test]
fn lower_parameter_stack_value_is_not_an_extra_throw_read() {
    let patched = lower_parameter_patch();
    let report = boundary_class_source(&patched, "ThrowProbe");
    assert_boundary_sources(
        &report,
        "ThrowProbe",
        "parameter",
        "(Ljava/lang/RuntimeException;)V",
        &[0, 3, 4, 5],
    );
    assert_boundary_value_or_quote(
        &report,
        "ThrowProbe",
        "parameter",
        "(Ljava/lang/RuntimeException;)V",
        &[3],
        Some("problem()"),
        &[],
    );
    let text = &boundary_member(
        &report,
        "ThrowProbe",
        "parameter",
        "(Ljava/lang/RuntimeException;)V",
    )
    .text;
    let effect = text
        .find("ThrowEffects.problem();")
        .expect("the lower stack effect remains presented");
    let throw = text
        .find("throw arg0;")
        .expect("the actual athrow operand remains presented");
    assert!(
        effect < throw,
        "the discarded lower value must run first: {text}"
    );
    assert_eq!(
        text.matches("ThrowEffects.problem();").count(),
        1,
        "the lower stack producer is evaluated once: {text}"
    );
    assert_eq!(
        text.matches("throw arg0;").count(),
        1,
        "the athrow operand is rendered once: {text}"
    );
}
