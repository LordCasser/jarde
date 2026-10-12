//! Planned public-API regression for local reference writes across debug profiles.
//!
//! Fixture files are copied from the exact javac8/javac23 original legs named in README.md.
//! This is a private draft: control text or internal proof gates are intentionally not guessed.

use jarde::*;
use jarde_jvm::ir::{Quality, Representation};
use jarde_reader::budget::{Budget, CancellationToken, UsageSnapshot};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::{ExecutionReport, TerminationReason};
use std::slice;

macro_rules! fixture {
    ($profile:ident, $jdk:ident, $class:ident) => {
        (
            stringify!($profile),
            stringify!($jdk),
            stringify!($class),
            include_bytes!(concat!(
                "fixtures/p3-local-reference-write-compatibility/",
                stringify!($profile), "/", stringify!($jdk), "/", stringify!($class), ".class"
            )),
        )
    };
}

const FIXTURES: &[(&str, &str, &str, &[u8])] = &[
    fixture!(debug, javac8, CF11OuterAssignmentTarget),
    fixture!(debug, javac23, CF11OuterAssignmentTarget),
    fixture!(none, javac8, CF11OuterAssignmentTarget),
    fixture!(none, javac23, CF11OuterAssignmentTarget),
    fixture!(debug, javac8, CF11CastReadControl),
    fixture!(debug, javac23, CF11CastReadControl),
    fixture!(none, javac8, CF11CastReadControl),
    fixture!(none, javac23, CF11CastReadControl),
    fixture!(debug, javac8, CF11CharSequenceControl),
    fixture!(debug, javac23, CF11CharSequenceControl),
    fixture!(none, javac8, CF11CharSequenceControl),
    fixture!(none, javac23, CF11CharSequenceControl),
    fixture!(debug, javac8, CF11ReferenceArrayControl),
    fixture!(debug, javac23, CF11ReferenceArrayControl),
    fixture!(none, javac8, CF11ReferenceArrayControl),
    fixture!(none, javac23, CF11ReferenceArrayControl),
    fixture!(debug, javac8, CF11ParameterControl),
    fixture!(debug, javac23, CF11ParameterControl),
    fixture!(none, javac8, CF11ParameterControl),
    fixture!(none, javac23, CF11ParameterControl),
    fixture!(debug, javac8, CF11SlotReuseControl),
    fixture!(debug, javac23, CF11SlotReuseControl),
    fixture!(none, javac8, CF11SlotReuseControl),
    fixture!(none, javac23, CF11SlotReuseControl),
    fixture!(debug, javac8, CF11HandlerWriteControl),
    fixture!(debug, javac23, CF11HandlerWriteControl),
    fixture!(none, javac8, CF11HandlerWriteControl),
    fixture!(none, javac23, CF11HandlerWriteControl),
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task default is a bounded budget")
}

fn opened(bytes: &[u8], internal_name: &str) -> (ArtifactSnapshot, ClassSourceRequest) {
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the frozen complete class opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(internal_name),
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
    (snapshot, request)
}

fn recover(
    snapshot: &ArtifactSnapshot,
    request: &ClassSourceRequest,
    evidence: &RecoveryEvidenceRequest,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(slice::from_ref(snapshot), request, evidence, &mut budget())
        .expect("one fixed complete class answers the public API request")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("one fixed class binds exactly once: {other:?}"),
    }
}

fn named_method<'a>(
    report: &'a ClassSourceReport,
    name: &str,
    descriptor: &str,
) -> (&'a ClassSourceMethod, &'a RecoveryReport) {
    let method = report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == name.as_bytes()
                && method.item.descriptor.raw().0 == descriptor.as_bytes()
        })
        .unwrap_or_else(|| panic!("physical UTF-8 member {name}{descriptor} is present"));
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("{name}{descriptor} has a method-local result: {:?}", method.outcome);
    };
    (method, report)
}

