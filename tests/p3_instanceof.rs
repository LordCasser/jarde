//! `recover-instanceof-expressions`: one complete Java 8 class-source fixture for real type tests.
//!
//! The permanent class keeps only positive, whole-class methods: Object/null inputs, ordinary and
//! array targets, source-level Object widening, a side-effecting String producer, a real Runnable
//! method reference, and boolean results consumed by a local, a call parameter, and an if.  The
//! unsupported 0/1 merge and discarded/duplicate-consumer shapes remain outside this fixture.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] = include_bytes!("fixtures/p3-instanceof/v8/InstanceOfProbe.class");
const PROBE_SOURCE: &str = include_str!("fixtures/p3-instanceof/InstanceOfProbe.java");
const SUPPORT_SOURCE: &str = include_str!("fixtures/p3-instanceof/InstanceOfSupport.java");
const RUNNER_SOURCE: &str = include_str!("fixtures/p3-instanceof/InstanceOfRunner.java");

const METHODS: [&str; 15] = [
    "<init>",
    "objectString",
    "nullValue",
    "objectRunnable",
    "primitiveArray",
    "referenceArray",
    "multiArray",
    "widenedString",
    "called",
    "local",
    "parameter",
    "branch",
    "functional",
    "keep",
    "empty",
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed instanceof fixture opens as a standalone CLASS")
}

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    class_source_named(snapshot, "InstanceOfProbe", evidence)
}

