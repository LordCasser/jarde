//! The fixed CF-16 Test2 void finally: one protected body of three ordinary loops, two cleanup
//! copies of one `close`, and a void completion. The certificate is deliberately closed, so every
//! verifier-valid neighbor of the frozen class refuses and stops stay whole.

use std::collections::BTreeSet;

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    StopReason, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, Quality};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CancellationToken, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const VOID_LOOP_TEST2: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test2-loop-finally/fixed/TestTryCatchFinally2$TestCls.class"
);
const VOID_LOOP_TEST2_NEIGHBORS: [(&str, &[u8]); 6] = [
    (
        "different-receiver",
        include_bytes!(
            "../../../openspec/changes/recover-void-loop-finally/verification/neighbors/different-receiver.class"
        ),
    ),
    (
        "different-target",
        include_bytes!(
            "../../../openspec/changes/recover-void-loop-finally/verification/neighbors/different-target.class"
        ),
    ),
    (
        "resource-definition-rewritten",
        include_bytes!(
            "../../../openspec/changes/recover-void-loop-finally/verification/neighbors/resource-definition-rewritten.class"
        ),
    ),
    (
        "self-row-expanded",
        include_bytes!(
            "../../../openspec/changes/recover-void-loop-finally/verification/neighbors/self-row-expanded.class"
        ),
    ),
    (
        "throwable-rewritten",
        include_bytes!(
            "../../../openspec/changes/recover-void-loop-finally/verification/neighbors/throwable-rewritten.class"
        ),
    ),
    (
        "loop-extra-exit",
        include_bytes!(
            "../../../openspec/changes/recover-void-loop-finally/verification/neighbors/loop-extra-exit.class"
        ),
    ),
];

/// Every instruction start of the fixed `test(OutputStream):void`, in BCI order: the physical
/// surface the certificate owns end to end.
const BCIS: [u32; 89] = [
    0, 3, 4, 5, 8, 9, 10, 11, 14, 15, 16, 19, 20, 23, 24, 27, 28, 29, 30, 32, 33, 35, 37, 39, 42,
    43, 45, 46, 48, 49, 50, 52, 55, 58, 61, 64, 65, 68, 69, 70, 71, 73, 74, 76, 78, 80, 83, 84, 86,
    87, 89, 91, 94, 96, 97, 99, 100, 103, 105, 107, 109, 110, 112, 113, 115, 117, 119, 122, 124,
    126, 127, 129, 130, 132, 135, 138, 141, 144, 147, 150, 153, 154, 157, 160, 162, 163, 166, 168,
    169,
];

fn recover_void_loop_test2(
    class: &[u8],
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
    recover_method(
        class,
        "test",
        "(Ljava/io/OutputStream;)V",
        "jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls",
        0x0001,
        recovery_budget,
    )
}

fn recover_method(
    class: &[u8],
    name: &str,
    descriptor: &str,
    owner: &str,
    flags: u16,
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
    let mut budget = Budget::new(limits());
    let snapshot =
        ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget).expect("opens");
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: class.len() as u64,
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
    };
    let domain = LoadDomain {
        loader: LoaderId("app".into()),
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
        &[snapshot.clone()],
        &jarde_jvm::ir::MethodAnalysisRequest {
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
    .expect("fixed class analyzes");
    let facts = RecoveryFacts::new(
        MethodFacts::new(name, descriptor, 2)
            .with_access_flags(flags)
            .with_declaring_class(DeclaringClass::new(owner, 0x0021)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

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

#[test]
fn void_loop_test2_has_one_finally_one_close_and_all_bci_origins() {
    let report = recover_void_loop_test2(VOID_LOOP_TEST2, None);
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert!(
        report.regions.iter().all(|region| region.structured),
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        report.text.matches("finally {").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches(".close();").count(),
        1,
        "{}",
        report.text
    );
    // The three normal loops survive as statements of their own: two outer traversals and the
    // nested parent traversal, in the order the bytecode writes them.
    let loops = report.text.matches("while (").count() + report.text.matches("for (").count();
    assert_eq!(loops, 3, "{}", report.text);
    for bci in BCIS {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci}: {}",
            report.text
        );
    }
}

#[test]
fn void_loop_test2_verifier_valid_neighbors_refuse_finally() {
    for (name, class) in VOID_LOOP_TEST2_NEIGHBORS {
        let report = recover_void_loop_test2(class, None);
        assert!(
            !report.text.contains("finally {"),
            "{name}: {}",
            report.text
        );
        assert_eq!(report.quality, Quality::Fallback, "{name}: {}", report.text);
        assert!(report.text.contains("@bytecode"), "{name}: {}", report.text);
    }
}

#[test]
fn void_loop_test2_budget_and_cancellation_discard_partial_output() {
    let mut tiny = limits();
    tiny.analysis_steps = 1;
    let stopped = recover_void_loop_test2(VOID_LOOP_TEST2, Some(Budget::new(tiny)));
    assert!(matches!(stopped.stop(), Some(StopReason::Budget { .. })));
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_void_loop_test2(
        VOID_LOOP_TEST2,
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
}

#[test]
fn void_loop_test2_owner_bcis_are_the_physical_statement_span() {
    let report = recover_void_loop_test2(VOID_LOOP_TEST2, None);
    assert!(report.produced(), "{}", report.text);
    let owned: BTreeSet<u32> = report
        .regions
        .iter()
        .flat_map(|region| region.blocks.iter().copied())
        .collect();
    // The statement owns every block but the one canonical block starts: the entry block the lead
    // shares with the body and the fused normal cleanup are among them; no block is left quoted.
    assert_eq!(
        owned,
        BTreeSet::from([0, 35, 42, 64, 76, 83, 115, 122, 147, 153, 160]),
        "{}",
        report.text
    );
}
