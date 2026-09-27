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
    ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
    PhysicalMethodId, PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const FIXTURE: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-shared-catchall-finally/v8/SharedFinallyJoin.class");
const EMPTY_CATCH_TEST16: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/Test16.class"
);
const EMPTY_CATCH_TEST16_NEIGHBORS: [&[u8]; 5] = [
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/near/different-target.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/near/cleanup-covered.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/near/rows-swapped.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/near/throwable-rewritten.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test16-empty-catch/classes/near/external-cleanup-entry.class"
    ),
];
const TWO_CATCH_TEST17: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test17-two-catches/classes/Test17.class"
);
const TWO_CATCH_TEST17_NEIGHBORS: [&[u8]; 7] = [
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test17-two-catches/classes/near/different-target.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test17-two-catches/classes/near/cleanup-covered.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test17-two-catches/classes/near/cleanup-self-protected.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test17-two-catches/classes/near/rows-swapped.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test17-two-catches/classes/near/saved-return-rewritten.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test17-two-catches/classes/near/throwable-rewritten.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test17-two-catches/classes/near/external-cleanup-entry.class"
    ),
];
const NESTED_TEST_CLS: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-28/cf16-nested-finally/TestTryCatchFinally12$TestCls.class"
);
const SEGMENTED_TEST13_FIXED: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/TestTryCatchFinally13$TestCls.fixed.class"
);
const SEGMENTED_TEST13_ACCEPTANCE: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/acceptance/TestTryCatchFinally13$TestCls.class"
);
const CONCAT_SAVED: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-concat-saved-finally/v8/FinallyOnce.class");
const CONDITIONAL_TEST14: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/TestTryCatchFinally14$TestCls.class"
);
const CONDITIONAL_TEST14_MINIMAL: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-conditional-finally/Test14-minimal.class");
const CONDITIONAL_TEST14_NEGATIVES: [&[u8]; 9] = [
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-field.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-call.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-predicate.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-self-protected.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-external-entry.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup/negatives/Test14-throwable.class"
    ),
    include_bytes!("../../../tests/fixtures/p3-conditional-finally/Test14-slot0.class"),
    include_bytes!("../../../tests/fixtures/p3-conditional-finally/Test14-second-field.class"),
    include_bytes!("../../../tests/fixtures/p3-conditional-finally/Test14-handler-call.class"),
];
const CONCAT_SAVED_NEGATIVES: [&[u8]; 3] = [
    include_bytes!("../../../tests/fixtures/p3-concat-saved-finally/negatives/non-concat.class"),
    include_bytes!(
        "../../../tests/fixtures/p3-concat-saved-finally/negatives/extra-consumer.class"
    ),
    include_bytes!("../../../tests/fixtures/p3-concat-saved-finally/negatives/range-shrunk.class"),
];
const SEGMENTED_TEST13_NEGATIVES: [&[u8]; 5] = [
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/negatives/cleanup-target.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/negatives/branch-bypass.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/negatives/range-expanded.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/negatives/rethrow-changed.class"
    ),
    include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-09-28/cf16-test13-multisegment/acceptance/external-entry.class"
    ),
];
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
fn empty_catch_test16_has_one_finally_and_two_real_rows() {
    let recover = |class, budget| {
        recover_method(
            class,
            "test",
            "()V",
            "jadx/tests/integration/trycatch/TestTryCatchFinally16$TestCls",
            0x0001,
            budget,
        )
    };
    let report = recover(EMPTY_CATCH_TEST16, None);
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert_eq!(
        report.text.matches("finally {").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("doFinally();").count(),
        1,
        "{}",
        report.text
    );
    assert!(
        report
            .text
            .contains("catch (java.lang.Exception arg1) {\n    }"),
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
        BTreeSet::from([0, 9, 16, 22])
    );
    assert_eq!(blocks.len(), 4);
    for bci in [0, 3, 6, 9, 10, 13, 16, 17, 20, 21, 22] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "missing BCI {bci}"
        );
    }
    for neighbor in EMPTY_CATCH_TEST16_NEIGHBORS {
        let refused = recover(neighbor, None);
        assert!(!refused.text.contains("finally {"), "{}", refused.text);
        assert!(refused.text.contains("@bytecode"), "{}", refused.text);
    }
    let mut tiny = limits();
    tiny.ir_items = 1;
    let stopped = recover(EMPTY_CATCH_TEST16, Some(Budget::new(tiny)));
    assert!(stopped.stop().is_some());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    let full = recover(EMPTY_CATCH_TEST16, Some(Budget::new(limits())));
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("complete run")
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover(EMPTY_CATCH_TEST16, Some(Budget::new(late)));
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    let mut output = limits();
    output.output_bytes = 1;
    let stopped = recover(EMPTY_CATCH_TEST16, Some(Budget::new(output)));
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::OutputBytes,
            ..
        })
    ));
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover(
        EMPTY_CATCH_TEST16,
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
}

