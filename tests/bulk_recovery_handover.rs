//! Bulk task 3.4 through the public entry: the container a class candidate came from, handed over
//! with the task that reads it.
//!
//! The reader hands a walk's container to its caller as an **active handle**
//! ([`jarde::ArtifactSnapshot::scope_cursor`]'s `container_facts` and the read
//! [`jarde::ArtifactSnapshot::prepared_class`] hands back), and the weak index of one snapshot makes
//! that product answerable to every other read of the same origin while a handle holds it. What the
//! bulk operation has to do with it is carry it: the walk opens one container at a time and leaves it
//! as soon as its entries are dispatched, so with more than one worker the ordinary case is a class
//! task that reads its class *after* the walk moved on. Without the handover that task finds nothing
//! holding the container and parses the directory again — a container with consumers would be rebuilt,
//! and the number of parses would grow with the number of classes instead of with the number of
//! containers the scope holds.
//!
//! What this file pins, and how:
//!
//! * **A container a class task is still using is parsed once.** The [`handover_fixture`] holds two
//!   nested containers and no class at the root's own level, every class task is held by the
//!   `class_delay` fault before it reads (so the traversal really runs ahead of the reads), and the
//!   count that carries the claim is `archive_entries` — one charge per entry a directory parse
//!   examines — beside the store's own `directory_parses` counter. Three store configurations (none
//!   attached, zero capacity, room for everything) and two windows (fewer class tasks than the scope
//!   holds classes, and more, which walks the scope to its end before any read starts) are run: the
//!   account is the containers' own entry count in all six, because the reuse follows the handle and
//!   not the store's admission decision.
//! * **Holding one is not retaining one.** When the run is over, nothing holds the container any
//!   more: a read *after* the run parses the directory again (the reuse is a lifetime, not a hidden
//!   store), and the same read through the store the run used is answered from retention — which is
//!   the store's own decision about a container no consumer is using, and only that.
//!
//! This target needs the `test-support` feature, because the hold it injects is a fault the request
//! carries and is compiled out of a production build (the root manifest says so beside the feature).
//! The delay changes nothing but *when* a class task reads: no dispatch, window, delivery or
//! cancellation semantics depend on it, and the counts asserted here hold whatever the interleaving.

mod bulk_support;

use bulk_support::{
    HANDOVER_PREFIXES, Recorder, container_roots, environment, handover_fixture, open, request,
    tree_scope,
};
use jarde::bulk::BulkFaults;
use jarde::*;
use std::time::Duration;

/// The containers the fixture's scope holds: the root container, the one that carries the classes
/// and the one that carries resources.
const CONTAINERS: u64 = 3;

/// The entries of those three containers, which is what one parse of each of them costs: two in the
/// root container (`lib/classes.jar`, `res/assets.jar`), four in the class container and two in the
/// asset container.
const CONTAINER_ENTRIES: u64 = 8;

/// The classes the scope holds: four, all of them in one nested container.
const CLASSES: u64 = 4;

/// What one rebuild of the class container's directory costs: its own four entries and the two of
/// the root container that reaches it.
const NESTED_REBUILD_ENTRIES: u64 = 6;

/// How long every class task is held before it reads its class.
///
/// The hold is what makes the window a window: the coordinator dispatches a whole window's worth of
/// classes — and, with more workers than the scope holds classes, walks the scope to its end — while
/// every worker is still inside this wait, so the read below really meets a walk that has moved on.
/// It is orders of magnitude longer than the discovery it lets run ahead, and it is the `class_delay`
/// fault the request carries: no production path waits.
const CLASS_DELAY: Duration = Duration::from_millis(150);

/// One run of the handover fixture: the snapshot it read, the report it returned, the sink that
/// recorded its stream, and the store it read through when it had one.
struct Run {
    snapshot: ArtifactSnapshot,
    report: BulkRecoveryReport,
    sink: Recorder,
    store: Option<FactsCache>,
}

impl Run {
    /// The first prepared class, as the stream published it.
    ///
    /// Every class of the fixture lives in the nested container that carries them, so this is the
    /// entry the release test re-reads: the one whose directory a fresh read has to reach through the
    /// root container that holds the archive.
    fn nested_entry(&self) -> &PhysicalEntryId {
        self.sink
            .prepared()
            .iter()
            .filter_map(|prepared| prepared.location.entry())
            .find(|entry| entry.raw_name.0.starts_with(b"p/"))
            .expect("the fixture's class container declares its classes under its prefix")
    }
}

