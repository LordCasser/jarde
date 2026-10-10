// Temporary test fragment for root review. Append inside the existing region.rs tests module;
// do not include its output as evidence until root applies and runs it.
#[test]
fn diagnose_cf12_switch_fallthrough_from_original_class_ir() {
    use jarde_jvm::engine::analyze_method_ir;
    use jarde_jvm::environment::ResolutionEnvironment;
    use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
    use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
        MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
        RuntimeView,
    };

    const CLASS: &[u8] = include_bytes!(
        "../../../openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/harness-v3-java/capture/TestSwitchWithFallThroughCase.test/input/TestSwitchWithFallThroughCase$TestCls.class"
    );
    assert_eq!(CLASS.len(), 1563, "frozen whole-class input length changed");
    eprintln!("CF12_REAL_IR class_bytes={} blake3={}", CLASS.len(), blake3::hash(CLASS).to_hex());

    let mut budget = Budget::new(jarde_reader::budget::Limits {
        input_bytes: 1 << 20,
        archive_entries: 100,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 100,
        method_bodies: 100,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 32,
        dependency_depth: 32,
        elapsed_millis: u64::MAX,
    });
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(CLASS.to_vec()), &mut budget)
        .expect("frozen whole CF12 class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(CLASS).to_hex().to_string()),
            length: u64::try_from(CLASS.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
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
    let request = MethodAnalysisRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 23,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        },
        method: PhysicalMethodId {
            owner: definition,
            name: JvmBytes(b"test".to_vec()),
            descriptor: JvmBytes(b"(IZZ)Ljava/lang/String;".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let analysis = analyze_method_ir(&[snapshot], &request, &mut budget)
        .expect("real test(IZZ)Ljava/lang/String; JVM analysis completes");
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("real canonical CFG exists");
    let ssa = ir.ssa().expect("real SSA exists");
    let code = ir.code().expect("real decoded method exists");
    let operations = Operations::of(code, ir.constant_pool());
    let chains = crate::concat::plan_four_conditional_strings(
        ssa,
        canonical,
        &operations,
        &crate::build::FieldCopies::default(),
        &mut budget,
    )
    .expect("existing concat planning completes on the real method");
    let view = crate::normal_flow::NormalFlowView::build(canonical, &mut budget)
        .expect("normal-flow projection builds from the real canonical CFG");
    let recovered = recover(
        canonical,
        &view,
        ssa,
        &operations,
        ir.constant_pool(),
        &chains,
        &crate::init::Sites::empty(),
        code,
        Some(false),
        false,
        &crate::pass::JAVA_8,
        &mut budget,
    )
    .expect("real switch method region recovery completes");

    let canonical_edges: Vec<_> = canonical
        .edges()
        .iter()
        .map(|edge| (
            (edge.from().bci(), edge.from().path().to_vec()),
            edge.kind(),
            (edge.to().bci(), edge.to().path().to_vec()),
        ))
        .collect();
    eprintln!("CF12_REAL_IR canonical_blocks={:?}", canonical.blocks().iter().map(|block| block.id()).collect::<Vec<_>>());
    eprintln!("CF12_REAL_IR canonical_edges={canonical_edges:#?}");
    eprintln!("CF12_REAL_IR canonical_unreachable={:?}", canonical.unreachable());
    eprintln!(
        "CF12_REAL_IR projection kept_edges={} return_edges={} excluded={:?}",
        view.kept_edges(),
        view.return_edges(),
        view.excluded(),
    );
    for id in view.ids() {
        let canonical_out: Vec<_> = canonical
            .edges()
            .iter()
            .filter(|edge| edge.from() == id)
            .map(|edge| (edge.kind(), edge.to().clone()))
            .collect();
        eprintln!(
            "CF12_REAL_IR node={id:?} view_successors={:?} canonical_out={canonical_out:?}",
            view.successor_ids(id),
        );
    }
    eprintln!("CF12_REAL_IR decoded_operations={:#?}", operations.iter().collect::<Vec<_>>());
    eprintln!("CF12_REAL_IR recovered_regions={:#?}", recovered.regions);
}
