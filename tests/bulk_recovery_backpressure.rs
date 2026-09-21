//! Bulk tasks 4.3 and 4.7 through the public entry: ordered delivery under a bounded window.
//!
//! What this file pins:
//!
//! * a **slow first class** does not stop the later classes from being dispatched, and it is never
//!   overtaken: the delivery order stays the physical order whatever the sink's speed is, and the
//!   run finishes instead of deadlocking on a full window;
//! * the **buffer stays bounded**: a window that holds two results runs two class tasks, and the
//!   retained weight never exceeds the window's own ceiling even when the sink is slow enough that
//!   every producer has to wait for its own slot;
//! * the window is **two levels** (task 4.7, design decision 5 as amended): every active class holds a
//!   **reservation** of one `max_result_weight` — the first result it hands over never waits — and the
//!   rest of the declared window is a **shared pool** whose capacity is derived from the declared
//!   window and the effective worker count alone, published in every header and report, and really
//!   used while the coordinator is busy with the earliest class. A pool of zero capacity is exactly
//!   the one-result-per-class window it replaced, which is what the case below states: no record is
//!   ever placed beyond its class's own reservation, and the run still finishes in order;
//! * a **single result larger than the fixed per-result ceiling** stops that method locally, with the
//!   weight and the ceiling in its record, its text discarded rather than truncated, and the other
//!   methods beside it still delivered;
//! * adding workers **never shrinks** the per-result ceiling: the same oversized result is refused
//!   with the same ceiling at one worker and at four.

mod bulk_support;

use bulk_support::{
    FLAT_PREFIXES, Recorded, Recorder, container_roots, environment, flat_fixture, nested_fixture,
    open, request, tree_scope,
};
use jarde::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

/// How long a case here lets a run take before its watchdog cancels it.
///
/// The property these cases state — the classes behind the earliest one keep making progress — is the
/// one property a test cannot bound from inside the run: a window that starved the earliest class
/// waits for room another class holds, so the call itself never returns and no assertion after it is
/// ever reached. Cancelling from beside the call turns that starvation into a report a case can read
/// (a status that is not `complete`) instead of a hung test, and it is what a caller does when an
/// operation it asked for stops making progress. The bound is two orders of magnitude above the
/// fixture's own work — its sink sleeps are tens of milliseconds — so a run that is not starved never
/// meets it.
const STARVATION_BOUND: Duration = Duration::from_secs(10);

/// Runs one recovery with a watchdog that cancels it after [`STARVATION_BOUND`].
fn recover_under_bound(
    content: &[ArtifactSnapshot],
    request: &BulkRecoveryRequest,
    budget: &mut Budget,
    sink: &mut Recorder,
) -> BulkRecoveryReport {
    let token = budget.cancellation_token();
    let ended = Arc::new(AtomicBool::new(false));
    let watcher_ended = Arc::clone(&ended);
    let watchdog = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + STARVATION_BOUND;
        while std::time::Instant::now() < deadline {
            if watcher_ended.load(Ordering::SeqCst) {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        token.cancel();
    });
    let report = Engine::new()
        .recover_all(content, request, budget, sink)
        .expect("the fixture's scope is recoverable");
    ended.store(true, Ordering::SeqCst);
    let _ = watchdog.join();
    report
}

/// One run of the nested fixture whose sink is slow on the first class's records, and how many of
/// them it really slept over.
///
/// The first class is the one the coordinator must deliver first, whatever the classes behind it are
/// doing, so a sink that takes its time there is what holds the window where the question is: can the
/// classes behind the earliest one keep producing results while it is being delivered?
///
/// `ceiling` is the fixed per-result ceiling of the run and `window_results` the declared window in
/// whole ceilings: the workers' own reservations and the shared pool together, which is the
/// declaration the pool's capacity is derived from.
fn run_behind_a_slow_first_class(
    workers: usize,
    ceiling: u64,
    window_results: u64,
    sleep: Duration,
) -> (BulkRecoveryReport, Recorder, usize) {
    let (snapshot, _opened) = open(nested_fixture());
    let content = vec![snapshot.clone()];
    let mut setup = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut setup, &bulk_support::NESTED_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment, workers).with_capacities(
        jarde::DEFAULT_MAX_CLASS_BYTES,
        ceiling,
        window_results * ceiling,
    );
    let slept = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&slept);
    let mut sink = Recorder::new();
    sink.hook = Some(Box::new(move |event: &Recorded| {
        if let Recorded::Method(method) = event
            && method.class_ordinal == 0
        {
            std::thread::sleep(sleep);
            counter.fetch_add(1, Ordering::SeqCst);
        }
    }));
    let mut budget = Budget::new(bulk_support::limits());
    let report = recover_under_bound(&content, &request, &mut budget, &mut sink);
    let slept = slept.load(Ordering::SeqCst);
    (report, sink, slept)
}

