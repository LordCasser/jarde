//! `preserve-lambda-descriptor-adaptation`: complete Java 8 class-source fixtures for proven
//! method-reference and captured lambda adaptation shapes. Support and runner classes stay
//! source-only; the primary method-reference probe has no generated `lambda$` helper methods.

use jarde::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::slice;
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-lambda-adaptation/v8/LambdaAdaptationProbe.class");
const CAPTURE_FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-lambda-adaptation/v8/CapturedLambdaAdaptationProbe.class");
const BOUND_NULL_FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-lambda-adaptation/v8/BoundNullLambdaAdaptationProbe.class");
const PROBE_SOURCE: &str = include_str!("fixtures/p3-lambda-adaptation/LambdaAdaptationProbe.java");
const SUPPORT_SOURCE: &str =
    include_str!("fixtures/p3-lambda-adaptation/LambdaAdaptationSupport.java");
const BOX_SOURCE: &str = include_str!("fixtures/p3-lambda-adaptation/LambdaAdaptationBox.java");
const RUNNER_SOURCE: &str =
    include_str!("fixtures/p3-lambda-adaptation/LambdaAdaptationRunner.java");
const CAPTURE_SOURCE: &str =
    include_str!("fixtures/p3-lambda-adaptation/CapturedLambdaAdaptationProbe.java");
const BOUND_NULL_SOURCE: &str =
    include_str!("fixtures/p3-lambda-adaptation/BoundNullLambdaAdaptationProbe.java");
const CAPTURE_SUPPORT_SOURCE: &str =
    include_str!("fixtures/p3-lambda-adaptation/LambdaCaptureAdaptationSupport.java");
const CAPTURE_RUNNER_SOURCE: &str =
    include_str!("fixtures/p3-lambda-adaptation/CapturedLambdaAdaptationRunner.java");

const METHODS: [&str; 9] = [
    "<init>",
    "strings",
    "wider",
    "arrays",
    "unbound",
    "constructor",
    "primitive",
    "primitiveValue",
    "genericString",
];

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed lambda-adaptation fixture opens as a standalone CLASS")
}

fn class_source_of(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    class_source_of_named(snapshot, evidence, "LambdaAdaptationProbe")
}

