//! Public API regressions for terminal returns owned by a conditional loop arm.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryFacts,
    RecoveryReport, RecoveryRequest, StopReason, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Quality};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, CountedBudgetDimension, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, ExecutionReport, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
    PhysicalMethodId, PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const ORIGINAL_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-proved-loop-arm-returns/original/debug/classes/jadx/tests/integration/loops/TestNestedLoops5$TestCls.class"
);
const ORIGINAL_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-proved-loop-arm-returns/original/no-debug/classes/jadx/tests/integration/loops/TestNestedLoops5$TestCls.class"
);

const A_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/A/javac8/debug/classes/NestedReturnA$TestCls.class"
);
const A_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/A/javac8/no-debug/classes/NestedReturnA$TestCls.class"
);
const A_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/A/javac23/debug/classes/NestedReturnA$TestCls.class"
);
const A_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/A/javac23/no-debug/classes/NestedReturnA$TestCls.class"
);
const B_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/B/javac8/debug/classes/NestedReturnB$TestCls.class"
);
const B_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/B/javac8/no-debug/classes/NestedReturnB$TestCls.class"
);
const B_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/B/javac23/debug/classes/NestedReturnB$TestCls.class"
);
const B_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-proved-loop-arm-returns/supplemental/B/javac23/no-debug/classes/NestedReturnB$TestCls.class"
);

const ORIGINAL_OWNER: &str = "jadx/tests/integration/loops/TestNestedLoops5$TestCls";

fn limits(analysis_steps: u64) -> Limits {
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
        analysis_steps,
        normalization_clones: 1 << 20,
        nested_depth: 32,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

struct Analyzed {
    analysis: jarde_jvm::method_ir::MethodIrAnalysis,
    facts: RecoveryFacts,
    method: PhysicalMethodId,
    instruction_bcis: Vec<u32>,
}

fn analyze_target(
    class: &[u8],
    expected_owner: &str,
    name: &str,
    descriptor: &str,
    parameter_slots: u16,
    analysis_steps: u64,
) -> (Analyzed, Budget) {
    let mut budget = Budget::new(limits(analysis_steps));
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("frozen target class opens");
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
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(class).to_hex().to_string()),
                length: u64::try_from(class.len()).expect("fixture length fits"),
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
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
            method: method.clone(),
            stages: AnalysisStage::ALL.to_vec(),
        },
        &mut budget,
    )
    .expect("same standalone class read analyzes the selected method");

    let declaration = analysis
        .ir()
        .declaration()
        .expect("same read supplies the selected declaration");
    assert_eq!(
        String::from_utf8_lossy(&declaration.class_name().0),
        expected_owner
    );
    assert_eq!(declaration.name().0.as_slice(), name.as_bytes());
    assert_eq!(declaration.descriptor().0.as_slice(), descriptor.as_bytes());
    assert_eq!(declaration.parameter_slots(), parameter_slots);
    assert!(
        !declaration.is_static(),
        "the receiver occupies local slot 0"
    );
    assert_eq!(
        declaration.identity(),
        &method,
        "body and declaration share identity"
    );
    let facts = RecoveryFacts::new(
        MethodFacts::new(
            String::from_utf8_lossy(&declaration.name().0).into_owned(),
            String::from_utf8_lossy(&declaration.descriptor().0).into_owned(),
            declaration.parameter_slots(),
        )
        .with_access_flags(declaration.access_flags())
        .with_declaring_class(DeclaringClass::new(
            String::from_utf8_lossy(&declaration.class_name().0).into_owned(),
            declaration.class_access_flags(),
        )),
    );

    let canonical = analysis.ir().canonical().expect("canonical graph exists");
    assert!(
        canonical.completeness().is_complete(),
        "canonical graph is complete"
    );
    let code = analysis
        .ir()
        .code()
        .expect("same read supplies decoded Code facts");
    assert!(
        code.stopped_at.is_none(),
        "the complete physical body decoded"
    );
    assert!(
        matches!(&code.execution, ExecutionReport::Complete { .. }),
        "decoded Code reports complete execution"
    );
    let instruction_bcis = code
        .instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect::<Vec<_>>();
    assert!(
        !instruction_bcis.is_empty(),
        "method has physical instructions"
    );
    assert!(
        instruction_bcis.windows(2).all(|pair| pair[0] < pair[1]),
        "decoded BCI order is strict"
    );
    let mut ssa_bcis = analysis
        .ir()
        .ssa()
        .expect("SSA covers the selected body")
        .blocks()
        .iter()
        .flat_map(|block| {
            block
                .instructions()
                .iter()
                .map(|instruction| instruction.bci())
        })
        .collect::<Vec<_>>();
    ssa_bcis.sort_unstable();
    assert_eq!(
        ssa_bcis, instruction_bcis,
        "SSA preserves every decoded physical instruction once"
    );

    (
        Analyzed {
            analysis,
            facts,
            method,
            instruction_bcis,
        },
        budget,
    )
}

