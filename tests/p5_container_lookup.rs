//! P5 `bound-container-lookup`: the directed container access, and the bounded reuse of the facts
//! it produces.
//!
//! The change this file belongs to narrows one lookup from "enumerate the whole artifact tree" to
//! "read the container that was declared and the ancestors that reach it", and lets a caller keep
//! the container's verified facts — its backing, its complete central directory and its raw-name
//! locator — on the existing facts-cache handle. What has to be true after it, and what is
//! asserted here:
//!
//! * **A local lookup is local.** The declared container and its ancestors are the only containers
//!   read; an unsearched sibling is materialized zero times, whether it is healthy or damaged.
//!   The explicit whole-tree enumeration still reports every container it finds, damage included,
//!   and a local result never claims whole-tree completeness.
//! * **The locator keeps physical entries apart.** Two records with one raw name stay two records,
//!   in central-directory order, with their own ordinals and origins — direct and warm alike.
//! * **An incomplete directory decides nothing.** A directory that stopped on the budget, on a
//!   cancellation or on damage is a refusal, never "this name is missing".
//! * **Reuse follows the physical identity.** The retained product is keyed by the snapshot's
//!   content identity, the complete container origin and the directory schema; a cache hit is
//!   bounded by the *current* request's depth, cancellation and time, reports its reuse without
//!   replaying old charges, and never turns a stop into a completion.
//! * **Retention is bounded and explained.** Entry and retained-byte limits are both enforced, a
//!   refusal leaves the current request the facts it already read, and `clear`/`drop` release
//!   what the store held.
//! * **The comparison is honest.** Direct, cold, warm and capacity-starved runs are compared on a
//!   *semantic* fingerprint (definitions, order, diagnostics, selection evidence, coverage — no
//!   usage and no cache state) and repeats with one cache initial state are compared on the full
//!   report with only `elapsed_millis` removed.
//!
//! ```text
//! verify:  cargo test --test p5_container_lookup --locked
//! measure: cargo test --test p5_container_lookup --locked -- --ignored --nocapture container_lookup_timings
//! ```
//!
//! Fixtures are generated in this file by `rawzip`'s writer and are pinned by digest, so the shapes
//! the counters are about are reproducible without a committed binary: a flat JAR of sibling
//! *archives* and a WAR whose `WEB-INF/lib` holds one target JAR (STORED or DEFLATED) among many
//! siblings. Nothing here claims a speedup: the counters are deterministic work counts and the
//! timings the ignored measurement prints are same-machine repeated readings, published with their
//! configuration and spread.

use jarde::*;
use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
use serde_json::Value;
use std::io::{Cursor, Write};
use std::time::Instant;

const STORE: u16 = 0;
const DEFLATE: u16 = 8;

/// Sibling libraries the fixtures carry. Enough that a whole-tree walk is visibly more work than a
/// local read, small enough that the fixture builds and runs in milliseconds.
const SIBLINGS: usize = 24;

// -------------------------------------------------------------------------------------------
// Fixtures
// -------------------------------------------------------------------------------------------

fn limits() -> Limits {
    Limits {
        input_bytes: 64 * 1024 * 1024,
        archive_entries: 100_000,
        entry_bytes: 64 * 1024 * 1024,
        read_bytes: 64 * 1024 * 1024,
        class_bytes: 64 * 1024 * 1024,
        attribute_bytes: 64 * 1024 * 1024,
        code_bytes: 64 * 1024 * 1024,
        result_items: 100_000,
        output_bytes: 64 * 1024 * 1024,
        class_headers: 1_000,
        method_bodies: 1_000,
        ir_items: 64 * 1024 * 1024,
        ir_edges: 64 * 1024 * 1024,
        analysis_steps: 64 * 1024 * 1024,
        normalization_clones: 64 * 1024 * 1024,
        nested_depth: 8,
        dependency_depth: 8,
        elapsed_millis: u64::MAX,
    }
}

/// The cheapest class file the reader accepts, with `this_class` set to `this_class`.
///
/// The same builder `tests/p2_resolution.rs` uses: the change moves how a class is *found*, and a
/// fixture that changed the class itself would move the subject of the comparison.
fn class_bytes(this_class: &[u8], major: u16) -> Vec<u8> {
    let object = b"java/lang/Object";
    let mut pool = Vec::new();
    pool.push(1_u8);
    pool.extend_from_slice(
        &u16::try_from(this_class.len())
            .expect("name fits u16")
            .to_be_bytes(),
    );
    pool.extend_from_slice(this_class);
    pool.extend_from_slice(&[7, 0, 1]);
    pool.push(1_u8);
    pool.extend_from_slice(
        &u16::try_from(object.len())
            .expect("object name fits u16")
            .to_be_bytes(),
    );
    pool.extend_from_slice(object);
    pool.extend_from_slice(&[7, 0, 3]);

    let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
    bytes.extend_from_slice(&0_u16.to_be_bytes());
    bytes.extend_from_slice(&major.to_be_bytes());
    bytes.extend_from_slice(&5_u16.to_be_bytes());
    bytes.extend_from_slice(&pool);
    bytes.extend_from_slice(&0x0021_u16.to_be_bytes());
    bytes.extend_from_slice(&2_u16.to_be_bytes());
    bytes.extend_from_slice(&4_u16.to_be_bytes());
    for _ in 0..4 {
        bytes.extend_from_slice(&0_u16.to_be_bytes());
    }
    bytes
}

/// One archive whose entries are written with the named method (STORED or DEFLATE).
fn archive(entries: &[(Vec<u8>, Vec<u8>, u16)]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = ZipArchiveWriter::new(&mut output);
        for (name, data, method) in entries {
            let (mut entry, config) = writer
                .new_file(EntryPath::verbatim(name.clone()))
                .compression_method(CompressionMethod::new(*method))
                .start()
                .expect("the fixture entry starts");
            if *method == DEFLATE {
                let encoder =
                    flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::default());
                let mut sink = config.wrap(encoder);
                sink.write_all(data).expect("the fixture entry is writable");
                let (encoder, descriptor) = sink.finish().expect("the fixture entry closes");
                encoder.finish().expect("the deflate stream finishes");
                entry
                    .finish(descriptor)
                    .expect("the fixture entry finishes");
            } else {
                let mut sink = config.wrap(&mut entry);
                sink.write_all(data).expect("the fixture entry is writable");
                let (_, descriptor) = sink.finish().expect("the fixture entry closes");
                entry
                    .finish(descriptor)
                    .expect("the fixture entry finishes");
            }
        }
        writer.finish().expect("the fixture archive finishes");
    }
    output.into_inner()
}

fn stored(name: &[u8], data: &[u8]) -> (Vec<u8>, Vec<u8>, u16) {
    (name.to_vec(), data.to_vec(), STORE)
}

/// One nested library JAR holding `p/S.class`, with a distinctive extra entry so no two siblings
/// are byte-equal.
fn library(index: usize, class: &[u8], method: u16) -> (Vec<u8>, Vec<u8>, u16) {
    let marker = format!("note-{index:03}.txt");
    let inner = archive(&[
        stored(b"p/S.class", class),
        stored(marker.as_bytes(), format!("library {index}").as_bytes()),
    ]);
    (
        format!("WEB-INF/lib/L{index:03}.jar").into_bytes(),
        inner,
        method,
    )
}

/// The **large flat JAR**: `count` sibling archive entries and one top-level class.
///
/// This is the fixture for "a whole-root enumeration materializes every sibling". The old path had
/// no alternative for a top-level ZIP root; the directed path parses the root's own directory (it
/// is the container that was declared) and materializes none of these nested archives.
fn flat_jar(count: usize) -> Vec<u8> {
    let class = class_bytes(b"p/S", 52);
    let mut entries = Vec::new();
    for index in 0..count {
        entries.push(library(
            index,
            &class,
            if index % 2 == 0 { STORE } else { DEFLATE },
        ));
    }
    entries.push(stored(b"p/Top.class", &class_bytes(b"p/Top", 52)));
    archive(&entries)
}

/// A WAR whose `WEB-INF/lib` holds `SIBLINGS` sibling libraries and one **target** library.
///
/// The target sits beside the siblings so a test can damage a *different* sibling and still address
/// the healthy one. `method` is the compression of the target entry itself (STORED or DEFLATED).
fn war_with_target(method: u16) -> Vec<u8> {
    let class = class_bytes(b"p/S", 52);
    let mut entries = vec![stored(
        b"WEB-INF/classes/App.class",
        &class_bytes(b"App", 52),
    )];
    for index in 0..SIBLINGS {
        entries.push(library(
            index,
            &class,
            if index % 2 == 0 { STORE } else { DEFLATE },
        ));
    }
    entries.push((
        b"WEB-INF/lib/Target.jar".to_vec(),
        archive(&[
            stored(b"p/S.class", &class),
            stored(b"p/Other.class", &class_bytes(b"p/Other", 52)),
        ]),
        method,
    ));
    archive(&entries)
}

/// One WAR with a two-level chain: `lib/outer.jar` holds `inner.jar`, which holds the class.
fn war_with_chain() -> Vec<u8> {
    let class = class_bytes(b"p/S", 52);
    let inner = archive(&[stored(b"p/S.class", &class)]);
    let outer = archive(&[
        stored(b"inner.jar", &inner),
        stored(b"filler.txt", b"nothing to see"),
    ]);
    archive(&[
        stored(b"lib/outer.jar", &outer),
        stored(b"WEB-INF/classes/App.class", &class_bytes(b"App", 52)),
    ])
}

fn digest(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut Budget::new(limits()))
        .expect("the fixture snapshot opens")
}