/// The delivery order of the nested fixture's eighteen members, class by class and in declaration
/// order: the order every run of this scope must publish, whatever the window did.
fn nested_order() -> Vec<(u64, u64)> {
    vec![
        (0, 0),
        (0, 1),
        (0, 2),
        (0, 3),
        (0, 4),
        (0, 5),
        (0, 6),
        (0, 7),
        (1, 0),
        (1, 1),
        (1, 2),
        (2, 0),
        (2, 1),
        (2, 2),
        (3, 0),
        (3, 1),
        (3, 2),
        (3, 3),
    ]
}

/// The members a recorded sink was handed, as (class ordinal, member ordinal) pairs.
fn recorded_order(sink: &Recorder) -> Vec<(u64, u64)> {
    sink.methods()
        .iter()
        .map(|method| (method.class_ordinal, method.member_ordinal))
        .collect()
}

/// One run of the nested fixture, with the sink's own timing.
fn run_slow(
    workers: usize,
    delay: std::time::Duration,
    hook: Option<bulk_support::SinkHook>,
) -> (BulkRecoveryReport, Recorder) {
    let (snapshot, _opened) = open(nested_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, &bulk_support::NESTED_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment, workers).with_capacities(
        jarde::DEFAULT_MAX_CLASS_BYTES,
        jarde::DEFAULT_MAX_RESULT_WEIGHT,
        2 * jarde::DEFAULT_MAX_RESULT_WEIGHT,
    );
    let mut sink = Recorder::new();
    sink.delay = delay;
    sink.hook = hook;
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");
    (report, sink)
}

#[test]
fn a_slow_first_class_does_not_starve_the_later_ones_or_the_order() {
    // The sink slows down exactly the first class's records, so the first class holds the window for
    // a while while the classes behind it produce their results and wait.
    let watched = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&watched);
    let hook = Box::new(move |event: &Recorded| {
        if let Recorded::Method(method) = event
            && method.class_ordinal == 0
            && method.member_ordinal < 3
        {
            std::thread::sleep(std::time::Duration::from_millis(30));
            counter.fetch_add(1, Ordering::SeqCst);
        }
    }) as bulk_support::SinkHook;
    let (report, sink) = run_slow(2, std::time::Duration::ZERO, Some(hook));

    assert_eq!(report.summary.status(), "complete");
    assert_eq!(
        watched.load(Ordering::SeqCst),
        3,
        "the slow first class really was delivered through the sink"
    );
    assert_eq!(report.summary.methods_delivered, 18);
    assert_eq!(
        report.window.active_classes, 2,
        "the window holds two results, so two class tasks run"
    );
    assert!(
        report.window.concurrent_classes_high_water <= 2,
        "never more class tasks than the window: {:?}",
        report.window
    );
    assert!(
        report.window.buffered_weight_high_water <= report.window.buffered_weight_limit,
        "the retained results stay inside the window's ceiling: {:?}",
        report.window
    );
    let ordinals: Vec<(u64, u64)> = sink
        .methods()
        .iter()
        .map(|method| (method.class_ordinal, method.member_ordinal))
        .collect();
    assert_eq!(
        ordinals,
        nested_order(),
        "the delivery order is the physical and declaration order, never the finish order"
    );
    let expected: Vec<Vec<u8>> = vec![
        b"Scope.class".to_vec(),
        b"Shape.class".to_vec(),
        b"LambdaSample.class".to_vec(),
        b"d/Holder.class".to_vec(),
    ];
    assert_eq!(
        sink.prepared()
            .iter()
            .map(|prepared| prepared.location.entry().unwrap().raw_name.0.clone())
            .collect::<Vec<_>>(),
        expected,
        "and the classes are the ones the traversal ordered, the nested one last"
    );
}

