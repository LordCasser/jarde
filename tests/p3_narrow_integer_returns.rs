//! `recover-narrow-integer-returns`: the permanent class has no explicit conversion opcode.  Its
//! Java source returns int, while the committed method descriptors are patched to byte/char/short
//! at selected ireturn sites.  The class therefore tests return-position semantics directly,
//! including local readback, field updates, and synchronized returns.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-narrow-integer-returns/v8/NarrowIntegerReturns.class");
const SOURCE: &str = include_str!("fixtures/p3-narrow-integer-returns/NarrowIntegerReturns.java");
const RUNNER: &str =
    include_str!("fixtures/p3-narrow-integer-returns/NarrowIntegerReturnsRunner.java");

const METHODS: &[&str] = &[
    "<init>",
    "directByte",
    "directChar",
    "directShort",
    "byteLocal",
    "charLocal",
    "shortLocal",
    "integerControl",
    "postByte",
    "preChar",
    "postShort",
    "syncByte",
    "syncChar",
    "syncShort",
];

const DESCRIPTORS: &[(&str, &str)] = &[
    ("<init>", "()V"),
    ("directByte", "(Ljava/lang/String;I)B"),
    ("directChar", "(Ljava/lang/Number;I)C"),
    ("directShort", "(ZI)S"),
    ("byteLocal", "(B)B"),
    ("charLocal", "(C)C"),
    ("shortLocal", "(S)S"),
    ("integerControl", "(BZ)I"),
    ("postByte", "(I)B"),
    ("preChar", "(J)C"),
    ("postShort", "(F)S"),
    ("syncByte", "(Ljava/lang/Object;I)B"),
    ("syncChar", "(Ljava/lang/String;I)C"),
    ("syncShort", "(Ljava/lang/Object;JI)S"),
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed narrow-return fixture opens as a standalone CLASS")
}

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("NarrowIntegerReturns"),
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
    assert_eq!(report.methods.len(), METHODS.len());
    for name in METHODS {
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

fn assert_full_sources(report: &ClassSourceReport) {
    for name in METHODS {
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
fn fixture_has_only_ireturn_narrowing_and_all_return_shapes() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);
    assert_full_sources(&report);
    for (name, descriptor) in DESCRIPTORS {
        assert_eq!(
            method(&report, name).item.descriptor.raw().0,
            descriptor.as_bytes(),
            "the frozen method descriptor for `{name}`"
        );
    }
}

#[test]
fn essential_and_all_share_the_narrow_return_body() {
    let snapshot = open(FIXTURE);
    let essential = class_source_of(&snapshot, RecoveryEvidenceRequest::essential());
    let all = class_source_of(&snapshot, RecoveryEvidenceRequest::all());
    assert_eq!(
        essential.text, all.text,
        "evidence selection changed the class source"
    );
    all_recovered(&essential);
    all_recovered(&all);

    for name in METHODS {
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
    assert_full_sources(&all);
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-narrow-integer-returns-{}-{nonce}",
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

fn write_target(dir: &Path, source: &str) {
    fs::write(dir.join("NarrowIntegerReturns.java"), source).expect("write the target source");
}

fn compile_original(dir: &Path) {
    fs::create_dir_all(dir).expect("create the original directory");
    write_target(dir, SOURCE);
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(dir.join("classes"))
        .arg("NarrowIntegerReturns.java")
        .current_dir(dir)
        .output()
        .expect("start javac for the source target");
    assert!(
        output.status.success(),
        "compiling the source target failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::write(dir.join("classes/NarrowIntegerReturns.class"), FIXTURE)
        .expect("install the frozen descriptor-patched class");
    compile_runner(dir);
}

fn compile_runner(dir: &Path) {
    fs::write(dir.join("NarrowIntegerReturnsRunner.java"), RUNNER)
        .expect("write the source-only runner");
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "-cp"])
        .arg(dir.join("classes"))
        .args(["-d"])
        .arg(dir.join("classes"))
        .arg("NarrowIntegerReturnsRunner.java")
        .current_dir(dir)
        .output()
        .expect("start javac for the runner");
    assert!(
        output.status.success(),
        "compiling the source-only runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = Command::new("java")
        .args(["-Xverify:all", "-cp"])
        .arg(dir.join("classes"))
        .arg("NarrowIntegerReturnsRunner")
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
fn recovered_complete_class_matches_descriptor_patched_original() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);
    assert_full_sources(&report);

    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered_dir = scratch.path().join("recovered");
    compile_original(&original);
    let original_output = run_runner(&original);
    assert_eq!(
        original_output.lines().count(),
        49,
        "fixture runtime case count"
    );
    assert!(
        original_output
            .lines()
            .any(|line| line.starts_with("post:"))
    );
    assert!(
        original_output
            .lines()
            .any(|line| line.starts_with("sync:"))
    );
    assert!(original_output.contains("null:java.lang.NullPointerException"));

    fs::create_dir_all(&recovered_dir).expect("create the recovered directory");
    write_target(&recovered_dir, &report.text);
    let output = Command::new("javac")
        .args(["--release", "8", "-g:none", "-d"])
        .arg(recovered_dir.join("classes"))
        .arg("NarrowIntegerReturns.java")
        .current_dir(&recovered_dir)
        .output()
        .expect("start javac for the recovered class");
    assert!(
        output.status.success(),
        "compiling the complete recovered class failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    compile_runner(&recovered_dir);
    let recovered_output = run_runner(&recovered_dir);
    assert_eq!(
        recovered_output, original_output,
        "complete recovered class output"
    );
}