#[test]
fn two_catch_test17_has_one_finally_and_all_physical_origins() {
    let recover = |class, budget| {
        recover_method(
            class,
            "test",
            "()I",
            "jadx/tests/integration/trycatch/TestTryCatchFinally17$TestCls",
            0x0001,
            budget,
        )
    };
    let report = recover(TWO_CATCH_TEST17, None);
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert_eq!(
        report.text.matches("finally {").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("doFinally();").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(report.text.matches("catch (").count(), 2, "{}", report.text);
    assert!(report.text.contains("return 1;"), "{}", report.text);
    assert!(report.text.contains("return 0;"), "{}", report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    let blocks: Vec<_> = report
        .regions
        .iter()
        .flat_map(|region| &region.blocks)
        .copied()
        .collect();
    assert_eq!(
        blocks.iter().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([0, 9, 16, 24, 30])
    );
    assert_eq!(blocks.len(), 5);
    for bci in [
        0, 3, 6, 9, 10, 13, 16, 17, 18, 19, 22, 23, 24, 25, 28, 29, 30, 31,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "missing BCI {bci}"
        );
    }
    for neighbor in TWO_CATCH_TEST17_NEIGHBORS {
        let refused = recover(neighbor, None);
        assert!(!refused.text.contains("finally {"), "{}", refused.text);
        assert!(refused.text.contains("@bytecode"), "{}", refused.text);
    }
    let full = recover(TWO_CATCH_TEST17, Some(Budget::new(limits())));
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("complete run")
    };
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover(TWO_CATCH_TEST17, Some(Budget::new(late)));
    assert!(stopped.stop().is_some());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    let mut output = limits();
    output.output_bytes = 1;
    let stopped = recover(TWO_CATCH_TEST17, Some(Budget::new(output)));
    assert!(stopped.stop().is_some());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover(
        TWO_CATCH_TEST17,
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
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

#[test]
fn segmented_test13_has_one_owner_and_all_physical_origins() {
    for class in [SEGMENTED_TEST13_FIXED, SEGMENTED_TEST13_ACCEPTANCE] {
        let report = recover_method(
            class,
            "test",
            "(I)V",
            "jadx/tests/integration/trycatch/TestTryCatchFinally13$TestCls",
            0x0001,
            None,
        );
        assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
        assert_eq!(
            report.text.matches("finally {").count(),
            1,
            "{}",
            report.text
        );
        assert_eq!(
            report.text.matches("this.doSomething4();").count(),
            1,
            "{}",
            report.text
        );
        assert!(
            report.text.contains("== -12) {\n            return;"),
            "{}",
            report.text
        );
        assert!(
            report.text.contains("catch (java.lang.Exception"),
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
            blocks.len(),
            blocks.iter().copied().collect::<BTreeSet<_>>().len()
        );
        assert_eq!(
            blocks.iter().copied().collect::<BTreeSet<_>>(),
            BTreeSet::from([0, 10, 15, 21, 28, 33, 37, 44, 56, 63]),
            "{}",
            report.text,
        );
        for bci in [
            0, 1, 4, 5, 7, 10, 11, 14, 15, 16, 18, 21, 22, 25, 28, 29, 30, 33, 34, 37, 38, 41, 44,
            45, 46, 49, 50, 53, 56, 57, 58, 61, 62, 63,
        ] {
            assert!(
                !report.source_map.of_bci(bci).is_empty(),
                "BCI {bci}: {}",
                report.text
            );
        }
    }
}

#[test]
fn segmented_test13_neighbors_and_stops_publish_no_partial_finally() {
    for class in SEGMENTED_TEST13_NEGATIVES {
        let report = recover_method(
            class,
            "test",
            "(I)V",
            "jadx/tests/integration/trycatch/TestTryCatchFinally13$TestCls",
            0x0001,
            None,
        );
        assert!(!report.text.contains("finally {"), "{}", report.text);
    }
    for budget in [
        {
            let mut tiny = limits();
            tiny.ir_items = 1;
            Budget::new(tiny)
        },
        {
            let mut tiny = limits();
            tiny.output_bytes = 1;
            Budget::new(tiny)
        },
        {
            let token = CancellationToken::new();
            token.cancel();
            Budget::with_cancellation_token(limits(), token)
        },
    ] {
        let report = recover_method(
            SEGMENTED_TEST13_ACCEPTANCE,
            "test",
            "(I)V",
            "jadx/tests/integration/trycatch/TestTryCatchFinally13$TestCls",
            0x0001,
            Some(budget),
        );
        assert!(report.text.is_empty() && report.source_map.is_empty());
        assert!(report.stop().is_some());
    }
}

#[test]
fn concat_saved_return_has_one_finally_and_complete_physical_origins() {
    let report = recover_method(
        CONCAT_SAVED,
        "handled",
        "(Z)Ljava/lang/String;",
        "FinallyOnce",
        0x0009,
        None,
    );
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        report.text.matches("finally {").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("return \"caught:\"").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("cleanupCount =").count(),
        2,
        "{}",
        report.text
    );
    assert!(
        report.text.contains("return \"normal\";"),
        "{}",
        report.text
    );
    let blocks: Vec<_> = report
        .regions
        .iter()
        .flat_map(|region| &region.blocks)
        .copied()
        .collect();
    assert_eq!(blocks.len(), blocks.iter().collect::<BTreeSet<_>>().len());
    for bci in [
        0, 1, 4, 5, 8, 11, 12, 14, 17, 18, 20, 21, 24, 25, 26, 29, 30, 31, 32, 35, 36, 39, 41, 44,
        45, 48, 51, 54, 55, 58, 59, 60, 63, 64, 65, 66, 69, 70, 71, 74, 75,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no origin"
        );
    }
}

