//! P1 boundary acceptance for high-fanout entries and the result-item budget.
//!
//! Design 3.3 requires this specific check: the scan orchestration accumulates one unit's
//! items (`unit_items`) *before* it charges `ResultItems` for publishing them, so a single
//! entry that mentions the requested target thousands of times is the case where a
//! "reliable prefix" could turn into unbounded construction. This file measures that
//! boundary through the public API only:
//!
//! 1. the published page is the reliable prefix of the same unpaged scan, and the stop
//!    names `ResultItems` while every other dimension stays inside its own small limit,
//! 2. the per-unit accumulation is bounded by the *input* dimensions the reader charges
//!    while decoding (`CodeBytes` per instruction), not by the result budget: a smaller
//!    code budget stops the unit at exactly the instructions it already paid for,
//! 3. pages taken with a larger budget and a page limit splice back to the unpaged scan
//!    without repeating or skipping an item, and the page that runs out of work claims
//!    completeness,
//! 4. cancellation at the unit boundary publishes nothing: a standalone root reports a
//!    cancelled scan, a one-entry archive aborts the request through the provider.
//!
//! The fixture is hand-written (no javac, no fixture files): one class with a single
//! `run()V` body holding `HITS` three-byte `invokevirtual` instructions that all name the
//! same `p/Target.hit:()V`, packed as the only entry of a STORED jar.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::io::{Cursor, Write};

/// Invocations in the high-fanout body: 6 144 bytes of code plus the `return`.
const HITS: usize = 2_048;

/// Code bytes the reader pays for this body: one 3-byte `invokevirtual` per hit plus the
/// 1-byte `return`.
const CODE_BYTES: u64 = (HITS as u64) * 3 + 1;

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
        nested_depth: 1,
        elapsed_millis: u64::MAX,
        ..Limits::default()
    }
}

/// Small limits around the one entry under test.
///
/// `code_bytes` is deliberately above [`CODE_BYTES`] in the prefix test: the whole body is
/// decoded there, so the only stopping dimension can be `ResultItems`.
fn small_limits() -> Limits {
    Limits {
        input_bytes: 64 * 1024,
        archive_entries: 8,
        entry_bytes: 32 * 1024,
        read_bytes: 64 * 1024,
        class_bytes: 32 * 1024,
        attribute_bytes: 16 * 1024,
        code_bytes: 16 * 1024,
        result_items: 1 << 20,
        output_bytes: 1 << 20,
        nested_depth: 1,
        elapsed_millis: u64::MAX,
        ..Limits::default()
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
            u16::try_from(text.len()).expect("fixture text fits u16"),
        );
        entry.extend_from_slice(text);
        self.push(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.push(entry)
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        let mut entry = vec![12];
        u16b(&mut entry, name);
        u16b(&mut entry, descriptor);
        self.push(entry)
    }

    fn method_ref(&mut self, class: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![10];
        u16b(&mut entry, class);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    fn bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }
}

/// One class whose single `run()V` body invokes `p/Target.hit:()V` exactly `hits` times.
fn class_bytes(hits: usize) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_name = pool.utf8(b"p/Bounds");
    let this_class = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let super_class = pool.class(object_name);
    let run = pool.utf8(b"run");
    let void_descriptor = pool.utf8(b"()V");
    let code_attribute = pool.utf8(b"Code");
    let target_name = pool.utf8(b"p/Target");
    let target_owner = pool.class(target_name);
    let hit = pool.utf8(b"hit");
    let hit_and_type = pool.name_and_type(hit, void_descriptor);
    let target = pool.method_ref(target_owner, hit_and_type);

    let mut code = Vec::with_capacity(hits * 3 + 1);
    for _ in 0..hits {
        code.push(0xb6); // invokevirtual p/Target.hit:()V
        u16b(&mut code, target);
    }
    code.push(0xb1); // return

    let mut body = Vec::new();
    u16b(&mut body, 1); // max_stack
    u16b(&mut body, 1); // max_locals (`this`)
    u32b(
        &mut body,
        u32::try_from(code.len()).expect("fixture code fits u32"),
    );
    body.extend_from_slice(&code);
    u16b(&mut body, 0); // exception table
    u16b(&mut body, 0); // code attributes

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, 52); // major: Java 8
    u16b(
        &mut bytes,
        u16::try_from(pool.entries.len() + 1).expect("fixture pool fits u16"),
    );
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, 0x0021); // public super
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, super_class);
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    u16b(&mut bytes, 1); // methods
    u16b(&mut bytes, 0x0001); // public
    u16b(&mut bytes, run);
    u16b(&mut bytes, void_descriptor);
    u16b(&mut bytes, 1); // method attributes
    u16b(&mut bytes, code_attribute);
    u32b(
        &mut bytes,
        u32::try_from(body.len()).expect("fixture body fits u32"),
    );
    bytes.extend_from_slice(&body);
    u16b(&mut bytes, 0); // class attributes
    bytes
}

