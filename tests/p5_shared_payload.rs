//! 2.2 (payload half): the facts one class's bytes produce are **shared**, not rebuilt per consumer.
//!
//! `openspec/changes/add-parallel-bulk-recovery` task 2.2 asks that the CP/Header payload of one
//! prepared class be handed to every consumer as the handle the store holds, and that a consumer that
//! already knows the class's content digest does not make the store hash the same backing again. This
//! file holds what a design sentence cannot: that the handle really is one allocation, that it stays
//! readable — with the content it was parsed from — after the store is cleared, after the store has
//! room for nothing, and after a later read writes the same key again, and that a trusted digest is
//! the *same key* as the bytes it was computed from.
//!
//! ```text
//! verify: cargo test --test p5_shared_payload --locked
//! ```
//!
//! What this file is not
//! --------------------
//!
//! The identity, invalidation, refusal-versus-eviction and counter semantics of the store are
//! `tests/p5_facts_cache.rs`, unchanged by this slice, and the container product of the same store is
//! `tests/p5_container_lookup.rs`. What is asserted here is only the *ownership* half: who holds a
//! payload, and what a hold survives. `tests/p1_budget_ledger.rs` holds the other half of slice 4.1,
//! the operation's shared total.

use jarde::*;
use std::sync::Arc;

/// The class the P5 corpus pins, under two releases: same definition, two contents.
const CONTROL: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v52/HistoricalControlFlow.class");
const OTHER: &[u8] =
    include_bytes!("fixtures/historical/ecj-4.6.1/v48/HistoricalControlFlow.class");

/// Headroom for one request, the same shape the other P5 files start from.
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

/// A fresh store holding up to `entries` answers.
fn store(entries: usize) -> FactsCache {
    FactsCache::current(FactsCapacity::new(entries, u64::MAX))
}

/// The facts the direct path produces for `bytes`, with no store in the request at all.
fn direct(bytes: &[u8]) -> ClassFacts {
    class_facts(bytes, &mut Budget::new(limits())).expect("the fixture parses")
}

/// One request over `bytes` that may consult `store`.
fn through(store: &FactsCache) -> Budget {
    Budget::new(limits()).with_facts_cache(store.clone())
}

/// The content identity a verified read publishes for `bytes`.
fn content_id(bytes: &[u8]) -> ClassBytesId {
    ClassBytesId {
        digest: Digest(blake3::hash(bytes).to_hex().to_string()),
        length: bytes.len() as u64,
    }
}

/// One content is one payload: every consumer reads the allocation the store holds.
#[test]
fn the_store_hands_out_the_one_allocation_it_holds() {
    let expected = direct(CONTROL);
    let store = store(4);
    let mut cold = through(&store);
    assert_eq!(
        class_facts(CONTROL, &mut cold).expect("the fixture parses"),
        expected
    );

    let mut consumer = through(&store);
    let first = store
        .structure_shared(CONTROL, &mut consumer)
        .expect("a lookup answers")
        .expect("the cold read stored the facts");
    let second = store
        .structure_shared(CONTROL, &mut consumer)
        .expect("a lookup answers")
        .expect("the second lookup is the same hit");
    assert!(
        Arc::ptr_eq(&first, &second),
        "two lookups of one content handed out two payloads, so a class consumed by several \
         methods is copied once per consumer"
    );
    assert_eq!(*first, expected);
    assert_eq!(
        consumer.usage().class_bytes,
        0,
        "a hit parsed class bytes, so it is not the shared payload it claims to be"
    );
    assert_eq!(store.report().hits, 2, "{:?}", store.report());
}

/// A handle a consumer holds is not the store's to release.
#[test]
fn a_live_handle_survives_a_clear() {
    let expected = direct(CONTROL);
    let store = store(4);
    let mut budget = through(&store);
    assert_eq!(
        class_facts(CONTROL, &mut budget).expect("the fixture parses"),
        expected
    );
    let held = store
        .structure_shared(CONTROL, &mut budget)
        .expect("a lookup answers")
        .expect("the read stored the facts");

    store.clear();
    assert_eq!(
        store.report().entries,
        0,
        "the store still retains an answer"
    );
    assert_eq!(
        *held, expected,
        "clearing the store invalidated or changed a payload a consumer already holds"
    );
    assert!(
        store
            .structure_shared(CONTROL, &mut budget)
            .expect("a lookup answers")
            .is_none(),
        "a cleared store answered from memory"
    );

    // The next read parses again — that is a new payload, and it does not touch the held one.
    let rebuilt = {
        let mut again = through(&store);
        class_facts(CONTROL, &mut again).expect("the fixture parses");
        store
            .structure_shared(CONTROL, &mut again)
            .expect("a lookup answers")
            .expect("the later read stored the facts")
    };
    assert!(
        !Arc::ptr_eq(&held, &rebuilt),
        "the rebuild handed back the payload the cleared store had released"
    );
    assert_eq!(*held, expected);
    assert_eq!(
        *rebuilt, expected,
        "the rebuilt entry answers another value"
    );
}

