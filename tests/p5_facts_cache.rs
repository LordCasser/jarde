//! P5 task 2.3: the CP/Header facts cache — its identity, its invalidation, and the direct-path
//! fallback the `facts-cache` spec requires of it.
//!
//! P5 2.1 measured the direct path and 2.2 recorded the key dimensions a cache would have to bind.
//! This file holds the half a record cannot: that the layer built under that record **answers with
//! the facts the direct path produces**, that each dimension of its identity can be turned
//! separately, and that a disabled, discarded or refused cache leaves the request it was attached
//! to exactly where the direct path leaves it.
//!
//! What is asserted here, and where
//! --------------------------------
//!
//! * **The value, not the counter.** Every "the cache answered" assertion is paired with an
//!   equality against a parse of the same bytes ([`upheld`]): a hit is worth as much as the value
//!   it hands back, and a comparison that only counted hits would accept any wrong answer.
//! * **One dimension at a time.** `the_key_binds_content_policy_and_declaration` flips the content,
//!   the parse policy, the entry format and the registry version one by one, so a key that dropped
//!   any of them fails exactly one case.
//! * **The refusals.** A parse that stops — on the budget, on a cancellation, on the strict version
//!   gate — writes nothing here, which is what makes "an incomplete answer never stands in for a
//!   complete one" and "a negative verdict is never replayed after the reason for it changed" true
//!   by construction rather than by a check that could be forgotten.
//! * **The budget.** The fallback runs under the budget the request already holds: a discarded entry
//!   under a limit the direct path cannot afford stops in the same dimension with the same numbers
//!   as a run with no cache at all, and a hit still polls (a cancelled request is never answered from
//!   memory) and still bills what it publishes.
//!
//! ```text
//! verify: cargo test --test p5_facts_cache --locked
//! ```
//!
//! What this file is not
//! --------------------
//!
//! The **cold/warm comparison** of a whole published result, its fingerprint and its resource
//! deltas lives in `tests/p5_benchmark.rs`, over the rows and the comparison report of tasks 1.2 and
//! 1.3: that is where a second path is judged. This file judges the layer itself, through the
//! reader's and the engine's own entry points. The **container** product of the same store — the
//! directed access, its raw-name locator, its dual bounds and the request limits a hit obeys — is
//! judged the same way in `tests/p5_container_lookup.rs`.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use std::collections::BTreeSet;
use std::io::{Cursor, Write};

const STORE: u16 = 0;

/// The archive A15's corpus list names, and the class the recovery dimension is carried by. Both are
/// the bytes the P5 corpus fingerprint pins.
const ARCHIVE: &[u8] = include_bytes!("../fuzz/corpus/query/minimal-jar");
const CONTROL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");

/// Headroom for one request, the same shape the benchmark's rows start from.
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
        class_headers: 1_000,
        method_bodies: 1_000,
        ir_items: 1 << 20,
        ir_edges: 1 << 20,
        analysis_steps: 1 << 20,
        normalization_clones: 1 << 20,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

/// The same headroom with two dimensions replaced: how a test states "this request cannot afford
/// the parse" or "this request cannot publish a diagnostic" without touching anything else.
fn limits_with(class_bytes: u64, result_items: u64) -> Limits {
    Limits {
        class_bytes,
        result_items,
        ..limits()
    }
}

/// A fresh cache holding up to `capacity` entries, read under this build's identity.
///
/// The byte bound is left untouched (`u64::MAX`): these cases are about the entry limit, and the
/// product of a class parse is always a handful of bytes.
fn cache(capacity: usize) -> FactsCache {
    FactsCache::current(FactsCapacity::new(capacity, u64::MAX))
}

/// The cheapest class file the reader accepts, with `this_class` set to `this_class`.
///
/// `major` is the only knob, and it is what makes a version fixture: two class files for one name
/// with different majors are two byte sequences, and a major above the registry's ceiling is one
/// this build refuses under `Strict` and reads under `Forensic`.
fn class_bytes(this_class: &[u8], major: u16) -> Vec<u8> {
    let object = b"java/lang/Object";
    let mut pool = Vec::new();
    pool.push(1_u8); // CONSTANT_Utf8 this_class
    pool.extend_from_slice(
        &u16::try_from(this_class.len())
            .expect("name fits u16")
            .to_be_bytes(),
    );
    pool.extend_from_slice(this_class);
    pool.extend_from_slice(&[7, 0, 1]); // CONSTANT_Class #1
    pool.push(1_u8); // CONSTANT_Utf8 "java/lang/Object"
    pool.extend_from_slice(
        &u16::try_from(object.len())
            .expect("object name fits u16")
            .to_be_bytes(),
    );
    pool.extend_from_slice(object);
    pool.extend_from_slice(&[7, 0, 3]); // CONSTANT_Class #3

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    bytes.extend_from_slice(&0_u16.to_be_bytes()); // minor
    bytes.extend_from_slice(&major.to_be_bytes());
    bytes.extend_from_slice(&5_u16.to_be_bytes()); // constant_pool_count
    bytes.extend_from_slice(&pool);
    bytes.extend_from_slice(&0x0021_u16.to_be_bytes()); // ACC_PUBLIC | ACC_SUPER
    bytes.extend_from_slice(&2_u16.to_be_bytes()); // this_class
    bytes.extend_from_slice(&4_u16.to_be_bytes()); // super_class
    for _ in 0..4 {
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces, fields, methods, attributes
    }
    bytes
}

/// A class file that declares **no** superclass: the end of a hierarchy the resolver can read.
///
/// [`class_bytes`] always names `java/lang/Object` as the parent, so a class *called*
/// `java/lang/Object` would be its own parent and the walk would refuse a cycle instead of ending.
/// This fixture is what a provider has to hold for a search to reach the top of a hierarchy.
fn superless_class_bytes(this_class: &[u8], major: u16) -> Vec<u8> {
    let mut bytes = class_bytes(this_class, major);
    // From the end of the builder's layout: attributes, methods, fields and interfaces are four
    // `u16`s, and `super_class` sits just before them. The assertion below is what keeps this from
    // silently rewriting another field if the builder grows.
    let super_class = bytes.len() - 10;
    assert_eq!(
        u16::from_be_bytes([bytes[super_class], bytes[super_class + 1]]),
        4,
        "the builder's super_class index moved, and this test would rewrite the wrong field"
    );
    bytes[super_class..super_class + 2].copy_from_slice(&0_u16.to_be_bytes());
    bytes
}

/// One stored ZIP with the given entries. Duplicates are allowed on purpose: two entries holding
/// the same class bytes are the origin fixture.
fn zip_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut archive = ZipArchiveWriter::new(&mut output);
        for (name, data) in entries {
            let (mut entry, config) = archive
                .new_file(EntryPath::verbatim(name.to_vec()))
                .compression_method(CompressionMethod::new(STORE))
                .start()
                .expect("the fixture entry starts");
            let mut writer = config.wrap(&mut entry);
            writer
                .write_all(data)
                .expect("the fixture entry is writable");
            let (_, descriptor) = writer.finish().expect("the fixture entry closes");
            entry
                .finish(descriptor)
                .expect("the fixture entry finishes");
        }
        archive.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut Budget::new(limits()))
        .expect("the fixture snapshot opens")
}

/// A snapshot opened on the same bytes through a budget that carries `cache`, so the open itself is
/// part of the request the cache is attached to.
fn open_with(bytes: Vec<u8>, cache: &FactsCache) -> ArtifactSnapshot {
    Engine::new()
        .open(
            ArtifactInput::bytes(bytes),
            &mut Budget::new(limits()).with_facts_cache(cache.clone()),
        )
        .expect("the fixture snapshot opens")
}

// ---------------------------------------------------------------------------------------------
// The equality a hit has to survive
// ---------------------------------------------------------------------------------------------

/// The facts of `bytes`, read with no cache attached.
fn direct(bytes: &[u8]) -> ClassFacts {
    class_facts(bytes, &mut Budget::new(limits())).expect("the fixture is a class file")
}

/// The facts of `bytes` as the cache answers them, with the report of what it did.
fn through(bytes: &[u8], cache: &FactsCache) -> (ClassFacts, FactsReport, UsageSnapshot) {
    let mut budget = Budget::new(limits()).with_facts_cache(cache.clone());
    let facts = class_facts(bytes, &mut budget).expect("the fixture is a class file");
    (facts, cache.report(), budget.usage())
}

/// The header of `bytes` under `mode`, read with no cache attached.
fn direct_header(bytes: &[u8], mode: InspectionMode) -> HeaderInspection {
    inspect_header(bytes, &mut Budget::new(limits()), mode).expect("the fixture's header reads")
}

