//! The selected-definition read reuse (change `reuse-selected-class-read`), on the terms the change
//! states — and on the terms it refuses to state.
//!
//! # What is reused, and the two paths every case compares
//!
//! One operation that has selected a physical definition performs one read of it: the entry at the
//! definition's own coordinates, located through the container's verified directory, read through the
//! verifying reader (CRC and uncompressed size re-established from the bytes) and digested. This
//! change keeps that read — the bytes, and the identity the read itself established — in the request's
//! facts store, under the definition's own identity, and answers the next request for **that same
//! definition** from it. Nothing else is reused: no parse, no `PreparedClass`, no method-level product,
//! and no read the *search* performs for the candidates it confirms.
//!
//! Every case therefore compares two paths of the same request:
//!
//! * **the direct path** — a budget with no store attached, which is what every entry point always
//!   did: this request locates, reads, verifies and digests the entry itself;
//! * **the reuse path** — a budget carrying one store, warmed by an earlier request of the same
//!   snapshot, which answers the read with the bytes and the identity that earlier read established.
//!
//! The two paths must publish the same result and must charge the same dimensions **except** the read
//! itself (`archive_entries`, `read_bytes`, `entry_bytes`), which is exactly what a hit stops paying.
//! This target asserts nothing about wall clocks: the change's admitted upper bound is 0.5% of a
//! workload window, and its claim is about work removed.
//!
//! # What the store's own readings are
//!
//! The store reports its definition-read layer (`definition_reads`, `definition_read_bytes`, the
//! consultation/hit/miss/stored counters and the two capacity refusals) *outside* every result, like
//! the two layers beside it: a hit changes what a request paid, never what it published. This target
//! reads those counters to say which path a request took, and compares the results themselves with the
//! charges and the clock stripped, exactly as `performance-gates` compares a cold and a warm run.
//!
//! # The one environment variable, and why it is not a product switch
//!
//! [`one_process_reads_one_tier_of_the_capacity_ladder`] reads `JARDE_REUSE_TIER` to pick which
//! capacity **this test process** runs one fixed sequence under, so the out-of-process RSS readings of
//! `evidence/implementation.md` can be taken per tier with `/usr/bin/time -l`. Nothing in the library
//! reads it: there is no reuse switch, no arm and no parallel path in the product — the engine has one
//! behaviour, and this variable only tells one test which store capacity to hand it. Every other case
//! here selects its store by argument, exactly as a caller does through `Budget::with_facts_cache`.
//!
//! ```text
//! cargo test --test p5_definition_read_reuse --all-features --locked -- --nocapture
//! ```

#![cfg(feature = "test-support")]

mod bulk_support;

use bulk_support::{DEFLATE, SCOPE, SHAPE, STORE, flat_fixture, zip};
use jarde::d0_counts::{self, Counts};
use jarde::*;
use serde_json::Value;
use std::sync::{Mutex, MutexGuard};

/// One test's own gate: the D0 counting port is one process-wide set of counters, so the cases that
/// read it run one at a time.
static GATE: Mutex<()> = Mutex::new(());

fn gate() -> MutexGuard<'static, ()> {
    GATE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn limits() -> Limits {
    bulk_support::limits()
}

/// A budget of this target's limits, with `store` attached when one is named.
///
/// No case here enables anything else: a store is the caller's own handle on its budget
/// (`Budget::with_facts_cache`), and a budget that names none reads exactly as every entry point did
/// before this change.
fn request_budget(store: Option<&FactsCache>) -> Budget {
    let budget = Budget::new(limits());
    match store {
        Some(store) => budget.with_facts_cache(store.clone()),
        None => budget,
    }
}

/// One store of this build's declaration and the given capacity.
fn new_store(capacity: FactsCapacity) -> FactsCache {
    FactsCache::current(capacity)
}

/// The capacity ladder, from "keeps nothing" to "keeps everything this target reads".
///
/// The five tiers are the ones the container layer's own ladder uses (`p5_container_lookup`), so a
/// reading of this layer can be read beside a reading of that one: `none` retains nothing and is a
/// counter, `e1` leaves room for one answer, `tiny` has entries to spare but no bytes, `medium` holds
/// a couple of answers and `roomy` holds the whole sequence.
const LADDER: [(&str, FactsCapacity); 5] = [
    ("none", FactsCapacity::none()),
    ("e1", FactsCapacity::new(1, 1 << 20)),
    ("tiny", FactsCapacity::new(8, 1 << 10)),
    ("medium", FactsCapacity::new(8, 1 << 20)),
    ("roomy", FactsCapacity::new(1 << 14, 1 << 27)),
];

fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes), &mut request_budget(None))
        .expect("the fixture snapshot opens")
}

fn tree_scope() -> PhysicalScope {
    PhysicalScope::ArtifactTree {
        root_container: ContainerId("root".to_owned()),
    }
}

/// The environment of one jar snapshot: its own root container at the root prefix.
fn plain_jar_environment(snapshot: &ArtifactSnapshot) -> EnvironmentRequest {
    EnvironmentRequest {
        snapshot: snapshot.id().clone(),
        scope: tree_scope(),
        policy: EnvironmentPolicy::PlainJar,
        profile: RuntimeProfile {
            java_release: 8,
            multi_release: MultiReleasePolicy::Disabled,
            layout: LayoutMode::Generic,
        },
        loader: LoaderId("app".to_owned()),
    }
}

/// One class declaration of a scope, as the listing states it.
fn declaration_at(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    store: Option<&FactsCache>,
    index: usize,
) -> ClassDeclarationItem {
    let mut budget = request_budget(store);
    engine
        .list_class_declarations(snapshot, &tree_scope(), &mut budget)
        .expect("the fixture's declarations are listable")
        .items
        .into_iter()
        .nth(index)
        .unwrap_or_else(|| panic!("the fixture declares more than {index} class(es)"))
}

/// Every definition a scope declares, in scope order, through the public listing entry.
///
/// The listing reads the **candidates** it confirms; it is the name-driven path of the change's §2.1
/// and must not consult the definition-read layer at all.
fn definitions(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    store: Option<&FactsCache>,
) -> Vec<PhysicalDefinitionId> {
    let mut budget = request_budget(store);
    engine
        .list_class_declarations(snapshot, &tree_scope(), &mut budget)
        .expect("the fixture's declarations are listable")
        .items
        .into_iter()
        .map(|item| item.definition)
        .collect()
}

/// One definition of one fixture, read through that listing.
fn definition_at(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    store: Option<&FactsCache>,
    index: usize,
) -> PhysicalDefinitionId {
    declaration_at(engine, snapshot, store, index).definition
}

/// The definition one fixture entry declares, built by hand: its coordinates, the identity of the
/// bytes it holds and the variant its raw name derives.
///
/// A damaged entry cannot be listed — the listing confirms a candidate by reading its header, and a
/// header that does not decode confirms nothing — so the cases about stopped reads state the identity
/// they address themselves. [`the_offered_bytes_are_the_entrys_own_bytes`] is what keeps such a value
/// honest: for the undamaged geometry it is the very definition the listing publishes.
fn definition_of_entry(
    snapshot: &ArtifactSnapshot,
    ordinal: u64,
    raw_name: &[u8],
    class: &[u8],
) -> PhysicalDefinitionId {
    PhysicalDefinitionId {
        location: PhysicalClassLocation::ArchiveEntry {
            entry: PhysicalEntryId {
                origin: ContainerOrigin {
                    snapshot: snapshot.id().clone(),
                    root_container: ContainerId("root".to_owned()),
                    steps: Vec::new(),
                },
                ordinal,
                raw_name: ArchiveNameBytes(raw_name.to_vec()),
            },
        },
        class_bytes: class_bytes_of(class),
        variant: physical_variant_for_path(raw_name),
    }
}

/// One class view by identity, as `(report, usage, what the D0 port counted)`.
///
/// The view asks for no body: the body-decoding half of the workload is the method-driven path below,
/// and keeping this request to the class levels makes the charges it must show (the header attempt,
/// the class read, the member walk) the ones the case is about.
fn view_by_identity(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    store: Option<&FactsCache>,
) -> (ClassViewReport, UsageSnapshot, Counts) {
    let mut budget = request_budget(store);
    let before = d0_counts::snapshot();
    let request = ClassViewRequest {
        class: ClassRef::Definition {
            definition: definition.clone(),
        },
        bodies: Vec::new(),
    };
    let view = match engine
        .class_view(snapshot, &tree_scope(), &request, &mut budget)
        .expect("the identity binds one definition")
    {
        OperationOutcome::Performed(view) => view,
        other => panic!("a class identity is never ambiguous or incomplete: {other:?}"),
    };
    let counts = before.since(d0_counts::snapshot());
    (view, budget.usage(), counts)
}

