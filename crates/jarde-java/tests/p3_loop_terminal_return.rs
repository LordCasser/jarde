//! The frozen CF-07 Java 8 loop return leaf has one owner and a proved source.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceRequest, RecoveryFacts, RecoveryRequest,
    pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Quality};
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
const SWITCH_SN_JAVAC8: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-switch-nonlocal-exits/javac8/SN.class");
const SWITCH_SN_JAVAC23: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-switch-nonlocal-exits/javac23/SN.class");
const SWITCH_SL_JAVAC8: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-switch-nonlocal-exits/javac8/SL.class");
const SWITCH_SL_JAVAC23: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-switch-nonlocal-exits/javac23/SL.class");
const SWITCH_LD_JAVAC8: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-switch-nonlocal-exits/javac8/LD.class");
const SWITCH_LD_JAVAC23: &[u8] =
    include_bytes!("../../../tests/fixtures/p3-switch-nonlocal-exits/javac23/LD.class");

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
    let declaration = analysis
        .ir()
        .declaration()
        .expect("reader states the exact method declaration");
    let declaration_name = String::from_utf8_lossy(&declaration.name().0).into_owned();
    let declaration_descriptor = String::from_utf8_lossy(&declaration.descriptor().0).into_owned();
    let declaration_owner = String::from_utf8_lossy(&declaration.class_name().0).into_owned();
    assert_eq!(declaration_name, name);
    assert_eq!(declaration_descriptor, descriptor);
    assert_eq!(declaration_owner, owner);
    let facts = RecoveryFacts::new(
        MethodFacts::new(
            declaration_name,
            declaration_descriptor,
            declaration.parameter_slots(),
        )
        .with_access_flags(declaration.access_flags())
        .with_declaring_class(DeclaringClass::new(
            declaration_owner,
            declaration.class_access_flags(),
        )),
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
        assert!(!report.text.contains("for ("), "{method}: {}", report.text);
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
fn decoded_switch_case_returns_are_loop_terminal_leaves() {
    for (class, digest) in [
        (
            SWITCH_SN_JAVAC8,
            "157d10b85e894aca95e95390fc5ee819659ee3e209f09300f82caeed83a97aab",
        ),
        (
            SWITCH_SN_JAVAC23,
            "1a989d509d52a95127e2d56f2df471e754358e4c6df43b019ba3f2471629b7ff",
        ),
    ] {
        assert_eq!(blake3::hash(class).to_hex().to_string(), digest);
        for (method, transfer_bci) in [("retInSwitchNoTail", 57), ("retInSwitchTail", 60)] {
            let report = recover_class_method_with_budget(
                class,
                "SN",
                method,
                "([I)I",
                RecoveryEvidenceRequest::all(),
                None,
            );
            assert!(report.produced(), "{method}: {:?}", report.outcome);
            assert!(
                !report.text.contains("@bytecode"),
                "{method}: {}",
                report.text
            );
            assert!(report.text.contains("case 3:"), "{method}: {}", report.text);
            let case = report.text.find("case 3:").expect("case-return arm");
            let default = report.text[case..]
                .find("default:")
                .map(|offset| case + offset)
                .expect("default arm follows the return arm");
            assert!(
                report.text[case..default].contains("return "),
                "the return remains in case 3: {}",
                report.text
            );
            assert!(
                !report.source_map.of_bci(44).is_empty(),
                "{method} return-leaf source is retained"
            );
            assert!(
                !report.source_map.of_bci(transfer_bci).is_empty(),
                "{method} back-edge goto BCI {transfer_bci} must retain its source"
            );
            let owners: Vec<_> = report
                .regions
                .iter()
                .filter(|region| region.structured && region.blocks.contains(&44))
                .collect();
            assert_eq!(
                owners.len(),
                1,
                "{method} leaf ownership: {:?}",
                report.regions
            );
            assert!(owners[0].rule.is_some_and(|rule| rule.rule() == "loop"));
        }

        let if_control = recover_class_method_with_budget(
            class,
            "SN",
            "retAfterTail",
            "([I)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(
            if_control.produced(),
            "if control: {:?}",
            if_control.outcome
        );
        assert!(
            !if_control.text.contains("@bytecode"),
            "{}",
            if_control.text
        );
    }
}

#[test]
fn four_way_switch_return_leaf_has_one_owner() {
    for (class, digest) in [
        (
            SWITCH_SL_JAVAC8,
            "c70528de7c76a1fb68f86c060c74149b85cb7e61589516f32b7044bb166969ff",
        ),
        (
            SWITCH_SL_JAVAC23,
            "e53178ff745dd8a1c2d68a4fa83f28895bfab910aa42677cfd9373f404a3e803",
        ),
    ] {
        assert_eq!(blake3::hash(class).to_hex().to_string(), digest);
        let report = recover_class_method_with_budget(
            class,
            "SL",
            "mixed",
            "([I)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "SL mixed: {:?}", report.outcome);
        assert!(!report.text.contains("@bytecode"), "{}", report.text);
        assert!(report.text.contains("case 3:"), "{}", report.text);
        assert!(
            !report.source_map.of_bci(58).is_empty(),
            "four-successor switch return leaf source is retained"
        );
        let owners: Vec<_> = report
            .regions
            .iter()
            .filter(|region| region.structured && region.blocks.contains(&58))
            .collect();
        assert_eq!(
            owners.len(),
            1,
            "SL return-leaf ownership: {:?}",
            report.regions
        );
        assert!(owners[0].rule.is_some_and(|rule| rule.rule() == "loop"));
    }
}

#[test]
fn switch_outer_breaks_keep_loop_owner_and_transfer_source() {
    for (class, owner, method) in [
        (SWITCH_SN_JAVAC8, "SN", "labBreak"),
        (SWITCH_SN_JAVAC23, "SN", "labBreak"),
        (SWITCH_SL_JAVAC8, "SL", "labeledBreak"),
        (SWITCH_SL_JAVAC23, "SL", "labeledBreak"),
        (SWITCH_LD_JAVAC8, "LD", "lab"),
        (SWITCH_LD_JAVAC23, "LD", "lab"),
    ] {
        let report = recover_class_method_with_budget(
            class,
            owner,
            method,
            "([I)I",
            RecoveryEvidenceRequest::all(),
            None,
        );
        assert!(report.produced(), "{owner}.{method}: {:?}", report.outcome);
        assert_eq!(
            report.quality,
            Quality::Structured,
            "{owner}.{method}: {}",
            report.text
        );
        assert!(
            !report.text.contains("@bytecode"),
            "{owner}.{method}: {}",
            report.text
        );
        let loop_owners: Vec<_> = report
            .regions
            .iter()
            .filter(|region| {
                region.structured && region.rule.is_some_and(|rule| rule.rule() == "loop")
            })
            .collect();
        assert_eq!(
            loop_owners.len(),
            1,
            "{owner}.{method} must have exactly one structured loop owner: {:?}",
            report.regions
        );
        assert!(
            report.text.contains("loop:") && report.text.contains("break loop;"),
            "{owner}.{method} must present the switch exit as a break to its loop: {}",
            report.text
        );
        assert!(
            !report.source_map.of_bci(44).is_empty(),
            "{owner}.{method} lost transfer BCI 44: {}",
            report.text
        );
    }
}

#[test]
fn switch_loop_join_late_budget_cancel_and_deadline_publish_no_partial_source() {
    use jarde_java::StopReason;
    use jarde_reader::model::ExecutionReport;

    for (class, owner, method) in [
        (SWITCH_SN_JAVAC8, "SN", "retInSwitchNoTail"),
        (SWITCH_SL_JAVAC8, "SL", "mixed"),
    ] {
        let full = recover_class_method_with_budget(
            class,
            owner,
            method,
            "([I)I",
            RecoveryEvidenceRequest::all(),
            Some(Budget::new(limits())),
        );
        assert!(full.produced(), "{owner}.{method}: {:?}", full.outcome);
        assert_eq!(
            full.quality,
            Quality::Structured,
            "{owner}.{method}: {}",
            full.text
        );
        let ExecutionReport::Complete { usage } = &full.execution else {
            panic!(
                "{owner}.{method} full recovery did not complete: {:?}",
                full.execution
            );
        };
        assert!(usage.analysis_steps > 0);

        let mut short_limits = limits();
        short_limits.analysis_steps = usage.analysis_steps - 1;
        let stopped = recover_class_method_with_budget(
            class,
            owner,
            method,
            "([I)I",
            RecoveryEvidenceRequest::all(),
            Some(Budget::new(short_limits)),
        );
        assert!(
            !stopped.produced(),
            "{owner}.{method}: {:?}",
            stopped.outcome
        );
        assert!(stopped.text.is_empty() && stopped.source_map.is_empty());
        assert!(matches!(
            stopped.stop(),
            Some(StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                ..
            })
        ));
    }

    let token = CancellationToken::new();
    token.cancel();
    let cancelled = recover_class_method_with_budget(
        SWITCH_SN_JAVAC8,
        "SN",
        "retInSwitchNoTail",
        "([I)I",
        RecoveryEvidenceRequest::all(),
        Some(Budget::with_cancellation_token(limits(), token)),
    );
    assert!(!cancelled.produced());
    assert!(cancelled.text.is_empty() && cancelled.source_map.is_empty());
    assert!(cancelled.stop().is_some_and(StopReason::is_cancelled));

    let mut deadline_limits = limits();
    deadline_limits.elapsed_millis = 1;
    let deadline = Budget::new(deadline_limits);
    std::thread::sleep(std::time::Duration::from_millis(2));
    let expired = recover_class_method_with_budget(
        SWITCH_SN_JAVAC8,
        "SN",
        "retInSwitchNoTail",
        "([I)I",
        RecoveryEvidenceRequest::all(),
        Some(deadline),
    );
    assert!(!expired.produced());
    assert!(expired.text.is_empty() && expired.source_map.is_empty());
    assert!(matches!(
        expired.stop(),
        Some(StopReason::Interrupted { code, .. }) if *code == "jre_budget_interrupted"
    ));
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