/// The origin of the fixture's nested container whose leaf entry name is `leaf_name`.
///
/// This is premise work: a caller that declares a container has its origin from somewhere, and the
/// explicit enumeration is the public way to learn one. It runs under its own budget and outside
/// every measured section.
fn origin_named(snapshot: &ArtifactSnapshot, leaf_name: &[u8]) -> ContainerOrigin {
    let mut budget = Budget::new(limits());
    let report = Engine::new()
        .enumerate_artifact_tree(snapshot, &mut budget)
        .expect("the fixture tree enumerates");
    let mut found: Vec<ContainerOrigin> = report
        .containers
        .iter()
        .filter(|container| {
            container
                .origin
                .steps
                .last()
                .is_some_and(|step| step.via_raw_name.0 == leaf_name)
        })
        .map(|container| container.origin.clone())
        .collect();
    assert_eq!(
        found.len(),
        1,
        "the fixture names exactly one container with leaf `{}`",
        String::from_utf8_lossy(leaf_name)
    );
    found.remove(0)
}

// -------------------------------------------------------------------------------------------
// Requests and the comparison vocabulary
// -------------------------------------------------------------------------------------------

fn loader(name: &str) -> LoaderId {
    LoaderId(name.to_string())
}

fn request_at(root: ContainerOrigin, class_name: &[u8]) -> ResolutionRequest {
    let caller = LoadDomain {
        loader: loader("app"),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: vec![LoadRoot::Container {
            origin: root,
            prefix: ArchiveNameBytes(Vec::new()),
        }],
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionRequest {
        environment: ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: caller
                        .roots
                        .iter()
                        .find_map(|root| match root {
                            LoadRoot::Container { origin, .. } => Some(origin.snapshot.clone()),
                            _ => None,
                        })
                        .expect("the request names a tree root"),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: caller.clone(),
            },
            domains: vec![caller],
            providers: Vec::new(),
        },
        target: SymbolRef::Class {
            owner: JvmBytes(class_name.to_vec()),
        },
        use_kind: ReferenceUse::ClassReference,
        caller: CallerContext {
            loader: loader("app"),
            enclosing: None,
        },
        dispatch: None,
    }
}

/// The snapshot's own record of one resolved definition, read through the directed access.
///
/// This is the authoritative entry a read is judged against — never a report the test built
/// itself, which is the point of the forged-metadata case below.
fn authoritative_entry(
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
) -> PhysicalEntry {
    let id = definition
        .entry()
        .expect("the fixture definition is an archive entry");
    let mut budget = Budget::new(limits());
    snapshot
        .container_record(id, &mut budget)
        .expect("the container reads")
        .expect("the record exists")
}

/// One resolution under a budget that carries `cache` when one is named.
fn resolve_with(
    snapshot: &ArtifactSnapshot,
    request: &ResolutionRequest,
    cache: Option<&FactsCache>,
    limits: Limits,
) -> (ResolutionReport, Budget) {
    let mut budget = match cache {
        Some(cache) => Budget::new(limits).with_facts_cache(cache.clone()),
        None => Budget::new(limits),
    };
    let report = Engine::new()
        .resolve_symbol(std::slice::from_ref(snapshot), request, &mut budget)
        .expect("a legal request is answered, not raised");
    (report, budget)
}

fn resolve(snapshot: &ArtifactSnapshot, request: &ResolutionRequest) -> ResolutionReport {
    resolve_with(snapshot, request, None, limits()).0
}

fn usage_of(report: &ResolutionReport) -> UsageSnapshot {
    match &report.execution {
        ExecutionReport::Complete { usage }
        | ExecutionReport::Partial { usage, .. }
        | ExecutionReport::Cancelled { usage }
        | ExecutionReport::Failed { usage, .. } => usage.clone(),
    }
}

fn charged(report: &ResolutionReport, dimension: CountedBudgetDimension) -> u64 {
    usage_of(report).counted_usage(dimension)
}

/// The **semantic fingerprint**: the published result with the measurement planes removed.
///
/// What participates is what the run decided and published — the environment identity, the state,
/// the definitions and candidates in publication order, the coverage, the diagnostics and the
/// evidence — and what is removed is what a *second path* is allowed to change: the request's own
/// charges (`execution.usage`) and the wall clock. The cache's state and counters are not part of a
/// report at all, so they cannot leak in here; they are read from the handle instead. This is not
/// the determinism check: that one is [`full_report_fingerprint`].
fn semantic_fingerprint(report: &ResolutionReport) -> String {
    let mut value = serde_json::to_value(report).expect("the report serializes");
    strip_keys(&mut value, &["usage", "elapsed_millis"]);
    digest(
        serde_json::to_string(&value)
            .expect("the document renders")
            .as_bytes(),
    )
}

/// The **full-report fingerprint**: the whole report with only the wall clock removed.
///
/// This is P5 1.3's existing normalization and the determinism half of the acceptance: two runs of
/// one configuration from the same cache initial state have to agree here field by field, budget
/// counters included. A cache that replayed a charge, skipped one, or let a hit turn a stop into a
/// completion moves a field this fingerprint keeps.
fn full_report_fingerprint(report: &ResolutionReport) -> String {
    let mut value = serde_json::to_value(report).expect("the report serializes");
    strip_keys(&mut value, &["elapsed_millis"]);
    digest(
        serde_json::to_string(&value)
            .expect("the document renders")
            .as_bytes(),
    )
}

/// Removes every object member with one of `keys`, at any depth, returning how many were removed.
fn strip_keys(value: &mut Value, keys: &[&str]) -> usize {
    let mut removed = 0;
    match value {
        Value::Object(map) => {
            for key in keys {
                if map.remove(*key).is_some() {
                    removed += 1;
                }
            }
            for child in map.values_mut() {
                removed += strip_keys(child, keys);
            }
        }
        Value::Array(items) => {
            for item in items {
                removed += strip_keys(item, keys);
            }
        }
        _ => {}
    }
    removed
}

/// A fresh store with room for `entries` products and `retained_bytes` weight.
fn cache(entries: usize, retained_bytes: u64) -> FactsCache {
    FactsCache::current(FactsCapacity::new(entries, retained_bytes))
}

/// What one store has answered so far.
fn counters(cache: &FactsCache) -> FactsReport {
    cache.report()
}

/// The evidence one resolution publishes, as a one-line reading for a failure message.
fn evidence_line(report: &ResolutionReport) -> String {
    serde_json::to_string(&serde_json::json!({
        "state": report.state,
        "resolved": report.resolved,
        "candidates": report.candidates,
        "reads": report.reads,
        "diagnostics": report.diagnostics,
    }))
    .expect("the evidence renders")
}

fn status_of_execution(execution: &ExecutionReport) -> &'static str {
    match execution {
        ExecutionReport::Complete { .. } => "complete",
        ExecutionReport::Partial { .. } => "partial",
        ExecutionReport::Cancelled { .. } => "cancelled",
        ExecutionReport::Failed { .. } => "failed",
    }
}

/// One archive whose stored entry `entry_name` carries one flipped data byte.
///
/// The damage is located through the public enumeration and lands inside the entry's data area, so
/// the central directory still lists the entry and only reading it fails — the fixture a local
/// lookup must not touch and an explicit whole-tree walk must report.
fn damage_entry(bytes: Vec<u8>, entry_name: &[u8]) -> (Vec<u8>, u64) {
    let snapshot = open(bytes.clone());
    let mut budget = Budget::new(limits());
    let report = snapshot.enumerate(&mut budget).expect("the fixture lists");
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.id.raw_name.0 == entry_name)
        .expect("the fixture holds the entry to damage");
    assert!(
        entry.layout.compressed_data.length > 0,
        "the entry to damage has data"
    );
    let offset = entry.layout.compressed_data.start;
    let index = usize::try_from(offset).expect("the offset fits usize");
    let mut damaged = bytes;
    damaged[index] ^= 0xff;
    (damaged, offset)
}

// -------------------------------------------------------------------------------------------
// 1.2/1.3 — the directed access
// -------------------------------------------------------------------------------------------

/// The fixture digests every counter in this file is about, pinned rather than recomputed so a
/// fixture edit fails here instead of quietly moving a published number.
///
/// The pinned bytes moved once, with `bind-prefixed-load-roots`: every class entry of these
/// fixtures has to declare in its own header the name its path states, because a candidate whose
/// `this_class` disagrees with the entry it was found under is refused at that candidate. The
/// shapes and the subjects of the tests below are unchanged.
#[test]
fn the_fixture_shapes_are_pinned_by_digest() {
    let flat = flat_jar(4);
    let stored_target = war_with_target(STORE);
    let deflated_target = war_with_target(DEFLATE);
    let chain = war_with_chain();
    let large = flat_jar(SIBLINGS * 4);
    let methods = war_with_methods();
    println!("flat-jar(4)      {} bytes {}", flat.len(), digest(&flat));
    println!("flat-jar(96)     {} bytes {}", large.len(), digest(&large));
    println!(
        "war with methods {} bytes {}",
        methods.len(),
        digest(&methods)
    );
    println!(
        "war target STORE {} bytes {}",
        stored_target.len(),
        digest(&stored_target)
    );
    println!(
        "war target DEFLATE {} bytes {}",
        deflated_target.len(),
        digest(&deflated_target)
    );
    println!("war chain        {} bytes {}", chain.len(), digest(&chain));
    assert_eq!(
        (
            flat.len(),
            digest(&flat),
            large.len(),
            digest(&large),
            methods.len(),
            digest(&methods),
            stored_target.len(),
            digest(&stored_target),
            deflated_target.len(),
            digest(&deflated_target),
            chain.len(),
            digest(&chain),
        ),
        (
            1_710,
            "402bb412553fa59cb9da6b19f6c42eb9e88b7e34f3a43aa9da14fcd6502296e2".to_string(),
            36_751,
            "45c26bf2052d653ce40e40e59a85b9bd1eb105246b85c6aae5fe21fba0269979".to_string(),
            1_238,
            "8bbc41257b38f1b4a9ef1240d9ea8d270ad31bbfd46aa788d8c0ddbbdc93c5e4".to_string(),
            9_849,
            "82e1139ff956990bfdde9fa75853ce6cb5563eafdca305ae68b4a08f60907297".to_string(),
            9_665,
            "95280d426669488179cdf32d43d37a1a7bc7930cf0b9fd83813ac8709e4a19dd".to_string(),
            782,
            "3461b72c47f3845bc314e36d0348adb82a3101fea46601e793e8f745bfb7bceb".to_string(),
        ),
        "a fixture shape moved; the counters and the digests published for it have to move with it"
    );
}