/// One class view by **name**: the search path, which reads its own candidates.
fn view_by_name(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    class: &str,
    store: Option<&FactsCache>,
) -> (ClassViewReport, UsageSnapshot) {
    let mut budget = request_budget(store);
    let request = ClassViewRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::dotted(class),
        },
        bodies: Vec::new(),
    };
    let view = match engine
        .class_view(snapshot, &tree_scope(), &request, &mut budget)
        .expect("the fixture's class is listed")
    {
        OperationOutcome::Performed(view) => view,
        other => panic!("one declared class is never ambiguous: {other:?}"),
    };
    (view, budget.usage())
}

/// One member listing under this target's own limits, with the D0 port read around it.
fn members(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    store: Option<&FactsCache>,
) -> (MemberListing, UsageSnapshot, Counts) {
    members_within(engine, snapshot, definition, store, limits())
}

/// The same listing under explicit limits: a case that is about where a request stops states the
/// allowance it stops under.
fn members_within(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    store: Option<&FactsCache>,
    limits: Limits,
) -> (MemberListing, UsageSnapshot, Counts) {
    let mut budget = match store {
        Some(store) => Budget::new(limits).with_facts_cache(store.clone()),
        None => Budget::new(limits),
    };
    let before = d0_counts::snapshot();
    let listing = engine
        .list_members(snapshot, definition, &mut budget)
        .expect("the fixture's member table is read");
    let counts = before.since(d0_counts::snapshot());
    (listing, budget.usage(), counts)
}

/// The first member of one class that declares a body, as the member listing states it.
fn first_member(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    store: Option<&FactsCache>,
) -> PhysicalMethodId {
    let (listing, _, _) = members(engine, snapshot, definition, store);
    listing
        .methods()
        .find(|method| matches!(method.body, MemberBodyEvidence::CodeAttribute { .. }))
        .map(|method| method.identity.clone())
        .expect("the fixture declares a member with a body")
}

/// One method recovery **by identity**: the method-driven read of §2.1, with the evidence selection
/// the admission experiment's W1 shape used.
fn recover_one(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    method: &PhysicalMethodId,
    store: Option<&FactsCache>,
) -> (MethodRecoveryReport, UsageSnapshot, Counts) {
    let mut budget = request_budget(store);
    let before = d0_counts::snapshot();
    let request = MethodOperationRequest {
        method: MethodRef::Method {
            method: method.clone(),
        },
        environment: plain_jar_environment(snapshot),
    };
    let recovered = match engine
        .recover_target_with_evidence(
            std::slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::essential(),
            &mut budget,
        )
        .expect("a legal request is answered, not raised")
    {
        OperationOutcome::Performed(report) => report,
        other => panic!("a member identity is never ambiguous or incomplete: {other:?}"),
    };
    let counts = before.since(d0_counts::snapshot());
    (recovered, budget.usage(), counts)
}

/// One document with its own **readings** removed: every `usage` object and every `elapsed_millis`,
/// which are properties of the run and not of what it published, and the *name* of the snapshot every
/// physical identity carries (two snapshots of one archive are two identities by construction, and
/// this helper exists to compare what two paths say about the same class).
fn normalized(mut value: Value) -> String {
    fn walk(value: &mut Value) -> usize {
        let mut removed = 0;
        match value {
            Value::Object(map) => {
                removed += usize::from(map.remove("usage").is_some());
                removed += usize::from(map.remove("elapsed_millis").is_some());
                removed += usize::from(map.remove("limits").is_some());
                for (key, child) in map.iter_mut() {
                    if key == "snapshot" {
                        *child = Value::String("(one snapshot)".to_owned());
                    } else {
                        removed += walk(child);
                    }
                }
            }
            Value::Array(entries) => {
                for child in entries.iter_mut() {
                    removed += walk(child);
                }
            }
            _ => {}
        }
        removed
    }
    let removed = walk(&mut value);
    assert!(
        removed > 0,
        "the document carries nothing to strip, so this comparison would be vacuous: {value}"
    );
    serde_json::to_string(&value).expect("a JSON value renders")
}

/// One document as JSON, charges included.
fn document<T: serde::Serialize>(value: &T) -> Value {
    serde_json::to_value(value).expect("the report renders as JSON")
}

fn digest_of(bytes: &[u8]) -> Digest {
    Digest(blake3::hash(bytes).to_hex().to_string())
}

fn class_bytes_of(bytes: &[u8]) -> ClassBytesId {
    ClassBytesId {
        digest: digest_of(bytes),
        length: u64::try_from(bytes.len()).expect("the fixture fits u64"),
    }
}

/// The error as the vocabulary the rest of the engine uses: a code, a dimension, `cancelled`.
fn code_of(error: &Error) -> String {
    match error {
        Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code.clone(),
        Error::BudgetExceeded { dimension, .. } => format!("{dimension:?}"),
        Error::Cancelled { .. } => "cancelled".to_owned(),
        Error::Io { operation, .. } => operation.clone(),
    }
}

