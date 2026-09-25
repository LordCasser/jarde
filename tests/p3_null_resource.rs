//! `recover-null-resource-headers`: a direct null literal is a resource only after the existing
//! close proof recognizes both close contours; its declaration is typed only from same-class facts.

use jarde::*;
use std::slice;

const SAMPLE: &[u8] = include_bytes!("fixtures/p3-null-resource/v8/NullResourceCore.class");
const ORDINARY_AND_NEAR: &[u8] =
    include_bytes!("fixtures/p3-null-resource/same-type/v8/NullResourceCore.class");
const BROKEN_SUPPRESSION: &[u8] =
    include_bytes!("fixtures/p3-null-resource/broken-suppression/v8/NullResourceCore.class");
const INHERITED_ONLY: &[u8] =
    include_bytes!("fixtures/p3-null-resource/inherited/v8/NullResourceInheritedOnly.class");
const EXCEPTIONAL: &[u8] =
    include_bytes!("fixtures/p3-null-resource/exceptional/v8/NullResourceExceptionalCore.class");

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 10,
        method_bodies: 10,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

struct Fixture {
    snapshot: ArtifactSnapshot,
    class_bytes: ClassBytesId,
}

fn fixture(bytes: &[u8]) -> Fixture {
    let engine = Engine::new();
    let mut budget = Budget::new(limits());
    let snapshot = engine
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget)
        .expect("the fixture opens as one standalone class");
    let inspected = engine
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut budget,
            InspectionMode::Strict,
        )
        .expect("the fixture's own header is readable");
    Fixture {
        snapshot,
        class_bytes: inspected.source.class_bytes.clone(),
    }
}

fn environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
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
    ResolutionEnvironment {
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
    }
}

fn request(fixture: &Fixture, name: &str) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: environment(&fixture.snapshot),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: fixture.snapshot.id().clone(),
                },
                class_bytes: fixture.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    }
}

fn recover(fixture: &Fixture, name: &str) -> RecoveredMethod {
    Engine::new()
        .recover_method_with_evidence(
            slice::from_ref(&fixture.snapshot),
            &request(fixture, name),
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits()),
        )
        .expect("a legal recovery request is answered")
}

fn class_source(
    fixture: &Fixture,
    class: &str,
    evidence: RecoveryEvidenceRequest,
) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(class),
        },
        environment: EnvironmentRequest {
            snapshot: fixture.snapshot.id().clone(),
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
            slice::from_ref(&fixture.snapshot),
            &request,
            &evidence,
            &mut Budget::new(limits()),
        )
        .expect("a legal class-source request is answered")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone fixture has {} matching definitions",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "an ample class-source request stopped with {} candidate(s)",
            candidates.candidates.len()
        ),
    }
}

fn member<'a>(report: &'a ClassSourceReport, name: &str) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("no method `{name}` in the fixture"))
}

fn recovered<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    match &member(report, name).outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!("`{name}` did not produce a recovery report: {other:?}"),
    }
}

#[test]
fn proved_null_header_has_a_current_class_type_and_all_cleanup_anchors() {
    let sample = fixture(SAMPLE);
    let report = class_source(&sample, "NullResourceCore", RecoveryEvidenceRequest::all());
    let method = member(&report, "useNullResource");
    let body = recovered(&report, "useNullResource");

    assert_eq!(body.quality, Quality::Structured, "{}", body.text);
    assert!(
        method.text.contains("try (NullResourceCore local0 = null)"),
        "the proved declaration uses the current class type:\n{}",
        method.text
    );
    assert!(
        !method.text.contains("catch (java.lang.Throwable")
            && !method.text.contains("addSuppressed"),
        "compiler cleanup is owned by the resource header:\n{}",
        method.text
    );

    // `javap -c` for this frozen class: null initializer 0..1, normal close 15, handler primary
    // 21, exceptional close 27, suppression load/call 34/36, and primary rethrow 40.
    for bci in [0, 1, 15, 21, 27, 34, 36, 40] {
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no source-map segment: {:?}",
            body.source_map.segments()
        );
    }
    assert!(!body.source_map.direct_of_bci(0).is_empty());
    for bci in [1, 15, 21, 27, 34, 36, 40] {
        assert!(
            !body.source_map.derived_of_bci(bci).is_empty(),
            "protocol BCI {bci} is not a derived origin"
        );
    }
}

#[test]
fn an_exception_from_the_null_resource_body_keeps_primary_identity_and_skips_close() {
    let sample = fixture(EXCEPTIONAL);
    let report = class_source(
        &sample,
        "NullResourceExceptionalCore",
        RecoveryEvidenceRequest::all(),
    );
    let method = member(&report, "useNullResourceException");
    let body = recovered(&report, "useNullResourceException");
    assert_eq!(body.quality, Quality::Structured, "{}", body.text);
    assert!(
        method
            .text
            .contains("try (NullResourceExceptionalCore local0 = null)"),
        "the throwing body remains inside the null resource:\n{}",
        method.text
    );
    assert!(
        !method.text.contains("catch (java.lang.Throwable")
            && !method.text.contains("addSuppressed"),
        "the compiler handler is not a user clause:\n{}",
        method.text
    );
}