/// A local lookup reads the declared container and the ancestors that reach it — nothing else.
///
/// The counters are the store's, attached with zero capacity so the request is transparently the
/// direct path: one parse of the root directory (the ancestor), one materialization of the target
/// entry, one parse of the target directory. Not one of the 25 sibling libraries is read.
#[test]
fn a_local_lookup_reads_only_its_container_and_its_ancestors() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    assert_eq!(target.steps.len(), 1, "the target is one level down");

    let probe = cache(0, 0);
    let request = request_at(target.clone(), b"p/S");
    let (report, _) = resolve_with(&snapshot, &request, Some(&probe), limits());
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert_eq!(
        report
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.definition.entry())
            .map(|entry| entry.origin.clone()),
        Some(target.clone()),
        "the definition is the target container's own entry: {}",
        evidence_line(&report)
    );

    let report = counters(&probe);
    assert_eq!(
        (
            report.directory_parses,
            report.nested_materializations,
            report.container_misses
        ),
        (2, 2, 3),
        "a local lookup parsed {} directories and materialized {} nested containers: {report:?}",
        report.directory_parses,
        report.nested_materializations
    );
    assert!(
        report.nested_materialized_bytes > 0,
        "the directed build materialized nothing: {report:?}"
    );
    println!(
        "local lookup: directories parsed {}, nested materialized {} ({} bytes), sibling \
         libraries in the fixture {}; the second materialization is the read's own fallback, \
         because a zero-capacity store kept nothing",
        report.directory_parses,
        report.nested_materializations,
        report.nested_materialized_bytes,
        SIBLINGS
    );
}

/// A damaged sibling is outside a local lookup's coverage — and inside the whole-tree report.
///
/// The damaged sibling's data byte is flipped, so reading it fails its CRC; the local result still
/// resolves through the healthy target, while the explicit whole-tree enumeration reports the
/// container it could not read. The two paths answer different questions with deliberately
/// different coverage, and the local answer is never extended into a whole-tree health claim.
#[test]
fn a_damaged_sibling_does_not_stop_a_local_result_and_the_tree_still_reports_it() {
    let fixture = war_with_target(STORE);
    let (damaged, _offset) = damage_entry(fixture, b"WEB-INF/lib/L000.jar");
    let snapshot = open(damaged);
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");

    let probe = cache(0, 0);
    let request = request_at(target.clone(), b"p/S");
    let (local, _) = resolve_with(&snapshot, &request, Some(&probe), limits());
    assert_eq!(
        local.state,
        Some(ResolutionState::Resolved),
        "a local lookup was stopped by a sibling it never searched: {}",
        evidence_line(&local)
    );
    let report = counters(&probe);
    assert_eq!(
        (report.directory_parses, report.nested_materializations),
        (2, 2),
        "the lookup walked into a sibling: {report:?}"
    );

    let mut budget = Budget::new(limits());
    let tree = Engine::new()
        .enumerate_artifact_tree(&snapshot, &mut budget)
        .expect("the tree enumeration answers with its own evidence");
    assert!(
        !matches!(tree.execution, ExecutionReport::Complete { .. }),
        "the explicit whole-tree enumeration did not report the damaged sibling: {:?}",
        tree.execution
    );
    assert!(
        tree.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "entry_integrity"),
        "the tree's diagnostics name the damage: {:?}",
        tree.diagnostics
    );
    println!(
        "local lookup complete over a damaged sibling; whole-tree enumeration: {} with {} \
         diagnostic(s): {:?}",
        status_of_execution(&tree.execution),
        tree.diagnostics.len(),
        tree.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>()
    );
}

/// Two records with one raw name stay two records: two candidates, their own ordinals, their own
/// origins — on the direct path and on a warm one.
#[test]
fn duplicate_physical_entries_keep_their_ordinals_direct_and_warm() {
    let source = class_bytes(b"p/S", 52);
    let duplicate = archive(&[stored(b"p/S.class", &source), stored(b"p/S.class", &source)]);
    let outer = archive(&[
        stored(b"WEB-INF/lib/Dup.jar", &duplicate),
        stored(b"WEB-INF/classes/App.class", &class_bytes(b"App", 52)),
    ]);
    let snapshot = open(outer);
    let dup = origin_named(&snapshot, b"WEB-INF/lib/Dup.jar");
    let request = request_at(dup.clone(), b"p/S");

    let direct = resolve(&snapshot, &request);
    assert_eq!(direct.state, Some(ResolutionState::Ambiguous));
    assert_eq!(direct.candidates.len(), 2, "{}", evidence_line(&direct));
    let positions: Vec<u64> = direct
        .candidates
        .iter()
        .map(|candidate| {
            candidate
                .definition
                .entry()
                .expect("an archive entry")
                .ordinal
        })
        .collect();
    assert_eq!(
        positions,
        vec![0, 1],
        "the candidate order is central-directory order: {positions:?}"
    );

    let store = cache(8, 1 << 20);
    let (cold, _) = resolve_with(&snapshot, &request, Some(&store), limits());
    let before = counters(&store);
    let (warm, _) = resolve_with(&snapshot, &request, Some(&store), limits());
    let after = counters(&store);
    assert_eq!(
        semantic_fingerprint(&cold),
        semantic_fingerprint(&direct),
        "the cold path changed the answer: {}",
        evidence_line(&cold)
    );
    assert_eq!(
        semantic_fingerprint(&warm),
        semantic_fingerprint(&direct),
        "the warm path merged the duplicate records: {}",
        evidence_line(&warm)
    );
    assert_eq!(
        warm.candidates.len(),
        2,
        "a reuse merged two physical entries by name or content"
    );
    assert!(
        after.container_hits > before.container_hits,
        "the warm request was not answered from retention: {after:?}"
    );
    assert_eq!(
        (
            after.directory_parses,
            after.nested_materializations,
            after.nested_materialized_bytes
        ),
        (
            before.directory_parses,
            before.nested_materializations,
            before.nested_materialized_bytes
        ),
        "the warm request parsed or materialized again: before {before:?}, after {after:?}"
    );
}

/// The target and every ancestor of a two-level chain are reachable, and each level is read once.
#[test]
fn a_two_level_chain_reaches_every_ancestor_once() {
    let snapshot = open(war_with_chain());
    let inner = origin_named(&snapshot, b"inner.jar");
    assert_eq!(inner.steps.len(), 2, "the fixture is two levels deep");
    let probe = cache(0, 0);
    let request = request_at(inner.clone(), b"p/S");
    let (report, _) = resolve_with(&snapshot, &request, Some(&probe), limits());
    assert_eq!(
        report.state,
        Some(ResolutionState::Resolved),
        "{}",
        evidence_line(&report)
    );
    assert_eq!(
        report
            .resolved
            .as_ref()
            .and_then(|resolved| resolved.definition.entry())
            .map(|entry| entry.origin.steps.len()),
        Some(2),
        "{}",
        evidence_line(&report)
    );
    let probe_report = counters(&probe);
    assert_eq!(
        (
            probe_report.directory_parses,
            probe_report.nested_materializations
        ),
        (3, 4),
        "the chain was not read once per level, plus the read's own fallback under a store that \
         retains nothing: {probe_report:?}"
    );

    // With a store that admits the products, the same request reads each level once and the read
    // that follows is answered from retention: no second parse, no second materialization.
    let store = cache(8, 1 << 20);
    let (cold, _) = resolve_with(&snapshot, &request, Some(&store), limits());
    let after_cold = counters(&store);
    assert_eq!(cold.state, Some(ResolutionState::Resolved));
    assert_eq!(
        (
            after_cold.directory_parses,
            after_cold.nested_materializations
        ),
        (3, 2),
        "a retaining store still read a level twice: {after_cold:?}"
    );
    let (warm, _) = resolve_with(&snapshot, &request, Some(&store), limits());
    let after_warm = counters(&store);
    assert_eq!(warm.state, Some(ResolutionState::Resolved));
    assert_eq!(
        (
            after_warm.directory_parses,
            after_warm.nested_materializations,
            after_warm.nested_materialized_bytes
        ),
        (
            after_cold.directory_parses,
            after_cold.nested_materializations,
            after_cold.nested_materialized_bytes
        ),
        "the warm chain parse or materialized again: {after_warm:?}"
    );
}