/// The two-level window (task 4.7), at the declaration where it collapses to one result per class.
///
/// The window is two whole per-result ceilings for two effective workers, so there is nothing left
/// over for a pool: the reservation of each active class *is* the whole window it is promised. The
/// case states three things about a run whose first class is delivered slowly while the classes
/// behind it work: the declared pool is zero, **no result was ever placed beyond its class's own
/// reservation** — which is the one-result-per-class window this replaced, and the reason the pool
/// capacity is published rather than assumed — and the run still finishes, in the physical order and
/// inside the window's ceiling. The watchdog is what makes "the earliest class is never starved" a
/// bounded claim here: a window that starved it would never return at all.
#[test]
fn a_window_that_only_funds_the_reservations_is_one_result_per_class() {
    let (report, sink, slow) = run_behind_a_slow_first_class(
        2,
        jarde::DEFAULT_MAX_RESULT_WEIGHT,
        2,
        Duration::from_millis(20),
    );
    assert_eq!(
        slow, 8,
        "the sink really took every record of the first class"
    );
    assert_eq!(
        report.summary.status(),
        "complete",
        "the first class was delivered without starving anything: {:?}",
        report.summary
    );
    assert!(report.final_delivered, "the stream reached its end");
    assert_eq!(report.summary.methods_delivered, 18);
    let limits = &report.summary.limits;
    assert_eq!(
        limits.workers_effective, 2,
        "the window holds two results, so two class tasks run: {limits:?}"
    );
    assert_eq!(
        limits.shared_result_pool_weight, 0,
        "the whole declared window is the two classes' reservations: {limits:?}"
    );
    assert_eq!(
        limits.shared_result_pool_weight
            + limits.workers_effective as u64 * limits.max_result_weight,
        limits.max_buffered_result_weight,
        "reservations and pool account for every byte of the declared window: {limits:?}"
    );
    assert_eq!(
        report.window.shared_pool_weight_high_water, 0,
        "no result was placed beyond its own class's reservation, so the window really held at most \
         one pending result per class: {:?}",
        report.window
    );
    assert!(
        report.window.buffered_weight_high_water <= report.window.buffered_weight_limit,
        "the retained results stay inside the window's ceiling: {:?}",
        report.window
    );
    assert_eq!(
        recorded_order(&sink),
        nested_order(),
        "the delivery order is the physical and declaration order, never the finish order"
    );
}

/// The two-level window at a declaration that funds a pool, with the classes behind the front busy.
///
/// Four workers and six whole per-result ceilings: four reservations and two further results' worth of
/// shared pool. The first class is again the slow one, so the classes behind it produce their results
/// while the coordinator is inside the earliest class's records — and those results cannot be
/// delivered, because delivery is ordered, so they can only wait in the window. The case states that
/// they really did: a pool high-water above zero is the reading that says "the classes behind the
/// front worked ahead", where the one-result-per-class window would have made them wait for their own
/// turn. Order, boundedness and completion are the same claims as above, and the proof that the pool
/// is what bought the extra work is the case beside this one, where a pool of zero capacity carries
/// nothing.
#[test]
fn a_shared_pool_lets_the_classes_behind_the_front_work_ahead() {
    let (report, sink, slow) = run_behind_a_slow_first_class(
        4,
        jarde::DEFAULT_MAX_RESULT_WEIGHT,
        6,
        Duration::from_millis(20),
    );
    assert_eq!(
        slow, 8,
        "the sink really took every record of the first class"
    );
    assert_eq!(
        report.summary.status(),
        "complete",
        "the pool neither starved the front class nor broke the stream: {:?}",
        report.summary
    );
    assert!(report.final_delivered, "the stream reached its end");
    assert_eq!(report.summary.methods_delivered, 18);
    let limits = &report.summary.limits;
    assert_eq!(limits.workers_effective, 4, "{limits:?}");
    assert_eq!(
        limits.shared_result_pool_weight,
        2 * jarde::DEFAULT_MAX_RESULT_WEIGHT,
        "the declared window outside the four reservations is the pool: {limits:?}"
    );
    assert!(
        report.window.shared_pool_weight_high_water > 0,
        "the classes behind the front really placed results in the pool while the earliest class was \
         being delivered: {:?}",
        report.window
    );
    assert!(
        report.window.shared_pool_weight_high_water <= limits.shared_result_pool_weight,
        "and the pool stayed inside its own capacity: {:?} against {limits:?}",
        report.window
    );
    assert!(
        report.window.buffered_weight_high_water <= report.window.buffered_weight_limit,
        "the retained results stay inside the window's ceiling: {:?}",
        report.window
    );
    assert_eq!(
        recorded_order(&sink),
        nested_order(),
        "what the pool changed is how much could be produced ahead, never who is delivered first"
    );
}

