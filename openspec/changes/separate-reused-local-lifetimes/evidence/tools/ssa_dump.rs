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
use std::{env, fs};
fn main() {
    let args: Vec<String> = env::args().collect();
    let class_path = &args[1];
    let method_name = &args[2];
    let descriptor = &args[3];
    let java_release = args
        .get(4)
        .and_then(|arg| arg.parse::<u16>().ok())
        .unwrap_or(8);
    let bytes = fs::read(class_path).unwrap();
    let mut limits = Limits::default();
    limits.input_bytes = 1 << 20;
    limits.read_bytes = 1 << 20;
    limits.output_bytes = 1 << 20;
    limits.class_bytes = 1 << 20;
    limits.attribute_bytes = 1 << 20;
    limits.code_bytes = 1 << 20;
    limits.result_items = 1 << 20;
    limits.class_headers = 100;
    limits.method_bodies = 100;
    limits.ir_items = 1 << 20;
    limits.ir_edges = 1 << 20;
    limits.analysis_steps = 1 << 20;
    limits.elapsed_millis = u64::MAX;
    let mut budget = Budget::new(limits);
    let snapshot =
        ArtifactSnapshot::open(ArtifactInput::bytes(bytes.clone()), &mut budget).unwrap();
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(&bytes).to_hex().to_string()),
            length: bytes.len() as u64,
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(method_name.as_bytes().to_vec()),
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
    let environment = ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: domain.clone(),
        },
        domains: vec![domain],
        providers: Vec::new(),
    };
    let request = MethodAnalysisRequest {
        environment,
        method,
        stages: AnalysisStage::ALL.to_vec(),
    };
    let analysis = analyze_method_ir(&[snapshot], &request, &mut budget).unwrap();
    let ir = analysis.ir();
    let ssa = match ir.ssa() {
        Some(ssa) => ssa,
        None => {
            eprintln!("REPORT {:#?}", analysis.report());
            return;
        }
    };
    println!("BLOCKS");
    for block in ssa.blocks() {
        println!(
            "block={} entry={:?} exit={:?}",
            block.block().bci(),
            block.entry(),
            block.exit()
        );
    }
    println!("SLOT 2/3 INSTRUCTIONS");
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            let selected_reads: Vec<_> = instruction
                .reads()
                .iter()
                .filter(|(slot, _)| matches!(slot, jarde_jvm::method_ir::Slot::Local(2 | 3)))
                .collect();
            let selected_writes: Vec<_> = instruction
                .writes()
                .iter()
                .filter(|(slot, _)| matches!(slot, jarde_jvm::method_ir::Slot::Local(2 | 3)))
                .collect();
            if !selected_reads.is_empty() || !selected_writes.is_empty() {
                println!(
                    "block={} bci={} reads={selected_reads:?} writes={selected_writes:?}",
                    block.block().bci(),
                    instruction.bci()
                );
            }
        }
    }
    println!("VALUES FOR SLOT 2/3 DEFINITIONS AND USES");
    for (id, value) in ssa.values_with_ids() {
        let relevant_def = matches!(
            value.def(),
            jarde_jvm::method_ir::Definition::Entry {
                slot: jarde_jvm::method_ir::Slot::Local(2 | 3),
                ..
            } | jarde_jvm::method_ir::Definition::Phi {
                slot: jarde_jvm::method_ir::Slot::Local(2 | 3),
                ..
            }
        );
        let relevant_insn = matches!(value.def(), jarde_jvm::method_ir::Definition::Instruction { bci, .. } if ssa.blocks().iter().flat_map(|b| b.instructions()).any(|i| i.bci() == *bci && i.writes().iter().any(|(slot, value_id)| matches!(slot, jarde_jvm::method_ir::Slot::Local(2 | 3)) && *value_id == id)));
        if relevant_def || relevant_insn {
            println!(
                "v{id:?} ty={:?} def={:?} replaced={:?} uses={:?}",
                value.ty(),
                value.def(),
                value.replaced_by(),
                value.uses()
            );
        }
    }
    println!("PHIS FOR SLOT 2/3");
    for phi in ssa
        .phis()
        .iter()
        .filter(|phi| matches!(phi.slot(), jarde_jvm::method_ir::Slot::Local(2 | 3)))
    {
        println!(
            "block={} slot={:?} value={:?} inputs={:?}",
            phi.block().bci(),
            phi.slot(),
            phi.value(),
            phi.inputs()
        );
    }
    println!("EDGES");
    for edge in ir.canonical().unwrap().edges() {
        println!(
            "{} -> {} {:?}",
            edge.from().bci(),
            edge.to().bci(),
            edge.kind()
        );
    }
}