/// A forged origin chain is refused where the derivation fails, not answered with another
/// container's contents.
#[test]
fn a_forged_container_origin_is_refused() {
    let snapshot = open(war_with_target(STORE));
    let mut forged = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    forged.steps[0].child_container = ContainerId("forged".to_string());
    let request = request_at(forged, b"p/S");
    let report = resolve(&snapshot, &request);
    assert_eq!(
        report.state,
        None,
        "a fabricated container identity decided a lookup: {}",
        evidence_line(&report)
    );
    let codes: Vec<&str> = report
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect();
    assert!(
        codes.contains(&"child_container_mismatch"),
        "the refusal does not name the derivation: {codes:?}"
    );
    println!("forged origin refused with {codes:?}");
}

/// A directory that could not be read completely decides nothing: the lookup is a budget stop
/// with the dimension it hit, never `Missing`.
#[test]
fn an_incomplete_directory_is_never_a_missing_verdict() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let probe = cache(0, 0);
    let request = request_at(target, b"p/S");
    let tight = Limits {
        archive_entries: 1,
        ..limits()
    };
    let (report, _) = resolve_with(&snapshot, &request, Some(&probe), tight);
    assert_ne!(
        report.state,
        Some(ResolutionState::Missing),
        "a stopped directory was read as a negative: {}",
        evidence_line(&report)
    );
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ArchiveEntries
                },
                ..
            }
        ),
        "the stop is not the directory's own: {:?}",
        report.execution
    );
    assert_eq!(
        counters(&probe).directory_parses,
        1,
        "the stopped parse is still one parse"
    );
    println!(
        "an incomplete directory stops the lookup: {}",
        evidence_line(&report)
    );
}

/// The explicit whole-tree enumeration keeps its own coverage: every container, every entry.
#[test]
fn explicit_tree_enumeration_still_reports_every_container() {
    let snapshot = open(flat_jar(4));
    let mut budget = Budget::new(limits());
    let tree = Engine::new()
        .enumerate_artifact_tree(&snapshot, &mut budget)
        .expect("the fixture tree enumerates");
    assert!(matches!(tree.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        tree.containers.len(),
        5,
        "the root plus four sibling archives"
    );
    let nested_entries: usize = tree
        .containers
        .iter()
        .filter(|container| !container.origin.steps.is_empty())
        .map(|container| container.entries.len())
        .sum();
    assert_eq!(nested_entries, 8, "two entries per nested library");

    // …while a local lookup of the root's own class parses the root directory once and
    // materializes no nested archive at all.
    let root = tree
        .containers
        .iter()
        .find(|container| container.origin.steps.is_empty())
        .expect("the root container")
        .origin
        .clone();
    let probe = cache(0, 0);
    let request = request_at(root, b"p/Top");
    let (report, _) = resolve_with(&snapshot, &request, Some(&probe), limits());
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let counters = counters(&probe);
    assert_eq!(
        (counters.directory_parses, counters.nested_materializations),
        (1, 0),
        "a root-level lookup read a nested container: {counters:?}"
    );
}

/// A large flat JAR does not add sibling materializations: the root directory is the ceiling.
///
/// A top-level ZIP root *is* the container that was declared, so its own central directory has to
/// be read — the cold cost does not disappear for the container itself. What must not scale with
/// the number of sibling archives is anything else: the directed lookup materializes none of them
/// at either size, while the records it charges grow only with the container it was asked about.
#[test]
fn a_large_flat_jar_does_not_add_sibling_materializations() {
    for count in [4, SIBLINGS * 4] {
        let snapshot = open(flat_jar(count));
        let root = ContainerOrigin {
            snapshot: snapshot.id().clone(),
            root_container: ContainerId("root".into()),
            steps: Vec::new(),
        };
        let probe = cache(0, 0);
        let request = request_at(root, b"p/Top");
        let (report, budget) = resolve_with(&snapshot, &request, Some(&probe), limits());
        assert_eq!(report.state, Some(ResolutionState::Resolved));
        let counters = counters(&probe);
        assert_eq!(
            (
                counters.directory_parses,
                counters.nested_materializations,
                counters.nested_materialized_bytes
            ),
            (1, 0, 0),
            "a root lookup over {count} siblings did extra work: {counters:?}"
        );
        let records = budget
            .usage()
            .counted_usage(CountedBudgetDimension::ArchiveEntries);
        assert!(
            records >= (count + 1) as u64 && records <= 2 * (count + 1) as u64,
            "a root lookup over {count} siblings charged {records} central-directory records: the \
             declared container's own records are the ceiling and the tree's {count} archives are \
             not part of it"
        );
        println!(
            "flat JAR with {count} sibling archives: directories parsed 1, nested materialized 0, \
             archive_entries {records}"
        );
    }
}

/// The whole-tree reference and the local lookup, measured over one fixture with one vocabulary.
///
/// The explicit enumeration is still the path that answers "what does this tree hold", and it is
/// also what a location used to cost: every container materialized, every directory parsed. The
/// directed lookup answers the narrower question and pays for the narrower set. Both numbers come
/// from the same counters, so the contrast is a count, not a claim about the clock.
#[test]
fn the_whole_tree_reference_and_the_local_lookup_differ_by_the_siblings() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");

    let tree_probe = cache(0, 0);
    let mut budget = Budget::new(limits()).with_facts_cache(tree_probe.clone());
    let tree = Engine::new()
        .enumerate_artifact_tree(&snapshot, &mut budget)
        .expect("the fixture tree enumerates");
    assert!(matches!(tree.execution, ExecutionReport::Complete { .. }));
    let reference = counters(&tree_probe);

    let lookup_probe = cache(0, 0);
    let request = request_at(target, b"p/S");
    let (report, _) = resolve_with(&snapshot, &request, Some(&lookup_probe), limits());
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let local = counters(&lookup_probe);

    assert_eq!(
        reference.nested_materializations as usize,
        SIBLINGS + 1,
        "the reference enumerated {} containers but materialized {}: {reference:?}",
        tree.containers.len(),
        reference.nested_materializations
    );
    assert_eq!(
        local.nested_materializations, 2,
        "the local lookup materialized {} nested containers; only the target (and the read's own \
         fallback under a store that retains nothing) is its business: {local:?}",
        local.nested_materializations
    );
    assert_eq!(
        local.directory_parses, 2,
        "the local lookup parsed {} directories: {local:?}",
        local.directory_parses
    );
    assert!(
        reference.directory_parses as usize > SIBLINGS,
        "the reference did not parse every container: {reference:?}"
    );
    println!(
        "whole tree ({} containers, {} siblings): directories parsed {}, nested materialized {} \
         ({} bytes); local lookup: directories parsed {}, nested materialized {} ({} bytes); the \
         damage proof that no sibling was read is `a_damaged_sibling_does_not_stop_a_local_result_\
         and_the_tree_still_reports_it`",
        tree.containers.len(),
        SIBLINGS,
        reference.directory_parses,
        reference.nested_materializations,
        reference.nested_materialized_bytes,
        local.directory_parses,
        local.nested_materializations,
        local.nested_materialized_bytes,
    );
}

// -------------------------------------------------------------------------------------------
// 2.1/2.2 — bounded reuse of the verified facts
// -------------------------------------------------------------------------------------------

/// Later requests of one snapshot reuse the container a first request retained.
#[test]
fn a_second_request_reuses_the_container_it_does_not_rebuild() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let store = cache(8, 1 << 20);

    let (cold, _) = resolve_with(
        &snapshot,
        &request_at(target.clone(), b"p/S"),
        Some(&store),
        limits(),
    );
    let after_cold = counters(&store);
    assert_eq!(cold.state, Some(ResolutionState::Resolved));
    assert_eq!(
        after_cold.containers, 2,
        "both the ancestor and the target should be retained: {after_cold:?}"
    );

    // A **different name in the same container** is the multi-method case: the container is
    // reused and only the name's own class entry is read.
    let (warm, budget) = resolve_with(
        &snapshot,
        &request_at(target.clone(), b"p/Other"),
        Some(&store),
        limits(),
    );
    let after_warm = counters(&store);
    assert_eq!(warm.state, Some(ResolutionState::Resolved));
    assert_eq!(
        (
            after_warm.directory_parses,
            after_warm.nested_materializations
        ),
        (
            after_cold.directory_parses,
            after_cold.nested_materializations
        ),
        "the second name rebuilt the container: before {after_cold:?}, after {after_warm:?}"
    );
    assert!(
        after_warm.container_hits > after_cold.container_hits,
        "the second lookup was not answered from retention: {after_warm:?}"
    );
    assert!(
        budget
            .usage()
            .counted_usage(CountedBudgetDimension::ClassHeaders)
            > 0,
        "the second name published a definition without reading a header"
    );
    println!(
        "two names of one container: directories parsed {}, nested materialized {}, hits {}",
        after_warm.directory_parses, after_warm.nested_materializations, after_warm.container_hits
    );
}

