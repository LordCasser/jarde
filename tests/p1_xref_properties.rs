//! P1 structural-XRef properties over structured, in-memory class and archive fixtures.
//!
//! The generator builds structurally valid Java 8 classes from a controlled shape —
//! member counts, constant-pool entries and instruction sequences — instead of feeding
//! random bytes to a parser that only accepts classes. Four properties are checked:
//!
//! 1. a paged continuation with random page sizes and per-page result-item budgets
//!    splices back to exactly the one full run of the same query identity,
//! 2. a combined consumer schema reports exactly the union of the single-category runs,
//!    and a single-category run never reports another consumer's item,
//! 3. inserting constant-pool entries no consumer uses changes no `mentions_symbol` or
//!    `literal_value` fact (the raw `constant_pool_contains` probe may gain candidates),
//! 4. identical class bytes in different physical origins stay distinct items.
//!
//! Both CI seeds run this file (`PROPTEST_RNG_SEED=5350648285461741569` and
//! `5350648285461741570`). A failure prints the seed and proptest's minimized
//! counterexample; a property is never relaxed to fit the implementation.

use jarde::*;
use proptest::prelude::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

const STORE: u16 = 0;
const DEFLATE: u16 = 8;

fn limits() -> Limits {
    Limits {
        input_bytes: 1 << 20,
        archive_entries: 1_000,
        entry_bytes: 1 << 20,
        read_bytes: 1 << 20,
        class_bytes: 1 << 20,
        attribute_bytes: 1 << 20,
        code_bytes: 1 << 20,
        result_items: 10_000,
        output_bytes: 1 << 20,
        nested_depth: 4,
        elapsed_millis: u64::MAX,
    }
}

fn u16b(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn u32b(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn push(&mut self, entry: Vec<u8>) -> u16 {
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("fixture pool fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("text fits u16"),
        );
        entry.extend_from_slice(text);
        self.push(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.push(entry)
    }

    fn string(&mut self, value: u16) -> u16 {
        let mut entry = vec![8];
        u16b(&mut entry, value);
        self.push(entry)
    }

    fn integer(&mut self, value: i32) -> u16 {
        let mut entry = vec![3];
        entry.extend_from_slice(&value.to_be_bytes());
        self.push(entry)
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        let mut entry = vec![12];
        u16b(&mut entry, name);
        u16b(&mut entry, descriptor);
        self.push(entry)
    }

    fn member(&mut self, tag: u8, class: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![tag];
        u16b(&mut entry, class);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    fn declared(&self) -> u16 {
        u16::try_from(self.entries.len() + 1).expect("fixture pool fits u16")
    }

    fn bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }
}

fn zip_bytes(entries: &[(&[u8], &[u8], u16)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data, method) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(*method))
                .start()
                .unwrap();
            if *method == DEFLATE {
                let encoder =
                    flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::default());
                let mut writer = config.wrap(encoder);
                writer.write_all(data).unwrap();
                let (encoder, descriptor) = writer.finish().unwrap();
                encoder.finish().unwrap();
                entry.finish(descriptor).unwrap();
            } else {
                let mut writer = config.wrap(&mut entry);
                writer.write_all(data).unwrap();
                let (_, descriptor) = writer.finish().unwrap();
                entry.finish(descriptor).unwrap();
            }
        }
        archive.finish().unwrap();
    }
    output.into_inner()
}

// ---------------------------------------------------------------------------
// Structured generator
// ---------------------------------------------------------------------------

/// The controlled shape of one generated class.
#[derive(Clone, Debug)]
struct Shape {
    /// Makes every generated name unique to this case.
    suffix: u16,
    /// Number of distinct used `Methodref`s (`1..=3`).
    targets: usize,
    /// Number of distinct string constants (`1..=2`).
    strings: usize,
    /// Number of distinct integer constants (`1..=2`).
    integers: usize,
    /// Extra instructions after the three forced invocations: `(kind, which)` with
    /// `0`/`1` an invocation, `2` a string load and `3` an integer load.
    instructions: Vec<(u8, u8)>,
    /// Number of unused `Methodref`s with their own owners (`1..=2`).
    unused: usize,
    /// Page sizes the pagination property cycles through (`1..=3`).
    page_sizes: Vec<u8>,
}