/// A STORED jar holding exactly the given entries.
fn jar_bytes(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .expect("fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer.write_all(data).expect("fixture payload writes");
            let (_, descriptor) = writer.finish().expect("fixture payload finishes");
            entry.finish(descriptor).expect("fixture entry finishes");
        }
        archive.finish().expect("fixture archive finishes");
    }
    output.into_inner()
}

/// The high-fanout class as the only entry of a jar: one scan unit, one entry.
fn high_fanout_jar() -> Vec<u8> {
    jar_bytes(&[(b"p/Bounds.class".as_slice(), class_bytes(HITS).as_slice())])
}

fn open(input: Vec<u8>) -> ArtifactSnapshot {
    let mut budget = Budget::new(limits());
    Engine::new()
        .open(ArtifactInput::bytes(input), &mut budget)
        .expect("the hand-written fixture must open")
}

/// One symbol, one consumer category, no page limit: the request under test.
fn request(snapshot: &ArtifactSnapshot, max_items: u64) -> QueryRequest {
    QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol {
            value: SymbolRef::Method {
                owner: JvmBytes(b"p/Target".to_vec()),
                name: JvmBytes(b"hit".to_vec()),
                descriptor: JvmBytes(b"()V".to_vec()),
            },
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Invocation]),
        max_items,
        cursor: None,
    }
}

fn run(snapshot: &ArtifactSnapshot, request: &QueryRequest, limits: Limits) -> QueryReport {
    let mut budget = Budget::new(limits);
    Engine::new()
        .query(snapshot, request, &mut budget)
        .expect("the fixture queries must return a report")
}

fn usage_of(execution: &ExecutionReport) -> &UsageSnapshot {
    match execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage,
    }
}

/// Every counted dimension stays inside the limit it was given, and both high-water depths
/// do too.
///
/// The counted set is read through [`CountedBudgetDimension::ALL`] and the by-value
/// accessors instead of one hand-written assertion per field: a dimension added to the budget
/// then joins this bound on its own, where a fixed list would keep compiling while quietly
/// skipping it. The bound also tightens by itself here, because this file leaves the P2
/// dimensions at a limit of zero: the moment a scan below charges one of them, these
/// assertions fail instead of passing unseen.
///
/// `elapsed_millis` is deliberately not compared: it is a measurement, not a charge, and
/// exceeding it is what ends a run.
fn assert_within(usage: &UsageSnapshot, limits: &Limits) {
    for dimension in CountedBudgetDimension::ALL {
        let used = usage.counted_usage(dimension);
        let allowed = limits.counted_limit(dimension);
        assert!(
            used <= allowed,
            "{dimension:?} crossed the limit: usage {used} > limit {allowed}"
        );
    }
    assert!(
        usage.nested_depth <= limits.nested_depth,
        "nested_depth crossed the limit: usage {} > limit {}",
        usage.nested_depth,
        limits.nested_depth
    );
    assert!(
        usage.dependency_depth <= limits.dependency_depth,
        "dependency_depth crossed the limit: usage {} > limit {}",
        usage.dependency_depth,
        limits.dependency_depth
    );
}

/// Serde name of one budget dimension — the key the usage snapshot publishes it under.
///
/// The two are one contract: `UsageSnapshot`'s fields carry the dimension names verbatim, so
/// the self-check below can address a dimension by that name instead of restating 18 field
/// names in a second list that could fall behind.
fn dimension_field(dimension: BudgetDimension) -> String {
    serde_json::to_string(&dimension)
        .expect("a budget dimension serializes")
        .trim_matches('"')
        .to_string()
}

/// Usage with `value` on exactly one dimension and every other dimension at zero.
fn usage_with(dimension: BudgetDimension, value: u64) -> UsageSnapshot {
    let mut fields = serde_json::to_value(UsageSnapshot::default()).expect("usage serializes");
    fields
        .as_object_mut()
        .expect("the usage snapshot is a JSON object")
        .insert(dimension_field(dimension), serde_json::Value::from(value));
    serde_json::from_value(fields).expect("the usage snapshot round-trips")
}

