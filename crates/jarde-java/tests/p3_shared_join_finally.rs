//! The fixed JADX shared-join finally class has one owner for every physical block.

use std::collections::BTreeSet;

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    StopReason, pass::JAVA_8, recover,
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

const FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoin.class");
const NESTED_TEST_CLS: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/TestTryCatchFinally12$TestCls.class"
);
const NESTED_OTHER_CONSTANT: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test3MatchingOtherConstant.class"
);
const NESTED_TEST3_NEGATIVES: [&[u8]; 5] = [
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test3DifferentConstant.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test3DifferentField.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test3DifferentTarget.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test3StoredResult.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test3WidenedRow.class"
    ),
];
const NESTED_TEST12_NEGATIVES: [(&str, &[u8]); 8] = [
    (
        "test1",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test1DifferentConstant.class"
        ),
    ),
    (
        "test1",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test1WidenedRow.class"
        ),
    ),
    (
        "test1",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test1BypassCleanup.class"
        ),
    ),
    (
        "test1",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test1ChangedRethrow.class"
        ),
    ),
    (
        "test2",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test2DifferentConstant.class"
        ),
    ),
    (
        "test2",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test2WidenedRow.class"
        ),
    ),
    (
        "test2",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test2BypassCleanup.class"
        ),
    ),
    (
        "test2",
        include_bytes!(
            "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/Test2ChangedRethrow.class"
        ),
    ),
];
const NEGATIVES: [&[u8]; 3] = [
    include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoinValueMismatch.class"
    ),
    include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoinRow.class"
    ),
    include_bytes!(
        "../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoinEdge.class"
    ),
];

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

fn recover_test(class: &[u8], recovery_budget: Option<Budget>) -> jarde_java::RecoveryReport {
    recover_method(
        class,
        "test",
        "(Ljava/lang/Object;)Z",
        "TestTryCatchFinally$TestCls",
        0x0002,
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
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("fixture opens");
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

#[test]
fn fixed_shared_join_has_one_finally_and_complete_physical_ownership() {
    let report = recover_test(FIXTURE, None);
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert_eq!(report.text.matches("this.f = true;").count(), 1);
    assert!(report.text.contains("finally {"), "{}", report.text);
    assert!(
        report.text.contains("}\n    return this.f;"),
        "{}",
        report.text
    );
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    let blocks: Vec<_> = report
        .regions
        .iter()
        .flat_map(|region| &region.blocks)
        .copied()
        .collect();
    assert_eq!(
        blocks.iter().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([0, 18, 31, 39])
    );
    assert_eq!(blocks.len(), 4, "each physical block has one region owner");
    for bci in [
        0, 1, 2, 5, 6, 9, 10, 11, 12, 15, 18, 19, 20, 23, 24, 25, 28, 31, 32, 33, 34, 37, 38, 39,
        40, 43,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no source origin"
        );
    }
}

#[test]
fn nested_test3_has_one_finally_and_complete_physical_ownership() {
    let report = recover_method(
        NESTED_TEST_CLS,
        "test3",
        "(I)V",
        "jadx/tests/integration/trycatch/TestTryCatchFinally12$TestCls",
        0x0001,
        None,
    );
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert_eq!(
        report.text.matches("this.sb.append(\"-finally\")").count(),
        1,
        "{}",
        report.text
    );
    assert!(report.text.contains("finally {"), "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    let blocks: Vec<_> = report
        .regions
        .iter()
        .flat_map(|region| &region.blocks)
        .copied()
        .collect();
    assert_eq!(
        blocks.iter().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([0, 18, 42, 55])
    );
    assert_eq!(blocks.len(), 4);
    for bci in [
        0, 1, 2, 5, 6, 9, 11, 14, 15, 18, 19, 20, 23, 25, 28, 29, 30, 33, 35, 38, 39, 42, 43, 44,
        47, 49, 52, 53, 54, 55,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no origin"
        );
    }
}

#[test]
fn nested_test1_and_test2_have_one_outer_finally_and_complete_ownership() {
    for (name, expected_blocks, bcis) in [
        (
            "test1",
            &[0, 8, 19, 42, 55][..],
            &[
                0, 1, 2, 5, 8, 9, 10, 13, 15, 18, 19, 20, 23, 25, 28, 29, 30, 33, 35, 38, 39, 42,
                43, 44, 47, 49, 52, 53, 54, 55,
            ][..],
        ),
        (
            "test2",
            &[0, 8, 19, 32][..],
            &[
                0, 1, 2, 5, 8, 9, 10, 13, 15, 18, 19, 20, 23, 25, 28, 29, 32, 33, 34, 37, 39, 42,
                43, 44, 45,
            ][..],
        ),
    ] {
        let report = recover_method(
            NESTED_TEST_CLS,
            name,
            "(I)V",
            "jadx/tests/integration/trycatch/TestTryCatchFinally12$TestCls",
            0x0001,
            None,
        );
        assert!(
            report.produced(),
            "{name}: {:?}\n{}",
            report.outcome,
            report.text
        );
        assert_eq!(
            report.text.matches("finally {").count(),
            1,
            "{name}: {}",
            report.text
        );
        assert_eq!(
            report.text.matches("this.sb.append(\"-finally\")").count(),
            1,
            "{name}: {}",
            report.text
        );
        assert!(
            report.text.contains("NullPointerException"),
            "{name}: {}",
            report.text
        );
        assert!(
            !report.text.contains("@bytecode"),
            "{name}: {}",
            report.text
        );
        let blocks: Vec<_> = report
            .regions
            .iter()
            .flat_map(|region| &region.blocks)
            .copied()
            .collect();
        assert_eq!(blocks, expected_blocks, "{name}: {}", report.text);
        for &bci in bcis {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "{name}: BCI {bci} has no origin"
            );
        }
    }
}

