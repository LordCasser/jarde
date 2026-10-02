//! The frozen CF-07 Java 8 loop return leaf has one owner and a proved source.

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

const FIXTURE: &[u8] = include_bytes!("../../../tests/fixtures/p3-loop-terminal/LoopCases.class");
const NEGATIVES: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-loop-terminal/cf07/LoopTerminalNegatives.class");
// From the same Java 8 class: change extraEntry's BCI 9 `iload 4; ireturn` to `goto 26`,
// add the BCI 26 StackMapTable frame, and update the two enclosing attribute lengths. This
// creates a second normal entry to the inner leaf; the class passes `java -Xverify:all`.
const EXTRA_ENTRY: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-loop-terminal/LoopTerminalExtraEntry.class");

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
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
) -> jarde_java::RecoveryReport {
    recover_method_with_budget(name, descriptor, evidence, None)
}

fn recover_method_with_budget(
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
    recovery_budget: Option<Budget>,
) -> jarde_java::RecoveryReport {
    recover_class_method_with_budget(
        FIXTURE,
        "cf07/LoopCases",
        name,
        descriptor,
        evidence,
        recovery_budget,
    )
}

fn recover_class_method_with_budget(
    class: &[u8],
    owner: &str,
    name: &str,
    descriptor: &str,
    evidence: RecoveryEvidenceRequest,
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
                length: u64::try_from(class.len()).expect("fixture length fits"),
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
    .expect("fixture method analysis completes");
    let facts = RecoveryFacts::new(
        MethodFacts::new(name, descriptor, 4)
            .with_access_flags(0x0008)
            .with_declaring_class(DeclaringClass::new(owner, 0x0031)),
    );
    let mut budget = recovery_budget.unwrap_or(budget);
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8).with_evidence(evidence),
        &mut budget,
    )
}

#[test]
fn cf07_loop_return_leaf_has_one_owner_and_source() {
    let report = recover_method("lastIndexOf", "([IIII)I", RecoveryEvidenceRequest::all());
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert_eq!(
        report.text.matches("return local4;").count(),
        1,
        "{}",
        report.text
    );
    for bci in [5, 11, 19, 21, 22] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost source: {}",
            report.text
        );
    }
    for bci in [5, 11, 19, 22] {
        let owners: Vec<_> = report
            .regions
            .iter()
            .filter(|region| region.blocks.contains(&bci))
            .collect();
        assert_eq!(owners.len(), 1, "BCI {bci} owners: {:?}", report.regions);
    }
    let leaf = report
        .regions
        .iter()
        .find(|region| region.blocks.contains(&19))
        .expect("return leaf has an owner");
    assert!(
        !leaf.blocks.contains(&28),
        "normal loop exit must remain outside the leaf"
    );
}

#[test]
fn cf07_unproved_terminal_candidates_keep_source_and_quote() {
    // `otherValue` left this list when the terminal-return classification generalized past the
    // `iload; ireturn` pair (`recover-return-in-do-while-false`): its `return array[i]` leaf is
    // an exclusive-predecessor terminal return like any other, so it is proved now and the
    // method recovers — see `cf07_computed_terminal_returns_recover` below. The shapes that
    // stay unproved keep their quote.
    for (class, method, leaf) in [
        (EXTRA_ENTRY, "extraEntry", 26),
        (NEGATIVES, "sharedLeaf", 27),
        (NEGATIVES, "exceptionalLeaf", 19),
    ] {
        let report = recover_class_method_with_budget(
            class,
            "cf07/LoopTerminalNegatives",
            method,
            "([IIII)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "{method}: {:?}", report.outcome);
        assert!(
            report.text.contains("@bytecode"),
            "{method}: {}",
            report.text
        );
        assert!(
            !report.source_map.of_bci(leaf).is_empty(),
            "{method} leaf lost provenance"
        );
        assert!(
            !report.regions.iter().any(|region| region.structured
                && region.blocks.contains(&leaf)
                && region.rule.is_some_and(|rule| rule.rule() == "loop")),
            "{method} falsely assigned the leaf to a loop: {:?}",
            report.regions
        );
    }
}

#[test]
fn cf07_computed_terminal_returns_recover() {
    // `otherValue`'s leaf is `aload_0; iload_4; iaload; ireturn` — a terminal return whose
    // value is computed rather than loaded straight from a local. The CF-07 slice proved the
    // `iload; ireturn` pair alone; `recover-return-in-do-while-false` generalized the edge to
    // any decoded `return` (the do-while(false) family's `return "early"` leaves have the same
    // shape class), so this method recovers with the return in its arm and no quote.
    let report = recover_class_method_with_budget(
        NEGATIVES,
        "cf07/LoopTerminalNegatives",
        "otherValue",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(!report.text.contains("@bytecode"), "{}", report.text);
    assert!(
        report.text.contains("return arg0[local4];"),
        "{}",
        report.text
    );
    for bci in [19u32, 20, 22, 23] {
        assert!(
            !report.source_map.of_bci(bci).is_empty(),
            "BCI {bci} lost source: {}",
            report.text
        );
    }
}

#[test]
fn cf07_nonterminal_break_does_not_become_a_terminal_return() {
    let report = recover_class_method_with_budget(
        NEGATIVES,
        "cf07/LoopTerminalNegatives",
        "nonterminalExit",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        None,
    );
    assert!(report.produced(), "{:?}", report.outcome);
    assert!(report.text.contains("break;"), "{}", report.text);
    assert_eq!(
        report.text.matches("return local4;").count(),
        1,
        "{}",
        report.text
    );
    assert!(!report.source_map.is_empty());
}

#[test]
fn cf07_candidate_budget_and_cancellation_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;
    let full = recover_method_with_budget(
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(limits())),
    );
    let ExecutionReport::Complete { usage } = full.execution else {
        panic!("the full candidate did not complete: {:?}", full.outcome);
    };
    assert!(usage.analysis_steps > 1);
    let mut late = limits();
    late.analysis_steps = usage.analysis_steps - 1;
    let stopped = recover_method_with_budget(
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::new(late)),
    );
    assert!(!stopped.produced());
    assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
    assert!(matches!(
        stopped.stop(),
        Some(StopReason::Budget {
            dimension: CountedBudgetDimension::AnalysisSteps,
            ..
        })
    ));
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_method_with_budget(
        "lastIndexOf",
        "([IIII)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));
}
