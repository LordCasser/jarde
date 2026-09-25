//! `recover-primitive-conversions`: one Java 8 class carries all fifteen explicit conversion
//! opcodes, conversion-driven overload calls, intermediate floating-point rounding, and an
//! effecting left-to-right producer.  The support and runner classes remain source-only.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-primitive-conversions/v8/PrimitiveConversions.class");
const PROBE_SOURCE: &str =
    include_str!("fixtures/p3-primitive-conversions/PrimitiveConversions.java");
const SUPPORT_SOURCE: &str =
    include_str!("fixtures/p3-primitive-conversions/PrimitiveConversionSupport.java");
const EFFECTS_SOURCE: &str =
    include_str!("fixtures/p3-primitive-conversions/PrimitiveConversionEffects.java");
const RUNNER_SOURCE: &str =
    include_str!("fixtures/p3-primitive-conversions/PrimitiveConversionsRunner.java");

const METHODS: &[&str] = &[
    "<init>",
    "i2l",
    "i2f",
    "i2d",
    "i2b",
    "i2c",
    "i2s",
    "l2i",
    "l2f",
    "l2d",
    "f2i",
    "f2l",
    "f2d",
    "d2i",
    "d2l",
    "d2f",
    "byteOverload",
    "shortOverload",
    "charOverload",
    "longIntOverload",
    "longFloatOverload",
    "floatDoubleOverload",
    "doubleIntOverload",
    "intFloatRound",
    "longFloatRound",
    "longDoubleRound",
    "doubleFloatRound",
    "floatByte",
    "doubleChar",
    "byteChar",
    "ordered",
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn limits() -> Limits {
    budget().limits().clone()
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed primitive-conversions fixture opens as a standalone CLASS")
}

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let request = class_source_request(snapshot);
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

fn class_source_request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("PrimitiveConversions"),
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

fn method_identity(
    snapshot: &ArtifactSnapshot,
    name: &[u8],
    descriptor: &[u8],
) -> PhysicalMethodId {
    let header = Engine::new()
        .inspect_header(
            snapshot,
            ClassTarget::Root,
            &mut budget(),
            InspectionMode::Strict,
        )
        .expect("the fixture header is readable");
    PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: header.source.class_bytes,
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    }
}