/// The same claim at a pool that is *tight*: one result's worth, measured against this fixture.
///
/// The declarations above fund a pool that is enormous beside this fixture's results, so the case
/// beside this one cannot show that a class's own seat is what carries it past a pool its neighbours
/// have filled. The per-result ceiling here is the **largest result this scope really produces**
/// (learned from a first run, the way the oversized case below learns its ceiling, so no result is
/// refused), a window of five ceilings is four reservations and one result's worth of pool, and the
/// classes behind the front fill that pool with their own results at once: ordered delivery means
/// their results cannot leave the window until the earliest class is done. The earliest class still
/// finishes, in order and inside the window's ceiling — the room it needs is its own reservation, and
/// no neighbour can hold it.
#[test]
fn a_pool_of_one_result_still_cannot_starve_the_earliest_class() {
    let (learned, learned_sink) = run_slow(1, std::time::Duration::ZERO, None);
    assert_eq!(learned.summary.status(), "complete");
    let ceiling = learned_sink
        .methods()
        .iter()
        .map(|method| method.weight)
        .max()
        .expect("the fixture declares method records to learn a ceiling from");
    assert!(ceiling > 0, "the fixture's results really retain something");

    let (report, sink, slow) =
        run_behind_a_slow_first_class(4, ceiling, 5, Duration::from_millis(20));
    assert_eq!(
        slow, 8,
        "the sink really took every record of the first class"
    );
    assert_eq!(
        report.summary.status(),
        "complete",
        "one result's worth of pool is not a pool the earliest class can be starved by: {:?}",
        report.summary
    );
    assert!(report.final_delivered, "the stream reached its end");
    assert_eq!(report.summary.methods_delivered, 18);
    let limits = &report.summary.limits;
    assert_eq!(limits.max_result_weight, ceiling, "{limits:?}");
    assert_eq!(limits.workers_effective, 4, "{limits:?}");
    assert_eq!(
        limits.shared_result_pool_weight, ceiling,
        "four reservations of the ceiling, and one more ceiling as the pool: {limits:?}"
    );
    assert!(
        report.window.shared_pool_weight_high_water > 0,
        "the classes behind the front really filled the pool while the earliest one was delivered: \
         {:?}",
        report.window
    );
    assert!(
        report.window.shared_pool_weight_high_water <= limits.shared_result_pool_weight,
        "and the pool stayed inside its own capacity: {:?} against {limits:?}",
        report.window
    );
    assert!(
        report.window.buffered_weight_high_water <= report.window.buffered_weight_limit,
        "the retained results stay inside the window's ceiling: {:?}",
        report.window
    );
    assert_eq!(
        recorded_order(&sink),
        nested_order(),
        "the tight pool did not reorder anything either"
    );
}