fn no_rule_details() -> RecoveryEvidenceRequest {
    RecoveryEvidenceRequest::essential()
        .with_kind(RecoveryEvidenceKind::SourceMap)
        .with_kind(RecoveryEvidenceKind::RegionDetails)
        .with_kind(RecoveryEvidenceKind::NameDetails)
}

fn assert_complete_java(body: &RecoveryReport, context: &str) {
    assert!(body.produced(), "{context}: {:?}", body.outcome);
    assert_eq!(body.representation, Representation::Java, "{context}");
    assert_eq!(body.quality, Quality::Structured, "{context}");
    assert!(matches!(&body.execution, ExecutionReport::Complete { .. }), "{context}");
}

fn assert_utf8_map(body: &RecoveryReport, context: &str) {
    for segment in body.source_map.segments() {
        assert!(segment.start() < segment.end(), "{context}: {segment:?}");
        assert!(body.text.is_char_boundary(segment.start()), "{context}: {segment:?}");
        assert!(body.text.is_char_boundary(segment.end()), "{context}: {segment:?}");
        assert!(body.text.get(segment.start()..segment.end()).is_some(), "{context}: {segment:?}");
    }
}

fn assert_full_physical_map(
    bytes: &[u8],
    method: &ClassSourceMethod,
    body: &RecoveryReport,
    name: &str,
    descriptor: &str,
) {
    // Member names/descriptors remain the exact decoded class-file UTF-8 byte strings.
    assert_eq!(method.item.name.raw().0, name.as_bytes());
    assert_eq!(method.item.descriptor.raw().0, descriptor.as_bytes());
    let mut bytecode_budget = budget();
    let physical = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut bytecode_budget,
    )
    .expect("the selected method's physical instructions are inspectable");
    assert!(physical.stopped_at.is_none(), "{:?}", physical.stopped_at);
    assert!(!physical.instructions.is_empty());
    for instruction in &physical.instructions {
        let segments = body.source_map.of_bci(instruction.bci);
        assert!(!segments.is_empty(), "unmapped physical BCI {}", instruction.bci);
        assert!(
            segments.iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| {
                        origin.bci() == instruction.bci
                            && origin.method() == Some(&method.item.identity)
                            && (instruction.constant_pool_index.is_none()
                                || origin.cp() == instruction.constant_pool_index)
                    })
            }),
            "physical BCI {} / CP {:?} has no exact method origin: {segments:?}",
            instruction.bci,
            instruction.constant_pool_index
        );
    }
}

fn object_local_declared(body: &RecoveryReport) -> String {
    body.text
        .lines()
        .find_map(|line| {
            let rest = line.trim().strip_prefix("java.lang.Object ")?;
            let name = rest.split([' ', '=', ';', '[']).next()?;
            (!name.is_empty()).then(|| name.to_owned())
        })
        .expect("the planned no-cast contract declares an Object target")
}

