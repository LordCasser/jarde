//! Public API regression for an outer loop containing a prefixed exceptional retry loop.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryFacts,
    RecoveryReport, RecoveryRequest, StopReason, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, Quality};
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

const DEBUG_CLASS: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-prefixed-exceptional-retry-loops/original/debug/jadx/tests/integration/loops/TestNestedLoops3$TestCls.class"
);
const NO_DEBUG_CLASS: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-prefixed-exceptional-retry-loops/original/no-debug/jadx/tests/integration/loops/TestNestedLoops3$TestCls.class"
);
const OWNER: &str = "jadx/tests/integration/loops/TestNestedLoops3$TestCls";
const METHOD: &str = "test";
const DESCRIPTOR: &str = "(I)I";
const PHYSICAL_BCIS: [u32; 21] = [
    0, 1, 4, 5, 6, 7, 10, 11, 14, 17, 20, 21, 23, 26, 29, 30, 33, 36, 37, 40, 41,
];

const ALPHA_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-prefixed-exceptional-retry-loops/supplemental/javac8/debug/PrefixRetryAlpha.class"
);
const ALPHA_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-prefixed-exceptional-retry-loops/supplemental/javac8/no-debug/PrefixRetryAlpha.class"
);
const ALPHA_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-prefixed-exceptional-retry-loops/supplemental/javac23/debug/PrefixRetryAlpha.class"
);
const ALPHA_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-prefixed-exceptional-retry-loops/supplemental/javac23/no-debug/PrefixRetryAlpha.class"
);
const BETA_JAVAC8_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-prefixed-exceptional-retry-loops/supplemental/javac8/debug/RetryPrefixBeta.class"
);
const BETA_JAVAC8_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-prefixed-exceptional-retry-loops/supplemental/javac8/no-debug/RetryPrefixBeta.class"
);
const BETA_JAVAC23_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-prefixed-exceptional-retry-loops/supplemental/javac23/debug/RetryPrefixBeta.class"
);
const BETA_JAVAC23_NO_DEBUG: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-prefixed-exceptional-retry-loops/supplemental/javac23/no-debug/RetryPrefixBeta.class"
);
const SUPPLEMENTAL_PHYSICAL_BCIS: [u32; 57] = [
    0, 1, 2, 3, 4, 7, 10, 11, 12, 13, 16, 18, 19, 20, 23, 24, 25, 28, 29, 30, 33, 34, 36, 37, 38,
    41, 43, 46, 49, 52, 53, 56, 58, 61, 64, 65, 66, 69, 72, 74, 75, 76, 79, 80, 81, 84, 85, 87, 88,
    91, 94, 97, 98, 99, 102, 103, 106,
];

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

fn analyze(class: &[u8], analysis_steps: u64) -> (Analyzed, Budget) {
    let (analyzed, budget) = analyze_target(class, OWNER, METHOD, DESCRIPTOR, 2, analysis_steps);
    assert_eq!(
        analyzed.instruction_bcis.as_slice(),
        PHYSICAL_BCIS.as_slice()
    );
    (analyzed, budget)
}

