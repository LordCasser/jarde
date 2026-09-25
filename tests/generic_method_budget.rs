use jarde::*;

const METHOD: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/GenericMethodProbe.class"
);
const THROWS: &[u8] = include_bytes!(
    "../openspec/evidence/java-syntax-2026-09-24/generic-method-signatures/GenericThrowsProbe.class"
);

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

fn report(bytes: &[u8], name: &str, evidence: &RecoveryEvidenceRequest) -> ClassSourceReport {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    match engine
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request(&snapshot, name),
            evidence,
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        outcome => panic!("unexpected class-source outcome: {outcome:?}"),
    }
}

fn stopped_report(bytes: &[u8], name: &str, budget: &mut Budget) -> ClassSourceReport {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    match engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request(&snapshot, name),
            budget,
        )
        .unwrap()
    {
        OperationOutcome::Performed(report) => report,
        outcome => panic!("unexpected class-source outcome: {outcome:?}"),
    }
}

fn outcome_with_limits(limits: Limits) -> OperationOutcome<ClassSourceReport> {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(METHOD.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request(&snapshot, "GenericMethodProbe"),
            &mut Budget::new(limits),
        )
        .unwrap()
}

#[test]
fn essential_and_all_evidence_keep_complete_generic_class_text_identical() {
    for (bytes, name) in [
        (METHOD, "GenericMethodProbe"),
        (THROWS, "GenericThrowsProbe"),
    ] {
        let essential = report(bytes, name, &RecoveryEvidenceRequest::essential());
        let all = report(bytes, name, &RecoveryEvidenceRequest::all());
        assert_eq!(essential.text, all.text, "{name}");
        assert!(
            essential.text.contains("<T extends java.lang.Number>"),
            "{name}: {}",
            essential.text
        );
    }
}

#[test]
fn body_budget_stop_keeps_the_physical_generic_member_without_a_partial_header() {
    let mut budget = task_budget(&[BudgetOverride::new("method_bodies", 1).unwrap()]).unwrap();
    let report = stopped_report(METHOD, "GenericMethodProbe", &mut budget);

    assert!(!report.text.contains("<T extends"), "{}", report.text);
    assert_eq!(
        report.methods.len(),
        2,
        "the physical method records remain represented"
    );
    let choose = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == b"choose")
        .expect("physical choose member is retained");
    assert_eq!(
        choose.item.descriptor.raw().0,
        b"(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;"
    );
    assert!(
        choose
            .markers
            .iter()
            .any(|marker| marker.contains("budget_exceeded_method_bodies")),
        "{choose:?}"
    );
    assert!(
        report.text.contains(
            "java.lang.Number choose(java.lang.Number arg0, java.lang.Number arg1, boolean arg2)"
        ),
        "{}",
        report.text
    );
    assert!(matches!(
        report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::MethodBodies
            },
            ..
        }
    ));
}

#[test]
fn pre_cancellation_reports_cancelled_without_a_generic_declaration() {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(METHOD.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut budget = Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation);
    let outcome = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request(&snapshot, "GenericMethodProbe"),
            &mut budget,
        )
        .unwrap();
    match outcome {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            assert!(!report.text.contains("<T extends"), "{}", report.text);
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
        }
        OperationOutcome::Ambiguous(_) => panic!("one frozen class must bind uniquely"),
    }
}

#[test]
fn internal_budget_stops_do_not_publish_a_partial_generic_header() {
    let baseline = report(
        METHOD,
        "GenericMethodProbe",
        &RecoveryEvidenceRequest::essential(),
    );
    let dimensions = [
        (
            "attribute_bytes",
            baseline.usage.attribute_bytes,
            BudgetDimension::AttributeBytes,
        ),
        (
            "ir_items",
            baseline.usage.ir_items,
            BudgetDimension::IrItems,
        ),
        (
            "analysis_steps",
            baseline.usage.analysis_steps,
            BudgetDimension::AnalysisSteps,
        ),
        (
            "output_bytes",
            baseline.usage.output_bytes,
            BudgetDimension::OutputBytes,
        ),
    ];
    for (dimension, successful_usage, expected_stop) in dimensions {
        assert!(
            successful_usage > 1,
            "{dimension} must have measurable successful use"
        );
        let mut limits = task_limits(&[]).unwrap();
        let limit = successful_usage - 1;
        match dimension {
            "attribute_bytes" => limits.attribute_bytes = limit,
            "ir_items" => limits.ir_items = limit,
            "analysis_steps" => limits.analysis_steps = limit,
            "output_bytes" => limits.output_bytes = limit,
            _ => unreachable!(),
        }
        let outcome = outcome_with_limits(limits);
        let (methods, text, execution) = match outcome {
            OperationOutcome::Performed(report) => (report.methods, report.text, report.execution),
            OperationOutcome::Incomplete(selection) => panic!(
                "{dimension}: the baseline-derived limit should stop after class selection: {selection:?}"
            ),
            OperationOutcome::Ambiguous(_) => panic!("frozen fixture binds one class"),
        };
        if text.contains("<T") {
            assert!(
                text.contains(
                    "public static <T extends java.lang.Number> T choose(T arg0, T arg1, boolean arg2)"
                ),
                "{dimension}: a generic declaration was only partly published: {text}"
            );
            assert!(
                text.contains("return arg2 ? arg0 : arg1;"),
                "{dimension}: generic declaration lacks its complete body: {text}"
            );
        } else {
            assert!(
                text.contains(
                    "java.lang.Number choose(java.lang.Number arg0, java.lang.Number arg1, boolean arg2)"
                ),
                "{dimension}: physical descriptor declaration missing: {text}"
            );
        }
        let choose = methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"choose")
            .unwrap_or_else(|| panic!("{dimension}: physical choose record absent"));
        assert_eq!(
            choose.item.descriptor.raw().0,
            b"(Ljava/lang/Number;Ljava/lang/Number;Z)Ljava/lang/Number;",
            "{dimension}"
        );
        assert!(
            matches!(
                execution,
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded { dimension },
                    ..
                } if dimension == expected_stop
            ),
            "{dimension}: {execution:?}"
        );
    }
}