#[test]
fn no_cast_iterator_assignment_preserves_object_target_and_full_source_map() {
    const METHOD: &str = "noCastAssign";
    const DESCRIPTOR: &str = "(Ljava/util/List;)Ljava/lang/Object;";
    for (profile, jdk, class, bytes) in FIXTURES.iter().filter(|row| row.2 == "CF11OuterAssignmentTarget") {
        let context = format!("{profile}/{jdk}/{class}/{METHOD}");
        let internal_name = format!("cf11/{class}");
        let (snapshot, request) = opened(bytes, &internal_name);
        let default = match Engine::new()
            .class_source(slice::from_ref(&snapshot), &request, &mut budget())
            .expect("default public request answers")
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("{context}: {other:?}"),
        };
        let all = recover(&snapshot, &request, &RecoveryEvidenceRequest::all());
        let plain = recover(&snapshot, &request, &no_rule_details());
        let (default_method, default_body) = named_method(&default, METHOD, DESCRIPTOR);
        let (all_method, all_body) = named_method(&all, METHOD, DESCRIPTOR);
        let (plain_method, plain_body) = named_method(&plain, METHOD, DESCRIPTOR);
        for body in [default_body, all_body, plain_body] {
            assert!(matches!(&body.execution, ExecutionReport::Complete { .. }), "{context}");
            assert_eq!(body.representation, Representation::Java, "{context}");
            assert_utf8_map(body, &context);
        }
        assert_eq!(default.text, all.text, "{context}");
        assert_eq!(default.text, plain.text, "{context}");
        assert_eq!(default_body.text, all_body.text, "{context}");
        assert_eq!(default_body.text, plain_body.text, "{context}");
        assert_eq!(default_body.source_map, all_body.source_map, "{context}");
        assert_eq!(default_body.source_map, plain_body.source_map, "{context}");
        assert_eq!(default_method.item.identity, all_method.item.identity, "{context}");
        assert_eq!(default_method.item.identity, plain_method.item.identity, "{context}");

        let local = object_local_declared(all_body);
        let lines: Vec<_> = all_body.text.lines().map(str::trim).collect();
        assert!(
            lines.iter().any(|line| line.contains(&format!("{local} = \"seed\";"))),
            "{context}: Object local receives the original seed:\n{}",
            all_body.text
        );
        assert!(
            lines.iter().any(|line| line.contains(&format!("{local} = ")) && line.contains(".next()")),
            "{context}: Object local receives iterator.next() without a cast:\n{}",
            all_body.text
        );
        assert!(
            lines.iter().any(|line| *line == format!("return {local};")),
            "{context}: method returns the same Object local:\n{}",
            all_body.text
        );
        assert_full_physical_map(bytes, all_method, all_body, METHOD, DESCRIPTOR);
    }
}

#[test]
fn reference_write_controls_keep_complete_public_results_across_evidence_requests() {
    const CONTROLS: &[(&str, &str, &str, &str)] = &[
        ("CF11OuterAssignmentTarget", "casted loop positive", "positive", "(Ljava/util/List;)Ljava/lang/String;"),
        ("CF11OuterAssignmentTarget", "effect before assignment", "bodyEffectBeforeAssign", "(Ljava/util/List;)Ljava/lang/String;"),
        ("CF11OuterAssignmentTarget", "extra next consumption", "extraNextConsumption", "(Ljava/util/List;)Ljava/lang/String;"),
        ("CF11CastReadControl", "explicit-cast read", "castRead", "(Ljava/util/List;)I"),
        ("CF11CharSequenceControl", "interface target", "builderLength", "()I"),
        ("CF11ReferenceArrayControl", "reference-array target", "lengthAt", "(Z)I"),
        ("CF11ParameterControl", "parameter target", "replace", "(Ljava/lang/Object;Ljava/util/List;)Ljava/lang/Object;"),
        ("CF11SlotReuseControl", "slot-reuse target", "afterStringLifetime", "(ZLjava/util/List;)Ljava/lang/Object;"),
        ("CF11HandlerWriteControl", "handler target", "handlerWrite", "()Ljava/lang/Object;"),
    ];
    for (profile, jdk, class, bytes) in FIXTURES {
        let Some((_, label, method_name, descriptor)) = CONTROLS.iter().find(|(target, _, _, _)| *target == *class) else {
            continue;
        };
        let context = format!("{profile}/{jdk}/{class}/{label}");
        let (snapshot, request) = opened(bytes, &format!("cf11/{class}"));
        let default = match Engine::new()
            .class_source(slice::from_ref(&snapshot), &request, &mut budget())
            .expect("default control request answers")
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("{context}: {other:?}"),
        };
        let all = recover(&snapshot, &request, &RecoveryEvidenceRequest::all());
        let plain = recover(&snapshot, &request, &no_rule_details());
        let (default_method, default_body) = named_method(&default, method_name, descriptor);
        let (all_method, all_body) = named_method(&all, method_name, descriptor);
        let (plain_method, plain_body) = named_method(&plain, method_name, descriptor);
        // Planned boundary: freeze exact control source expectations from a fresh public-API
        // baseline before tightening these assertions. This draft only requires evidence-invariant
        // complete outputs and never infers a candidate's internal proof gate.
        for body in [default_body, all_body, plain_body] {
            assert!(matches!(&body.execution, ExecutionReport::Complete { .. }), "{context}");
            assert_eq!(body.representation, Representation::Java, "{context}");
            assert_utf8_map(body, &context);
        }
        assert_eq!(default.text, all.text, "{context}");
        assert_eq!(default.text, plain.text, "{context}");
        assert_eq!(default_body.text, all_body.text, "{context}");
        assert_eq!(default_body.text, plain_body.text, "{context}");
        assert_eq!(default_body.source_map, all_body.source_map, "{context}");
        assert_eq!(default_body.source_map, plain_body.source_map, "{context}");
        assert_eq!(default_method.item.identity, all_method.item.identity, "{context}");
        assert_eq!(default_method.item.identity, plain_method.item.identity, "{context}");
    }
}