/// The containers of `snapshot`'s tree and the entries they hold, as an enumeration reports them.
fn shape(snapshot: &ArtifactSnapshot) -> (u64, u64) {
    let mut setup = Budget::new(bulk_support::limits());
    let report = snapshot
        .enumerate_artifact_tree(&mut setup)
        .expect("the fixture's artifact tree is readable");
    let containers = u64::try_from(report.containers.len()).expect("the fixture is small");
    let entries = report.containers.iter().fold(0_u64, |total, container| {
        total.saturating_add(u64::try_from(container.entries.len()).unwrap_or(u64::MAX))
    });
    (containers, entries)
}

/// Runs the fixture's scope with `workers` class tasks, every one of them held for [`CLASS_DELAY`]
/// before it reads its class, under the facts-store configuration `store`.
fn run(workers: usize, store: Option<FactsCache>) -> Run {
    let (snapshot, _opened) = open(handover_fixture());
    let content = vec![snapshot.clone()];
    let (containers, entries) = shape(&snapshot);
    assert_eq!(
        (containers, entries),
        (CONTAINERS, CONTAINER_ENTRIES),
        "the fixture is the shape this file states: two nested containers of two classes each, \
         hanging from a root container that holds only the two archives"
    );
    let mut setup = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut setup, &HANDOVER_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let mut request = request(environment, workers);
    request.faults = BulkFaults {
        class_delay: Some(CLASS_DELAY),
        ..BulkFaults::default()
    };
    let mut budget = Budget::new(bulk_support::limits());
    if let Some(store) = store.as_ref() {
        budget = budget.with_facts_cache(store.clone());
    }
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");
    Run {
        snapshot,
        report,
        sink,
        store,
    }
}

/// The same run every configuration must agree on: the whole scope, recovered, through `workers`
/// class tasks' window.
fn recovered(run: &Run, configuration: &str, workers: u64) {
    assert_eq!(
        run.report.summary.status(),
        "complete",
        "{configuration}: the operation really ran: {:?}",
        run.report.summary
    );
    assert_eq!(run.report.summary.classes_seen, CLASSES, "{configuration}");
    assert_eq!(
        run.report.summary.classes_prepared, CLASSES,
        "{configuration}"
    );
    assert_eq!(run.report.summary.classes_refused, 0, "{configuration}");
    assert_eq!(
        run.report.summary.methods_delivered, run.report.summary.methods_declared,
        "{configuration}: every declared member of the scope is delivered"
    );
    // What the run's own window states: it ran the class tasks the request asked for, and it held one
    // slot per class it dispatched — the slots whose handover the counts below are about. (Whether two
    // class tasks really run at the same time is the lifecycle target's claim, and a task held at its
    // head is not doing work while it waits.)
    assert_eq!(
        run.report.window.active_classes, workers,
        "{configuration}: the window ran the class tasks the request asked for: {:?}",
        run.report.window
    );
    assert_eq!(
        run.report.window.window_slots_high_water,
        workers.min(CLASSES),
        "{configuration}: and it held one slot per dispatched class: {:?}",
        run.report.window
    );
}

