//! The active container facts a read hands to its consumers, and the class-bytes ceiling a class
//! read is admitted under (independent review R2/R5/R8, reader half).
//!
//! What this file pins, and why it is a file of its own:
//!
//! * **A container read once per walk is read once for the walk.** A scope walk holds the container
//!   it is inside ([`jarde::ArtifactSnapshot::scope_cursor`]'s cursor, and the read
//!   [`jarde::ArtifactSnapshot::prepared_class`] hands back), and every later read of that container
//!   — the class entry's own verified read and a method request's loader binding query — is answered
//!   from the product those handles are *already holding*, never from a fresh directory parse. The
//!   fixtures below measure it on the dimension that carries it: `archive_entries`, one charge per
//!   entry a directory parse examines, in **four** store configurations (none attached, zero
//!   capacity, a capacity too small for a container product, and a store large enough to retain
//!   one). The reviewer's counterexample was 92 entries without a store against 4 with one; the
//!   number now is the container's own entry count in all four, because the reuse follows the live
//!   handle and not the store's admission decision.
//! * **The bulk operation the review measured is the same shape**: the numbers above are also
//!   produced through [`jarde::Engine::recover_all`] on the same four-entry flat fixture, so the
//!   claim is about the operation and not only about a hand-wired reader test.
//! * **A reuse ends when its last consumer does.** Without a live handle and without retention the
//!   directory *is* parsed again — the reuse is a lifetime, not a hidden store.
//! * **One preparation, one constant pool, one member table.** Every method of a prepared class
//!   consumes the very bundle its preparation produced: the payload's pool *is* the prepared class's
//!   pool allocation, proven by pointer identity rather than by timing.
//! * **The binding path shares that bundle instead of copying it.** The JVM consumers take the
//!   prepared class's facts by handle; the guard at the end of this file reads the two modules that
//!   decide it, because a deep copy of a constant pool is not visible in any published number.
//! * **A class that crosses its ceiling is refused before it is materialized**, and a directory
//!   record that understates its class does not get the whole entry past that ceiling.

mod bulk_support;

use bulk_support::{FLAT_PREFIXES, Recorder, container_roots, environment, flat_fixture};
use jarde::*;
use jarde_jvm::engine::analyze_prepared_method_ir;
use jarde_reader::model::physical_variant_for_path;
use jarde_reader::prepared::PreparedClass;
use std::path::Path;
use std::ptr;

/// The four-entry flat fixture this file measures: its root container holds exactly the four class
/// entries the scope walks, which is the number `archive_entries` has to equal.
const ROOT_ENTRIES: u64 = 4;

/// The container the four classes of the flat fixture live in, as an environment's own root.
fn flat_roots(snapshot: &ArtifactSnapshot) -> Vec<LoadRoot> {
    let mut setup = Budget::new(bulk_support::limits());
    let roots = container_roots(snapshot, &mut setup, &FLAT_PREFIXES);
    assert_eq!(roots.len(), 1, "the flat fixture has one searched root");
    roots
}