fn shape_strategy() -> impl Strategy<Value = Shape> {
    (
        0u16..4096,
        1usize..=3,
        1usize..=2,
        1usize..=2,
        prop::collection::vec((0u8..4, 0u8..4), 0..=4),
        1usize..=2,
        prop::collection::vec(1u8..=3, 1..=4),
    )
        .prop_map(
            |(suffix, targets, strings, integers, instructions, unused, page_sizes)| Shape {
                suffix,
                targets,
                strings,
                integers,
                instructions,
                unused,
                page_sizes,
            },
        )
}

fn emit_invoke(code: &mut Vec<u8>, constant_pool_index: u16) -> u32 {
    let bci = u32::try_from(code.len()).expect("fixture code fits u32");
    code.push(0xb6);
    u16b(code, constant_pool_index);
    bci
}

fn emit_ldc(code: &mut Vec<u8>, constant_pool_index: u16) -> u32 {
    let bci = u32::try_from(code.len()).expect("fixture code fits u32");
    code.push(0x12);
    code.push(u8::try_from(constant_pool_index).expect("fixture pool index fits one byte"));
    bci
}

/// One generated class plus the names and coordinates the properties assert against.
struct BuiltClass {
    bytes: Vec<u8>,
    /// Used members: `(owner, name)` with descriptor `()V`.
    targets: Vec<(Vec<u8>, Vec<u8>)>,
    /// Members whose `Methodref` no instruction uses.
    unused: Vec<(Vec<u8>, Vec<u8>)>,
    /// Owners that exist only in the appended, unused constant-pool entries.
    extras: Vec<(Vec<u8>, Vec<u8>)>,
    string_values: Vec<Vec<u8>>,
    integer_values: Vec<i32>,
    /// BCI of every emitted invocation, grouped by the used member it names.
    invoke_bcis: Vec<Vec<u32>>,
}

/// Builds one class for `shape`, appending `extras` unused constant-pool entries after
/// every entry the class really uses.
///
/// The appended entries only extend the pool: the existing entries keep their indexes and
/// bytes, so the two builds of the same shape differ exactly by the trailing entries.
fn build_class(shape: &Shape, extras: usize) -> BuiltClass {
    let mut pool = Pool::default();
    let class_name = format!("p/Prop{}", shape.suffix).into_bytes();
    let class_name_index = pool.utf8(&class_name);
    let this_class = pool.class(class_name_index);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let run_name = pool.utf8(b"run");
    let void_descriptor = pool.utf8(b"()V");

    let mut targets = Vec::new();
    let mut target_refs = Vec::new();
    for index in 0..shape.targets {
        let owner = format!("p/T{index}Prop{}", shape.suffix).into_bytes();
        let owner_index = pool.utf8(&owner);
        let owner_class = pool.class(owner_index);
        let name_and_type = pool.name_and_type(run_name, void_descriptor);
        target_refs.push(pool.member(10, owner_class, name_and_type));
        targets.push((owner, b"run".to_vec()));
    }

    let mut string_values = Vec::new();
    let mut string_refs = Vec::new();
    for index in 0..shape.strings {
        let value = format!("s{index}p{}", shape.suffix).into_bytes();
        let value_index = pool.utf8(&value);
        string_refs.push(pool.string(value_index));
        string_values.push(value);
    }

    let mut integer_values = Vec::new();
    let mut integer_refs = Vec::new();
    for index in 0..shape.integers {
        let value = i32::from(shape.suffix) * 16 + i32::try_from(index).expect("small index");
        integer_refs.push(pool.integer(value));
        integer_values.push(value);
    }

    let mut unused = Vec::new();
    for index in 0..shape.unused {
        let owner = format!("p/Unused{index}Prop{}", shape.suffix).into_bytes();
        let owner_index = pool.utf8(&owner);
        let owner_class = pool.class(owner_index);
        let name = format!("never{index}p{}", shape.suffix).into_bytes();
        let name_index = pool.utf8(&name);
        let name_and_type = pool.name_and_type(name_index, void_descriptor);
        pool.member(10, owner_class, name_and_type);
        unused.push((owner, name));
    }

    let mut extra_entries = Vec::new();
    for index in 0..extras {
        let owner = format!("p/Extra{index}Prop{}", shape.suffix).into_bytes();
        let owner_index = pool.utf8(&owner);
        let owner_class = pool.class(owner_index);
        let name = format!("extra{index}p{}", shape.suffix).into_bytes();
        let name_index = pool.utf8(&name);
        let name_and_type = pool.name_and_type(name_index, void_descriptor);
        pool.member(10, owner_class, name_and_type);
        let text = pool.utf8(format!("extra-{index}-p{}", shape.suffix).as_bytes());
        pool.string(text);
        pool.integer(10_000 + i32::try_from(index).expect("small index"));
        extra_entries.push((owner, name));
    }

    let main_name = pool.utf8(b"main");
    let code_name = pool.utf8(b"Code");

    let mut code = Vec::new();
    let mut invoke_bcis: Vec<Vec<u32>> = vec![Vec::new(); targets.len()];
    // Three forced invocations of the first target, one forced string load and one forced
    // integer load make the pagination and literal-category properties non-vacuous; the
    // shape's own instructions follow.
    for _ in 0..3 {
        invoke_bcis[0].push(emit_invoke(&mut code, target_refs[0]));
    }
    emit_ldc(&mut code, string_refs[0]);
    emit_ldc(&mut code, integer_refs[0]);
    for &(kind, which) in &shape.instructions {
        match kind {
            0 | 1 => {
                let index = usize::from(which) % target_refs.len();
                invoke_bcis[index].push(emit_invoke(&mut code, target_refs[index]));
            }
            2 => {
                let index = usize::from(which) % string_refs.len();
                emit_ldc(&mut code, string_refs[index]);
            }
            _ => {
                let index = usize::from(which) % integer_refs.len();
                emit_ldc(&mut code, integer_refs[index]);
            }
        }
    }
    code.push(0xb1);

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 52);
    u16b(&mut bytes, pool.declared());
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, 0x0021);
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, object_class);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 0);
    u16b(&mut bytes, 1);
    u16b(&mut bytes, 0x0009);
    u16b(&mut bytes, main_name);
    u16b(&mut bytes, void_descriptor);
    u16b(&mut bytes, 1);
    u16b(&mut bytes, code_name);
    let mut body = Vec::new();
    u16b(&mut body, 2);
    u16b(&mut body, 1);
    u32b(
        &mut body,
        u32::try_from(code.len()).expect("fixture code fits u32"),
    );
    body.extend_from_slice(&code);
    u16b(&mut body, 0);
    u16b(&mut body, 0);
    u32b(
        &mut bytes,
        u32::try_from(body.len()).expect("fixture body fits u32"),
    );
    bytes.extend_from_slice(&body);
    u16b(&mut bytes, 0);

    BuiltClass {
        bytes,
        targets,
        unused,
        extras: extra_entries,
        string_values,
        integer_values,
        invoke_bcis,
    }
}

