use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest, StageState};
use jarde_jvm::method_ir::{Definition, ValueId};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 64,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        class_headers: 4,
        method_bodies: 4,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 2,
        elapsed_millis: u64::MAX,
    }
}

fn analyze(code: &[u8]) -> jarde_jvm::method_ir::MethodIrAnalysis {
    let bytes = jarde_reader::classfile::test_class::single_method(52, 16, 0, code);
    let snapshot = ArtifactSnapshot::open(
        ArtifactInput::bytes(bytes.clone()),
        &mut Budget::new(limits()),
    )
    .expect("the synthetic class opens");
    let loader = LoaderId("app".to_owned());
    let app = LoadDomain {
        loader: loader.clone(),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let environment = ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: jarde_reader::view::LayoutMode::Generic,
            },
            load_domain: app.clone(),
        },
        domains: vec![app],
        providers: Vec::new(),
    };
    let method = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(&bytes).to_hex().to_string()),
                length: bytes.len() as u64,
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(b"method".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    };
    analyze_method_ir(
        std::slice::from_ref(&snapshot),
        &MethodAnalysisRequest {
            environment,
            method,
            stages: vec![AnalysisStage::Ssa],
        },
        &mut Budget::new(limits()),
    )
    .expect("the method analysis request is valid")
}

fn frame_and_ssa_complete(analysis: &jarde_jvm::method_ir::MethodIrAnalysis) -> bool {
    let report = analysis.report();
    [AnalysisStage::Frame, AnalysisStage::Ssa]
        .iter()
        .all(|stage| {
            report
                .stages
                .iter()
                .find(|result| result.stage == *stage)
                .is_some_and(|result| result.state == StageState::Completed)
        })
}

/// Run one stack-copy shape. Inputs are distinct constant-producing instructions so equal types
/// still have different SSA identities. `outputs` lists each post-op stack value bottom-to-top as
/// an index into the input list.
fn assert_copy_shape(name: &str, inputs: &[u8], widths: &[u8], opcode: u8, outputs: &[usize]) {
    assert_eq!(inputs.len(), widths.len(), "{name}: input widths");
    let mut code = inputs.to_vec();
    let op_bci = u32::try_from(code.len()).expect("small test code");
    code.push(opcode);
    for source in outputs.iter().rev() {
        code.push(match widths[*source] {
            1 => 0x57, // pop
            2 => 0x58, // pop2
            width => panic!("{name}: invalid fixture width {width}"),
        });
    }
    code.push(0xb1); // return

    let analysis = analyze(&code);
    assert!(
        frame_and_ssa_complete(&analysis),
        "{name}: Frame and SSA should complete: {:?}",
        analysis.report().stages
    );
    let ssa = analysis.ir().ssa().expect("SSA is published");
    let instruction = ssa
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| instruction.bci() == op_bci && instruction.opcode() == opcode)
        .unwrap_or_else(|| panic!("{name}: copy instruction missing at BCI {op_bci}"));

    let input_ids = inputs
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let bci = u32::try_from(index).expect("small fixture");
            ssa.values_with_ids()
                .find_map(|(id, value)| {
                    matches!(value.def(), Definition::Instruction { bci: def_bci, .. } if *def_bci == bci)
                        .then_some(id)
                })
                .unwrap_or_else(|| panic!("{name}: input value at BCI {bci} missing"))
        })
        .collect::<Vec<ValueId>>();
    let expected_reads = input_ids.iter().rev().copied().collect::<Vec<_>>();
    assert_eq!(
        instruction
            .reads()
            .iter()
            .map(|(_, id)| *id)
            .collect::<Vec<_>>(),
        expected_reads,
        "{name}: stack operands must be read top-to-bottom with distinct input identities"
    );

    let writes = instruction
        .writes()
        .iter()
        .map(|(_, id)| *id)
        .collect::<Vec<_>>();
    assert_eq!(
        writes.len(),
        outputs.len(),
        "{name}: each pushed value is a write"
    );
    let copy_map = instruction
        .copies()
        .iter()
        .copied()
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        copy_map.len(),
        outputs.len(),
        "{name}: every output has one copy source"
    );
    assert_eq!(
        copy_map
            .keys()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        writes.iter().copied().collect(),
        "{name}: copy outputs must be exactly this instruction's writes"
    );
    for (output_id, source_index) in writes.iter().zip(outputs) {
        let source_id = input_ids[*source_index];
        assert_eq!(
            copy_map.get(output_id),
            Some(&source_id),
            "{name}: output {output_id:?} must copy input {source_id:?}"
        );
        let output_value = ssa.value(*output_id);
        let source_value = ssa.value(source_id);
        assert_eq!(output_value.ty(), source_value.ty(), "{name}: copied type");
        assert!(
            matches!(output_value.def(), Definition::Instruction { bci, .. } if *bci == op_bci),
            "{name}: copy output must be defined at the copy instruction"
        );
        assert!(
            matches!(source_value.def(), Definition::Instruction { bci, .. } if *bci < op_bci),
            "{name}: copy input must retain its earlier producer"
        );
    }
}