/// The key binds the input, the chain and the declaration: each one changed is a miss.
#[test]
fn a_changed_content_chain_or_declaration_misses() {
    let first_bytes = war_with_target(STORE);
    let first = open(first_bytes.clone());
    let first_target = origin_named(&first, b"WEB-INF/lib/Target.jar");
    let store = cache(16, 1 << 20);
    let _ = resolve_with(
        &first,
        &request_at(first_target.clone(), b"p/S"),
        Some(&store),
        limits(),
    );
    let admitted = counters(&store);

    // (a) Content: one more sibling entry moves the snapshot's content identity, so the retained
    //     directory cannot be the new input's facts.
    let mut entries: Vec<(Vec<u8>, Vec<u8>, u16)> = Vec::new();
    let class = class_bytes(b"p/S", 52);
    for index in 0..SIBLINGS {
        entries.push(library(index, &class, STORE));
    }
    entries.push((
        b"WEB-INF/lib/Target.jar".to_vec(),
        archive(&[
            stored(b"p/S.class", &class),
            stored(b"p/Other.class", &class_bytes(b"p/Other", 52)),
        ]),
        STORE,
    ));
    entries.push(stored(
        b"WEB-INF/lib/Extra.jar",
        &archive(&[stored(b"x", b"y")]),
    ));
    entries.push(stored(
        b"WEB-INF/classes/App.class",
        &class_bytes(b"App", 52),
    ));
    let changed = open(archive(&entries));
    assert_ne!(
        changed.id(),
        first.id(),
        "the fixture change did not move the snapshot identity"
    );
    let changed_target = origin_named(&changed, b"WEB-INF/lib/Target.jar");
    let (report, _) = resolve_with(
        &changed,
        &request_at(changed_target, b"p/S"),
        Some(&store),
        limits(),
    );
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let after_content = counters(&store);
    assert_eq!(
        after_content.directory_parses,
        admitted.directory_parses + 2,
        "the changed content reused the old snapshot's facts: {after_content:?}"
    );

    // (b) Chain: another container of the *same* snapshot is another product, not a hit on the
    //     first one.
    let sibling = origin_named(&first, b"WEB-INF/lib/L001.jar");
    let (report, _) = resolve_with(&first, &request_at(sibling, b"p/S"), Some(&store), limits());
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let after_chain = counters(&store);
    assert_eq!(
        after_chain.directory_parses,
        after_content.directory_parses + 1,
        "another container's own directory was not parsed (the shared ancestor should be the only \
         reuse here): {after_chain:?}"
    );

    // (c) Declaration: a store read under another registry version discards the products it holds
    //     instead of serving them, exactly as it does for a CP/Header entry. The directory schema
    //     is the third key dimension and is compiled in, so this is the dimension a run can flip.
    let declared = cache(16, 1 << 20);
    let _ = resolve_with(
        &first,
        &request_at(first_target.clone(), b"p/S"),
        Some(&declared),
        limits(),
    );
    assert_eq!(
        counters(&declared).containers,
        2,
        "two products were retained"
    );
    let other = declared.over(FactsIdentity::new(
        declared.identity().registry + 1,
        declared.identity().format,
    ));
    let (report, _) = resolve_with(
        &first,
        &request_at(first_target, b"p/S"),
        Some(&other),
        limits(),
    );
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let discarded = counters(&declared);
    assert!(
        discarded.discarded_registry >= 2,
        "a store read under another declaration served its old products: {discarded:?}"
    );
    println!(
        "changed content, chain and declaration each missed; the declaration change discarded {} \
         products and rebuilt {}",
        discarded.discarded_registry, discarded.container_stored
    );
}

/// A container that stopped is never published: the store keeps nothing, and a later complete
/// request parses it again.
#[test]
fn only_a_complete_container_is_published() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let store = cache(16, 1 << 20);

    let tight = Limits {
        archive_entries: 3,
        ..limits()
    };
    let (stopped, _) = resolve_with(
        &snapshot,
        &request_at(target.clone(), b"p/S"),
        Some(&store),
        tight,
    );
    assert_ne!(stopped.state, Some(ResolutionState::Resolved));
    let after_stop = counters(&store);
    assert_eq!(
        (after_stop.containers, after_stop.container_stored),
        (0, 0),
        "a stopped directory was published as a complete one: {after_stop:?}"
    );

    let (report, _) = resolve_with(
        &snapshot,
        &request_at(target, b"p/S"),
        Some(&store),
        limits(),
    );
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let after_full = counters(&store);
    assert_eq!(
        after_full.directory_parses,
        after_stop.directory_parses + 2,
        "the complete request did not parse the directory it never retained: {after_full:?}"
    );
}

/// A cancelled or damaged build publishes nothing.
#[test]
fn a_cancelled_or_damaged_container_is_not_published() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let request = request_at(target.clone(), b"p/S");

    // (a) Cancelled before anything was read.
    let cancelled = cache(8, 1 << 20);
    let token = CancellationToken::new();
    token.cancel();
    let mut budget =
        Budget::with_cancellation_token(limits(), token).with_facts_cache(cancelled.clone());
    let report = Engine::new()
        .resolve_symbol(std::slice::from_ref(&snapshot), &request, &mut budget)
        .expect("a cancelled request is answered, not raised");
    assert!(matches!(
        report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    let after_cancel = counters(&cancelled);
    assert_eq!(
        (after_cancel.containers, after_cancel.container_stored),
        (0, 0),
        "a cancelled build published a container: {after_cancel:?}"
    );

    // (b) A sibling that cannot be read is never built, so it is never published either: the store
    //     holds the ancestor and the target — exactly the two containers the lookup needed — and
    //     nothing about the damaged library.
    let damaged_store = cache(8, 1 << 20);
    let (damaged_fixture, _) = damage_entry(war_with_target(STORE), b"WEB-INF/lib/L000.jar");
    let damaged = open(damaged_fixture);
    let damaged_target = origin_named(&damaged, b"WEB-INF/lib/Target.jar");
    let (report, _) = resolve_with(
        &damaged,
        &request_at(damaged_target, b"p/S"),
        Some(&damaged_store),
        limits(),
    );
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let after_damage = counters(&damaged_store);
    assert_eq!(
        after_damage.containers, 2,
        "the store holds containers other than the ancestor and the target: {after_damage:?}"
    );
    assert_eq!(
        after_damage.directory_parses, 2,
        "the lookup parsed a directory other than the ancestor's and the target's: {after_damage:?}"
    );
    println!(
        "cancelled build: {} containers stored; local lookup beside a damaged sibling: {} \
         containers stored (ancestor + target), {}",
        after_cancel.container_stored,
        after_damage.container_stored,
        report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
}

/// Both bounds are enforced, and the byte bound is the sum of the products' own weights.
#[test]
fn the_entry_and_byte_bounds_are_both_enforced() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let request = request_at(target, b"p/S");

    // (a) The entry bound: one entry is not enough for the ancestor and the target.
    let tiny = cache(1, u64::MAX);
    let _ = resolve_with(&snapshot, &request, Some(&tiny), limits());
    let report = counters(&tiny);
    assert_eq!(report.containers, 1, "{report:?}");
    assert!(
        report.refused_capacity >= 1,
        "the entry bound refused nothing: {report:?}"
    );

    // (b) The byte bound: with a roomy entry limit, the same request either fits or is refused by
    //     the byte bound alone, and the weight the store reports is what it holds.
    let roomy = cache(8, u64::MAX);
    let (complete, _) = resolve_with(&snapshot, &request, Some(&roomy), limits());
    assert_eq!(complete.state, Some(ResolutionState::Resolved));
    let admitted = counters(&roomy);
    assert!(admitted.retained_bytes > 0, "{admitted:?}");
    assert!(
        admitted.retained_bytes >= snapshot.len(),
        "the root's backing is part of the retained weight: {admitted:?}"
    );
    let exact = cache(8, admitted.retained_bytes);
    let (again, _) = resolve_with(&snapshot, &request, Some(&exact), limits());
    assert_eq!(again.state, Some(ResolutionState::Resolved));
    assert_eq!(
        counters(&exact).retained_bytes,
        admitted.retained_bytes,
        "the same products weigh something else in an identically shaped store"
    );
    let half = cache(8, admitted.retained_bytes / 2);
    let (refused, _) = resolve_with(&snapshot, &request, Some(&half), limits());
    assert_eq!(refused.state, Some(ResolutionState::Resolved));
    let refusal = counters(&half);
    assert_eq!(
        refusal.containers, 1,
        "a byte-bounded store kept more than it could hold: {refusal:?}"
    );
    assert!(
        refusal.refused_capacity_bytes >= 1 && refusal.refused_capacity == 0,
        "the refusal is not attributed to the byte bound: {refusal:?}"
    );

    // (c) A shared backing is counted once: a second request of the same container rewrites the
    //     same product and the retained weight does not grow.
    let (repeat, _) = resolve_with(&snapshot, &request, Some(&roomy), limits());
    assert_eq!(repeat.state, Some(ResolutionState::Resolved));
    assert_eq!(
        counters(&roomy).retained_bytes,
        admitted.retained_bytes,
        "re-reading the same container grew the retained weight"
    );
    println!(
        "bounds: entries {} / {}, retained {} bytes, refusals entries {} bytes {}",
        admitted.entries,
        admitted.capacity.entries,
        admitted.retained_bytes,
        admitted.refused_capacity,
        admitted.refused_capacity_bytes
    );
}

/// A refusal keeps the current request's work: it is not a reason to read anything again.
#[test]
fn a_capacity_refusal_does_not_rescan_the_request() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let refused = cache(0, 0);
    let (report, _) = resolve_with(
        &snapshot,
        &request_at(target, b"p/S"),
        Some(&refused),
        limits(),
    );
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    let counters = counters(&refused);
    assert_eq!(
        (counters.containers, counters.container_stored),
        (0, 0),
        "a zero-capacity store retained something: {counters:?}"
    );
    assert!(counters.refused_capacity > 0, "{counters:?}");
    assert_eq!(
        counters.directory_parses, 2,
        "the refused request read its container again instead of using what it built: {counters:?}"
    );
}