fn class_source_named(
    snapshot: &ArtifactSnapshot,
    class_name: &str,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    match class_source_with_budget(snapshot, class_name, &evidence, &mut budget()) {
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

fn class_source_with_budget(
    snapshot: &ArtifactSnapshot,
    class_name: &str,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    let request = ClassSourceRequest {
        source_target: jarde::SourceTarget::JAVA_8,
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
    };
    Engine::new()
        .class_source_with_evidence(slice::from_ref(snapshot), &request, evidence, budget)
        .expect("a legal class-source request reports its bounded outcome")
}

fn method_recovery_with_budget(
    snapshot: &ArtifactSnapshot,
    method: &PhysicalMethodId,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> OperationOutcome<MethodRecoveryReport> {
    let request = MethodOperationRequest {
        method: MethodRef::Method {
            method: method.clone(),
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
        .recover_target_with_evidence(
            slice::from_ref(snapshot),
            jarde::SourceTarget::JAVA_8,
            &request,
            evidence,
            budget,
        )
        .expect("a legal physical-method request is answered")
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
fn fixture_covers_positive_instanceof_shapes_without_merge_negative() {
    assert!(PROBE_SOURCE.contains("value instanceof String"));
    assert!(PROBE_SOURCE.contains("(Object) null instanceof String"));
    assert!(PROBE_SOURCE.contains("value instanceof int[]"));
    assert!(PROBE_SOURCE.contains("value instanceof String[]"));
    assert!(PROBE_SOURCE.contains("value instanceof String[][]"));
    assert!(PROBE_SOURCE.contains("(Object) value instanceof Integer"));
    assert!(PROBE_SOURCE.contains("(Object) InstanceOfSupport.value() instanceof Integer"));
    assert!(PROBE_SOURCE.contains("((Runnable) InstanceOfProbe::empty) instanceof Runnable"));
    assert!(PROBE_SOURCE.contains("boolean found = value instanceof String"));
    assert!(PROBE_SOURCE.contains("return keep(value instanceof String)"));
    assert!(PROBE_SOURCE.contains("if (value instanceof String)"));
    assert!(SUPPORT_SOURCE.contains("throw new IllegalStateException"));
    assert!(RUNNER_SOURCE.contains("called-fail="));
    assert!(
        !PROBE_SOURCE.contains("? 0 : 1"),
        "the positive class has no 0/1 merge probe"
    );

    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);
    assert!(
        report.fields.is_empty(),
        "the probe has no unrelated field shape"
    );
}

#[test]
fn recovered_instanceof_text_keeps_operands_targets_and_boolean_consumers() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);

    for (name, target) in [
        ("objectString", "java.lang.String"),
        ("nullValue", "java.lang.String"),
        ("objectRunnable", "java.lang.Runnable"),
        ("primitiveArray", "int[]"),
        ("referenceArray", "java.lang.String[]"),
        ("multiArray", "java.lang.String[][]"),
    ] {
        let text = &recovered(&report, name).text;
        assert!(
            text.contains("instanceof"),
            "`{name}` has no type test:\n{text}"
        );
        assert!(
            text.contains(target),
            "`{name}` lost target `{target}`:\n{text}"
        );
    }

    let widened = &recovered(&report, "widenedString").text;
    assert!(
        widened.contains("java.lang.Object") && widened.contains("java.lang.Integer"),
        "String input keeps a safe Object widening before Integer test:\n{widened}"
    );
    assert!(
        !widened.contains("java.lang.String) arg0 instanceof java.lang.Integer"),
        "the source-level String operand was not narrowed to an incompatible test:\n{widened}"
    );

    let called = &recovered(&report, "called").text;
    assert_eq!(
        called.matches("InstanceOfSupport.value()").count(),
        1,
        "{called}"
    );
    assert!(
        called.contains("java.lang.Object") && called.contains("java.lang.Integer"),
        "{called}"
    );

    let local = &recovered(&report, "local").text;
    assert!(
        local.contains("boolean") && local.contains("instanceof java.lang.String"),
        "{local}"
    );
    let parameter = &recovered(&report, "parameter").text;
    assert!(
        parameter.contains("keep(") && parameter.contains("instanceof java.lang.String"),
        "{parameter}"
    );
    let branch = &recovered(&report, "branch").text;
    assert!(
        branch.contains("if (")
            && branch.contains("instanceof java.lang.String")
            && branch.contains("return 1;")
            && branch.contains("return 0;"),
        "{branch}"
    );

    let functional = &recovered(&report, "functional").text;
    assert!(
        functional.contains("InstanceOfProbe::empty"),
        "{functional}"
    );
    assert!(functional.contains("java.lang.Runnable"), "{functional}");
    assert!(functional.contains("instanceof"), "{functional}");
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

#[test]
fn instanceof_method_replay_and_budget_stops_preserve_atomic_text_and_sources() {
    let snapshot = open(FIXTURE);
    let identity = method_identity(&snapshot, b"objectString", b"(Ljava/lang/Object;)Z");

    let essential = method_recovery_with_budget(
        &snapshot,
        &identity,
        &RecoveryEvidenceRequest::essential(),
        &mut budget(),
    );
    let all = method_recovery_with_budget(
        &snapshot,
        &identity,
        &RecoveryEvidenceRequest::all(),
        &mut budget(),
    );
    let replay = method_recovery_with_budget(
        &snapshot,
        &identity,
        &RecoveryEvidenceRequest::all(),
        &mut budget(),
    );
    let (
        OperationOutcome::Performed(essential),
        OperationOutcome::Performed(all),
        OperationOutcome::Performed(replay),
    ) = (essential, all, replay)
    else {
        panic!("one physical instanceof method completes all evidence selections")
    };
    let essential = essential.recovered.recovery();
    let complete = all.recovered.recovery();
    let replay = replay.recovered.recovery();
    assert!(complete.produced());
    assert_eq!(
        essential.text, complete.text,
        "evidence does not change the body"
    );
    assert!(essential.source_map.is_empty());
    assert_eq!(
        essential.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::NotRequested
    );
    assert_eq!(
        complete.text, replay.text,
        "replaying all evidence keeps the body"
    );
    assert_eq!(complete.source_map, replay.source_map);
    assert_eq!(
        complete.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    );
    let test_segments = complete.source_map.direct_of_bci(1);
    assert!(
        !test_segments.is_empty(),
        "instanceof BCI 1 has a direct source origin"
    );
    assert!(test_segments.iter().any(|segment| {
        segment.origin().primary().method() == Some(&identity)
            && segment.text(&complete.text).contains("instanceof")
    }));

    // The measured method output minus one byte reaches the actual expression emitter and must
    // return either no method artifact or a complete bytecode quote, never partial Java text.
    let mut output_limits = budget().limits().clone();
    output_limits.output_bytes = all
        .usage
        .output_bytes
        .checked_sub(1)
        .expect("the method writes output");
    let stopped = method_recovery_with_budget(
        &snapshot,
        &identity,
        &RecoveryEvidenceRequest::all(),
        &mut Budget::new(output_limits),
    );
    match stopped {
        OperationOutcome::Incomplete(candidates) => assert!(matches!(
            candidates.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::OutputBytes
                },
                ..
            }
        )),
        OperationOutcome::Ambiguous(candidates) => {
            panic!(
                "the physical method is not ambiguous: {}",
                candidates.candidates.len()
            )
        }
        OperationOutcome::Performed(report) => {
            let recovery = report.recovered.recovery();
            assert!(matches!(
                recovery.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: BudgetDimension::OutputBytes
                    },
                    ..
                }
            ));
            assert!(recovery.source_map.is_empty());
            if recovery.produced() {
                assert_eq!(recovery.representation, Representation::Bytecode);
                assert_eq!(recovery.quality, Quality::Fallback);
                assert!(recovery.text.contains("@bytecode"));
            } else {
                assert!(recovery.text.is_empty());
            }
        }
    }

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = class_source_with_budget(
        &snapshot,
        "InstanceOfProbe",
        &RecoveryEvidenceRequest::all(),
        &mut Budget::with_cancellation_token(budget().limits().clone(), token),
    );
    match cancelled {
        OperationOutcome::Incomplete(candidates) => {
            assert!(matches!(
                candidates.execution,
                ExecutionReport::Cancelled { .. }
            ));
        }
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone class is not ambiguous: {}",
            candidates.candidates.len()
        ),
        OperationOutcome::Performed(report) => panic!(
            "a pre-cancelled class-source request cannot publish a partial class: {report:?}"
        ),
    }

    // Evidence replay follows committed text. One fewer IR item than the complete run prevents
    // only the last source-map record from being charged and leaves the body byte-for-byte stable.
    let mut source_limits = budget().limits().clone();
    source_limits.ir_items = all
        .usage
        .ir_items
        .checked_sub(1)
        .expect("the method charges IR and source records");
    let partial = method_recovery_with_budget(
        &snapshot,
        &identity,
        &RecoveryEvidenceRequest::all(),
        &mut Budget::new(source_limits),
    );
    let OperationOutcome::Performed(partial) = partial else {
        panic!("source-map replay stops after the method body is committed: {partial:?}")
    };
    let partial = partial.recovered.recovery();
    assert!(partial.produced());
    assert_eq!(partial.text, complete.text);
    assert_eq!(partial.representation, Representation::Java);
    assert_eq!(partial.source_map.len() + 1, complete.source_map.len());
    assert!(
        partial
            .source_map
            .segments()
            .iter()
            .zip(complete.source_map.segments())
            .all(|(partial, complete)| partial == complete)
    );
    assert_eq!(
        partial.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Partial {
            delivered: partial.source_map.len() as u64
        }
    );
}

