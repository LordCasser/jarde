//! Verifier-valid negative controls for shared-producer OR chains.

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
use std::collections::BTreeSet;

const FIXTURE: &[u8] = include_bytes!(
    "../../../tests/fixtures/p3-conditional-values/short-circuit-chain-shared-true-controls/ChainOrFieldDuplicatePhi.class"
);

fn refused_chain_report() -> jarde_java::RecoveryReport {
    let mut budget = Budget::new(Limits {
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
        nested_depth: 32,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    });
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(FIXTURE.to_vec()), &mut budget)
        .expect("the verifier-valid control opens");
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
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(FIXTURE).to_hex().to_string()),
                length: FIXTURE.len() as u64,
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(b"assign".to_vec()),
        descriptor: JvmBytes(b"(ZZ)V".to_vec()),
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
    .expect("fresh class analysis completes");
    let facts = RecoveryFacts::new(
        MethodFacts::new("assign", "(ZZ)V", 1)
            .with_access_flags(0x0008)
            .with_declaring_class(DeclaringClass::new("ChainOrFieldDuplicatePhi", 0x0021)),
    );
    recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    )
}

#[test]
fn duplicate_consumer_in_three_test_shared_true_chain_is_fully_quoted() {
    let report = refused_chain_report();
    assert!(report.produced(), "{:?}", report.outcome);
    assert_eq!(
        report.quality,
        jarde_jvm::ir::Quality::Fallback,
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("ChainOrFieldDuplicatePhi.result ="),
        "{}",
        report.text
    );
    assert!(
        !report.text.contains("ChainOrFieldDuplicatePhi.mirror ="),
        "{}",
        report.text
    );

    let quoted: BTreeSet<u32> = report
        .text
        .lines()
        .filter(|line| line.trim_start().starts_with("// @bytecode "))
        .flat_map(|line| {
            line.split_whitespace()
                .skip(2)
                .take_while(|part| part.parse::<u32>().is_ok())
                .map(|part| part.parse().expect("quoted BCI is numeric"))
        })
        .collect();
    let mapped: BTreeSet<u32> = report
        .source_map
        .segments()
        .iter()
        .flat_map(|segment| segment.origin().bcis().clone())
        .collect();
    for bci in [0, 1, 4, 5, 8, 11, 14, 15, 18, 19, 20, 23, 26] {
        assert!(
            quoted.contains(&bci),
            "BCI {bci} not quoted:\n{}",
            report.text
        );
        assert!(
            mapped.contains(&bci),
            "BCI {bci} not mapped:\n{}",
            report.text
        );
    }
}