/// Every dimension a usage snapshot publishes: the counted set first, then the two
/// high-water depths, then the clock.
fn published_dimensions() -> Vec<BudgetDimension> {
    let mut dimensions = CountedBudgetDimension::ALL
        .iter()
        .map(|dimension| BudgetDimension::from(*dimension))
        .collect::<Vec<_>>();
    dimensions.push(BudgetDimension::NestedDepth);
    dimensions.push(BudgetDimension::DependencyDepth);
    dimensions.push(BudgetDimension::ElapsedMillis);
    dimensions
}

/// Self-check of [`assert_within`]: the bound must reject an over-limit usage for **every**
/// dimension it claims to cover.
///
/// The runs above keep the P2 dimensions at zero, so a traversal that dropped one of them —
/// or the hand-written list this file used before 1.3 — would leave every other assertion in
/// this file green: the bound would get quieter without turning red. Handing the bound one
/// usage that is over exactly one dimension's limit makes that failure name its dimension.
/// The same loop pins the check's extent against the published usage schema, so a dimension
/// added to `UsageSnapshot` later cannot stay out of either the schema or the traversal
/// without failing here.
#[test]
fn the_within_bound_rejects_every_dimension_over_its_limit() {
    let bounds = limits();
    let dimensions = published_dimensions();
    let schema = serde_json::to_value(UsageSnapshot::default()).expect("usage serializes");
    let schema = schema
        .as_object()
        .expect("the usage snapshot is a JSON object");
    assert_eq!(
        schema.len(),
        dimensions.len(),
        "the checked dimension list must name every published usage field"
    );

    for dimension in &dimensions {
        assert!(
            schema.contains_key(&dimension_field(*dimension)),
            "{dimension:?} does not name a published usage field"
        );

        if *dimension == BudgetDimension::ElapsedMillis {
            // Published but not bounded: the clock is a measurement, not a charge, so this
            // bound lets it pass even against a zero `elapsed_millis` limit. That is the one
            // dimension left uncovered on purpose; every dimension below must be rejected
            // once it is over its limit.
            let no_clock = Limits {
                elapsed_millis: 0,
                ..bounds.clone()
            };
            assert_within(&usage_with(*dimension, 1), &no_clock);
            continue;
        }

        let allowed = match CountedBudgetDimension::try_from(*dimension) {
            Ok(counted) => bounds.counted_limit(counted),
            Err(()) => match dimension {
                BudgetDimension::NestedDepth => bounds.nested_depth,
                BudgetDimension::DependencyDepth => bounds.dependency_depth,
                other => panic!("{other:?} is neither a counted nor a high-water dimension"),
            },
        };
        assert!(
            allowed < u64::MAX,
            "these limits must leave room to exceed {dimension:?}"
        );

        // Exactly at the limit is inside it: the bound is `usage <= limit`, not `<`.
        assert_within(&usage_with(*dimension, allowed), &bounds);

        let over = allowed + 1;
        assert!(
            std::panic::catch_unwind(|| assert_within(&usage_with(*dimension, over), &bounds))
                .is_err(),
            "{dimension:?} is over its limit ({over} > {allowed}) and must be rejected"
        );
    }

    // Not a blanket rejection either: the zero usage sits inside every limit this file sets,
    // including the P2 dimensions it leaves at zero.
    assert_within(&UsageSnapshot::default(), &bounds);
}

/// The unpaged reference run: every hit is a published item.
fn unpaged(snapshot: &ArtifactSnapshot) -> QueryReport {
    let report = run(snapshot, &request(snapshot, 0), limits());
    assert_eq!(
        report.items.len(),
        HITS,
        "the fixture must publish one item per invocation"
    );
    assert!(
        matches!(report.execution, ExecutionReport::Complete { .. }),
        "the unbounded run must complete: {:?}",
        report.execution
    );
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::CompleteWithinSchema
    );
    assert_eq!(
        report.coverage.scanned_items,
        u64::try_from(HITS).expect("hit count fits u64")
    );
    assert_eq!(report.coverage.unknown_candidates, 0);
    report
}

