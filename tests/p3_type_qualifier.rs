//! `preserve-type-qualifier-bindings`: generated local names must not capture the first
//! component of a static owner.  The permanent class has two default-package owners named
//! `arg0` and `arg0_2`, plus a same-class static-call control.  Its helper and runner sources are
//! compiled only by the ignored whole-class JVM comparison.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-type-qualifier/v8/TypeQualifierProbe.class");
const PROBE_SOURCE: &str = include_str!("fixtures/p3-type-qualifier/TypeQualifierProbe.java");
const ARG0_SOURCE: &str = include_str!("fixtures/p3-type-qualifier/arg0.java");
const ARG0_2_SOURCE: &str = include_str!("fixtures/p3-type-qualifier/arg0_2.java");
const OTHER_SOURCE: &str = include_str!("fixtures/p3-type-qualifier/ShadowOther.java");
const RUNNER_SOURCE: &str = include_str!("fixtures/p3-type-qualifier/TypeQualifierRunner.java");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed type-qualifier fixture opens as a standalone CLASS")
}

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("TypeQualifierProbe"),
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
            slice::from_ref(snapshot),
            &request,
            &evidence,
            &mut budget(),
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
        other => panic!("`{name}` was not recovered: {other:?}"),
    }
}

fn all_recovered(report: &ClassSourceReport) {
    assert_eq!(
        report.methods.len(),
        6,
        "constructor plus five code methods"
    );
    for name in ["<init>", "ownPick", "ownCall", "invoke", "read", "write"] {
        let body = recovered(report, name);
        assert!(body.produced(), "`{name}` produced no body: {body:?}");
        assert_eq!(body.representation, Representation::Java, "`{name}`");
        assert_eq!(body.quality, Quality::Structured, "`{name}`");
        assert_eq!(
            body.content,
            RecoveryContent::ContainsStatements,
            "`{name}`"
        );
        assert!(
            !body.text.contains("@bytecode"),
            "`{name}` remains quoted:\n{}",
            body.text
        );
    }
}

fn assert_all_source_map(report: &ClassSourceReport) {
    for name in ["<init>", "ownPick", "ownCall", "invoke", "read", "write"] {
        let body = recovered(report, name);
        assert_eq!(
            body.evidence.requested(),
            &RecoveryEvidenceRequest::all(),
            "all `{name}` must echo the complete source request"
        );
        assert_eq!(
            body.evidence.state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::Complete,
            "all `{name}` source-map status"
        );
        assert!(
            !body.source_map.is_empty(),
            "all `{name}` has no source-map segments"
        );
        for segment in body.source_map.segments() {
            let origin = segment.origin();
            assert!(!origin.bcis().is_empty(), "segment has no BCI: {origin:?}");
            assert!(
                origin.primary().member().is_some(),
                "segment has no member: {origin:?}"
            );
            for derived in origin.derived() {
                assert!(
                    derived.member().is_some(),
                    "derived origin has no member: {origin:?}"
                );
            }
        }
    }
}

#[test]
fn all_fixture_methods_are_structured_with_sources() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);
    assert_all_source_map(&report);
}

#[test]
fn static_owners_survive_generated_parameter_and_suffix_names() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());

    let own_call = &recovered(&report, "ownCall").text;
    assert!(
        own_call.contains("return ownPick(1)"),
        "same-class call changed:\n{own_call}"
    );
    let own_pick = &report.text;
    assert!(
        own_pick.contains("ownPick(int arg0)"),
        "an unaffected method was renamed by a class-wide owner scan:\n{own_pick}"
    );

    let invoke = &recovered(&report, "invoke").text;
    assert!(
        invoke.contains("arg0.pick(1)"),
        "external owner disappeared:\n{invoke}"
    );
    assert!(
        invoke.contains("arg0_2.pick(1)"),
        "second owner disappeared:\n{invoke}"
    );
    assert!(
        !report.text.contains("invoke(ShadowOther arg0)"),
        "parameter captured arg0:\n{invoke}"
    );
    assert!(
        !report.text.contains("invoke(ShadowOther arg0_2)"),
        "parameter captured arg0_2:\n{invoke}"
    );

    let read = &recovered(&report, "read").text;
    assert!(
        read.contains("arg0.value + arg0_2.value"),
        "static fields changed:\n{read}"
    );
    assert!(
        !report.text.contains("read(ShadowOther arg0)")
            && !report.text.contains("read(ShadowOther arg0_2)"),
        "parameter captured a static owner:\n{read}"
    );

    let write = &recovered(&report, "write").text;
    assert!(
        write.contains("arg0.value ="),
        "first static write changed:\n{write}"
    );
    assert!(
        write.contains("arg0_2.value ="),
        "second static write changed:\n{write}"
    );
    assert!(
        !report.text.contains("write(ShadowOther arg0,")
            && !report.text.contains("write(ShadowOther arg0_2,"),
        "parameter captured a static owner:\n{write}"
    );
}

