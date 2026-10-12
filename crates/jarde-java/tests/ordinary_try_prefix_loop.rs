//! A foreach under one named catch remains wholly inside that catch's protected body.

use jarde_java::{
    DeclaringClass, MethodFacts, RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryFacts,
    RecoveryOutcome, RecoveryRequest, pass::JAVA_8, recover,
};
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
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

const CLASS: &[u8] = include_bytes!(
    "../../../tests/fixtures/cf11-structured-list-controls/UpstreamNestedLoops.class"
);
const OWNER: &str = "cf11/UpstreamNestedLoops";
const METHOD: &str = "handled";
const DESCRIPTOR: &str = "(Ljava/util/List;Ljava/util/List;)V";

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
        nested_depth: 32,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn analyzed() -> (jarde_jvm::method_ir::MethodIrAnalysis, RecoveryFacts) {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("frozen full CF11 class opens");
    let owner = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(CLASS).to_hex().to_string()),
            length: u64::try_from(CLASS.len()).expect("class length fits"),
        },
        variant: PhysicalVariant::Base,
    };
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
        owner,
        name: JvmBytes(METHOD.as_bytes().to_vec()),
        descriptor: JvmBytes(DESCRIPTOR.as_bytes().to_vec()),
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
    .expect("the frozen handled method analyzes");
    let declaration = analysis
        .ir()
        .declaration()
        .expect("same read states the method declaration");
    assert_eq!(
        String::from_utf8_lossy(&declaration.class_name().0),
        OWNER,
        "facts must describe the class that supplied this body"
    );
    assert_eq!(declaration.name().0.as_slice(), METHOD.as_bytes());
    assert_eq!(declaration.descriptor().0.as_slice(), DESCRIPTOR.as_bytes());
    assert_eq!(declaration.parameter_slots(), 3);
    assert_eq!(declaration.access_flags(), 0x0001);
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
    (analysis, facts)
}

#[test]
fn ordinary_try_prefix_loop_keeps_default_all_output_and_every_physical_origin() {
    let (analysis, facts) = analyzed();
    let mut default_budget = Budget::new(limits());
    let default = recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8),
        &mut default_budget,
    );
    let mut all_budget = Budget::new(limits());
    let all = recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut all_budget,
    );
    let mut source_map_budget = Budget::new(limits());
    let source_map_only = recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8).with_evidence(
            RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap),
        ),
        &mut source_map_budget,
    );
    assert!(default.outcome.produced(), "default: {:?}", default.outcome);
    assert!(all.outcome.produced(), "all: {:?}", all.outcome);
    assert!(
        source_map_only.outcome.produced(),
        "source map only: {:?}",
        source_map_only.outcome
    );
    assert_eq!(default.text, all.text);
    assert_eq!(source_map_only.text, default.text);
    assert_eq!(source_map_only.text, all.text);
    assert!(
        !all.source_map.is_empty(),
        "all evidence materializes the source map"
    );
    assert_eq!(source_map_only.source_map, all.source_map);
    assert!(
        !source_map_only.source_map.is_empty(),
        "the explicit source-map request materializes it"
    );
    assert!(!all.text.contains("@bytecode"), "{}", all.text);

    let protected = all
        .regions
        .iter()
        .filter(|region| [0, 7, 16, 50].iter().all(|bci| region.blocks.contains(bci)))
        .collect::<Vec<_>>();
    assert_eq!(
        protected.len(),
        1,
        "the try owns prefix, loop and catch: {:?}",
        all.regions
    );
    assert!(
        protected[0].structured,
        "the complete protected body is structured"
    );
    for (bci, label) in [
        (0, "prefix"),
        (7, "loop header"),
        (16, "loop body"),
        (47, "bridge"),
        (50, "handler"),
        (59, "join"),
    ] {
        let owners = all
            .regions
            .iter()
            .filter(|region| region.blocks.contains(&bci))
            .count();
        assert_eq!(owners, 1, "{label} BCI {bci} has {owners} region owners");
        assert!(
            !all.source_map.of_bci(bci).is_empty(),
            "{label} BCI {bci} lost its source map: {}",
            all.text
        );
    }
}

#[test]
fn cancellation_during_the_selected_recovery_commits_no_partial_region_or_source_map() {
    let (analysis, facts) = analyzed();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    let mut budget = Budget::with_cancellation_token(
        Limits {
            elapsed_millis: u64::MAX,
            ..limits()
        },
        cancellation,
    );
    let stopped = recover(
        &RecoveryRequest::new(analysis.ir(), &facts, JAVA_8)
            .with_evidence(RecoveryEvidenceRequest::all()),
        &mut budget,
    );
    assert!(matches!(stopped.outcome, RecoveryOutcome::Stopped(_)));
    assert!(stopped.text.is_empty());
    assert!(stopped.source_map.is_empty());
    assert!(stopped.regions.is_empty());
}