#[test]
fn a_container_a_class_task_is_still_using_is_never_parsed_again() {
    // Six runs: two windows — three class tasks, fewer than the four classes the scope holds, so the
    // walk leaves the container the classes came from while classes dispatched from it are still
    // waiting to be read; and eight, more than the scope holds, which walks the scope to its end
    // before the first read even starts — under three store configurations: none attached, one that
    // retains nothing, and one with room for everything.
    //
    // The claim is the same in all six: the directory of each of the three containers is parsed
    // exactly once, by the discovery that opened it, and every class task that reads long after the
    // walk moved on is answered from the product its own slot handed over — however little the
    // caller's store retains. The account does not move with the store's admission decision, which is
    // the point: a container with consumers is not the store's business.
    let zero = FactsCapacity::new(0, 0);
    let roomy = FactsCapacity::new(1 << 12, 1 << 26);
    for workers in [3_usize, 8] {
        for configuration in ["no store attached", "zero capacity", "a store with room"] {
            let store = match configuration {
                "no store attached" => None,
                "zero capacity" => Some(FactsCache::current(zero)),
                _ => Some(FactsCache::current(roomy)),
            };
            let run = run(workers, store);
            let case = format!("{configuration}, {workers} worker(s)");
            recovered(
                &run,
                &case,
                u64::try_from(workers).expect("the fixture's worker count fits"),
            );
            assert_eq!(
                run.report.usage.archive_entries, CONTAINER_ENTRIES,
                "{case}: a directory is parsed for the container, not for each class read out of \
                 it: {:?}",
                run.report.usage
            );
            assert_eq!(
                run.report.discovery_usage.archive_entries, CONTAINER_ENTRIES,
                "{case}: and it is the discovery that pays for the containers' own entries — the \
                 class reads add no parse of their own: {:?}",
                run.report.discovery_usage
            );
            assert_eq!(
                run.report.method_usage.archive_entries, 0,
                "{case}: every member's loader binding query is answered from the container its own \
                 class task holds, so a method costs no directory either: {:?}",
                run.report.method_usage
            );
            match run.store.as_ref().map(FactsCache::report) {
                None => {}
                Some(facts) => {
                    assert_eq!(
                        facts.directory_parses, CONTAINERS,
                        "{case}: every container of the scope is parsed once and no container is \
                         parsed twice: {facts:?}"
                    );
                    if facts.capacity == zero {
                        // The store that retains nothing still counts what it was asked and what it
                        // refused — and it is the handles above, not this store, that the reads were
                        // answered from.
                        assert_eq!(facts.entries, 0, "{case}: {facts:?}");
                        assert_eq!(facts.containers, 0, "{case}: {facts:?}");
                        assert!(facts.answered_nothing(), "{case}: {facts:?}");
                        assert!(
                            facts.refused_capacity + facts.refused_capacity_bytes > 0,
                            "{case}: the container products were offered and refused: {facts:?}"
                        );
                    } else {
                        assert!(
                            facts.container_hits > 0 && facts.container_stored > 0,
                            "{case}: the store with room really kept and answered container facts: \
                             {facts:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn a_run_holds_a_container_only_until_the_class_task_reading_it_ends() {
    // The reuse is a lifetime and nothing else. Two runs of the same scope differ in one thing — the
    // store they read through — and that difference is exactly what outlives them.
    //
    // The run that had **no store** leaves nothing behind: a read *after* it parses the class
    // container's directory again, because at that point nothing is using it. That is the release: a
    // handle ends with the class task that held it, and no product of a finished run survives it.
    // The run whose store **kept** the container is answered from what it holds instead, which is the
    // store's own decision about a container no consumer is using — the other half of the split, and
    // the only half a retention capacity may decide.
    let without = run(3, None);
    recovered(&without, "no store attached", 3);
    assert_eq!(
        without.report.usage.archive_entries, CONTAINER_ENTRIES,
        "the run itself parsed each container once: {:?}",
        without.report.usage
    );
    let entry = without.nested_entry().clone();

    // (1) No store and no run: the container the entry lives in is reached from the root container
    //     that holds its archive, so the directory is parsed again — the four entries of the class
    //     container and the two of the root one.
    let mut fresh = Budget::new(bulk_support::limits());
    without
        .snapshot
        .prepared_class(&entry, &mut fresh)
        .expect("the fixture's class is readable again");
    assert_eq!(
        fresh.usage().archive_entries,
        NESTED_REBUILD_ENTRIES,
        "the run's handles are released with the class tasks that held them, so a later read parses \
         the directory again instead of finding a product nothing holds: {:?}",
        fresh.usage()
    );

    // (2) The same read, after a run that read through a store with room: retention kept the
    //     container, so the read is answered from it and no directory is parsed at all.
    let roomy = FactsCache::current(FactsCapacity::new(1 << 12, 1 << 26));
    let retaining = run(3, Some(roomy.clone()));
    recovered(&retaining, "the store the run read through", 3);
    assert_eq!(
        retaining.report.usage.archive_entries, CONTAINER_ENTRIES,
        "and that run parsed each container once too: {:?}",
        retaining.report.usage
    );
    let entry = retaining.nested_entry().clone();
    let before = roomy.report();
    let mut retained = Budget::new(bulk_support::limits()).with_facts_cache(roomy.clone());
    retaining
        .snapshot
        .prepared_class(&entry, &mut retained)
        .expect("the fixture's class is readable through the store");
    assert_eq!(
        retained.usage().archive_entries,
        0,
        "a store that kept the container answers the read without parsing it: {:?}",
        retained.usage()
    );
    let after = roomy.report();
    assert!(
        after.container_hits > before.container_hits,
        "and the answer really came from retention: {} -> {} hits ({after:?})",
        before.container_hits,
        after.container_hits
    );
    assert!(
        after.containers > 0,
        "the store still holds the container it kept: {after:?}"
    );
}