/// Every handle reads one store, and clearing it releases what it held.
#[test]
fn clearing_the_store_releases_the_products() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let store = cache(8, 1 << 20);
    let handle = store.clone();
    let _ = resolve_with(
        &snapshot,
        &request_at(target.clone(), b"p/S"),
        Some(&store),
        limits(),
    );
    assert_eq!(
        counters(&handle).containers,
        2,
        "two products were retained"
    );
    store.clear();
    let cleared = counters(&handle);
    assert_eq!(
        (cleared.entries, cleared.containers, cleared.retained_bytes),
        (0, 0, 0),
        "clear did not release the store: {cleared:?}"
    );
    let (report, _) = resolve_with(
        &snapshot,
        &request_at(target, b"p/S"),
        Some(&handle),
        limits(),
    );
    assert_eq!(report.state, Some(ResolutionState::Resolved));
    assert!(
        counters(&handle).directory_parses > cleared.directory_parses,
        "a cleared store answered from products it no longer held"
    );
    assert!(
        Budget::new(limits()).facts_cache().is_none(),
        "a budget without a handle now carries one"
    );
}

// -------------------------------------------------------------------------------------------
// 2.3/2.4 — the selected-entry read, and the limits the hit obeys
// -------------------------------------------------------------------------------------------

/// The warm selected-entry read skips the directory and the parent inflate, and still verifies the
/// class it returns.
///
/// The second request runs with **zero** `archive_entries`: a directory scan of any size would stop
/// it. It completes, so no directory was scanned; the class read it still performs is charged, so
/// the result is not a replayed answer.
#[test]
fn a_warm_selected_entry_read_neither_scan_again_nor_replays_a_charge() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let store = cache(8, 1 << 20);
    let (cold, _) = resolve_with(
        &snapshot,
        &request_at(target.clone(), b"p/S"),
        Some(&store),
        limits(),
    );
    assert_eq!(cold.state, Some(ResolutionState::Resolved));

    let no_directory = Limits {
        archive_entries: 0,
        ..limits()
    };
    let (warm, budget) = resolve_with(
        &snapshot,
        &request_at(target, b"p/S"),
        Some(&store),
        no_directory,
    );
    assert_eq!(
        warm.state,
        Some(ResolutionState::Resolved),
        "a warm lookup still scanned a directory: {}",
        evidence_line(&warm)
    );
    let usage = budget.usage();
    assert_eq!(
        usage.counted_usage(CountedBudgetDimension::ArchiveEntries),
        0,
        "the warm path charged central-directory records"
    );
    assert!(
        usage.counted_usage(CountedBudgetDimension::ReadBytes) > 0
            && usage.counted_usage(CountedBudgetDimension::EntryBytes) > 0,
        "the warm path returned a class without reading it: {usage:?}"
    );
    assert_eq!(
        usage.counted_usage(CountedBudgetDimension::ClassHeaders),
        1,
        "the warm path published a definition without one header read"
    );

    // The same warm store refuses forged metadata: the retained directory is the authority, and
    // the class entry's own integrity is still checked against it.
    let definition = &warm
        .resolved
        .as_ref()
        .expect("the warm request resolved")
        .definition;
    let entry = authoritative_entry(&snapshot, definition);
    let mut forged = entry.clone();
    forged.compression = EntryCompression::Deflated;
    let mut budget = Budget::new(limits()).with_facts_cache(store.clone());
    let error = snapshot
        .read_entry(&forged, &mut budget)
        .expect_err("forged metadata was read as if it were the record");
    assert!(
        format!("{error}").contains("entry_metadata_mismatch"),
        "the refusal does not name the mismatch: {error}"
    );
    let mut honest = Budget::new(limits()).with_facts_cache(store.clone());
    let materialized = snapshot
        .read_entry(&entry, &mut honest)
        .expect("the retained record's own entry is readable");
    assert_eq!(
        digest(&materialized.bytes),
        materialized.content_digest.0,
        "the read returned bytes it did not digest"
    );
    println!(
        "warm read: archive_entries {} read_bytes {} entry_bytes {} class_headers {}",
        usage.counted_usage(CountedBudgetDimension::ArchiveEntries),
        usage.counted_usage(CountedBudgetDimension::ReadBytes),
        usage.counted_usage(CountedBudgetDimension::EntryBytes),
        usage.counted_usage(CountedBudgetDimension::ClassHeaders)
    );
}

/// A verified DEFLATED backing saves the directory and the parent inflate, while the class read
/// that still happens is still charged.
#[test]
fn a_verified_deflated_backing_saves_work_without_granting_a_budget() {
    let snapshot = open(war_with_target(DEFLATE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let store = cache(8, 1 << 20);
    let (cold, cold_budget) = resolve_with(
        &snapshot,
        &request_at(target.clone(), b"p/S"),
        Some(&store),
        limits(),
    );
    let (warm, warm_budget) = resolve_with(
        &snapshot,
        &request_at(target, b"p/S"),
        Some(&store),
        limits(),
    );
    assert_eq!(cold.state, Some(ResolutionState::Resolved));
    assert_eq!(warm.state, Some(ResolutionState::Resolved));
    assert_eq!(
        semantic_fingerprint(&cold),
        semantic_fingerprint(&warm),
        "the warm answer differs: {}",
        evidence_line(&warm)
    );
    let (cold, warm) = (cold_budget.usage(), warm_budget.usage());
    for dimension in [
        CountedBudgetDimension::ArchiveEntries,
        CountedBudgetDimension::ReadBytes,
        CountedBudgetDimension::EntryBytes,
    ] {
        assert!(
            warm.counted_usage(dimension) < cold.counted_usage(dimension),
            "the warm path did not save {dimension:?}: cold {} warm {}",
            cold.counted_usage(dimension),
            warm.counted_usage(dimension)
        );
    }
    assert!(
        warm.counted_usage(CountedBudgetDimension::ReadBytes) > 0
            && warm.counted_usage(CountedBudgetDimension::EntryBytes) > 0,
        "the class read was waived: {warm:?}"
    );
    println!(
        "DEFLATED backing: archive_entries {} -> {}, read_bytes {} -> {}, entry_bytes {} -> {}",
        cold.counted_usage(CountedBudgetDimension::ArchiveEntries),
        warm.counted_usage(CountedBudgetDimension::ArchiveEntries),
        cold.counted_usage(CountedBudgetDimension::ReadBytes),
        warm.counted_usage(CountedBudgetDimension::ReadBytes),
        cold.counted_usage(CountedBudgetDimension::EntryBytes),
        warm.counted_usage(CountedBudgetDimension::EntryBytes)
    );
}

/// A pre-cancelled warm request stays cancelled; a hit is not a way past the request's own state.
#[test]
fn a_precancelled_warm_request_is_still_cancelled() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let store = cache(8, 1 << 20);
    let _ = resolve_with(
        &snapshot,
        &request_at(target.clone(), b"p/S"),
        Some(&store),
        limits(),
    );
    let token = CancellationToken::new();
    token.cancel();
    let mut budget =
        Budget::with_cancellation_token(limits(), token).with_facts_cache(store.clone());
    let report = Engine::new()
        .resolve_symbol(
            std::slice::from_ref(&snapshot),
            &request_at(target, b"p/S"),
            &mut budget,
        )
        .expect("a cancelled request is answered, not raised");
    assert_eq!(report.state, None, "{}", evidence_line(&report));
    assert!(
        matches!(report.execution, ExecutionReport::Cancelled { .. }),
        "a hit replaced the cancellation: {:?}",
        report.execution
    );
}

/// An expired clock is not restored by a hit.
#[test]
fn an_expired_warm_request_stays_expired() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let store = cache(8, 1 << 20);
    let _ = resolve_with(
        &snapshot,
        &request_at(target.clone(), b"p/S"),
        Some(&store),
        limits(),
    );
    let expired = Limits {
        elapsed_millis: 0,
        ..limits()
    };
    let (report, _) = resolve_with(
        &snapshot,
        &request_at(target, b"p/S"),
        Some(&store),
        expired,
    );
    assert_ne!(
        report.state,
        Some(ResolutionState::Resolved),
        "an expired request completed from memory: {}",
        evidence_line(&report)
    );
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ElapsedMillis
                },
                ..
            }
        ),
        "an expired request completed from memory: {:?}",
        report.execution
    );
}

/// A budget that already stopped is not restarted by a hit.
#[test]
fn an_already_terminated_budget_is_not_restarted() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let store = cache(8, 1 << 20);
    let _ = resolve_with(
        &snapshot,
        &request_at(target.clone(), b"p/S"),
        Some(&store),
        limits(),
    );
    // The dimension is one the warm path still charges: the class read itself. A hit skips the
    // *parse*, never the read, so a budget that cannot afford the read is a budget that stops.
    let terminated = Limits {
        read_bytes: 0,
        ..limits()
    };
    let mut budget = Budget::new(terminated).with_facts_cache(store.clone());
    let request = request_at(target, b"p/S");
    let first = Engine::new()
        .resolve_symbol(std::slice::from_ref(&snapshot), &request, &mut budget)
        .expect("a stopped request is answered, not raised");
    assert_ne!(first.state, Some(ResolutionState::Resolved));
    let second = Engine::new()
        .resolve_symbol(std::slice::from_ref(&snapshot), &request, &mut budget)
        .expect("a stopped request is answered, not raised");
    assert_ne!(
        second.state,
        Some(ResolutionState::Resolved),
        "a hit restarted a terminated budget: {}",
        evidence_line(&second)
    );
    assert!(
        !matches!(second.execution, ExecutionReport::Complete { .. }),
        "a hit turned a stop into a completion: {:?}",
        second.execution
    );
}