/// The pool's capacity is a consequence of the declaration, for every declaration.
///
/// The window is a per-result ceiling, a worker count and a whole-window ceiling, and nothing else:
/// the reservations are `workers_effective * max_result_weight` of it, the pool is what is left, and
/// the effective worker count is the window's own reduction of the requested one. This case states
/// that identity across declarations — a requested count below the window, above it, and a window of
/// exactly one result — and reads it back from the header as well as from the report, because the
/// configuration is published to a consumer that only ever sees the stream. The per-result ceiling is
/// the same in every row: no worker count moves it.
#[test]
fn the_shared_pool_is_the_declared_window_outside_the_reservations() {
    let ceiling = jarde::DEFAULT_MAX_RESULT_WEIGHT;
    for (workers, window_results) in [
        (1_usize, 16_u64),
        (2, 16),
        (4, 6),
        (8, 3),
        (4, 3),
        (2, 2),
        (3, 1),
    ] {
        let (snapshot, _opened) = open(flat_fixture());
        let content = vec![snapshot.clone()];
        let mut setup = Budget::new(bulk_support::limits());
        let roots = container_roots(&snapshot, &mut setup, &FLAT_PREFIXES);
        let environment = environment(&snapshot, tree_scope(), roots);
        let declared = window_results * ceiling;
        let request = request(environment, workers).with_capacities(
            jarde::DEFAULT_MAX_CLASS_BYTES,
            ceiling,
            declared,
        );
        let mut budget = Budget::new(bulk_support::limits());
        let mut sink = Recorder::new();
        let report = Engine::new()
            .recover_all(&content, &request, &mut budget, &mut sink)
            .expect("the fixture's scope is recoverable");
        let limits = &report.summary.limits;
        let effective = workers.min(usize::try_from(window_results).unwrap()).max(1);
        let reservations = u64::try_from(effective).unwrap() * ceiling;
        assert_eq!(limits.workers_requested, workers, "{limits:?}");
        assert_eq!(
            limits.workers_effective, effective,
            "the window's own reduction of the requested count, published rather than hidden: \
             {limits:?}"
        );
        assert_eq!(limits.max_result_weight, ceiling, "{limits:?}");
        assert_eq!(limits.max_buffered_result_weight, declared, "{limits:?}");
        assert_eq!(
            limits.shared_result_pool_weight,
            declared - reservations,
            "the pool is the declared window outside the reservations of the effective workers: \
             {limits:?}"
        );
        assert_eq!(
            limits.shared_result_pool_weight + reservations,
            limits.max_buffered_result_weight,
            "so the two levels account for the whole declaration: {limits:?}"
        );
        assert_eq!(
            sink.header()
                .expect("every stream starts with its configuration")
                .limits,
            *limits,
            "and the header a consumer reads states the same configuration the report does"
        );
        assert_eq!(report.summary.status(), "complete", "{:?}", report.summary);
        assert!(
            report.window.shared_pool_weight_high_water <= limits.shared_result_pool_weight,
            "no run ever holds more of the pool than its declaration funds: {:?} against {limits:?}",
            report.window
        );
    }
}

#[test]
fn one_result_larger_than_the_ceiling_stops_that_method_only() {
    // Learn this fixture's own result weights first, with a ceiling nothing can exceed, so the
    // per-result ceiling below can be placed in the middle of the real distribution: some methods
    // fit it and some do not. A ceiling taken from thin air would either refuse everything or
    // nothing, and prove neither half.
    let (learned, sink) = run_slow(1, std::time::Duration::ZERO, None);
    assert_eq!(learned.summary.status(), "complete");
    let mut weights: Vec<u64> = sink.methods().iter().map(|method| method.weight).collect();
    weights.sort_unstable();
    let ceiling = weights[weights.len() / 2];
    assert!(
        ceiling > 0 && *weights.last().unwrap() > ceiling,
        "the fixture has results below and above its median weight: {weights:?}"
    );

    for workers in [1_usize, 4] {
        let (snapshot, _opened) = open(flat_fixture());
        let content = vec![snapshot.clone()];
        let mut budget = Budget::new(bulk_support::limits());
        let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
        let environment = environment(&snapshot, tree_scope(), roots);
        let request = request(environment, workers).with_capacities(
            jarde::DEFAULT_MAX_CLASS_BYTES,
            ceiling,
            2 * ceiling,
        );
        let mut sink = Recorder::new();
        let report = Engine::new()
            .recover_all(&content, &request, &mut budget, &mut sink)
            .expect("the fixture's scope is recoverable");

        let methods = sink.methods();
        let refused: Vec<&bulk_support::MethodRecord> = methods
            .iter()
            .filter(|method| method.outcome == BulkMethodOutcome::Oversized)
            .collect();
        let delivered: Vec<&bulk_support::MethodRecord> =
            methods.iter().filter(|method| method.weight > 0).collect();
        assert!(
            !refused.is_empty() && !delivered.is_empty(),
            "the ceiling {ceiling} splits the methods into refused and delivered ones at {workers} \
             worker(s)"
        );
        for method in &refused {
            assert_eq!(
                method.weight, 0,
                "a refused result retains nothing: its text is discarded rather than truncated"
            );
            assert!(
                method.text.is_none(),
                "and no text is delivered in its place"
            );
            let (weight, limit) = method
                .oversized
                .expect("a refused record states the weight and the ceiling it exceeded");
            assert!(
                weight > limit,
                "the record's reason is locatable: {weight} exceeded {limit}"
            );
            assert_eq!(limit, ceiling);
            assert!(!method.diagnostic_codes.is_empty(), "{method:?}");
        }
        for method in &delivered {
            assert!(
                method.weight <= ceiling,
                "a delivered result stays inside the ceiling"
            );
            assert!(method.text.is_some(), "a delivered result carries its text");
        }
        assert_eq!(
            report.summary.methods_delivered as usize,
            methods.len(),
            "every record the sink confirmed is counted as delivered"
        );
        assert_eq!(report.summary.methods_executed, 18);
        assert_eq!(
            report.summary.status(),
            "partial",
            "a stopped method cannot leave the operation complete"
        );
        assert_eq!(
            report.summary.methods_not_executed, 0,
            "a refused result is not an unexecuted method: it ran and was stopped"
        );
        assert_eq!(
            report.summary.limits.max_result_weight, ceiling,
            "the ceiling is the fixed one, whatever the worker count"
        );
        assert_eq!(
            report.window.largest_result_weight,
            *weights
                .iter()
                .filter(|weight| **weight <= ceiling)
                .max()
                .unwrap(),
            "the largest result the window really retained is the largest one that fit"
        );
        assert!(
            report.window.buffered_weight_high_water <= report.window.buffered_weight_limit,
            "the window stays inside its own ceiling: {:?}",
            report.window
        );
    }
}

