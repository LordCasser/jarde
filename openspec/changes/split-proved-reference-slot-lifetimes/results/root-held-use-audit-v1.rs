//! Temporary root-only real SSA audit for HeldUse.run()Ljava/lang/Object;.
//! Standalone source; intentionally outside the repository. Build/run only by root when ready.

use jarde::*;
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::method_ir::{MethodIr, Slot};
use std::slice;

const HELD_USE: &[u8] = include_bytes!(
    "/private/tmp/jarde-ref-ref-slot-negative-20261009/replay-cli9-v3/held-use/classes/HeldUse.class"
);

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
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
        nested_depth: 8,
        dependency_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn bytes(value: &[u8]) -> JvmBytes {
    JvmBytes(value.to_vec())
}

fn run(ir: &MethodIr) {
    let canonical = ir.canonical().expect("canonical graph published");
    let ssa = ir.ssa().expect("SSA published");

    println!("canonical edges:");
    for edge in canonical.edges() {
        println!("  {} -> {} {:?}", edge.from().bci(), edge.to().bci(), edge.kind());
    }
    println!("SSA phis:");
    for phi in ssa.phis() {
        println!(
            "  block={} slot={:?} value={:?} inputs={:?} replaced_by={:?}",
            phi.block().bci(),
            phi.slot(),
            phi.value(),
            phi.inputs(),
            ssa.value(phi.value()).replaced_by()
        );
    }

    println!("all instruction effects (reads/writes include operand stack slots):");
    let mut slot_zero_reads = Vec::new();
    let mut slot_zero_writes = Vec::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            println!(
                "  block={} bci={} opcode=0x{:02x} reads={:?} writes={:?}",
                block.block().bci(),
                instruction.bci(),
                instruction.opcode(),
                instruction.reads(),
                instruction.writes()
            );
            for (slot, value) in instruction.reads() {
                if *slot == Slot::Local(0) {
                    slot_zero_reads.push((instruction.bci(), *value));
                }
            }
            for (slot, value) in instruction.writes() {
                if *slot == Slot::Local(0) {
                    slot_zero_writes.push((instruction.bci(), *value));
                }
            }
        }
    }
    assert_eq!(slot_zero_writes.iter().map(|(bci, _)| *bci).collect::<Vec<_>>(), [9, 19]);
    assert_eq!(slot_zero_reads.iter().map(|(bci, _)| *bci).collect::<Vec<_>>(), [10]);

    println!("local 0 anchors and value facts:");
    for (kind, anchors) in [("WRITE", &slot_zero_writes), ("READ", &slot_zero_reads)] {
        for (bci, value) in anchors {
            let fact = ssa.value(*value);
            println!(
                "  {kind} bci={bci} value={value:?} ty={:?} def={:?} origin={:?} replaced_by={:?} uses={:?}",
                fact.ty(),
                fact.def(),
                fact.origin(),
                fact.replaced_by(),
                fact.uses()
            );
        }
    }

    println!("every SSA value and its recorded uses:");
    for (id, value) in ssa.values_with_ids() {
        println!(
            "  value={id:?} ty={:?} def={:?} origin={:?} replaced_by={:?} uses={:?}",
            value.ty(),
            value.def(),
            value.origin(),
            value.replaced_by(),
            value.uses()
        );
    }
}

#[test]
#[ignore = "root-only real SSA audit for held operand-stack value across local overwrite"]
fn root_audits_held_use_ssa() {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(HELD_USE.to_vec()), &mut budget)
        .expect("frozen standalone class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(HELD_USE).to_hex().to_string()),
            length: u64::try_from(HELD_USE.len()).expect("class length fits u64"),
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
    let request = MethodAnalysisRequest {
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
        method: PhysicalMethodId {
            owner: definition,
            name: bytes(b"run"),
            descriptor: bytes(b"()Ljava/lang/Object;"),
        },
        stages: vec![AnalysisStage::Ssa],
    };
    let analysis = analyze_method_ir(slice::from_ref(&snapshot), &request, &mut budget)
        .expect("real method SSA analysis completes");
    assert_eq!(analysis.report().body, MethodBodyState::Present);
    run(analysis.ir());
}