/// A depth limit narrower than the one the facts were built under still applies.
#[test]
fn a_warm_container_obeys_the_current_depth_limit() {
    let snapshot = open(war_with_chain());
    let inner = origin_named(&snapshot, b"inner.jar");
    let store = cache(8, 1 << 20);
    let (cold, _) = resolve_with(
        &snapshot,
        &request_at(inner.clone(), b"p/S"),
        Some(&store),
        limits(),
    );
    assert_eq!(cold.state, Some(ResolutionState::Resolved));
    let shallow = Limits {
        nested_depth: 1,
        ..limits()
    };
    let (report, _) = resolve_with(&snapshot, &request_at(inner, b"p/S"), Some(&store), shallow);
    assert_ne!(
        report.state,
        Some(ResolutionState::Resolved),
        "a hit bypassed the current depth limit: {}",
        evidence_line(&report)
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("nested_depth")
                || diagnostic.code.contains("nested_depth")),
        "the refusal does not name the depth limit: {:?}",
        report.diagnostics
    );
    assert!(
        matches!(
            report.execution,
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::NestedDepth
                },
                ..
            }
        ),
        "the warm request bypassed the depth limit: {:?}",
        report.execution
    );
}

/// A warm read still obeys the output limit of the request that asked for it.
#[test]
fn a_warm_read_obeys_the_current_output_limit() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let store = cache(8, 1 << 20);
    let (cold, _) = resolve_with(
        &snapshot,
        &request_at(target.clone(), b"p/S"),
        Some(&store),
        limits(),
    );
    let definition = &cold
        .resolved
        .as_ref()
        .expect("the cold request resolved")
        .definition;
    let entry = authoritative_entry(&snapshot, definition);
    let class_len = definition.class_bytes.length;
    let tight = Limits {
        output_bytes: class_len - 1,
        ..limits()
    };
    let mut budget = Budget::new(tight).with_facts_cache(store.clone());
    let error = snapshot
        .read_entry(&entry, &mut budget)
        .expect_err("a warm read exceeded its own output limit");
    assert!(
        format!("{error}").contains("OutputBytes"),
        "the refusal does not name the output limit: {error}"
    );
    assert_eq!(
        budget
            .usage()
            .counted_usage(CountedBudgetDimension::OutputBytes),
        0,
        "a refused read charged output it never returned"
    );
}

// -------------------------------------------------------------------------------------------
// 3.1 — the off/cold/warm/capacity comparison
// -------------------------------------------------------------------------------------------

/// The committed ECJ 4.6.1 / 52.0 class inside a nested library, beside one sibling.
///
/// The class is the repository's own fixture (`tests/fixtures/historical/`, compiled from
/// `HistoricalControlFlow.java` with the provenance recorded in that directory's README), so the
/// methods a multi-method request presents are real bodies and not a synthetic shape.
fn war_with_methods() -> Vec<u8> {
    let class: &[u8] =
        include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");
    let target = archive(&[stored(b"HistoricalControlFlow.class", class)]);
    let sibling = archive(&[stored(b"HistoricalControlFlow.class", class)]);
    archive(&[
        stored(b"WEB-INF/lib/Target.jar", &target),
        stored(b"WEB-INF/lib/Sibling.jar", &sibling),
    ])
}

/// The name the historical fixture's own `this_class` declares.
///
/// A member request verifies that the definition a loader claims really declares the class that
/// loader resolves (`resolution_definition_unbound` otherwise), so the fixture's entry name is the
/// class's own name and not a synthetic one.
const HISTORICAL_NAME: &[u8] = b"HistoricalControlFlow";

/// The definition a class name resolves to at one declared container.
fn definition_at(snapshot: &ArtifactSnapshot, origin: &ContainerOrigin) -> PhysicalDefinitionId {
    let report = resolve(snapshot, &request_at(origin.clone(), HISTORICAL_NAME));
    report
        .resolved
        .expect("the fixture's class resolves")
        .definition
}

fn method_request(
    environment: ResolutionEnvironment,
    owner: PhysicalDefinitionId,
    name: &[u8],
    descriptor: &[u8],
) -> MethodAnalysisRequest {
    MethodAnalysisRequest {
        environment,
        method: PhysicalMethodId {
            owner,
            name: JvmBytes(name.to_vec()),
            descriptor: JvmBytes(descriptor.to_vec()),
        },
        stages: AnalysisStage::ALL.to_vec(),
    }
}

/// One `recover_method` request under a budget that carries `cache` when one is named.
fn recover_with(
    snapshot: &ArtifactSnapshot,
    request: &MethodAnalysisRequest,
    cache: Option<&FactsCache>,
    limits: Limits,
) -> (RecoveredMethod, Budget) {
    let mut budget = match cache {
        Some(cache) => Budget::new(limits).with_facts_cache(cache.clone()),
        None => Budget::new(limits),
    };
    let recovered = Engine::new()
        .recover_method(std::slice::from_ref(snapshot), request, &mut budget)
        .expect("a legal request is answered, not raised");
    (recovered, budget)
}

/// The **semantic** fingerprint of one recovery: the published result with the measurement planes
/// removed, exactly like [`semantic_fingerprint`] over a resolution report.
fn method_semantic_fingerprint(recovered: &RecoveredMethod) -> String {
    let mut value = serde_json::to_value(recovered).expect("the recovery serializes");
    strip_keys(&mut value, &["usage", "elapsed_millis"]);
    digest(
        serde_json::to_string(&value)
            .expect("the document renders")
            .as_bytes(),
    )
}

/// The **full-report** fingerprint of one recovery: everything but the wall clock.
fn method_full_report_fingerprint(recovered: &RecoveredMethod) -> String {
    let mut value = serde_json::to_value(recovered).expect("the recovery serializes");
    strip_keys(&mut value, &["elapsed_millis"]);
    digest(
        serde_json::to_string(&value)
            .expect("the document renders")
            .as_bytes(),
    )
}

fn method_status(recovered: &RecoveredMethod) -> &'static str {
    status_of_execution(&recovered.analysis().execution)
}

/// Two independent runs of one **resolution** from the same cache initial state are field-identical
/// but for the wall clock — the full-report check over the local path, with the charges in it.
#[test]
fn identically_warmed_resolutions_publish_the_same_full_report() {
    let snapshot = open(war_with_target(STORE));
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let request = request_at(target, b"p/S");

    let first_store = cache(8, 1 << 20);
    let second_store = cache(8, 1 << 20);
    let (first_cold, first_cold_budget) =
        resolve_with(&snapshot, &request, Some(&first_store), limits());
    let (second_cold, second_cold_budget) =
        resolve_with(&snapshot, &request, Some(&second_store), limits());
    let (first_warm, first_warm_budget) =
        resolve_with(&snapshot, &request, Some(&first_store), limits());
    let (second_warm, _) = resolve_with(&snapshot, &request, Some(&second_store), limits());

    assert_eq!(
        full_report_fingerprint(&first_cold),
        full_report_fingerprint(&second_cold),
        "two cold resolutions of one configuration differ"
    );
    assert_eq!(
        full_report_fingerprint(&first_warm),
        full_report_fingerprint(&second_warm),
        "two warm resolutions of one configuration differ"
    );
    // The full-report check carries the charges, which is what makes it a different (and stricter)
    // statement than the semantic one: the warm run's own counted work is in the document.
    assert_eq!(
        charged(&first_warm, CountedBudgetDimension::ReadBytes),
        charged(&second_warm, CountedBudgetDimension::ReadBytes),
        "the two warm resolutions charged different reads"
    );
    assert!(
        charged(&first_warm, CountedBudgetDimension::ClassHeaders) > 0,
        "the warm resolution published a definition without reading a header"
    );
    assert_ne!(
        full_report_fingerprint(&first_cold),
        full_report_fingerprint(&first_warm),
        "a warm run has to differ from its own cold run in the charges the full report keeps; if \
         it does not, the cache changed nothing and this comparison is vacuous"
    );
    println!(
        "identically warmed resolutions: full report equal (only elapsed_millis excluded); \
         archive_entries cold {} warm {}",
        usage_of(&first_cold).counted_usage(CountedBudgetDimension::ArchiveEntries),
        usage_of(&first_warm).counted_usage(CountedBudgetDimension::ArchiveEntries)
    );
    let _ = first_cold_budget;
    let _ = second_cold_budget;
    let _ = first_warm_budget;
}