// ---------------------------------------------------------------------------
// Instanceof consumption boundaries: these are exact method_info replacements over the frozen
// positive class.  They are legal JVM inputs, but each one deliberately leaves the normal Java
// expression subset: a discarded test result, a duplicated test value, a stale local overwrite,
// or an int-only consumer.  No class-file parser is needed; each complete method_info below is
// anchored by the frozen method header and Code attribute.
// ---------------------------------------------------------------------------

fn patch_complete_method_info(bytes: &[u8], original: &[u8], patched: &[u8]) -> Vec<u8> {
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

fn unused_result_pop_patch() -> Vec<u8> {
    // `called()Z` becomes `called()V`: the producer and instanceof still run once, then POP
    // discards the boolean before the legal void RETURN.  This changes only one complete method_info
    // and changes the descriptor to match the new JVM return opcode.
    patch_complete_method_info(
        FIXTURE,
        &[
            0x00, 0x09, 0x00, 0x2e, 0x00, 0x27, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x13,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0xb8, 0x00, 0x13, 0xc1, 0x00, 0x11,
            0xac, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            0x00, 0x09, 0x00, 0x2e, 0x00, 0x06, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x14,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0xb8, 0x00, 0x13, 0xc1, 0x00, 0x11,
            0x57, 0xb1, 0x00, 0x00, 0x00, 0x00,
        ],
    )
}

fn duplicate_consumption_patch() -> Vec<u8> {
    // `local(Object)Z` duplicates the instanceof result at BCI 4 and consumes the two copies in
    // separate stores at BCIs 5 and 6.  The returned local remains the first copy's value.
    patch_complete_method_info(
        FIXTURE,
        &[
            0x00, 0x09, 0x00, 0x2f, 0x00, 0x25, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x13,
            0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x07, 0x2a, 0xc1, 0x00, 0x07, 0x3c, 0x1b,
            0xac, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            0x00, 0x09, 0x00, 0x2f, 0x00, 0x25, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x15,
            0x00, 0x02, 0x00, 0x03, 0x00, 0x00, 0x00, 0x09, 0x2a, 0xc1, 0x00, 0x07, 0x59, 0x3c,
            0x3d, 0x1b, 0xac, 0x00, 0x00, 0x00, 0x00,
        ],
    )
}

fn stale_local_overwrite_patch() -> Vec<u8> {
    // `local(Object)Z` writes the test to local1 at BCI 4, overwrites that slot with false at BCIs
    // 5-6, and reads the new value at BCI 7.  The first test must not be reused as the returned local.
    patch_complete_method_info(
        FIXTURE,
        &[
            0x00, 0x09, 0x00, 0x2f, 0x00, 0x25, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x13,
            0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x07, 0x2a, 0xc1, 0x00, 0x07, 0x3c, 0x1b,
            0xac, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            0x00, 0x09, 0x00, 0x2f, 0x00, 0x25, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x15,
            0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x09, 0x2a, 0xc1, 0x00, 0x07, 0x3c, 0x03,
            0x3c, 0x1b, 0xac, 0x00, 0x00, 0x00, 0x00,
        ],
    )
}

fn retained_old_local_patch() -> Vec<u8> {
    // The load at BCI 5 stays on the stack while BCI 7 overwrites local1 with false. The
    // return at BCI 8 must consume the old test result, not the slot's newer value.
    patch_complete_method_info(
        FIXTURE,
        &[
            0x00, 0x09, 0x00, 0x2f, 0x00, 0x25, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x13,
            0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 0x07, 0x2a, 0xc1, 0x00, 0x07, 0x3c, 0x1b,
            0xac, 0x00, 0x00, 0x00, 0x00,
        ],
        &[
            0x00, 0x09, 0x00, 0x2f, 0x00, 0x25, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x15,
            0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x00, 0x09, 0x2a, 0xc1, 0x00, 0x07, 0x3c, 0x1b,
            0x03, 0x3c, 0xac, 0x00, 0x00, 0x00, 0x00,
        ],
    )
}

fn boolean_to_int_consumer_patch() -> Vec<u8> {
    // `branch(Object)I` keeps its int descriptor but replaces the branch with `instanceof; iconst_1;
    // iand; ireturn`.  This is verifier-valid because JVM booleans and ints share the int stack
    // category, while Java cannot spell a boolean-to-int bitwise consumer.
    patch_complete_method_info(
        FIXTURE,
        &[
            0x00, 0x09, 0x00, 0x31, 0x00, 0x32, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x20,
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0b, 0x2a, 0xc1, 0x00, 0x07, 0x99, 0x00,
            0x05, 0x04, 0xac, 0x03, 0xac, 0x00, 0x00, 0x00, 0x01, 0x00, 0x33, 0x00, 0x00, 0x00,
            0x03, 0x00, 0x01, 0x09,
        ],
        &[
            0x00, 0x09, 0x00, 0x31, 0x00, 0x32, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x13,
            0x00, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x07, 0x2a, 0xc1, 0x00, 0x07, 0x04, 0x7e,
            0xac, 0x00, 0x00, 0x00, 0x00,
        ],
    )
}

fn boundary_class_source(bytes: &[u8]) -> ClassSourceReport {
    class_source_of(&open(bytes), RecoveryEvidenceRequest::all())
}

fn boundary_member<'a>(
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
        .unwrap_or_else(|| panic!("no member `{name}{descriptor}` in InstanceOfProbe"))
}