#[test]
fn a_result_weighs_what_it_owns_and_not_the_length_it_writes() {
    // The window's ceiling is only as honest as the weight each result is accounted at. This case reads
    // the same public surface the library's model reads, and states three things about one run of the
    // nested fixture:
    //
    // * every delivered result's weight **covers** what that result owns — its text's own buffer, its
    //   source map's segment table and the tables of both reports, each by capacity where the surface
    //   hands over a buffer this process owns;
    // * the result that owns most weighs most (and the one that owns least weighs least): the weight
    //   follows what a result holds rather than a constant;
    // * what the *older* model charged — the text's length plus a fixed cost per retained record — is
    //   below what a result really owns, so a model that charged that proxy cannot cover it.
    let (report, sink) = run_slow(1, std::time::Duration::ZERO, None);
    assert_eq!(report.summary.status(), "complete", "{:?}", report.summary);
    let methods = sink.methods();
    assert_eq!(methods.len(), 18, "the whole scope was delivered");

    for method in &methods {
        assert!(
            method.weight >= method.owned_bytes,
            "a result's weight covers what it owns: {} byte(s) of weight for {} byte(s) of owned \
             capacity, for {}#{}",
            method.weight,
            method.owned_bytes,
            method.class_ordinal,
            method.member_ordinal
        );
    }
    let largest = methods
        .iter()
        .max_by_key(|method| method.owned_bytes)
        .expect("the scope delivered method records");
    let smallest = methods
        .iter()
        .min_by_key(|method| method.owned_bytes)
        .expect("the scope delivered method records");
    assert!(
        largest.owned_bytes > smallest.owned_bytes,
        "the fixture's results really differ in what they own: {} against {}",
        largest.owned_bytes,
        smallest.owned_bytes
    );
    assert!(
        largest.weight > smallest.weight,
        "and the weight follows what they own: {} against {}",
        largest.weight,
        smallest.weight
    );
    assert!(
        largest.owned_bytes > largest.proxy_bytes,
        "the length-plus-constant proxy is below what the largest result owns: {} against {}",
        largest.owned_bytes,
        largest.proxy_bytes
    );
    let weights: Vec<u64> = methods.iter().map(|method| method.weight).collect();
    assert!(
        weights.iter().max() != weights.iter().min(),
        "the weight is a reading of each result, not one constant: {weights:?}"
    );
    assert_eq!(
        report.window.largest_result_weight,
        *weights.iter().max().expect("the scope delivered methods"),
        "the window's own reading is the largest weight it retained: {:?}",
        report.window
    );
    assert!(
        report.window.buffered_weight_high_water <= report.window.buffered_weight_limit,
        "and the high-water mark stays inside the window's ceiling: {:?}",
        report.window
    );
}