/// The counted dimensions one request charged, as one line, so a failure is read beside its numbers.
fn charges(usage: &UsageSnapshot, dimensions: &[CountedBudgetDimension]) -> String {
    dimensions
        .iter()
        .map(|dimension| format!("{dimension:?}={}", usage.counted_usage(*dimension)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The three dimensions a definition read is billed in, and the one thing a hit must not charge.
const READ_DIMENSIONS: &[CountedBudgetDimension] = &[
    CountedBudgetDimension::ArchiveEntries,
    CountedBudgetDimension::ReadBytes,
    CountedBudgetDimension::EntryBytes,
];

/// Every counted dimension **except** the three a read is billed in: the ones a hit must charge
/// exactly as the direct path does.
fn unread_dimensions() -> Vec<CountedBudgetDimension> {
    CountedBudgetDimension::ALL
        .into_iter()
        .filter(|dimension| !READ_DIMENSIONS.contains(dimension))
        .collect()
}

/// Whether one request charged nothing for the **entry read** itself.
///
/// The two dimensions are the ones only the read charges: `read_bytes` is the compressed bytes the
/// read takes and `entry_bytes` the uncompressed bytes it produces. `archive_entries` is deliberately
/// not in this predicate, because the container layer's directory parse charges it too (one record
/// per entry it walks) and that layer is a component of its own with its own store state — a case that
/// wants to say "this request's read was answered from retention" reads the same question the answer
/// is about, and compares `archive_entries` beside it.
fn charged_no_entry_read(usage: &UsageSnapshot) -> bool {
    usage.counted_usage(CountedBudgetDimension::ReadBytes) == 0
        && usage.counted_usage(CountedBudgetDimension::EntryBytes) == 0
}

/// One entry's own data range, located through the archive's central directory.
fn entry_data_range(bytes: &[u8], ordinal: usize) -> (usize, usize) {
    let archive = rawzip::ZipArchive::from_slice(bytes).expect("the fixture is a ZIP");
    let mut iterator = archive.entries();
    let mut current = 0_usize;
    loop {
        let header = iterator
            .next_entry()
            .expect("the fixture's central directory reads")
            .expect("the fixture holds the entry asked for");
        if current == ordinal {
            let local = archive
                .get_entry(header.wayfinder())
                .expect("the fixture's local header reads");
            let (start, end) = local.compressed_data_range();
            return (
                usize::try_from(start).expect("the offset fits usize"),
                usize::try_from(end).expect("the offset fits usize"),
            );
        }
        current += 1;
    }
}

/// A copy of `bytes` whose entry `ordinal` has one byte of its own data overwritten.
///
/// The archive's directory, local headers and declared sizes are untouched, so the damage is exactly
/// what a read meets while it produces the entry's bytes: a deflate stream that cannot be decoded, or
/// a stored payload whose CRC no longer matches the record.
fn damaged_at(bytes: &[u8], ordinal: usize) -> Vec<u8> {
    let (start, _end) = entry_data_range(bytes, ordinal);
    let mut damaged = bytes.to_vec();
    damaged[start] ^= 0xff;
    damaged
}

// ---------------------------------------------------------------------------------------------
// R01 — the identity of a retained read is every dimension of the definition
// ---------------------------------------------------------------------------------------------

/// One retained read answers for its definition and for nothing "near" it: every other dimension of
/// the identity is mutated, one at a time, and every one of them misses.
///
/// The case offers the read through the snapshot's own entrance (`remember_definition_read`, the one
/// `jarde-jvm` and the facade both reach) and looks it up through the matching read entrance
/// (`retained_definition_read`), which is where the identity rule lives. The dimensions are the ones
/// the change's §1.1 names: the snapshot, the complete physical location (container chain, ordinal and
/// raw name), the content digest, the declared length, and the variant the raw name derives.
#[test]
fn a_retained_read_answers_for_its_own_definition_and_for_no_other() {
    let _gate = gate();
    let engine = Engine::new();
    // Two entries holding the *same* class bytes at different coordinates, and a versioned copy of
    // the same bytes: three definitions of one content, so a rule that answered by "the content looks
    // the same" would answer all three.
    let bytes = zip(&[
        (b"Shape.class", SHAPE, STORE),
        (b"dir/Shape.class", SHAPE, STORE),
        (b"META-INF/versions/9/Shape.class", SHAPE, DEFLATE),
        (b"notes.txt", b"nothing to see", STORE),
    ]);
    let snapshot = open(bytes);
    let cache = new_store(FactsCapacity::new(8, 1 << 20));
    let listed = definitions(&engine, &snapshot, Some(&cache));
    assert!(
        listed.len() >= 3,
        "the fixture declares three definitions of one content: {listed:?}"
    );
    let definition = listed[0].clone();
    let same_content = listed
        .iter()
        .find(|other| **other != definition && other.class_bytes == definition.class_bytes)
        .expect("the fixture holds the same bytes at another coordinate")
        .clone();
    assert_eq!(
        definition.class_bytes, same_content.class_bytes,
        "the two definitions are one content at two coordinates, which is what makes the mutations \
         below discriminating"
    );

    // The read is offered under the definition it really is: the identity it established is the
    // definition's own declaration.
    let class_bytes = class_bytes_of(SHAPE);
    snapshot.remember_definition_read(
        &definition,
        SHAPE,
        &class_bytes.digest,
        &request_budget(Some(&cache)),
    );

    // The definition itself: answered, with the bytes that read produced and the identity it
    // established — never a copy of the caller's claim.
    let mut lookup = request_budget(Some(&cache));
    let (returned, identity) = snapshot
        .retained_definition_read(&definition, &mut lookup)
        .expect("the definition names this snapshot")
        .expect("the read was offered under exactly this definition");
    assert_eq!(*returned, SHAPE, "the bytes are not the read's own bytes");
    assert_eq!(identity, class_bytes, "the identity is not the read's own");

    // One dimension at a time. Each mutation differs from the offered definition in exactly one
    // dimension, so a miss says which dimension was compared.
    let location = definition
        .location
        .entry()
        .expect("the fixture definition is an archive entry")
        .clone();
    let mut other_ordinal = location.clone();
    other_ordinal.ordinal += 1;
    let mut other_name = location.clone();
    other_name.raw_name = ArchiveNameBytes(b"elsewhere/Shape.class".to_vec());
    let mut other_container = location.clone();
    other_container.origin.root_container = ContainerId("elsewhere".to_owned());
    let mut other_digest = definition.clone();
    other_digest.class_bytes.digest = Digest("0".repeat(64));
    let mut other_length = definition.clone();
    other_length.class_bytes.length += 1;
    let mut other_variant = definition.clone();
    other_variant.variant = PhysicalVariant::MultiRelease { version: 9 };
    let mutations: [(&str, PhysicalDefinitionId); 6] = [
        (
            "the same raw name at another ordinal",
            PhysicalDefinitionId {
                location: PhysicalClassLocation::ArchiveEntry {
                    entry: other_ordinal,
                },
                class_bytes: definition.class_bytes.clone(),
                variant: definition.variant.clone(),
            },
        ),
        (
            "the same bytes under another raw name",
            PhysicalDefinitionId {
                location: PhysicalClassLocation::ArchiveEntry { entry: other_name },
                class_bytes: definition.class_bytes.clone(),
                variant: definition.variant.clone(),
            },
        ),
        (
            "the same coordinates in another container",
            PhysicalDefinitionId {
                location: PhysicalClassLocation::ArchiveEntry {
                    entry: other_container,
                },
                class_bytes: definition.class_bytes.clone(),
                variant: definition.variant.clone(),
            },
        ),
        ("another digest", other_digest),
        ("another declared length", other_length),
        ("another variant", other_variant),
    ];
    for (name, mutated) in &mutations {
        assert_ne!(
            mutated, &definition,
            "the mutation {name:?} is not a mutation, so it proves nothing"
        );
        let mut lookup = request_budget(Some(&cache));
        let answer = snapshot
            .retained_definition_read(mutated, &mut lookup)
            .expect("a definition of this snapshot is looked up, not refused");
        assert!(
            answer.is_none(),
            "{name:?} was answered from the retained read: a read must not answer for a definition \
             that differs in any one dimension"
        );
    }
    // The same bytes at their **other real coordinate**: a definition the listing really declared, so
    // the miss is about identity rather than about a fabricated coordinate.
    let mut lookup = request_budget(Some(&cache));
    assert!(
        snapshot
            .retained_definition_read(&same_content, &mut lookup)
            .expect("the other definition names this snapshot")
            .is_none(),
        "a definition of the same content at another coordinate was answered from the retained read"
    );

    let report = cache.report();
    assert_eq!(report.definition_read_stored, 1, "{report:?}");
    assert_eq!(report.definition_read_hits, 1, "{report:?}");
    assert_eq!(
        report.definition_read_misses,
        1 + u64::try_from(mutations.len()).expect("the mutation count fits u64"),
        "{report:?}"
    );
    println!(
        "one read, one hit, {} identity miss(es) ({}), and the read itself weighed {} byte(s)",
        report.definition_read_misses,
        mutations
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(", "),
        report.definition_read_bytes
    );
}

/// Two snapshots that hold one class's bytes at the same coordinates share its digest and are still
/// two identities: the second snapshot's read is its own, and the first one's read answers only the
/// first.
///
/// This is the discriminating case the admission experiment recorded (`optimize-demand-workloads` §3):
/// a key that bound the coordinates and the digest but not the snapshot would answer the second
/// archive from the first one's read, and the two archives differ only in an unrelated entry — the
/// class bytes, their coordinate and their digest are identical.
#[test]
fn two_snapshots_of_one_class_do_not_answer_for_each_other() {
    let _gate = gate();
    let engine = Engine::new();
    let first = zip(&[
        (b"Shape.class", SHAPE, STORE),
        (b"notes.txt", b"one", STORE),
    ]);
    let second = zip(&[
        (b"Shape.class", SHAPE, STORE),
        (b"notes.txt", b"two", STORE),
    ]);
    let cache = new_store(FactsCapacity::new(8, 1 << 20));
    let snapshot_one = open(first);
    let snapshot_two = open(second);
    assert_ne!(
        snapshot_one.id(),
        snapshot_two.id(),
        "the two archives are two snapshots, or this case says nothing"
    );

    let definition_one = definition_at(&engine, &snapshot_one, Some(&cache), 0);
    let definition_two = definition_at(&engine, &snapshot_two, Some(&cache), 0);
    assert_eq!(
        definition_one.class_bytes, definition_two.class_bytes,
        "the class is the same bytes under the same name, so a snapshot-blind key could answer it"
    );
    // The two definitions differ in exactly one dimension — the snapshot they were read from — and in
    // nothing else, which is what makes this case the snapshot dimension's own.
    let mut blind_one = definition_one.clone();
    let mut blind_two = definition_two.clone();
    blind_one.location = PhysicalClassLocation::StandaloneRoot {
        snapshot: SnapshotId("(one snapshot)".to_owned()),
    };
    blind_two.location = PhysicalClassLocation::StandaloneRoot {
        snapshot: SnapshotId("(one snapshot)".to_owned()),
    };
    assert_eq!(
        blind_one, blind_two,
        "the two definitions are one class at one coordinate but for their snapshot"
    );

    let (view_one, _, _) = view_by_identity(&engine, &snapshot_one, &definition_one, Some(&cache));
    let after_first = cache.report();
    let (view_two, _, _) = view_by_identity(&engine, &snapshot_two, &definition_two, Some(&cache));
    let after_second = cache.report();
    let (view_one_again, _, _) =
        view_by_identity(&engine, &snapshot_one, &definition_one, Some(&cache));
    let after_again = cache.report();

    // One answer three times, charges and snapshot names set aside: the class, its members, its
    // coverage and its diagnostics do not depend on which read answered them.
    assert_eq!(
        normalized(document(&view_one)),
        normalized(document(&view_two)),
        "the two snapshots' views of one class differ"
    );
    assert_eq!(
        normalized(document(&view_one)),
        normalized(document(&view_one_again)),
        "the same definition answered twice differently"
    );
    // And the reads are two: the second snapshot's view may not be answered from the first's read.
    assert_eq!(
        after_second.definition_read_hits, after_first.definition_read_hits,
        "a definition read of another snapshot was answered from the first snapshot's retention: \
         {after_first:?} -> {after_second:?}"
    );
    assert_eq!(
        after_second.definition_read_stored,
        after_first.definition_read_stored + 1,
        "the second snapshot's own read was not retained as its own: {after_second:?}"
    );
    assert_eq!(
        after_again.definition_read_hits,
        after_second.definition_read_hits + 1,
        "the first snapshot's definition was not answered from its own read: {after_again:?}"
    );
    println!(
        "one class, two snapshots, one store: hits {} -> {} -> {}, reads kept {}, consultations {}",
        after_first.definition_read_hits,
        after_second.definition_read_hits,
        after_again.definition_read_hits,
        after_again.definition_reads,
        after_again.definition_read_consultations
    );
}

// ---------------------------------------------------------------------------------------------
// R02 — only a complete read is ever published
// ---------------------------------------------------------------------------------------------

/// Every way a read can stop leaves the retention empty, and a read that reaches the end fills it.
///
/// The four termination points the change names are exercised on one store each: a cancelled request,
/// a request whose budget stops the read, an entry whose bytes cannot be produced (a stream that stops
/// before the entry's declared end) and an entry whose bytes are produced but whose verification
/// refuses. The control beside them is the same fixture undamaged, so the zeros are the stopping and
/// not a store that never keeps anything.
#[test]
fn a_read_that_stopped_is_never_retained_and_a_read_that_finished_is() {
    let _gate = gate();
    let engine = Engine::new();
    let healthy = zip(&[(b"Shape.class", SHAPE, STORE)]);
    let entry_bytes = u64::try_from(SHAPE.len()).expect("the fixture fits u64");

    // (1) Cancelled before the read: the request ends cancelled and nothing is kept.
    let cancelled = new_store(FactsCapacity::new(8, 1 << 20));
    let snapshot = open(healthy.clone());
    let definition = definition_at(&engine, &snapshot, Some(&cancelled), 0);
    let token = CancellationToken::new();
    token.cancel();
    let mut budget =
        Budget::with_cancellation_token(limits(), token).with_facts_cache(cancelled.clone());
    let error = engine
        .list_members(&snapshot, &definition, &mut budget)
        .expect_err("a cancelled request reads nothing");
    assert_eq!(code_of(&error), "cancelled", "{error}");
    assert_eq!(
        cancelled.report().definition_reads,
        0,
        "a cancelled request retained a read: {:?}",
        cancelled.report()
    );

    // (2) Stopped by the budget: the entry read is what the limit refuses.
    let starved = new_store(FactsCapacity::new(8, 1 << 20));
    let mut tight = limits();
    tight.entry_bytes = entry_bytes - 1;
    let mut budget = Budget::new(tight).with_facts_cache(starved.clone());
    let error = engine
        .list_members(&snapshot, &definition, &mut budget)
        .expect_err("a budget that cannot pay for the entry stops the read");
    assert_eq!(code_of(&error), "EntryBytes", "{error}");
    assert_eq!(
        starved.report().definition_reads,
        0,
        "a read the budget stopped was retained: {:?}",
        starved.report()
    );

    // (3) A read that stops while producing the bytes: the entry's deflate stream is damaged, so the
    //     bytes never reach the entry's declared end.
    let partial = zip(&[(b"Shape.class", SHAPE, DEFLATE)]);
    let partial_snapshot = open(damaged_at(&partial, 0));
    let partial_store = new_store(FactsCapacity::new(8, 1 << 20));
    let partial_definition = definition_of_entry(&partial_snapshot, 0, b"Shape.class", SHAPE);
    let mut budget = request_budget(Some(&partial_store));
    let error = engine
        .list_members(&partial_snapshot, &partial_definition, &mut budget)
        .expect_err("a damaged stream stops the read");
    assert_eq!(
        code_of(&error),
        "entry_integrity",
        "the read stopped for a reason this case does not name: {error}"
    );
    assert_eq!(
        partial_store.report().definition_reads,
        0,
        "a partially produced read was retained: {:?}",
        partial_store.report()
    );

    // (4) A complete production whose verification refuses: the stored payload is a byte off, so the
    //     CRC the record states does not describe the bytes that came out.
    let damaged = open(damaged_at(&healthy, 0));
    let verification = new_store(FactsCapacity::new(8, 1 << 20));
    let damaged_definition = definition_of_entry(&damaged, 0, b"Shape.class", SHAPE);
    let mut budget = request_budget(Some(&verification));
    let error = engine
        .list_members(&damaged, &damaged_definition, &mut budget)
        .expect_err("an entry whose CRC does not match is not a read");
    assert_eq!(
        code_of(&error),
        "entry_integrity",
        "the refusal does not name the verification that failed: {error}"
    );
    assert_eq!(
        verification.report().definition_reads,
        0,
        "a read whose verification did not complete was retained: {:?}",
        verification.report()
    );

    // The control: the same read, complete, over the same fixture geometry, fills exactly one slot.
    let control = new_store(FactsCapacity::new(8, 1 << 20));
    let control_snapshot = open(healthy);
    let control_definition = definition_at(&engine, &control_snapshot, Some(&control), 0);
    let (_, _, _) = members(
        &engine,
        &control_snapshot,
        &control_definition,
        Some(&control),
    );
    let report = control.report();
    assert_eq!(
        (report.definition_reads, report.definition_read_stored),
        (1, 1),
        "a read that ran to the end and verified was not retained, so this case proves nothing: \
         {report:?}"
    );
    assert_eq!(
        report.definition_read_bytes, entry_bytes,
        "the retained read does not weigh the class it holds: {report:?}"
    );
    println!(
        "stopped reads retained: cancelled 0, limit 0, partial 0, unverified 0; complete read: {} \
         definition(s), {} byte(s)",
        report.definition_reads, report.definition_read_bytes
    );
}

// ---------------------------------------------------------------------------------------------
// R03 — the two paths publish the same thing
// ---------------------------------------------------------------------------------------------

/// One step of the sequence below: what it published, what it charged, what it counted.
struct Step {
    name: &'static str,
    document: Value,
    usage: UsageSnapshot,
    counts: Counts,
    /// Whether this step is one the D0 port counts class materializations for (the two entrances of
    /// §2.1); a member listing and a name-driven view are not.
    counts_materializations: bool,
}

/// The same sequence of requests over the same snapshot, run once with no store and once with one:
/// every result is identical and only the reads move.
///
/// The sequence walks the two entrances of §2.1 (the facade's identity-bound read through
/// `class_view`, the method-driven read through `recover_method`), the member listing, and the name
/// search that must not reach this layer at all. What is compared per request is the document with its
/// charges and clock stripped, the charges of every dimension the read is *not* billed in, and the D0
/// port's counts: preparations, body decodes, recovery presentations and the owning records are the
/// same work in both arms, and only `class_materializations` — the class bytes a request really read —
/// falls.
#[test]
fn the_direct_path_and_the_reuse_path_publish_one_answer_and_one_workload() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(flat_fixture());
    // The store of this case is deliberately **smaller than the container product** and larger than a
    // definition read: the container layer is refused in both arms (so its directory parse is charged
    // in both), and the only thing the two arms can differ in is the definition read. That is the
    // single-factor shape the admission experiment required — one variable, one comparison — with the
    // switch replaced by a capacity, because the product has no switch.
    let cache = new_store(FactsCapacity::new(8, 1 << 10));
    let first = declaration_at(&engine, &snapshot, None, 0);
    let member = first_member(&engine, &snapshot, &first.definition, None);
    let name = first.declaration.this_class.escaped();

    let direct = sequence(&engine, &snapshot, &first.definition, &member, &name, None);
    let reuse = sequence(
        &engine,
        &snapshot,
        &first.definition,
        &member,
        &name,
        Some(&cache),
    );

    let mut answered = 0_u64;
    for (step, (direct, reuse)) in direct.iter().zip(reuse.iter()).enumerate() {
        assert_eq!(
            normalized(direct.document.clone()),
            normalized(reuse.document.clone()),
            "step {step} ({}) published a different document",
            direct.name
        );
        for dimension in unread_dimensions() {
            assert_eq!(
                reuse.usage.counted_usage(dimension),
                direct.usage.counted_usage(dimension),
                "step {step} ({}) charged {dimension:?} differently: direct {} reuse {}",
                direct.name,
                direct.usage.counted_usage(dimension),
                reuse.usage.counted_usage(dimension)
            );
        }
        if charged_no_entry_read(&reuse.usage) {
            // The store answered this step's read: the three dimensions of the read are gone, and the
            // D0 port must not claim the request materialized a class it did not read.
            assert!(
                !charged_no_entry_read(&direct.usage),
                "step {step} ({}) charged nothing in the direct arm either, so this comparison is \
                 empty",
                direct.name
            );
            answered += 1;
        } else {
            for dimension in READ_DIMENSIONS.iter().copied() {
                assert_eq!(
                    reuse.usage.counted_usage(dimension),
                    direct.usage.counted_usage(dimension),
                    "step {step} ({}) differed in {dimension:?} without being answered from \
                     retention",
                    direct.name
                );
            }
        }
        for dimension in READ_DIMENSIONS.iter().copied() {
            assert!(
                reuse.usage.counted_usage(dimension) <= direct.usage.counted_usage(dimension),
                "step {step} ({}) charged more {dimension:?} on the reuse path than on the direct \
                 one, so the case is not a comparison: direct {} reuse {}",
                direct.name,
                direct.usage.counted_usage(dimension),
                reuse.usage.counted_usage(dimension)
            );
        }
        if charged_no_entry_read(&reuse.usage) && !charged_no_entry_read(&direct.usage) {
            // The read is answered, and the read really is what disappeared: the direct arm paid the
            // entry (and the locating scan it needs) and the reuse arm paid neither.
            assert!(
                reuse
                    .usage
                    .counted_usage(CountedBudgetDimension::EntryBytes)
                    == 0
                    && direct
                        .usage
                        .counted_usage(CountedBudgetDimension::EntryBytes)
                        > 0,
                "step {step} ({}) did not compare an entry read with its absence",
                direct.name
            );
            assert!(
                reuse
                    .usage
                    .counted_usage(CountedBudgetDimension::ArchiveEntries)
                    < direct
                        .usage
                        .counted_usage(CountedBudgetDimension::ArchiveEntries),
                "step {step} ({}) was answered from retention and still located the entry: direct \
                 {} reuse {}",
                direct.name,
                direct
                    .usage
                    .counted_usage(CountedBudgetDimension::ArchiveEntries),
                reuse
                    .usage
                    .counted_usage(CountedBudgetDimension::ArchiveEntries)
            );
        }
        println!(
            "  direct all [{}]\n  reuse  all [{}]",
            charges(&direct.usage, &CountedBudgetDimension::ALL),
            charges(&reuse.usage, &CountedBudgetDimension::ALL)
        );
        println!(
            "step {step} ({}) direct [{}] reuse [{}] materializations {} vs {}",
            direct.name,
            charges(&direct.usage, READ_DIMENSIONS),
            charges(&reuse.usage, READ_DIMENSIONS),
            direct.counts.class_materializations,
            reuse.counts.class_materializations
        );
        println!(
            "  direct all [{}]\n  reuse  all [{}]",
            charges(&direct.usage, &CountedBudgetDimension::ALL),
            charges(&reuse.usage, &CountedBudgetDimension::ALL)
        );
    }

    // The work the change does **not** touch: every preparation, decoded body, recovery presentation
    // and published record is the same figure in both arms.
    for (step, (direct, reuse)) in direct.iter().zip(reuse.iter()).enumerate() {
        assert_eq!(
            Counts {
                class_materializations: 0,
                ..reuse.counts
            },
            Counts {
                class_materializations: 0,
                ..direct.counts
            },
            "step {step} ({}) is not the same work in the two arms",
            direct.name
        );
    }
    // And the reading the change is about: the repeated reads are what stops, one per answered step.
    let direct_reads: u64 = direct
        .iter()
        .map(|step| step.counts.class_materializations)
        .sum();
    let reuse_reads: u64 = reuse
        .iter()
        .map(|step| step.counts.class_materializations)
        .sum();
    assert!(
        reuse_reads < direct_reads,
        "no read was stopped, so this comparison is about nothing: {direct_reads} against \
         {reuse_reads}"
    );
    assert_eq!(
        answered,
        cache.report().definition_read_hits,
        "the steps the store answered are not the steps whose read disappeared: {:?}",
        cache.report()
    );
    // The port's own reading of the same steps: a step whose read is counted counts exactly one
    // materialization when it performed the read and none when the store answered it, and the steps
    // whose reads are not counted at all are left out of the claim.
    for (step, (direct, reuse)) in direct.iter().zip(reuse.iter()).enumerate() {
        if !direct.counts_materializations {
            continue;
        }
        let answered_here =
            charged_no_entry_read(&reuse.usage) && !charged_no_entry_read(&direct.usage);
        assert_eq!(
            direct.counts.class_materializations, 1,
            "step {step} ({}) did not count the read it performed: {:?}",
            direct.name, direct.counts
        );
        assert_eq!(
            reuse.counts.class_materializations,
            u64::from(!answered_here),
            "step {step} ({}) counted {} materialization(s) for a read that was{} answered from \
             retention: {:?}",
            direct.name,
            reuse.counts.class_materializations,
            if answered_here { "" } else { " not" },
            reuse.counts
        );
    }
    println!(
        "the same documents and the same work on both paths; class materializations {} -> {} over \
         {} answered step(s)",
        direct_reads, reuse_reads, answered
    );
}

/// Steps 0–1 view a definition twice, steps 2–3 recover one member twice, step 4 lists the members,
/// step 5 views the class **by name** — the search path, which reads its own candidates and never
/// reaches the definition-read layer.
fn sequence(
    engine: &Engine,
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    member: &PhysicalMethodId,
    name: &str,
    store: Option<&FactsCache>,
) -> Vec<Step> {
    let mut steps = Vec::new();
    for name in ["class_view #1", "class_view #2"] {
        let (view, usage, counts) = view_by_identity(engine, snapshot, definition, store);
        steps.push(Step {
            name,
            document: document(&view),
            usage,
            counts,
            counts_materializations: true,
        });
    }
    for name in ["recover_method #1", "recover_method #2"] {
        let (recovered, usage, counts) = recover_one(engine, snapshot, member, store);
        steps.push(Step {
            name,
            document: document(&recovered),
            usage,
            counts,
            counts_materializations: true,
        });
    }
    let (listing, usage, counts) = members(engine, snapshot, definition, store);
    steps.push(Step {
        name: "list_members",
        document: document(&listing),
        usage,
        counts,
        counts_materializations: false,
    });
    let (named, usage) = view_by_name(engine, snapshot, name, store);
    steps.push(Step {
        name: "class_view by name",
        document: document(&named),
        usage,
        counts: Counts::default(),
        counts_materializations: false,
    });
    steps
}

// ---------------------------------------------------------------------------------------------
// R04 — the fallbacks, one at a time
// ---------------------------------------------------------------------------------------------

/// Each way the reuse can fail to apply takes the ordinary read and changes nothing: no store, a
/// capacity that refuses, an identity that matches no retained read, a cancelled request, and an entry
/// another declaration wrote.
///
/// The "changes nothing" half is the comparison: everything the request published and charged is what
/// the same request publishes and charges with no store at all.
#[test]
fn every_fallback_reads_the_definition_itself() {
    let _gate = gate();
    let engine = Engine::new();
    let bytes = zip(&[
        (b"Shape.class", SHAPE, STORE),
        (b"notes.txt", b"note", STORE),
    ]);
    let snapshot = open(bytes);
    let definition = definition_at(&engine, &snapshot, None, 0);

    // The reference: the request with no store attached at all.
    let (reference_view, reference_usage, _) =
        view_by_identity(&engine, &snapshot, &definition, None);
    let reference = normalized(document(&reference_view));

    // (1) No store. The same call, stated as its own case: a budget with no store is what every entry
    //     point has always been handed, and nothing here may consult anything.
    let bare = request_budget(None);
    assert!(
        bare.facts_cache().is_none(),
        "a bare budget carries a store"
    );

    // (2) Capacity refused: a store with room for nothing answers nothing and refuses what is offered,
    //     and the request is completed by its own read.
    let none = new_store(FactsCapacity::none());
    let (view, usage, _) = view_by_identity(&engine, &snapshot, &definition, Some(&none));
    assert_eq!(
        normalized(document(&view)),
        reference,
        "a store with room for nothing changed what the request published"
    );
    assert_eq!(
        charges(&usage, READ_DIMENSIONS),
        charges(&reference_usage, READ_DIMENSIONS),
        "a refused retention removed a read the request still owes"
    );
    let refused = none.report();
    assert_eq!(refused.definition_reads, 0, "{refused:?}");
    assert!(
        refused.refused_capacity + refused.refused_capacity_bytes > 0,
        "the refused retention was not reported as a capacity refusal: {refused:?}"
    );
    assert_eq!(
        refused.definition_read_hits, 0,
        "a store with room for nothing answered something: {refused:?}"
    );

    // (3) An identity that matches no retained read: the store holds another snapshot's definition,
    //     and this request's own definition is read here.
    let other_snapshot = open(zip(&[
        (b"Shape.class", SHAPE, STORE),
        (b"more.txt", b"another archive", STORE),
    ]));
    let other_definition = definition_at(&engine, &other_snapshot, None, 0);
    let foreign = new_store(FactsCapacity::new(8, 1 << 20));
    let (_, other_usage, _) =
        view_by_identity(&engine, &other_snapshot, &other_definition, Some(&foreign));
    let before = foreign.report();
    let (view, usage, _) = view_by_identity(&engine, &snapshot, &definition, Some(&foreign));
    assert_eq!(
        normalized(document(&view)),
        reference,
        "a store holding another definition's read changed what this request published"
    );
    assert_eq!(
        charges(&usage, READ_DIMENSIONS),
        charges(&reference_usage, READ_DIMENSIONS),
        "the request was answered by another definition's read"
    );
    let after = foreign.report();
    assert_eq!(
        after.definition_read_hits, before.definition_read_hits,
        "another snapshot's definition was answered from the retained read: {after:?}"
    );
    assert!(
        other_usage.counted_usage(CountedBudgetDimension::ReadBytes) > 0,
        "the other snapshot's own request charged no read, so this case is empty"
    );

    // (3b) The same, for a definition that is *mutated* rather than foreign: a digest, a length or a
    //      variant that is not the bytes' own is refused by the request's own identity check, and the
    //      refusal is the one the direct path gives — the store neither answers it nor keeps anything
    //      from it.
    let mut wrong_digest = definition.clone();
    wrong_digest.class_bytes.digest = Digest("0".repeat(64));
    let mut wrong_length = definition.clone();
    wrong_length.class_bytes.length += 1;
    let mut wrong_variant = definition.clone();
    wrong_variant.variant = PhysicalVariant::MultiRelease { version: 9 };
    for (name, mutated) in [
        ("another digest", wrong_digest),
        ("another declared length", wrong_length),
        ("another variant", wrong_variant),
    ] {
        let without_store = engine
            .class_view(
                &snapshot,
                &tree_scope(),
                &ClassViewRequest {
                    class: ClassRef::Definition {
                        definition: mutated.clone(),
                    },
                    bodies: Vec::new(),
                },
                &mut request_budget(None),
            )
            .expect_err("a definition that does not describe the bytes at its location is refused");
        let before = foreign.report();
        let with_store = engine
            .class_view(
                &snapshot,
                &tree_scope(),
                &ClassViewRequest {
                    class: ClassRef::Definition {
                        definition: mutated,
                    },
                    bodies: Vec::new(),
                },
                &mut request_budget(Some(&foreign)),
            )
            .expect_err("a store does not make a wrong identity right");
        assert_eq!(
            code_of(&with_store),
            code_of(&without_store),
            "{name:?} is refused differently with a store attached: {} against {}",
            code_of(&with_store),
            code_of(&without_store)
        );
        assert_eq!(
            foreign.report().definition_read_hits,
            before.definition_read_hits,
            "{name:?} was answered from retention"
        );
    }

    // (4) Cancelled: a hit cannot revive a request that is already over, and the cancellation is what
    //     is reported.
    let warm = new_store(FactsCapacity::new(8, 1 << 20));
    let (_, _, _) = view_by_identity(&engine, &snapshot, &definition, Some(&warm));
    let before = warm.report();
    assert!(
        before.definition_reads > 0 && before.definition_read_hits == 0,
        "the store does not hold exactly this definition's read, so this case is empty: {before:?}"
    );
    let token = CancellationToken::new();
    token.cancel();
    let mut budget =
        Budget::with_cancellation_token(limits(), token).with_facts_cache(warm.clone());
    let error = engine
        .class_view(
            &snapshot,
            &tree_scope(),
            &ClassViewRequest {
                class: ClassRef::Definition {
                    definition: definition.clone(),
                },
                bodies: Vec::new(),
            },
            &mut budget,
        )
        .expect_err("a cancelled request is over");
    assert_eq!(code_of(&error), "cancelled", "{error}");
    assert_eq!(
        warm.report().definition_read_hits,
        before.definition_read_hits,
        "a hit was served to a cancelled request"
    );
    // The read entrance's own checkpoint, below the facade's: a cancelled request is refused before
    // anything is looked up, so a hit can never be the first thing a cancelled request reaches.
    let token = CancellationToken::new();
    token.cancel();
    let mut lookup =
        Budget::with_cancellation_token(limits(), token).with_facts_cache(warm.clone());
    let error = snapshot
        .retained_definition_read(&definition, &mut lookup)
        .expect_err("a cancelled request is refused by the read entrance itself");
    assert_eq!(code_of(&error), "cancelled", "{error}");
    assert_eq!(
        warm.report().definition_read_hits,
        before.definition_read_hits,
        "the refusal counted a hit"
    );

    // (5) An entry another declaration wrote: a handle reading under another identity discards what it
    //     finds instead of answering with it (the same rule the CP/Header and container layers keep).
    let declared = new_store(FactsCapacity::new(8, 1 << 20));
    let (_, _, _) = view_by_identity(&engine, &snapshot, &definition, Some(&declared));
    let before = declared.report();
    assert!(before.definition_read_stored > 0, "{before:?}");
    let reader = declared.over(FactsIdentity::new(
        HIGHEST_REGISTERED_MAJOR,
        FACTS_FORMAT + 1,
    ));
    let (view, usage, _) = view_by_identity(&engine, &snapshot, &definition, Some(&reader));
    let after = reader.report();
    assert_eq!(
        normalized(document(&view)),
        reference,
        "an entry of another declaration answered this request"
    );
    assert_eq!(
        charges(&usage, READ_DIMENSIONS),
        charges(&reference_usage, READ_DIMENSIONS),
        "an entry of another declaration answered the read"
    );
    assert_eq!(
        after.definition_read_hits, before.definition_read_hits,
        "an entry written under another declaration was served: {after:?}"
    );
    assert!(
        after.discarded_format + after.discarded_registry
            > before.discarded_format + before.discarded_registry,
        "the foreign entry was neither served nor discarded: {after:?}"
    );
    println!(
        "fallbacks: no store ({} charged), refused {}/{}, foreign snapshot {} hit(s), cancelled, \
         foreign declaration discarded {}/{}",
        charges(&reference_usage, READ_DIMENSIONS),
        refused.refused_capacity,
        refused.refused_capacity_bytes,
        after.definition_read_hits,
        after.discarded_format,
        after.discarded_registry
    );
}

// ---------------------------------------------------------------------------------------------
// R05 — a hit neither buys a budget nor claims a completion
// ---------------------------------------------------------------------------------------------

/// A read answered from retention removes exactly its own charge — and therefore may let a tight
/// request go further. Both halves are asserted: the stop a limit causes without a store is reported
/// truthfully, and the completion a hit makes possible reports only work that really fitted.
#[test]
fn a_tight_budget_stops_truthfully_with_and_without_a_hit() {
    let _gate = gate();
    let engine = Engine::new();
    let bytes = zip(&[
        (b"Shape.class", SHAPE, STORE),
        (b"notes.txt", b"note", STORE),
    ]);
    let snapshot = open(bytes);
    let definition = definition_at(&engine, &snapshot, None, 0);
    let entry_bytes = u64::try_from(SHAPE.len()).expect("the fixture fits u64");
    let request = ClassViewRequest {
        class: ClassRef::Definition {
            definition: definition.clone(),
        },
        bodies: Vec::new(),
    };

    // Cold: a budget one byte short of the entry's own read. The request stops before the entry is
    // read, and the dimension it names is the one that stopped it.
    let mut tight = limits();
    tight.entry_bytes = entry_bytes - 1;
    let cold_error = engine
        .class_view(
            &snapshot,
            &tree_scope(),
            &request,
            &mut Budget::new(tight.clone()),
        )
        .expect_err("a budget that cannot pay for the entry stops the request");
    assert_eq!(code_of(&cold_error), "EntryBytes", "{cold_error}");

    // Warm: the same tight budget, but the request's store already holds this definition's read.
    let warm = new_store(FactsCapacity::new(8, 1 << 20));
    let (roomy, roomy_usage, _) = view_by_identity(&engine, &snapshot, &definition, Some(&warm));
    let mut budget = Budget::new(tight).with_facts_cache(warm.clone());
    let view = match engine
        .class_view(&snapshot, &tree_scope(), &request, &mut budget)
        .expect("a request whose read is already paid for fits its budget")
    {
        OperationOutcome::Performed(view) => view,
        other => panic!("a class identity is never ambiguous or incomplete: {other:?}"),
    };
    // What it published is what the roomy request published, and what it paid is that request's
    // charges minus the read — so the completion is accounted for rather than asserted away.
    assert_eq!(
        normalized(document(&view)),
        normalized(document(&roomy)),
        "the hit changed what the request published"
    );
    let usage = budget.usage();
    assert!(
        charged_no_entry_read(&usage),
        "the request was answered from retention and still paid for a read: {}",
        charges(&usage, READ_DIMENSIONS)
    );
    assert_eq!(
        usage.counted_usage(CountedBudgetDimension::ClassHeaders),
        roomy_usage.counted_usage(CountedBudgetDimension::ClassHeaders),
        "the header attempt was waived"
    );
    println!(
        "cold stop `{}` (entry_bytes {} of {}); warm completion with the entry read waived [{}]",
        code_of(&cold_error),
        entry_bytes - 1,
        entry_bytes,
        charges(&usage, READ_DIMENSIONS)
    );

    // The other half: a limit that stops the request **after** the read is still a stop, whether or
    // not the read was answered from retention. A hit must not turn it into a completion.
    //
    // The member listing is the case, because its own item publication is where a stop is *published*
    // rather than raised: the class is read (a hit here), the member table is read, and then the first
    // item the listing would publish cannot be afforded. The limit is the reader's own charge, read
    // out of the generous run: the reader's item charges are the difference between what the run
    // charged and what it published, so the listing passes that and stops on its own first item.
    let (listing, generous_usage, _) = members(&engine, &snapshot, &definition, Some(&warm));
    let published = u64::try_from(listing.items.len()).expect("the item count fits u64");
    let charged = generous_usage.counted_usage(CountedBudgetDimension::ResultItems);
    assert!(
        charged > published && published > 0,
        "the generous run charged {charged} item(s) for {published} published item(s), so this case \
         cannot separate the reader's charges from the publication's"
    );
    let mut later = limits();
    later.result_items = charged - published;
    let (listing, listing_usage, _) =
        members_within(&engine, &snapshot, &definition, Some(&warm), later);
    assert!(
        listing.items.is_empty(),
        "an item was published under an allowance that stops at the first one: {:?}",
        listing.items
    );
    match &listing.execution {
        ExecutionReport::Partial { reason, .. } => assert!(
            matches!(
                reason,
                TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::ResultItems
                }
            ),
            "the stop does not name the limit that caused it: {reason:?}"
        ),
        other => panic!(
            "a request stopped after its read reported {other:?} instead of a partial execution"
        ),
    }
    // The hit is real and the stop is not: the store answered this request's read, and the request
    // still reports the limit that stopped it.
    assert!(
        budget.facts_cache().is_some() && warm.report().definition_read_hits > 0,
        "the case did not actually hit, so it says nothing about a hit under a stop"
    );
    assert!(
        charged_no_entry_read(&listing_usage),
        "the read this request was stopped after was not answered from retention: {}",
        charges(&listing_usage, READ_DIMENSIONS)
    );
    println!(
        "a hit under a spent item allowance: {} item(s) published, stop `{:?}`",
        listing.items.len(),
        listing.execution
    );

    // And a budget that is already spent is not refilled by a hit: one header attempt per request, so
    // the second request over a spent budget is refused where it asks, before any read.
    let mut spent = limits();
    spent.class_headers = 1;
    let mut budget = Budget::new(spent).with_facts_cache(warm.clone());
    let first = engine
        .class_view(&snapshot, &tree_scope(), &request, &mut budget)
        .expect("the first request fits its one header attempt");
    assert!(matches!(first, OperationOutcome::Performed(_)));
    let error = engine
        .class_view(&snapshot, &tree_scope(), &request, &mut budget)
        .expect_err("the second request over the same budget has no header attempt left");
    assert_eq!(code_of(&error), "ClassHeaders", "{error}");
    assert_eq!(
        budget
            .usage()
            .counted_usage(CountedBudgetDimension::ClassHeaders),
        1,
        "the refused attempt or the hit moved the account: {:?}",
        budget.usage()
    );
}

// ---------------------------------------------------------------------------------------------
// The capacity ladder and what it weighs
// ---------------------------------------------------------------------------------------------

/// The five capacities the container layer's own ladder uses, walked over the same sequence: every
/// tier publishes the same answer, the refusals are reported, and the residency is exactly the
/// definitions the store really kept.
#[test]
fn the_capacity_ladder_keeps_the_answer_and_states_what_it_refused() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(flat_fixture());
    let listed = definitions(&engine, &snapshot, None);
    assert!(
        listed.len() >= 3,
        "the fixture declares classes: {listed:?}"
    );
    let sizes: Vec<u64> = listed
        .iter()
        .map(|definition| definition.class_bytes.length)
        .collect();

    // The reference: every request with no store at all.
    let reference: Vec<String> = listed
        .iter()
        .map(|definition| {
            normalized(document(
                &view_by_identity(&engine, &snapshot, definition, None).0,
            ))
        })
        .collect();

    for (tier, capacity) in LADDER {
        let cache = new_store(capacity);
        let mut stored_sizes: Vec<u64> = Vec::new();
        let mut documents = Vec::new();
        // Each definition is asked for **twice**: the first request is the read, and the second is
        // the one a tier with room answers from it. A tier that refuses the read answers the second
        // request by reading the entry again, which is what the refusals are for.
        for round in 0..2 {
            for (index, definition) in listed.iter().enumerate() {
                let size = *sizes.get(index).expect("the definition's size");
                let before = cache.report();
                let (view, usage, _) =
                    view_by_identity(&engine, &snapshot, definition, Some(&cache));
                let after = cache.report();
                documents.push(normalized(document(&view)));
                if after.definition_read_stored > before.definition_read_stored {
                    stored_sizes.push(size);
                    assert_eq!(
                        after.definition_reads,
                        before.definition_reads + 1,
                        "the {tier} tier's request {round}/{index} stored a read it does not hold: \
                         {after:?}"
                    );
                }
                let answered = after.definition_read_hits > before.definition_read_hits;
                if round == 0 {
                    assert!(
                        !answered,
                        "the {tier} tier's first request for a definition was already a hit: \
                         {after:?}"
                    );
                }
                if answered {
                    assert!(
                        charged_no_entry_read(&usage),
                        "the {tier} tier's request {round}/{index} was answered and still charged a \
                         read: {}",
                        charges(&usage, READ_DIMENSIONS)
                    );
                } else {
                    assert!(
                        !charged_no_entry_read(&usage),
                        "the {tier} tier's request {round}/{index} was not answered yet charged no \
                         read: {}",
                        charges(&usage, READ_DIMENSIONS)
                    );
                    if round == 0 {
                        // The first request for a definition always reads the entry itself, whether
                        // or not the retention then accepted it.
                        assert_eq!(
                            usage.counted_usage(CountedBudgetDimension::EntryBytes),
                            size,
                            "the {tier} tier's first request for definition {index} did not read \
                             the entry itself: {usage:?}"
                        );
                    }
                }
            }
        }
        // Every definition is asked for twice, so a tier that kept a read must have answered the
        // second request from it — and a tier that kept none must have answered none.
        assert_eq!(
            cache.report().definition_read_hits,
            u64::try_from(stored_sizes.len()).expect("the count fits u64"),
            "the {tier} tier's hits are not the reads it kept and was asked for again: {:?}",
            cache.report()
        );
        // The documents are one answer per definition per round, in one order.
        let mut one_round = documents.clone();
        let second_round = one_round.split_off(listed.len());
        assert_eq!(
            one_round, reference,
            "the {tier} tier published a different answer"
        );
        assert_eq!(
            second_round, reference,
            "the {tier} tier's second round published a different answer"
        );

        let report = cache.report();
        assert_eq!(
            report.definition_reads,
            stored_sizes.len(),
            "the {tier} tier's kept reads are not the reads it accepted: {report:?}"
        );
        assert_eq!(
            report.definition_read_bytes,
            stored_sizes.iter().sum::<u64>(),
            "the {tier} tier's residency is not the definitions it holds: {report:?}"
        );
        assert!(
            report.retained_bytes >= report.definition_read_bytes,
            "the layer weighs more than the store holds: {report:?}"
        );
        assert!(
            report.definition_reads <= capacity.entries,
            "the {tier} tier kept more answers than it was declared: {report:?}"
        );
        assert!(
            report.definition_read_bytes <= capacity.retained_bytes,
            "the {tier} tier kept more bytes than it was declared: {report:?}"
        );
        assert!(
            report.definition_read_consultations
                >= u64::try_from(2 * listed.len()).expect("the fixture count fits u64"),
            "the {tier} tier consulted nothing, so its reading means nothing: {report:?}"
        );
        assert_eq!(
            report.definition_read_consultations,
            report.definition_read_hits + report.definition_read_misses,
            "the {tier} tier's consultations are not its hits and misses: {report:?}"
        );
        println!(
            "{tier}: {} request(s) over {} definition(s) twice -> {} kept ({} of {} retained bytes), \
             hits {}, misses {}, refused {}/{}",
            2 * listed.len(),
            listed.len(),
            report.definition_reads,
            report.definition_read_bytes,
            report.retained_bytes,
            report.definition_read_hits,
            report.definition_read_misses,
            report.refused_capacity,
            report.refused_capacity_bytes
        );
    }
}

