//! P3 1.1 acceptance: the read-only IR handoff of one method-analysis request.
//!
//! The payload this slice lands (`jarde_jvm::method_ir::MethodIr`) is the input the recovery layer
//! consumes in 1.2/1.3, so what this file has to prove through the public surface is:
//!
//! 1. a real body really reaches it: one committed fixture, through the real entry point, and the
//!    payload holds the canonical graph, the frames and the names of that run — with concrete
//!    numbers, not with "not empty";
//! 2. what the payload holds is the body's own structure: the BCIs the graph covers are the
//!    instruction starts an independent read of the same bytes reports, the effect table has one
//!    record per decoded instruction, and the frames' locals array is the body's `max_locals`;
//! 3. it is **not** a re-derivation from the report: every number 1 and 2 state is a quantity the
//!    report has no field for — the report of the same run carries no block, no BCI, no value and
//!    no phi, and its own `origin` plane is empty (it stays empty by the P2 contract, which
//!    publishes the summary and not the artifact) while the payload's blocks anchor real
//!    `MethodPoint`s at the BCIs the body really has;
//! 4. the payload states the phase validity of its run: a request that publishes only the
//!    canonical graph hands over exactly that table and nothing above it;
//! 5. handing the payload over changes nothing about the run: the same request through the two
//!    entry points performs the same work (identical counted usage), and a cancelled one keeps its
//!    cancellation and hands over no table at all;
//! 6. and the seam cannot be written through: no published function takes `&mut self`, returns a
//!    `&mut` table, or exposes a field of one — checked at source level, because a mutable seam
//!    would compile and would leave every behavioural test above green.
//!
//! The consumer shape is exercised the way 1.3 will use it: `read_payload` takes `&MethodIr` and
//! nothing else — no report, no snapshot, no budget — so every number it states comes from the
//! tables alone.

use jarde::*;
use jarde_jvm::engine::analyze_method_ir;
use jarde_jvm::method_ir::{
    CanonicalBlockId, CanonicalEdgeKind, Definition, MethodIr, PhiInput, Slot,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::slice;

/// The ECJ 4.6.1 v45 fixture: its `finallyPath(I)I` is the one committed body whose canonical
/// graph is more than a single node.
const V45: &[u8] = include_bytes!("fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class");
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

/// Opens one class file and derives the physical identity of one of its methods.
fn fixture_of(class: &[u8], name: &[u8], descriptor: &[u8]) -> Fixture {
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.to_vec()), &mut budget)
        .expect("the fixture opens as a standalone CLASS");
    let definition = PhysicalDefinitionId {
        location: PhysicalClassLocation::StandaloneRoot {
            snapshot: snapshot.id().clone(),
        },
        class_bytes: ClassBytesId {
            digest: Digest(blake3::hash(class).to_hex().to_string()),
            length: u64::try_from(class.len()).expect("fixture length fits u64"),
        },
        variant: PhysicalVariant::Base,
    };
    let method = PhysicalMethodId {
        owner: definition,
        name: bytes(name),
        descriptor: bytes(descriptor),
    };
    Fixture { snapshot, method }
}

/// One caller domain rooted at the fixture, and nothing else.
fn environment(fixture: &Fixture) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Snapshot {
            snapshot: fixture.snapshot.id().clone(),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
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
    }
}

fn request(fixture: &Fixture, stages: Vec<AnalysisStage>) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment: environment(fixture),
        method: fixture.method.clone(),
        stages,
    }
}

/// Everything a recovery consumer reads out of one payload, through the payload's borrows alone.
#[derive(Debug, PartialEq, Eq)]
struct Reading {
    canonical_blocks: usize,
    canonical_edges: usize,
    canonical_normal_edges: usize,
    canonical_call_edges: usize,
    canonical_return_edges: usize,
    canonical_throw_sites: usize,
    canonical_handler_rows: usize,
    canonical_unreachable: usize,
    canonical_complete: bool,
    canonical_clones: usize,
    blocks_with_origin: usize,
    covered_starts: BTreeSet<u32>,
    instruction_spans: usize,
    frame_blocks: usize,
    frame_locals_slots: usize,
    frame_deepest_stack: usize,
    entry_locals: usize,
    entry_stack: usize,
    entry_inputs: usize,
    ssa_values: usize,
    ssa_phis: usize,
    ssa_blocks: usize,
    ssa_effect_records: usize,
    ssa_throwing_effects: usize,
    ssa_anchored_effects: usize,
    value_definitions: BTreeSet<String>,
    values_with_origin: usize,
    direct_phi_inputs: usize,
    itself_phi_inputs: usize,
    replaced_phis: usize,
    ssa_instruction_records: usize,
}

