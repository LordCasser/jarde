//! An exact 0/1 stack Phi may be projected to its Boolean test only at an `ireturn Z`.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, CountedBudgetDimension, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const INSTANCEOF_MERGE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-25/instanceof-boolean-merge/InstanceOfMerge.class"
);
const BOOLEAN_MERGE_CONTROLS: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-25/instanceof-boolean-merge/BooleanMergeControls.class"
);

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
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
        nested_depth: 16,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn recover_method(
    class_bytes: &[u8],
    class_name: &str,
    name: &str,
    descriptor: &str,
) -> jarde_java::RecoveryReport {
    recover_method_with_recovery_budget(class_bytes, class_name, name, descriptor, None)
}

fn recover_method_with_recovery_budget(
    class_bytes: &[u8],
    class_name: &str,
    name: &str,
    descriptor: &str,
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class_bytes.to_vec()), &mut budget)
        .expect("frozen class opens");
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class_bytes).to_hex().to_string()),
                length: u64::try_from(class_bytes.len()).expect("class length fits"),
            },
            variant: PhysicalVariant::Base,
        },
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
    let analysis = analyze_method_ir(
        std::slice::from_ref(&snapshot),
        &MethodAnalysisRequest {
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
        },
        &mut budget,
    )
    .expect("frozen method analysis completes");
    let facts = RecoveryFacts::new(
        MethodFacts::new(name, descriptor, 1)
            .with_access_flags(0x0008)
            .with_declaring_class(DeclaringClass::new(class_name, 0x0031)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

#[test]
fn exact_boolean_merges_keep_the_test_polarity_and_all_origins() {
    let inverted = recover_method(
        INSTANCEOF_MERGE,
        "InstanceOfMerge",
        "inverted",
        "(Ljava/lang/Object;)Z",
    );
    assert!(inverted.produced(), "{:?}", inverted.outcome);
    assert!(
        inverted
            .text
            .contains("return !(value(arg0) instanceof java.lang.String);"),
        "{}",
        inverted.text
    );
    assert_eq!(inverted.text.matches("value(arg0)").count(), 1);
    for bci in [4, 7, 10, 11, 14, 15] {
        assert!(
            !inverted.source_map.of_bci(bci).is_empty(),
            "inverted misses source BCI {bci}"
        );
    }

    let direct = recover_method(
        INSTANCEOF_MERGE,
        "InstanceOfMerge",
        "direct",
        "(Ljava/lang/Object;)Z",
    );
    assert!(direct.produced(), "{:?}", direct.outcome);
    assert!(
        direct
            .text
            .contains("return value(arg0) instanceof java.lang.String;"),
        "{}",
        direct.text
    );

    let positive = recover_method(
        BOOLEAN_MERGE_CONTROLS,
        "BooleanMergeControls",
        "positive",
        "(Ljava/lang/Object;)Z",
    );
    assert!(positive.produced(), "{:?}", positive.outcome);
    assert!(
        positive
            .text
            .contains("return value(arg0) instanceof java.lang.String;"),
        "{}",
        positive.text
    );
    assert_eq!(positive.text.matches("value(arg0)").count(), 1);

    let negative = recover_method(
        BOOLEAN_MERGE_CONTROLS,
        "BooleanMergeControls",
        "negative",
        "(Ljava/lang/Object;)Z",
    );
    assert!(negative.produced(), "{:?}", negative.outcome);
    assert!(
        negative
            .text
            .contains("return !(value(arg0) instanceof java.lang.String);"),
        "{}",
        negative.text
    );
    assert_eq!(negative.text.matches("value(arg0)").count(), 1);
}

#[test]
fn non_boolean_integer_phi_keeps_the_low_bit_conversion() {
    // In the frozen positive method, each arm is one `iconst` followed by the proved join. Change
    // only those opcodes to `iconst_2` and `iconst_3`; both retain verifier type `int` and create a
    // useful negative case without changing the frozen evidence files.
    let mut non_boolean = BOOLEAN_MERGE_CONTROLS.to_vec();
    let positive = [
        0x2a, 0xb8, 0x00, 0x0d, 0xc1, 0x00, 0x11, 0x99, 0x00, 0x07, 0x04, 0xa7, 0x00, 0x04, 0x03,
        0xac,
    ];
    let hits = non_boolean
        .windows(positive.len())
        .enumerate()
        .filter_map(|(index, bytes)| (bytes == positive).then_some(index))
        .collect::<Vec<_>>();
    let [start] = hits.as_slice() else {
        panic!("frozen positive bytecode sequence must be unique: {hits:?}");
    };
    non_boolean[start + 10] = 0x05; // iconst_2
    non_boolean[start + 14] = 0x06; // iconst_3

    let recovered = recover_method(
        &non_boolean,
        "BooleanMergeControls",
        "positive",
        "(Ljava/lang/Object;)Z",
    );
    assert!(recovered.produced(), "{:?}", recovered.outcome);
    assert!(
        recovered.text.contains("? 2 : 3) % 2 != 0"),
        "non-0/1 return lost low-bit behavior:\n{}",
        recovered.text
    );
}

#[test]
fn boolean_projection_stops_atomically_on_output_budget_or_cancellation() {
    use jarde_java::StopReason;

    let mut bounded = limits();
    bounded.output_bytes = 1;
    let output = recover_method_with_recovery_budget(
        BOOLEAN_MERGE_CONTROLS,
        "BooleanMergeControls",
        "negative",
        "(Ljava/lang/Object;)Z",
        Some(Budget::new(bounded)),
    );
    assert!(!output.produced());
    assert!(output.text.is_empty() && output.source_map.is_empty());
    assert!(
        matches!(
            output.stop(),
            Some(StopReason::Budget {
                dimension: CountedBudgetDimension::OutputBytes,
                ..
            })
        ),
        "unexpected stop: {:?}",
        output.stop()
    );

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_method_with_recovery_budget(
        BOOLEAN_MERGE_CONTROLS,
        "BooleanMergeControls",
        "negative",
        "(Ljava/lang/Object;)Z",
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(
        cancelled
            .stop()
            .expect("cancelled run has a stop")
            .is_cancelled()
    );
}