/// A store that is full refuses the new answer instead of evicting what a consumer holds.
#[test]
fn a_full_store_refuses_without_invalidating_a_live_payload() {
    let expected = direct(CONTROL);
    let store = store(1);
    let mut first_budget = through(&store);
    assert_eq!(
        class_facts(CONTROL, &mut first_budget).expect("the fixture parses"),
        expected
    );
    let held = store
        .structure_shared(CONTROL, &mut first_budget)
        .expect("a lookup answers")
        .expect("the read stored the facts");

    let mut second_budget = through(&store);
    assert_eq!(
        class_facts(OTHER, &mut second_budget).expect("the other fixture parses"),
        direct(OTHER),
        "a refused store changed the answer of the request it refused"
    );
    assert_eq!(
        store.report().refused_capacity,
        1,
        "the second class was not refused, so this case proves nothing: {:?}",
        store.report()
    );
    assert_eq!(
        *held, expected,
        "a capacity refusal evicted or changed the payload the store still holds"
    );
    assert!(
        Arc::ptr_eq(
            &held,
            &store
                .structure_shared(CONTROL, &mut first_budget)
                .expect("a lookup answers")
                .expect("the store still holds the class it accepted")
        ),
        "the store stopped answering the payload it still holds"
    );
}

/// A store with room for nothing is a counter: it retains nothing and changes no answer.
#[test]
fn a_store_with_room_for_nothing_retains_nothing_and_answers_no_wrong_value() {
    let expected = direct(CONTROL);
    let counter = FactsCache::current(FactsCapacity::none());
    assert_eq!(counter.capacity(), FactsCapacity::none());

    let mut budget = through(&counter);
    assert_eq!(
        class_facts(CONTROL, &mut budget).expect("the fixture parses"),
        expected,
        "a store that retains nothing changed what the request reads"
    );
    assert!(
        counter
            .structure_shared(CONTROL, &mut budget)
            .expect("a lookup answers")
            .is_none(),
        "a store with no capacity answered from memory"
    );
    let report = counter.report();
    assert_eq!(
        (
            report.entries,
            report.containers,
            report.stored,
            report.hits,
            report.refused_capacity
        ),
        (0, 0, 0, 0, 1),
        "a zero-capacity store did not behave as a refusing counter: {report:?}"
    );

    // ... and a store that does retain is unaffected by that refusal: holds are per request and per
    // store, and one store's refusal is not another's invalidation.
    let retaining = store(4);
    let mut retaining_budget = through(&retaining);
    assert_eq!(
        class_facts(CONTROL, &mut retaining_budget).expect("the fixture parses"),
        expected
    );
    let held = retaining
        .structure_shared(CONTROL, &mut retaining_budget)
        .expect("a lookup answers")
        .expect("the read stored the facts");
    assert_eq!(*held, expected);
    assert_eq!(retaining.report().refused_capacity, 0);
}

