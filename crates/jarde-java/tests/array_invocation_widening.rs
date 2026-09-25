//! Java 8 array upcasts at invocation sites, from committed verifier-valid class files.

use jarde_java::{
    MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryOutcome, RecoveryRequest, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::classfile::class_facts;
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const CORE: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-array-invocation-widening/v8/ArrayReferenceCore.class"
);
const OVERLOAD: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-array-invocation-widening/v8/ArrayReferenceOverloadTarget.class"
);
const UNKNOWN_RELATION: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-array-invocation-widening/v8/UnknownArrayRelation.class"
);
const BUILTIN_INTERFACES: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-array-invocation-widening/v8/ArrayInterfaceProbe.class"
);

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

fn recover_method(
    class: &[u8],
    name: &str,
    descriptor: &str,
    parameter_count: u16,
) -> jarde_java::RecoveryReport {
    recover_method_with_options(class, name, descriptor, parameter_count, true, None, false)
}

fn recover_method_with_options(
    class: &[u8],
    name: &str,
    descriptor: &str,
    parameter_count: u16,
    all_evidence: bool,
    output_limit: Option<u64>,
    cancel_before_recovery: bool,
) -> jarde_java::RecoveryReport {
    let mut run_limits = limits();
    if let Some(output_limit) = output_limit {
        run_limits.output_bytes = output_limit;
    }
    let token = CancellationToken::new();
    let mut budget = Budget::with_cancellation_token(run_limits, token.clone());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("fixture opens as one standalone class");
    let header = class_facts(class, &mut budget).expect("fixture is a class file");
    assert!(
        header
            .methods
            .iter()
            .find(|member| {
                member.name.raw().0 == name.as_bytes()
                    && member.descriptor.raw().0 == descriptor.as_bytes()
            })
            .is_some(),
        "fixture declares the selected descriptor"
    );
    let owner = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("class length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: owner.clone(),
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let environment = ResolutionEnvironment {
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
    };
    let analysis = analyze_method_ir(
        &[snapshot],
        &MethodAnalysisRequest {
            environment,
            method,
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("fixture method analysis completes");
    let facts = RecoveryFacts::new(MethodFacts::new(name, descriptor, parameter_count));
    let mut request = RecoveryRequest::new(analysis.ir(), &facts, jarde_java::pass::JAVA_8);
    if all_evidence {
        request = request.with_evidence(RecoveryEvidenceRequest::all());
    }
    if cancel_before_recovery {
        token.cancel();
    }
    recover(&request, &mut budget)
}

fn assert_recovered(report: &jarde_java::RecoveryReport, needle: &str) {
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert!(
        report.text.contains(needle),
        "expected {needle:?} in recovered body:\n{}",
        report.text
    );
}

fn assert_argument_origin(
    report: &jarde_java::RecoveryReport,
    expression: &str,
    producer: u32,
    call: u32,
) {
    let segment = report
        .source_map
        .segments()
        .iter()
        .find(|segment| {
            let text = segment.text(&report.text);
            text.contains(expression) && text.contains("java.lang.Object")
        })
        .unwrap_or_else(|| {
            panic!(
                "missing target-typed expression {expression:?}: {}",
                report.text
            )
        });
    let bcis = segment.origin().bcis();
    assert!(
        bcis.contains(&producer),
        "producer BCI {producer}: {bcis:?}"
    );
    assert!(bcis.contains(&call), "invocation BCI {call}: {bcis:?}");
}

#[test]
fn object_component_upcasts_keep_the_invocation_parameter_type() {
    for (name, descriptor, argument_type) in [
        (
            "typedIntMatrix",
            "([[I)Ljava/lang/String;",
            "java.lang.Object[]",
        ),
        (
            "typedStringArray",
            "([Ljava/lang/String;)Ljava/lang/String;",
            "java.lang.Object[]",
        ),
        (
            "typedStringMatrix",
            "([[Ljava/lang/String;)Ljava/lang/String;",
            "java.lang.Object[][]",
        ),
    ] {
        let report = recover_method(CORE, name, descriptor, 1);
        assert_recovered(&report, "overload(");
        assert_recovered(&report, argument_type);
        assert_argument_origin(&report, "arg0", 0, 1);
    }
}

#[test]
fn built_in_array_marker_interfaces_are_legal_invocation_targets() {
    for (name, descriptor, target_type) in [
        (
            "intMatrix",
            "([[I)Ljava/lang/String;",
            "java.lang.Cloneable[]",
        ),
        (
            "stringMatrix",
            "([[Ljava/lang/String;)Ljava/lang/String;",
            "java.io.Serializable[]",
        ),
        ("intVector", "([I)Ljava/lang/String;", "java.lang.Cloneable"),
    ] {
        let report = recover_method(BUILTIN_INTERFACES, name, descriptor, 1);
        assert_recovered(&report, target_type);
    }
}

#[test]
fn local_upcast_keeps_the_object_array_overload_and_effectful_argument_once() {
    let local = recover_method(
        OVERLOAD,
        "localObjectTarget",
        "([Ljava/lang/String;)Ljava/lang/String;",
        1,
    );
    assert_recovered(&local, "overload(");
    assert_recovered(&local, "java.lang.Object[]");
    assert_argument_origin(&local, "local1", 2, 3);

    let effectful = recover_method(OVERLOAD, "effectfulObjectTarget", "()Ljava/lang/String;", 0);
    assert_recovered(&effectful, "effectfulArray()");
    assert_recovered(&effectful, "java.lang.Object[]");
    assert_eq!(
        effectful.text.matches("effectfulArray()").count(),
        1,
        "{}",
        effectful.text
    );
}

#[test]
fn unknown_user_array_supertypes_keep_the_complete_fallback() {
    for (name, descriptor) in [
        (
            "childToBase",
            "([LUnknownArrayRelation$Child;)Ljava/lang/String;",
        ),
        (
            "childToMarker",
            "([LUnknownArrayRelation$MarkedChild;)Ljava/lang/String;",
        ),
    ] {
        let report = recover_method(UNKNOWN_RELATION, name, descriptor, 1);
        assert!(
            report
                .text
                .contains("no safe reference conversion evidence"),
            "the real invocation refusal was not retained:\n{}",
            report.text
        );
        assert!(
            !report.text_of_bci(1).is_empty(),
            "the refusal lost the real invocation BCI 1: {}",
            report.text
        );
        assert!(
            !report.text_of_bci(0).is_empty(),
            "the refusal lost the real argument load BCI 0: {}",
            report.text
        );
    }
}

#[test]
fn default_and_all_evidence_emit_the_same_successful_body_and_stop_before_partial_output() {
    let essential = recover_method_with_options(
        CORE,
        "typedStringArray",
        "([Ljava/lang/String;)Ljava/lang/String;",
        1,
        false,
        None,
        false,
    );
    let all = recover_method_with_options(
        CORE,
        "typedStringArray",
        "([Ljava/lang/String;)Ljava/lang/String;",
        1,
        true,
        None,
        false,
    );
    assert!(essential.produced() && all.produced());
    assert_eq!(essential.text, all.text);

    let stopped = recover_method_with_options(
        CORE,
        "typedStringArray",
        "([Ljava/lang/String;)Ljava/lang/String;",
        1,
        false,
        Some(8),
        false,
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty());
    assert!(stopped.source_map.is_empty());
    assert!(matches!(stopped.outcome, RecoveryOutcome::Stopped(_)));

    let cancelled = recover_method_with_options(
        CORE,
        "typedStringArray",
        "([Ljava/lang/String;)Ljava/lang/String;",
        1,
        false,
        None,
        true,
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty());
    assert!(cancelled.source_map.is_empty());
    assert!(
        cancelled
            .stop()
            .expect("cancelled run stops")
            .is_cancelled()
    );
}