#[test]
fn result_budget_publishes_the_reliable_prefix_of_a_high_fanout_entry() {
    let snapshot = open(high_fanout_jar());
    let full = unpaged(&snapshot);

    const BUDGET: u64 = 16;
    let tight = Limits {
        result_items: BUDGET,
        ..small_limits()
    };
    let bounded = run(&snapshot, &request(&snapshot, 0), tight.clone());

    // The public budget bounds what is published, and the page describes itself.
    // The budget is shared with the physical provider: the one enumerated entry already
    // cost one result item (the P0 rule "each returned item costs one `ResultItems`
    // before it is published"), so the starved page publishes `BUDGET - 1` query items.
    const ENUMERATED_ENTRIES: u64 = 1;
    let expected_items = BUDGET - ENUMERATED_ENTRIES;
    assert_eq!(
        u64::try_from(bounded.items.len()).expect("page length fits u64"),
        expected_items,
        "the enumerated entry and the published items must spend exactly the budget"
    );
    assert_eq!(bounded.page.returned_items, expected_items);
    assert_eq!(
        bounded.items,
        full.items[..usize::try_from(expected_items).expect("budget fits usize")],
        "the starved page is the reliable prefix of the same scan"
    );
    match &bounded.execution {
        ExecutionReport::Partial { reason, usage } => {
            assert_eq!(
                reason,
                &TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ResultItems
                },
                "the stop names the exhausted dimension"
            );
            assert_eq!(usage.result_items, BUDGET);
        }
        other => panic!("an exhausted item budget is a partial execution, got {other:?}"),
    }
    // No other dimension is pushed past the small limits it was given.
    assert_within(usage_of(&bounded.execution), &tight);
    assert!(
        usage_of(&bounded.execution).code_bytes > 0,
        "the page still paid for the code it read"
    );
    assert_eq!(
        bounded.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial,
        "a stopped scan never claims completeness"
    );
    assert!(
        bounded
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "budget_exceeded_result_items"),
        "the terminal diagnostic explains the stop: {:?}",
        bounded.diagnostics
    );
    assert!(
        bounded.page.has_more && bounded.page.cursor.is_some(),
        "a page that stopped mid-entry keeps a continuation"
    );

    // The buffer measurement behind the design question: publication stopped after 16
    // items, yet the reader charged the *whole* body, so the per-unit accumulation really
    // happened ahead of the `ResultItems` charge. It stays a function of the paid input
    // dimensions (`CodeBytes` per decoded instruction), which is what the next test pins
    // from the other side — the result budget does not constrain construction, but the
    // input budgets do, and neither the buffer nor the page can grow past them.
    assert_eq!(
        usage_of(&bounded.execution).code_bytes,
        CODE_BYTES,
        "the whole body was decoded and charged before publication stopped"
    );
}

#[test]
fn the_per_unit_accumulation_is_bounded_by_the_input_dimensions() {
    let snapshot = open(high_fanout_jar());
    let full = unpaged(&snapshot);

    // Two thirds of the body: 1 024 paid invocations. The result budget is large, so only
    // the code bytes can stop this run.
    let paid_hits = 1_024_u64;
    let code_budget = paid_hits * 3;
    let tight = Limits {
        code_bytes: code_budget,
        ..small_limits()
    };
    let report = run(&snapshot, &request(&snapshot, 0), tight.clone());

    match &report.execution {
        ExecutionReport::Partial { reason, usage } => {
            assert_eq!(
                reason,
                &TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::CodeBytes
                },
                "the stop names the input dimension that ran out"
            );
            assert_eq!(
                usage.code_bytes, code_budget,
                "every paid code byte is charged exactly once"
            );
        }
        other => panic!("an exhausted code budget is a partial execution, got {other:?}"),
    }
    assert_within(usage_of(&report.execution), &tight);
    // The accumulation is bounded by the bytes the reader already charged: one item per
    // three paid code bytes here, never more.
    let published = u64::try_from(report.items.len()).expect("page length fits u64");
    assert_eq!(
        published, paid_hits,
        "the unit publishes exactly the instructions it paid for"
    );
    assert!(
        published * 3 <= usage_of(&report.execution).code_bytes,
        "no item is constructed without paying for its instruction"
    );
    assert_eq!(
        report.items,
        full.items[..usize::try_from(published).expect("count fits usize")],
        "the stop inside one entry still publishes the reliable prefix"
    );
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(
        report.page.has_more,
        "the entry was not examined to its end"
    );
}