#[test]
fn nested_test3_equal_other_string_is_still_one_finally() {
    let report = recover_method(
        NESTED_OTHER_CONSTANT,
        "test3",
        "(I)V",
        "jadx/tests/integration/trycatch/TestTryCatchFinally12$TestCls",
        0x0001,
        None,
    );
    assert!(report.text.contains("finally {"), "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(report.text.matches("this.sb.append(\"-catch\")").count(), 2);
}

#[test]
fn nested_test3_verifier_valid_near_misses_refuse_shared_finally() {
    for class in NESTED_TEST3_NEGATIVES {
        let report = recover_method(
            class,
            "test3",
            "(I)V",
            "jadx/tests/integration/trycatch/TestTryCatchFinally12$TestCls",
            0x0001,
            None,
        );
        assert!(!report.text.contains("finally {"), "{}", report.text);
        assert!(report.text.contains("@bytecode"), "{}", report.text);
    }
}

#[test]
fn nested_test1_and_test2_verifier_valid_near_misses_refuse_outer_finally() {
    for (method, class) in NESTED_TEST12_NEGATIVES {
        let report = recover_method(
            class,
            method,
            "(I)V",
            "jadx/tests/integration/trycatch/TestTryCatchFinally12$TestCls",
            0x0001,
            None,
        );
        assert!(
            !report.text.contains("finally {"),
            "{method}: {}",
            report.text
        );
        assert!(
            report.text.contains("@bytecode"),
            "{method}: {}",
            report.text
        );
    }
}

#[test]
fn nested_test1_and_test2_stop_discards_text_and_source_map() {
    for method in ["test1", "test2"] {
        let recover = |budget| {
            recover_method(
                NESTED_TEST_CLS,
                method,
                "(I)V",
                "jadx/tests/integration/trycatch/TestTryCatchFinally12$TestCls",
                0x0001,
                Some(budget),
            )
        };
        let mut tiny = limits();
        tiny.ir_items = 1;
        let stopped = recover(Budget::new(tiny));
        assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
        assert!(matches!(stopped.stop(), Some(StopReason::Budget { .. })));
        let token = CancellationToken::new();
        token.cancel();
        let cancelled = recover(Budget::with_cancellation_token(limits(), token));
        assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
        assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
    }
}

#[test]
fn nested_test3_stop_discards_text_and_source_map() {
    let recover = |budget| {
        recover_method(
            NESTED_TEST_CLS,
            "test3",
            "(I)V",
            "jadx/tests/integration/trycatch/TestTryCatchFinally12$TestCls",
            0x0001,
            Some(budget),
        )
    };
    let mut tiny = limits();
    tiny.ir_items = 1;
    let stopped = recover(Budget::new(tiny));
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(stopped.stop(), Some(StopReason::Budget { .. })));
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover(Budget::with_cancellation_token(limits(), token));
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}

#[test]
fn verifier_valid_boundary_variants_do_not_publish_a_partial_finally() {
    for class in NEGATIVES {
        let report = recover_test(class, None);
        assert!(!report.text.contains("finally {"), "{}", report.text);
        assert!(
            report.text.contains("BCI 31") && report.text.contains("@bytecode"),
            "{}",
            report.text
        );
    }
}

#[test]
fn shared_join_stop_discards_text_and_source_map() {
    let mut tiny = limits();
    tiny.ir_items = 1;
    let stopped = recover_test(FIXTURE, Some(Budget::new(tiny)));
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::IrItems,
            ..
        })
    ));
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_test(
        FIXTURE,
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}