/// A same-method request over one container answers identically on all four paths.
///
/// `off` is no handle at all, `cold` the first request through an empty store, `warm` the second
/// request through the same store, and `starved` a store that admits nothing. The four runs have
/// to agree on everything the result *is*; only their charges and the store's counters may move.
#[test]
fn the_four_paths_agree_on_the_semantic_fingerprint() {
    let snapshot = open(war_with_methods());
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let owner = definition_at(&snapshot, &target);
    let environment = request_at(target.clone(), HISTORICAL_NAME).environment;
    let request = method_request(environment, owner.clone(), b"add", b"(II)I");

    let (off, off_budget) = recover_with(&snapshot, &request, None, limits());
    assert_eq!(
        method_status(&off),
        "complete",
        "{:?}",
        serde_json::json!({
            "diagnostics": off.analysis().diagnostics,
            "execution": off.analysis().execution,
        })
    );

    let store = cache(8, 1 << 20);
    let (cold, _) = recover_with(&snapshot, &request, Some(&store), limits());
    let (warm, warm_budget) = recover_with(&snapshot, &request, Some(&store), limits());
    let starved = cache(0, 0);
    let (starved_run, _) = recover_with(&snapshot, &request, Some(&starved), limits());

    for (label, run) in [("cold", &cold), ("warm", &warm), ("starved", &starved_run)] {
        assert_eq!(
            method_status(run),
            "complete",
            "the {label} row did not complete"
        );
        assert_eq!(
            method_semantic_fingerprint(run),
            method_semantic_fingerprint(&off),
            "the {label} row's result differs from the direct one"
        );
    }
    assert!(
        method_full_report_fingerprint(&off) != method_semantic_fingerprint(&off),
        "the two fingerprints are the same check under two names"
    );
    assert!(
        off_budget
            .usage()
            .counted_usage(CountedBudgetDimension::ArchiveEntries)
            > 0,
        "the direct recovery scanned no directory, so this comparison is vacuous"
    );
    assert_eq!(
        warm_budget
            .usage()
            .counted_usage(CountedBudgetDimension::ArchiveEntries),
        0,
        "the warm recovery still scanned a directory"
    );
    // The warm run's own read: the definition it selected is answered from the read the cold run
    // performed (`reuse-selected-class-read`), so the entry access is gone — and what it publishes
    // is still the very definition the request named, checked against the bytes that read handed
    // over. The store's own counters are what say the answering happened: a warm run that charged no
    // read and consulted no store would be a run that never verified the class at all.
    assert_eq!(
        warm_budget
            .usage()
            .counted_usage(CountedBudgetDimension::ReadBytes),
        0,
        "the warm recovery read the definition's entry itself: {:?}",
        warm_budget.usage()
    );
    assert_eq!(
        warm.analysis().reads.last().map(|read| &read.definition),
        Some(&owner),
        "the warm run published a definition other than the one it was asked for"
    );
    assert_eq!(
        store.report().definition_read_hits,
        1,
        "the warm run paid for no read and the store answered none: {:?}",
        store.report()
    );
    println!(
        "off/cold/warm/starved all complete with one semantic fingerprint; archive_entries off {} \
         warm {}, read_bytes off {} warm {}",
        off_budget
            .usage()
            .counted_usage(CountedBudgetDimension::ArchiveEntries),
        warm_budget
            .usage()
            .counted_usage(CountedBudgetDimension::ArchiveEntries),
        off_budget
            .usage()
            .counted_usage(CountedBudgetDimension::ReadBytes),
        warm_budget
            .usage()
            .counted_usage(CountedBudgetDimension::ReadBytes)
    );
}

/// Two independent runs from the same cache initial state produce the same full report.
///
/// This is the determinism half of the acceptance: the two warm runs differ in their wall clock
/// and in nothing else — budget counters included. The cache state they start from is stated: an
/// empty store of the same capacity, warmed by the same cold request.
#[test]
fn identically_warmed_repeats_publish_the_same_full_report() {
    let snapshot = open(war_with_methods());
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let owner = definition_at(&snapshot, &target);
    let environment = request_at(target.clone(), HISTORICAL_NAME).environment;
    let request = method_request(environment, owner, b"add", b"(II)I");

    let first_store = cache(8, 1 << 20);
    let second_store = cache(8, 1 << 20);
    let (first_cold, first_cold_budget) =
        recover_with(&snapshot, &request, Some(&first_store), limits());
    let (second_cold, second_cold_budget) =
        recover_with(&snapshot, &request, Some(&second_store), limits());
    let (first_warm, _) = recover_with(&snapshot, &request, Some(&first_store), limits());
    let (second_warm, _) = recover_with(&snapshot, &request, Some(&second_store), limits());

    assert_eq!(
        method_full_report_fingerprint(&first_cold),
        method_full_report_fingerprint(&second_cold),
        "two cold runs of one configuration differ"
    );
    assert_eq!(
        method_full_report_fingerprint(&first_warm),
        method_full_report_fingerprint(&second_warm),
        "two warm runs of one configuration differ"
    );
    let (first_usage, second_usage) = (first_cold_budget.usage(), second_cold_budget.usage());
    for dimension in CountedBudgetDimension::ALL {
        assert_eq!(
            first_usage.counted_usage(dimension),
            second_usage.counted_usage(dimension),
            "the two cold runs disagree on {dimension:?}"
        );
    }
    println!(
        "identically warmed repeats: full-report fingerprints equal (only elapsed_millis \
         excluded); cold archive_entries {} read_bytes {} entry_bytes {} class_headers {}",
        first_usage.counted_usage(CountedBudgetDimension::ArchiveEntries),
        first_usage.counted_usage(CountedBudgetDimension::ReadBytes),
        first_usage.counted_usage(CountedBudgetDimension::EntryBytes),
        first_usage.counted_usage(CountedBudgetDimension::ClassHeaders)
    );
}

/// Two different methods of one snapshot reuse one retained container.
#[test]
fn two_methods_of_one_snapshot_reuse_one_container() {
    let snapshot = open(war_with_methods());
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let owner = definition_at(&snapshot, &target);
    let environment = request_at(target.clone(), HISTORICAL_NAME).environment;
    let store = cache(8, 1 << 20);

    let first = method_request(environment.clone(), owner.clone(), b"add", b"(II)I");
    let second = method_request(environment, owner, b"finallyPath", b"(I)I");
    let (add, _) = recover_with(&snapshot, &first, Some(&store), limits());
    let after_first = counters(&store);
    let (second_run, _) = recover_with(&snapshot, &second, Some(&store), limits());
    let after_second = counters(&store);
    assert_eq!(method_status(&add), "complete");
    assert_eq!(method_status(&second_run), "complete");
    assert_eq!(
        (
            after_second.directory_parses,
            after_second.nested_materializations,
            after_second.nested_materialized_bytes
        ),
        (
            after_first.directory_parses,
            after_first.nested_materializations,
            after_first.nested_materialized_bytes
        ),
        "the second method rebuilt the container: before {after_first:?}, after {after_second:?}"
    );
    assert!(
        after_second.container_hits > after_first.container_hits,
        "the second method did not reuse the retained container: {after_second:?}"
    );
    println!(
        "two methods, one container: directories parsed {}, nested materialized {}, hits {}",
        after_second.directory_parses,
        after_second.nested_materializations,
        after_second.container_hits
    );
}

/// The capacity-starved row answers the same question and reports what it could not keep.
#[test]
fn the_capacity_starved_row_answers_the_same_question() {
    let snapshot = open(war_with_methods());
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let owner = definition_at(&snapshot, &target);
    let environment = request_at(target.clone(), HISTORICAL_NAME).environment;
    let request = method_request(environment, owner, b"add", b"(II)I");

    let starved = cache(0, 0);
    let (first, _) = recover_with(&snapshot, &request, Some(&starved), limits());
    let after_first = counters(&starved);
    let (second, _) = recover_with(&snapshot, &request, Some(&starved), limits());
    let after_second = counters(&starved);

    assert_eq!(method_status(&first), "complete");
    assert_eq!(
        method_semantic_fingerprint(&first),
        method_semantic_fingerprint(&second),
        "a store that kept nothing changed the answer"
    );
    assert_eq!(
        (after_first.containers, after_first.retained_bytes),
        (0, 0),
        "a zero-capacity store retained something: {after_first:?}"
    );
    assert!(
        after_second.refused_capacity > 0,
        "the starved row reports no refusal: {after_second:?}"
    );
    assert!(
        after_second.directory_parses > after_first.directory_parses,
        "the starved row stopped parsing, so it must have retained something"
    );
    println!(
        "capacity-starved row: complete, same semantic fingerprint, retentions {} refusals {} \
         directories parsed {}",
        after_second.container_stored, after_second.refused_capacity, after_second.directory_parses
    );
}

/// The timing reading of the rows, for a caller that asks for it explicitly.
///
/// **Not a gate and not a speedup claim.** One machine, one build, serial repeats of one request
/// per row; the numbers are printed so a later comparison has the same-metric reading beside it.
/// The deterministic counters are asserted by the tests above; a wall-clock threshold on a shared
/// machine would be a flake, so none is asserted here.
#[test]
#[ignore]
fn container_lookup_timings() {
    let snapshot = open(war_with_methods());
    let target = origin_named(&snapshot, b"WEB-INF/lib/Target.jar");
    let owner = definition_at(&snapshot, &target);
    let environment = request_at(target.clone(), HISTORICAL_NAME).environment;
    let request = method_request(environment, owner, b"add", b"(II)I");
    const REPEATS: usize = 50;

    let warm_store = cache(8, 1 << 20);
    let _ = recover_with(&snapshot, &request, Some(&warm_store), limits());

    let mut timings: Vec<(&str, Vec<u128>)> = Vec::new();
    for label in ["off", "cold", "warm"] {
        let mut samples = Vec::new();
        let mut cold_store = cache(8, 1 << 20);
        for _ in 0..REPEATS {
            // `off` attaches no handle, `cold` a fresh empty store per repeat, and `warm` the one
            // store that was warmed before the row started. Each row is the same request, the same
            // snapshot, the same limits and the same machine.
            let attached = match label {
                "off" => None,
                "cold" => Some(&cold_store),
                _ => Some(&warm_store),
            };
            let started = Instant::now();
            let (run, _) = recover_with(&snapshot, &request, attached, limits());
            let elapsed = started.elapsed().as_micros();
            assert_eq!(method_status(&run), "complete");
            samples.push(elapsed);
            cold_store = cache(8, 1 << 20);
        }
        samples.sort_unstable();
        timings.push((label, samples));
    }
    println!(
        "container lookup timings: one machine (macOS/arm64), one process, {} serial repeats per \
         row, the committed ECJ v52 class inside a nested library, limits input 64 MiB / entries \
         100000 / read 64 MiB / nested_depth 8, cache 8 entries / 8 MiB; no threshold asserted and \
         no speedup claimed",
        REPEATS
    );
    for (label, samples) in &timings {
        let median = samples[samples.len() / 2];
        let p90 = samples[samples.len() * 9 / 10];
        println!(
            "{label:>5}: min {} median {} p90 {} max {} (µs)",
            samples[0],
            median,
            p90,
            samples[samples.len() - 1]
        );
    }
}