/// One key written again holds another payload, and a handle a consumer holds keeps its content.
///
/// The store has one slot per key, and the entry under a key can be written again when a read that
/// could not answer it writes its own: a store read under another declaration discards this entry —
/// its identity rule, held by `tests/p5_facts_cache.rs` — and the same key carries another payload
/// afterwards. What a consumer already holds is not that slot, and the slot moving must never change
/// it.
#[test]
fn a_key_written_again_holds_another_payload_and_a_held_handle_keeps_its_content() {
    let expected = direct(CONTROL);
    let store = store(4);
    let mut budget = through(&store);
    assert_eq!(
        class_facts(CONTROL, &mut budget).expect("the fixture parses"),
        expected
    );
    let held = store
        .structure_shared(CONTROL, &mut budget)
        .expect("a lookup answers")
        .expect("the read stored the facts");

    // A store read under another declaration cannot answer this entry, so it writes its own under
    // the same key.
    let identity = store.identity();
    let foreign = store.over(FactsIdentity::new(
        identity.registry.wrapping_add(1),
        identity.format,
    ));
    let mut foreign_budget = Budget::new(limits()).with_facts_cache(foreign);
    assert_eq!(
        class_facts(CONTROL, &mut foreign_budget).expect("the fixture parses"),
        expected
    );
    let report = store.report();
    assert_eq!(report.discarded_registry, 1, "{report:?}");
    assert_eq!(
        report.entries, 1,
        "the same key was not written again, so this case proves nothing: {report:?}"
    );
    assert_eq!(
        *held, expected,
        "writing the key again changed the payload a consumer already holds"
    );

    // The current declaration is not answered with the other declaration's entry either, and the
    // read that follows writes a third payload: the slot and the handle are different allocations
    // from the first lookup on.
    assert!(
        store
            .structure_shared(CONTROL, &mut budget)
            .expect("a lookup answers")
            .is_none(),
        "one declaration was answered with another declaration's entry"
    );
    assert_eq!(
        class_facts(CONTROL, &mut budget).expect("the fixture parses"),
        expected
    );
    let rebuilt = store
        .structure_shared(CONTROL, &mut budget)
        .expect("a lookup answers")
        .expect("the read stored the facts");
    assert!(
        !Arc::ptr_eq(&held, &rebuilt),
        "the payload a consumer holds is the store's slot, so a later write could change it"
    );
    assert_eq!(*held, expected);
    assert_eq!(
        *rebuilt, expected,
        "the rebuilt entry answers another value"
    );
}

/// A trusted content digest is the same key as the bytes it was computed from, for both products.
#[test]
fn a_trusted_digest_answers_the_entry_the_bytes_wrote() {
    let expected = direct(CONTROL);
    let store = store(8);
    let mut cold = through(&store);
    assert_eq!(
        class_facts(CONTROL, &mut cold).expect("the fixture parses"),
        expected
    );
    let mut header_budget = through(&store);
    inspect_header(CONTROL, &mut header_budget, InspectionMode::Forensic)
        .expect("the fixture's header reads");

    let id = content_id(CONTROL);
    let mut consumer = through(&store);
    let by_bytes = store
        .structure_shared(CONTROL, &mut consumer)
        .expect("a lookup answers")
        .expect("the entry is there");
    let by_digest = store
        .trusted_structure(&id, &mut consumer)
        .expect("a trusted lookup answers")
        .expect("a trusted digest has to answer the entry the bytes wrote");
    assert!(
        Arc::ptr_eq(&by_bytes, &by_digest),
        "a trusted digest and the bytes are two keys for one content"
    );
    assert_eq!(*by_digest, expected);

    // A digest or a length that does not belong to the bytes is another key: it finds nothing, and
    // never answers another class's facts.
    assert!(
        store
            .trusted_structure(&content_id(OTHER), &mut consumer)
            .expect("a trusted lookup answers")
            .is_none(),
        "another class's digest was answered with this class's facts"
    );
    let longer = ClassBytesId {
        digest: id.digest.clone(),
        length: id.length + 1,
    };
    assert!(
        store
            .trusted_structure(&longer, &mut consumer)
            .expect("a trusted lookup answers")
            .is_none(),
        "a length that does not belong to the content was answered anyway"
    );

    // The header product is keyed the same way, and the parse policy stays part of the key.
    let shared_header = store
        .header_shared(CONTROL, InspectionMode::Forensic, &mut consumer)
        .expect("a lookup answers")
        .expect("the header read stored the facts");
    let trusted_header = store
        .trusted_header(&id, InspectionMode::Forensic, &mut consumer)
        .expect("a trusted lookup answers")
        .expect("the trusted digest has to answer the header entry too");
    assert!(Arc::ptr_eq(&shared_header, &trusted_header));
    assert!(
        store
            .trusted_header(&id, InspectionMode::Strict, &mut consumer)
            .expect("a trusted lookup answers")
            .is_none(),
        "a strict request was answered with a forensic read"
    );
    assert_eq!(
        consumer.usage().class_bytes,
        0,
        "a lookup that was answered from the store parsed class bytes"
    );
}