/// Reads one payload the way the recovery layer will: `&MethodIr`, nothing else.
///
/// The consistency checks inside are the ones a consumer can state from the published surface
/// alone — every endpoint, every throw site, every handler and every name resolves to a published
/// block — so a payload whose tables disagree with each other is caught here, not by luck.
fn read_payload(ir: &MethodIr) -> Reading {
    let canonical = ir
        .canonical()
        .expect("this request published the canonical graph");
    let frames = ir.frames().expect("this request published the frames");
    let ssa = ir.ssa().expect("this request published the names");

    let mut published: BTreeSet<CanonicalBlockId> = BTreeSet::new();
    let mut covered_starts: BTreeSet<u32> = BTreeSet::new();
    let mut intervals: Vec<(u32, u32)> = Vec::new();
    let mut blocks_with_origin = 0usize;
    for block in canonical.blocks() {
        published.insert(block.id().clone());
        assert_eq!(
            block.origin_bcis(),
            block.blocks(),
            "a canonical block maps back to the original BCIs it stands for"
        );
        assert!(
            !block.origin().members.is_empty(),
            "a canonical block carries at least one physical anchor"
        );
        assert_eq!(
            block.id().bci(),
            block.blocks()[0],
            "a canonical node names its own start first"
        );
        assert_eq!(
            block.id().is_clone(),
            !block.id().path().is_empty(),
            "a node is a clone exactly when a call path reaches it"
        );
        assert!(
            block.blocks()[0] < block.end_bci(),
            "a canonical node covers a non-empty interval of the body"
        );
        blocks_with_origin += 1;
        covered_starts.extend(block.blocks().iter().copied());
        intervals.push((block.blocks()[0], block.end_bci()));
        for ordinal in block.protected() {
            assert!(
                canonical
                    .handler_rows()
                    .iter()
                    .any(|row| row.ordinal() == *ordinal),
                "a protected ordinal names a published handler row"
            );
        }
    }
    // Every instruction a later table names has to sit inside a block the graph published: the
    // graph's own coverage of the body, read from the blocks' intervals.
    let inside = |bci: u32| {
        intervals
            .iter()
            .any(|(start, end)| *start <= bci && bci < *end)
    };
    let mut normal_edges = 0usize;
    let mut call_edges = 0usize;
    let mut return_edges = 0usize;
    for edge in canonical.edges() {
        assert!(
            published.contains(edge.from()) && published.contains(edge.to()),
            "every edge endpoint is a published block"
        );
        match edge.kind() {
            CanonicalEdgeKind::Normal => normal_edges += 1,
            CanonicalEdgeKind::Exception { handler_ordinal } => assert!(
                canonical
                    .handler_rows()
                    .iter()
                    .any(|row| row.ordinal() == handler_ordinal),
                "an exception edge names a published handler row"
            ),
            CanonicalEdgeKind::Call { call_site } => {
                assert!(
                    inside(call_site),
                    "a `jsr` edge names an instruction of the body"
                );
                call_edges += 1;
            }
            CanonicalEdgeKind::Return { call_site } => {
                assert!(
                    inside(call_site),
                    "a return edge names an instruction of the body"
                );
                return_edges += 1;
            }
        }
    }
    for site in canonical.throw_sites() {
        assert!(
            published.contains(site.block()),
            "a throw site names a published block"
        );
        assert!(
            !site.origin().members.is_empty(),
            "a throw site carries its own physical anchor"
        );
    }
    for row in canonical.handler_rows() {
        if let Some(handler) = row.handler() {
            assert!(
                published.contains(handler),
                "a handler row names a published block"
            );
        }
        for block in row.protected() {
            assert!(
                published.contains(block),
                "a covered block of a handler row is a published block"
            );
        }
    }

    let frame_blocks = frames.blocks();
    let entry = frame_blocks
        .first()
        .expect("a published frame table holds the entry state");
    assert!(
        published.contains(entry.block()),
        "the entry state belongs to a published block"
    );
    for block_frame in frame_blocks {
        assert!(
            published.contains(block_frame.block()),
            "every frame describes a published block"
        );
    }
    let entry_inputs = entry.inputs().len();

    let mut value_definitions: BTreeSet<String> = BTreeSet::new();
    let mut values_with_origin = 0usize;
    let mut replaced_phis = 0usize;
    for (id, value) in ssa.values_with_ids() {
        assert_eq!(
            ssa.value(id),
            value,
            "one value is addressable by its identity"
        );
        if !value.origin().members.is_empty() {
            values_with_origin += 1;
        }
        match value.def() {
            Definition::Entry { block, slot } => {
                assert!(
                    published.contains(block),
                    "an entry definition names a published block"
                );
                value_definitions.insert(format!("entry:{}", slot_name(slot)));
            }
            Definition::Instruction { block, bci } => {
                assert!(
                    published.contains(block) && inside(*bci),
                    "an instruction definition names a published block and an instruction of the body"
                );
                value_definitions.insert("instruction".to_string());
            }
            Definition::Phi { block, slot } => {
                assert!(
                    published.contains(block),
                    "a phi definition names a published block"
                );
                value_definitions.insert(format!("phi:{}", slot_name(slot)));
            }
            Definition::Caught { block, bci } => {
                assert!(
                    published.contains(block) && inside(*bci),
                    "a caught definition names a published block and a throwing instruction"
                );
                value_definitions.insert("caught".to_string());
            }
        }
        for use_site in value.uses() {
            assert!(
                published.contains(use_site.block()),
                "a use names a published block"
            );
        }
        if value.replaced_by().is_some() {
            replaced_phis += 1;
        }
    }
    let mut direct_phi_inputs = 0usize;
    let mut itself_phi_inputs = 0usize;
    for phi in ssa.phis() {
        assert!(
            published.contains(phi.block()),
            "a phi names a published block"
        );
        assert!(
            matches!(
                ssa.value(phi.value()).def(),
                Definition::Phi { block, .. } if block == phi.block()
            ),
            "a phi's definition is the one the table holds for it"
        );
        for input in phi.inputs() {
            match input {
                PhiInput::Value(value) => {
                    let _ = ssa.value(*value);
                    direct_phi_inputs += 1;
                }
                PhiInput::Itself => itself_phi_inputs += 1,
            }
        }
    }
    let mut ssa_instruction_records = 0usize;
    for block in ssa.blocks() {
        assert!(
            published.contains(block.block()),
            "a named block is a published block"
        );
        for (slot, value) in block.entry().iter().chain(block.exit()) {
            assert!(
                matches!(slot, Slot::Local(_) | Slot::Stack(_)),
                "a name is one slot's value"
            );
            let _ = ssa.value(*value);
        }
        for instruction in block.instructions() {
            assert!(
                inside(instruction.bci()),
                "a named instruction is an instruction of the body"
            );
            for (_, value) in instruction.reads().iter().chain(instruction.writes()) {
                let _ = ssa.value(*value);
            }
            ssa_instruction_records += 1;
        }
    }
    let mut throwing_effects = 0usize;
    let mut anchored_effects = 0usize;
    for effect in ssa.effects().instructions() {
        assert!(
            published.contains(effect.block()) && inside(effect.bci()),
            "an effect record names a published block and an instruction of the body"
        );
        if !effect.origin().members.is_empty() {
            anchored_effects += 1;
        }
        for ordinal in effect.handlers() {
            assert!(
                canonical
                    .handler_rows()
                    .iter()
                    .any(|row| row.ordinal() == *ordinal),
                "an effect's handler names a published row"
            );
        }
        if effect.may_throw() {
            throwing_effects += 1;
        }
    }

    Reading {
        canonical_blocks: canonical.blocks().len(),
        canonical_edges: canonical.edges().len(),
        canonical_normal_edges: normal_edges,
        canonical_call_edges: call_edges,
        canonical_return_edges: return_edges,
        canonical_throw_sites: canonical.throw_sites().len(),
        canonical_handler_rows: canonical.handler_rows().len(),
        canonical_unreachable: canonical.unreachable().len(),
        canonical_complete: canonical.completeness().is_complete(),
        canonical_clones: canonical
            .blocks()
            .iter()
            .filter(|block| block.id().is_clone())
            .count(),
        blocks_with_origin,
        covered_starts,
        instruction_spans: intervals.len(),
        frame_blocks: frame_blocks.len(),
        frame_locals_slots: frames.locals_slots(),
        frame_deepest_stack: frames.deepest_stack(),
        entry_locals: entry.locals().len(),
        entry_stack: entry.stack().len(),
        entry_inputs,
        ssa_values: ssa.values().len(),
        ssa_phis: ssa.phis().len(),
        ssa_blocks: ssa.blocks().len(),
        ssa_effect_records: ssa.effects().instructions().len(),
        ssa_throwing_effects: throwing_effects,
        ssa_anchored_effects: anchored_effects,
        value_definitions,
        values_with_origin,
        direct_phi_inputs,
        itself_phi_inputs,
        replaced_phis,
        ssa_instruction_records,
    }
}

