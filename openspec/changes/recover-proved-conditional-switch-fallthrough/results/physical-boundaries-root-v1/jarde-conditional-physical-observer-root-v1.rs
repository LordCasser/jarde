//! Temporary, read-only observer for the five frozen conditional-switch boundary methods.
//! Copy into crates/jarde-java/tests only for a guarded temporary cargo test, then remove it.

use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::environment::ResolutionEnvironment;
use jarde_jvm::ir::{AnalysisStage, MethodAnalysisRequest};
use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
use jarde_reader::budget::{Budget, Limits};
use jarde_reader::classfile::{MethodSelector, inspect_method_bytecode};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant,
};
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};

const CLASS_PINS: [(&str, &[u8], usize, &str); 4] = [
    ("javac8-A-multiple-exit", include_bytes!("/private/tmp/jarde-conditional-physical-root-v1/cases/javac8/A-multiple-exit/classes/ConditionalSwitchBoundaries.class"), 1494, "94cda1eeb3cb95dd3b211989fcc5ed9ad6744162c225f47483e7a02cc8a19350"),
    ("javac8-B-nonadjacent", include_bytes!("/private/tmp/jarde-conditional-physical-root-v1/cases/javac8/B-nonadjacent/classes/ConditionalSwitchBoundaries.class"), 1494, "006e5a27f8afaedcbbdc65f603721edb2c89f128e85ad4d8679915f268e133b1"),
    ("javac23-A-multiple-exit", include_bytes!("/private/tmp/jarde-conditional-physical-root-v1/cases/javac23/A-multiple-exit/classes/ConditionalSwitchBoundaries.class"), 1488, "afff2fb119e9a53d5d13b5b88a66720e0ec303a62c384651b751091cccbe515d"),
    ("javac23-B-nonadjacent", include_bytes!("/private/tmp/jarde-conditional-physical-root-v1/cases/javac23/B-nonadjacent/classes/ConditionalSwitchBoundaries.class"), 1488, "1420bbeb909bf3b9790353e7eaeafc5cbe6f71665d267f707e8b732a1b9d6ad5"),
];
const METHODS: [(&str, &str); 1] = [("partialBreak", "(II)Ljava/lang/String;")];

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
        nested_depth: 16,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