#[test]
fn pre_cancel_does_not_publish_partial_class_or_method_artifacts() {
    for (profile, jdk, class, bytes) in FIXTURES {
        let context = format!("{profile}/{jdk}/{class}");
        let (snapshot, request) = opened(bytes, &format!("cf11/{class}"));
        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
        let outcome = Engine::new()
            .class_source_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut cancelled,
            )
            .expect("pre-cancel is an ordinary public execution state");
        let execution = match &outcome {
            OperationOutcome::Incomplete(report) => &report.execution,
            OperationOutcome::Performed(report) => &report.execution,
            OperationOutcome::Ambiguous(candidates) => panic!("{context}: {candidates:?}"),
        };
        let ExecutionReport::Cancelled { usage } = execution else {
            panic!("{context}: expected cancellation, got {execution:?}");
        };
        let mut counted = usage.clone();
        counted.elapsed_millis = 0;
        assert_eq!(counted, UsageSnapshot::default(), "{context}");
        match outcome {
            OperationOutcome::Incomplete(report) => assert!(report.candidates.is_empty(), "{context}"),
            OperationOutcome::Performed(report) => {
                assert!(report.text.is_empty(), "{context}");
                assert!(report.methods.is_empty(), "{context}");
                assert!(report.fields.is_empty(), "{context}");
            }
            OperationOutcome::Ambiguous(_) => unreachable!(),
        }
    }
}

/// Temporary root probe: measure a complete target method, then rerun with one fewer analysis step.
/// Do not add numeric expectations until root observes this candidate/profile's actual usage.
#[allow(dead_code)]
fn late_stop_measurement_probe(
    bytes: &[u8],
    internal_name: &str,
    method_name: &str,
    descriptor: &str,
) -> (u64, OperationOutcome<ClassSourceReport>) {
    let (snapshot, request) = opened(bytes, internal_name);
    let complete = recover(&snapshot, &request, &RecoveryEvidenceRequest::all());
    let (_, body) = named_method(&complete, method_name, descriptor);
    let ExecutionReport::Complete { usage } = &body.execution else {
        panic!("the probe needs an observed complete method usage: {:?}", body.execution);
    };
    let stop_limit = usage.analysis_steps.saturating_sub(1);
    let mut limits = budget().limits().clone();
    limits.analysis_steps = stop_limit;
    let outcome = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits),
        )
        .expect("late-stop probing returns the ordinary execution state");
    (stop_limit, outcome)
}

#[allow(dead_code)]
fn planned_target_stop_shape(body: &RecoveryReport) -> bool {
    matches!(
        &body.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps
            },
            ..
        }
    ) && !body.produced()
        && body.text.is_empty()
        && body.source_map.is_empty()
        && body.content == RecoveryContent::NotProduced
        && body.news.is_empty()
}