#[test]
fn essential_and_all_share_text_but_only_all_publishes_sources() {
    let snapshot = open(FIXTURE);
    let essential = class_source_of(&snapshot, RecoveryEvidenceRequest::essential());
    let all = class_source_of(&snapshot, RecoveryEvidenceRequest::all());
    assert_eq!(
        essential.text, all.text,
        "evidence selection changed the class source"
    );
    all_recovered(&essential);
    all_recovered(&all);

    for name in ["<init>", "ownPick", "ownCall", "invoke", "read", "write"] {
        let essential_body = recovered(&essential, name);
        assert_eq!(
            essential_body.evidence.requested(),
            &RecoveryEvidenceRequest::essential(),
            "default `{name}` must request no optional evidence"
        );
        assert!(
            essential_body.source_map.is_empty(),
            "essential `{name}` published a source map"
        );
        assert_eq!(
            essential_body
                .evidence
                .state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::NotRequested,
            "essential `{name}` source-map status"
        );
    }
    assert_all_source_map(&all);
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-type-qualifier-{}-{nonce}",
            std::process::id()
        ));
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
    fs::write(dir.join("TypeQualifierProbe.java"), probe).expect("write the probe source");
    fs::write(dir.join("arg0.java"), ARG0_SOURCE).expect("write arg0 source");
    fs::write(dir.join("arg0_2.java"), ARG0_2_SOURCE).expect("write arg0_2 source");
    fs::write(dir.join("ShadowOther.java"), OTHER_SOURCE).expect("write the helper source");
    fs::write(dir.join("TypeQualifierRunner.java"), RUNNER_SOURCE)
        .expect("write the runner source");
}

fn javac(dir: &Path, probe: &str) {
    fs::create_dir_all(dir).expect("create the javac directory");
    write_sources(dir, probe);
    let output = Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-d")
        .arg(dir.join("classes"))
        .args([
            "TypeQualifierProbe.java",
            "arg0.java",
            "arg0_2.java",
            "ShadowOther.java",
            "TypeQualifierRunner.java",
        ])
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "compiling complete recovered class failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir.join("classes"))
        .arg("TypeQualifierRunner")
        .current_dir(dir)
        .output()
        .expect("execute the fixture runner");
    assert!(
        output.status.success(),
        "the fixture runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the fixture runner writes UTF-8")
}

#[test]
#[ignore = "requires JDK: compile and execute the original and recovered complete classes"]
fn recovered_complete_class_matches_frozen_original() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);
    assert_all_source_map(&report);

    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered_dir = scratch.path().join("recovered");

    // Compile the helper and runner first, then replace the compiler-produced probe with the
    // exact committed bytes.  The original JVM comparison is therefore against the frozen input.
    javac(&original, PROBE_SOURCE);
    fs::write(original.join("classes/TypeQualifierProbe.class"), FIXTURE)
        .expect("install the committed original class bytes");
    let original_output = run_runner(&original);

    javac(&recovered_dir, &report.text);
    let recovered_output = run_runner(&recovered_dir);

    let expected = concat!(
        "null:own:51:3:7:null\n",
        "null:invoke:32:3:7:null\n",
        "null:read:10:3:7:null\n",
        "null:write:done:9:10:null\n",
        "object:own:51:3:7:70\n",
        "object:invoke:32:3:7:70\n",
        "object:read:10:3:7:70\n",
        "object:write:done:9:10:70\n",
    );
    assert_eq!(original_output, expected, "frozen original JVM output");
    assert_eq!(
        recovered_output, original_output,
        "complete recovered class output"
    );
}