fn boundary_recovered<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> &'a RecoveryReport {
    match &boundary_member(report, name, descriptor).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}{descriptor}` was not recovered: {other:?}"),
    }
}

fn quoted_bcis(text: &str) -> Vec<u32> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|list| list.split_whitespace())
        .map(|bci| bci.parse().expect("a quoted BCI is an integer"))
        .collect()
}

fn assert_boundary_sources(report: &ClassSourceReport, name: &str, descriptor: &str, bcis: &[u32]) {
    let member = boundary_member(report, name, descriptor);
    let recovery = boundary_recovered(report, name, descriptor);
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
            member.text
        );
        assert!(
            segments
                .iter()
                .any(|segment| segment.origin().primary().method() == Some(&member.item.identity)),
            "BCI {bci} of `{name}{descriptor}` has no segment owned by its real member: {:?}",
            segments
        );
    }
}

fn assert_boundary_quote(
    report: &ClassSourceReport,
    name: &str,
    descriptor: &str,
    required_bcis: &[u32],
) {
    let member = boundary_member(report, name, descriptor);
    let recovery = boundary_recovered(report, name, descriptor);
    assert!(
        member.text.contains("@bytecode"),
        "{name}{descriptor}: {}",
        member.text
    );
    assert!(
        recovery.produced(),
        "{name}{descriptor} produced no refusal artifact"
    );
    assert_eq!(
        recovery.representation,
        Representation::Mixed,
        "{name}{descriptor}"
    );
    assert_eq!(recovery.quality, Quality::Fallback, "{name}{descriptor}");
    assert_eq!(
        recovery.syntax_status,
        SyntaxStatus::NotJava,
        "{name}{descriptor}"
    );
    let quoted = quoted_bcis(&member.text);
    for bci in required_bcis {
        assert!(
            quoted.contains(bci),
            "{name}{descriptor} did not quote required BCI {bci}: {}",
            member.text
        );
        assert!(
            !recovery.text_of_bci(*bci).is_empty(),
            "{name}{descriptor} lost source text for quoted BCI {bci}"
        );
    }
}

#[test]
fn frozen_instanceof_patches_have_expected_class_sizes() {
    // Exact lengths complement the unique method_info anchors. JVM execution is recorded in
    // refusal-boundaries/; class length alone does not verify runtime behavior.
    assert_eq!(
        unused_result_pop_patch().len(),
        1549,
        "pop patch changes only the called method_info"
    );
    assert_eq!(
        duplicate_consumption_patch().len(),
        1550,
        "duplicate patch changes only the local method_info"
    );
    assert_eq!(
        stale_local_overwrite_patch().len(),
        1550,
        "overwrite patch changes only the local method_info"
    );
    assert_eq!(
        boolean_to_int_consumer_patch().len(),
        1535,
        "int-consumer patch removes the branch StackMapTable"
    );
}

#[test]
fn unsupported_instanceof_consumers_keep_producers_and_real_boundary_sources() {
    let pop = boundary_class_source(&unused_result_pop_patch());
    assert_boundary_sources(&pop, "called", "()V", &[0, 3, 6, 7]);
    assert_boundary_quote(&pop, "called", "()V", &[0, 3, 6]);
    let pop_text = &boundary_member(&pop, "called", "()V").text;
    assert!(
        !pop_text.contains("InstanceOfSupport.value();"),
        "{pop_text}"
    );

    let duplicate = boundary_class_source(&duplicate_consumption_patch());
    assert_boundary_sources(&duplicate, "local", "(Ljava/lang/Object;)Z", &[1, 4, 5, 6]);
    assert_boundary_quote(&duplicate, "local", "(Ljava/lang/Object;)Z", &[1, 4, 5, 6]);

    let stale = boundary_class_source(&stale_local_overwrite_patch());
    assert_boundary_sources(&stale, "local", "(Ljava/lang/Object;)Z", &[1, 4, 5, 6]);
    let stale_text = &boundary_member(&stale, "local", "(Ljava/lang/Object;)Z").text;
    assert!(
        !stale_text.contains("@bytecode"),
        "the current-value overwrite remains a positive presentation:\n{stale_text}"
    );
    assert!(
        stale_text.contains("arg0 instanceof java.lang.String")
            && stale_text.contains("local1 = false;")
            && stale_text.contains("return local1;"),
        "the current value follows the test and overwrite:\n{stale_text}"
    );

    let retained = boundary_class_source(&retained_old_local_patch());
    assert_boundary_sources(
        &retained,
        "local",
        "(Ljava/lang/Object;)Z",
        &[1, 4, 5, 7, 8],
    );
    assert_old_value_physical_sources(
        &retained,
        &retained_old_local_patch(),
        "local",
        "(Ljava/lang/Object;)Z",
        false,
    );
    let retained_text = &boundary_member(&retained, "local", "(Ljava/lang/Object;)Z").text;
    assert!(
        retained_text.contains("instanceof java.lang.String"),
        "{retained_text}"
    );
    assert!(retained_text.contains("local1 = false;"), "{retained_text}");
}

#[test]
fn boolean_to_int_consumer_keeps_refusal_sources_independently() {
    let int_consumer = boundary_class_source(&boolean_to_int_consumer_patch());
    assert_boundary_sources(&int_consumer, "branch", "(Ljava/lang/Object;)I", &[1, 5, 6]);
    assert_boundary_quote(&int_consumer, "branch", "(Ljava/lang/Object;)I", &[1, 5, 6]);
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-instanceof-{}-{nonce}",
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
    fs::write(dir.join("InstanceOfProbe.java"), probe).expect("write the probe source");
    fs::write(dir.join("InstanceOfSupport.java"), SUPPORT_SOURCE)
        .expect("write the source-only helper");
    fs::write(dir.join("InstanceOfRunner.java"), RUNNER_SOURCE)
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
            "InstanceOfSupport.java",
            "InstanceOfProbe.java",
            "InstanceOfRunner.java",
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
        .arg("InstanceOfRunner")
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
#[ignore = "requires JDK: compile and execute original and recovered complete classes"]
fn recovered_complete_class_matches_original_runtime() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);

    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered_dir = scratch.path().join("recovered");

    // Compile helper and runner first, then restore the exact frozen probe bytes.  The original
    // side therefore executes the committed class rather than a newly generated local class.
    javac(&original, PROBE_SOURCE);
    fs::write(original.join("InstanceOfProbe.class"), FIXTURE)
        .expect("install the committed original class bytes");
    let original_output = run_runner(&original);

    javac(&recovered_dir, &report.text);
    let recovered_output = run_runner(&recovered_dir);

    let expected = concat!(
        "string-text=true\n",
        "string-null=false\n",
        "string-number=false\n",
        "null=false\n",
        "runnable=true\n",
        "runnable-null=false\n",
        "primitive-array=true\n",
        "primitive-array-string=false\n",
        "reference-array=true\n",
        "reference-array-object=false\n",
        "multi-array=true\n",
        "multi-array-one=false\n",
        "widened-string=false\n",
        "local-true=true\n",
        "local-false=false\n",
        "parameter-true=true\n",
        "parameter-false=false\n",
        "branch-true=1\n",
        "branch-false=0\n",
        "functional=true\n",
        "called=false:1\n",
        "called-fail=java.lang.IllegalStateException:1\n"
    );
    assert_eq!(original_output, expected, "the frozen class runtime oracle");
    assert_eq!(
        recovered_output, original_output,
        "the complete recovered class preserves behavior"
    );
}

#[test]
fn retained_old_local_whole_class_matches_original_runtime() {
    for (bytes, expected) in [
        (retained_old_local_patch(), "true\nfalse\nfalse\n"),
        (stale_local_overwrite_patch(), "false\nfalse\nfalse\n"),
    ] {
        let report = boundary_class_source(&bytes);
        let scratch = Scratch::new();
        let original = scratch.path().join("original");
        let recovered = scratch.path().join("recovered");
        let driver = "public class OldValueDriver { public static void main(String[] args) { for(Object x : new Object[] {\"text\", new Object(), null}) System.out.println(InstanceOfProbe.local(x)); } }";
        for (dir, source) in [
            (&original, PROBE_SOURCE),
            (&recovered, report.text.as_str()),
        ] {
            javac(dir, source);
            fs::write(dir.join("OldValueDriver.java"), driver).unwrap();
            let compile = std::process::Command::new("javac")
                .args(["--release", "8", "-g:none", "-cp"])
                .arg(dir)
                .arg(dir.join("OldValueDriver.java"))
                .output()
                .unwrap();
            assert!(
                compile.status.success(),
                "{}",
                String::from_utf8_lossy(&compile.stderr)
            );
        }
        fs::write(original.join("InstanceOfProbe.class"), bytes).unwrap();
        let outputs: Vec<String> = [&original, &recovered]
            .iter()
            .map(|dir| {
                let out = std::process::Command::new("java")
                    .args(["-Xverify:all", "-cp"])
                    .arg(dir)
                    .arg("OldValueDriver")
                    .output()
                    .unwrap();
                assert!(
                    out.status.success(),
                    "{}",
                    String::from_utf8_lossy(&out.stderr)
                );
                String::from_utf8(out.stdout).unwrap()
            })
            .collect();
        assert_eq!(outputs[0], expected);
        assert_eq!(outputs[1], outputs[0]);
        assert_eq!(
            run_runner(&recovered),
            run_runner(&original),
            "whole class including calls and abrupt completion"
        );
    }
}

// Generated classes vary names, targets, values and instruction encodings independently and
// together. They are same-origin invariance probes, not an independent holdout corpus.
fn old_value_class(
    class: &str,
    method: &str,
    numeric: bool,
    wide: bool,
    alternate: bool,
    padding: bool,
) -> Vec<u8> {
    fn u16b(out: &mut Vec<u8>, n: u16) {
        out.extend(n.to_be_bytes());
    }
    fn utf8(out: &mut Vec<u8>, s: &str) {
        out.push(1);
        u16b(out, s.len().try_into().unwrap());
        out.extend(s.as_bytes());
    }
    let mut out = 0xcafebabe_u32.to_be_bytes().to_vec();
    for n in [0, 49, 10] {
        u16b(&mut out, n);
    }
    utf8(&mut out, class);
    out.push(7);
    u16b(&mut out, 1);
    utf8(&mut out, "java/lang/Object");
    out.push(7);
    u16b(&mut out, 3);
    utf8(&mut out, method);
    utf8(
        &mut out,
        if numeric {
            "(JJ)I"
        } else {
            "(Ljava/lang/Object;)Z"
        },
    );
    utf8(&mut out, "Code");
    utf8(
        &mut out,
        if alternate {
            "java/lang/CharSequence"
        } else {
            "java/lang/String"
        },
    );
    out.push(7);
    u16b(&mut out, 8);
    for n in [0x21, 2, 4, 0, 0, 1, 0x09, 5, 6, 1, 7] {
        u16b(&mut out, n);
    }
    let mut code = Vec::new();
    if padding {
        code.extend([0x00, 0x00]);
    }
    let mut local = |ordinary: u8, generic: u8, slot: u8| {
        if wide {
            code.extend([0xc4, generic, 0, slot]);
        } else {
            code.push(ordinary);
        }
    };
    if numeric {
        local(0x1e, 0x16, 0);
    } else {
        local(0x2a, 0x19, 0);
    }
    if numeric {
        code.push(if alternate { 0x0a } else { 0x09 });
        if wide {
            code.extend([0xc4, 0x37, 0, 0, 0xc4, 0x16, 0, 2]);
        } else {
            code.extend([0x3f, 0x20]);
        }
        code.extend([
            0x94,
            0x9a,
            0,
            6,
            0x10,
            if alternate { 11 } else { 7 },
            0xac,
            0x10,
            if alternate { 13 } else { 9 },
            0xac,
        ]);
    } else {
        code.extend([0xc1, 0, 9]);
        if wide {
            code.extend([0xc4, 0x36, 0, 3, 0xc4, 0x15, 0, 3]);
        } else {
            code.extend([0x3c, 0x1b]);
        }
        code.push(if alternate { 0x04 } else { 0x03 });
        if wide {
            code.extend([0xc4, 0x36, 0, 3]);
        } else {
            code.push(0x3c);
        }
        code.push(0xac);
    }
    out.extend((12 + code.len() as u32).to_be_bytes());
    for n in [6, 4] {
        u16b(&mut out, n);
    }
    out.extend((code.len() as u32).to_be_bytes());
    out.extend(code);
    for _ in 0..3 {
        u16b(&mut out, 0);
    }
    out
}

fn assert_old_value_physical_sources(
    report: &ClassSourceReport,
    bytes: &[u8],
    name: &str,
    descriptor: &str,
    padding: bool,
) {
    use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
    let member = boundary_member(report, name, descriptor);
    let recovered = boundary_recovered(report, name, descriptor);
    assert_eq!(
        recovered.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    );
    assert!(recovered.produced());
    if !padding {
        assert!(recovered.fallbacks.is_empty());
    }
    assert_eq!(
        recovered.representation,
        if padding {
            Representation::Mixed
        } else {
            Representation::Java
        },
        "{}",
        member.text
    );
    assert_eq!(
        quoted_bcis(&member.text),
        if padding { vec![0, 1] } else { vec![] }
    );
    let physical = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut budget(),
    )
    .unwrap();
    assert!(physical.stopped_at.is_none());
    for segment in recovered.source_map.segments() {
        assert!(segment.start() < segment.end());
        assert!(recovered.text.get(segment.start()..segment.end()).is_some());
        for origin in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(origin.method(), Some(&member.item.identity));
            assert!(physical.instructions.iter().any(|i| i.bci == origin.bci()));
        }
    }
    for instruction in &physical.instructions {
        {
            assert!(
                !recovered.source_map.of_bci(instruction.bci).is_empty(),
                "missing physical {:?}: {}",
                instruction,
                member.text
            );
        }
    }
    let text = &member.text;
    let ty = if descriptor == "(JJ)I" {
        "long"
    } else {
        "boolean"
    };
    let (saved, definition) = text
        .lines()
        .enumerate()
        .find_map(|(i, line)| {
            let declaration = line.trim().strip_prefix(&format!("{ty} "))?;
            let (name, value) = declaration.trim_end_matches(';').split_once(" = ")?;
            (!value.contains("instanceof")).then_some((name.to_owned(), i))
        })
        .expect("a typed old-value definition");
    let lines: Vec<_> = text.lines().map(str::trim).collect();
    let overwrite = lines
        .iter()
        .position(|line| {
            if descriptor == "(JJ)I" {
                line.starts_with("arg0 = ")
            } else {
                line.starts_with("local") && !line.starts_with("boolean") && line.ends_with(';')
            }
        })
        .unwrap();
    let terminal = lines
        .iter()
        .position(|line| {
            if descriptor == "(JJ)I" {
                line.starts_with(&format!("if ({saved} == arg2)"))
            } else {
                **line == format!("return {saved};")
            }
        })
        .unwrap();
    assert!(definition < overwrite && overwrite < terminal, "{text}");
}

#[test]
fn old_value_renamed_revalued_and_wide_classes_match_whole_jvm_output() {
    for numeric in [false, true] {
        for renamed in [false, true] {
            for alternate in [false, true] {
                for wide in [false, true] {
                    for padding in [false, true] {
                        let class = if renamed {
                            "ChangedOldOperand"
                        } else {
                            "OldOperand"
                        };
                        let method = if renamed { "evaluate" } else { "compute" };
                        let descriptor = if numeric {
                            "(JJ)I"
                        } else {
                            "(Ljava/lang/Object;)Z"
                        };
                        let bytes =
                            old_value_class(class, method, numeric, wide, alternate, padding);
                        let report = class_source_named(
                            &open(&bytes),
                            class,
                            RecoveryEvidenceRequest::all(),
                        );
                        eprintln!(
                            "variant numeric={numeric} renamed={renamed} alternate={alternate} wide={wide} padding={padding}: {}",
                            report.text
                        );
                        assert_old_value_physical_sources(
                            &report, &bytes, method, descriptor, padding,
                        );
                        let scratch = Scratch::new();
                        let original = scratch.path().join("original");
                        let recovered = scratch.path().join("recovered");
                        for dir in [&original, &recovered] {
                            fs::create_dir(dir).unwrap();
                        }
                        fs::write(original.join(format!("{class}.class")), &bytes).unwrap();
                        fs::write(recovered.join(format!("{class}.java")), &report.text).unwrap();
                        let body = if numeric {
                            format!(
                                "long[] xs = {{Long.MIN_VALUE,-1,0,1,Long.MAX_VALUE}}; for(long a:xs) for(long b:xs) {{ int result = {class}.{method}(a,b); if(result != (a == b ? {} : {})) throw new AssertionError(); System.out.println(result); }}",
                                if alternate { 11 } else { 7 },
                                if alternate { 13 } else { 9 }
                            )
                        } else {
                            format!(
                                "Object[] xs = {{\"text\",new Object(),null,new StringBuilder(\"x\")}}; for(Object x:xs) {{ boolean result = {class}.{method}(x); if(result != (x instanceof {})) throw new AssertionError(); System.out.println(result); }}",
                                if alternate { "CharSequence" } else { "String" }
                            )
                        };
                        let driver = format!(
                            "public class OldOperandDriver {{ public static void main(String[] args) {{ {body} }} }}"
                        );
                        let mut outputs = Vec::new();
                        for dir in [&original, &recovered] {
                            fs::write(dir.join("OldOperandDriver.java"), &driver).unwrap();
                            let mut command = std::process::Command::new("javac");
                            command
                                .args(["--release", "8", "-g:none", "-cp"])
                                .arg(dir)
                                .arg(dir.join("OldOperandDriver.java"));
                            if dir == &recovered {
                                command.arg(dir.join(format!("{class}.java")));
                            }
                            let compile = command.output().unwrap();
                            assert!(
                                compile.status.success(),
                                "{}\n{}",
                                String::from_utf8_lossy(&compile.stderr),
                                report.text
                            );
                            let run = std::process::Command::new("java")
                                .args(["-Xverify:all", "-cp"])
                                .arg(dir)
                                .arg("OldOperandDriver")
                                .output()
                                .unwrap();
                            assert!(
                                run.status.success(),
                                "{}",
                                String::from_utf8_lossy(&run.stderr)
                            );
                            outputs.push(run.stdout);
                        }
                        assert_eq!(outputs[0], outputs[1]);
                    }
                }
            }
        }
    }
}
