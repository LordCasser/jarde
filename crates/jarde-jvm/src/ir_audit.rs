//! 5.3's structural invariants over the published IR payloads.
//!
//! `ssa_oracle` compares the *names* against a naive reaching-definition reading, and
//! `frame_oracle` recomputes the *frames*. Neither of them audits the structural facts the design
//! states about the published artifacts themselves, which is what this module does — over the
//! **published** tables, by recomputing each invariant from the data a consumer can see:
//!
//! * **the canonical graph covers the set it claims**: the blocks the entry reaches through the
//!   published edges are exactly the published block list, the `unreachable` list is disjoint from
//!   it, every edge endpoint is a block of the graph, and the blocks' code intervals do not
//!   overlap (`blocks_cover_the_reachable_set_and_nothing_twice`);
//! * **origin is traceable, and a clone is one-to-many**: every block's origin names original
//!   block starts the body really has, the first of them is the block's own start, and a shared
//!   subroutine's original instructions appear in more than one canonical node — the same BCI in
//!   several nodes, told apart by the call path
//!   (`origin_is_traceable_to_the_original_bcis`, `a_clone_is_one_bci_in_several_nodes`);
//! * **def-use agrees in both directions**: the occurrence records of every value are exactly the
//!   occurrences the published table holds — every read of it and every phi operand naming it,
//!   counted per occurrence and not per place — and every value's definition is witnessed by the
//!   instruction, the phi or the entry record the table holds
//!   (`def_use_agrees_in_both_directions`);
//! * **one phi input per logical predecessor**: a phi's arity is the block's *logical* input
//!   count — the frames' own record, which is finer than the aggregated canonical edges — plus the
//!   caller's contribution at the entry block (`one_phi_input_per_logical_predecessor`).
//!
//! The last one is the invariant the design calls out by name: an exception edge aggregates every
//! throw site its record covers, so a handler fed by two sites of one block has **one** edge and
//! **two** logical inputs. The test states both numbers for that body, so an implementation that
//! counted the edges would fail here.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use jarde_reader::budget::Budget;
use jarde_reader::classfile::{class_facts, method_code_facts};
use jarde_reader::model::{
    ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId,
    PhysicalVariant, SnapshotId,
};
use jarde_reader::view::LoaderId;

use crate::call_context::{CallContextOutcome, call_contexts};
use crate::canonical::{CanonicalBlockId, CanonicalCfg, CanonicalOutcome, canonical_cfg};
use crate::cfg::raw_cfg;
use crate::frame::{FrameMethod, FrameOutcome, FrameTable, RefType, Value, frames};
use crate::ssa::{Definition, PhiInput, SsaOutcome, SsaTable, ValueId, ssa};

/// One fixture body, through every layer up to the names.
struct Body {
    facts: jarde_reader::classfile::MethodCodeFacts,
    canonical: CanonicalCfg,
    frames: FrameTable,
    ssa: SsaTable,
}

fn limits() -> jarde_reader::budget::Limits {
    jarde_reader::budget::Limits {
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 1 << 20,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        elapsed_millis: u64::MAX,
        ..jarde_reader::budget::Limits::default()
    }
}

fn budget() -> Budget {
    Budget::new(limits())
}