/// One process reads one tier of the ladder, for the out-of-process RSS readings of the change's
/// evidence: the same sequence as [`the_capacity_ladder_keeps_the_answer_and_states_what_it_refused`],
/// printed as the store's own residency reading for whichever tier `JARDE_REUSE_TIER` names.
///
/// The variable is the **test's** parameter and nothing else: the library reads no environment, has no
/// reuse switch and no second path, and this case runs the same code whatever the variable says. Run
/// it under `/usr/bin/time -l` (macOS) once per tier to read a process's maximum RSS beside the
/// store's own weight, which is what "RSS and `retained_bytes` are reported apart" means here.
#[test]
fn one_process_reads_one_tier_of_the_capacity_ladder() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(flat_fixture());
    let listed = definitions(&engine, &snapshot, None);
    let tier = std::env::var("JARDE_REUSE_TIER").unwrap_or_else(|_| "roomy".to_owned());
    let (name, cache) = match tier.as_str() {
        // The baseline: the same sequence with no store at all, which is the path every entry point
        // had before this change could answer anything.
        "off" => ("off".to_owned(), None),
        _ => {
            let (tier_name, capacity) = LADDER
                .iter()
                .find(|(tier_name, _)| *tier_name == tier)
                .copied()
                .unwrap_or_else(|| {
                    panic!("unknown tier {tier:?}: expected `off` or one of the ladder's names")
                });
            (tier_name.to_owned(), Some(new_store(capacity)))
        }
    };
    let mut requests = 0_u64;
    for _round in 0..2 {
        for definition in &listed {
            let (_, _, _) = view_by_identity(&engine, &snapshot, definition, cache.as_ref());
            requests += 1;
        }
    }
    let summary = match &cache {
        Some(cache) => cache.report().residency(),
        None => "no store attached (the direct path)".to_owned(),
    };
    println!(
        "tier {name}: {requests} request(s) over {} definition(s); {summary}",
        listed.len(),
    );
    if let Some(cache) = &cache {
        assert!(
            cache.report().definition_reads <= listed.len(),
            "a run kept more definition reads than the fixture has definitions: {:?}",
            cache.report()
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The name search is not this path
// ---------------------------------------------------------------------------------------------

/// The reads a **search** performs for the candidates it confirms are the search's own cost: they are
/// not consulted here, not retained here and not answered here.
///
/// The case states it as a reading: listings and name-driven views over a roomy store consult the
/// definition-read layer zero times and charge exactly what they charge with no store at all — while
/// the identity-bound read of the same class under the same store does reach the layer.
#[test]
fn the_name_search_reads_its_candidates_itself() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(zip(&[
        (b"Shape.class", SHAPE, STORE),
        (b"notes.txt", b"note", STORE),
    ]));
    let first = declaration_at(&engine, &snapshot, None, 0);
    let name = first.declaration.this_class.escaped();
    let cache = new_store(FactsCapacity::new(8, 1 << 20));

    // Listings and name-driven views, twice each: the search path.
    for round in 0..2 {
        let mut budget = request_budget(Some(&cache));
        engine
            .list_class_declarations(&snapshot, &tree_scope(), &mut budget)
            .expect("the fixture's declarations are listable");
        let (_, name_usage) = view_by_name(&engine, &snapshot, &name, Some(&cache));
        let report = cache.report();
        assert_eq!(
            report.definition_read_consultations, 0,
            "round {round}: the name search consulted the definition-read layer: {report:?}"
        );
        assert_eq!(report.definition_read_stored, 0, "{report:?}");
        assert!(
            name_usage.counted_usage(CountedBudgetDimension::ReadBytes) > 0,
            "round {round}: the name-driven view read no candidate at all: {name_usage:?}"
        );
    }

    // The same class read **by identity** under the same store does reach the layer, so the zero
    // above is the search's path and not a store that does nothing.
    let (_, identity_usage, _) =
        view_by_identity(&engine, &snapshot, &first.definition, Some(&cache));
    let (_, second_usage, _) =
        view_by_identity(&engine, &snapshot, &first.definition, Some(&cache));
    let report = cache.report();
    assert_eq!(
        report.definition_read_hits, 1,
        "the identity-bound read never reached the layer: {report:?}"
    );
    assert!(
        charged_no_entry_read(&second_usage),
        "the second identity-bound read paid for a read the store held: {}",
        charges(&second_usage, READ_DIMENSIONS)
    );
    assert!(
        identity_usage.counted_usage(CountedBudgetDimension::ReadBytes) > 0,
        "the first identity-bound read charged nothing, so the pair says nothing"
    );
    println!(
        "name path: 0 consultations over two search rounds; identity path: {} hit(s) of {} \
         consultation(s)",
        report.definition_read_hits, report.definition_read_consultations
    );
}

/// One store answers its own snapshot's definitions across entry points, and answers nothing for
/// another snapshot's.
#[test]
fn one_store_answers_across_entry_points_and_not_across_snapshots() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(zip(&[
        (b"Shape.class", SHAPE, STORE),
        (b"SCOPE.class", SCOPE, STORE),
    ]));
    let second = open(flat_fixture());
    let cache = new_store(FactsCapacity::new(8, 1 << 20));
    let listed = definitions(&engine, &snapshot, Some(&cache));
    assert_eq!(listed.len(), 2, "{listed:?}");
    for definition in &listed {
        let (_, _, _) = view_by_identity(&engine, &snapshot, definition, Some(&cache));
    }
    let warm = cache.report();
    assert_eq!(
        (warm.definition_reads, warm.definition_read_stored),
        (2, 2),
        "{warm:?}"
    );
    assert_eq!(
        warm.definition_read_bytes,
        listed
            .iter()
            .map(|definition| definition.class_bytes.length)
            .sum::<u64>(),
        "the two reads kept do not weigh the two classes: {warm:?}"
    );

    // The member list of a definition already read: answered from the same read, with the header
    // attempt still charged.
    let (listing, listing_usage, _) = members(&engine, &snapshot, &listed[0], Some(&cache));
    assert!(
        !listing.items.is_empty(),
        "the fixture's members are listed"
    );
    assert!(
        charged_no_entry_read(&listing_usage),
        "a read the store holds was paid for again: {}",
        charges(&listing_usage, READ_DIMENSIONS)
    );
    assert_eq!(
        listing_usage.counted_usage(CountedBudgetDimension::ClassHeaders),
        1,
        "the header attempt was waived"
    );

    // Another snapshot's classes are its own: every one of its reads is a miss, and none of them is
    // answered by the first snapshot's retention.
    let other_defined = definitions(&engine, &second, Some(&cache));
    let before = cache.report();
    for definition in &other_defined {
        let (_, usage, _) = view_by_identity(&engine, &second, definition, Some(&cache));
        assert!(
            !charged_no_entry_read(&usage),
            "a definition of another snapshot was answered from this store: {}",
            charges(&usage, READ_DIMENSIONS)
        );
    }
    let after = cache.report();
    assert_eq!(
        after.definition_read_hits, before.definition_read_hits,
        "another snapshot's reads were answered from retention: {before:?} -> {after:?}"
    );
    assert_eq!(
        after.definition_read_stored,
        before.definition_read_stored
            + u64::try_from(other_defined.len()).expect("the count fits u64"),
        "another snapshot's reads were not retained as its own: {after:?}"
    );
    println!(
        "one store: {} definition(s) of two snapshots, {} read(s) kept, {} hit(s), {} miss(es)",
        after.definition_reads,
        after.definition_read_stored,
        after.definition_read_hits,
        after.definition_read_misses
    );
}