// ---------------------------------------------------------------------------
// Query helpers
// ---------------------------------------------------------------------------

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut budget)
        .expect("generated fixture must open")
}

fn method_target(owner: &[u8], name: &[u8]) -> QueryTarget {
    QueryTarget::Symbol {
        value: SymbolRef::Method {
            owner: JvmBytes(owner.to_vec()),
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
    }
}

fn string_target(value: &[u8]) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::String {
            value: JvmBytes(value.to_vec()),
        },
    }
}

fn integer_target(value: i32) -> QueryTarget {
    QueryTarget::Literal {
        value: LiteralValue::Integer { value },
    }
}

fn query_request(
    snapshot: &ArtifactSnapshot,
    relation: QueryRelation,
    target: QueryTarget,
    kinds: &[ConsumerKind],
    max_items: u64,
    scope: PhysicalScope,
) -> QueryRequest {
    QueryRequest {
        relation,
        target,
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope,
        },
        consumers: ConsumerSchema::new(1, kinds.to_vec()),
        max_items,
        cursor: None,
    }
}

fn run(snapshot: &ArtifactSnapshot, request: &QueryRequest, limits: Limits) -> QueryReport {
    let mut budget = Budget::new(limits);
    Engine::new()
        .query(snapshot, request, &mut budget)
        .expect("generated fixture queries must return a report")
}

fn snapshot_all() -> PhysicalScope {
    PhysicalScope::SnapshotAll
}