/// Every layer up to the names, over one class file that must reach the `ssa` phase.
fn body_of(bytes: &[u8], name: &'static [u8], descriptor: &'static [u8]) -> Body {
    let mut budget = budget();
    let header = class_facts(bytes, &mut budget).expect("the fixture is a class file");
    let member = header
        .methods
        .iter()
        .find(|member| {
            member.name.raw().0.as_slice() == name
                && member.descriptor.raw().0.as_slice() == descriptor
        })
        .expect("the fixture declares the method the audit names");
    let facts = method_code_facts(bytes, member, &mut budget).expect("the fixture decodes");
    let pool = header.constant_pool.clone();
    let raw = raw_cfg(&facts, &mut budget).expect("the fixture has a raw graph");
    let contexts = match call_contexts(&facts, &raw, header.major_version, &mut budget)
        .expect("the call-context walk runs")
    {
        CallContextOutcome::Established(contexts) => contexts,
        other => panic!("the fixture must establish contexts, got {other:?}"),
    };
    let method_id = PhysicalMethodId {
        owner: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: SnapshotId("test".to_string()),
            },
            class_bytes: ClassBytesId {
                digest: Digest("test".to_string()),
                length: 0,
            },
            variant: PhysicalVariant::Base,
        },
        name: JvmBytes(name.to_vec()),
        descriptor: JvmBytes(descriptor.to_vec()),
    };
    let canonical = match canonical_cfg(&facts, &raw, &contexts, &method_id, &mut budget)
        .expect("the budget is ample")
    {
        CanonicalOutcome::Canonical(graph) => *graph,
        CanonicalOutcome::Fallback { message } => {
            panic!("the fixture must normalize, but stopped: {message}")
        }
    };
    let loader = LoaderId("app".to_string());
    let method = FrameMethod {
        access_flags: member.access_flags,
        name,
        descriptor,
        owner: b"Test",
        super_class: Some(b"java/lang/Object"),
        pool: &pool,
        loader: &loader,
    };
    let frames = match frames(&facts, &canonical, &method, &mut budget).expect("a legal run") {
        FrameOutcome::Frames(table) => *table,
        other => panic!("the fixture must produce frames, got {other:?}"),
    };
    let ssa = match ssa(&facts, &canonical, &frames, &method, &mut budget).expect("a legal run") {
        SsaOutcome::Ssa(table) => *table,
        SsaOutcome::Inconsistent { message } => {
            panic!("the fixture must publish names, but was refused: {message}")
        }
    };
    Body {
        facts,
        canonical,
        frames,
        ssa,
    }
}

/// The constant-pool entry of the class the named catch type of the overlap fixture names: the
/// `Class` entry **9**, whose name is the `Utf8` entry 8 the pool of [`class_of`] writes just
/// before it. The pool holds both for every fixture; only the overlap fixture's second record
/// refers to them.
const THROWABLE_CLASS: u16 = 9;

/// One class file of `Test` with one method whose `Code` is `code` at class-file version `major`.
///
/// One `Code` attribute's `exception_table` record is `(start_pc, end_pc, handler_pc,
/// catch_type)`: a `catch_type` of 0 is the catch-all record, and [`THROWABLE_CLASS`] is the
/// named one the overlap fixture uses. A catch-all record is what every fixture but that one
/// declares, so the pool entry the named one needs is written beside the seven the others share
/// and stays inert for them.
#[allow(clippy::too_many_arguments, reason = "one fixture builder per module")]
fn class_of(
    major: u16,
    flags: u16,
    name: &[u8],
    descriptor: &[u8],
    code: &[u8],
    max_stack: u16,
    max_locals: u16,
    handlers: &[(u16, u16, u16, u16)],
) -> Vec<u8> {
    let mut pool = Vec::new();
    utf8(&mut pool, b"Test");
    class(&mut pool, 1);
    utf8(&mut pool, b"java/lang/Object");
    class(&mut pool, 3);
    utf8(&mut pool, name);
    utf8(&mut pool, descriptor);
    utf8(&mut pool, b"Code");
    utf8(&mut pool, b"java/lang/Throwable");
    class(&mut pool, THROWABLE_CLASS - 1);

    let mut content = Vec::new();
    u16b(&mut content, max_stack);
    u16b(&mut content, max_locals);
    content.extend_from_slice(
        &u32::try_from(code.len())
            .expect("a fixture body fits u32")
            .to_be_bytes(),
    );
    content.extend_from_slice(code);
    u16b(
        &mut content,
        u16::try_from(handlers.len()).expect("fixture handlers fit u16"),
    );
    for (start, end, handler, catch_type) in handlers {
        u16b(&mut content, *start);
        u16b(&mut content, *end);
        u16b(&mut content, *handler);
        u16b(&mut content, *catch_type);
    }
    u16b(&mut content, 0);

    let mut method = Vec::new();
    u16b(&mut method, flags);
    u16b(&mut method, 5);
    u16b(&mut method, 6);
    u16b(&mut method, 1);
    u16b(&mut method, 7);
    method.extend_from_slice(
        &u32::try_from(content.len())
            .expect("fixture Code content fits u32")
            .to_be_bytes(),
    );
    method.extend_from_slice(&content);

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0);
    u16b(&mut bytes, major);
    u16b(&mut bytes, 10);
    bytes.extend_from_slice(&pool);
    u16b(&mut bytes, 0x0021);
    u16b(&mut bytes, 2);
    u16b(&mut bytes, 4);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 1);
    bytes.extend_from_slice(&method);
    u16b(&mut bytes, 0);
    bytes
}