/// The header of `bytes` under `mode` as the cache answers it.
fn header_through(
    bytes: &[u8],
    mode: InspectionMode,
    cache: &FactsCache,
) -> (HeaderInspection, FactsReport) {
    let mut budget = Budget::new(limits()).with_facts_cache(cache.clone());
    let inspection = inspect_header(bytes, &mut budget, mode).expect("the fixture's header reads");
    (inspection, cache.report())
}

/// The first half of "the cache answered": the value equals the direct path's value **and** the
/// store was really in the loop.
///
/// A value without a consultation would not show that the cache was asked at all; whether an
/// answer was a hit or a miss is asserted by each test that means it, because a cold read that is
/// equal is worth less than a hit that is.
fn upheld(
    expected: &ClassFacts,
    (value, report, _): (ClassFacts, FactsReport, UsageSnapshot),
) -> FactsReport {
    assert_eq!(
        &value, expected,
        "a cached answer differs from the facts the direct path reads from the same bytes"
    );
    assert!(
        report.consultations > 0,
        "the value was equal and the store was never consulted, so nothing was proven about the \
         cache: {report:?}"
    );
    report
}

/// Removes every `usage` object from a serialized report, in place.
///
/// The charge beside a run is the **resource** half of the planes task 1.3's report separates, and
/// comparing it across two configurations compares a saving rather than a result. Nothing else is
/// removed: the terminal statuses, the reasons, the evidence and the diagnostics stay, because those
/// are results.
fn without_charges(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.remove("usage");
            for child in map.values_mut() {
                without_charges(child);
            }
        }
        serde_json::Value::Array(entries) => {
            for child in entries.iter_mut() {
                without_charges(child);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------------------------

/// Every consumer category, so a full-range row is the broadest structural scan the relation admits.
const ALL_CONSUMERS: [ConsumerKind; 13] = [
    ConsumerKind::Invocation,
    ConsumerKind::Field,
    ConsumerKind::Type,
    ConsumerKind::Constant,
    ConsumerKind::Exception,
    ConsumerKind::Signature,
    ConsumerKind::Annotation,
    ConsumerKind::InnerNest,
    ConsumerKind::Module,
    ConsumerKind::Bootstrap,
    ConsumerKind::Resource,
    ConsumerKind::Verification,
    ConsumerKind::Debug,
];

fn full_range_request(snapshot: &ArtifactSnapshot) -> QueryRequest {
    QueryRequest {
        relation: QueryRelation::MentionsSymbol,
        target: QueryTarget::Symbol {
            value: SymbolRef::Class {
                owner: JvmBytes(b"java/lang/Object".to_vec()),
            },
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, ALL_CONSUMERS),
        max_items: 0,
        cursor: None,
    }
}

fn pool_probe_request(snapshot: &ArtifactSnapshot) -> QueryRequest {
    QueryRequest {
        relation: QueryRelation::ConstantPoolContains,
        target: QueryTarget::Symbol {
            value: SymbolRef::Class {
                owner: JvmBytes(b"java/lang/Object".to_vec()),
            },
        },
        physical: PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
        },
        consumers: ConsumerSchema::new(1, [ConsumerKind::Type]),
        max_items: 0,
        cursor: None,
    }
}

/// One query with no cache attached.
fn query(snapshot: &ArtifactSnapshot, request: &QueryRequest) -> (QueryReport, UsageSnapshot) {
    let mut budget = Budget::new(limits());
    let report = Engine::new()
        .query(snapshot, request, &mut budget)
        .expect("a legal query is answered, not raised");
    (report, budget.usage())
}

/// The same query with `cache` attached.
fn query_with(
    snapshot: &ArtifactSnapshot,
    request: &QueryRequest,
    cache: &FactsCache,
) -> (QueryReport, UsageSnapshot) {
    let mut budget = Budget::new(limits()).with_facts_cache(cache.clone());
    let report = Engine::new()
        .query(snapshot, request, &mut budget)
        .expect("a legal query is answered, not raised");
    (report, budget.usage())
}

/// A usage snapshot with the wall clock zeroed: the charge form two runs are compared in.
fn charges(usage: &UsageSnapshot) -> UsageSnapshot {
    let mut copy = usage.clone();
    copy.elapsed_millis = 0;
    copy
}

/// `usage.counted_usage(dimension)` by name, for messages and for the assertions below.
fn charged(usage: &UsageSnapshot, dimension: CountedBudgetDimension) -> u64 {
    usage.counted_usage(dimension)
}

/// The charged dimensions of one usage snapshot, as one line: the form two budgets are compared in.
fn charges_line(usage: &UsageSnapshot) -> String {
    let charged: Vec<String> = CountedBudgetDimension::ALL
        .iter()
        .filter(|dimension| usage.counted_usage(**dimension) != 0)
        .map(|dimension| {
            format!(
                "{}={}",
                serde_json::to_string(dimension)
                    .expect("a dimension serializes")
                    .trim_matches('"'),
                usage.counted_usage(*dimension)
            )
        })
        .collect();
    if charged.is_empty() {
        "nothing charged".to_string()
    } else {
        charged.join(" ")
    }
}

fn status_of(execution: &ExecutionReport) -> &'static str {
    match execution {
        ExecutionReport::Complete { .. } => "complete",
        ExecutionReport::Partial { .. } => "partial",
        ExecutionReport::Cancelled { .. } => "cancelled",
        ExecutionReport::Failed { .. } => "failed",
    }
}

/// The published items as the identity evidence they state, in publication order.
fn identities(report: &QueryReport) -> Vec<String> {
    report
        .items
        .iter()
        .map(|item| {
            serde_json::to_string(&serde_json::json!({
                "consumer": item.consumer,
                "derivation": item.derivation,
                "operation": item.operation,
                "source": item.source,
                "target": item.target,
            }))
            .expect("an identity renders")
        })
        .collect()
}

/// The derivation names a report published, counted.
fn derivations(report: &QueryReport) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    for item in &report.items {
        let name = serde_json::to_string(&item.derivation)
            .expect("a derivation serializes")
            .trim_matches('"')
            .to_string();
        *counts.entry(name).or_insert(0) += 1;
    }
    counts
}

/// The classes of the fixture archive, through the public enumeration: name and bytes.
fn archive_classes(snapshot: &ArtifactSnapshot) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut budget = Budget::new(limits());
    let listed = snapshot.enumerate(&mut budget).expect("the fixture lists");
    let mut classes = Vec::new();
    for entry in &listed.entries {
        if !entry.id.raw_name.0.ends_with(b".class") {
            continue;
        }
        let materialized = snapshot
            .read_entry(&entry.clone(), &mut budget)
            .expect("a listed entry reads");
        classes.push((entry.id.raw_name.0.clone(), materialized.bytes));
    }
    assert!(
        !classes.is_empty(),
        "the fixture archive holds no class entry, so a claim about parsing one would be vacuous"
    );
    classes
}

// ---------------------------------------------------------------------------------------------
// The environment a resolution request runs under
// ---------------------------------------------------------------------------------------------

/// The load root one fixture's own content is: a standalone CLASS snapshot is one whole
/// definition, and a ZIP snapshot is searched in its root container with an empty prefix. The
/// fixture decides which shape it is; several rows of this file run over the committed class file
/// and the rest over archives built in memory.
fn snapshot_root(snapshot: &ArtifactSnapshot) -> LoadRoot {
    match snapshot.kind() {
        ArtifactKind::StandaloneClass => LoadRoot::StandaloneClass {
            snapshot: snapshot.id().clone(),
        },
        ArtifactKind::Zip => LoadRoot::Container {
            origin: ContainerOrigin {
                snapshot: snapshot.id().clone(),
                root_container: ContainerId("root".into()),
                steps: Vec::new(),
            },
            prefix: ArchiveNameBytes(Vec::new()),
        },
    }
}

fn domain(snapshot: &ArtifactSnapshot) -> LoadDomain {
    LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![snapshot_root(snapshot)],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    }
}

fn environment(
    snapshot: &ArtifactSnapshot,
    java_release: u16,
    providers: Vec<HeaderProvider>,
) -> ResolutionEnvironment {
    let domain = domain(snapshot);
    ResolutionEnvironment {
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
        providers,
    }
}

fn class_request(environment: ResolutionEnvironment, class_name: &[u8]) -> ResolutionRequest {
    ResolutionRequest {
        environment,
        target: SymbolRef::Class {
            owner: JvmBytes(class_name.to_vec()),
        },
        use_kind: ReferenceUse::ClassReference,
        caller: CallerContext {
            loader: LoaderId("app".to_string()),
            enclosing: None,
        },
        dispatch: None,
    }
}