/// Multiset equality, because the combined schema's item order may interleave categories
/// differently from the concatenated single-category runs.
fn assert_same_item_multiset(left: &[XrefItem], right: &[XrefItem], context: &str) {
    assert_eq!(left.len(), right.len(), "{context}: item counts differ");
    let mut remaining: Vec<&XrefItem> = right.iter().collect();
    for item in left {
        let position = remaining
            .iter()
            .position(|candidate| *candidate == item)
            .unwrap_or_else(|| {
                panic!("{context}: the combined run reported an item no single category produced: {item:?}")
            });
        remaining.remove(position);
    }
    assert!(remaining.is_empty(), "{context}: an item is missing");
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

/// 1. Pagination splice equivalence.
fn pagination_property(shape: &Shape) {
    let built = build_class(shape, 0);
    let snapshot = open(built.bytes.clone());
    let target = method_target(&built.targets[0].0, &built.targets[0].1);
    let request = |max_items: u64| {
        query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &[ConsumerKind::Invocation],
            max_items,
            snapshot_all(),
        )
    };

    let full = run(&snapshot, &request(0), limits());
    assert!(
        matches!(full.execution, ExecutionReport::Complete { .. }),
        "the full run must complete: {:?}",
        full.execution
    );
    assert_eq!(
        full.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert!(
        full.items.len() >= 3,
        "the forced invocations must make the page sequence non-vacuous: {:?}",
        full.items
    );
    assert_eq!(
        full.items
            .iter()
            .map(|item| item.evidence.bci)
            .collect::<Vec<_>>(),
        built.invoke_bcis[0]
            .iter()
            .copied()
            .map(Some)
            .collect::<Vec<_>>(),
        "one item per invocation of this member, in instruction order"
    );

    let mut cursor = None;
    let mut spliced: Vec<XrefItem> = Vec::new();
    let mut pages = 0_u32;
    // Whether the page sequence reached the end of the range. The generated shape always
    // does — ample limits, page sizes of at least one item, and a standalone CLASS charges
    // no provider `ResultItems` — but the assertions below follow the public contract
    // instead of that shape: a legal stop that publishes nothing more and hands out no
    // cursor (`has_more` with no cursor) is not a splice failure, it is a run that proves
    // less, and the prefix it did publish must still be the full run's prefix.
    let mut completed = false;
    loop {
        pages += 1;
        assert!(pages <= 64, "pagination must terminate");
        let page_size = u64::from(shape.page_sizes[(pages as usize - 1) % shape.page_sizes.len()]);
        // Alternate a page-limited page with a budget-limited page: a page that stops
        // because `max_items` was reached and one that stops because its result-item
        // budget ran out must splice the same way.
        let result_limit = if pages % 2 == 1 {
            page_size
        } else {
            page_size.saturating_sub(1).max(1)
        };
        let mut page_request = request(page_size);
        page_request.cursor = cursor.clone();
        let mut page_limits = limits();
        page_limits.result_items = result_limit;
        let report = run(&snapshot, &page_request, page_limits);
        assert_eq!(
            u64::try_from(report.items.len()).expect("page length fits u64"),
            report.page.returned_items,
            "the page reports its own item count"
        );
        spliced.extend(report.items.iter().cloned());
        match report.page.cursor.clone() {
            Some(next) => {
                assert!(
                    report.page.has_more,
                    "a page that hands out a cursor stopped before the end"
                );
                assert!(
                    next.engine_schema == QUERY_ENGINE_SCHEMA,
                    "the cursor binds the engine schema"
                );
                assert_eq!(next.target, page_request.target);
                cursor = Some(next);
            }
            None => {
                match &report.execution {
                    ExecutionReport::Complete { .. } => {
                        assert!(
                            !report.page.has_more,
                            "a completed scan published every item it found"
                        );
                        completed = true;
                    }
                    interrupted => {
                        // The contract's other terminal shape: the range is not exhausted
                        // and no cursor was issued, so the caller has to repeat the same
                        // request. Only what still holds is asserted — what was published
                        // is a reliable prefix of the one full run, in order and without
                        // repeats.
                        assert!(
                            report.page.has_more,
                            "a stop without a continuation is not the end of the range: \
                             {interrupted:?}"
                        );
                        assert!(
                            spliced.len() <= full.items.len(),
                            "a stopped sequence cannot hold more items than the full run"
                        );
                        assert_eq!(
                            spliced.as_slice(),
                            &full.items[..spliced.len()],
                            "a stopped page sequence must still be the full run's prefix"
                        );
                    }
                }
                break;
            }
        }
    }
    if completed {
        assert_eq!(
            spliced, full.items,
            "paged continuations must splice to the one full run"
        );
    }
}

/// 2. Consumer-category combination consistency.
fn category_combination_property(shape: &Shape) {
    let built = build_class(shape, 0);
    let snapshot = open(built.bytes.clone());
    let candidates = [
        ConsumerKind::Invocation,
        ConsumerKind::Field,
        ConsumerKind::Type,
        ConsumerKind::Constant,
        ConsumerKind::Exception,
        ConsumerKind::Signature,
        ConsumerKind::Annotation,
        ConsumerKind::InnerNest,
        ConsumerKind::Bootstrap,
        ConsumerKind::Resource,
        ConsumerKind::Module,
    ];
    let chosen = |offset: usize, len: usize| -> Vec<ConsumerKind> {
        (0..len)
            .map(|index| candidates[(offset + index) % candidates.len()])
            .collect()
    };

    // The symbol query is non-vacuous: its own category answers the forced invocations.
    let target = method_target(&built.targets[0].0, &built.targets[0].1);
    let invocation_only = run(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &[ConsumerKind::Invocation],
            0,
            snapshot_all(),
        ),
        limits(),
    );
    assert!(invocation_only.items.len() >= 3);
    assert!(invocation_only.items.iter().all(|item| {
        item.consumer == Some(ConsumerKind::Invocation)
            && item.operation == XrefOperation::InvokeVirtual
    }));

    let mut symbol_kinds = chosen(
        usize::from(shape.suffix) % candidates.len(),
        1 + shape.instructions.len() % 3,
    );
    // Keep the combination non-vacuous: the symbol query's own category must be in the
    // schema, otherwise an empty result would satisfy the equivalence trivially.
    if !symbol_kinds.contains(&ConsumerKind::Invocation) {
        symbol_kinds.push(ConsumerKind::Invocation);
    }
    let symbol_union = combine_runs(
        &snapshot,
        QueryRelation::MentionsSymbol,
        target,
        &symbol_kinds,
        "symbol query",
    );
    assert!(
        !symbol_union.is_empty(),
        "the combined run reports the invocations"
    );

    // The literal query is non-vacuous as well: the first string is loaded once.
    let literal = string_target(&built.string_values[0]);
    let literal_only = run(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::LiteralValue,
            literal.clone(),
            &[ConsumerKind::Constant],
            0,
            snapshot_all(),
        ),
        limits(),
    );
    assert!(
        !literal_only.items.is_empty(),
        "the first string is loaded by the forced instruction shape"
    );
    assert!(literal_only.items.iter().all(|item| {
        item.consumer == Some(ConsumerKind::Constant) && item.operation == XrefOperation::Ldc
    }));

    let mut literal_kinds = chosen(
        usize::from(shape.suffix) % candidates.len(),
        1 + shape.integers % 3,
    );
    if !literal_kinds.contains(&ConsumerKind::Constant) {
        literal_kinds.push(ConsumerKind::Constant);
    }
    let literal_union = combine_runs(
        &snapshot,
        QueryRelation::LiteralValue,
        literal,
        &literal_kinds,
        "literal query",
    );
    assert!(
        !literal_union.is_empty(),
        "the combined literal run reports the forced load"
    );
}