#[test]
fn concat_saved_return_neighbors_and_stops_refuse_atomically() {
    for class in CONCAT_SAVED_NEGATIVES {
        let report = recover_method(
            class,
            "handled",
            "(Z)Ljava/lang/String;",
            "FinallyOnce",
            0x0009,
            None,
        );
        assert!(!report.text.contains("finally {"), "{}", report.text);
        assert!(report.text.contains("@bytecode"), "{}", report.text);
    }
    for budget in [
        {
            let mut tiny = limits();
            tiny.ir_items = 1;
            Budget::new(tiny)
        },
        {
            let mut tiny = limits();
            tiny.output_bytes = 1;
            Budget::new(tiny)
        },
        {
            let token = CancellationToken::new();
            token.cancel();
            Budget::with_cancellation_token(limits(), token)
        },
    ] {
        let report = recover_method(
            CONCAT_SAVED,
            "handled",
            "(Z)Ljava/lang/String;",
            "FinallyOnce",
            0x0009,
            Some(budget),
        );
        assert!(report.text.is_empty() && report.source_map.is_empty());
        assert!(report.stop().is_some());
    }
}

#[test]
fn conditional_test14_has_two_bounded_if_regions_and_all_origins() {
    let report = recover_method(
        CONDITIONAL_TEST14,
        "test",
        "()V",
        "jadx/tests/integration/trycatch/TestTryCatchFinally14$TestCls",
        0x0001,
        None,
    );
    assert!(report.produced(), "{:?}\n{}", report.outcome, report.text);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        report.text.matches("finally {").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("this.t != null").count(),
        2,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("this.t.doSomething();").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(
        report.text.matches("this.t.doFinally();").count(),
        1,
        "{}",
        report.text
    );
    assert_eq!(report.regions.len(), 1);
    assert_eq!(report.regions[0].blocks, [0, 7, 14, 21, 31, 39, 46, 48]);
    for bci in [
        0, 1, 4, 7, 8, 11, 14, 15, 18, 21, 22, 25, 28, 31, 32, 33, 36, 39, 40, 43, 46, 47, 48,
    ] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} has no origin"
        );
    }
    let minimal = recover_method(
        CONDITIONAL_TEST14_MINIMAL,
        "test",
        "()V",
        "jadx/tests/integration/trycatch/TestTryCatchFinally14$TestCls",
        0x0001,
        None,
    );
    assert_eq!(minimal.text, report.text);
}

#[test]
fn conditional_test14_neighbors_and_stops_refuse_atomically() {
    for class in CONDITIONAL_TEST14_NEGATIVES {
        let report = recover_method(
            class,
            "test",
            "()V",
            "jadx/tests/integration/trycatch/TestTryCatchFinally14$TestCls",
            0x0001,
            None,
        );
        assert!(!report.text.contains("finally {"), "{}", report.text);
        assert!(report.text.contains("@bytecode"), "{}", report.text);
    }
    for budget in [
        {
            let mut tiny = limits();
            tiny.ir_items = 1;
            Budget::new(tiny)
        },
        {
            let mut tiny = limits();
            tiny.output_bytes = 1;
            Budget::new(tiny)
        },
        {
            let token = CancellationToken::new();
            token.cancel();
            Budget::with_cancellation_token(limits(), token)
        },
    ] {
        let report = recover_method(
            CONDITIONAL_TEST14,
            "test",
            "()V",
            "jadx/tests/integration/trycatch/TestTryCatchFinally14$TestCls",
            0x0001,
            Some(budget),
        );
        assert!(report.text.is_empty() && report.source_map.is_empty());
        assert!(report.stop().is_some());
    }
}
