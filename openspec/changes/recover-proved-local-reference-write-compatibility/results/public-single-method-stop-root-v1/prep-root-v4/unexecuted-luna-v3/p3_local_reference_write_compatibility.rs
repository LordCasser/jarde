//! Planned public-API regression for local reference writes across debug profiles.
//!
//! Fixture files are copied from the exact javac8/javac23 original legs named in README.md.
//! This is a private draft: control text or internal proof gates are intentionally not guessed.

use jarde::*;
use jarde_jvm::ir::{Quality, Representation};
use jarde_reader::budget::{Budget, CancellationToken, UsageSnapshot};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::ExecutionReport;
use std::slice;

macro_rules! fixture {
    ($profile:ident, $jdk:ident, $class:ident) => {
        (
            stringify!($profile),
            stringify!($jdk),
            stringify!($class),
            include_bytes!(concat!(
                "fixtures/p3-local-reference-write-compatibility/",
                stringify!($profile),
                "/",
                stringify!($jdk),
                "/",
                stringify!($class),
                ".class"
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
        panic!(
            "{name}{descriptor} has a method-local result: {:?}",
            method.outcome
        );
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
    assert!(
        matches!(&body.execution, ExecutionReport::Complete { .. }),
        "{context}"
    );
}

fn assert_class_identity(
    report: &ClassSourceReport,
    method: &ClassSourceMethod,
    expected_internal_name: &str,
    bytes: &[u8],
    context: &str,
) {
    let declaration = report
        .declaration
        .as_ref()
        .expect("a complete public class-source report has its class declaration");
    assert_eq!(
        report.class, declaration.item.definition,
        "{context}: source class identity"
    );
    assert_eq!(
        report.class, method.item.identity.owner,
        "{context}: method owner identity"
    );
    assert_eq!(
        declaration.item.declaration.this_class.raw().0,
        expected_internal_name.as_bytes(),
        "{context}: this_class matches the name query",
    );
    assert_eq!(
        report.class.class_bytes.length,
        bytes.len() as u64,
        "{context}: physical identity binds the complete input class bytes",
    );
}

fn assert_control_shape(body: &RecoveryReport, context: &str) {
    assert!(body.produced(), "{context}: {:?}", body.outcome);
    assert!(
        matches!(&body.execution, ExecutionReport::Complete { .. }),
        "{context}"
    );
    match (body.quality, body.representation) {
        (Quality::Structured, Representation::Java) => {}
        (Quality::Fallback, Representation::Mixed) => {}
        (quality, representation) => {
            panic!("{context}: unsupported observed control shape {quality:?}/{representation:?}")
        }
    }
}

fn assert_utf8_map(body: &RecoveryReport, context: &str) {
    for segment in body.source_map.segments() {
        assert!(segment.start() < segment.end(), "{context}: {segment:?}");
        assert!(
            body.text.is_char_boundary(segment.start()),
            "{context}: {segment:?}"
        );
        assert!(
            body.text.is_char_boundary(segment.end()),
            "{context}: {segment:?}"
        );
        assert!(
            body.text.get(segment.start()..segment.end()).is_some(),
            "{context}: {segment:?}"
        );
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
        assert!(
            !segments.is_empty(),
            "unmapped physical BCI {}",
            instruction.bci
        );
        assert!(
            segments.iter().any(|segment| {
                std::iter::once(segment.origin().primary())
                    .chain(segment.origin().derived())
                    .any(|origin| {
                        origin.bci() == instruction.bci
                            && origin.method() == Some(&method.item.identity)
                            // Literal origins can omit a CP anchor. An attached CP anchor must
                            // still agree with this exact physical instruction's pool operand.
                            && (origin.cp().is_none()
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
    for (profile, jdk, class, bytes) in FIXTURES
        .iter()
        .filter(|row| row.2 == "CF11OuterAssignmentTarget")
    {
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
            assert_complete_java(body, &context);
            assert_utf8_map(body, &context);
        }
        assert_class_identity(&all, all_method, &internal_name, bytes, &context);
        assert_eq!(default.text, all.text, "{context}");
        assert_eq!(default.text, plain.text, "{context}");
        assert_eq!(default_body.text, all_body.text, "{context}");
        assert_eq!(default_body.text, plain_body.text, "{context}");
        assert_eq!(default_body.source_map, all_body.source_map, "{context}");
        assert_eq!(default_body.source_map, plain_body.source_map, "{context}");
        assert_eq!(
            default_method.item.identity, all_method.item.identity,
            "{context}"
        );
        assert_eq!(
            default_method.item.identity, plain_method.item.identity,
            "{context}"
        );
        assert_class_identity(&default, default_method, &internal_name, bytes, &context);
        assert_class_identity(&plain, plain_method, &internal_name, bytes, &context);

        let local = object_local_declared(all_body);
        let lines: Vec<_> = all_body.text.lines().map(str::trim).collect();
        assert!(
            lines
                .iter()
                .any(|line| line.contains(&format!("{local} = \"seed\";"))),
            "{context}: Object local receives the original seed:\n{}",
            all_body.text
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains(&format!("{local} = ")) && line.contains(".next()")),
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
fn measured_public_single_method_late_stop_does_not_publish_partial_target_artifacts() {
    const METHOD: &str = "noCastAssign";
    const DESCRIPTOR: &str = "(Ljava/util/List;)Ljava/lang/Object;";

    for (profile, jdk, class, bytes) in FIXTURES
        .iter()
        .copied()
        .filter(|row| row.2 == "CF11OuterAssignmentTarget")
    {
        let context = format!("{profile}/{jdk}/{class}/{METHOD}");
        let internal_name = format!("cf11/{class}");
        let (snapshot, class_request) = opened(bytes, &internal_name);

        // The class-source path names the exact physical member. Measure only the public
        // single-method request below, using that same snapshot, environment and identity.
        let complete_class = recover(&snapshot, &class_request, &RecoveryEvidenceRequest::all());
        let (class_method, class_body) = named_method(&complete_class, METHOD, DESCRIPTOR);
        assert_complete_java(class_body, &context);
        let expected_identity = class_method.item.identity.clone();
        let method_request = jarde::ir::MethodAnalysisRequest {
            environment: class_request
                .environment
                .build(slice::from_ref(&snapshot))
                .expect("the opened fixture request builds its same environment"),
            method: expected_identity.clone(),
            stages: jarde::ir::AnalysisStage::ALL.to_vec(),
        };

        let mut complete_budget = budget();
        let complete = Engine::new()
            .recover_method_with_evidence(
                slice::from_ref(&snapshot),
                &method_request,
                &RecoveryEvidenceRequest::all(),
                &mut complete_budget,
            )
            .expect("the exact physical method completes through the public single-method API");
        assert_eq!(complete.analysis().method, expected_identity, "{context}");
        assert_complete_java(complete.recovery(), &context);
        let measured_total = complete_budget.usage();
        let ExecutionReport::Complete { usage: body_usage } = &complete.recovery().execution else {
            panic!("{context}: full method body has usage: {:?}", complete.recovery().execution);
        };
        eprintln!(
            "[local-reference-write-public-method-stop-baseline] {context} external_total={} body_usage={}",
            measured_total.analysis_steps,
            body_usage.analysis_steps,
        );
        assert!(
            measured_total.analysis_steps > 1,
            "{context}: external Budget measured real method work: {measured_total:?}"
        );

        let analysis_step_limit = measured_total.analysis_steps - 1;
        let mut limits = budget().limits().clone();
        limits.analysis_steps = analysis_step_limit;
        let mut stopped_budget = Budget::new(limits);
        let stopped = Engine::new()
            .recover_method_with_evidence(
                slice::from_ref(&snapshot),
                &method_request,
                &RecoveryEvidenceRequest::all(),
                &mut stopped_budget,
            )
            .expect("the single-method budget stop is returned as a report");
        assert_eq!(stopped.analysis().method, expected_identity, "{context}");
        assert!(
            matches!(
                &stopped.analysis().execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: BudgetDimension::AnalysisSteps
                    },
                    ..
                }
            ),
            "{context}: analysis reports an AnalysisSteps stop: {:?}",
            stopped.analysis().execution
        );
        let body = stopped.recovery();
        assert!(
            matches!(
                &body.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded {
                        dimension: BudgetDimension::AnalysisSteps
                    },
                    ..
                }
            ),
            "{context}: recovery reports an AnalysisSteps stop: {:?}",
            body.execution
        );
        let stop = body.stop().expect("the stopped method body carries its public stop");
        assert!(stop.at().is_some(), "{context}: stop has a location: {stop:?}");
        let stopped_total = stopped_budget.usage();
        assert!(
            stopped_total.analysis_steps <= analysis_step_limit,
            "{context}: stopped external budget respects its limit: {stopped_total:?}"
        );
        let ExecutionReport::Partial {
            usage: body_partial_usage,
            ..
        } = &body.execution
        else {
            unreachable!("the recovery report was checked as partial above")
        };
        eprintln!(
            "[local-reference-write-public-method-stop] {context} measured_total={} limit={} stopped_total={} body_partial={} stop={stop:?}",
            measured_total.analysis_steps,
            analysis_step_limit,
            stopped_total.analysis_steps,
            body_partial_usage.analysis_steps,
        );

        // This public API test checks method-level atomicity, not the exact instruction where
        // analysis stops. The private core test covers the final local Return consumer directly.
        assert!(!body.produced(), "{context}");
        assert_eq!(body.text, "", "{context}");
        assert!(body.source_map.is_empty(), "{context}");
        assert_eq!(body.content, RecoveryContent::NotProduced, "{context}");
        assert!(body.news.is_empty(), "{context}");
    }
}

#[test]
fn reference_write_controls_keep_complete_public_results_across_evidence_requests() {
    const CONTROLS: &[(&str, &str, &str, &str)] = &[
        (
            "CF11OuterAssignmentTarget",
            "casted loop positive",
            "positive",
            "(Ljava/util/List;)Ljava/lang/String;",
        ),
        (
            "CF11OuterAssignmentTarget",
            "effect before assignment",
            "bodyEffectBeforeAssign",
            "(Ljava/util/List;)Ljava/lang/String;",
        ),
        (
            "CF11OuterAssignmentTarget",
            "extra next consumption",
            "extraNextConsumption",
            "(Ljava/util/List;)Ljava/lang/String;",
        ),
        (
            "CF11CastReadControl",
            "explicit-cast read",
            "castRead",
            "(Ljava/util/List;)I",
        ),
        (
            "CF11CharSequenceControl",
            "interface target",
            "builderLength",
            "()I",
        ),
        (
            "CF11ReferenceArrayControl",
            "reference-array target",
            "lengthAt",
            "(Z)I",
        ),
        (
            "CF11ParameterControl",
            "parameter target",
            "replace",
            "(Ljava/lang/Object;Ljava/util/List;)Ljava/lang/Object;",
        ),
        (
            "CF11SlotReuseControl",
            "slot-reuse target",
            "afterStringLifetime",
            "(ZLjava/util/List;)Ljava/lang/Object;",
        ),
        (
            "CF11HandlerWriteControl",
            "handler target",
            "handlerWrite",
            "()Ljava/lang/Object;",
        ),
    ];
    for (profile, jdk, class, bytes) in FIXTURES {
        for (_, label, method_name, descriptor) in CONTROLS
            .iter()
            .filter(|(target, _, _, _)| *target == *class)
        {
            let context = format!("{profile}/{jdk}/{class}/{label}");
            let internal_name = format!("cf11/{class}");
            let (snapshot, request) = opened(bytes, &internal_name);
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
                assert_control_shape(body, &context);
                assert_utf8_map(body, &context);
            }
            assert_eq!(default_body.quality, all_body.quality, "{context}");
            assert_eq!(default_body.quality, plain_body.quality, "{context}");
            assert_eq!(
                default_body.representation, all_body.representation,
                "{context}"
            );
            assert_eq!(
                default_body.representation, plain_body.representation,
                "{context}"
            );
            assert_eq!(default_body.content, all_body.content, "{context}");
            assert_eq!(default_body.content, plain_body.content, "{context}");
            assert_eq!(default.text, all.text, "{context}");
            assert_eq!(default.text, plain.text, "{context}");
            assert_eq!(default_body.text, all_body.text, "{context}");
            assert_eq!(default_body.text, plain_body.text, "{context}");
            assert_eq!(default_body.source_map, all_body.source_map, "{context}");
            assert_eq!(default_body.source_map, plain_body.source_map, "{context}");
            assert_eq!(
                default_method.item.identity, all_method.item.identity,
                "{context}"
            );
            assert_eq!(
                default_method.item.identity, plain_method.item.identity,
                "{context}"
            );
            assert_class_identity(&default, default_method, &internal_name, bytes, &context);
            assert_class_identity(&all, all_method, &internal_name, bytes, &context);
            assert_class_identity(&plain, plain_method, &internal_name, bytes, &context);
        }
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
            OperationOutcome::Incomplete(report) => {
                assert!(report.candidates.is_empty(), "{context}")
            }
            OperationOutcome::Performed(report) => {
                assert!(report.text.is_empty(), "{context}");
                assert!(report.methods.is_empty(), "{context}");
                assert!(report.fields.is_empty(), "{context}");
            }
            OperationOutcome::Ambiguous(_) => unreachable!(),
        }
    }
}