/// The explicit classpath a method request runs under: the snapshot's own root container, searched
/// with no prefix, exactly as the bulk operation's environment does it.
fn resolution_environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
    let domain = LoadDomain {
        loader: LoaderId("app".to_owned()),
        parent_loader: None,
        delegation: DelegationPolicy::ParentFirst,
        roots: flat_roots(snapshot),
        module_mode: ModuleMode::ClassPath,
        external_override: RuntimeUncertainty::None,
        runtime_transformation: RuntimeUncertainty::None,
    };
    ResolutionEnvironment {
        runtime: RuntimeView {
            physical: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: bulk_support::tree_scope(),
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

/// One budget under one facts-store configuration, and the name a failure states it by.
///
/// The four configurations are the reviewer's: no store at all, a store that retains nothing, a
/// store too small for even one container product, and a store with room for one.
fn open_budget(configuration: &str) -> Budget {
    let limits = bulk_support::limits();
    match configuration {
        "no store attached" => Budget::new(limits),
        "zero capacity" => Budget::new(limits).with_facts_cache(FactsCache::new(
            FactsIdentity::current(),
            FactsCapacity::none(),
        )),
        "one byte of retained weight" => Budget::new(limits).with_facts_cache(FactsCache::new(
            FactsIdentity::current(),
            FactsCapacity::new(1, 1),
        )),
        "a store that can retain a container" => Budget::new(limits).with_facts_cache(
            FactsCache::new(FactsIdentity::current(), FactsCapacity::new(8, 1 << 20)),
        ),
        other => panic!("unknown store configuration `{other}`"),
    }
}

const STORE_CONFIGURATIONS: [&str; 4] = [
    "no store attached",
    "zero capacity",
    "one byte of retained weight",
    "a store that can retain a container",
];

/// Walks the fixture's scope with a cursor and prepares every class it yields, analysing every
/// member through the prepared class — the class-task loop a bulk operation runs, without the
/// facade.
///
/// The wiring is deliberately the caller's own: the cursor hands its container out as the active
/// handle ([`jarde_reader::scope_cursor::ScopeCursor::container_facts`]) and the class reads reach
/// the same product by origin, and every method request runs the same loader binding query the
/// operation runs. Nothing here stores a fact of its own.
fn walk_and_analyse(
    snapshot: &ArtifactSnapshot,
    content: &[ArtifactSnapshot],
    budget: &mut Budget,
) {
    let environment = resolution_environment(snapshot);
    let mut cursor = snapshot
        .scope_cursor(&bulk_support::tree_scope())
        .expect("the flat fixture is a zip and its tree scope is its own root");
    let mut prepared_classes = 0_u64;
    while let Some(class) = cursor
        .next_class(budget)
        .expect("the walk runs under budget")
    {
        let entry = class.entry.expect("every candidate of a zip is an entry");
        // The walk's own container, handed over as the active handle a class task keeps: it is the
        // same product the class read and every binding query below reach by origin.
        let held = cursor
            .container_facts()
            .expect("a walk inside a container holds its facts");
        assert_eq!(
            held.origin().current_container(),
            &jarde::ContainerId("root".into()),
            "the walk hands over the container it is inside"
        );
        // The class task runs where the walk is: the cursor is inside this container for as long as
        // its classes are read, exactly as a bulk coordinator's discovery loop keeps it.
        let read = snapshot
            .prepared_class(&entry, budget)
            .expect("a class of the fixture is read under budget");
        assert_eq!(
            read.container_facts()
                .expect("a class read carries the container it came from")
                .origin(),
            held.origin(),
            "and the class read reaches that very product"
        );
        let prepared = PreparedClass::prepare(&read, budget).expect("the class prepares");
        for slot in prepared.method_slots() {
            let request = MethodAnalysisRequest {
                environment: environment.clone(),
                method: jarde_reader::model::PhysicalMethodId {
                    owner: definition_of(&read),
                    name: slot.name.raw().clone(),
                    descriptor: slot.descriptor.raw().clone(),
                },
                stages: AnalysisStage::ALL.to_vec(),
            };
            let _ = analyze_prepared_method_ir(content, &prepared, &request, budget)
                .expect("every fixture member is answerable from its prepared class");
        }
        prepared_classes += 1;
    }
    assert_eq!(
        prepared_classes, ROOT_ENTRIES,
        "the fixture's root holds four class candidates, all of them prepared"
    );
}

/// The physical definition one prepared read is, as the request that consumes it names it.
fn definition_of(read: &jarde_reader::prepared::PreparedClassRead) -> PhysicalDefinitionId {
    let variant = match read.location.entry() {
        Some(entry) => physical_variant_for_path(&entry.raw_name.0),
        None => PhysicalVariant::Base,
    };
    PhysicalDefinitionId {
        location: read.location.clone(),
        class_bytes: read.class_bytes.clone(),
        variant,
    }
}

#[test]
fn a_scope_walk_reads_the_container_once_whatever_its_store_is_doing() {
    // The reviewer's counterexample, as a number: this fixture's root container holds four entries
    // and the whole walk prepares four classes and analyses eighteen methods out of it. Whatever the
    // caller's store does — nothing, refuse everything, refuse the container, or retain it — the
    // directory is parsed once, so the walk bills the container's own entry count and nothing more.
    for configuration in STORE_CONFIGURATIONS {
        let (snapshot, _opened) = bulk_support::open(flat_fixture());
        let content = vec![snapshot.clone()];
        let mut setup = Budget::new(bulk_support::limits());
        let entries = snapshot
            .enumerate(&mut setup)
            .expect("the fixture's directory is enumerable")
            .entries
            .len() as u64;
        assert_eq!(
            entries, ROOT_ENTRIES,
            "the fixture is what this test states"
        );

        let mut budget = open_budget(configuration);
        walk_and_analyse(&snapshot, &content, &mut budget);
        assert_eq!(
            budget.usage().archive_entries,
            ROOT_ENTRIES,
            "{configuration}: the walk parses the container directory once, so the charges are its \
             entries and nothing else; {:?}",
            budget.usage()
        );
    }
}

#[test]
fn the_bulk_operation_bills_the_container_once_in_every_store_configuration() {
    // The same number through the operation the review measured. The walk holds the container, every
    // class read reaches that product through the snapshot, and every method request's binding query
    // is answered from it — so the operation's own account does not move with the store's admission
    // decision.
    for configuration in STORE_CONFIGURATIONS {
        let (snapshot, _opened) = bulk_support::open(flat_fixture());
        let content = vec![snapshot.clone()];
        let mut setup = Budget::new(bulk_support::limits());
        let roots = container_roots(&snapshot, &mut setup, &FLAT_PREFIXES);
        let environment = environment(&snapshot, bulk_support::tree_scope(), roots);
        let request = bulk_support::request(environment, 1);
        let mut budget = open_budget(configuration);
        let mut sink = Recorder::new();
        let report = Engine::new()
            .recover_all(&content, &request, &mut budget, &mut sink)
            .expect("the fixture's scope is recoverable");
        assert_eq!(
            report.summary.status(),
            "complete",
            "{configuration}: the operation really ran: {:?}",
            report.summary
        );
        assert_eq!(
            report.summary.classes_prepared, 4,
            "{configuration}: four classes are prepared"
        );
        assert_eq!(
            report.usage.archive_entries, ROOT_ENTRIES,
            "{configuration}: the operation's own account bills the container's entries once, not \
             one directory parse per method binding: {:?}",
            report.usage
        );
    }
}

#[test]
fn a_container_is_read_again_only_after_its_last_consumer_let_go() {
    // Reuse follows a lifetime and nothing else. A held container is not rebuilt even when the store
    // that could have retained it is cleared in the middle of the walk; and once the last handle is
    // gone — the reads dropped and the store cleared — the next class of that container pays for the
    // directory again, because at that point nothing holds it.
    let (snapshot, _opened) = bulk_support::open(flat_fixture());
    let mut setup = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut setup, &FLAT_PREFIXES);
    assert_eq!(roots.len(), 1);

    let store = FactsCache::current(FactsCapacity::new(8, 1 << 20));
    let mut budget = Budget::new(bulk_support::limits()).with_facts_cache(store.clone());
    let mut cursor = snapshot
        .scope_cursor(&bulk_support::tree_scope())
        .expect("the tree scope is the root container");
    let first = cursor
        .next_class(&mut budget)
        .expect("the walk runs")
        .expect("the fixture declares a class first");
    let entry = first.entry.expect("a zip candidate is an entry");
    let read = snapshot
        .prepared_class(&entry, &mut budget)
        .expect("the first class reads");
    let held = read
        .container_facts()
        .expect("a class read out of a container carries its facts");

    let after_read = budget.usage().archive_entries;
    assert_eq!(after_read, ROOT_ENTRIES, "the directory was parsed once");

    // The store forgets everything, and the read keeps the container alive: the next class of that
    // container is read from the product the read is holding, not from a second parse.
    store.clear();
    let second = cursor
        .next_class(&mut budget)
        .expect("the walk runs")
        .expect("the fixture declares a second class");
    let second_entry = second.entry.expect("a zip candidate is an entry");
    let second_read = snapshot
        .prepared_class(&second_entry, &mut budget)
        .expect("the second class reads");
    assert_eq!(
        budget.usage().archive_entries,
        after_read,
        "a cleared store does not make a held container be read again; {:?}",
        budget.usage()
    );
    assert_eq!(
        held.origin(),
        second_read
            .container_facts()
            .expect("the second read carries the same container")
            .origin(),
        "both reads state the container they came from"
    );

    // Nothing holds it any more: every read is dropped, the walk itself is dropped (it was the
    // holder that outlived them), the store is empty again — and the same class is read again, at
    // the price of a directory. This is the control that makes the assertions above evidence of a
    // lifetime rather than of a hidden store.
    drop(read);
    drop(second_read);
    drop(held);
    drop(cursor);
    store.clear();
    let reread = snapshot
        .prepared_class(&entry, &mut budget)
        .expect("the first class reads again");
    drop(reread);
    assert_eq!(
        budget.usage().archive_entries,
        after_read + ROOT_ENTRIES,
        "with no holder and no retention, the directory is parsed again: {:?}",
        budget.usage()
    );
}

#[test]
fn a_class_crossing_its_ceiling_is_refused_before_it_is_materialized() {
    // The pre-materialization half of a class-bytes ceiling: the refusal is decided from the
    // container directory's own record, before the entry is selected, decompressed or charged. A
    // caller that admits one byte of class therefore pays nothing for the four classes it refuses —
    // where the review measured 1,813 bytes already materialized.
    let (snapshot, _opened) = bulk_support::open(flat_fixture());
    let mut setup = Budget::new(bulk_support::limits());
    let mut cursor = snapshot
        .scope_cursor(&bulk_support::tree_scope())
        .expect("the tree scope is the root container");
    let mut classes = 0_u64;
    while let Some(class) = cursor.next_class(&mut setup).expect("the walk runs") {
        let entry = class.entry.expect("a zip candidate is an entry");
        let before = setup.usage();
        let refused = match snapshot.prepared_class_within(&entry, 1, &mut setup) {
            Ok(read) => panic!(
                "no fixture class is one byte, but this one was admitted: {} bytes",
                read.class_bytes.length
            ),
            Err(error) => error_code(error),
        };
        assert_eq!(
            refused, "class_bytes_ceiling",
            "the ceiling is refused under its own code"
        );
        let after = setup.usage();
        assert_eq!(
            after.entry_bytes, before.entry_bytes,
            "nothing of the refused class was materialized"
        );
        assert_eq!(
            after.read_bytes, before.read_bytes,
            "and nothing of it was selected"
        );
        assert_eq!(
            after.output_bytes, before.output_bytes,
            "and nothing of it was handed on"
        );
        classes += 1;
    }
    assert_eq!(classes, ROOT_ENTRIES, "every class was walked and refused");

    // The same entries are admitted under a ceiling that really admits them, and the read is the one
    // the ceiling-free entry performs: the ceiling refuses classes, it does not change what a class
    // that fits is read as.
    let entry = first_entry(&snapshot);
    let declared = declared_class_length(&snapshot, &entry);
    let admitted = snapshot
        .prepared_class_within(&entry, declared, &mut setup)
        .expect("the class fits exactly");
    assert_eq!(
        admitted.class_bytes.length, declared,
        "the admitted read states the class's own length"
    );
    assert!(snapshot.prepared_class(&entry, &mut setup).is_ok());
}

#[test]
fn a_directory_record_that_understates_its_class_stops_the_read_at_the_ceiling() {
    // The second, continuous half of the ceiling: the record the directory states is *declared*
    // evidence, and a class whose bytes are longer than that record claims must not be materialized
    // past the ceiling either. The fixture below is the flat one with its first entry's declared
    // uncompressed size rewritten to `DECLARED` — the descriptor and the central record agree on it,
    // so every verification before the read still passes and only the read itself can tell.
    const DECLARED: u64 = 16;
    let bytes = understated_first_entry(flat_fixture(), DECLARED as u32);
    let (snapshot, _opened) = bulk_support::open(bytes);
    let entry = first_entry(&snapshot);

    // Without a ceiling the read produces the whole entry and only then finds that the record and
    // the bytes disagree — the entry's own integrity check is the thing that catches it, at the end
    // of the stream. The classes the review measured were paid for exactly this way: the whole class
    // materialized before anything refused it.
    let mut unceiled = Budget::new(bulk_support::limits());
    let error = snapshot
        .prepared_class(&entry, &mut unceiled)
        .expect_err("the understated record contradicts the bytes");
    assert!(
        matches!(error_code(error).as_str(), "entry_integrity"),
        "an unceiled read is refused by the entry's own integrity check"
    );
    let real = unceiled.usage().entry_bytes;
    assert!(
        real > DECLARED,
        "the fixture's first class is longer than the record claims ({real} bytes)"
    );

    // With a ceiling between the declared size and the real one, the read stops **at the ceiling**:
    // each chunk is asked for with only the bytes the ceiling still allows, so the class that the
    // record understated cannot be produced at all — the bytes the read charges are bounded by the
    // ceiling, not by the class. (The entry's own integrity check is what names the refusal here,
    // because it is what discovers that the stream is longer than the entry declared itself to be.)
    let ceiling = DECLARED * 4;
    let mut ceiled = Budget::new(bulk_support::limits());
    let error = snapshot
        .prepared_class_within(&entry, ceiling, &mut ceiled)
        .expect_err("the class cannot be produced inside the ceiling");
    assert!(
        matches!(
            error_code(error).as_str(),
            "class_bytes_ceiling" | "entry_integrity"
        ),
        "the read is refused rather than answered"
    );
    assert!(
        ceiled.usage().entry_bytes <= ceiling + 1,
        "the read produced at most one byte past the ceiling, not the whole {real}-byte class: {:?}",
        ceiled.usage()
    );
    assert!(
        ceiled.usage().entry_bytes < real,
        "and strictly less than the class the record understated: {:?}",
        ceiled.usage()
    );
}

#[test]
fn one_preparation_produces_one_pool_for_every_method_of_its_class() {
    // The copy the review found: the prepared driver took the class's whole constant pool out of the
    // prepared facts, per method. The payload's pool is now the *preparation's own allocation*, which
    // pointer identity proves on a class with more than one method: every payload of one class reads
    // the same `CpEntryFacts` slice, and that slice is the prepared class's own pool.
    let (snapshot, _opened) = bulk_support::open(flat_fixture());
    let content = vec![snapshot.clone()];
    let entry = first_entry(&snapshot);
    let mut budget = Budget::new(bulk_support::limits());
    let read = snapshot
        .prepared_class(&entry, &mut budget)
        .expect("the class reads");
    let prepared = PreparedClass::prepare(&read, &mut budget).expect("the class prepares");
    let environment = resolution_environment(&snapshot);

    let mut analyses = Vec::new();
    for slot in prepared.method_slots() {
        if slot.code == jarde_reader::prepared::MethodCodeAttribute::Absent {
            continue;
        }
        let request = MethodAnalysisRequest {
            environment: environment.clone(),
            method: jarde_reader::model::PhysicalMethodId {
                owner: definition_of(&read),
                name: slot.name.raw().clone(),
                descriptor: slot.descriptor.raw().clone(),
            },
            stages: AnalysisStage::ALL.to_vec(),
        };
        let analyzed =
            analyze_prepared_method_ir(&content, &prepared, &request, &mut budget).expect("runs");
        assert!(
            !analyzed.ir().constant_pool().is_empty(),
            "the fixture's class declares a constant pool"
        );
        assert!(
            ptr::eq(
                prepared.facts_handle().constant_pool.as_slice(),
                analyzed.ir().constant_pool()
            ),
            "the payload's pool is the preparation's own pool allocation, not a copy of it"
        );
        analyses.push(analyzed);
    }
    assert!(
        analyses.len() > 1,
        "the fixture's first class declares more than one body"
    );
    let pools: Vec<&[jarde_reader::classfile::CpEntryFacts]> = analyses
        .iter()
        .map(|analyzed| analyzed.ir().constant_pool())
        .collect();
    for pool in &pools[1..] {
        assert!(
            ptr::eq(pools[0], *pool),
            "every method of one prepared class reads the same pool allocation"
        );
    }
}

#[test]
fn the_jvm_consumers_share_the_prepared_bundle_instead_of_copying_it() {
    // The second half of the same finding: the binding path deep-copied the whole `ClassFacts` bundle
    // — the pool and the member tables — once per method, and no published number can show that,
    // because a copy is not charged. The guard reads the two modules that decide it.
    let providers = read_repository_file("crates/jarde-jvm/src/providers.rs");
    let bundle = providers
        .find("pub(crate) struct ClassHeaderFacts {")
        .expect("the header bundle is declared in this module");
    let bundle = &providers[bundle..];
    let bundle = &bundle[..bundle.find("\n}\n").expect("the bundle declaration ends")];
    assert!(
        bundle.contains("pub(crate) facts: Arc<ClassFacts>,"),
        "the header bundle holds the facts by handle: {bundle}"
    );
    let bind = providers
        .find("pub(crate) fn bind_definition(")
        .expect("the binding entry is in this module");
    let bind = &providers[bind..];
    let bind = &bind[..bind
        .find("\n    /// The header of a decided")
        .expect("the binding entry ends where the next helper begins")];
    assert!(
        bind.contains("facts: &Arc<ClassFacts>,"),
        "the binding entry takes the caller's own handle: {bind}"
    );
    assert!(
        bind.contains("facts: Arc::clone(facts),"),
        "and shares it into the resolution instead of copying the bundle: {bind}"
    );
    assert!(
        !bind.contains("facts.clone()"),
        "no deep copy of the bundle is left in the binding path: {bind}"
    );

    let engine = read_repository_file("crates/jarde-jvm/src/engine.rs");
    let prepared = engine
        .find("fn read_prepared_driver_method(")
        .expect("the prepared driver pass is in this file");
    let prepared = &engine[prepared..];
    let prepared = &prepared[..prepared
        .find("\n/// The termination and diagnostic of a reader")
        .expect("the prepared pass ends where the next helper begins")];
    assert!(
        prepared.contains("facts: Arc::clone(prepared.facts_handle()),"),
        "the prepared pass shares the class's own bundle with the run: {prepared}"
    );
    assert!(
        !prepared.contains("constant_pool.clone()"),
        "and copies no constant pool per method: {prepared}"
    );
    assert!(
        engine.contains("facts: Arc::clone(&read.header.facts),"),
        "the direct read shares the bundle its own single read produced"
    );
    // The single-method entry's own discipline is unchanged: the same one read, the same one decode
    // and the same declaration taken from the header facts that read holds.
    let direct = engine
        .find("fn read_driver_method(")
        .expect("the direct driver pass is in this file");
    let direct = &engine[direct..];
    let direct = &direct[..direct
        .find("\n/// Whether the member declares a `Code` attribute")
        .expect("the pass ends where the next helper begins")];
    assert_eq!(
        direct.matches("method_code_facts(").count(),
        1,
        "the direct read still decodes exactly one body"
    );
    assert!(
        !direct.contains("std::mem::take"),
        "and no longer takes the pool out of the facts it shares"
    );
}

// -----------------------------------------------------------------------------------------------
// Fixtures and small readers
// -----------------------------------------------------------------------------------------------

/// The first class candidate of one zip snapshot, as the walk yields it.
fn first_entry(snapshot: &ArtifactSnapshot) -> jarde_reader::model::PhysicalEntryId {
    let mut budget = Budget::new(bulk_support::limits());
    let mut cursor = snapshot
        .scope_cursor(&bulk_support::tree_scope())
        .expect("a zip snapshot's tree scope is its own root");
    cursor
        .next_class(&mut budget)
        .expect("the walk runs")
        .expect("the fixture declares a class")
        .entry
        .expect("a zip candidate is an entry")
}

/// The class length one entry's directory record states.
fn declared_class_length(
    snapshot: &ArtifactSnapshot,
    entry: &jarde_reader::model::PhysicalEntryId,
) -> u64 {
    let mut budget = Budget::new(bulk_support::limits());
    snapshot
        .enumerate(&mut budget)
        .expect("the directory is enumerable")
        .entries
        .into_iter()
        .find(|record| record.id == *entry)
        .expect("the entry is in the directory")
        .uncompressed_size
}

/// The flat fixture with its first entry's declared uncompressed size rewritten to `declared`.
///
/// The two places a ZIP states that size for this fixture are written together — the data descriptor
/// that follows the local entry's data and the central-directory record — so every check *before* the
/// read still holds (the descriptor and the record agree, and the local header states no size because
/// the entry declares a descriptor) and only the read itself can find the entry longer than it was
/// declared.
fn understated_first_entry(mut bytes: Vec<u8>, declared: u32) -> Vec<u8> {
    let archive = rawzip::ZipArchive::from_slice(&bytes).expect("the fixture is a zip");
    let mut entries = archive.entries();
    let header = entries
        .next_entry()
        .expect("the fixture's central directory is readable")
        .expect("the fixture declares an entry");
    let local = archive
        .get_entry(header.wayfinder())
        .expect("the entry's local header is readable");
    let (_, data_end) = local.compressed_data_range();
    let descriptor = data_end as usize;
    assert_eq!(
        &bytes[descriptor..descriptor + 4],
        b"PK\x07\x08",
        "the entry declares its sizes in a data descriptor"
    );
    // Signature, CRC, compressed size, uncompressed size.
    bytes[descriptor + 12..descriptor + 16].copy_from_slice(&declared.to_le_bytes());

    let eocd = bytes
        .windows(4)
        .rposition(|window| window == b"PK\x05\x06")
        .expect("the fixture has an end-of-directory record");
    let central = u32::from_le_bytes(
        bytes[eocd + 16..eocd + 20]
            .try_into()
            .expect("the end-of-directory record states the directory offset"),
    ) as usize;
    assert_eq!(
        &bytes[central..central + 4],
        b"PK\x01\x02",
        "the central directory starts where the record says"
    );
    // Signature, version, flags, method, times, CRC, compressed size, uncompressed size.
    bytes[central + 24..central + 28].copy_from_slice(&declared.to_le_bytes());
    bytes
}

/// The refusal code of one structured error.
fn error_code(error: Error) -> String {
    match error {
        Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code,
        other => panic!("expected a structured refusal, got {other}"),
    }
}

/// One repository file, for the guards that read a seam's own source.
fn read_repository_file(relative: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(root.join(relative))
        .unwrap_or_else(|error| panic!("{relative} is readable: {error}"))
}
