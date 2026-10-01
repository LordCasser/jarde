//! An equality comparison's branch-selected 0/1 is a boolean at a `Z` call parameter.
//!
//! The patrol fixture S1 quotes `b.append(t == t.intern())`: `if_acmpXX` lowers the comparison to
//! two arms of `iconst_1`/`iconst_0`, and the value they merge into reaches the `append:(Z)` slot
//! as an `int` no evidence converts. What this file proves through the public surface:
//!
//! 1. the **argument position** now reads that merged value the way the `Z` `ireturn` position
//!    already does — the branch's own comparison is the argument's text (`a == b`, `a != b` spelled
//!    by the branch's sense), for reference equality and numeric equality alike, including a call
//!    whose two parameters are both `Z`;
//! 2. the slice's boundary holds: an **ordering** branch (`<`) and a **zero test** (`ifeq`, the
//!    shape `x == 0` lowers to) keep the refusal they have always had, because neither is the
//!    `if_acmpXX`/`if_icmpXX` pair comparison this change presents;
//! 3. the neighbours do not move: a comparison result stored into a local and passed on, and an
//!    eager `&` a call already accepts as boolean text, recover exactly as before.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const S1: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/string-ops-patrol/fixture/S1.class"
);
const R1: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/string-ops-patrol/repeq-variants/fixture/R1.class"
);
const R2: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/string-ops-patrol/repeq-variants/fixture/R2.class"
);
const R3: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/string-ops-patrol/repeq-variants/fixture/R3.class"
);
const N1: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/string-ops-patrol/repeq-variants/fixture/N1.class"
);
const N2: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/string-ops-patrol/repeq-variants/fixture/N2.class"
);
const N3: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-02/string-ops-patrol/repeq-variants/fixture/N3.class"
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
    parameters: u16,
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
        MethodFacts::new(name, descriptor, parameters)
            .with_access_flags(0x0008)
            .with_declaring_class(DeclaringClass::new(class_name, 0x0009)),
    );
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

/// The patrol anchor: the quoted `append(Z)` of a reference-equality result is the comparison.
#[test]
fn a_reference_equality_argument_is_the_comparison_itself() {
    let report = recover_method(S1, "S1", "ops", "(Ljava/lang/String;)Ljava/lang/String;", 1);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report
            .text
            .contains("local1.append(local2 == local2.intern());"),
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("no proven conversion"),
        "{}",
        report.text
    );
    // The collapsed argument is the consumer's own node: the `append` site keeps its direct
    // anchor, and the branch that selected the arms stays visible as a derived one.
    assert!(
        !report.source_map.direct_of_bci(156).is_empty(),
        "{}",
        report.text
    );
    assert!(
        report
            .source_map
            .segments()
            .iter()
            .any(|segment| segment.origin().derived().iter().any(|origin| {
                origin.bci() == 148
                    && segment
                        .text(&report.text)
                        .contains("local2 == local2.intern()")
            })),
        "{}",
        report.text
    );
}

/// `!=` is the spelling of the inverted sense: the fall-through arm still carries the `1`.
#[test]
fn a_reference_inequality_argument_keeps_the_branches_own_sense() {
    let report = recover_method(R1, "R1", "ops", "(Ljava/lang/String;)Ljava/lang/String;", 1);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report.text.contains("local1.append(local2 != arg0);"),
        "{}",
        report.text
    );
}

/// Numeric equality feeds a two-`Z` static call beside an already-boolean second argument.
#[test]
fn a_numeric_equality_argument_reaches_a_two_boolean_parameter_call() {
    let report = recover_method(R2, "R2", "ops", "(Ljava/lang/String;)Ljava/lang/String;", 1);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report
            .text
            .contains("return mark(local1.length() == arg0.length(), arg0.isEmpty());"),
        "{}",
        report.text
    );
}

/// The store path is the declaration plan's own decision and does not move: the stored local keeps
/// its `boolean` declaration and the load keeps the text it had before this change.
#[test]
fn a_stored_comparison_result_passes_on_unchanged() {
    let report = recover_method(R3, "R3", "ops", "(Ljava/lang/String;)Ljava/lang/String;", 1);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report.text.contains("boolean local3 = local2 == arg0;"),
        "{}",
        report.text
    );
    assert!(
        report.text.contains("local1.append(local3);"),
        "{}",
        report.text
    );
}

/// An ordering comparison is not this slice's pair equality: the refusal stands.
#[test]
fn an_ordering_comparison_argument_stays_refused() {
    let report = recover_method(N1, "N1", "ops", "(Ljava/lang/String;)Ljava/lang/String;", 1);
    assert!(
        report.text.contains("no proven conversion to `boolean`"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("take("), "{}", report.text);
}

/// `x == 0` lowers to a zero test, not `if_icmpXX`: the refusal stands there too.
#[test]
fn a_zero_test_equality_argument_stays_refused() {
    let report = recover_method(N3, "N3", "ops", "(Ljava/lang/String;)Ljava/lang/String;", 1);
    assert!(
        report.text.contains("no proven conversion to `boolean`"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("take("), "{}", report.text);
}

/// An eager `&` of two booleans already states its own boolean type; nothing about this change
/// rewrites it.
#[test]
fn an_eager_boolean_and_argument_recovers_as_before() {
    let report = recover_method(N2, "N2", "ops", "(ZZ)Ljava/lang/String;", 2);
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(
        report.text.contains("return take(arg0 & arg1);"),
        "{}",
        report.text
    );
}