fn slot_name(slot: &Slot) -> String {
    match slot {
        Slot::Local(index) => format!("local{index}"),
        Slot::Stack(depth) => format!("stack{depth}"),
    }
}

/// One usage snapshot with the wall clock removed, so two runs compare on what they charged.
fn counted_usage(execution: &ExecutionReport) -> UsageSnapshot {
    let usage = match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage.clone(),
    };
    UsageSnapshot {
        elapsed_millis: 0,
        ..usage
    }
}

fn diagnostic_codes(report: &MethodAnalysisReport) -> Vec<&str> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

fn stage_states(report: &MethodAnalysisReport) -> Vec<StageState> {
    report
        .stages
        .iter()
        .map(|stage| stage.state.clone())
        .collect()
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn read_repository_file(relative: &str) -> String {
    let path = repository_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{relative} is unreadable at {}: {error}", path.display()))
}

/// The reader's own decode of the fixture body, used as the independent byte-level witness.
#[derive(Debug)]
struct Body {
    max_locals: u16,
    max_stack: u16,
    instruction_starts: BTreeSet<u32>,
    instruction_count: usize,
    handler_records: usize,
}

fn decoded_body(class: &[u8], fixture: &Fixture) -> Body {
    let mut budget = Budget::new(limits());
    let header = class_facts(class, &mut budget).expect("the fixture is a class file");
    let member = header
        .methods
        .iter()
        .find(|member| {
            member.name.raw().0 == fixture.method.name.0
                && member.descriptor.raw().0 == fixture.method.descriptor.0
        })
        .expect("the fixture declares the method under test");
    let facts =
        method_code_facts(class, member, &mut budget).expect("the fixture body decodes completely");
    assert!(facts.stopped_at.is_none(), "the whole body was decoded");
    Body {
        max_locals: facts.max_locals,
        max_stack: facts.max_stack,
        instruction_starts: facts.instructions.iter().map(|fact| fact.bci).collect(),
        instruction_count: facts.instructions.len(),
        handler_records: facts.exception_handlers.len(),
    }
}

/// The files that publish the payload and its tables: the seam this slice adds, and the evidence
/// the source guard below reads.
const SEAM_FILES: [&str; 5] = [
    "crates/jarde-jvm/src/method_ir.rs",
    "crates/jarde-jvm/src/canonical.rs",
    "crates/jarde-jvm/src/cfg.rs",
    "crates/jarde-jvm/src/frame.rs",
    "crates/jarde-jvm/src/ssa.rs",
];

#[test]
fn the_payload_carries_the_tables_of_one_real_run() {
    // `HistoricalControlFlow.finallyPath(I)I` of the ECJ 4.6.1 v45 fixture: the one committed
    // body whose canonical graph is more than a single node — the ECJ `jsr`-based `finally`, with
    // two clones, three nodes the entry cannot reach and one throwing instruction.
    let fixture = fixture_of(V45, b"finallyPath", b"(I)I");
    let mut budget = Budget::new(limits());
    let analysis = analyze_method_ir(
        slice::from_ref(&fixture.snapshot),
        &request(&fixture, vec![AnalysisStage::Ssa]),
        &mut budget,
    )
    .expect("a legal request is answered, not raised");

    let report = analysis.report();
    assert_eq!(
        stage_states(report),
        vec![StageState::Completed; 6],
        "every scheduled phase completed, so the payload is the artifact of a finished run"
    );
    assert!(
        diagnostic_codes(report).is_empty(),
        "a completed pipeline reports no diagnostic: {:?}",
        diagnostic_codes(report)
    );
    assert_eq!(report.quality, Quality::Conservative);
    assert_eq!(
        report.semantic_validation,
        SemanticValidation::LocalInvariants
    );
    assert_eq!(report.body, MethodBodyState::Present);
    // The P2 report plane: what the summary states — the run, not the structure it produced. Its
    // `origin` is empty, and no field of it carries a block, a BCI, a value or a phi.
    assert!(
        report.origin.members.is_empty(),
        "the report of this build states no origin: {:?}",
        report.origin
    );

    let reading = read_payload(analysis.ir());
    assert_eq!(
        reading,
        Reading {
            canonical_blocks: 6,
            canonical_edges: 4,
            canonical_normal_edges: 0,
            canonical_call_edges: 2,
            canonical_return_edges: 2,
            canonical_throw_sites: 1,
            canonical_handler_rows: 0,
            canonical_unreachable: 3,
            canonical_complete: true,
            canonical_clones: 2,
            blocks_with_origin: 6,
            covered_starts: BTreeSet::from([0, 8, 11, 15, 17]),
            instruction_spans: 6,
            frame_blocks: 3,
            frame_locals_slots: 5,
            frame_deepest_stack: 1,
            entry_locals: 5,
            entry_stack: 0,
            entry_inputs: 0,
            ssa_values: 10,
            ssa_phis: 0,
            ssa_blocks: 3,
            ssa_effect_records: 10,
            ssa_throwing_effects: 0,
            ssa_anchored_effects: 0,
            value_definitions: BTreeSet::from([
                "entry:local0".to_string(),
                "entry:local1".to_string(),
                "instruction".to_string(),
            ]),
            values_with_origin: 8,
            direct_phi_inputs: 0,
            itself_phi_inputs: 0,
            replaced_phis: 0,
            ssa_instruction_records: 10,
        },
        "the payload of this body is this structure and nothing else"
    );

    // The same bytes, decoded by the reader alone: the numbers above describe this body. A payload
    // rebuilt from the report could not state any of them — the report carries no such quantity —
    // and it could not guess these either.
    let body = decoded_body(V45, &fixture);
    assert_eq!(body.instruction_count, 14);
    assert_eq!(body.handler_records, 1);
    assert_eq!(
        usize::from(body.max_locals),
        reading.frame_locals_slots,
        "the frames' locals array is the body's own `max_locals`"
    );
    assert!(
        reading.frame_deepest_stack <= usize::from(body.max_stack),
        "no block is entered with a deeper stack than the body's `max_stack`"
    );
    assert!(
        reading.covered_starts.is_subset(&body.instruction_starts),
        "every block start the graph publishes is an instruction start of the body: {:?}",
        reading.covered_starts
    );
    assert_eq!(
        reading.ssa_effect_records, reading.ssa_instruction_records,
        "one effect record per instruction the names hold"
    );
    assert!(
        reading.ssa_instruction_records < body.instruction_count,
        "the names cover the blocks the entry reaches, which here is fewer instructions than the body"
    );

    // The three zeroes above are this body's truth, not a hole in the surface: `finallyPath` has
    // no handler record that covers a site of a reached block and no merge of disagreeing values.
    // The phi surface itself is pinned by `method_ir`'s own unit test over an assembled branch.
}

#[test]
fn a_request_that_publishes_only_the_graph_hands_over_only_that_table() {
    let fixture = fixture_of(V45, b"finallyPath", b"(I)I");
    let stages = vec![AnalysisStage::CanonicalCfg];
    let mut budget = Budget::new(limits());
    let analysis = analyze_method_ir(
        slice::from_ref(&fixture.snapshot),
        &request(&fixture, stages.clone()),
        &mut budget,
    )
    .expect("a legal request is answered, not raised");

    let ir = analysis.ir();
    assert!(
        ir.canonical().is_some(),
        "the graph was the requested phase and was published"
    );
    assert_eq!(
        ir.canonical().expect("published").blocks().len(),
        6,
        "the same body, the same graph"
    );
    assert!(ir.frames().is_none(), "the frames were never requested");
    assert!(ir.ssa().is_none(), "the names were never requested");
    assert_eq!(
        stage_states(analysis.report()),
        vec![StageState::Completed; 4],
        "the four phases up to the graph ran, and nothing stands behind them"
    );

    // The same request through the product entry point: one run each, and the same work charged.
    // Handing the payload over is not a second analysis and does not pay for one.
    let mut product_budget = Budget::new(limits());
    let product = Engine::new()
        .analyze_method(
            slice::from_ref(&fixture.snapshot),
            &request(&fixture, stages),
            &mut product_budget,
        )
        .expect("a legal request is answered, not raised");
    assert_eq!(
        counted_usage(&product.execution),
        counted_usage(&analysis.report().execution),
        "the two entries charge the same budget for the same request"
    );
    assert_eq!(
        stage_states(&product),
        stage_states(analysis.report()),
        "and they state the same phase results"
    );
}

#[test]
fn a_cancelled_request_hands_over_no_table_and_keeps_its_cancellation() {
    let fixture = fixture_of(V45, b"finallyPath", b"(I)I");
    let token = CancellationToken::new();
    token.cancel();
    let stages = vec![AnalysisStage::Ssa];
    let mut budget = Budget::with_cancellation_token(limits(), token.clone());
    let analysis = analyze_method_ir(
        slice::from_ref(&fixture.snapshot),
        &request(&fixture, stages.clone()),
        &mut budget,
    )
    .expect("a cancelled request is answered with a cancellation, not raised");

    assert!(
        matches!(
            analysis.report().execution,
            ExecutionReport::Cancelled { .. }
        ),
        "a cancelled request keeps its cancellation: {:?}",
        analysis.report().execution
    );
    let ir = analysis.ir();
    assert!(
        ir.canonical().is_none() && ir.frames().is_none() && ir.ssa().is_none(),
        "a cancelled run published no table, so the payload is empty"
    );

    // The stop of the product entry, for the same request under the same cancelled token.
    let mut product_budget = Budget::with_cancellation_token(limits(), token);
    let product = Engine::new()
        .analyze_method(
            slice::from_ref(&fixture.snapshot),
            &request(&fixture, stages),
            &mut product_budget,
        )
        .expect("a cancelled request is answered with a cancellation, not raised");
    assert_eq!(
        counted_usage(&product.execution),
        counted_usage(&analysis.report().execution),
        "the handoff neither repeats work nor changes what was charged"
    );
    assert_eq!(
        stage_states(&product),
        stage_states(analysis.report()),
        "and the cancellation reaches the same phases in both entries"
    );
}

#[test]
fn the_handoff_seam_exposes_no_mutable_or_publicly_stored_table() {
    // The behavioural tests above cannot state this one: a `pub fn blocks_mut(&mut self)` or a
    // public field of `CanonicalCfg` would compile, would leave every assertion above green, and
    // would let the recovery layer write the producer's tables. The guard reads the seam's own
    // source instead, so the whole crate's public surface is checked at once.
    let mut mutating = Vec::new();
    let mut public_fields = Vec::new();
    let mut accessors = 0usize;

    let src = Path::new("crates/jarde-jvm/src");
    for entry in walk(&repository_root().join(src)) {
        let relative = entry
            .strip_prefix(repository_root())
            .expect("a source file below the repository root")
            .to_string_lossy()
            .to_string();
        let source = read_repository_file(&relative);
        for (index, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.contains("pub fn") {
                accessors += 1;
                if line.contains("&mut self") || line.contains("-> &mut") {
                    mutating.push(format!("{relative}:{}: {line}", index + 1));
                }
            }
            if SEAM_FILES.contains(&relative.as_str())
                && let Some(rest) = line.strip_prefix("pub ")
            {
                let item = rest.split([' ', '(', '<']).next().unwrap_or_default();
                if !matches!(
                    item,
                    "fn" | "struct" | "enum" | "use" | "mod" | "const" | "type"
                ) {
                    public_fields.push(format!("{relative}:{}: {line}", index + 1));
                }
            }
        }
    }

    assert!(
        mutating.is_empty(),
        "no public function of this crate takes `&mut self` or hands out a mutable view: {mutating:#?}"
    );
    assert!(
        public_fields.is_empty(),
        "the handoff publishes accessors, never a field of a table: {public_fields:#?}"
    );
    // Not vacuous: the seam really is where those accessors live, and the payload type really is
    // declared there.
    assert!(
        accessors > 40,
        "the crate's public functions are the seam and the entry points: {accessors}"
    );
    assert!(
        read_repository_file("crates/jarde-jvm/src/method_ir.rs").contains("pub struct MethodIr {"),
        "the payload type is declared by the seam module"
    );
    assert!(
        read_repository_file("crates/jarde-jvm/src/method_ir.rs")
            .contains("pub fn canonical(&self) -> Option<&CanonicalCfg>"),
        "and the graph is read through a shared borrow"
    );
}

/// Every file below one directory, in a deterministic order.
fn walk(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("{} is readable: {error}", directory.display()))
            .map(|entry| entry.expect("one directory entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            if entry.is_dir() {
                pending.push(entry);
            } else if entry.extension().is_some_and(|extension| extension == "rs") {
                files.push(entry);
            }
        }
    }
    files.sort();
    files
}