fn resolve_with(
    content: &[ArtifactSnapshot],
    request: &ResolutionRequest,
    cache: Option<&FactsCache>,
) -> (ResolutionReport, UsageSnapshot) {
    let mut budget = match cache {
        Some(cache) => Budget::new(limits()).with_facts_cache(cache.clone()),
        None => Budget::new(limits()),
    };
    let report = Engine::new()
        .resolve_symbol(content, request, &mut budget)
        .expect("a legal request is answered, not raised");
    (report, budget.usage())
}

// ---------------------------------------------------------------------------------------------
// The switch, and the value behind it
// ---------------------------------------------------------------------------------------------

/// The switch is off until a caller declares it, and a cache nobody attached is never consulted.
///
/// This is the half of "disabled" that a source scan cannot hold: `Budget::new` — the constructor
/// every existing entry point already uses — carries no cache, so no run of this engine consults one
/// unless its caller attached one, and every consumer below that budget reads today's bytes through
/// today's code.
#[test]
fn the_default_path_consults_no_cache() {
    assert!(
        Budget::new(limits()).facts_cache().is_none(),
        "a budget built the way every caller builds one carries a facts cache, so the cache would be \
         on in the default path"
    );

    // A cache that this test holds and never attaches: the run below must not touch it, which is
    // the statement "attaching one is what turns it on", not "no cache exists".
    let unattached = cache(8);
    let snapshot = open(ARCHIVE.to_vec());
    let request = full_range_request(&snapshot);
    let (first, usage) = query(&snapshot, &request);
    let (second, _) = query(&snapshot, &request);
    assert_eq!(
        identities(&first),
        identities(&second),
        "two runs of one request over one snapshot published different items"
    );
    assert!(
        charged(&usage, CountedBudgetDimension::ClassBytes) > 0,
        "the run parsed no class bytes at all, so it is not the direct path this test means to pin"
    );
    let observed = unattached.report();
    assert_eq!(
        observed.consultations, 0,
        "a cache nobody attached was consulted {} time(s): the default path has a cache in it",
        observed.consultations
    );
    assert_eq!(observed.entries, 0, "{observed:?}");
    println!(
        "the default path parsed {} class byte(s) and {attribute} attribute byte(s) with no cache \
         consulted",
        charged(&usage, CountedBudgetDimension::ClassBytes),
        attribute = charged(&usage, CountedBudgetDimension::AttributeBytes)
    );
}

