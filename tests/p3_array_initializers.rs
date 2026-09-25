//! `recover-array-initializers`: one proven allocation/store chain becomes one ordered Java
//! initializer, while allocation shapes outside the proof remain on the ordinary array path.

use jarde::*;
use std::slice;

const FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-array-initializers/v8/ArrayInitializerProbe.class");
const NO_ARRAY_FIXTURE: &[u8] = include_bytes!("fixtures/p3-array-types/v8/ArrayTypes.class");
const MULTI_ARRAY_FIXTURE: &[u8] =
    include_bytes!("fixtures/p3-deferred-value-order/v8/DeferredValueOrder.class");

fn budget() -> Budget {
    task_budget(&[]).expect("the task defaults are a bounded budget")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("the committed Java 8 fixture opens as a standalone CLASS")
}

fn request(snapshot: &ArtifactSnapshot) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("ArrayInitializerProbe"),
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

fn method_request(
    snapshot: &ArtifactSnapshot,
    name: &[u8],
    descriptor: &[u8],
) -> MethodAnalysisRequest {
    let inspected = Engine::new()
        .inspect_header(
            snapshot,
            ClassTarget::Root,
            &mut budget(),
            InspectionMode::Strict,
        )
        .expect("the no-array fixture header is readable");
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
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: inspected.source.class_bytes,
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
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
        method,
        stages: AnalysisStage::ALL.to_vec(),
    }
}

fn class_source(
    snapshot: &ArtifactSnapshot,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> ClassSourceReport {
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request(snapshot),
            evidence,
            budget,
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("the fixture answers one class-source request: {other:?}"),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no member `{name}` in ArrayInitializerProbe"))
}

fn recovery<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` has no recovery body: {other:?}"),
    }
}

#[test]
fn complete_initializer_chains_have_stable_bodies_and_claim_every_source() {
    let snapshot = open(FIXTURE);
    let essential = class_source(
        &snapshot,
        &RecoveryEvidenceRequest::essential(),
        &mut budget(),
    );
    let all = class_source(&snapshot, &RecoveryEvidenceRequest::all(), &mut budget());
    assert_eq!(
        essential.text, all.text,
        "evidence selection cannot change text"
    );

    for (name, expected, sources) in [
        (
            "literalInts",
            "return new int[]{1, 2, 3, -4};",
            &[
                0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 19, 20,
            ][..],
        ),
        (
            "literalStrings",
            "return new java.lang.String[]{\"left\", null, \"right\"};",
            &[0, 1, 4, 5, 6, 8, 9, 10, 11, 12, 13, 14, 15, 17, 18][..],
        ),
        (
            "effectfulInts",
            "return new int[]{intElement(arg0, 0), intElement(arg0, 1), intElement(arg0, 2)};",
            &[
                0, 1, 3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 15, 18, 19, 20, 21, 22, 23, 26, 27,
            ][..],
        ),
        (
            "effectfulStrings",
            "return new java.lang.String[]{stringElement(arg0, 0), stringElement(arg0, 1), stringElement(arg0, 2)};",
            &[
                0, 1, 4, 5, 6, 7, 8, 11, 12, 13, 14, 15, 16, 19, 20, 21, 22, 23, 24, 27, 28,
            ][..],
        ),
    ] {
        let default_run = recovery(&essential, name);
        let all_run = recovery(&all, name);
        assert_eq!(default_run.text, all_run.text, "`{name}` body is stable");
        assert!(
            all_run.produced(),
            "`{name}` produced no artifact: {all_run:?}"
        );
        assert_eq!(all_run.representation, Representation::Java, "{name}");
        assert_eq!(all_run.quality, Quality::Structured, "{name}");
        assert!(
            all_run.text.contains(expected),
            "`{name}`:\n{}",
            all_run.text
        );
        assert!(
            !all_run.text.contains("@bytecode"),
            "`{name}` still quotes a source instruction:\n{}",
            all_run.text
        );
        assert!(
            default_run.source_map.is_empty(),
            "essential evidence leaves `{name}`'s source map unmaterialized"
        );
        for &bci in sources {
            assert!(
                !all_run.source_map.of_bci(bci).is_empty(),
                "`{name}` omitted initializer BCI {bci} from its source map: {:?}",
                all_run.source_map.segments()
            );
        }
    }

    for (name, expected) in [
        ("emptyInts", "return new int[0];"),
        ("emptyStrings", "return new java.lang.String[0];"),
    ] {
        let body = recovery(&all, name);
        assert!(body.text.contains(expected), "`{name}`:\n{}", body.text);
        assert!(
            !body.text.contains("[]{"),
            "empty ordinary allocation stays sized"
        );
    }
}

#[test]
fn an_exhausted_output_budget_does_not_publish_a_partial_class() {
    let snapshot = open(FIXTURE);
    let stopped = Engine::new()
        .class_source_with_evidence(
            slice::from_ref(&snapshot),
            &request(&snapshot),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(Limits {
                output_bytes: 0,
                ..task_limits(&[]).expect("the task defaults are bounded")
            }),
        )
        .expect("an exhausted output budget is an operation outcome");
    assert!(
        matches!(stopped, OperationOutcome::Incomplete(_)),
        "the whole class is withheld when its output cannot be committed: {stopped:?}"
    );
}

#[test]
fn a_method_without_array_allocations_keeps_its_tight_ir_budget() {
    let snapshot = open(NO_ARRAY_FIXTURE);
    let request = method_request(
        &snapshot,
        b"text",
        b"(Ljava/lang/String;)Ljava/lang/String;",
    );
    let mut tight = Budget::new(Limits {
        // This no-array body completes at exactly 84 IR items. The allowance intentionally leaves
        // no room for charging an unrelated effect map while probing for initializer candidates.
        ir_items: 84,
        ..task_limits(&[]).expect("the task defaults are bounded")
    });
    let recovered = Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut tight,
        )
        .expect("a no-array method recovers under the bounded IR allowance");
    assert!(
        recovered.recovery().produced(),
        "a body with no array allocation stays on its existing recovery path: {:?}",
        recovered.recovery().stop()
    );
    assert_eq!(
        tight.usage().ir_items,
        84,
        "the fixture reaches the bound exactly"
    );
}

#[test]
fn a_partial_dimension_allocation_does_not_enter_the_initializer_candidate_path() {
    let snapshot = open(MULTI_ARRAY_FIXTURE);
    let request = method_request(&snapshot, b"multiArray", b"(I)[[I");
    let mut tight = Budget::new(Limits {
        // `multianewarray` allocates only part of the descriptor's dimensions and is not a
        // one-dimensional initializer candidate; leave no room for candidate-effect charges.
        ir_items: 104,
        ..task_limits(&[]).expect("the task defaults are bounded")
    });
    let recovered = Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut tight,
        )
        .expect("a multianewarray method recovers under the bounded IR allowance");
    assert!(
        recovered.recovery().produced(),
        "a partial-dimension allocation keeps its existing recovery path: {:?}",
        recovered.recovery().stop()
    );
    assert_eq!(
        tight.usage().ir_items,
        104,
        "the fixture reaches the bound exactly"
    );
}