fn u16b(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn utf8(pool: &mut Vec<u8>, text: &[u8]) {
    pool.push(1);
    u16b(
        pool,
        u16::try_from(text.len()).expect("a fixture name fits u16"),
    );
    pool.extend_from_slice(text);
}

fn class(pool: &mut Vec<u8>, name: u16) {
    pool.push(7);
    u16b(pool, name);
}

/// The descriptor of the exception fixtures.
const EXCEPTION_METHOD: &[u8] = b"(Ljava/lang/Object;)Ljava/lang/Object;";

/// The committed ECJ 45 fixture: `finallyPath(I)I` compiles with two `jsr` call sites sharing one
/// subroutine, so the canonical graph holds one clone per call site.
const V45: &[u8] =
    include_bytes!("../../../tests/fixtures/historical/ecj-4.6.1/v45/HistoricalControlFlow.class");

/// Two throw sites of one block under one catch-all record: one edge, two logical inputs.
fn two_throw_sites() -> Body {
    const CODE: &[u8] = &[
        0x01, 0x4c, // 0: aconst_null; 1: astore_1
        0x04, 0x03, 0x6c, 0x57, // 2: iconst_1; iconst_0; idiv; pop
        0x2a, 0x4c, 0x2a, 0x4d, // 6: aload_0; 7: astore_1; 8: aload_0; 9: astore_2
        0x04, 0x03, 0x6c, 0x57, // 10: iconst_1; iconst_0; idiv; pop
        0x2a, 0xc7, 0xff, 0xf3, // 14: aload_0; 15: ifnonnull 2
        0x2b, 0xb0, // 18: aload_1; 19: areturn
        0x57, 0x2b, 0xb0, // 20: pop; 21: aload_1; 22: areturn
    ];
    let class = class_of(
        49,
        0x0009,
        b"method",
        EXCEPTION_METHOD,
        CODE,
        2,
        3,
        &[(2, 18, 20, 0)],
    );
    body_of(&class, b"method", EXCEPTION_METHOD)
}

/// **Two records** that name the same handler for the same site: record 0 names
/// `java/lang/Throwable` and record 1 catches everything.
///
/// The two records are two canonical edges between **one pair of blocks**, and that is the shape
/// which tells an input record keyed by its source from one keyed by its edge: same source, same
/// handler, the record's own ordinal the only thing that tells the two contributions apart. The
/// merge of a named catch and a catch-all is the unknown reference, so the handler's entry state
/// is *not* the class of either contribution — a table that keeps one record per source while the
/// merge still saw both states a class no input it published defines, which is what the `ssa`
/// phase refuses as a contradiction of its own artifacts. The order is the one the fuzz corpus
/// seed `method_analysis/exception-overlap-mixed.class` declares: the named record comes first,
/// so the record a table keyed by its source keeps is the catch-all while the input the names
/// resolve is the named one, and the refusal is the one a caller sees.
fn two_records_one_handler() -> Body {
    const CODE: &[u8] = &[
        0x04, 0x03, 0x6c, 0x57, // 0: iconst_1; iconst_0; idiv; pop
        0xb1, // 4: return
        0x57, 0xb1, // 5: pop; return, the handler both records name
    ];
    let class = class_of(
        49,
        0x0009,
        b"method",
        b"()V",
        CODE,
        2,
        1,
        &[(0, 4, 5, THROWABLE_CLASS), (0, 4, 5, 0)],
    );
    body_of(&class, b"method", b"()V")
}

/// A loop whose header is entered from the entry block, from a forward branch and from a back
/// edge: three logical inputs, three edges, and one phi per local that changes.
fn three_input_loop() -> Body {
    const CODE: &[u8] = &[
        0x03, 0x3b, // 0: iconst_0; 1: istore_0
        0x04, 0x3c, // 2: iconst_1; 3: istore_1
        0x1a, 0x06, 0xa2, 0x00, 0x0b, // 4: iload_0; 5: iconst_3; 6: if_icmpge 17
        0x1a, 0x04, 0x60, 0x3b, // 9: iload_0; 10: iconst_1; 11: iadd; 12: istore_0
        0x1b, 0x9a, 0xff, 0xf6, // 13: iload_1; 14: ifne 4
        0x1a, 0xac, // 17: iload_0; 18: ireturn
    ];
    let class = class_of(49, 0x0009, b"method", b"()I", CODE, 2, 2, &[]);
    body_of(&class, b"method", b"()I")
}

/// The wide forms and both switches, for the structural audits over a body with many blocks.
fn wide_and_switch() -> Body {
    let mut code = Vec::new();
    let mut fixups: Vec<(usize, usize, usize)> = Vec::new();
    code.push(0x04); // 0: iconst_1
    code.extend_from_slice(&[0xc4, 0x36, 0x01, 0x2c]); // 1: wide istore 300
    code.extend_from_slice(&[0xc4, 0x15, 0x01, 0x2c]); // 5: wide iload 300
    let switch_at = code.len();
    code.push(0xaa); // 9: tableswitch over {-1, 0}
    while !code.len().is_multiple_of(4) {
        code.push(0);
    }
    let default_operand = code.len();
    code.extend_from_slice(&[0; 4]);
    code.extend_from_slice(&(-1_i32).to_be_bytes());
    code.extend_from_slice(&0_i32.to_be_bytes());
    let case0 = code.len();
    code.extend_from_slice(&[0; 4]);
    let case1 = code.len();
    code.extend_from_slice(&[0; 4]);
    let t0 = code.len();
    code.push(0xb1);
    let t1 = code.len();
    code.push(0xb1);
    let default_block = code.len();
    code.push(0xb1);
    fixups.push((default_operand, switch_at, default_block));
    fixups.push((case0, switch_at, t0));
    fixups.push((case1, switch_at, t1));
    for (operand, opcode, target) in fixups {
        let offset = i32::try_from(target).expect("a fixture body fits i32")
            - i32::try_from(opcode).expect("a fixture body fits i32");
        code[operand..operand + 4].copy_from_slice(&offset.to_be_bytes());
    }
    let class = class_of(52, 0x0009, b"method", b"()V", &code, 1, 301, &[]);
    body_of(&class, b"method", b"()V")
}

/// The instruction BCIs of one body, and the starts of the blocks the raw graph cut it into.
fn instruction_starts(facts: &jarde_reader::classfile::MethodCodeFacts) -> BTreeSet<u32> {
    facts.instructions.iter().map(|fact| fact.bci).collect()
}

// ---------------------------------------------------------------------------
// The invariants
// ---------------------------------------------------------------------------

/// The blocks the entry reaches over the published edges, by block identity.
fn reachable(body: &Body) -> BTreeSet<CanonicalBlockId> {
    let entry = CanonicalBlockId {
        bci: 0,
        path: Vec::new(),
    };
    let mut index: BTreeMap<&CanonicalBlockId, Vec<&CanonicalBlockId>> = BTreeMap::new();
    for block in &body.canonical.blocks {
        index.entry(&block.id).or_default();
    }
    for edge in &body.canonical.edges {
        index.entry(&edge.from).or_default().push(&edge.to);
    }
    let mut reached = BTreeSet::new();
    let mut queue = VecDeque::new();
    if body.canonical.blocks.iter().any(|block| block.id == entry) {
        reached.insert(entry.clone());
        queue.push_back(entry);
    }
    while let Some(block) = queue.pop_front() {
        for successor in index.get(&block).into_iter().flatten() {
            if reached.insert((*successor).clone()) {
                queue.push_back((*successor).clone());
            }
        }
    }
    reached
}

/// Every occurrence of one value in the published table, counted as the multiset the use records
/// claim to be: one per instruction read, one per phi operand.
fn occurrences(body: &Body, value: ValueId) -> (usize, usize) {
    let mut reads = 0;
    let mut operands = 0;
    for block in body.ssa.blocks() {
        for instruction in &block.instructions {
            for (_, named) in &instruction.reads {
                if *named == value {
                    reads += 1;
                }
            }
        }
    }
    for phi in body.ssa.phis() {
        for input in &phi.inputs {
            if let PhiInput::Value(named) = input
                && *named == value
            {
                operands += 1;
            }
        }
    }
    (reads, operands)
}

/// Every value the published table references — as an entry or exit record, a read, a write, a
/// phi's own value or a phi operand.
///
/// The audit asserts that this set is the whole value list, so a value no record names is a
/// finding rather than a value nothing checks.
fn referenced_values(body: &Body) -> BTreeSet<ValueId> {
    let mut referenced = BTreeSet::new();
    for block in body.ssa.blocks() {
        for (_, value) in &block.entry {
            referenced.insert(*value);
        }
        for (_, value) in &block.exit {
            referenced.insert(*value);
        }
        for instruction in &block.instructions {
            for (_, value) in &instruction.reads {
                referenced.insert(*value);
            }
            for (_, value) in &instruction.writes {
                referenced.insert(*value);
            }
        }
    }
    for phi in body.ssa.phis() {
        referenced.insert(phi.value);
        for input in &phi.inputs {
            if let PhiInput::Value(value) = input {
                referenced.insert(*value);
            }
        }
    }
    referenced
}

/// The values one value's entry, exit, instruction writes and phis define, in the table's terms.
fn definitions_of(body: &Body, value: ValueId) -> usize {
    let mut witnesses = 0;
    for block in body.ssa.blocks() {
        if block.entry.iter().any(|(_, named)| *named == value) {
            witnesses += 1;
        }
        if block.exit.iter().any(|(_, named)| *named == value) {
            witnesses += 1;
        }
        for instruction in &block.instructions {
            if instruction.writes.iter().any(|(_, named)| *named == value) {
                witnesses += 1;
            }
        }
    }
    if body.ssa.phis().iter().any(|phi| phi.value == value) {
        witnesses += 1;
    }
    witnesses
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The five bodies every structural invariant is checked over.
    fn corpus() -> Vec<(&'static str, Body)> {
        vec![
            ("ecj v45 finallyPath", body_of(V45, b"finallyPath", b"(I)I")),
            ("two throw sites", two_throw_sites()),
            ("two records, one handler", two_records_one_handler()),
            ("three-input loop", three_input_loop()),
            ("wide and switch", wide_and_switch()),
        ]
    }

    #[test]
    fn the_reachable_set_is_the_published_blocks_minus_the_unreachable_ones() {
        // The published block list is the nodes the normalization created, and `unreachable`
        // marks the ones the entry cannot get to (the ECJ body's dead `jsr` call site is the
        // example the corpus carries). The two readings of the same list must agree exactly:
        // a node marked unreachable that the entry does reach, or a node neither reachable nor
        // marked, is a discontinuity between the graph and its own truth table.
        for (name, body) in corpus() {
            let reached = reachable(&body);
            let published: BTreeSet<&CanonicalBlockId> = body
                .canonical
                .blocks
                .iter()
                .map(|block| &block.id)
                .collect();
            let expected: BTreeSet<&CanonicalBlockId> = published
                .iter()
                .filter(|id| !body.canonical.unreachable.contains(id))
                .copied()
                .collect();
            assert_eq!(
                reached.iter().collect::<BTreeSet<&CanonicalBlockId>>(),
                expected,
                "{name}: the entry reaches exactly the published blocks that are not marked \
                 unreachable"
            );
            // Every edge endpoint is a block the graph states, and every instruction is covered
            // by at most one block *per call path* — a clone covers the same original
            // instructions as the node it was cloned from, which is what makes it a clone.
            for edge in &body.canonical.edges {
                for endpoint in [&edge.from, &edge.to] {
                    assert!(
                        published.contains(endpoint),
                        "{name}: the edge {edge:?} names a block the graph does not state"
                    );
                }
            }
            let starts = instruction_starts(&body.facts);
            let mut by_path: BTreeMap<Vec<u32>, BTreeMap<u32, usize>> = BTreeMap::new();
            for block in &body.canonical.blocks {
                let start = *block.blocks.first().expect("a block stands for a start");
                assert!(
                    block.end_bci > start,
                    "{name}: {:?} covers a non-empty interval",
                    block.id
                );
                let covered = by_path.entry(block.id.path.clone()).or_default();
                for bci in starts.range(start..block.end_bci) {
                    *covered.entry(*bci).or_default() += 1;
                }
            }
            for (path, covered) in &by_path {
                for (bci, count) in covered {
                    assert_eq!(
                        *count, 1,
                        "{name}: BCI {bci} is covered by {count} blocks of the call path {path:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn origin_is_traceable_to_the_original_bcis() {
        for (name, body) in corpus() {
            let starts = instruction_starts(&body.facts);
            for block in &body.canonical.blocks {
                let origins = block.origin_bcis();
                assert!(
                    !origins.is_empty(),
                    "{name}: {:?} states at least one origin",
                    block.id
                );
                assert_eq!(
                    origins, block.blocks,
                    "{name}: {:?} states one origin per original block it stands for, in order",
                    block.id
                );
                for bci in &origins {
                    assert!(
                        starts.contains(bci),
                        "{name}: {:?} states the origin {bci}, which is no instruction start",
                        block.id
                    );
                }
                assert_eq!(
                    origins.first(),
                    Some(&block.id.bci),
                    "{name}: {:?} starts at its first origin",
                    block.id
                );
            }
        }
    }

    #[test]
    fn a_clone_is_one_bci_in_several_nodes() {
        // The ECJ 45 body's `finally` subroutine is entered from two call sites, so its original
        // blocks stand for **two** canonical nodes each: one BCI, several nodes, told apart by
        // the call path the node carries.
        let body = body_of(V45, b"finallyPath", b"(I)I");
        let mut holders: BTreeMap<u32, Vec<&CanonicalBlockId>> = BTreeMap::new();
        for block in &body.canonical.blocks {
            for bci in block.origin_bcis() {
                holders.entry(bci).or_default().push(&block.id);
            }
        }
        let many: Vec<(u32, Vec<&CanonicalBlockId>)> = holders
            .into_iter()
            .filter(|(_, nodes)| nodes.len() >= 2)
            .collect();
        assert!(
            !many.is_empty(),
            "the shared subroutine's instructions stand for several canonical nodes"
        );
        for (bci, nodes) in &many {
            let paths: BTreeSet<&Vec<u32>> = nodes.iter().map(|node| &node.path).collect();
            assert_eq!(
                paths.len(),
                nodes.len(),
                "BCI {bci} appears in {} nodes and {} distinct call paths",
                nodes.len(),
                paths.len()
            );
            assert!(
                paths.iter().any(|path| !path.is_empty()),
                "BCI {bci} is held by a clone, whose node carries the call path it runs under"
            );
            assert!(
                *bci >= 17 && *bci < 23,
                "the BCI {bci} held by several nodes is inside the shared subroutine the two \
                 call sites enter"
            );
        }
        assert_eq!(
            body.canonical.clones, 2,
            "the two call sites are two clones, the same number the golden records"
        );
    }

    #[test]
    fn def_use_agrees_in_both_directions() {
        for (name, body) in corpus() {
            // Every value the table holds is referenced by some record, and every referenced
            // value is in the table: the two readings of the same list agree.
            let referenced = referenced_values(&body);
            assert_eq!(
                referenced.len(),
                body.ssa.values().len(),
                "{name}: every value the table holds is named by a definition or a use"
            );
            // Forward: every occurrence the table holds is named by exactly one record.
            for id in &referenced {
                let index = 0;
                let value = body.ssa.value(*id);
                let (reads, operands) = occurrences(&body, *id);
                let named_reads = value
                    .uses
                    .iter()
                    .filter(|use_record| use_record.bci.is_some())
                    .count();
                let named_operands = value
                    .uses
                    .iter()
                    .filter(|use_record| use_record.bci.is_none())
                    .count();
                assert_eq!(
                    named_reads, reads,
                    "{name}: value {index} is read {reads} time(s) and {} record(s) name a read",
                    named_reads
                );
                assert_eq!(
                    named_operands, operands,
                    "{name}: value {index} is a phi operand {operands} time(s) and {named_operands} \
                     record(s) name one"
                );
                // A use record names a block the table holds, and a read record names an
                // instruction of it.
                for use_record in &value.uses {
                    let block = body.ssa.block(&use_record.block).unwrap_or_else(|| {
                        panic!(
                            "{name}: a use names the unknown block {:?}",
                            use_record.block
                        )
                    });
                    if let Some(bci) = use_record.bci {
                        assert!(
                            block
                                .instructions
                                .iter()
                                .any(|instruction| instruction.bci == bci),
                            "{name}: a read record names BCI {bci} of {:?}",
                            use_record.block
                        );
                    }
                }
            }
            // Backward: every referenced value is defined and witnessed by the table.
            for id in &referenced {
                let index = 0;
                let value = body.ssa.value(*id);
                let witnesses = definitions_of(&body, *id);
                match &value.def {
                    Definition::Entry { .. } => assert!(
                        witnesses >= 1,
                        "{name}: an entry value is witnessed by the entry record"
                    ),
                    Definition::Instruction { block, bci } => {
                        assert!(
                            witnesses >= 1,
                            "{name}: the value defined by {block:?}@{bci} is witnessed by a write"
                        );
                    }
                    Definition::Phi { block, slot } => {
                        let phi = body
                            .ssa
                            .phis()
                            .iter()
                            .find(|phi| phi.value == *id)
                            .unwrap_or_else(|| {
                                panic!(
                                    "{name}: the value {index} claims a phi at {block:?} {slot:?}"
                                )
                            });
                        assert_eq!(
                            phi.block, *block,
                            "{name}: the phi is the one the value claims"
                        );
                        assert!(witnesses >= 1, "{name}: and the table holds it");
                    }
                    Definition::Caught { block, bci } => {
                        assert!(
                            body.canonical
                                .throw_sites
                                .iter()
                                .any(|site| site.block == *block && site.bci == *bci),
                            "{name}: a caught value names a throw site the graph holds"
                        );
                    }
                }
                // A replacement chain has an end, and a replaced value is not referenced as a
                // definition by any read.
                let mut current = Some(*id);
                let mut steps = 0;
                while let Some(step) = current {
                    steps += 1;
                    assert!(steps < 10_000, "{name}: the replacement chain is acyclic");
                    current = body.ssa.value(step).replaced_by;
                }
            }
        }
    }

    #[test]
    fn one_phi_input_per_logical_predecessor() {
        for (name, body) in corpus() {
            let entry = CanonicalBlockId {
                bci: 0,
                path: Vec::new(),
            };
            for phi in body.ssa.phis() {
                let frame = body
                    .frames
                    .entry(&phi.block)
                    .unwrap_or_else(|| panic!("{name}: the phi names a block with no frame"));
                let expected = frame.inputs.len() + usize::from(phi.block == entry);
                assert_eq!(
                    phi.inputs.len(),
                    expected,
                    "{name}: the phi of {:?} {:?} takes one operand per logical predecessor \
                     ({} records{})",
                    phi.block,
                    phi.slot,
                    frame.inputs.len(),
                    if phi.block == entry {
                        " plus the caller"
                    } else {
                        ""
                    }
                );
            }
        }
    }

    #[test]
    fn two_records_of_one_source_keep_two_logical_inputs_and_two_operands() {
        // The shape of the defect this audit was extended for: **two** exception records name one
        // handler for one site, so the same source block reaches the same handler through two
        // edges. The two contributions do not merge to either of them (a catch-all merged with a
        // named class is the unknown reference), so the handler's entry state has to be the merge
        // of *both* records and the phi of its caught reference has to take one operand per
        // record: one edge is not one input here either.
        let body = two_records_one_handler();
        let source = CanonicalBlockId {
            bci: 0,
            path: Vec::new(),
        };
        let handler = CanonicalBlockId {
            bci: 5,
            path: Vec::new(),
        };
        let edges: Vec<&crate::canonical::CanonicalEdge> = body
            .canonical
            .edges
            .iter()
            .filter(|edge| edge.to == handler)
            .collect();
        assert_eq!(
            edges.len(),
            2,
            "the two records of the table are two edges from one source to one handler: {:?}",
            body.canonical.edges
        );
        assert!(edges.iter().all(|edge| edge.from == source));

        let frame = body
            .frames
            .entry(&handler)
            .expect("the handler has a frame");
        assert_eq!(
            frame.inputs.len(),
            2,
            "and the handler is entered with one logical input per record: {:?}",
            frame.inputs
        );
        assert_eq!(
            frame.stack,
            vec![Value::Ref(RefType::Unknown)],
            "the two records merge to the unknown reference, which is neither of their classes"
        );
        let phis: Vec<&crate::ssa::SsaPhi> = body
            .ssa
            .phis()
            .iter()
            .filter(|phi| phi.block == handler)
            .collect();
        assert_eq!(
            phis.len(),
            1,
            "the handler's entry state has one value to name — the caught reference — and one phi \
             for it: {:?}",
            phis
        );
        let phi = phis[0];
        assert!(
            matches!(phi.slot, crate::ssa::Slot::Stack(0)),
            "the phi names the caught reference, not {:?}",
            phi.slot
        );
        assert_eq!(
            body.ssa.value(phi.value).ty,
            Value::Ref(RefType::Unknown),
            "and its class is the merge 4.1 stated for the slot, not the class of either record"
        );
        assert_eq!(
            phi.inputs.len(),
            2,
            "the phi of {:?} takes one operand per logical predecessor, not per distinct \
             contribution: {:?}",
            phi.slot,
            phi.inputs
        );
    }

    #[test]
    fn a_handler_with_two_sites_has_two_logical_inputs_and_one_edge() {
        // The design's sentence made checkable: the record aggregates the two sites into one
        // canonical edge, and the handler's phi counts the sites.
        let body = two_throw_sites();
        let handler = CanonicalBlockId {
            bci: 20,
            path: Vec::new(),
        };
        let edges: Vec<&crate::canonical::CanonicalEdge> = body
            .canonical
            .edges
            .iter()
            .filter(|edge| edge.to == handler)
            .collect();
        assert_eq!(
            edges.len(),
            1,
            "the two sites of the body's block feed one aggregated edge"
        );
        let frame = body
            .frames
            .entry(&handler)
            .expect("the handler has a frame");
        assert_eq!(
            frame.inputs.len(),
            2,
            "and the handler is entered with two logical inputs: {:?}",
            frame.inputs
        );
        let phis: Vec<&crate::ssa::SsaPhi> = body
            .ssa
            .phis()
            .iter()
            .filter(|phi| phi.block == handler)
            .collect();
        assert!(
            !phis.is_empty(),
            "the handler's entry state has phis of its own"
        );
        for phi in phis {
            assert_eq!(
                phi.inputs.len(),
                2,
                "the phi of {:?} takes one operand per site, not per edge",
                phi.slot
            );
        }
    }

    #[test]
    fn the_audited_bodies_really_hold_the_shapes_they_are_about() {
        // Non-vacuity: the audited bodies hold blocks, edges, phis and clones, so the invariants
        // above are not the agreement of empty sets.
        let mut blocks = 0;
        let mut edges = 0;
        let mut phis = 0;
        let mut values = 0;
        for (_, body) in corpus() {
            blocks += body.canonical.blocks.len();
            edges += body.canonical.edges.len();
            phis += body.ssa.phis().len();
            values += body.ssa.values().len();
        }
        assert!(blocks >= 10, "the corpus holds {blocks} blocks");
        assert!(edges >= 10, "and {edges} edges");
        assert!(phis >= 2, "and {phis} entry phis");
        assert!(values >= 20, "and {values} values");
    }
}
