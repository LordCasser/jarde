//! Public API regression tests for the bounded nested saved-return finally.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryFacts,
    RecoveryRequest, StopReason, pass::JAVA_8, recover,
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

const JAVAC_23: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-return-through-nested-finally-cleanup/v8/NestedFinally.class"
);
const JAVAC_8: &[u8] = include_bytes!(
    "../../../tests/fixtures/recover-return-through-nested-finally-cleanup/v8-javac8/NestedFinally.class"
);
const SEALED_EX: &[u8] = include_bytes!(
    "../../../openspec/evidence/java-syntax-2026-10-05/exception-shape-family-patrol/fixture/EX.class"
);

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
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

struct Analyzed {
    snapshot: ArtifactSnapshot,
    analysis: jarde_jvm::method_ir::MethodIrAnalysis,
    facts: RecoveryFacts,
    instruction_bcis: Vec<u32>,
}

fn analyze(class: &[u8], owner: &str, step_limit: u64) -> (Analyzed, Budget) {
    let mut budget = Budget::new(limits(step_limit));
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("nested-finally fixture opens");
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
        name: JvmBytes(b"nestedFin".to_vec()),
        descriptor: JvmBytes(b"(I)I".to_vec()),
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
    .expect("physical nestedFin method analyzes");
    let canonical = analysis.ir().canonical().expect("canonical CFG exists");
    assert!(
        canonical.completeness().is_complete(),
        "canonical CFG is complete"
    );
    let instruction_bcis = analysis
        .ir()
        .ssa()
        .expect("SSA is published for the complete method")
        .blocks()
        .iter()
        .flat_map(|block| {
            block
                .instructions()
                .iter()
                .map(|instruction| instruction.bci())
        })
        .collect();
    let facts = RecoveryFacts::new(
        MethodFacts::new("nestedFin", "(I)I", 1)
            .with_access_flags(0x0008)
            .with_declaring_class(DeclaringClass::new(owner, 0x0021)),
    );
    (
        Analyzed {
            snapshot,
            analysis,
            facts,
            instruction_bcis,
        },
        budget,
    )
}

fn recovered_default(analyzed: &Analyzed, budget: &mut Budget) -> jarde_java::RecoveryReport {
    recover(
        &RecoveryRequest::new(analyzed.analysis.ir(), &analyzed.facts, JAVA_8),
        budget,
    )
}

fn recovered(
    analyzed: &Analyzed,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> jarde_java::RecoveryReport {
    recover(
        &RecoveryRequest::new(analyzed.analysis.ir(), &analyzed.facts, JAVA_8)
            .with_evidence(evidence.clone()),
        budget,
    )
}

fn assert_complete_report(label: &str, report: &jarde_java::RecoveryReport) {
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

#[test]
fn nested_saved_return_is_complete_for_both_compilers_and_sealed_ex() {
    for (compiler, class, owner) in [
        ("javac 23", JAVAC_23, "NestedFinally"),
        ("javac 8", JAVAC_8, "NestedFinally"),
        ("sealed EX", SEALED_EX, "EX"),
    ] {
        let (analyzed, mut budget) = analyze(class, owner, 1 << 20);
        let _snapshot = &analyzed.snapshot;
        let default = recovered_default(&analyzed, &mut budget);
        let essential = recovered(
            &analyzed,
            &RecoveryEvidenceRequest::essential(),
            &mut budget,
        );
        let all = recovered(&analyzed, &RecoveryEvidenceRequest::all(), &mut budget);
        let source_map_only = recovered(
            &analyzed,
            &RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
            &mut budget,
        );
        assert_complete_report(&format!("{compiler}/default"), &default);
        assert_complete_report(&format!("{compiler}/essential"), &essential);
        assert_complete_report(&format!("{compiler}/all"), &all);
        assert_eq!(
            default.text, all.text,
            "{compiler}: default evidence changed source"
        );
        assert_eq!(
            default.text, essential.text,
            "{compiler}: explicit essential differs from default"
        );
        assert_complete_report(&format!("{compiler}/source-map-only"), &source_map_only);
        assert_eq!(
            essential.text, all.text,
            "{compiler}: evidence changed source"
        );
        assert_eq!(
            all.text, source_map_only.text,
            "{compiler}: source-map request changed source"
        );
        assert_eq!(
            all.source_map, source_map_only.source_map,
            "{compiler}: map request parity"
        );
        assert!(!all.text.contains("@bytecode"), "{compiler}: {}", all.text);
        for required in ["try", "finally", "System.out.println", "in", "out", "10"] {
            assert!(
                all.text.contains(required),
                "{compiler}: missing {required:?}\n{}",
                all.text
            );
        }
        assert!(
            !all.source_map.is_empty(),
            "{compiler}: source map is empty"
        );
        for bci in &analyzed.instruction_bcis {
            assert!(
                !all.source_map.of_bci(*bci).is_empty(),
                "{compiler}: physical instruction BCI {bci} is unmapped; source={:?}",
                all.text_of_bci(*bci)
            );
        }
        if compiler == "sealed EX" {
            for bci in [3, 4, 10, 18, 30, 34, 36, 43, 51, 64, 69, 71] {
                assert!(
                    !all.source_map.of_bci(bci).is_empty(),
                    "{compiler}: required arithmetic/copy/return BCI {bci} has no provenance"
                );
            }
        }
    }
}

#[test]
fn nested_saved_return_late_budget_publishes_no_partial_source_or_map() {
    for (compiler, class, owner) in [
        ("javac 23", JAVAC_23, "NestedFinally"),
        ("javac 8", JAVAC_8, "NestedFinally"),
        ("sealed EX", SEALED_EX, "EX"),
    ] {
        let (complete, mut complete_budget) = analyze(class, owner, 1 << 20);
        let analyzed_steps = complete_budget.usage().analysis_steps;
        let complete_report = recovered(
            &complete,
            &RecoveryEvidenceRequest::all(),
            &mut complete_budget,
        );
        assert_complete_report(&format!("{compiler}/complete"), &complete_report);
        let used_steps = complete_budget.usage().analysis_steps;
        assert!(
            used_steps > analyzed_steps + 1,
            "{compiler}: no recovery work to limit"
        );
        let recovery_steps = used_steps - analyzed_steps;
        assert!(
            recovery_steps > 1,
            "{compiler}: budget would stop before late recovery work"
        );

        let (limited, mut limited_budget) = analyze(class, owner, used_steps - 1);
        assert_eq!(limited_budget.usage().analysis_steps, analyzed_steps);
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
            "{compiler}: expected late recovery budget stop, got {:?}",
            stopped.outcome
        );
        assert!(
            !stopped.produced(),
            "{compiler}: partial result was published"
        );
        assert!(
            stopped.text.is_empty(),
            "{compiler}: partial source was published"
        );
        assert!(
            stopped.source_map.is_empty(),
            "{compiler}: partial map was published"
        );
    }
}
