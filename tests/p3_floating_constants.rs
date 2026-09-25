//! `recover-floating-point-constants`: one complete Java 8 class-source fixture for exact
//! float/double values, typed overloads, special values, nested arithmetic, and comparisons.
//!
//! The permanent class is the frozen `FloatingConstants.class`.  `FloatingSupport.java` and
//! `FloatingRunner.java` remain source-only inputs; the ignored JDK test compiles those inputs,
//! replaces the compiler-produced original class with the frozen bytes, and then compiles and
//! executes the complete Engine output without removing or replacing methods.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-floating-constants/v8/FloatingConstants.class");
const PROBE_SOURCE: &str = include_str!("fixtures/p3-floating-constants/FloatingConstants.java");
const SUPPORT_SOURCE: &str = include_str!("fixtures/p3-floating-constants/FloatingSupport.java");
const RUNNER_SOURCE: &str = include_str!("fixtures/p3-floating-constants/FloatingRunner.java");

const METHODS: [&str; 29] = [
    "<init>",
    "floatZero",
    "floatNegativeZero",
    "floatOne",
    "floatTwo",
    "floatNegativeOne",
    "floatFraction",
    "floatMinimum",
    "floatNormal",
    "floatMaximum",
    "floatNan",
    "floatPositiveInfinity",
    "floatNegativeInfinity",
    "doubleZero",
    "doubleNegativeZero",
    "doubleOne",
    "doubleTwo",
    "doubleNegativeOne",
    "doubleFraction",
    "doubleMinimum",
    "doubleNormal",
    "doubleMaximum",
    "doubleNan",
    "doublePositiveInfinity",
    "doubleNegativeInfinity",
    "floatArgument",
    "doubleArgument",
    "nested",
    "threshold",
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed floating-constants fixture opens as a standalone CLASS")
}

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("FloatingConstants"),
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
        METHODS.len(),
        "all Code methods are present"
    );
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

#[test]
fn fixture_covers_typed_finite_and_special_float_values() {
    assert!(PROBE_SOURCE.contains("return 0.0f"));
    assert!(PROBE_SOURCE.contains("return -0.0f"));
    assert!(PROBE_SOURCE.contains("return Float.MIN_VALUE"));
    assert!(PROBE_SOURCE.contains("return Float.MIN_NORMAL"));
    assert!(PROBE_SOURCE.contains("return Float.MAX_VALUE"));
    assert!(PROBE_SOURCE.contains("return Float.NaN"));
    assert!(PROBE_SOURCE.contains("return Float.POSITIVE_INFINITY"));
    assert!(PROBE_SOURCE.contains("return Float.NEGATIVE_INFINITY"));
    assert!(PROBE_SOURCE.contains("return 0.0d"));
    assert!(PROBE_SOURCE.contains("return -0.0d"));
    assert!(PROBE_SOURCE.contains("return Double.MIN_VALUE"));
    assert!(PROBE_SOURCE.contains("return Double.MIN_NORMAL"));
    assert!(PROBE_SOURCE.contains("return Double.MAX_VALUE"));
    assert!(PROBE_SOURCE.contains("return Double.NaN"));
    assert!(PROBE_SOURCE.contains("return Double.POSITIVE_INFINITY"));
    assert!(PROBE_SOURCE.contains("return Double.NEGATIVE_INFINITY"));
    assert!(PROBE_SOURCE.contains("FloatingSupport.pick(0.25f)"));
    assert!(PROBE_SOURCE.contains("FloatingSupport.pick(0.25d)"));
    assert!(PROBE_SOURCE.contains("return -(x + -0.0f)"));
    assert!(PROBE_SOURCE.contains("if(x > 0.0f)"));
    assert!(SUPPORT_SOURCE.contains("pick(float x)"));
    assert!(SUPPORT_SOURCE.contains("pick(double x)"));
    assert!(RUNNER_SOURCE.contains("floatToRawIntBits"));
    assert!(RUNNER_SOURCE.contains("doubleToRawLongBits"));

    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);
    assert!(
        report.fields.is_empty(),
        "the probe has no field declarations"
    );
}

