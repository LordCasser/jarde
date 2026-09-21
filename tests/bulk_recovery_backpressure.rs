//! Bulk task 4.3 through the public entry: ordered delivery under a bounded window.
//!
//! What this file pins:
//!
//! * a **slow first class** does not stop the later classes from being dispatched, and it is never
//!   overtaken: the delivery order stays the physical order whatever the sink's speed is, and the
//!   run finishes instead of deadlocking on a full window;
//! * the **buffer stays bounded**: a window that holds two results runs two class tasks, and the
//!   retained weight never exceeds the window's own ceiling even when the sink is slow enough that
//!   every producer has to wait for its own slot;
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
use std::sync::atomic::{AtomicUsize, Ordering};

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
        ],
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