/// Runs one combined request and every single-category request, and asserts the combined
/// items are exactly the multiset union of the single-category items.
///
/// The relation is justified by the scan contract: one item is one structural fact, a
/// fact has exactly one producing consumer category, and the combined schema enables the
/// union of the producers. Item order is pinned elsewhere (goldens), so this property
/// compares the sets of facts rather than their interleaving.
fn combine_runs(
    snapshot: &ArtifactSnapshot,
    relation: QueryRelation,
    target: QueryTarget,
    kinds: &[ConsumerKind],
    context: &str,
) -> Vec<XrefItem> {
    let combined = run(
        snapshot,
        &query_request(snapshot, relation, target.clone(), kinds, 0, snapshot_all()),
        limits(),
    );
    assert!(
        combined.coverage.unsupported_categories.is_empty(),
        "{context}: the candidates are all implemented categories"
    );
    let mut union = Vec::new();
    for kind in kinds {
        let single = run(
            snapshot,
            &query_request(
                snapshot,
                relation,
                target.clone(),
                &[*kind],
                0,
                snapshot_all(),
            ),
            limits(),
        );
        for item in &single.items {
            assert_eq!(
                item.consumer,
                Some(*kind),
                "{context}: a single-category request reported another consumer's item: {item:?}"
            );
        }
        union.extend(single.items.iter().cloned());
    }
    assert_same_item_multiset(&combined.items, &union, context);
    combined.items
}