#[test]
fn dup2_x2_cat1_cat1_cat1_cat1_copies_the_expected_roots() {
    // Bottom-to-top [A, B, C, D] -> [C, D, A, B, C, D].
    assert_copy_shape(
        "dup2_x2 form 1",
        &[0x03, 0x04, 0x05, 0x06],
        &[1, 1, 1, 1],
        0x5e,
        &[2, 3, 0, 1, 2, 3],
    );
}

#[test]
fn dup2_x2_cat2_cat1_cat1_copies_the_expected_roots() {
    // Bottom-to-top [long A, int B, int C] -> [B, C, A, B, C].
    assert_copy_shape(
        "dup2_x2 form 2",
        &[0x09, 0x03, 0x04],
        &[2, 1, 1],
        0x5e,
        &[1, 2, 0, 1, 2],
    );
}

#[test]
fn dup2_x2_cat1_cat1_cat2_copies_the_expected_roots() {
    // Bottom-to-top [int A, int B, long C] -> [C, A, B, C].
    assert_copy_shape(
        "dup2_x2 form 3",
        &[0x03, 0x04, 0x09],
        &[1, 1, 2],
        0x5e,
        &[2, 0, 1, 2],
    );
}

#[test]
fn dup2_x2_cat2_cat2_copies_the_expected_roots() {
    // Bottom-to-top [long A, long B] -> [B, A, B].
    assert_copy_shape("dup2_x2 form 4", &[0x09, 0x0a], &[2, 2], 0x5e, &[1, 0, 1]);
}

#[test]
fn dup2_x2_rejects_illegal_category_one_category_two_category_one_form() {
    // This is not a JVMS shape, even though the remaining instructions consume its stack.
    let analysis = analyze(&[0x03, 0x09, 0x04, 0x5e, 0x57, 0x58, 0x57, 0x57, 0x58, 0xb1]);
    assert!(
        !frame_and_ssa_complete(&analysis),
        "the invalid [cat1, cat2, cat1] stack form must not complete Frame/SSA"
    );
    assert!(analysis.ir().frames().is_none());
    assert!(analysis.ir().ssa().is_none());
}

#[test]
fn other_stack_permutations_publish_their_actual_copy_sources() {
    let cases: &[(&str, &[u8], &[u8], u8, &[usize])] = &[
        ("dup", &[0x03], &[1], 0x59, &[0, 0]),
        ("dup_x1", &[0x03, 0x04], &[1, 1], 0x5a, &[1, 0, 1]),
        (
            "dup_x2 form 1",
            &[0x03, 0x04, 0x05],
            &[1, 1, 1],
            0x5b,
            &[2, 0, 1, 2],
        ),
        ("dup_x2 form 2", &[0x09, 0x03], &[2, 1], 0x5b, &[1, 0, 1]),
        ("dup2 form 1", &[0x03, 0x04], &[1, 1], 0x5c, &[0, 1, 0, 1]),
        ("dup2 form 2", &[0x09], &[2], 0x5c, &[0, 0]),
        (
            "dup2_x1 form 1",
            &[0x03, 0x04, 0x05],
            &[1, 1, 1],
            0x5d,
            &[1, 2, 0, 1, 2],
        ),
        ("dup2_x1 form 2", &[0x03, 0x09], &[1, 2], 0x5d, &[1, 0, 1]),
        ("swap", &[0x03, 0x04], &[1, 1], 0x5f, &[1, 0]),
    ];
    for (name, inputs, widths, opcode, outputs) in cases {
        assert_copy_shape(name, inputs, widths, *opcode, outputs);
    }
}
