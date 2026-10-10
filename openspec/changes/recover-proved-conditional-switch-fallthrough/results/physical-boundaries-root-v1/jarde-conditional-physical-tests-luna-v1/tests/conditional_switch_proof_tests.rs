//! Real canonical-class boundary checks for the conditional switch certificate.
//!
//! Append this module body inside `crates/jarde-java/src/region.rs`'s existing private `tests`
//! module after composing the certificate helper. Every proof invocation receives the full edge
//! iterator from a reader-produced canonical graph; no synthetic edge rows or fake CFG are used.

fn prove_real_conditional_switch(
    class: &[u8],
    expected_edges: &[(u32, u32)],
) -> (Option<BTreeMap<u32, u32>>, Vec<(Vec<i64>, bool, u32)>) {
    use jarde_jvm::engine::analyze_method_ir;
    use jarde_jvm::environment::ResolutionEnvironment;
    use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
    use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
    use jarde_reader::budget::{Budget, Limits};
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode,
        MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty,
        RuntimeView,
    };

    let mut analysis_budget = Budget::new(Limits {
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
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut analysis_budget)
        .expect("complete frozen boundary class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot { snapshot: snapshot.id().clone() },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("fixture length fits"),
        },
        variant: PhysicalVariant::Base,
    };
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass { snapshot: snapshot.id().clone() }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(b"partialBreak".to_vec()),
        descriptor: JvmBytes(b"(II)Ljava/lang/String;".to_vec()),
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
        &mut analysis_budget,
    )
    .expect("reader analyzes the exact class method");
    let ir = analysis.ir();
    let canonical = ir.canonical().expect("canonical CFG is available");
    let ssa = ir.ssa().expect("SSA is available");
    let code = ir.code().expect("decoded instructions are available");
    let declaration = ir.declaration().expect("reader supplies the method declaration");
    assert_eq!(declaration.name().0.as_slice(), b"partialBreak");
    assert_eq!(declaration.descriptor().0.as_slice(), b"(II)Ljava/lang/String;");
    assert_eq!(canonical.unreachable(), &[], "complete fixture has no unreachable blocks");
    assert_eq!(canonical.blocks().len(), 6);
    assert_eq!(canonical.edges().len(), expected_edges.len());

    let block = |bci| {
        canonical
            .blocks()
            .iter()
            .map(|block| block.id())
            .find(|id| id.bci() == bci && id.path().is_empty())
            .expect("observed canonical block with an empty clone path")
    };
    let observed_blocks = canonical
        .blocks()
        .iter()
        .map(|block| {
            assert!(block.id().path().is_empty(), "fixture must retain its real empty clone path");
            block.id().bci()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(observed_blocks, BTreeSet::from([0, 36, 40, 57, 67, 74]));

    let observed_edges = canonical
        .edges()
        .iter()
        .map(|edge| {
            assert_eq!(edge.kind(), CanonicalEdgeKind::Normal);
            assert!(edge.from().path().is_empty() && edge.to().path().is_empty());
            (edge.from().bci(), edge.to().bci())
        })
        .collect::<BTreeSet<_>>();
    let expected_edges = expected_edges.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(observed_edges, expected_edges, "all canonical rows are pinned");

    let view = NormalFlowView::build(canonical, &mut analysis_budget)
        .expect("normal-flow projection comes from the exact canonical graph");
    assert_eq!(view.kept_edges(), canonical.edges().len());
    let dispatch = block(0);
    let dispatch_node = view.index_of(dispatch).expect("switch dispatch node");
    let switch_bci = 9;
    let operations = Operations::of(code, ir.constant_pool());
    let (cases, default) = operations
        .get(switch_bci)
        .and_then(Operation::switch)
        .expect("decoded switch instruction at BCI 9");

    // Use the same target grouping as Walker::switch_region: labels sharing one target become a
    // group, and the default target joins that group when applicable.
    let mut groups: Vec<(Vec<i64>, bool, u32)> = Vec::new();
    for (key, target) in cases {
        match groups.iter_mut().find(|group| group.2 == *target) {
            Some(group) => group.0.push(*key),
            None => groups.push((vec![*key], false, *target)),
        }
    }
    if Some(default) != Some(74) {
        match groups.iter_mut().find(|group| group.2 == default) {
            Some(group) => group.1 = true,
            None => groups.push((Vec::new(), true, default)),
        }
    }
    let join_node = view
        .immediate_post_dominator(dispatch_node)
        .expect("reader graph has one immediate switch join");
    assert_eq!(view.id_of(join_node).map(CanonicalBlockId::bci), Some(74));
    let targets = groups
        .iter()
        .filter(|group| group.2 != 74)
        .map(|group| {
            let node = view.index_of(block(group.2)).expect("decoded case target is in graph");
            (group.2, node)
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(targets.keys().copied().collect::<Vec<_>>(), vec![36, 57, 67]);

    // This is the same certificate API used by the production caller. Keep the unmodified full
    // canonical row set so the result describes the real class, including row multiplicity.
    let full_rows = canonical
        .edges()
        .iter()
        .map(|edge| (edge.from(), edge.kind(), edge.to()))
        .collect::<Vec<_>>();
    let mut proof_budget = Budget::new(Limits {
        analysis_steps: 1 << 20,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    });
    let proof = prove_switch_fallthroughs(
        full_rows.iter().copied(),
        &view,
        ssa,
        &operations,
        &groups,
        &targets,
        Some(join_node),
        Some(74),
        dispatch_node,
        switch_bci,
        &mut proof_budget,
    )
    .expect("ample proof budget completes");
    (proof, groups)
}

#[test]
fn real_multiple_exit_class_refuses_conditional_switch_certificate() {
    const CLASS: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-switch-boundaries/javac8/A-multiple-exit/ConditionalSwitchBoundaries.class"
    );
    assert_eq!(CLASS.len(), 1494);
    assert_eq!(
        blake3::hash(CLASS).to_hex().to_string(),
        "d185c867cc368dd8ff7a662b2351b8a18390a6c95e510046fe65d22b39798406"
    );
    let (proof, _) = prove_real_conditional_switch(
        CLASS,
        &[
            (0, 36), (0, 57), (0, 67), (36, 40), (36, 57), (40, 67), (57, 74), (67, 74),
        ],
    );
    assert_eq!(proof, None, "case 36 has two distinct case-entry exits: 57 and 67");
}

#[test]
fn real_nonadjacent_class_proves_map_but_caller_order_gate_must_refuse_it() {
    const CLASS: &[u8] = include_bytes!(
        "../../../tests/fixtures/p3-conditional-switch-boundaries/javac8/B-nonadjacent/ConditionalSwitchBoundaries.class"
    );
    assert_eq!(CLASS.len(), 1494);
    assert_eq!(
        blake3::hash(CLASS).to_hex().to_string(),
        "63978aeef832dec2f8a52e34d969576555032b163726ffd75307da433ebdb707"
    );
    let (proof, groups) = prove_real_conditional_switch(
        CLASS,
        &[
            (0, 36), (0, 57), (0, 67), (36, 40), (36, 67), (40, 67), (57, 74), (67, 74),
        ],
    );
    let fallthroughs = proof.expect("the complete real canonical graph proves one case exit");
    assert_eq!(fallthroughs, BTreeMap::from([(36, 67)]));

    // Mirror only the caller's label-order invariant here. The public recovery draft below
    // exercises Walker::switch_region itself and verifies that this class was not admitted.
    let mut ordered_groups = groups;
    ordered_groups.sort_by_key(|group| group.2);
    let positions = ordered_groups
        .iter()
        .enumerate()
        .map(|(index, group)| (group.2, index))
        .collect::<BTreeMap<_, _>>();
    assert!(!fallthroughs.iter().all(|(from, to)| {
        from < to
            && positions
                .get(to)
                .zip(positions.get(from))
                .is_some_and(|(to, from)| *to == from.saturating_add(1))
    }), "entry 57 lies between source 36 and target 67");
}