#[test]
fn recovered_text_keeps_float_double_types_grouping_and_consumers() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);

    for name in [
        "floatZero",
        "floatNegativeZero",
        "floatOne",
        "floatTwo",
        "floatNegativeOne",
        "floatFraction",
        "floatMinimum",
        "floatNormal",
        "floatMaximum",
    ] {
        let text = &recovered(&report, name).text;
        assert!(text.contains("return"), "`{name}` has no return:\n{text}");
        let return_line = text
            .lines()
            .find(|line| line.trim_start().starts_with("return "))
            .unwrap_or_else(|| panic!("`{name}` has no return statement:\n{text}"));
        assert!(
            return_line.trim_end().ends_with("f;"),
            "`{name}` lost its float suffix:\n{text}"
        );
        assert!(
            return_line.contains("0x"),
            "`{name}` lost its exact hex literal:\n{text}"
        );
    }
    for name in [
        "doubleZero",
        "doubleNegativeZero",
        "doubleOne",
        "doubleTwo",
        "doubleNegativeOne",
        "doubleFraction",
        "doubleMinimum",
        "doubleNormal",
        "doubleMaximum",
    ] {
        let text = &recovered(&report, name).text;
        assert!(text.contains("return"), "`{name}` has no return:\n{text}");
        let return_line = text
            .lines()
            .find(|line| line.trim_start().starts_with("return "))
            .unwrap_or_else(|| panic!("`{name}` has no return statement:\n{text}"));
        assert!(
            return_line.trim_end().ends_with("d;"),
            "`{name}` lost its double suffix:\n{text}"
        );
        assert!(
            return_line.contains("0x"),
            "`{name}` lost its exact hex literal:\n{text}"
        );
    }

    for (name, required) in [
        ("floatNan", "/"),
        ("floatPositiveInfinity", "/"),
        ("floatNegativeInfinity", "/"),
        ("doubleNan", "/"),
        ("doublePositiveInfinity", "/"),
        ("doubleNegativeInfinity", "/"),
    ] {
        assert!(
            recovered(&report, name).text.contains(required),
            "`{name}` does not retain its special-value expression:\n{}",
            recovered(&report, name).text
        );
    }
    let float_argument = &recovered(&report, "floatArgument").text;
    assert!(
        float_argument.contains("FloatingSupport.pick"),
        "{float_argument}"
    );
    assert!(float_argument.contains('f'), "{float_argument}");
    let double_argument = &recovered(&report, "doubleArgument").text;
    assert!(
        double_argument.contains("FloatingSupport.pick"),
        "{double_argument}"
    );
    assert!(double_argument.contains("d"), "{double_argument}");
    let nested = &recovered(&report, "nested").text;
    assert!(nested.contains("+"), "nested grouping was lost:\n{nested}");
    assert!(
        nested.matches('-').count() >= 2,
        "nested negations were lost:\n{nested}"
    );
    let threshold = &recovered(&report, "threshold").text;
    assert!(
        threshold.contains("if ("),
        "comparison branch was lost:\n{threshold}"
    );
    assert!(
        threshold.contains(">"),
        "comparison polarity was lost:\n{threshold}"
    );
}