fn analyze_target(
    class: &[u8],
    owner: &str,
    method_name: &str,
    descriptor: &str,
    parameter_slots: u16,
    analysis_steps: u64,
) -> (Analyzed, Budget) {
    let mut budget = Budget::new(limits(analysis_steps));
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("frozen loop fixture opens");
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
        name: JvmBytes(method_name.as_bytes().to_vec()),
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
    .expect("physical target method analyzes");
    let declaration = analysis
        .ir()
        .declaration()
        .expect("same physical read supplies the target declaration");
    assert_eq!(String::from_utf8_lossy(&declaration.class_name().0), owner);
    assert_eq!(declaration.name().0.as_slice(), method_name.as_bytes());
    assert_eq!(declaration.descriptor().0.as_slice(), descriptor.as_bytes());
    assert_eq!(declaration.parameter_slots(), parameter_slots);
    assert!(
        !declaration.is_static(),
        "the receiver occupies local slot 0"
    );
    assert_eq!(declaration.identity(), &method, "same physical method body");
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
    let canonical = analysis.ir().canonical().expect("canonical CFG exists");
    assert!(
        canonical.completeness().is_complete(),
        "canonical CFG is complete"
    );
    let mut instruction_bcis = analysis
        .ir()
        .ssa()
        .expect("SSA covers the whole target method")
        .blocks()
        .iter()
        .flat_map(|block| {
            block
                .instructions()
                .iter()
                .map(|instruction| instruction.bci())
        })
        .collect::<Vec<_>>();
    instruction_bcis.sort_unstable();
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

fn recovered_default(analyzed: &Analyzed, budget: &mut Budget) -> RecoveryReport {
    recover(
        &RecoveryRequest::new(analyzed.analysis.ir(), &analyzed.facts, JAVA_8),
        budget,
    )
}

fn recovered(
    analyzed: &Analyzed,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> RecoveryReport {
    recover(
        &RecoveryRequest::new(analyzed.analysis.ir(), &analyzed.facts, JAVA_8)
            .with_evidence(evidence.clone()),
        budget,
    )
}

fn assert_complete(label: &str, report: &RecoveryReport) {
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
}

fn assert_origin_is_target_method(
    label: &str,
    origin: &jarde_java::source_map::Origin,
    method: &PhysicalMethodId,
    instruction_bcis: &[u32],
) {
    assert_eq!(origin.method(), Some(method), "{label}: origin identity");
    assert!(
        instruction_bcis.contains(&origin.bci()),
        "{label}: BCI {} is outside the physical method instructions",
        origin.bci()
    );
}

fn assert_every_physical_bci_has_local_utf8_source(analyzed: &Analyzed, report: &RecoveryReport) {
    assert_source_map_covers_physical_bcis(analyzed, report, &[14, 26, 33, 37]);
}

fn assert_source_map_covers_physical_bcis(
    analyzed: &Analyzed,
    report: &RecoveryReport,
    required_bcis: &[u32],
) {
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
        let _source = segment.text(&report.text);
        let origin = segment.origin();
        assert_origin_is_target_method(
            "source-map primary",
            origin.primary(),
            &analyzed.method,
            &analyzed.instruction_bcis,
        );
        for derived in origin.derived() {
            assert_origin_is_target_method(
                "source-map derived",
                derived,
                &analyzed.method,
                &analyzed.instruction_bcis,
            );
        }
    }
    for bci in &analyzed.instruction_bcis {
        assert!(
            !report.source_map.of_bci(*bci).is_empty(),
            "physical instruction BCI {bci} has no source-map segment\n{}",
            report.text
        );
    }
    for bci in required_bcis {
        assert!(
            !report.source_map.of_bci(*bci).is_empty(),
            "required physical BCI {bci} has no source origin\n{}",
            report.text
        );
    }
}

#[test]
fn prefixed_exceptional_retry_loop_is_structured_for_debug_and_no_debug_inputs() {
    for (profile, class) in [("debug", DEBUG_CLASS), ("no-debug", NO_DEBUG_CLASS)] {
        let (analyzed, _analysis_budget) = analyze(class, 1 << 20);
        let mut default_budget = Budget::new(limits(1 << 20));
        let default = recovered_default(&analyzed, &mut default_budget);
        let mut essential_budget = Budget::new(limits(1 << 20));
        let essential = recovered(
            &analyzed,
            &RecoveryEvidenceRequest::essential(),
            &mut essential_budget,
        );
        let mut all_budget = Budget::new(limits(1 << 20));
        let all = recovered(&analyzed, &RecoveryEvidenceRequest::all(), &mut all_budget);
        let mut source_map_budget = Budget::new(limits(1 << 20));
        let source_map_only = recovered(
            &analyzed,
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            &mut source_map_budget,
        );
        assert_complete(&format!("{profile}/default"), &default);
        assert_complete(&format!("{profile}/essential"), &essential);
        assert_complete(&format!("{profile}/all"), &all);
        assert_complete(&format!("{profile}/map-only"), &source_map_only);
        assert_eq!(
            default.text, essential.text,
            "{profile}: default/essential text"
        );
        assert_eq!(default.text, all.text, "{profile}: default/all text");
        assert_eq!(
            default.text, source_map_only.text,
            "{profile}: default/map text"
        );
        assert_eq!(
            all.source_map, source_map_only.source_map,
            "{profile}: map parity"
        );
        assert!(!all.source_map.is_empty(), "{profile}: all evidence map");
        assert!(!all.text.contains("@bytecode"), "{profile}: {}", all.text);
        assert_every_physical_bci_has_local_utf8_source(&analyzed, &all);
    }
}

#[test]
fn prefixed_exceptional_retry_late_recovery_budget_publishes_no_source_or_map() {
    for (profile, class) in [("debug", DEBUG_CLASS), ("no-debug", NO_DEBUG_CLASS)] {
        let (complete, mut complete_budget) = analyze(class, 1 << 20);
        let analysis_steps = complete_budget.usage().analysis_steps;
        let report = recovered(
            &complete,
            &RecoveryEvidenceRequest::all(),
            &mut complete_budget,
        );
        assert_complete(&format!("{profile}/complete"), &report);
        let total_steps = complete_budget.usage().analysis_steps;
        assert!(
            total_steps > analysis_steps + 1,
            "{profile}: no recovery work"
        );
        assert!(
            total_steps - analysis_steps > 1,
            "{profile}: not a late stop"
        );

        let (limited, mut limited_budget) = analyze(class, total_steps - 1);
        assert_eq!(limited_budget.usage().analysis_steps, analysis_steps);
        let stopped = recovered(
            &limited,
            &RecoveryEvidenceRequest::all(),
            &mut limited_budget,
        );
        assert!(
            matches!(
                stopped.stop(),
                Some(StopReason::Budget {
                    dimension: jarde_reader::budget::CountedBudgetDimension::AnalysisSteps,
                    ..
                })
            ),
            "{profile}: expected late recovery budget stop, got {:?}",
            stopped.outcome
        );
        assert!(
            !stopped.produced(),
            "{profile}: partial artifact was published"
        );
        assert!(
            stopped.text.is_empty(),
            "{profile}: partial source was published"
        );
        assert!(
            stopped.source_map.is_empty(),
            "{profile}: partial source map was published"
        );
    }
}

#[test]
fn supplemental_pure_latch_bridges_keep_every_physical_source_origin() {
    for (compiler, debug_profile, owner, class) in [
        ("javac8", "debug", "PrefixRetryAlpha", ALPHA_JAVAC8_DEBUG),
        (
            "javac8",
            "no-debug",
            "PrefixRetryAlpha",
            ALPHA_JAVAC8_NO_DEBUG,
        ),
        ("javac23", "debug", "PrefixRetryAlpha", ALPHA_JAVAC23_DEBUG),
        (
            "javac23",
            "no-debug",
            "PrefixRetryAlpha",
            ALPHA_JAVAC23_NO_DEBUG,
        ),
        ("javac8", "debug", "RetryPrefixBeta", BETA_JAVAC8_DEBUG),
        (
            "javac8",
            "no-debug",
            "RetryPrefixBeta",
            BETA_JAVAC8_NO_DEBUG,
        ),
        ("javac23", "debug", "RetryPrefixBeta", BETA_JAVAC23_DEBUG),
        (
            "javac23",
            "no-debug",
            "RetryPrefixBeta",
            BETA_JAVAC23_NO_DEBUG,
        ),
    ] {
        let label = format!("{owner}/{compiler}/{debug_profile}");
        let (analyzed, _analysis_budget) =
            analyze_target(class, owner, "execute", "(III)I", 4, 1 << 20);
        assert_eq!(
            analyzed.instruction_bcis.as_slice(),
            SUPPLEMENTAL_PHYSICAL_BCIS.as_slice(),
            "{label}: receipt-pinned physical instruction BCIs"
        );
        for bci in [46, 61, 69, 91, 94] {
            assert!(
                analyzed.instruction_bcis.contains(&bci),
                "{label}: required physical BCI {bci} is absent"
            );
        }

        let mut default_budget = Budget::new(limits(1 << 20));
        let default = recovered_default(&analyzed, &mut default_budget);
        let mut all_budget = Budget::new(limits(1 << 20));
        let all = recovered(&analyzed, &RecoveryEvidenceRequest::all(), &mut all_budget);
        let mut map_budget = Budget::new(limits(1 << 20));
        let map_only = recovered(
            &analyzed,
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            &mut map_budget,
        );
        assert_complete(&format!("{label}/default"), &default);
        assert_complete(&format!("{label}/all"), &all);
        assert_complete(&format!("{label}/map-only"), &map_only);
        assert_eq!(default.text, all.text, "{label}: default/all source parity");
        assert_eq!(
            all.text, map_only.text,
            "{label}: all/map-only source parity"
        );
        assert_eq!(
            all.source_map, map_only.source_map,
            "{label}: source-map parity"
        );
        assert!(
            !all.source_map.is_empty(),
            "{label}: all-evidence map is empty"
        );
        assert_source_map_covers_physical_bcis(&analyzed, &all, &[46, 61, 69, 91, 94]);
    }
}
