//! Public class-source coverage for the proven if-arm transfer into a nested loop.
use jarde::*;
use jarde_java::Provenance;
use jarde_jvm::ir::Quality;
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::ExecutionReport;
use std::collections::BTreeSet;
use std::slice;

const ORIGINAL_CLASS: &[u8] =
    include_bytes!("fixtures/cf08-if-loop-entry/original/loops/TestLoopRestore2.class");
const ORIGINAL_JAR: &[u8] = include_bytes!("fixtures/cf08-if-loop-entry/original.jar");
const EXPECTED_SOURCE: &str =
    include_str!("fixtures/cf08-if-loop-entry/baseline/TestLoopRestore2.java");
const OWNER: &str = "loops/TestLoopRestore2";
const METHOD: &str = "test";
const DESCRIPTOR: &str = "([I[I)V";

fn source(bytes: &[u8], policy: EnvironmentPolicy, all: bool) -> ClassSourceReport {
    let mut budget = task_budget(&[]).expect("default task budget is valid");
    let engine = Engine::new();
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("frozen class or JAR opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(OWNER),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    let result = if all {
        engine.class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget,
        )
    } else {
        engine.class_source(slice::from_ref(&snapshot), &request, &mut budget)
    }
    .expect("class-source request returns an outcome");
    match result {
        OperationOutcome::Performed(report) => report,
        other => panic!("the frozen class source completes: {other:?}"),
    }
}

fn target(report: &ClassSourceReport) -> &ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| {
            method.item.name.raw().0 == METHOD.as_bytes()
                && method.item.descriptor.raw().0 == DESCRIPTOR.as_bytes()
        })
        .expect("the physical test method remains in the class report")
}

fn method_request(engine: &Engine, snapshot: &ArtifactSnapshot) -> MethodAnalysisRequest {
    let mut header_budget = task_budget(&[]).expect("header budget is valid");
    let inspected = engine
        .inspect_header(
            snapshot,
            ClassTarget::Root,
            &mut header_budget,
            InspectionMode::Strict,
        )
        .expect("fixture header is inspectable");
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    MethodAnalysisRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        },
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: inspected.source.class_bytes,
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(METHOD.as_bytes().to_vec()),
            descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    }
}

fn physical_bcis() -> BTreeSet<u32> {
    let mut budget = task_budget(&[]).expect("reader task budget is valid");
    let code = inspect_method_bytecode(
        ORIGINAL_CLASS,
        MethodSelector {
            name: JvmBytes(METHOD.as_bytes().to_vec()),
            descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
        },
        &mut budget,
    )
    .expect("the frozen method Code parses completely");
    assert!(code.stopped_at.is_none());
    code.instructions
        .into_iter()
        .map(|instruction| instruction.bci)
        .collect()
}

fn assert_proved_entry(report: &ClassSourceReport) {
    assert_eq!(
        report.text, EXPECTED_SOURCE,
        "the public result is whole-class source"
    );
    assert_eq!(
        report.methods.len(),
        1,
        "the frozen class has only its test method"
    );
    let method = target(report);
    assert_eq!(method.item.identity.owner, report.class);
    assert_eq!(method.item.identity.name.0, METHOD.as_bytes());
    assert_eq!(method.item.identity.descriptor.0, DESCRIPTOR.as_bytes());
    let ClassSourceOutcome::Recovered { report: body, .. } = &method.outcome else {
        panic!(
            "the target has a same-run recovered body: {:?}",
            method.outcome
        );
    };
    assert_eq!(body.quality, Quality::Structured, "{}", body.text);

    let physical = physical_bcis();
    assert_eq!(
        physical,
        BTreeSet::from([0, 1, 2, 5, 8, 11, 14, 17, 20, 23])
    );
    let mut mapped = BTreeSet::new();
    for segment in body.source_map.segments() {
        assert!(
            body.text.get(segment.start()..segment.end()).is_some(),
            "UTF-8 span"
        );
        for anchor in std::iter::once(segment.origin().primary()).chain(segment.origin().derived())
        {
            assert_eq!(anchor.method(), Some(&method.item.identity));
            assert!(
                physical.contains(&anchor.bci()),
                "anchor is in this method's Code"
            );
            mapped.insert(anchor.bci());
        }
    }
    assert_eq!(
        mapped, physical,
        "every physical instruction has exact method provenance"
    );

    let if_span = body
        .source_map
        .segments()
        .iter()
        .find(|segment| {
            segment.origin().primary().bci() == 5 && segment.text(&body.text).contains("if (false)")
        })
        .expect("the existing if presentation remains the primary span");
    assert!(if_span.origin().derived().iter().any(|origin| {
        origin.bci() == 8
            && origin.method() == Some(&method.item.identity)
            && origin.provenance() == Provenance::Derived
    }));
    assert!(
        if_span
            .origin()
            .derived()
            .iter()
            .any(|origin| { origin.bci() == 17 && origin.method() == Some(&method.item.identity) }),
        "the existing exit origin stays on the if span"
    );
}