/// Every byte the cache answers for is answered with the value the direct path produces — for both
/// products of this layer, under both parse policies, and for a release the strict gate refuses.
///
/// The equality is the whole claim: the key is the content, so the answer has to be the value those
/// bytes produce, and a test that counted hits would accept anything.
#[test]
fn the_cached_facts_are_the_facts_the_direct_path_produces() {
    let archive = open(ARCHIVE.to_vec());
    let archive_class = archive_classes(&archive)
        .into_iter()
        .next()
        .expect("the fixture archive holds a class")
        .1;
    let subjects: [(&str, Vec<u8>); 4] = [
        ("the control class", CONTROL.to_vec()),
        ("a class of the fixture archive", archive_class),
        ("a synthetic class", class_bytes(b"p/Same", 52)),
        (
            "a release the registry does not record",
            class_bytes(b"p/Future", 72),
        ),
    ];

    for (name, bytes) in &subjects {
        let expected = direct(bytes);
        let store = cache(4);
        let (cold, first, cold_usage) = through(bytes, &store);
        assert_eq!(
            &cold, &expected,
            "{name}: the first cached read differs from the direct read of the same bytes"
        );
        assert_eq!(
            (first.consultations, first.misses, first.stored, first.hits),
            (1, 1, 1, 0),
            "{name}: a cold read is one consultation, one miss and one store: {first:?}"
        );
        assert_eq!(
            charged(&cold_usage, CountedBudgetDimension::ClassBytes),
            bytes.len() as u64,
            "{name}: a cold read has to pay the parse the direct path pays"
        );

        let report = upheld(&expected, through(bytes, &store));
        assert_eq!(
            (
                report.consultations,
                report.misses,
                report.stored,
                report.hits,
                report.entries
            ),
            (2, 1, 1, 1, 1),
            "{name}: a warm read is one hit over the entry the cold read stored: {report:?}"
        );
        let (_, again, warm_usage) = through(bytes, &store);
        assert_eq!(
            (
                again.consultations,
                again.hits,
                again.misses,
                again.stored,
                again.entries
            ),
            (3, 2, 1, 1, 1),
            "{name}: the third read is the second hit and stores nothing: {again:?}"
        );
        for dimension in [
            CountedBudgetDimension::ClassBytes,
            CountedBudgetDimension::AttributeBytes,
        ] {
            assert_eq!(
                charged(&warm_usage, dimension),
                0,
                "{name}: a hit charged {dimension:?}, which measures a parse it did not run: {}",
                charged(&cold_usage, dimension)
            );
        }

        // The header product, under both policies. A refusal is compared as a refusal: the strict
        // gate's answer is part of what the cache has to reproduce.
        for mode in [InspectionMode::Strict, InspectionMode::Forensic] {
            let direct = inspect_header(bytes, &mut Budget::new(limits()), mode)
                .map_err(|error| error.to_string());
            let store = cache(4);
            let mut cold_budget = Budget::new(limits()).with_facts_cache(store.clone());
            let cold =
                inspect_header(bytes, &mut cold_budget, mode).map_err(|error| error.to_string());
            assert_eq!(
                direct, cold,
                "{name}: the first cached header read under {mode:?} differs from the direct read"
            );
            let mut warm_budget = Budget::new(limits()).with_facts_cache(store.clone());
            let warm =
                inspect_header(bytes, &mut warm_budget, mode).map_err(|error| error.to_string());
            assert_eq!(
                direct, warm,
                "{name}: the warm header read under {mode:?} differs from the direct read"
            );
            match (&direct, cold, warm) {
                (Ok(_expected), _, _) => {
                    let report = store.report();
                    assert_eq!(
                        (report.entries, report.stored),
                        (1, 1),
                        "{name} under {mode:?}: a readable header was not stored once: {report:?}"
                    );
                    assert_eq!(
                        (report.hits, report.misses),
                        (1, 1),
                        "{name} under {mode:?}: the warm read was not the one hit: {report:?}"
                    );
                }
                (Err(_refusal), _, _) => assert_eq!(
                    store.report().stored,
                    0,
                    "{name} under {mode:?}: a refused header read was stored, so a request that the \
                     strict gate refuses could later be answered from the cache"
                ),
            }
        }
        println!(
            "{name}: {} class byte(s), {} attribute byte(s), {} constant-pool entrie(s) — equal on \
             both paths",
            bytes.len(),
            charged(&cold_usage, CountedBudgetDimension::AttributeBytes),
            expected.constant_pool.len()
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Invalidation: one dimension at a time
// ---------------------------------------------------------------------------------------------

/// Each dimension of the identity is turned on its own, and the case that turns it is the case a
/// key that dropped it would pass.
///
/// The four flips are the ones the `facts-cache` requirement names for this layer — the class
/// content, the parse policy and the parser/registry declaration (with its entry format) — plus the
/// pair of *non*-flips the layering scenario requires of a raw layer: a runtime profile, an output
/// level and a recovery configuration are not dimensions here, and a cache that took one of them
/// would be a cache that misses for a reason that cannot change its answer.
#[test]
fn the_key_binds_content_policy_and_declaration() {
    // (a) content: two classes are two entries, and neither answers for the other.
    let first = class_bytes(b"p/First", 52);
    let second = class_bytes(b"p/Second", 52);
    let store = cache(4);
    upheld(&direct(&first), through(&first, &store));
    let report = upheld(&direct(&second), through(&second, &store));
    assert_eq!(
        (
            report.entries,
            report.stored,
            report.consultations,
            report.misses,
            report.hits
        ),
        (2, 2, 2, 2, 0),
        "two classes share one entry, or one of them was answered by the other's: {report:?}"
    );
    let report = upheld(&direct(&second), through(&second, &store));
    assert_eq!((report.hits, report.entries), (1, 2), "{report:?}");

    // (b) parse policy: the structural facts, the strict header and the forensic header of one
    // class are three answers, and each policy is answered by its own entry.
    let bytes = CONTROL.to_vec();
    let store = cache(4);
    upheld(&direct(&bytes), through(&bytes, &store));
    for mode in [InspectionMode::Strict, InspectionMode::Forensic] {
        let expected = direct_header(&bytes, mode);
        let before = store.report().hits;
        let (value, report) = header_through(&bytes, mode, &store);
        assert_eq!(
            value, expected,
            "the header under {mode:?} differs from the direct read"
        );
        assert_eq!(
            report.hits, before,
            "the header under {mode:?} was answered by an entry another policy wrote: {report:?}"
        );
        let (value, report) = header_through(&bytes, mode, &store);
        assert_eq!(value, expected, "{mode:?}");
        assert_eq!(
            report.hits,
            before + 1,
            "the second read under {mode:?} was not its own entry's hit: {report:?}"
        );
    }
    let report = store.report();
    assert_eq!(
        (report.entries, report.stored, report.misses, report.hits),
        (3, 3, 3, 2),
        "one class under three policies is three entries, and no policy answered another's read: \
         {report:?}"
    );

    // (c) declaration: an entry written in another entry format is discarded, counted, and the
    // direct path answers — for the reader that declares the other format *and* for the one that
    // declared the first, whose entry is gone rather than left behind.
    let store = cache(4);
    upheld(&direct(&bytes), through(&bytes, &store));
    let identity = store.identity();
    let other = store.over(FactsIdentity::new(identity.registry, identity.format + 1));
    let (value, report, usage) = through(&bytes, &other);
    assert_eq!(
        value,
        direct(&bytes),
        "the discard did not fall back to the parse of the same bytes"
    );
    assert_eq!(
        (
            report.discarded_format,
            report.entries,
            report.stored,
            report.misses
        ),
        (1, 1, 2, 2),
        "an entry in another format was served, kept, or not re-parsed: {report:?}"
    );
    assert_eq!(
        charged(&usage, CountedBudgetDimension::ClassBytes),
        bytes.len() as u64,
        "the fallback did not pay the parse it replaced"
    );
    let (_, report, _) = through(&bytes, &store);
    assert_eq!(
        (report.discarded_format, report.stored, report.entries),
        (2, 3, 1),
        "the entry the other declaration discarded was still there for the declaration that \
         wrote it: {report:?}"
    );

    // (d) declaration: an entry parsed under another registry version is discarded the same way.
    let store = cache(4);
    upheld(&direct(&bytes), through(&bytes, &store));
    let identity = store.identity();
    let newer = store.over(FactsIdentity::new(identity.registry + 1, identity.format));
    let (value, report, _) = through(&bytes, &newer);
    assert_eq!(
        value,
        direct(&bytes),
        "the registry discard did not fall back"
    );
    assert_eq!(
        (report.discarded_registry, report.entries, report.stored),
        (1, 1, 2),
        "an entry parsed under another registry version was served or kept: {report:?}"
    );

    println!(
        "content, parse policy, entry format and registry version were each turned alone; every \
         turn was answered by the direct parse"
    );
}

/// The parse policy is not bookkeeping: a strict request must never be answered with a forensic read
/// of the same bytes.
///
/// The fixture is a release above the registry's highest registered major — one `Strict` refuses and
/// `Forensic` reads, with the diagnostic that says why. If the policy were dropped from the key, the
/// strict request would be answered with the forensic entry: a request that must be refused would
/// return a readable header, which is the fail-open a cache must not introduce.
#[test]
fn a_strict_request_is_never_answered_with_a_forensic_read() {
    let bytes = class_bytes(b"p/Future", 72);
    assert!(
        inspect_header(&bytes, &mut Budget::new(limits()), InspectionMode::Strict).is_err(),
        "the fixture has to be one the strict gate refuses, or this test compares two readable \
         headers and proves nothing"
    );

    // The forensic read first: it reads, it publishes one diagnostic, it stores.
    let store = cache(4);
    let (forensic, report) = header_through(&bytes, InspectionMode::Forensic, &store);
    assert!(
        !forensic.diagnostics.is_empty(),
        "the forensic read of a release the registry does not record has to publish the \
         diagnostics this test reads the difference off; it published none"
    );
    assert_eq!((report.entries, report.stored), (1, 1), "{report:?}");

    let mut budget = Budget::new(limits()).with_facts_cache(store.clone());
    let refused = inspect_header(&bytes, &mut budget, InspectionMode::Strict);
    assert!(
        refused.is_err(),
        "a strict request was answered with a forensic entry: the cache turned a refusal into a \
         readable header ({refused:?})"
    );
    let report = store.report();
    assert_eq!(
        (
            report.entries,
            report.stored,
            report.hits,
            report.misses,
            report.discarded_format + report.discarded_registry + report.discarded_product
        ),
        (1, 1, 0, 2, 0),
        "the strict request used the forensic entry, discarded it, or stored something of its own: \
         {report:?}"
    );

    // The other order, because a refusal is a state a cache must not have recorded: nothing was
    // stored by the failed strict read, so the forensic read after it is still a cold one.
    let store = cache(4);
    let mut budget = Budget::new(limits()).with_facts_cache(store.clone());
    assert!(
        inspect_header(&bytes, &mut budget, InspectionMode::Strict).is_err(),
        "the fixture's strict read has to keep failing"
    );
    assert_eq!(
        store.report().entries,
        0,
        "a refusal was stored as an answer: {report:?}",
        report = store.report()
    );
    let (again, _) = header_through(&bytes, InspectionMode::Forensic, &store);
    assert_eq!(again, forensic, "the forensic read changed after a refusal");
    println!(
        "a strict read of major 72 was refused and still refused after a forensic entry for the \
         same bytes existed; the forensic entry stayed for the policy that wrote it"
    );
}

// ---------------------------------------------------------------------------------------------
// The fallback, the budget and the refusals
// ---------------------------------------------------------------------------------------------

/// A discarded entry falls back to the direct path **under the budget the request already holds**.
///
/// Three runs are compared for each budget: one with no cache at all, one whose cache holds an entry
/// it must throw away, and — in the last case — a whole request that had *already charged* for
/// opening the artifact and reading its entries before it reached the parse. The cache may not buy a
/// request a second attempt, a fresh limit or a smaller bill, and the last case is the one that says
/// so: a fallback that handed the lifecycle back would wipe what the request had already paid, and
/// the totals would not match.
#[test]
fn a_discarded_entry_falls_back_under_the_same_budget() {
    let bytes = CONTROL.to_vec();
    let direct_facts = direct(&bytes);
    let stale = |store: &FactsCache| {
        let identity = store.identity();
        store.over(FactsIdentity::new(identity.registry, identity.format + 1))
    };

    // (1) headroom: the fallback completes and charges exactly what the cache-off run charges.
    let writer = cache(4);
    upheld(&direct_facts, through(&bytes, &writer));
    let reader = stale(&writer);
    let mut off_budget = Budget::new(limits());
    let off = class_facts(&bytes, &mut off_budget).expect("the fixture is a class file");
    let mut on_budget = Budget::new(limits()).with_facts_cache(reader.clone());
    let on = class_facts(&bytes, &mut on_budget).expect("the fixture is a class file");
    assert_eq!(off, on, "the fallback answered with different facts");
    assert_eq!(
        charges(&off_budget.usage()),
        charges(&on_budget.usage()),
        "the fallback charged a different budget than the run with no cache"
    );
    assert_eq!(
        reader.report().discarded_format,
        1,
        "the stale entry was not the reason for the fallback"
    );

    // (2) a budget that cannot afford the parse: the fallback stops in the same dimension with the
    // same numbers, twice in a row.
    let writer = cache(4);
    upheld(&direct_facts, through(&bytes, &writer));
    let reader = stale(&writer);
    let tight = limits_with(1, 1 << 20);
    let mut off_budget = Budget::new(tight.clone());
    let off = class_facts(&bytes, &mut off_budget).expect_err("1 class byte cannot pay for 300");
    let mut on_budget = Budget::new(tight.clone()).with_facts_cache(reader.clone());
    let on = class_facts(&bytes, &mut on_budget).expect_err("1 class byte cannot pay for 300");
    assert_eq!(
        off.to_string(),
        on.to_string(),
        "the fallback stopped for a different reason than the run with no cache"
    );
    assert_eq!(
        charges(&off_budget.usage()),
        charges(&on_budget.usage()),
        "the fallback charged differently under an exhausted budget"
    );
    let mut again_budget = Budget::new(tight).with_facts_cache(reader.clone());
    let again =
        class_facts(&bytes, &mut again_budget).expect_err("1 class byte cannot pay for 300");
    assert_eq!(
        again.to_string(),
        on.to_string(),
        "the second attempt answered differently: a refused budget was reset or retried"
    );
    assert_eq!(
        charges(&again_budget.usage()),
        charges(&on_budget.usage()),
        "the second attempt charged differently"
    );
    let report = reader.report();
    assert_eq!(
        (report.stored, report.entries),
        (1, 0),
        "the only write of this store is the priming one, and a parse the budget stopped stored \
         nothing: {report:?}"
    );

    // (3) The same rule where the request has *already charged*: a query enumerates and reads the
    // archive before it parses anything, so a fallback that handed the request a fresh lifecycle
    // would give back what the enumeration and the reads had paid. Those counters are the request's
    // and cannot be refunded, whatever the cache does with the parse.
    let archive = open(ARCHIVE.to_vec());
    let class = archive_classes(&archive)
        .into_iter()
        .next()
        .expect("the fixture archive holds a class")
        .1;
    let writer = cache(8);
    upheld(&direct(&class), through(&class, &writer));
    let identity = writer.identity();
    let stale = writer.over(FactsIdentity::new(identity.registry, identity.format + 1));

    let snapshot = open_with(ARCHIVE.to_vec(), &stale);
    let request = full_range_request(&snapshot);
    let (off, off_usage) = query(&snapshot, &request);
    let (on, on_usage) = query_with(&snapshot, &request, &stale);
    assert_eq!(
        identities(&off),
        identities(&on),
        "the fallback published a different result than the run with no cache"
    );
    let before_the_parse = |usage: &UsageSnapshot| {
        [
            CountedBudgetDimension::ArchiveEntries,
            CountedBudgetDimension::EntryBytes,
            CountedBudgetDimension::ReadBytes,
            CountedBudgetDimension::OutputBytes,
            CountedBudgetDimension::ResultItems,
        ]
        .map(|dimension| charged(usage, dimension))
    };
    assert!(
        before_the_parse(&off_usage)[0] > 0 && before_the_parse(&off_usage)[2] > 0,
        "this case only means something if the request charged for its enumeration and its reads \
         before the parse: {}",
        charges_line(&off_usage)
    );
    assert_eq!(
        before_the_parse(&on_usage),
        before_the_parse(&off_usage),
        "the fallback handed the request a fresh lifecycle, giving back what it had already paid \
         ({} against {})",
        charges_line(&on_usage),
        charges_line(&off_usage)
    );
    for dimension in [
        CountedBudgetDimension::ClassBytes,
        CountedBudgetDimension::AttributeBytes,
    ] {
        assert!(
            charged(&on_usage, dimension) <= charged(&off_usage, dimension),
            "{dimension:?} grew with a cache attached: {} over {}",
            charges_line(&on_usage),
            charges_line(&off_usage)
        );
    }
    assert!(
        stale.report().discarded_format >= 1
            && charged(&on_usage, CountedBudgetDimension::ClassBytes) > 0,
        "the run did not both discard an entry and fall back to a parse, so it is not the case this \
         one is about: {:?} over {}",
        stale.report(),
        charges_line(&on_usage)
    );
    println!(
        "an entry the reader had to discard was answered by the direct parse: {} class byte(s) \
         under headroom, the same refusal under a 1-byte allowance twice, and — over a whole query \
         that had already paid for {} archive entrie(s) and {} read byte(s) — those charges stayed \
         the request's ({} against {})",
        bytes.len(),
        charged(&on_usage, CountedBudgetDimension::ArchiveEntries),
        charged(&on_usage, CountedBudgetDimension::ReadBytes),
        charges_line(&on_usage),
        charges_line(&off_usage)
    );
}

/// A read that stopped writes nothing, and a request that was cancelled is not answered from memory.
///
/// The two are one rule: the cache stores *complete* answers only, so "an incomplete result never
/// stands in for a complete one" holds without a completeness check on the way out — there is no
/// incomplete entry to check. Cancellation is checked on the way *in*, because serving a fact from
/// memory is not a reason to answer a request that has already been cancelled.
#[test]
fn a_stopped_read_stores_nothing_and_a_cancelled_request_is_never_answered() {
    let bytes = CONTROL.to_vec();
    let store = cache(4);

    let mut budget = Budget::new(limits_with(1, 1 << 20)).with_facts_cache(store.clone());
    let stopped = class_facts(&bytes, &mut budget).expect_err("1 class byte cannot pay for 300");
    let report = store.report();
    assert_eq!(
        (
            report.entries,
            report.stored,
            report.consultations,
            report.misses,
            report.hits
        ),
        (0, 0, 1, 1, 0),
        "a parse the budget stopped was stored, or the lookup did not happen at all: {report:?} \
         (the stop was `{stopped}`)"
    );

    // The next request has the headroom the first one lacked, and it parses from scratch.
    let expected = direct(&bytes);
    let report = upheld(&expected, through(&bytes, &store));
    assert_eq!(report.entries, 1, "{report:?}");

    // Cancelled: the entry exists and is not used.
    let mut budget = Budget::new(limits()).with_facts_cache(store.clone());
    let token = budget.cancellation_token();
    token.cancel();
    let error = class_facts(&bytes, &mut budget)
        .expect_err("a cancelled request was answered from the cache");
    assert!(
        matches!(error, Error::Cancelled { .. }),
        "a cancelled request over a warm cache did not report the cancellation: {error:?}"
    );
    let report = upheld(&expected, through(&bytes, &store));
    assert_eq!(
        (report.entries, report.hits, report.stored),
        (1, 1, 1),
        "the cancelled run changed the store: {report:?}"
    );

    // The same rule at the entry point: a cancelled query over warm bytes is cancelled.
    let snapshot = open_with(ARCHIVE.to_vec(), &store);
    let request = full_range_request(&snapshot);
    let (warm, _) = query_with(&snapshot, &request, &store);
    assert_eq!(status_of(&warm.execution), "complete");
    let (off, _) = {
        let mut budget = Budget::new(limits());
        let token = budget.cancellation_token();
        token.cancel();
        let report = Engine::new()
            .query(&snapshot, &request, &mut budget)
            .expect("a legal query is answered, not raised");
        (report, ())
    };
    let mut budget = Budget::new(limits()).with_facts_cache(store.clone());
    let token = budget.cancellation_token();
    token.cancel();
    let on = Engine::new()
        .query(&snapshot, &request, &mut budget)
        .expect("a legal query is answered, not raised");
    assert_eq!(
        status_of(&on.execution),
        "cancelled",
        "a warm cache turned a cancelled query into a complete answer"
    );
    assert_eq!(
        identities(&on),
        identities(&off),
        "the cancelled query published a different prefix with a warm cache"
    );
    println!(
        "a budget-stopped parse stored nothing; a cancelled request over a warm cache reported \
         `{status}` with the same {} published item(s) as the cache-off run",
        off.items.len(),
        status = status_of(&on.execution)
    );
}

/// A hit publishes the same diagnostics, so it pays the bill for publishing them — and it pays
/// nothing for the structure it did not parse.
///
/// The charge for a published item or diagnostic is not a cost of parsing: it is the price of putting
/// something in the report, and a cache cannot make publishing free. With no `ResultItems` headroom
/// both paths refuse on that dimension, and the hit's refusal asks for exactly the diagnostics it
/// would have published — which is what makes it evidence about hits rather than about misses.
#[test]
fn a_hit_still_bills_what_it_publishes() {
    let bytes = class_bytes(b"p/Future", 72);
    let store = cache(4);
    let (forensic, _) = header_through(&bytes, InspectionMode::Forensic, &store);
    assert!(
        !forensic.diagnostics.is_empty(),
        "the fixture has to publish a diagnostic for this test to have a bill to charge"
    );

    let mut budget = Budget::new(limits_with(1 << 20, 0));
    let direct = inspect_header(&bytes, &mut budget, InspectionMode::Forensic)
        .expect_err("publishing is billed, and this budget allows no item");
    let mut budget = Budget::new(limits_with(1 << 20, 0)).with_facts_cache(store.clone());
    let warm = inspect_header(&bytes, &mut budget, InspectionMode::Forensic)
        .expect_err("a hit published diagnostics without being billed for them");
    let billed = |error: Error| match error {
        Error::BudgetExceeded {
            dimension,
            requested,
            ..
        } => (dimension, requested),
        other => panic!("the refusal is a budget refusal on the published items: {other:?}"),
    };
    assert_eq!(
        billed(direct),
        (BudgetDimension::ResultItems, 1),
        "the direct read's first bill is the structure it parses, one item at a time"
    );
    assert_eq!(
        billed(warm),
        (
            BudgetDimension::ResultItems,
            u64::try_from(forensic.diagnostics.len()).expect("a diagnostic count fits u64")
        ),
        "the hit was not billed for exactly the diagnostics it published"
    );
    assert_eq!(
        store.report().hits,
        1,
        "the refusal happened before the hit, so this test says nothing about hits: {:?}",
        store.report()
    );
    println!(
        "a hit with no `result_items` headroom asked for {} item(s) — the diagnostics it would \
         publish — while the direct read asked for the structure it parses",
        forensic.diagnostics.len()
    );
}

/// The capacity is a bound on the store, and a store that is full refuses to *write*, not to answer.
///
/// A refused entry is parsed every time rather than answered from a store that never held it, which
/// is the difference between "the cache saved nothing here" and "the cache answered with something
/// it did not have".
#[test]
fn the_capacity_is_a_bound_and_a_refusal_is_not_a_hit() {
    let first = class_bytes(b"p/First", 52);
    let second = class_bytes(b"p/Second", 52);
    let store = cache(1);
    upheld(&direct(&first), through(&first, &store));
    let report = upheld(&direct(&second), through(&second, &store));
    assert_eq!(
        (
            report.entries,
            report.stored,
            report.refused_capacity,
            report.hits
        ),
        (1, 1, 1, 0),
        "a store of one entry took a second one, or answered the second class from the first: \
         {report:?}"
    );
    let (value, report, usage) = through(&second, &store);
    assert_eq!(value, direct(&second), "the refused class was mis-answered");
    assert_eq!(
        (
            report.entries,
            report.refused_capacity,
            report.hits,
            report.misses
        ),
        (1, 2, 0, 3),
        "the second read of a refused class was not parsed again: {report:?}"
    );
    assert_eq!(
        charged(&usage, CountedBudgetDimension::ClassBytes),
        second.len() as u64,
        "a refused store answered without parsing"
    );
    println!(
        "a one-entry store refused the second class rather than evicting the first, and every read \
         of it was parsed again"
    );
}

// ---------------------------------------------------------------------------------------------
// Transparency at the entry points
// ---------------------------------------------------------------------------------------------

/// The cache removes repeated parses of one class and moves nothing else.
///
/// The entry point under test is the engine's own scan, twice over the same bytes: one request that
/// asks for every consumer category reads a class once per consumer stream, and a later request reads
/// it again. With a cache attached the parses collapse — the cold run already removes the repeats
/// inside one request, the warm run removes the last one — while the reads, the enumeration, the
/// published items, the coverage, the diagnostics and the derivations stay exactly where the direct
/// run left them. That last half is what "the read still happens" means as a measurement.
#[test]
fn the_cache_removes_repeated_parses_and_changes_nothing_else() {
    let snapshot = open(ARCHIVE.to_vec());
    for (name, request, items_are_candidates) in [
        ("a full-range scan", full_range_request(&snapshot), false),
        (
            "a raw constant-pool probe",
            pool_probe_request(&snapshot),
            true,
        ),
    ] {
        let (off, off_usage) = query(&snapshot, &request);
        let store = cache(8);
        let (_, cold_usage) = query_with(&snapshot, &request, &store);
        let (on, on_usage) = query_with(&snapshot, &request, &store);

        assert_eq!(
            status_of(&off.execution),
            status_of(&on.execution),
            "{name}: the cache changed the terminal status"
        );
        assert_eq!(
            identities(&off),
            identities(&on),
            "{name}: the cache changed the published facts"
        );
        assert!(
            !off.items.is_empty(),
            "{name}: the comparison published nothing, so it proves nothing"
        );
        assert_eq!(
            serde_json::to_value(&off.coverage).expect("coverage serializes"),
            serde_json::to_value(&on.coverage).expect("coverage serializes"),
            "{name}: the cache changed which ranges the scan reports as scanned"
        );
        assert_eq!(
            serde_json::to_value(&off.diagnostics).expect("diagnostics serialize"),
            serde_json::to_value(&on.diagnostics).expect("diagnostics serialize"),
            "{name}: the cache changed the diagnostics"
        );
        assert_eq!(
            derivations(&off),
            derivations(&on),
            "{name}: the cache changed the derivations the scan claims"
        );

        // The reading half is untouched: the entry is still listed, still read, still paid for.
        for dimension in [
            CountedBudgetDimension::InputBytes,
            CountedBudgetDimension::ArchiveEntries,
            CountedBudgetDimension::EntryBytes,
            CountedBudgetDimension::ReadBytes,
            CountedBudgetDimension::OutputBytes,
            CountedBudgetDimension::ClassHeaders,
            CountedBudgetDimension::ResultItems,
        ] {
            assert_eq!(
                charged(&on_usage, dimension),
                charged(&off_usage, dimension),
                "{name}: {dimension:?} moved, and it measures a read or a publication the cache \
                 may not remove"
            );
        }
        assert!(
            charged(&on_usage, CountedBudgetDimension::ClassBytes)
                < charged(&off_usage, CountedBudgetDimension::ClassBytes),
            "{name}: the warm run removed no parse at all ({} class byte(s) either way)",
            charged(&off_usage, CountedBudgetDimension::ClassBytes)
        );
        assert!(
            charged(&cold_usage, CountedBudgetDimension::ClassBytes)
                <= charged(&off_usage, CountedBudgetDimension::ClassBytes),
            "{name}: the cold cached run charged more than the direct one"
        );
        assert!(
            store.report().hits > 0,
            "{name}: no parse was answered from the store: {:?}",
            store.report()
        );

        // A01, in the shape each relation admits: a pool match is a candidate and never an
        // XRef fact. The full-range scan publishes no pool candidate and no item without a
        // consumer; the pool probe publishes exactly those candidates, still unverified.
        if items_are_candidates {
            for item in &on.items {
                assert_eq!(
                    item.consumer, None,
                    "{name}: a pool entry was published with a consumer category"
                );
                assert_eq!(
                    item.derivation,
                    XrefDerivation::ConstantPoolCandidate,
                    "{name}: the probe published something else"
                );
            }
        } else {
            assert_eq!(
                derivations(&on)
                    .get("constant_pool_candidate")
                    .copied()
                    .unwrap_or(0),
                0,
                "{name}: a pool entry reached the scan as an item"
            );
            assert_eq!(
                on.items
                    .iter()
                    .filter(|item| item.consumer.is_none())
                    .count(),
                0,
                "{name}: an item reached the report without a consumer that verified it (A01)"
            );
        }
        println!(
            "{name}: {} item(s) either way, {} class byte(s) direct / {} cold / {} warm, {} read \
             byte(s) either way, {} hit(s)",
            on.items.len(),
            charged(&off_usage, CountedBudgetDimension::ClassBytes),
            charged(&cold_usage, CountedBudgetDimension::ClassBytes),
            charged(&on_usage, CountedBudgetDimension::ClassBytes),
            charged(&on_usage, CountedBudgetDimension::ReadBytes),
            store.report().hits
        );
    }
}

/// Two entries with the same bytes share one entry, and the report says which origin each fact came
/// from — because the cache never held an origin to merge.
///
/// The fixture is a stored archive with one class under two names: the same content, two physical
/// definitions. A content-keyed cache answers both parses with one entry, and the published items
/// still name the entry each one was read from, since the origin is bound by the caller that read
/// the bytes and not by the entry that skipped the parse.
#[test]
fn content_is_shared_and_two_origins_are_never_merged() {
    let bytes = class_bytes(b"p/Same", 52);
    let archive = zip_of(&[(b"a/Same.class", &bytes), (b"b/Same.class", &bytes)]);
    let store = cache(4);
    let snapshot = open_with(archive, &store);
    let (report, _) = query_with(&snapshot, &full_range_request(&snapshot), &store);

    // The origin each item states, read from the published document: the physical entry the class
    // bytes were read from, by its raw name.
    let names: BTreeSet<Vec<u8>> = report
        .items
        .iter()
        .map(|item| {
            let source = serde_json::to_value(&item.source).expect("an origin serializes");
            source["location"]["definition"]["location"]["entry"]["raw_name"]
                .as_array()
                .expect("the origin names its entry's raw name as octets")
                .iter()
                .map(|octet| {
                    u8::try_from(octet.as_u64().expect("an octet is a number"))
                        .expect("an octet is a byte")
                })
                .collect()
        })
        .collect();
    assert_eq!(
        names,
        BTreeSet::from([b"a/Same.class".to_vec(), b"b/Same.class".to_vec()]),
        "the two entries did not keep their own origins in the published items"
    );

    let observed = store.report();
    assert_eq!(
        observed.entries, 1,
        "two entries holding the same bytes did not share one entry: {observed:?}"
    );
    assert!(
        observed.hits > 0,
        "the second parse of the same bytes was not answered from the first one's entry: \
         {observed:?}"
    );
    println!(
        "one entry answered {} parse(s) of byte-equal classes and the items still name their own \
         entries: {:?}",
        observed.hits,
        names
            .iter()
            .map(|name| String::from_utf8_lossy(name).into_owned())
            .collect::<Vec<_>>()
    );
}

/// The layer above: a profile is not a dimension of this layer's key, and the entries below a
/// profile change are still the entries of the same bytes.
///
/// The resolution layer's own record carries the platform — `environment_identity.runtime.profile`
/// is in the report of every run — and the *facts* below it do not depend on that profile, so a
/// profile change may not invalidate them. The read evidence is the control: the second run still
/// reads the same definition, charges the same `ClassHeaders`, and reports the same `reads`, because
/// the cache removes the parse and not the read. The recovery layer, which owns its own passes and
/// output level, is exercised over the same store for the same reason.
#[test]
fn a_raw_layer_entry_survives_the_profile_and_the_recovery_above_it() {
    let snapshot = open(CONTROL.to_vec());
    let content = [snapshot.clone()];
    let facts = direct(CONTROL);
    let owner = facts.this_class.raw().0.clone();

    let ask =
        |java_release: u16| class_request(environment(&snapshot, java_release, Vec::new()), &owner);
    let store = cache(4);
    let (first, first_usage) = resolve_with(&content, &ask(8), Some(&store));
    let (second, second_usage) = resolve_with(&content, &ask(11), Some(&store));

    assert_eq!(
        first.environment_identity.runtime.profile.java_release, 8,
        "the result has to carry the platform it was asked under"
    );
    assert_eq!(
        second.environment_identity.runtime.profile.java_release, 11,
        "the second result has to carry its own platform, not the first one's"
    );
    assert_eq!(
        serde_json::to_value(&first.reads).expect("evidence serializes"),
        serde_json::to_value(&second.reads).expect("evidence serializes"),
        "the second run reported different read evidence under another profile, so the cache \
         changed what the request did rather than what it paid"
    );
    let observed = store.report();
    assert!(
        observed.hits > 0,
        "the profile change was answered by a miss, so this layer took the platform into its key: \
         {observed:?}"
    );
    assert_eq!(
        observed.entries, 1,
        "one class under two profiles is one content-keyed entry: {observed:?}"
    );
    assert_eq!(
        charged(&first_usage, CountedBudgetDimension::ClassBytes),
        CONTROL.len() as u64,
        "the first run has to pay for the parse"
    );
    assert_eq!(
        charged(&second_usage, CountedBudgetDimension::ClassBytes),
        0,
        "the second run re-parsed the same bytes under another profile"
    );
    assert_eq!(
        charged(&first_usage, CountedBudgetDimension::ClassHeaders),
        charged(&second_usage, CountedBudgetDimension::ClassHeaders),
        "the cache changed how many header reads the request attempted"
    );

    // The same answers with no cache at all: the profiles select the same definition here, which is
    // why the entries may be shared.
    for (java_release, resolution) in [(8u16, &first), (11, &second)] {
        let (off, _) = resolve_with(&content, &ask(java_release), None);
        assert_eq!(
            serde_json::to_value(resolution.state).expect("a state serializes"),
            serde_json::to_value(off.state).expect("a state serializes"),
            "the cached run under release {java_release} answered differently than the direct run"
        );
        assert_eq!(
            serde_json::to_value(&resolution.resolved).expect("a definition serializes"),
            serde_json::to_value(&off.resolved).expect("a definition serializes"),
            "the cached run under release {java_release} selected another definition"
        );
    }

    // The recovery layer reads the same class through the same store: its passes, output level and
    // profile are its own inputs, and none of them is a dimension here.
    let premise = Engine::new()
        .inspect_header(
            &snapshot,
            ClassTarget::Root,
            &mut Budget::new(limits()),
            InspectionMode::Strict,
        )
        .expect("the control class's own header reads");
    let request = || MethodAnalysisRequest {
        environment: environment(&snapshot, 8, Vec::new()),
        method: PhysicalMethodId {
            owner: PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.id().clone(),
                },
                class_bytes: premise.source.class_bytes.clone(),
                variant: PhysicalVariant::Base,
            },
            name: JvmBytes(b"add".to_vec()),
            descriptor: JvmBytes(b"(II)V".to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    };
    let recovered_off = Engine::new()
        .recover_method(&content, &request(), &mut Budget::new(limits()))
        .expect("the named member is presented");
    let before = store.report().hits;
    let mut budget = Budget::new(limits()).with_facts_cache(store.clone());
    let recovered_on = Engine::new()
        .recover_method(&content, &request(), &mut budget)
        .expect("the named member is presented");
    let presented = |recovery: &RecoveryReport| {
        let mut value = serde_json::to_value(recovery).expect("a presentation serializes");
        without_charges(&mut value);
        value
    };
    assert_eq!(
        presented(recovered_on.recovery()),
        presented(recovered_off.recovery()),
        "the recovery layer presented something else when the class parse came from the cache; the \
         charge beside each run is the only part this comparison leaves out"
    );
    let terminal = |execution: &ExecutionReport| {
        let mut value = serde_json::to_value(execution).expect("an execution report serializes");
        without_charges(&mut value);
        value
    };
    assert_eq!(
        terminal(&recovered_on.analysis().execution),
        terminal(&recovered_off.analysis().execution),
        "the cached recovery run ended differently: {} over {}",
        status_of(&recovered_on.analysis().execution),
        status_of(&recovered_off.analysis().execution)
    );
    assert!(
        store.report().hits > before,
        "the recovery run did not read through the store, so this half of the test says nothing \
         about the layers above"
    );
    println!(
        "a profile change re-read the same definition, charged {} header read attempt(s) on both \
         sides, and paid {} class byte(s) once; the recovery presentation was identical",
        charged(&second_usage, CountedBudgetDimension::ClassHeaders),
        CONTROL.len()
    );
}

/// A dependency that becomes available is answered by a fresh search, because this layer never held
/// a verdict about it.
///
/// The scenario is the spec's `Dependency becomes available`: a member search that cannot read a
/// class of its hierarchy reports `UnresolvedDependency` and names it, and the same search under an
/// environment that provides that content answers over the complete hierarchy. What the cache has to
/// show is that it cannot stand between the two: a verdict about a *name* has no bytes, so this
/// layer has nothing to store for it, and the second request re-runs the search while the class
/// facts it reads are shared with the first.
#[test]
fn a_provider_added_later_is_answered_by_a_fresh_search() {
    // The class whose hierarchy the environment cannot complete: its own parent is a name nothing
    // provides until the provider below arrives.
    let owner_bytes = class_bytes(b"p/Owner", 52);
    let owner_snapshot = open(zip_of(&[(b"p/Owner.class", &owner_bytes)]));
    let facts = direct(&owner_bytes);
    let parent = facts
        .super_class
        .as_ref()
        .expect("the fixture declares a superclass")
        .raw()
        .0
        .clone();

    // The content the environment does not provide yet: the parent, with no superclass of its own so
    // that a search which reads it can reach the end of the hierarchy.
    let parent_entry = [parent.as_slice(), b".class"].concat();
    let provider = open(zip_of(&[(
        parent_entry.as_slice(),
        &superless_class_bytes(&parent, 52),
    )]));

    let ask = |environment: ResolutionEnvironment| ResolutionRequest {
        environment,
        target: SymbolRef::Method {
            owner: JvmBytes(b"p/Owner".to_vec()),
            name: JvmBytes(b"absent".to_vec()),
            descriptor: JvmBytes(b"(II)V".to_vec()),
        },
        use_kind: ReferenceUse::InvokeVirtual,
        caller: CallerContext {
            loader: LoaderId("app".to_string()),
            enclosing: None,
        },
        dispatch: None,
    };
    let roots = |provider: Option<&ArtifactSnapshot>| {
        let mut roots = vec![snapshot_root(&owner_snapshot)];
        roots.extend(provider.map(snapshot_root));
        roots
    };
    let domain_of = |provider: Option<&ArtifactSnapshot>| LoadDomain {
        loader: LoaderId("app".to_string()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: roots(provider),
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    let environment = |provider: Option<&ArtifactSnapshot>| ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: owner_snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
            },
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            load_domain: domain_of(provider),
        },
        domains: vec![domain_of(provider)],
        providers: provider
            .map(|snapshot| {
                vec![HeaderProvider {
                    id: ProviderId("app-headers".to_string()),
                    roots: vec![snapshot_root(snapshot)],
                }]
            })
            .unwrap_or_default(),
    };
    let without = ask(environment(None));
    let with = ask(environment(Some(&provider)));

    let store = cache(8);
    let (off_without, _) = resolve_with(std::slice::from_ref(&owner_snapshot), &without, None);
    let (on_without, _) = resolve_with(
        std::slice::from_ref(&owner_snapshot),
        &without,
        Some(&store),
    );
    assert_eq!(
        on_without.state,
        Some(ResolutionState::UnresolvedDependency),
        "the fixture has to leave a dependency unread for this test to have a negative: \
         {on_without:?}"
    );
    assert_eq!(
        serde_json::to_value(&on_without).expect("a report serializes")["unresolved_dependencies"]
            .clone(),
        serde_json::to_value(&off_without).expect("a report serializes")["unresolved_dependencies"]
            .clone(),
        "the cached negative differs from the direct one"
    );
    assert!(
        !on_without.unresolved_dependencies.is_empty(),
        "the negative has to name the class it could not read: {on_without:?}"
    );

    let content = [owner_snapshot.clone(), provider.clone()];
    let (off_with, _) = resolve_with(&content, &with, None);
    let (on_with, _) = resolve_with(&content, &with, Some(&store));

    assert_eq!(
        on_with.environment_identity.providers,
        vec![ProviderId("app-headers".to_string())],
        "the second result has to carry the provider set it was asked under"
    );
    assert!(
        on_without.environment_identity.providers.is_empty(),
        "the first result has to carry its own, empty provider set"
    );
    assert_ne!(
        on_with.state,
        Some(ResolutionState::UnresolvedDependency),
        "the old negative stood in for the new answer after the provider was added: {on_with:?}"
    );
    assert!(
        on_with.unresolved_dependencies.is_empty(),
        "the complete hierarchy still reports unread classes: {on_with:?}"
    );
    assert_eq!(
        serde_json::to_value(on_with.state).expect("a state serializes"),
        serde_json::to_value(off_with.state).expect("a state serializes"),
        "the cached answer with the provider differs from the direct one"
    );

    // The negative was not stored, and nothing the cache does hold can answer a question about a
    // name: the store holds class facts by content, and a verdict about a name has none.
    let report = store.report();
    assert_eq!(
        report.discarded_format + report.discarded_registry + report.discarded_product,
        0,
        "a negative entry was discarded rather than never written: {report:?}"
    );
    assert_eq!(
        (report.entries, report.hits),
        (2, 1),
        "the store holds one entry per class it read (the owner and the provided parent), and the \
         owner's facts were reused across the two environments: {report:?}"
    );

    // And the first environment still answers as it did: one cache, two environments, two answers.
    let (again_without, _) = resolve_with(
        std::slice::from_ref(&owner_snapshot),
        &without,
        Some(&store),
    );
    assert_eq!(
        serde_json::to_value(again_without.state).expect("a state serializes"),
        serde_json::to_value(off_without.state).expect("a state serializes"),
        "adding a provider changed the answer of the environment that does not name it"
    );
    let report = store.report();
    assert_eq!(
        (report.entries, report.hits, report.stored),
        (2, 2, 2),
        "the third request neither reused the owner's facts nor left the store alone: {report:?}"
    );
    println!(
        "one store answered {} class fact read(s): without the provider the member search reported \
         `{:?}`, with it `{:?}`",
        report.hits, off_without.state, off_with.state
    );
}

// ---------------------------------------------------------------------------------------------
// A18: the cache half of "跨快照 token/缓存隔离"
// ---------------------------------------------------------------------------------------------

/// The published result of one request over one byte source, with the charge records removed.
///
/// This is the form task 3.1's differential compares two configurations in: the planes
/// `Cold and warm results` names are the result, and the charges beside them are the resource half a
/// cache is *supposed* to move.
fn result_of(bytes: Vec<u8>, store: Option<&FactsCache>) -> String {
    let snapshot = match store {
        Some(store) => open_with(bytes, store),
        None => open(bytes),
    };
    let request = full_range_request(&snapshot);
    let report = match store {
        Some(store) => query_with(&snapshot, &request, store).0,
        None => query(&snapshot, &request).0,
    };
    let mut document = serde_json::to_value(&report).expect("a report serializes");
    without_charges(&mut document);
    serde_json::to_string(&document).expect("a document renders")
}

/// A18's cache half over one store and two snapshots: neither may be answered with the other's facts.
///
/// The acceptance is "跨快照 token/缓存隔离", and the P1 half of it (a snapshot stays stable, an old
/// cursor is rejected) is already accepted (`p1_query_api.rs`). This is the half P5 introduces, and
/// it is turned into the cases a store that dropped one of its dimensions would **pass**:
///
/// * **(a) different content, one store.** Two snapshots holding different classes share one store.
///   A store keyed by anything but the content digest — one slot, one name, one "the class I read
///   last" — answers the second snapshot with the first one's facts, and the published result then
///   differs from that snapshot's own direct run;
/// * **(b) the same content at two origins.** Content *is* shared, and the hit may not launder an
///   identity: the entry carries no origin, so the second snapshot still publishes its own entry's
///   name and its own snapshot. This is the case a store that cached the *read* rather than the
///   *parse* would pass by publishing the first origin's bytes;
/// * **(c) what a stop leaves behind.** A cancelled request writes nothing, and the next request —
///   over the *other* snapshot, on its own budget and its own token — is answered exactly as its
///   direct run.
///
/// Each case asserts equality against the **direct** run of the same bytes, so the test does not
/// pin a fingerprint this file invented: it pins that the cached path is the direct path.
#[test]
fn one_store_serves_two_snapshots_without_answering_either_with_the_other() {
    let alpha_bytes = class_bytes(b"p/Alpha", 52);
    let beta_bytes = CONTROL.to_vec();
    let alpha = zip_of(&[(b"p/Alpha.class", &alpha_bytes)]);
    let beta = zip_of(&[(b"HistoricalControlFlow.class", &beta_bytes)]);

    // (a) Different content, one store.
    let alpha_direct = result_of(alpha.clone(), None);
    let beta_direct = result_of(beta.clone(), None);
    assert_ne!(
        alpha_direct, beta_direct,
        "the two corpora publish the same result, so an answer taken from the other one would not \
         be visible in this test"
    );
    let store = cache(8);
    let alpha_cold = result_of(alpha.clone(), Some(&store));
    let alpha_warm = result_of(alpha.clone(), Some(&store));
    let beta_cold = result_of(beta.clone(), Some(&store));
    let beta_warm = result_of(beta.clone(), Some(&store));
    assert_eq!(
        alpha_cold, alpha_direct,
        "the first snapshot's cached run differs from its own direct run"
    );
    assert_eq!(alpha_warm, alpha_direct);
    assert_eq!(
        beta_cold, beta_direct,
        "the second snapshot was answered with facts read from the first one: the store is not keyed \
         by the content it holds (A18's cross-snapshot isolation)"
    );
    assert_eq!(beta_warm, beta_direct);
    let observed = store.report();
    assert_eq!(
        observed.entries, 2,
        "two different classes under one store are two entries: {observed:?}"
    );
    assert!(
        observed.hits > 0,
        "the store answered nothing, so nothing here is about a cache: {observed:?}"
    );

    // (b) One content, two origins, two snapshots: shared entry, unshared identity.
    let shared = class_bytes(b"p/Shared", 52);
    let first = zip_of(&[(b"a/Shared.class", &shared)]);
    let second = zip_of(&[(b"b/Shared.class", &shared)]);
    let first_direct = result_of(first.clone(), None);
    let second_direct = result_of(second.clone(), None);
    assert_ne!(
        first_direct, second_direct,
        "the two snapshots have to publish different origins for this half to be observable"
    );
    let store = cache(8);
    assert_eq!(result_of(first.clone(), Some(&store)), first_direct);
    assert_eq!(
        result_of(second, Some(&store)),
        second_direct,
        "the second snapshot published the first one's origin or identity, so a hit laundered what \
         the entry could not hold"
    );
    let observed = store.report();
    assert_eq!(
        observed.entries, 1,
        "byte-equal classes at two origins are one content-keyed entry: {observed:?}"
    );
    assert!(
        observed.hits > 0,
        "the byte-equal class was parsed again instead of being shared: {observed:?}"
    );

    // (c) A cancelled request leaves nothing for the other snapshot to be answered with.
    let store = cache(8);
    let engine = Engine::new();
    let snapshot = open_with(alpha, &store);
    let token = CancellationToken::new();
    token.cancel();
    let mut budget =
        Budget::with_cancellation_token(limits(), token).with_facts_cache(store.clone());
    let cancelled = engine
        .query(&snapshot, &full_range_request(&snapshot), &mut budget)
        .expect("a cancelled scan still returns a report");
    assert!(
        matches!(cancelled.execution, ExecutionReport::Cancelled { .. }),
        "the middle run of this case has to be the cancelled one: {:?}",
        cancelled.execution
    );
    assert_eq!(
        store.report().entries,
        0,
        "a cancelled request stored facts, and a later request over another snapshot could be \
         answered with them: {:?}",
        store.report()
    );
    assert_eq!(
        result_of(beta, Some(&store)),
        result_of(
            zip_of(&[(b"HistoricalControlFlow.class", &beta_bytes)]),
            None
        ),
        "the request after the cancelled one was not answered the way its direct run is"
    );
    println!(
        "one store: {} entries over two snapshots, {} hit(s), and nothing left by the cancelled run",
        store.report().entries,
        store.report().hits
    );
}