#[test]
fn ordinary_null_catch_stays_a_catch_and_a_broken_suppression_is_refused_as_twr() {
    let sample = fixture(ORDINARY_AND_NEAR);
    let report = class_source(&sample, "NullResourceCore", RecoveryEvidenceRequest::all());
    let ordinary = member(&report, "ordinaryNullThenCatch");
    let ordinary_body = recovered(&report, "ordinaryNullThenCatch");
    assert_eq!(
        ordinary_body.quality,
        Quality::Structured,
        "{}",
        ordinary.text
    );
    assert!(ordinary.text.contains("catch (java.lang.RuntimeException"));
    assert!(!ordinary.text.contains("try ("));

    let broken = fixture(BROKEN_SUPPRESSION);
    let refused = recover(&broken, "useNullResource");
    let body = refused.recovery();
    assert_eq!(body.quality, Quality::Fallback, "{}", body.text);
    assert!(body.fallbacks.contains(&"jre_guard_suppressed"), "{body:?}");
    assert!(
        !body.text.contains("catch (java.lang.Throwable") && !body.text.contains("try ("),
        "a damaged cleanup is refused, not presented as a user catch:\n{}",
        body.text
    );
    assert!(
        body.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "jre_guard_suppressed" && diagnostic.message.contains("BCI")
        }),
        "the refusal names the broken suppression's BCI: {:?}",
        body.diagnostics
    );
}

#[test]
fn inherited_autocloseable_does_not_supply_a_current_class_resource_type() {
    let sample = fixture(INHERITED_ONLY);
    let report = class_source(
        &sample,
        "NullResourceInheritedOnly",
        RecoveryEvidenceRequest::all(),
    );
    let method = member(&report, "useNullResource");
    let body = recovered(&report, "useNullResource");
    assert_eq!(body.quality, Quality::Fallback, "{}", method.text);
    assert!(
        method
            .text
            .contains("does not directly declare `java/lang/AutoCloseable`"),
        "the missing direct interface is the stated refusal:\n{}",
        method.text
    );
    assert!(!method.text.contains("try (NullResourceInheritedOnly "));
    assert!(!method.text.contains("catch (java.lang.Throwable"));
}

#[test]
fn evidence_selection_and_stops_keep_the_existing_body_contract() {
    let sample = fixture(SAMPLE);
    let essential = class_source(
        &sample,
        "NullResourceCore",
        RecoveryEvidenceRequest::essential(),
    );
    let all = class_source(&sample, "NullResourceCore", RecoveryEvidenceRequest::all());
    assert_eq!(
        essential.text, all.text,
        "optional evidence changed the body"
    );
    let essential_body = recovered(&essential, "useNullResource");
    let all_body = recovered(&all, "useNullResource");
    assert!(essential_body.source_map.is_empty());
    assert_eq!(
        essential_body.evidence.requested(),
        &RecoveryEvidenceRequest::essential()
    );
    assert!(!all_body.source_map.is_empty());
    assert_eq!(
        all_body.evidence.requested(),
        &RecoveryEvidenceRequest::all()
    );

    let body_request = request(&sample, "useNullResource");
    let ample = Engine::new()
        .recover_method(
            slice::from_ref(&sample.snapshot),
            &body_request,
            &mut Budget::new(limits()),
        )
        .expect("the ample default request completes");
    let ExecutionReport::Complete { usage } = &ample.recovery().execution else {
        panic!(
            "the ample default request completes: {:?}",
            ample.recovery().execution
        );
    };
    let charged = usage.output_bytes;
    let text_bytes = u64::try_from(ample.recovery().text.len()).expect("artifact length fits u64");
    assert!(charged > text_bytes, "the run charges analysis before text");
    let output_limit = charged - text_bytes;
    let stopped = Engine::new()
        .recover_method(
            slice::from_ref(&sample.snapshot),
            &body_request,
            &mut Budget::new(Limits {
                output_bytes: output_limit,
                ..limits()
            }),
        )
        .expect("an exhausted budget is reported as a stopped run");
    assert!(
        matches!(
            stopped.recovery().outcome,
            RecoveryOutcome::Stopped(StopReason::Budget {
                dimension: CountedBudgetDimension::OutputBytes,
                ..
            })
        ),
        "outcome={:?}, charged={charged}, text_bytes={text_bytes}, output_limit={output_limit}",
        stopped.recovery().outcome
    );
    assert_eq!(stopped.recovery().content, RecoveryContent::NotProduced);
    assert!(stopped.recovery().text.is_empty());
    assert!(stopped.recovery().source_map.is_empty());

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = Engine::new()
        .recover_method(
            slice::from_ref(&sample.snapshot),
            &body_request,
            &mut Budget::with_cancellation_token(limits(), token),
        )
        .expect("cancellation is reported as a stopped run");
    assert!(matches!(
        cancelled.analysis().execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_eq!(cancelled.recovery().content, RecoveryContent::NotProduced);
    assert!(cancelled.recovery().text.is_empty());
    assert!(cancelled.recovery().source_map.is_empty());
}