fn class_source_of_named(
    snapshot: &ArtifactSnapshot,
    evidence: RecoveryEvidenceRequest,
    class_name: &str,
) -> ClassSourceReport {
    let mut budget = budget();
    let request = ClassSourceRequest {
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
    match Engine::new()
        .class_source_with_evidence(slice::from_ref(snapshot), &request, &evidence, &mut budget)
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

fn recovered_method(
    snapshot: &ArtifactSnapshot,
    member: &ClassSourceMethod,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> RecoveredMethod {
    recover_method_result(snapshot, member, evidence, budget)
        .expect("the one selected method recovery is returned")
}

fn recover_method_result(
    snapshot: &ArtifactSnapshot,
    member: &ClassSourceMethod,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> jarde::Result<RecoveredMethod> {
    recover_method_result_named(snapshot, member, evidence, budget, "LambdaAdaptationProbe")
}

fn recover_method_result_named(
    snapshot: &ArtifactSnapshot,
    member: &ClassSourceMethod,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
    class_name: &str,
) -> jarde::Result<RecoveredMethod> {
    let environment = ClassSourceRequest {
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
    .environment
    .build(slice::from_ref(snapshot))
    .expect("the fixture's single-class environment is valid");
    let request = jarde::ir::MethodAnalysisRequest {
        environment,
        method: member.item.identity.clone(),
        stages: jarde::ir::AnalysisStage::ALL.to_vec(),
    };
    Engine::new().recover_method_with_evidence(
        slice::from_ref(snapshot),
        &request,
        evidence,
        budget,
    )
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
fn all_fixture_methods_are_structured() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);
    assert!(
        report.fields.is_empty(),
        "the permanent probe has no fields"
    );
}

#[test]
fn recovered_text_keeps_dynamic_checks_receiver_order_and_controls() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);

    let strings = &recovered(&report, "strings").text;
    assert!(strings.contains("LambdaAdaptationSupport"), "{strings}");
    assert!(strings.contains("pick"), "{strings}");
    assert!(
        strings.contains("java.lang.String"),
        "dynamic String check was lost:\n{strings}"
    );
    assert!(
        strings.contains("(java.lang.Object p0) ->"),
        "the explicit lambda parameter must use the erased SAM type:\n{strings}"
    );
    assert!(
        strings.contains("pick((java.lang.String) p0)"),
        "the dynamic String check must be inside the lambda body:\n{strings}"
    );

    let wider = &recovered(&report, "wider").text;
    assert!(wider.contains("LambdaAdaptationSupport"), "{wider}");
    assert!(wider.contains("wider"), "{wider}");
    assert!(
        wider.contains("java.lang.String"),
        "dynamic String check was lost:\n{wider}"
    );
    assert!(
        wider.contains("java.lang.Object"),
        "implementation Object type was lost:\n{wider}"
    );
    assert!(
        wider.contains("wider((java.lang.Object) (java.lang.String) p0)"),
        "the dynamic check must precede the outer cast that fixes the Object overload:\n{wider}"
    );

    let arrays = &recovered(&report, "arrays").text;
    assert!(arrays.contains("pickArray"), "{arrays}");
    assert!(
        arrays.contains("java.lang.String[]"),
        "array component check was lost:\n{arrays}"
    );
    assert!(
        arrays.contains("pickArray((java.lang.String[]) p0)"),
        "the array check must be an explicit parameter adaptation:\n{arrays}"
    );

    let unbound = &recovered(&report, "unbound").text;
    assert!(unbound.contains("instancePick"), "{unbound}");
    assert!(unbound.contains("LambdaAdaptationSupport"), "{unbound}");
    assert!(
        unbound.contains("java.lang.String"),
        "unbound dynamic check was lost:\n{unbound}"
    );
    assert!(
        unbound.contains("((LambdaAdaptationSupport) p0).instancePick((java.lang.String) p1)"),
        "the unbound receiver and method argument must keep their input order:\n{unbound}"
    );

    let constructor = &recovered(&report, "constructor").text;
    assert!(
        constructor.contains("new LambdaAdaptationBox"),
        "{constructor}"
    );
    assert!(
        constructor.contains("java.lang.String"),
        "constructor parameter type was lost:\n{constructor}"
    );
    assert!(
        constructor.contains("new LambdaAdaptationBox((java.lang.String) p0)"),
        "the dynamic check must run before constructor invocation:\n{constructor}"
    );

    let primitive = &recovered(&report, "primitive").text;
    assert!(primitive.contains("primitiveValue"), "{primitive}");
    assert!(
        primitive.contains("LambdaAdaptationProbe::primitiveValue"),
        "the identity primitive control should remain a method reference:\n{primitive}"
    );
    let primitive_value = &recovered(&report, "primitiveValue").text;
    assert!(primitive_value.contains("+ 1"), "{primitive_value}");

    let generic = &recovered(&report, "genericString").text;
    assert!(generic.contains("genericValue"), "{generic}");
    assert!(
        generic.contains("LambdaAdaptationSupport::genericValue"),
        "the raw Supplier return control should remain a method reference:\n{generic}"
    );
    assert!(
        !generic.contains("((java.lang.String)"),
        "the generic return control gained an unsupported body cast:\n{generic}"
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

#[test]
fn one_adapted_method_keeps_the_real_site_origin_and_evidence_independent_body() {
    let snapshot = open(FIXTURE);
    let inventory = class_source_of(&snapshot, RecoveryEvidenceRequest::essential());
    let selected = method(&inventory, "strings");
    let mut essential_budget = budget();
    let essential = recovered_method(
        &snapshot,
        selected,
        &RecoveryEvidenceRequest::essential(),
        &mut essential_budget,
    );
    let mut all_budget = budget();
    let all = recovered_method(
        &snapshot,
        selected,
        &RecoveryEvidenceRequest::all(),
        &mut all_budget,
    );
    let essential = essential.recovery();
    let all = all.recovery();

    assert_eq!(
        essential.text, all.text,
        "evidence selection changes no body"
    );
    assert!(essential.text.contains("(java.lang.Object p0) ->"));
    assert!(essential.text.contains("pick((java.lang.String) p0)"));
    assert!(essential.source_map.is_empty());
    assert_eq!(
        essential.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::NotRequested
    );
    assert_eq!(
        all.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    );

    // In this frozen method the only dynamic instruction is invokedynamic at BCI 0, CP #7;
    // the factory descriptor has no captured arguments. The inserted parameter cast therefore
    // keeps the real site origin and must not claim a synthetic checkcast BCI or a capture origin.
    let site_segments = all
        .source_map
        .segments()
        .iter()
        .filter(|segment| {
            let site = segment.origin().primary();
            site.bci() == 0
                && site.cp() == Some(7)
                && site.method() == Some(&selected.item.identity)
        })
        .collect::<Vec<_>>();
    assert!(
        !site_segments.is_empty(),
        "real lambda site origin was lost"
    );
    assert!(
        site_segments
            .iter()
            .any(|segment| segment.text(&all.text).contains("(java.lang.String) p0")),
        "the emitted dynamic cast is not mapped to invokedynamic #7"
    );
    assert!(
        all.source_map
            .segments()
            .iter()
            .all(|segment| segment.origin().derived().is_empty()),
        "the zero-capture site must not invent capture origins"
    );
}

#[test]
fn captured_lambda_adaptation_keeps_the_site_and_real_capture_producer_origins() {
    let snapshot = open(CAPTURE_FIXTURE);
    let inventory = class_source_of_named(
        &snapshot,
        RecoveryEvidenceRequest::essential(),
        "CapturedLambdaAdaptationProbe",
    );
    let selected = method(&inventory, "captured");
    let mut essential_budget = budget();
    let essential = recover_method_result_named(
        &snapshot,
        selected,
        &RecoveryEvidenceRequest::essential(),
        &mut essential_budget,
        "CapturedLambdaAdaptationProbe",
    )
    .expect("captured adapter recovery is returned")
    .recovery()
    .clone();
    let mut all_budget = budget();
    let all = recover_method_result_named(
        &snapshot,
        selected,
        &RecoveryEvidenceRequest::all(),
        &mut all_budget,
        "CapturedLambdaAdaptationProbe",
    )
    .expect("captured adapter recovery with source evidence is returned")
    .recovery()
    .clone();

    assert_eq!(
        essential.text, all.text,
        "evidence selection changes no body"
    );
    assert!(
        essential.text.contains("java.lang.String"),
        "{}",
        essential.text
    );
    assert!(
        essential.text.contains("lambda$captured$0"),
        "the recovered use site still calls its generated implementation helper:\n{}",
        essential.text
    );
    assert!(
        essential.text.contains("(java.lang.String) p0"),
        "the erased SAM input must receive a dynamic String check:\n{}",
        essential.text
    );
    assert!(essential.source_map.is_empty());
    assert_eq!(
        all.evidence.state(RecoveryEvidenceKind::SourceMap),
        EvidenceState::Complete
    );

    // javac emits the prefix producer at BCI 0, stores it at BCI 3, loads the captured operand
    // at BCI 4, and executes the site at BCI 5, CP #13. The generated Java cast has no checkcast
    // instruction: its primary origin is the site, with capture load BCI 4 derived on the lambda.
    let adapted = all
        .source_map
        .segments()
        .iter()
        .filter(|segment| segment.text(&all.text).contains("(java.lang.String) p0"))
        .collect::<Vec<_>>();
    assert!(
        !adapted.is_empty(),
        "dynamic cast has no source-map segment"
    );
    assert!(
        adapted.iter().any(|segment| {
            let origin = segment.origin();
            origin.primary().bci() == 5
                && origin.primary().cp() == Some(13)
                && origin.primary().method() == Some(&selected.item.identity)
                && origin.derived().iter().any(|derived| {
                    derived.bci() == 4 && derived.method() == Some(&selected.item.identity)
                })
        }),
        "cast source must retain site CP #13 and captured producer BCI 4: {:?}",
        adapted
            .iter()
            .map(|segment| segment.origin())
            .collect::<Vec<_>>()
    );
}

#[test]
fn bound_null_control_keeps_its_creation_time_refusal() {
    let snapshot = open(BOUND_NULL_FIXTURE);
    let report = class_source_of_named(
        &snapshot,
        RecoveryEvidenceRequest::all(),
        "BoundNullLambdaAdaptationProbe",
    );
    let body = recovered(&report, "boundNull");
    assert!(
        !body.text.contains("->"),
        "a bound-null factory must not be rewritten as a deferred lambda:\n{}",
        body.text
    );
    assert!(
        body.text
            .contains("adapting this bound receiver would move its null failure"),
        "the method-reference creation-time check remains an explicit refusal:\n{}",
        body.text
    );
    assert!(
        body.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "jre_lambda_sam_types"
                && diagnostic
                    .message
                    .contains("functional-value creation to invocation")
        }),
        "the bound-null timing refusal keeps its lambda diagnostic: {:?}",
        body.diagnostics
    );
}

#[test]
fn adaptation_budget_stops_and_precancellation_do_not_publish_a_partial_body() {
    let snapshot = open(FIXTURE);
    let inventory = class_source_of(&snapshot, RecoveryEvidenceRequest::essential());
    let selected = method(&inventory, "strings");
    let mut baseline_budget = budget();
    let _baseline = recovered_method(
        &snapshot,
        selected,
        &RecoveryEvidenceRequest::all(),
        &mut baseline_budget,
    );
    let measured = baseline_budget.usage();
    let mut essential_budget = budget();
    let essential = recovered_method(
        &snapshot,
        selected,
        &RecoveryEvidenceRequest::essential(),
        &mut essential_budget,
    );
    assert!(essential.recovery().produced());
    let essential_usage = essential_budget.usage();
    // Search only this selected method under essential evidence, so source-map replay cannot be
    // mistaken for AST construction. A refused two-node Local/Cast batch leaves one IR permit
    // unused, names the invokedynamic at BCI 0, and has already paid for the full descriptor plan.
    let mut adapter_stop = None;
    for limit in 1..essential_usage.ir_items {
        let mut limited = task_budget(&[
            BudgetOverride::new("ir_items", limit).expect("a positive IR limit is valid")
        ])
        .expect("the task budget is valid");
        let Ok(result) = recover_method_result(
            &snapshot,
            selected,
            &RecoveryEvidenceRequest::essential(),
            &mut limited,
        ) else {
            continue;
        };
        let recovery = result.recovery();
        let usage = limited.usage();
        if matches!(
            &recovery.outcome,
            RecoveryOutcome::Stopped(StopReason::Budget {
                dimension: CountedBudgetDimension::IrItems,
                at: Some(0),
                ..
            })
        ) && usage.analysis_steps == essential_usage.analysis_steps
            && usage.ir_items + 1 == limit
            && usage.output_bytes < essential_usage.output_bytes
        {
            assert!(recovery.text.is_empty(), "{recovery:?}");
            assert!(recovery.source_map.is_empty(), "{recovery:?}");
            adapter_stop = Some((limit, usage.ir_items));
            break;
        }
    }
    assert!(
        adapter_stop.is_some(),
        "no IR bound stopped inside the two-node adapter for `strings`: essential usage {essential_usage:?}"
    );
    assert!(
        measured.ir_items > 1,
        "fixture should charge bounded IR work"
    );
    assert!(
        measured.analysis_steps > 1,
        "fixture should charge bounded analysis work"
    );
    assert!(
        measured.output_bytes > 1,
        "fixture should emit a method body"
    );
    for (name, dimension, limit) in [
        ("ir_items", BudgetDimension::IrItems, measured.ir_items - 1),
        (
            "analysis_steps",
            BudgetDimension::AnalysisSteps,
            measured.analysis_steps - 1,
        ),
    ] {
        let mut constrained = task_budget(&[
            BudgetOverride::new(name, limit).expect("a positive budget limit is valid")
        ])
        .expect("the task budget is valid");
        let result = recover_method_result(
            &snapshot,
            selected,
            &RecoveryEvidenceRequest::all(),
            &mut constrained,
        );
        let recovery = match result {
            Ok(result) => result.recovery().clone(),
            Err(Error::BudgetExceeded {
                dimension: actual, ..
            }) => {
                assert_eq!(actual, dimension, "{name} stop dimension");
                continue;
            }
            Err(error) => panic!("{name} produced unexpected error: {error:?}"),
        };
        if recovery.produced() {
            assert!(
                recovery.text.contains("(java.lang.Object p0) ->")
                    && recovery.text.contains("pick((java.lang.String) p0)"),
                "{name} exhaustion published a half-adapted method: {recovery:?}"
            );
        } else {
            assert!(recovery.text.is_empty(), "{name}: {recovery:?}");
            assert!(recovery.source_map.is_empty(), "{name}: {recovery:?}");
        }
        assert!(
            matches!(
                &recovery.execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded { dimension: actual },
                    ..
                } if *actual == dimension
            ),
            "{name} exhaustion must be reported on the selected method: {:?}",
            recovery.execution
        );
    }

    let mut output_limited =
        task_budget(&[
            BudgetOverride::new("output_bytes", measured.output_bytes - 1)
                .expect("a positive output bound is valid"),
        ])
        .expect("the task budget is valid");
    let output_result = recover_method_result(
        &snapshot,
        selected,
        &RecoveryEvidenceRequest::all(),
        &mut output_limited,
    );
    match output_result {
        Ok(result) => {
            let recovery = result.recovery();
            if recovery.produced() {
                assert!(
                    recovery.text.contains("(java.lang.Object p0) ->")
                        && recovery.text.contains("pick((java.lang.String) p0)"),
                    "output exhaustion published a half-adapted method: {recovery:?}"
                );
            } else {
                assert!(recovery.text.is_empty(), "{recovery:?}");
                assert!(recovery.source_map.is_empty(), "{recovery:?}");
            }
            assert!(
                matches!(
                    &recovery.execution,
                    ExecutionReport::Partial {
                        reason: TerminationReason::BudgetExceeded {
                            dimension: BudgetDimension::OutputBytes
                        },
                        ..
                    }
                ),
                "output exhaustion is reported on the selected method: {recovery:?}"
            );
        }
        Err(Error::BudgetExceeded {
            dimension: BudgetDimension::OutputBytes,
            ..
        }) => {}
        Err(error) => panic!("output stop produced unexpected error: {error:?}"),
    }

    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(budget().limits().clone(), token);
    let cancelled_result = recover_method_result(
        &snapshot,
        selected,
        &RecoveryEvidenceRequest::all(),
        &mut cancelled,
    );
    match cancelled_result {
        Ok(cancelled_result) => {
            let recovery = cancelled_result.recovery();
            assert!(
                !recovery.produced()
                    && recovery.text.is_empty()
                    && recovery.source_map.is_empty()
                    && !matches!(&recovery.execution, ExecutionReport::Complete { .. }),
                "pre-cancellation publishes no partially adapted method: {recovery:?}"
            );
        }
        Err(Error::Cancelled { .. }) => {}
        Err(error) => panic!("pre-cancellation produced unexpected error: {error:?}"),
    }
}

#[test]
fn captured_adapter_budget_stops_inside_nested_value_rendering() {
    let snapshot = open(CAPTURE_FIXTURE);
    let inventory = class_source_of_named(
        &snapshot,
        RecoveryEvidenceRequest::essential(),
        "CapturedLambdaAdaptationProbe",
    );
    let selected = method(&inventory, "captured");
    let mut complete_budget = budget();
    let complete = recover_method_result_named(
        &snapshot,
        selected,
        &RecoveryEvidenceRequest::essential(),
        &mut complete_budget,
        "CapturedLambdaAdaptationProbe",
    )
    .expect("the selected captured method is recoverable");
    assert!(complete.recovery().produced());
    let complete_usage = complete_budget.usage();

    let mut found = false;
    for limit in 1..complete_usage.ir_items {
        let mut limited = task_budget(&[
            BudgetOverride::new("ir_items", limit).expect("a positive IR bound is valid")
        ])
        .expect("the task budget is valid");
        let Ok(result) = recover_method_result_named(
            &snapshot,
            selected,
            &RecoveryEvidenceRequest::essential(),
            &mut limited,
            "CapturedLambdaAdaptationProbe",
        ) else {
            continue;
        };
        let recovery = result.recovery();
        let usage = limited.usage();
        if matches!(
            recovery.outcome,
            RecoveryOutcome::Stopped(StopReason::Budget {
                dimension: CountedBudgetDimension::IrItems,
                at: Some(5),
                ..
            })
        ) && usage.analysis_steps == complete_usage.analysis_steps
            && usage.ir_items + 1 == limit
            && usage.output_bytes < complete_usage.output_bytes
        {
            assert!(recovery.text.is_empty(), "{recovery:?}");
            assert!(recovery.source_map.is_empty(), "{recovery:?}");
            found = true;
            break;
        }
    }
    assert!(
        found,
        "no shared IR bound stopped the captured adapter's Local/Cast batch at BCI 5: {complete_usage:?}"
    );
}

#[test]
fn bound_null_refusal_does_not_swallow_an_internal_plan_budget_stop() {
    let snapshot = open(BOUND_NULL_FIXTURE);
    let inventory = class_source_of_named(
        &snapshot,
        RecoveryEvidenceRequest::essential(),
        "BoundNullLambdaAdaptationProbe",
    );
    let selected = method(&inventory, "boundNull");
    let mut complete_budget = budget();
    let complete = recover_method_result_named(
        &snapshot,
        selected,
        &RecoveryEvidenceRequest::essential(),
        &mut complete_budget,
        "BoundNullLambdaAdaptationProbe",
    )
    .expect("the bound-null refusal is reported");
    assert!(
        complete
            .recovery()
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "jre_lambda_sam_types")
    );

    let mut found = false;
    for limit in 1..complete_budget.usage().analysis_steps {
        let mut limited = task_budget(&[BudgetOverride::new("analysis_steps", limit)
            .expect("a positive analysis bound is valid")])
        .expect("the task budget is valid");
        let Ok(result) = recover_method_result_named(
            &snapshot,
            selected,
            &RecoveryEvidenceRequest::essential(),
            &mut limited,
            "BoundNullLambdaAdaptationProbe",
        ) else {
            continue;
        };
        let recovery = result.recovery();
        if matches!(
            recovery.outcome,
            RecoveryOutcome::Stopped(StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                at: Some(9),
                ..
            })
        ) && !recovery.produced()
        {
            assert!(recovery.text.is_empty(), "{recovery:?}");
            assert!(recovery.source_map.is_empty(), "{recovery:?}");
            assert!(
                recovery
                    .diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.code != "jre_lambda_sam_types"),
                "a budget stop must not be recast as the ordinary bound-null refusal"
            );
            found = true;
            break;
        }
    }
    assert!(
        found,
        "the bound-null plan never exposed its internal budget stop"
    );
}

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the system clock is after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "jarde-p3-lambda-adaptation-{}-{nonce}",
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
    fs::write(dir.join("LambdaAdaptationProbe.java"), probe).expect("write the probe source");
    fs::write(dir.join("LambdaAdaptationSupport.java"), SUPPORT_SOURCE)
        .expect("write the source-only support");
    fs::write(dir.join("LambdaAdaptationBox.java"), BOX_SOURCE)
        .expect("write the source-only constructor target");
    fs::write(dir.join("LambdaAdaptationRunner.java"), RUNNER_SOURCE)
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
            "LambdaAdaptationSupport.java",
            "LambdaAdaptationBox.java",
            "LambdaAdaptationProbe.java",
            "LambdaAdaptationRunner.java",
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
        .arg("LambdaAdaptationRunner")
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