/// A premise for [`a_retained_read_answers_for_its_own_definition_and_for_no_other`]: the bytes this
/// target offers by hand are the bytes the entry really holds, so that offer is an honest read of the
/// fixture rather than a value the store was told to believe.
#[test]
fn the_offered_bytes_are_the_entrys_own_bytes() {
    let _gate = gate();
    let engine = Engine::new();
    let snapshot = open(zip(&[(b"Shape.class", SHAPE, STORE)]));
    let cached = new_store(FactsCapacity::new(8, 1 << 20));
    let definition = definition_at(&engine, &snapshot, Some(&cached), 0);
    let record = snapshot
        .container_record(
            definition
                .entry()
                .expect("the fixture definition is an entry"),
            &mut request_budget(None),
        )
        .expect("the container reads")
        .expect("the entry exists");
    let materialized = snapshot
        .read_entry_for_analysis(&record, &mut request_budget(None))
        .expect("the entry reads");
    assert_eq!(
        materialized.bytes, SHAPE,
        "the fixture's entry does not hold the bytes this target offers by hand"
    );
    assert_eq!(
        materialized.content_digest,
        digest_of(SHAPE),
        "the fixture's entry digest is not the digest of the bytes it holds"
    );
    assert_eq!(
        definition.class_bytes,
        class_bytes_of(SHAPE),
        "the listed definition does not name the bytes the entry holds"
    );
}
