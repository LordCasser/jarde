//! Temporary root-only proof audit for the two javac 23 / -g:none reference-slot cases.
//!
//! To run after copying this file into `tests/` as an ignored integration test:
//! `cargo test -p jarde-java --test reference_slot_ir_root_audit -- --ignored --nocapture`.
//! This file is not a product or test edit in the repository.

use jarde::*;
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalEdgeKind, MethodIr, Slot, ValueId,
};
use std::collections::{BTreeMap, BTreeSet};
use std::slice;

const ARRAY_THEN_LIST: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-reference-slot-lifetimes/javac23/ArrayThenList.class"
);
const LIST_THEN_MAP: &[u8] = include_bytes!(
    "../../../../tests/fixtures/p3-reference-slot-lifetimes/javac23/ListThenMap.class"
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

struct Fixture {
    snapshot: ArtifactSnapshot,
    method: PhysicalMethodId,
}

fn fixture(class: &[u8], name: &[u8], descriptor: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("standalone frozen class opens");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("class length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    Fixture {
        snapshot,
        method: PhysicalMethodId {
            owner: definition,
            name: bytes(name),
            descriptor: bytes(descriptor),
        },
    }
}

fn request(fixture: &Fixture) -> MethodAnalysisRequest {
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::StandaloneClass {
            snapshot: fixture.snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    MethodAnalysisRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: fixture.snapshot.id().clone(),
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
        method: fixture.method.clone(),
        stages: vec![AnalysisStage::Ssa],
    }
}

#[derive(Clone, Debug)]
struct Access {
    bci: u32,
    block: CanonicalBlockId,
    value: ValueId,
}

fn representative_chain(ssa: &jarde_jvm::method_ir::SsaTable, value: ValueId) -> Vec<ValueId> {
    let mut chain = Vec::new();
    let mut current = value;
    for _ in 0..=ssa.values().len() {
        chain.push(current);
        let Some(next) = ssa.value(current).replaced_by() else {
            return chain;
        };
        current = next;
    }
    panic!("replacement chain cycles: {chain:?}");
}

fn representative(ssa: &jarde_jvm::method_ir::SsaTable, value: ValueId) -> ValueId {
    *representative_chain(ssa, value)
        .last()
        .expect("representative chain has a start")
}

fn read_payload(ir: &MethodIr, expected: &[(u32, u32)], name: &str) {
    let canonical = ir.canonical().expect("canonical CFG is published");
    let ssa = ir.ssa().expect("SSA is published");
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    for block in ssa.blocks() {
        for instruction in block.instructions() {
            for (slot, value) in instruction.reads() {
                if *slot == Slot::Local(2) {
                    reads.push(Access {
                        bci: instruction.bci(),
                        block: block.block().clone(),
                        value: *value,
                    });
                }
            }
            for (slot, value) in instruction.writes() {
                if *slot == Slot::Local(2) {
                    writes.push(Access {
                        bci: instruction.bci(),
                        block: block.block().clone(),
                        value: *value,
                    });
                }
            }
        }
    }
    reads.sort_by_key(|access| access.bci);
    writes.sort_by_key(|access| access.bci);
    assert_eq!(
        writes.iter().map(|access| access.bci).collect::<Vec<_>>(),
        expected
            .iter()
            .map(|(write, _)| *write)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>(),
        "{name}: local 2 store anchors"
    );
    assert_eq!(
        reads.iter().map(|access| access.bci).collect::<Vec<_>>(),
        expected.iter().map(|(_, read)| *read).collect::<Vec<_>>(),
        "{name}: expected local 2 read anchors"
    );

    println!("\n{name}: canonical edges");
    for edge in canonical.edges() {
        println!("  {} -> {} {:?}", edge.from().bci(), edge.to().bci(), edge.kind());
        assert_eq!(edge.kind(), CanonicalEdgeKind::Normal, "ordinary CFG only");
    }
    println!("{name}: SSA phis");
    for phi in ssa.phis() {
        println!(
            "  block={} slot={:?} value={:?} inputs={:?} replacement={:?}",
            phi.block().bci(),
            phi.slot(),
            phi.value(),
            phi.inputs(),
            ssa.value(phi.value()).replaced_by()
        );
    }

    let definitions = writes
        .iter()
        .map(|write| (representative(ssa, write.value), write))
        .collect::<Vec<_>>();
    for write in &writes {
        let value = ssa.value(write.value);
        let chain = representative_chain(ssa, write.value);
        println!(
            "{name}: STORE slot=2 bci={} block={} value={:?} ty={:?} def={:?} chain={:?}",
            write.bci,
            write.block.bci(),
            write.value,
            value.ty(),
            value.def(),
            chain
        );
        for id in chain.iter().copied() {
            let item = ssa.value(id);
            println!("  chain value={id:?} ty={:?} def={:?} replaced_by={:?}", item.ty(), item.def(), item.replaced_by());
            for use_ in item.uses() {
                println!(
                    "    use value={id:?} block={} bci={:?}",
                    use_.block().bci(),
                    use_.bci()
                );
            }
        }
    }
    for read in &reads {
        let value = ssa.value(read.value);
        let chain = representative_chain(ssa, read.value);
        let owners = definitions
            .iter()
            .filter(|(defined, _)| *defined == representative(ssa, read.value))
            .map(|(_, write)| write.bci)
            .collect::<Vec<_>>();
        println!(
            "{name}: READ slot=2 bci={} block={} value={:?} ty={:?} def={:?} chain={:?} owners={owners:?}",
            read.bci,
            read.block.bci(),
            read.value,
            value.ty(),
            value.def(),
            chain
        );
        for id in chain.iter().copied() {
            for use_ in ssa.value(id).uses() {
                println!(
                    "  read-chain use value={id:?} block={} bci={:?}",
                    use_.block().bci(),
                    use_.bci()
                );
            }
        }
        assert_eq!(owners.len(), 1, "{name}: every slot-2 read has one store owner");
        assert_eq!(owners[0], expected.iter().find(|(_, read_bci)| *read_bci == read.bci).unwrap().0);
    }

    // Each segment is one value identity in these fixtures. The earlier value must have no use at
    // or after the second store; that includes a value held on the operand stack for a later call.
    let boundary = writes[1].bci;
    let earlier = representative(ssa, writes[0].value);
    for use_ in ssa.value(earlier).uses() {
        println!(
            "{name}: EARLIER-SEGMENT USE block={} bci={:?}",
            use_.block().bci(),
            use_.bci()
        );
        assert!(
            use_.bci().is_some_and(|bci| bci < boundary),
            "{name}: earlier local value is live across split boundary {boundary}"
        );
    }

    // Validate that no normal path after a later-segment access can return to a block with an
    // earlier-segment access. Starting from successors admits the normal same-block sequence but
    // still catches a genuine graph edge cycle back into that block.
    let earlier_blocks = reads
        .iter()
        .chain(&writes)
        .filter(|access| access.bci < boundary)
        .map(|access| access.block.clone())
        .collect::<BTreeSet<_>>();
    let later_blocks = reads
        .iter()
        .chain(&writes)
        .filter(|access| access.bci >= boundary)
        .map(|access| access.block.clone())
        .collect::<BTreeSet<_>>();
    let mut edges: BTreeMap<CanonicalBlockId, Vec<CanonicalBlockId>> = BTreeMap::new();
    for edge in canonical.edges() {
        edges.entry(edge.from().clone()).or_default().push(edge.to().clone());
    }
    let mut pending = later_blocks
        .iter()
        .flat_map(|block| edges.get(block).into_iter().flatten().cloned())
        .collect::<Vec<_>>();
    let mut visited = BTreeSet::new();
    while let Some(block) = pending.pop() {
        assert!(
            !earlier_blocks.contains(&block),
            "{name}: later segment can reach an earlier access block {}",
            block.bci()
        );
        if visited.insert(block.clone()) {
            pending.extend(edges.get(&block).into_iter().flatten().cloned());
        }
    }
}

fn audit(class: &[u8], class_name: &[u8], descriptor: &[u8], expected: &[(u32, u32)]) {
    let fixture = fixture(class, b"run", descriptor);
    let mut budget = Budget::new(limits());
    let analysis = analyze_method_ir(
        slice::from_ref(&fixture.snapshot),
        &request(&fixture),
        &mut budget,
    )
    .expect("method IR analysis completes");
    assert_eq!(analysis.report().body, MethodBodyState::Present);
    assert!(analysis.report().diagnostics.is_empty());
    read_payload(analysis.ir(), expected, &String::from_utf8_lossy(class_name));
}

#[test]
#[ignore = "root-only independent dump of real SSA and canonical facts"]
fn root_audits_reference_slot_owners_and_uses() {
    // ArrayThenList.run([I)I: write BCI -> representative owner expected by each read BCI.
    audit(
        ARRAY_THEN_LIST,
        b"ArrayThenList",
        b"([I)I",
        &[(3, 4), (44, 45), (44, 54), (44, 64), (44, 75)],
    );
    // ListThenMap.run(I)I.
    audit(
        LIST_THEN_MAP,
        b"ListThenMap",
        b"(I)I",
        &[(7, 8), (7, 17), (7, 28), (40, 41), (40, 52), (40, 56)],
    );
}