fn javac_capture(dir: &Path, captured_source: &str) {
    fs::create_dir_all(dir).expect("create the captured-lambda javac directory");
    for (name, source) in [
        ("CapturedLambdaAdaptationProbe.java", captured_source),
        ("BoundNullLambdaAdaptationProbe.java", BOUND_NULL_SOURCE),
        (
            "LambdaCaptureAdaptationSupport.java",
            CAPTURE_SUPPORT_SOURCE,
        ),
        ("CapturedLambdaAdaptationRunner.java", CAPTURE_RUNNER_SOURCE),
    ] {
        fs::write(dir.join(name), source).expect("write the captured-lambda source");
    }
    let output = std::process::Command::new("javac")
        .arg("--release")
        .arg("8")
        .arg("-g:none")
        .arg("-d")
        .arg(dir)
        .args([
            "CapturedLambdaAdaptationProbe.java",
            "BoundNullLambdaAdaptationProbe.java",
            "LambdaCaptureAdaptationSupport.java",
            "CapturedLambdaAdaptationRunner.java",
        ])
        .current_dir(dir)
        .output()
        .expect("start javac for the captured lambda");
    assert!(
        output.status.success(),
        "compiling captured-lambda recovery failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_capture_runner(dir: &Path) -> String {
    let output = std::process::Command::new("java")
        .arg("-Xverify:all")
        .arg("-cp")
        .arg(dir)
        .arg("CapturedLambdaAdaptationRunner")
        .current_dir(dir)
        .output()
        .expect("execute the captured-lambda runner");
    assert!(
        output.status.success(),
        "the captured-lambda runner failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("the captured-lambda runner writes UTF-8")
}

#[test]
#[ignore = "requires JDK: compile and execute original and recovered captured-lambda classes"]
fn recovered_captured_lambda_matches_original_runtime_with_bound_null_control() {
    let report = class_source_of_named(
        &open(CAPTURE_FIXTURE),
        RecoveryEvidenceRequest::all(),
        "CapturedLambdaAdaptationProbe",
    );
    let scratch = Scratch::new();
    let original = scratch.path().join("captured-original");
    let recovered = scratch.path().join("captured-recovered");

    javac_capture(&original, CAPTURE_SOURCE);
    fs::write(
        original.join("CapturedLambdaAdaptationProbe.class"),
        CAPTURE_FIXTURE,
    )
    .expect("install the committed positive fixture");
    fs::write(
        original.join("BoundNullLambdaAdaptationProbe.class"),
        BOUND_NULL_FIXTURE,
    )
    .expect("install the committed bound-null control");
    let original_output = run_capture_runner(&original);

    // The bound-null class is the same original control on this side: Jarde explicitly refuses
    // that rewrite. The first three lines exercise the complete recovered positive class.
    javac_capture(&recovered, &report.text);
    let recovered_output = run_capture_runner(&recovered);
    assert_eq!(
        original_output,
        "35\n31\njava.lang.ClassCastException\nboundNull:java.lang.NullPointerException\n"
    );
    assert_eq!(recovered_output, original_output);
}

#[test]
#[ignore = "requires JDK: compile and execute the original and recovered complete classes"]
fn recovered_complete_class_matches_original_order_exceptions_and_calls() {
    let report = class_source_of(&open(FIXTURE), RecoveryEvidenceRequest::all());
    all_recovered(&report);

    let scratch = Scratch::new();
    let original = scratch.path().join("original");
    let recovered_dir = scratch.path().join("recovered");

    // Compile all source-only helpers and runner first, then restore the exact frozen target class
    // bytes.  The recovered side receives the complete Engine output and all generated methods.
    javac(&original, PROBE_SOURCE);
    fs::write(original.join("LambdaAdaptationProbe.class"), FIXTURE)
        .expect("install the committed original class bytes");
    let original_output = run_runner(&original);

    javac(&recovered_dir, &report.text);
    let recovered_output = run_runner(&recovered_dir);

    assert_eq!(
        original_output,
        "string:11\nnull:11\nwrong:java.lang.ClassCastException\nwiderString:12:calls=1\nwiderNull:12:calls=1\nwiderWrong:java.lang.ClassCastException:calls=0\narray:14\narrayNull:14\narrayWrong:java.lang.ClassCastException\nunbound:21\nunboundNull:21\nunboundWrong:java.lang.ClassCastException\nconstructor:text\nprimitive:8\ngenericRawString:java.lang.String:text\ngenericTypedString:text\ngenericRawInteger:java.lang.Integer:7\ngenericTypedInteger:java.lang.ClassCastException\nreceiverNull:java.lang.NullPointerException:calls=0\nreceiverNullWrongArg:java.lang.ClassCastException:calls=0\nreceiverWrong:java.lang.ClassCastException:calls=0\nreceiverWrongNullArg:java.lang.ClassCastException:calls=0\nreceiverValidNullArg:21:calls=1\nreceiverValidWrongArg:java.lang.ClassCastException:calls=0\nconstructorText:text:calls=1\nconstructorNull:null:calls=1\nconstructorWrong:java.lang.ClassCastException:calls=0\n"
    );
    assert_eq!(
        recovered_output, original_output,
        "recovered behavior and evaluation order"
    );
}