#[test]
fn high_fanout_pages_splice_back_to_the_whole_scan() {
    let snapshot = open(high_fanout_jar());
    let full = unpaged(&snapshot);

    const PAGE: u64 = 128;
    let mut cursor = None;
    let mut collected: Vec<XrefItem> = Vec::new();
    let mut pages = 0_u32;
    loop {
        pages += 1;
        assert!(pages <= 64, "pagination must terminate");
        let mut page_request = request(&snapshot, PAGE);
        page_request.cursor = cursor.clone();
        // A budget far above the page size: every page stops on its own item limit, so
        // the splice is not accidentally hiding a budget stop.
        let page = run(&snapshot, &page_request, limits());
        assert_eq!(
            page.page.returned_items,
            u64::try_from(page.items.len()).expect("page length fits u64")
        );
        assert!(
            page.page.returned_items <= PAGE,
            "a page never exceeds its own item limit"
        );
        assert!(
            matches!(page.execution, ExecutionReport::Complete { .. }),
            "a page limit with a sufficient budget is not a degraded execution: {:?}",
            page.execution
        );
        let start = collected.len();
        let length = page.items.len();
        collected.extend(page.items.iter().cloned());
        assert_eq!(
            collected[start..],
            full.items[start..start + length],
            "each page carries the next items of the unpaged scan"
        );
        match page.page.cursor.clone() {
            Some(next) => {
                assert!(page.page.has_more);
                assert_eq!(
                    page.coverage.dimensions.artifact_structural.state,
                    CoverageState::Partial,
                    "a page that stopped at its limit does not claim completeness"
                );
                cursor = Some(next);
            }
            None => {
                assert!(
                    !page.page.has_more,
                    "a page that ran out of work has no continuation"
                );
                assert_eq!(
                    page.coverage.dimensions.artifact_structural.state,
                    CoverageState::CompleteWithinSchema,
                    "the page that examined every unit and ran out of work claims completeness"
                );
                break;
            }
        }
    }
    assert_eq!(
        pages,
        u32::try_from(HITS / usize::try_from(PAGE).expect("page fits usize")).expect("page count"),
        "the whole body is covered by whole pages"
    );
    assert_eq!(
        collected, full.items,
        "the pages must not repeat or skip items"
    );
}

#[test]
fn cancellation_at_the_high_fanout_unit_boundary_publishes_nothing() {
    // The standalone CLASS root reports the cancelled scan itself.
    let class = class_bytes(HITS);
    let mut budget = Budget::new(limits());
    let snapshot = Engine::new()
        .open(ArtifactInput::bytes(class.clone()), &mut budget)
        .expect("the class fixture must open");
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = Engine::new()
        .query(&snapshot, &request(&snapshot, 0), &mut budget)
        .expect("a cancelled scan still returns a report");
    assert!(
        matches!(report.execution, ExecutionReport::Cancelled { .. }),
        "a cancelled scan is never complete: {:?}",
        report.execution
    );
    assert!(report.items.is_empty());
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(
        report.page.has_more && report.page.cursor.is_none(),
        "the scan never reached the end and published no boundary"
    );
    assert_eq!(
        usage_of(&report.execution).result_items,
        0,
        "nothing was published, so nothing was billed"
    );

    // The same bytes as the only entry of an archive: the provider's own enumeration is
    // cancelled too, so the entry is never established and the scan reports the
    // cancellation with an empty published prefix (the skipped ordinal stays visible in
    // the coverage ranges instead of being reported as complete).
    let snapshot = open(high_fanout_jar());
    let token = CancellationToken::new();
    token.cancel();
    let mut budget = Budget::with_cancellation_token(limits(), token);
    let report = Engine::new()
        .query(&snapshot, &request(&snapshot, 0), &mut budget)
        .expect("a cancelled enumeration is reported, not returned as an error");
    assert!(
        matches!(report.execution, ExecutionReport::Cancelled { .. }),
        "a cancelled enumeration is never complete: {:?}",
        report.execution
    );
    assert!(report.items.is_empty());
    assert_eq!(
        report.coverage.dimensions.artifact_structural.state,
        CoverageState::Partial
    );
    assert!(
        !report
            .coverage
            .dimensions
            .artifact_structural
            .skipped
            .is_empty(),
        "the entry the cancelled enumeration never established stays skipped"
    );
    assert!(
        report.page.has_more && report.page.cursor.is_none(),
        "the scan never reached the end and published no boundary"
    );
    assert_eq!(usage_of(&report.execution).result_items, 0);
}