// CLI default class-source requests SourceMap; library essential alone materializes no map.
fn recover_default(analyzed: &Analyzed, budget: &mut Budget) -> RecoveryReport {
    recover(
        &RecoveryRequest::new(analyzed.analysis.ir(), &analyzed.facts, JAVA_8).with_evidence(
            RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        ),
        budget,
    )
}

fn recover_all(analyzed: &Analyzed, budget: &mut Budget) -> RecoveryReport {
    recover(
        &RecoveryRequest::new(analyzed.analysis.ir(), &analyzed.facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        budget,
    )
}

fn assert_structured(label: &str, report: &RecoveryReport) {
    assert!(
        report.produced(),
        "{label}: {:?}\n{}",
        report.outcome,
        report.text
    );
    assert_eq!(
        report.quality,
        Quality::Structured,
        "{label}: {}",
        report.text
    );
    assert!(report.fallbacks.is_empty(), "{label}: {}", report.text);
    assert!(
        !report.text.contains("@bytecode"),
        "{label}: {}",
        report.text
    );
}

fn assert_origin_is_same_method(
    label: &str,
    origin: &jarde_java::source_map::Origin,
    method: &PhysicalMethodId,
    instruction_bcis: &[u32],
) {
    assert_eq!(
        origin.method(),
        Some(method),
        "{label}: physical method identity"
    );
    assert!(
        instruction_bcis.contains(&origin.bci()),
        "{label}: BCI {} is outside this method's decoded Code facts",
        origin.bci()
    );
}

fn assert_full_method_source_map(analyzed: &Analyzed, report: &RecoveryReport) {
    for segment in report.source_map.segments() {
        let start = segment.start();
        let end = segment.end();
        assert!(
            start <= end && end <= report.text.len(),
            "invalid source-map span {start}..{end}"
        );
        assert!(
            report.text.is_char_boundary(start),
            "source-map start is not a UTF-8 boundary"
        );
        assert!(
            report.text.is_char_boundary(end),
            "source-map end is not a UTF-8 boundary"
        );
        let _slice = segment.text(&report.text);
        let origin = segment.origin();
        assert_origin_is_same_method(
            "primary",
            origin.primary(),
            &analyzed.method,
            &analyzed.instruction_bcis,
        );
        for derived in origin.derived() {
            assert_origin_is_same_method(
                "derived",
                derived,
                &analyzed.method,
                &analyzed.instruction_bcis,
            );
        }
    }
    for bci in &analyzed.instruction_bcis {
        assert!(
            !report.source_map.of_bci(*bci).is_empty(),
            "physical BCI {bci} has no source-map segment\n{}",
            report.text
        );
    }
}

fn assert_default_all_parity(label: &str, default: &RecoveryReport, all: &RecoveryReport) {
    assert_structured(&format!("{label}/default"), default);
    assert_structured(&format!("{label}/all"), all);
    assert_eq!(default.text, all.text, "{label}: default/all text parity");
    assert_eq!(
        default.source_map, all.source_map,
        "{label}: default/all source-map parity"
    );
}

fn assert_late_budget_does_not_publish(
    class: &[u8],
    owner: &str,
    method: &str,
    descriptor: &str,
    slots: u16,
) {
    let (complete, mut complete_budget) =
        analyze_target(class, owner, method, descriptor, slots, 1 << 20);
    let analysis_steps = complete_budget.usage().analysis_steps;
    let complete_report = recover_all(&complete, &mut complete_budget);
    assert_structured("late-budget complete reference", &complete_report);
    let total_steps = complete_budget.usage().analysis_steps;
    assert!(
        total_steps > analysis_steps,
        "late-budget reference did not spend recovery steps"
    );

    let (limited, mut limited_budget) =
        analyze_target(class, owner, method, descriptor, slots, total_steps - 1);
    assert_eq!(
        limited_budget.usage().analysis_steps,
        analysis_steps,
        "analysis fits before recovery limit"
    );
    let stopped = recover_all(&limited, &mut limited_budget);
    assert!(
        matches!(
            stopped.stop(),
            Some(StopReason::Budget {
                dimension: CountedBudgetDimension::AnalysisSteps,
                ..
            })
        ),
        "expected a late request budget stop, got {:?}",
        stopped.outcome
    );
    assert!(
        !stopped.produced(),
        "a stopped request must not publish a partial report"
    );
    assert!(
        stopped.text.is_empty(),
        "a stopped request must not publish source"
    );
    assert!(
        stopped.source_map.is_empty(),
        "a stopped request must not publish source-map entries"
    );
}

#[test]
fn original_nl5_loop_arm_returns_recover_all_three_physical_methods() {
    for (profile, class) in [("debug", ORIGINAL_DEBUG), ("no-debug", ORIGINAL_NO_DEBUG)] {
        let (test_for, _) = analyze_target(class, ORIGINAL_OWNER, "testFor", "()I", 1, 1 << 20);
        let mut default_budget = Budget::new(limits(1 << 20));
        let default = recover_default(&test_for, &mut default_budget);
        let mut all_budget = Budget::new(limits(1 << 20));
        let all = recover_all(&test_for, &mut all_budget);
        assert_default_all_parity(&format!("original/{profile}/testFor"), &default, &all);
        assert_full_method_source_map(&test_for, &all);
        let early_return = all
            .text
            .find("return 0;")
            .expect("the inner method-completing return remains");
        let tail_return = all
            .text
            .rfind("return ")
            .expect("normal method completion remains");
        assert!(
            early_return < tail_return,
            "early return precedes the normal tail return: {}",
            all.text
        );
        assert!(
            !all.text.contains("continue;"),
            "the original method has no invented continue: {}",
            all.text
        );

        let (constructor, _) = analyze_target(class, ORIGINAL_OWNER, "<init>", "()V", 1, 1 << 20);
        let mut constructor_default_budget = Budget::new(limits(1 << 20));
        let constructor_default = recover_default(&constructor, &mut constructor_default_budget);
        let mut constructor_all_budget = Budget::new(limits(1 << 20));
        let constructor_all = recover_all(&constructor, &mut constructor_all_budget);
        assert_default_all_parity(
            &format!("original/{profile}/constructor"),
            &constructor_default,
            &constructor_all,
        );
        assert_full_method_source_map(&constructor, &constructor_all);
        assert!(
            constructor_all.text.contains("super();"),
            "constructor projection is retained: {}",
            constructor_all.text
        );
        assert!(
            constructor_all.text.contains("return;"),
            "constructor completion is retained: {}",
            constructor_all.text
        );

        let (check, _) = analyze_target(class, ORIGINAL_OWNER, "check", "()V", 1, 1 << 20);
        let mut check_default_budget = Budget::new(limits(1 << 20));
        let check_default = recover_default(&check, &mut check_default_budget);
        let mut check_all_budget = Budget::new(limits(1 << 20));
        let check_all = recover_all(&check, &mut check_all_budget);
        assert_default_all_parity(
            &format!("original/{profile}/check"),
            &check_default,
            &check_all,
        );
        assert_full_method_source_map(&check, &check_all);
        assert!(
            check_all
                .text
                .contains("JadxAssertions.assertThat(this.testFor()).isEqualTo(7);"),
            "the non-target check method projection is preserved: {}",
            check_all.text
        );
    }
}

#[test]
fn parameterized_a_and_b_loop_arm_returns_cover_all_javac_and_debug_inputs() {
    for (variant, compiler, profile, owner, class) in [
        (
            "A",
            "javac8",
            "debug",
            "NestedReturnA$TestCls",
            A_JAVAC8_DEBUG,
        ),
        (
            "A",
            "javac8",
            "no-debug",
            "NestedReturnA$TestCls",
            A_JAVAC8_NO_DEBUG,
        ),
        (
            "A",
            "javac23",
            "debug",
            "NestedReturnA$TestCls",
            A_JAVAC23_DEBUG,
        ),
        (
            "A",
            "javac23",
            "no-debug",
            "NestedReturnA$TestCls",
            A_JAVAC23_NO_DEBUG,
        ),
        (
            "B",
            "javac8",
            "debug",
            "NestedReturnB$TestCls",
            B_JAVAC8_DEBUG,
        ),
        (
            "B",
            "javac8",
            "no-debug",
            "NestedReturnB$TestCls",
            B_JAVAC8_NO_DEBUG,
        ),
        (
            "B",
            "javac23",
            "debug",
            "NestedReturnB$TestCls",
            B_JAVAC23_DEBUG,
        ),
        (
            "B",
            "javac23",
            "no-debug",
            "NestedReturnB$TestCls",
            B_JAVAC23_NO_DEBUG,
        ),
    ] {
        let label = format!("{variant}/{compiler}/{profile}");
        let (run, _) = analyze_target(class, owner, "run", "(IIIII)I", 6, 1 << 20);
        let mut default_budget = Budget::new(limits(1 << 20));
        let default = recover_default(&run, &mut default_budget);
        let mut all_budget = Budget::new(limits(1 << 20));
        let all = recover_all(&run, &mut all_budget);
        assert_default_all_parity(&label, &default, &all);
        assert_full_method_source_map(&run, &all);
        let early_return = all
            .text
            .find("return 0;")
            .expect("early return is represented");
        let tail_effect = all
            .text
            .find("tailVisits")
            .expect("normal tail effects remain represented");
        let final_return = all
            .text
            .rfind("return ")
            .expect("normal completion return remains");
        assert!(
            early_return < tail_effect && tail_effect < final_return,
            "early return skips the normal tail, which precedes method completion: {}",
            all.text
        );
        assert!(
            all.text.matches("return ").count() >= 2,
            "both early and normal returns remain: {}",
            all.text
        );

        let (constructor, _) = analyze_target(class, owner, "<init>", "()V", 1, 1 << 20);
        let mut constructor_default_budget = Budget::new(limits(1 << 20));
        let constructor_default = recover_default(&constructor, &mut constructor_default_budget);
        let mut constructor_all_budget = Budget::new(limits(1 << 20));
        let constructor_all = recover_all(&constructor, &mut constructor_all_budget);
        assert_default_all_parity(
            &format!("{label}/constructor non-target"),
            &constructor_default,
            &constructor_all,
        );
        assert_full_method_source_map(&constructor, &constructor_all);
        assert!(
            constructor_all.text.contains("return;"),
            "constructor projection is preserved: {}",
            constructor_all.text
        );
    }
}

#[test]
fn late_loop_arm_recovery_budget_stops_publish_no_partial_source_or_map() {
    assert_late_budget_does_not_publish(ORIGINAL_DEBUG, ORIGINAL_OWNER, "testFor", "()I", 1);
    assert_late_budget_does_not_publish(
        A_JAVAC8_DEBUG,
        "NestedReturnA$TestCls",
        "run",
        "(IIIII)I",
        6,
    );
    assert_late_budget_does_not_publish(
        B_JAVAC23_NO_DEBUG,
        "NestedReturnB$TestCls",
        "run",
        "(IIIII)I",
        6,
    );
}