fn analyze_and_print(label: &str, class: &[u8], name: &str, descriptor: &str) {
    let mut budget = Budget::new(limits());
    let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("frozen class snapshot opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("class length fits"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: JvmBytes(name.as_bytes().to_vec()),
        descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
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
    .expect("real method IR analysis completes");
    assert_eq!(analysis.report().method, method);
    assert!(
        analysis.ir().canonical().is_some()
            && analysis.ir().ssa().is_some()
            && analysis.ir().code().is_some(),
        "all requested IR stages must be present"
    );

    let inspection = inspect_method_bytecode(
        class,
        MethodSelector {
            name: JvmBytes(name.as_bytes().to_vec()),
            descriptor: JvmBytes(descriptor.as_bytes().to_vec()),
        },
        &mut Budget::new(limits()),
    )
    .expect("exact physical method bytecode decodes");
    assert!(
        inspection.stopped_at.is_none(),
        "physical decode stopped: {:?}",
        inspection.stopped_at
    );

    println!(
        "METHOD_BEGIN class_leg={label} owner=ConditionalSwitchBoundaries name={name} descriptor={descriptor}"
    );
    println!("ANALYSIS_REPORT {:#?}", analysis.report());
    println!(
        "PHYSICAL_CODE max_stack={} max_locals={} code_span={:?} execution={:?}",
        inspection.max_stack, inspection.max_locals, inspection.code_span, inspection.execution
    );
    for row in &inspection.instructions {
        println!(
            "PHYSICAL_INSTRUCTION bci={} opcode=0x{:02x} width={} span={:?} operands_span={:?} cp={:?}",
            row.bci, row.opcode, row.width, row.span, row.operands_span, row.constant_pool_index
        );
    }
    for handler in &inspection.exception_handlers {
        println!("PHYSICAL_EXCEPTION_TABLE {handler:?}");
    }
    if let Some(code) = analysis.ir().code() {
        assert_eq!(
            inspection.instructions, code.instructions,
            "inspection and analyzed decode differ"
        );
        assert_eq!(
            inspection.exception_handlers, code.exception_handlers,
            "exception table differs between public readers"
        );
        assert_eq!(
            code.instructions.len(),
            code.operands().len(),
            "instruction/operand facts are not aligned"
        );
        for (instruction, operands) in code.instructions.iter().zip(code.operands()) {
            println!(
                "PHYSICAL_TYPED_INSTRUCTION bci={} opcode=0x{:02x} effective_opcode=0x{:02x} width={} operands={operands:?}",
                instruction.bci, instruction.opcode, operands.effective_opcode, instruction.width
            );
        }
        for target in code
            .control_flow_targets()
            .expect("decoded targets validate")
        {
            println!("PHYSICAL_TARGET {target:?}");
        }
    } else {
        println!("PHYSICAL_CODE_FACTS none");
    }

    if let Some(canonical) = analysis.ir().canonical() {
        println!(
            "CANONICAL completeness={:?} unreachable={:?}",
            canonical.completeness(),
            canonical.unreachable()
        );
        for block in canonical.blocks() {
            println!(
                "CANONICAL_BLOCK bci={} path={:?} end_bci={} original_starts={:?} protected_rows={:?} origin={:?}",
                block.id().bci(),
                block.id().path(),
                block.end_bci(),
                block.blocks(),
                block.protected(),
                block.origin()
            );
        }
        for edge in canonical.edges() {
            println!(
                "CANONICAL_EDGE from={} from_path={:?} kind={:?} to={} to_path={:?}",
                edge.from().bci(),
                edge.from().path(),
                edge.kind(),
                edge.to().bci(),
                edge.to().path()
            );
        }
        for row in canonical.handler_rows() {
            println!("CANONICAL_HANDLER_ROW {row:?}");
        }
        for site in canonical.throw_sites() {
            println!("CANONICAL_THROW_SITE {site:?}");
        }
    } else {
        println!("CANONICAL none");
    }

    if let Some(ssa) = analysis.ir().ssa() {
        for block in ssa.blocks() {
            println!(
                "SSA_BLOCK bci={} path={:?} entry={:?} exit={:?}",
                block.block().bci(),
                block.block().path(),
                block.entry(),
                block.exit()
            );
            for instruction in block.instructions() {
                println!(
                    "SSA_INSTRUCTION block={} path={:?} bci={} opcode=0x{:02x} reads={:?} writes={:?}",
                    block.block().bci(),
                    block.block().path(),
                    instruction.bci(),
                    instruction.opcode(),
                    instruction.reads(),
                    instruction.writes()
                );
            }
        }
        for phi in ssa.phis() {
            println!(
                "SSA_PHI block={} path={:?} slot={:?} value={:?} inputs={:?}",
                phi.block().bci(),
                phi.block().path(),
                phi.slot(),
                phi.value(),
                phi.inputs()
            );
        }
    } else {
        println!("SSA none");
    }
    println!("METHOD_END class_leg={label} name={name} descriptor={descriptor}");
}

#[test]
fn print_real_conditional_switch_boundary_ir_for_both_frozen_classes() {
    for (label, class, expected_length, expected_sha256) in CLASS_PINS {
        assert_eq!(class.len(), expected_length, "{label} class length changed");
        // SHA-256 values are recorded here for the captured input identity. The same bytes also
        // print their runtime BLAKE3 identity via the exact PhysicalDefinitionId used in analysis.
        println!(
            "CLASS_INPUT leg={label} length={} sha256={expected_sha256} blake3={}",
            class.len(),
            blake3::hash(class).to_hex()
        );
        for (name, descriptor) in METHODS {
            analyze_and_print(label, class, name, descriptor);
        }
    }
}