#[test]
fn if_loop_entry_origin_is_public_for_class_and_jar_inputs() {
    for (input, policy) in [
        (ORIGINAL_CLASS, EnvironmentPolicy::SingleClass),
        (ORIGINAL_JAR, EnvironmentPolicy::PlainJar),
    ] {
        let default = source(input, policy.clone(), false);
        let all = source(input, policy, true);
        assert_proved_entry(&default);
        assert_proved_entry(&all);
        assert_eq!(default.text, all.text);
        let default_method = target(&default);
        let all_method = target(&all);
        assert_eq!(default_method.text, all_method.text);
        let (
            ClassSourceOutcome::Recovered {
                report: default_body,
                ..
            },
            ClassSourceOutcome::Recovered {
                report: all_body, ..
            },
        ) = (&default_method.outcome, &all_method.outcome)
        else {
            panic!("both evidence modes recover the target method");
        };
        assert_eq!(default_body.source_map, all_body.source_map);
    }
}

#[test]
fn if_loop_entry_public_source_propagates_precancellation() {
    let mut open_budget = task_budget(&[]).expect("default task budget is valid");
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(ORIGINAL_CLASS.to_vec()),
            &mut open_budget,
        )
        .expect("fixture opens");
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(OWNER),
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
    let token = CancellationToken::new();
    token.cancel();
    let mut cancelled = Budget::with_cancellation_token(open_budget.limits().clone(), token);
    match engine
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut cancelled,
        )
        .expect("cancellation is an execution state")
    {
        OperationOutcome::Incomplete(selection) => {
            assert!(matches!(
                selection.execution,
                ExecutionReport::Cancelled { .. }
            ));
        }
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            assert_ne!(report.text, EXPECTED_SOURCE);
        }
        OperationOutcome::Ambiguous(_) => panic!("the frozen class is unique"),
    }

    // Bound the scan by this method's measured successful edge usage. Only accept a stop that
    // reaches the real derived-origin proof site through the public method recovery pipeline.
    let request = method_request(&engine, &snapshot);
    let mut complete_budget = task_budget(&[]).expect("normal method budget is valid");
    let complete = engine
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::essential(),
            &mut complete_budget,
        )
        .expect("the unbounded method request returns a report");
    assert!(complete.recovery().outcome.produced());
    let edge_upper_bound = complete_budget.usage().ir_edges;
    assert!(
        edge_upper_bound > 0,
        "the complete proof read canonical edges"
    );

    let mut proof_site_stop = None;
    for edge_limit in 0..=edge_upper_bound {
        let mut limits = task_budget(&[])
            .expect("normal task limits are valid")
            .limits()
            .clone();
        limits.ir_edges = edge_limit;
        let mut budget = Budget::new(limits);
        let stopped = engine
            .recover_method_with_evidence(
                slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::essential(),
                &mut budget,
            )
            .expect("an edge-budget boundary is reported by method recovery");
        let recovery = stopped.recovery();
        let at_proof_site = matches!(
            &recovery.outcome,
            RecoveryOutcome::Stopped(StopReason::Budget {
                dimension: CountedBudgetDimension::IrEdges,
                at: Some(8),
                ..
            })
        );
        if at_proof_site {
            assert!(matches!(
                &recovery.execution,
                ExecutionReport::Partial {
                    reason: jarde_reader::model::TerminationReason::BudgetExceeded {
                        dimension: jarde_reader::budget::BudgetDimension::IrEdges,
                    },
                    ..
                }
            ));
            assert_eq!(recovery.content, jarde_java::RecoveryContent::NotProduced);
            assert!(recovery.text.is_empty(), "no method body was published");
            assert!(
                recovery.source_map.is_empty(),
                "no partial map was published"
            );
            assert!(
                recovery.regions.is_empty(),
                "no partial region was committed"
            );
            assert!(recovery.declaration.is_none());
            assert!(recovery.artifact.binding().is_none());
            let ExecutionReport::Partial { usage, .. } = &recovery.execution else {
                unreachable!("the budget stop is partial");
            };
            assert_eq!(
                usage.ir_edges,
                budget.usage().ir_edges,
                "reported usage retains the work consumed before the stop"
            );
            assert!(usage.ir_edges <= edge_limit);
            proof_site_stop = Some((edge_limit, usage.ir_edges));
            break;
        }
    }
    let Some((edge_limit, charged_edges)) = proof_site_stop else {
        panic!(
            "bounded scan through successful usage {edge_upper_bound} must reach the proof-site stop"
        );
    };
    assert!(
        edge_limit < edge_upper_bound,
        "the successful run exceeds the stopping cap"
    );
    assert!(
        charged_edges <= edge_limit,
        "the measured use fits the accepted cap"
    );
}