fn recover_method_with_budget(
    snapshot: &ArtifactSnapshot,
    identity: &PhysicalMethodId,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> OperationOutcome<MethodRecoveryReport> {
    let request = MethodOperationRequest {
        method: MethodRef::Method {
            method: identity.clone(),
        },
        environment: class_source_request(snapshot).environment,
    };
    Engine::new()
        .recover_target_with_evidence(slice::from_ref(snapshot), &request, evidence, budget)
        .expect("a legal class-source request is answered")
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

fn performed_method(outcome: &OperationOutcome<MethodRecoveryReport>) -> &MethodRecoveryReport {
    match outcome {
        OperationOutcome::Performed(report) => report,
        other => panic!("the named conversion method is recoverable: {other:?}"),
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
fn all_fixture_methods_are_structured_with_sources() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);
    assert_full_sources(&report);
}

#[test]
fn essential_and_all_share_one_conversion_body() {
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

    let selected = method(&all, "longFloatRound");
    let identity = &selected.item.identity;
    let essential_body = recover_method_with_budget(
        &snapshot,
        identity,
        &RecoveryEvidenceRequest::essential(),
        &mut budget(),
    );
    let all_body = recover_method_with_budget(
        &snapshot,
        identity,
        &RecoveryEvidenceRequest::all(),
        &mut budget(),
    );
    let replayed_body = recover_method_with_budget(
        &snapshot,
        identity,
        &RecoveryEvidenceRequest::all(),
        &mut budget(),
    );
    let essential_recovery = performed_method(&essential_body).recovered.recovery();
    let all_recovery = performed_method(&all_body).recovered.recovery();
    let replayed_recovery = performed_method(&replayed_body).recovered.recovery();
    assert_eq!(essential_recovery.text, all_recovery.text);
    assert!(essential_recovery.source_map.is_empty());
    assert_eq!(
        essential_recovery
            .evidence
            .state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::NotRequested
    );
    assert_eq!(all_recovery.text, replayed_recovery.text);
    assert_eq!(all_recovery.source_map, replayed_recovery.source_map);
    assert!(all_recovery.produced());
    assert_eq!(
        all_recovery.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    );

    // In the frozen method: lload_0; l2f; f2l; lreturn. Each nested Cast owns its instruction
    // anchor, and replaying the same request preserves both those anchors and their member.
    for (bci, spelling) in [(1, "float"), (2, "long")] {
        let segments = all_recovery.source_map.direct_of_bci(bci);
        assert!(
            !segments.is_empty(),
            "conversion BCI {bci} has a direct node"
        );
        assert!(
            segments.iter().any(|segment| {
                segment.origin().primary().method() == Some(identity)
                    && segment.text(&all_recovery.text).contains(spelling)
            }),
            "conversion BCI {bci} keeps its typed cast and owning method"
        );
    }
}

fn assert_budget_stop_has_no_partial_conversion(
    snapshot: &ArtifactSnapshot,
    identity: &PhysicalMethodId,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
    label: &str,
) {
    match recover_method_with_budget(snapshot, identity, evidence, budget) {
        OperationOutcome::Incomplete(_) => {}
        OperationOutcome::Ambiguous(candidates) => panic!(
            "the physical method identity is not ambiguous: {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Performed(report) => {
            let recovery = report.recovered.recovery();
            assert!(
                recovery.stop().is_some()
                    || !matches!(
                        report.recovered.analysis().execution,
                        ExecutionReport::Complete { .. }
                    ),
                "{label} must report the budget/cancellation stop; usage={:?}, limits={:?}, stop={:?}, execution={:?}",
                report.usage,
                report.limits,
                recovery.stop(),
                report.recovered.analysis().execution
            );
            if recovery.produced() {
                assert_eq!(
                    recovery.representation,
                    Representation::Bytecode,
                    "{label} may only return a complete quote, never partial Java"
                );
                assert_eq!(recovery.quality, Quality::Fallback, "{label}");
                assert!(
                    recovery.text.contains("@bytecode"),
                    "{label}: {}",
                    recovery.text
                );
                assert!(
                    recovery.source_map.is_empty(),
                    "{label} quote has no Java map"
                );
            } else {
                assert!(recovery.text.is_empty(), "{label} stop published text");
                assert!(recovery.source_map.is_empty(), "{label} stop published map");
            }
        }
    }
}

#[test]
fn conversion_recovery_budget_stops_and_cancellation_publish_no_partial_cast_body() {
    let snapshot = open(FIXTURE);
    let identity = method_identity(&snapshot, b"longFloatRound", b"(J)J");
    let complete = match recover_method_with_budget(
        &snapshot,
        &identity,
        &RecoveryEvidenceRequest::all(),
        &mut budget(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("the baseline conversion method completes: {other:?}"),
    };
    assert!(complete.recovered.recovery().produced());
    assert!(complete.usage.output_bytes > 0);
    assert!(complete.usage.ir_items > 0);
    assert!(complete.usage.analysis_steps > 0);
    let analyzed_ir_items = match &complete.recovered.analysis().execution {
        ExecutionReport::Complete { usage } => usage.ir_items,
        other => panic!("the baseline analysis completes: {other:?}"),
    };
    assert!(analyzed_ir_items > 0);

    let mut output_limits = limits();
    output_limits.output_bytes = complete
        .usage
        .output_bytes
        .checked_sub(1)
        .expect("the body writes at least one output byte");
    let output_snapshot = open(FIXTURE);
    let output_identity = method_identity(&output_snapshot, b"longFloatRound", b"(J)J");
    assert_budget_stop_has_no_partial_conversion(
        &output_snapshot,
        &output_identity,
        &RecoveryEvidenceRequest::all(),
        &mut Budget::new(output_limits),
        "one byte below the measured method output",
    );

    let mut ir_limits = limits();
    ir_limits.ir_items = analyzed_ir_items
        .checked_sub(1)
        .expect("the method analysis charges IR items");
    let ir_snapshot = open(FIXTURE);
    let ir_identity = method_identity(&ir_snapshot, b"longFloatRound", b"(J)J");
    assert_budget_stop_has_no_partial_conversion(
        &ir_snapshot,
        &ir_identity,
        &RecoveryEvidenceRequest::all(),
        &mut Budget::new(ir_limits),
        "one item below the measured IR use",
    );

    // Analysis work is charged before it runs, in the CFG/region analysis that precedes expression
    // building. A limit one step below the same method's measured analysis work must stop the
    // request before a conversion body can be committed.
    let mut analysis_limits = limits();
    analysis_limits.analysis_steps = complete
        .usage
        .analysis_steps
        .checked_sub(1)
        .expect("the method analysis charges work steps");
    let analysis_snapshot = open(FIXTURE);
    let analysis_identity = method_identity(&analysis_snapshot, b"longFloatRound", b"(J)J");
    match recover_method_with_budget(
        &analysis_snapshot,
        &analysis_identity,
        &RecoveryEvidenceRequest::all(),
        &mut Budget::new(analysis_limits),
    ) {
        OperationOutcome::Incomplete(candidates) => assert!(matches!(
            candidates.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps
                },
                ..
            }
        )),
        OperationOutcome::Ambiguous(candidates) => panic!(
            "the physical method identity is not ambiguous: {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Performed(report) => {
            let recovery = report.recovered.recovery();
            assert!(matches!(
                recovery.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: BudgetDimension::AnalysisSteps
                    },
                    ..
                }
            ));
            assert!(!recovery.produced(), "analysis stop published a body");
            assert!(recovery.text.is_empty(), "analysis stop published text");
            assert!(
                recovery.source_map.is_empty(),
                "analysis stop published a map"
            );
        }
    }

    // Source maps have no independent budget dimension. Their replay charges one IrItems per
    // anchored span after the Java text and earlier evidence are committed. Reducing the complete
    // run's measured total by one therefore refuses the final replay charge: the full body remains
    // intact and the map contains only its charged record prefix.
    let mut source_limits = limits();
    source_limits.ir_items = complete
        .usage
        .ir_items
        .checked_sub(1)
        .expect("the complete run charges IR and source records");
    let source_snapshot = open(FIXTURE);
    let source_identity = method_identity(&source_snapshot, b"longFloatRound", b"(J)J");
    match recover_method_with_budget(
        &source_snapshot,
        &source_identity,
        &RecoveryEvidenceRequest::all(),
        &mut Budget::new(source_limits),
    ) {
        OperationOutcome::Incomplete(candidates) => panic!(
            "source-map exhaustion preserves the already committed method artifact: {candidates:?}"
        ),
        OperationOutcome::Ambiguous(candidates) => panic!(
            "the physical method identity is not ambiguous: {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Performed(report) => {
            let recovery = report.recovered.recovery();
            assert!(recovery.produced(), "source-map stop lost committed text");
            assert_eq!(recovery.text, complete.recovered.recovery().text);
            assert_eq!(recovery.representation, Representation::Java);
            assert_eq!(
                recovery.source_map.len() + 1,
                complete.recovered.recovery().source_map.len(),
                "the one refused charge must not create its source record"
            );
            assert!(
                recovery
                    .source_map
                    .segments()
                    .iter()
                    .zip(complete.recovered.recovery().source_map.segments())
                    .all(|(partial, whole)| partial == whole)
            );
            assert_eq!(
                recovery.evidence.state(RecoveryEvidenceKind::SourceMap),
                EvidenceState::Partial {
                    delivered: recovery.source_map.len() as u64
                },
                "source-map exhaustion publishes only complete charged records"
            );
            assert!(matches!(
                recovery.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: BudgetDimension::IrItems
                    },
                    ..
                }
            ));
        }
    }

    let token = CancellationToken::new();
    token.cancel();
    let cancelled_snapshot = open(FIXTURE);
    let cancelled_identity = method_identity(&cancelled_snapshot, b"longFloatRound", b"(J)J");
    let cancelled = recover_method_with_budget(
        &cancelled_snapshot,
        &cancelled_identity,
        &RecoveryEvidenceRequest::all(),
        &mut Budget::with_cancellation_token(limits(), token),
    );
    match cancelled {
        OperationOutcome::Incomplete(candidates) => assert!(matches!(
            candidates.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Ambiguous(candidates) => panic!(
            "the physical method identity is not ambiguous: {} candidates",
            candidates.candidates.len()
        ),
        OperationOutcome::Performed(report) => {
            let recovery = report.recovered.recovery();
            assert!(
                matches!(
                    report.recovered.analysis().execution,
                    ExecutionReport::Cancelled { .. }
                ) || recovery.stop().is_some_and(StopReason::is_cancelled),
                "the request states its cancellation"
            );
            assert!(!recovery.produced(), "cancellation published a body");
            assert!(recovery.text.is_empty(), "cancellation published text");
            assert!(
                recovery.source_map.is_empty(),
                "cancellation published a map"
            );
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
            "jarde-p3-primitive-conversions-{}-{nonce}",
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
    fs::write(dir.join("PrimitiveConversions.java"), probe).expect("write the probe source");
    fs::write(dir.join("PrimitiveConversionSupport.java"), SUPPORT_SOURCE)
        .expect("write the support source");
    fs::write(dir.join("PrimitiveConversionEffects.java"), EFFECTS_SOURCE)
        .expect("write the effects source");
    fs::write(dir.join("PrimitiveConversionsRunner.java"), RUNNER_SOURCE)
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
            "PrimitiveConversions.java",
            "PrimitiveConversionSupport.java",
            "PrimitiveConversionEffects.java",
            "PrimitiveConversionsRunner.java",
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
        .arg("PrimitiveConversionsRunner")
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
    assert_full_sources(&report);

    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered_dir = scratch.path().join("recovered");

    // Compile the source-only classes first, then replace the compiler-produced probe with the
    // exact frozen bytes so the original JVM side is tied to the committed fixture.
    javac(&original, PROBE_SOURCE);
    fs::write(original.join("classes/PrimitiveConversions.class"), FIXTURE)
        .expect("install the committed original class bytes");
    let original_output = run_runner(&original);
    assert_eq!(
        original_output.lines().count(),
        41,
        "fixture runtime case count"
    );
    assert!(
        original_output
            .lines()
            .any(|line| line.starts_with("chain:"))
    );
    assert_eq!(
        original_output
            .lines()
            .filter(|line| line.starts_with("order:"))
            .count(),
        8,
        "effect-order cases"
    );

    javac(&recovered_dir, &report.text);
    let recovered_output = run_runner(&recovered_dir);
    assert_eq!(
        recovered_output, original_output,
        "complete recovered class output"
    );
}