#[test]
fn essential_and_all_have_the_same_text_but_only_all_has_source_origins() {
    let snapshot = open(FIXTURE);
    let essential = class_source_of(&snapshot, RecoveryEvidenceRequest::essential());
    let all = class_source_of(&snapshot, RecoveryEvidenceRequest::all());
    assert_eq!(
        essential.text, all.text,
        "evidence selection does not change source"
    );
    all_recovered(&essential);
    all_recovered(&all);

    for name in METHODS {
        let essential_run = recovered(&essential, name);
        assert!(
            essential_run.source_map.is_empty(),
            "essential `{name}` published a source map"
        );
        assert_eq!(
            essential_run
                .evidence
                .state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::NotRequested,
            "essential `{name}` source-map status"
        );

        let all_run = recovered(&all, name);
        assert_eq!(
            all_run.evidence.requested(),
            &RecoveryEvidenceRequest::all(),
            "all `{name}` must echo the complete evidence request"
        );
        assert_eq!(
            all_run.evidence.state(RecoveryEvidenceKind::SourceMap),
            EvidenceState::Complete,
            "all `{name}` source-map status"
        );
        assert!(
            !all_run.source_map.is_empty(),
            "all `{name}` has no source-map segments"
        );
        for segment in all_run.source_map.segments() {
            let origin = segment.origin();
            assert!(!origin.bcis().is_empty(), "segment has no BCI: {origin:?}");
            assert!(
                origin.primary().member().is_some(),
                "segment has no member: {origin:?}"
            );
            for derived in origin.derived() {
                assert!(
                    derived.member().is_some(),
                    "derived origin has no member: {derived:?}"
                );
            }
        }
    }
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-floating-constants-{}-{nonce}",
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
    fs::write(dir.join("FloatingConstants.java"), probe).expect("write the probe source");
    fs::write(dir.join("FloatingSupport.java"), SUPPORT_SOURCE)
        .expect("write the source-only helper");
    fs::write(dir.join("FloatingRunner.java"), RUNNER_SOURCE)
        .expect("write the source-only runner");
}

fn javac(dir: &Path, probe: &str) {
    fs::create_dir_all(dir).expect("create the javac directory");
    write_sources(dir, probe);
    let output = std::process::Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-d")
        .arg(dir)
        .args([
            "FloatingSupport.java",
            "FloatingConstants.java",
            "FloatingRunner.java",
        ])
        .current_dir(dir)
        .output()
        .expect("start javac");
    assert!(
        output.status.success(),
        "compiling generated recovery failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_runner(dir: &Path) -> String {
    let output = std::process::Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg("FloatingRunner")
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
fn recovered_complete_class_matches_frozen_original_raw_bits() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);

    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered_dir = scratch.path().join("recovered");

    // Compile the source-only helper and runner first, then replace the probe with the exact
    // frozen class bytes.  The original side therefore executes the committed input, while the
    // recovered side compiles the complete Engine output without method substitution.
    javac(&original, PROBE_SOURCE);
    fs::write(original.join("FloatingConstants.class"), FIXTURE)
        .expect("install the committed original class bytes");
    let original_output = run_runner(&original);

    javac(&recovered_dir, &report.text);
    let recovered_output = run_runner(&recovered_dir);

    assert_eq!(
        original_output,
        "floatZero=0\nfloatNegativeZero=80000000\nfloatOne=3f800000\nfloatTwo=40000000\nfloatNegativeOne=bf800000\nfloatFraction=3dcccccd\nfloatMinimum=1\nfloatNormal=800000\nfloatMaximum=7f7fffff\nfloatNan=7fc00000\nfloatPositiveInfinity=7f800000\nfloatNegativeInfinity=ff800000\ndoubleZero=0\ndoubleNegativeZero=8000000000000000\ndoubleOne=3ff0000000000000\ndoubleTwo=4000000000000000\ndoubleNegativeOne=bff0000000000000\ndoubleFraction=3fb999999999999a\ndoubleMinimum=1\ndoubleNormal=10000000000000\ndoubleMaximum=7fefffffffffffff\ndoubleNan=7ff8000000000000\ndoublePositiveInfinity=7ff0000000000000\ndoubleNegativeInfinity=fff0000000000000\nfloatArgument=1\ndoubleArgument=2\nnested=ffc00000\nthreshold=9\nnested=3f800000\nthreshold=9\nnested=0\nthreshold=9\nnested=80000000\nthreshold=9\nnested=bf800000\nthreshold=7\n"
    );
    assert_eq!(
        recovered_output, original_output,
        "recovered raw-bit runtime result"
    );
}