/// 3. Unused constant-pool insertion does not create X1 facts.
fn unused_pool_insertion_property(shape: &Shape) {
    let base = build_class(shape, 0);
    let extras = 1 + usize::from(shape.suffix % 3);
    let extended = build_class(shape, extras);
    assert!(
        extended.bytes.len() > base.bytes.len(),
        "the extended class really appends pool entries"
    );
    let base_snapshot = open(base.bytes.clone());
    let extended_snapshot = open(extended.bytes.clone());

    // Every fact the base class reports is still reported by the extended class. The
    // comparison is on facts, not on byte coordinates: the pool insertion moves every
    // byte after the pool, so class-file spans, offsets and the class digest necessarily
    // change while the reported facts must not.
    let commands: Vec<(QueryRelation, QueryTarget, Vec<ConsumerKind>)> = {
        let mut commands = Vec::new();
        for (owner, name) in &base.targets {
            commands.push((
                QueryRelation::MentionsSymbol,
                method_target(owner, name),
                vec![ConsumerKind::Invocation],
            ));
        }
        for (owner, name) in &base.unused {
            commands.push((
                QueryRelation::MentionsSymbol,
                method_target(owner, name),
                vec![ConsumerKind::Invocation],
            ));
        }
        for value in &base.string_values {
            commands.push((
                QueryRelation::LiteralValue,
                string_target(value),
                vec![ConsumerKind::Constant],
            ));
        }
        for value in &base.integer_values {
            commands.push((
                QueryRelation::LiteralValue,
                integer_target(*value),
                vec![ConsumerKind::Constant],
            ));
        }
        commands
    };
    for (relation, target, kinds) in commands {
        let base_report = run(
            &base_snapshot,
            &query_request(
                &base_snapshot,
                relation,
                target.clone(),
                &kinds,
                0,
                snapshot_all(),
            ),
            limits(),
        );
        let extended_report = run(
            &extended_snapshot,
            &query_request(
                &extended_snapshot,
                relation,
                target.clone(),
                &kinds,
                0,
                snapshot_all(),
            ),
            limits(),
        );
        let base_facts: Vec<FactKey> = base_report.items.iter().map(fact_key).collect();
        let extended_facts: Vec<FactKey> = extended_report.items.iter().map(fact_key).collect();
        assert_eq!(
            base_facts, extended_facts,
            "unused pool entries changed the facts for {target:?}"
        );
    }

    // The appended entries are new: the raw probe sees them in the extended class only,
    // and no consumer claims them as mentions.
    for (owner, name) in &extended.extras {
        let target = method_target(owner, name);
        let base_probe = run(
            &base_snapshot,
            &query_request(
                &base_snapshot,
                QueryRelation::ConstantPoolContains,
                target.clone(),
                &[ConsumerKind::Invocation],
                0,
                snapshot_all(),
            ),
            limits(),
        );
        assert!(
            base_probe.items.is_empty(),
            "the appended owner must not exist in the base class"
        );
        let extended_probe = run(
            &extended_snapshot,
            &query_request(
                &extended_snapshot,
                QueryRelation::ConstantPoolContains,
                target.clone(),
                &[ConsumerKind::Invocation],
                0,
                snapshot_all(),
            ),
            limits(),
        );
        assert!(
            !extended_probe.items.is_empty(),
            "the raw probe reports the appended candidate"
        );
        assert!(
            extended_probe
                .items
                .iter()
                .all(|item| item.consumer.is_none()
                    && item.derivation == XrefDerivation::ConstantPoolCandidate)
        );
        let extended_mentions = run(
            &extended_snapshot,
            &query_request(
                &extended_snapshot,
                QueryRelation::MentionsSymbol,
                target,
                &[
                    ConsumerKind::Invocation,
                    ConsumerKind::Field,
                    ConsumerKind::Type,
                    ConsumerKind::Constant,
                ],
                0,
                snapshot_all(),
            ),
            limits(),
        );
        assert!(
            extended_mentions.items.is_empty(),
            "an unused pool entry must not become an X1 fact: {:?}",
            extended_mentions.items
        );
    }
}

/// 4. Identical class bytes in different physical origins are never merged.
fn identical_bytes_keep_origins_property(shape: &Shape) {
    let built = build_class(shape, 0);
    let inner = zip_bytes(&[(b"p/Dup.class", &built.bytes, STORE)]);
    let root = zip_bytes(&[
        (b"p/DupRoot.class", &built.bytes, STORE),
        (b"lib/a.jar", &inner, STORE),
        (b"lib/b.jar", &inner, DEFLATE),
    ]);
    let snapshot = open(root);
    let target = method_target(&built.targets[0].0, &built.targets[0].1);

    let tree = run(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target.clone(),
            &[ConsumerKind::Invocation],
            0,
            PhysicalScope::ArtifactTree {
                root_container: ContainerId("root".into()),
            },
        ),
        limits(),
    );
    let invocation_bcis: Vec<Option<u32>> =
        built.invoke_bcis[0].iter().copied().map(Some).collect();
    let per_copy = invocation_bcis.len();
    assert!(per_copy >= 3);
    assert_eq!(
        tree.items.len(),
        3 * per_copy,
        "one root copy and two nested copies, each with its own items"
    );

    // Group the items by the physical entry they come from: three groups, never merged,
    // each holding the same complete, identically-ordered fact sequence.
    let mut groups: Vec<(PhysicalEntryId, Vec<&XrefItem>)> = Vec::new();
    for item in &tree.items {
        let definition = match &item.source.location {
            Location::Code { method, .. } => &method.owner,
            other => panic!("expected a code location, got {other:?}"),
        };
        let entry = definition
            .entry()
            .expect("every copy is an archive entry")
            .clone();
        match groups.iter_mut().find(|(id, _)| *id == entry) {
            Some((_, items)) => items.push(item),
            None => groups.push((entry, vec![item])),
        }
    }
    assert_eq!(
        groups.len(),
        3,
        "equal bytes must keep three distinct origins"
    );
    let expected_target = XrefTarget::Symbol {
        value: SymbolRef::Method {
            owner: JvmBytes(built.targets[0].0.clone()),
            name: JvmBytes(built.targets[0].1.clone()),
            descriptor: JvmBytes(b"()V".to_vec()),
        },
    };
    for (entry, items) in &groups {
        assert_eq!(items.len(), per_copy, "{entry:?}");
        assert_eq!(
            items
                .iter()
                .map(|item| item.evidence.bci)
                .collect::<Vec<_>>(),
            invocation_bcis,
            "{entry:?}: the same bytes answer at the same BCIs"
        );
        for item in items {
            assert_eq!(item.evidence.constant_pool_index, Some(10));
            assert_eq!(item.evidence.opcode, Some(0xb6));
            assert_eq!(
                item.evidence.attribute.as_ref().map(|name| name.0.clone()),
                Some(b"Code".to_vec())
            );
            assert_eq!(item.target, expected_target);
        }
    }
    let class_bytes: Vec<ClassBytesId> = groups
        .iter()
        .map(|(entry, items)| {
            let definition = match &items[0].source.location {
                Location::Code { method, .. } => &method.owner,
                other => panic!("expected a code location, got {other:?}"),
            };
            assert_eq!(
                definition.entry().expect("entry"),
                entry,
                "an item must belong to the group it was filed under"
            );
            definition.class_bytes.clone()
        })
        .collect();
    assert!(
        class_bytes.windows(2).all(|pair| pair[0] == pair[1]),
        "the copies really are the same bytes: {class_bytes:?}"
    );
    let steps: Vec<usize> = groups
        .iter()
        .map(|(entry, _)| entry.origin.steps.len())
        .collect();
    assert_eq!(
        steps.iter().filter(|count| **count == 0).count(),
        1,
        "one root copy: {steps:?}"
    );
    assert_eq!(
        steps.iter().filter(|count| **count == 1).count(),
        2,
        "two nested copies: {steps:?}"
    );
    let nested_ordinals: Vec<u64> = groups
        .iter()
        .filter(|(entry, _)| entry.origin.steps.len() == 1)
        .map(|(entry, _)| entry.origin.steps[0].via_ordinal)
        .collect();
    assert_ne!(
        nested_ordinals[0], nested_ordinals[1],
        "the nested copies come from different parent entries"
    );

    // The root-scope query reads only the root copy; nested copies are not silently
    // merged into it either.
    let flat = run(
        &snapshot,
        &query_request(
            &snapshot,
            QueryRelation::MentionsSymbol,
            target,
            &[ConsumerKind::Invocation],
            0,
            snapshot_all(),
        ),
        limits(),
    );
    assert_eq!(flat.items.len(), per_copy);
    for item in &flat.items {
        match &item.source.location {
            Location::Code { method, .. } => {
                let entry = method.owner.entry().expect("root entry");
                assert_eq!(entry.ordinal, 0);
                assert!(entry.origin.steps.is_empty());
            }
            other => panic!("expected a code location, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Coordinate-free fact projection
// ---------------------------------------------------------------------------

/// One reported fact without the coordinates the insertion legitimately moves.
#[derive(Debug, PartialEq)]
struct FactKey {
    relation: QueryRelation,
    target: XrefTarget,
    consumer: Option<ConsumerKind>,
    operation: XrefOperation,
    derivation: XrefDerivation,
    certainty: XrefCertainty,
    resolution: QueryResolution,
    constant_pool_index: Option<u16>,
    bci: Option<u32>,
    opcode: Option<u8>,
    attribute: Option<Vec<u8>>,
    via: Vec<BootstrapVia>,
    location: FactLocation,
}

#[derive(Debug, PartialEq)]
enum FactLocation {
    Container {
        id: ContainerId,
    },
    Entry {
        ordinal: u64,
        raw_name: Vec<u8>,
        steps: usize,
    },
    ClassOffset {
        origin: FactOrigin,
        variant: PhysicalVariant,
    },
    Code {
        origin: FactOrigin,
        variant: PhysicalVariant,
        name: Vec<u8>,
        descriptor: Vec<u8>,
    },
    Attribute {
        origin: FactOrigin,
        variant: PhysicalVariant,
        path: String,
    },
    Resource {
        ordinal: u64,
        raw_name: Vec<u8>,
        steps: usize,
    },
}

/// The physical origin of a class location without the snapshot identity.
///
/// A snapshot id is the blake3 digest of the whole byte sequence, so appending pool
/// entries necessarily changes it; the insertion property compares reported facts, not
/// container identity. The structural parts of the origin (entry ordinal, raw name,
/// nested step count) stay in the projection.
#[derive(Debug, PartialEq)]
enum FactOrigin {
    Standalone,
    ArchiveEntry {
        ordinal: u64,
        raw_name: Vec<u8>,
        steps: usize,
    },
}

fn fact_origin(location: &PhysicalClassLocation) -> FactOrigin {
    match location {
        PhysicalClassLocation::StandaloneRoot { .. } => FactOrigin::Standalone,
        PhysicalClassLocation::ArchiveEntry { entry } => FactOrigin::ArchiveEntry {
            ordinal: entry.ordinal,
            raw_name: entry.raw_name.0.clone(),
            steps: entry.origin.steps.len(),
        },
    }
}

fn fact_key(item: &XrefItem) -> FactKey {
    let location = match &item.source.location {
        Location::Container { id, .. } => FactLocation::Container { id: id.clone() },
        Location::Entry { id, .. } => FactLocation::Entry {
            ordinal: id.ordinal,
            raw_name: id.raw_name.0.clone(),
            steps: id.origin.steps.len(),
        },
        Location::ClassOffset { definition, .. } => FactLocation::ClassOffset {
            origin: fact_origin(&definition.location),
            variant: definition.variant.clone(),
        },
        Location::Code { method, .. } => FactLocation::Code {
            origin: fact_origin(&method.owner.location),
            variant: method.owner.variant.clone(),
            name: method.name.0.clone(),
            descriptor: method.descriptor.0.clone(),
        },
        Location::Attribute { owner, path, .. } => FactLocation::Attribute {
            origin: fact_origin(&owner.location),
            variant: owner.variant.clone(),
            path: path.clone(),
        },
        Location::Resource { entry, .. } => FactLocation::Resource {
            ordinal: entry.ordinal,
            raw_name: entry.raw_name.0.clone(),
            steps: entry.origin.steps.len(),
        },
    };
    FactKey {
        relation: item.relation,
        target: item.target.clone(),
        consumer: item.consumer,
        operation: item.operation,
        derivation: item.derivation,
        certainty: item.certainty,
        resolution: item.resolution,
        constant_pool_index: item.evidence.constant_pool_index,
        bci: item.evidence.bci,
        opcode: item.evidence.opcode,
        attribute: item.evidence.attribute.as_ref().map(|name| name.0.clone()),
        via: item.evidence.via.clone(),
        location,
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

    #[test]
    fn paged_continuations_splice_to_the_full_run(shape in shape_strategy()) {
        pagination_property(&shape);
    }

    #[test]
    fn combined_categories_equal_the_single_category_union(shape in shape_strategy()) {
        category_combination_property(&shape);
    }

    #[test]
    fn unused_constant_pool_insertion_keeps_mentions_unchanged(shape in shape_strategy()) {
        unused_pool_insertion_property(&shape);
    }

    #[test]
    fn identical_bytes_keep_distinct_origins(shape in shape_strategy()) {
        identical_bytes_keep_origins_property(&shape);
    }
}
