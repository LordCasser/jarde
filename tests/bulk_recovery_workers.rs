//! Bulk tasks 4.1–4.5 through the public entry: the bounded worker window.
//!
//! What this file pins, on one fixture read twice with different worker counts:
//!
//! * **determinism**: a full run with one worker and a full run with four publish the same method
//!   identities in the same order, the same per-method content, text, source map, rules, diagnostics
//!   and semantic coverage (compared as fingerprints with the resource readings removed), and the
//!   same per-outcome counts — the scheduling of two class tasks is not part of the result;
//! * **real overlap**: the class tasks really ran at the same time, which the operation's own
//!   high-water count states and which the reporting sink sees as records of two live classes;
//! * **the window is the effective worker count**: `min(requested, window / per-result ceiling)` is
//!   published in both values, the reduction is never silent, and no more class tasks are active than
//!   the effective count;
//! * **the configuration is checked before anything runs**: a request that is not a configuration at
//!   all is an input error with its own code, never a silent clamp.

mod bulk_support;

use bulk_support::{
    FLAT_PREFIXES, Recorder, container_roots, environment, flat_fixture, open, request, tree_scope,
};
use jarde::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// One run: the report plus everything the sink recorded.
struct Run {
    report: BulkRecoveryReport,
    sink: Recorder,
}

fn run(workers: usize) -> Run {
    run_with(workers, |request| request)
}

fn run_with(workers: usize, adapt: impl FnOnce(BulkRecoveryRequest) -> BulkRecoveryRequest) -> Run {
    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = adapt(request(environment, workers));
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");
    Run { report, sink }
}

#[test]
fn one_worker_and_four_publish_the_same_stream() {
    let serial = run(1);
    let parallel = run(4);

    assert_eq!(serial.report.summary.status(), "complete");
    assert_eq!(parallel.report.summary.status(), "complete");
    // The two summaries differ in exactly one thing — the worker count that produced them — which is
    // the scheduling parameter the change explicitly keeps out of the semantic comparison. Every
    // count, every outcome bucket and the aggregate state are the same.
    assert_eq!(serial.report.summary.limits.workers_requested, 1);
    assert_eq!(parallel.report.summary.limits.workers_requested, 4);
    let mut one = serial.report.summary.clone();
    let mut many = parallel.report.summary.clone();
    one.limits.workers_requested = 0;
    one.limits.workers_effective = 0;
    many.limits.workers_requested = 0;
    many.limits.workers_effective = 0;
    assert_eq!(
        bulk_support::fingerprint(&one),
        bulk_support::fingerprint(&many),
        "the two runs agree on the whole bounded summary, scheduling and resources aside"
    );
    assert_eq!(many.methods_delivered, 18);
    assert!(serial.report.final_delivered && parallel.report.final_delivered);

    // The delivery order and every per-method fingerprint: identical, and identical to the traversal
    // order the two runs walked.
    let serial_methods = serial.sink.methods();
    let parallel_methods = parallel.sink.methods();
    assert_eq!(serial_methods.len(), 18);
    assert_eq!(
        serial_methods
            .iter()
            .map(|method| method.key())
            .collect::<Vec<_>>(),
        parallel_methods
            .iter()
            .map(|method| method.key())
            .collect::<Vec<_>>(),
        "one worker and four publish the same identities in the same order"
    );
    for (one, many) in serial_methods.iter().zip(parallel_methods.iter()) {
        assert_eq!(
            one.fingerprint,
            many.fingerprint,
            "{} `{}` differs between one worker and four",
            String::from_utf8_lossy(&one.method.name.0),
            String::from_utf8_lossy(&one.method.descriptor.0)
        );
        assert_eq!(one.outcome, many.outcome);
        assert_eq!(one.text, many.text);
    }
    let serial_classes: Vec<Vec<u8>> = serial
        .sink
        .prepared()
        .iter()
        .map(|prepared| prepared.location.entry().unwrap().raw_name.0.clone())
        .collect();
    let parallel_classes: Vec<Vec<u8>> = parallel
        .sink
        .prepared()
        .iter()
        .map(|prepared| prepared.location.entry().unwrap().raw_name.0.clone())
        .collect();
    assert_eq!(
        serial_classes,
        vec![
            b"Scope.class".to_vec(),
            b"Shape.class".to_vec(),
            b"LambdaSample.class".to_vec(),
            b"Holder.class".to_vec()
        ],
        "the classes arrive in the container's own order"
    );
    assert_eq!(serial_classes, parallel_classes);
    assert_eq!(
        serial.report.summary.outcomes,
        parallel.report.summary.outcomes
    );

    // Real overlap, as the operation itself observed it: four class tasks were dispatched and at
    // least two of them were inside the class task at the same time.
    assert_eq!(parallel.report.window.active_classes, 4);
    assert!(
        parallel.report.window.concurrent_classes_high_water >= 2,
        "four workers really ran two class tasks at once: {:?}",
        parallel.report.window
    );
    assert!(
        parallel.report.window.concurrent_classes_high_water <= 4,
        "no more class tasks than the declared window: {:?}",
        parallel.report.window
    );
    assert_eq!(
        serial.report.window.concurrent_classes_high_water, 1,
        "one worker is one class task at a time: {:?}",
        serial.report.window
    );
    assert!(
        parallel.report.window.buffered_weight_high_water
            <= parallel.report.window.buffered_weight_limit,
        "the result window stays inside its own ceiling: {:?}",
        parallel.report.window
    );
    assert_eq!(
        parallel.report.summary.methods_delivered, parallel.report.summary.methods_executed,
        "a full run delivers every record it executed"
    );
}

#[test]
fn a_class_task_overlaps_another_while_the_first_one_is_still_working() {
    // The evidence is a rendezvous the class tasks pass through: with two workers and a slow first
    // class, the second class task is inside the class task while the first one is still holding its
    // own slot. A run that executed class tasks one at a time could never reach a peak of two.
    let peak = Arc::new(AtomicUsize::new(0));
    let live = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&live);
    let seen = Arc::clone(&peak);
    let mut sink = Recorder::new();
    sink.hook = Some(Box::new(move |event| {
        if let bulk_support::Recorded::Method(method) = event {
            let live = observed.fetch_add(1, Ordering::SeqCst) + 1;
            seen.fetch_max(live, Ordering::SeqCst);
            if method.member_ordinal == 0 {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            observed.fetch_sub(1, Ordering::SeqCst);
        }
    }));

    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = request(environment, 2);
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");
    assert_eq!(report.summary.status(), "complete");
    assert!(
        report.window.concurrent_classes_high_water >= 2,
        "two class tasks ran at the same time: {:?}",
        report.window
    );
    assert!(
        peak.load(Ordering::SeqCst) == 1,
        "the sink is called by one thread only, so no callback ever overlaps another"
    );
    assert_eq!(report.summary.methods_delivered, 18);
}

#[test]
fn the_window_reduces_the_workers_and_publishes_both_numbers() {
    // A window that holds exactly two results, whatever the caller asks for.
    let run = run_with(6, |request| {
        request.with_capacities(
            jarde::DEFAULT_MAX_CLASS_BYTES,
            jarde::DEFAULT_MAX_RESULT_WEIGHT,
            2 * jarde::DEFAULT_MAX_RESULT_WEIGHT,
        )
    });
    assert_eq!(run.report.summary.limits.workers_requested, 6);
    assert_eq!(
        run.report.summary.limits.workers_effective, 2,
        "the window holds two results, so two class tasks run"
    );
    assert_eq!(
        run.sink.header().unwrap().limits,
        run.report.summary.limits,
        "the reduction is published in the header, not silently applied"
    );
    assert_eq!(run.report.window.active_classes, 2);
    assert!(
        run.report.window.concurrent_classes_high_water <= 2,
        "the reduction really bounds the class tasks: {:?}",
        run.report.window
    );
    assert_eq!(
        run.report.summary.methods_delivered, 18,
        "a smaller window still covers the whole scope"
    );
    assert_eq!(
        run.report.summary.limits.max_result_weight,
        jarde::DEFAULT_MAX_RESULT_WEIGHT,
        "a smaller window never shrinks the per-result ceiling"
    );
}

#[test]
fn the_request_is_checked_before_anything_is_dispatched() {
    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut setup = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut setup, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);

    let cases: [(&str, BulkRecoveryRequest, &str); 4] = [
        (
            "bulk_workers_zero",
            request(environment.clone(), 0),
            "no worker at all is not a configuration",
        ),
        (
            "bulk_max_result_weight_zero",
            request(environment.clone(), 1).with_capacities(1024, 0, 1024),
            "a per-result ceiling of zero refuses every result",
        ),
        (
            "bulk_window_too_small",
            request(environment.clone(), 1).with_capacities(1024, 1024, 512),
            "a window smaller than one result is not a window",
        ),
        (
            "bulk_max_class_bytes_zero",
            request(environment.clone(), 1).with_capacities(0, 1024, 1024),
            "a preparation ceiling of zero refuses every class",
        ),
    ];
    for (code, request, why) in cases {
        let mut budget = Budget::new(bulk_support::limits());
        let mut sink = Recorder::new();
        let error = Engine::new()
            .recover_all(&content, &request, &mut budget, &mut sink)
            .expect_err(why);
        match &error {
            Error::InvalidInput { code: found, .. } => {
                assert_eq!(found, code, "{why}: {error}");
            }
            other => panic!("{why}: expected an input error, got {other}"),
        }
        assert!(
            sink.events.is_empty(),
            "a refused request publishes no event at all: {:?}",
            sink.events.len()
        );
        assert_eq!(
            budget.usage().archive_entries,
            0,
            "and reads nothing: {why}"
        );
    }
}

#[test]
fn the_operations_total_and_a_methods_local_limit_are_two_declarations() {
    // A tight **local** limit stops the method that met it and leaves the rest of the scope alone; the
    // operation's own total stays the one the caller opened it with, and the two are published as
    // themselves — `limits.method` for the local one and `limits.total` for the operation's. A
    // per-method limit inherited from the operation's ceiling would make that impossible to see: the
    // numbers would be one, and a whole-package ceiling would read as a per-method allowance.
    let mut total = bulk_support::limits();
    total.output_bytes = 1 << 30;
    let mut method = bulk_support::limits();
    // 512 until P3 2c.26: the longest artifact this scope produced was `Holder.<clinit>` **quoted**
    // — the construction that assigns `Holder.TOKEN` was refused, and the quote is longer than the
    // statement that replaced it. That member now writes `Holder.TOKEN = new java.lang.Object();`
    // where its `putstatic` runs, every text in the scope is under 512, and the local allowance
    // stopped nothing at all. 256 is the same declaration at a bound the fixture still meets: the
    // members longer than that stop, with their own reason and no text that claims to be an artifact.
    method.output_bytes = 256;

    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(total.clone());
    let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let request = BulkRecoveryRequest::for_scope(environment, 1, method.clone());
    let mut sink = Recorder::new();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the fixture's scope is recoverable");

    assert_eq!(
        report.summary.limits.method, method,
        "the effective configuration publishes the local limits the request declared"
    );
    assert_eq!(
        report.summary.limits.total, total,
        "and the operation's own total beside them"
    );
    assert_eq!(
        sink.header()
            .expect("the stream published its header")
            .limits,
        report.summary.limits,
        "the header states both, before anything is discovered"
    );

    // The local limit really bound methods: this fixture's bodies emit more than 256 bytes of text
    // between them, so some method met its own allowance and stopped — with its own stop reason and
    // without a text that claims to be an artifact.
    let methods = sink.methods();
    let stopped: Vec<&StopReason> = methods
        .iter()
        .filter_map(|method| method.stop_reason.as_ref())
        .collect();
    assert!(
        !stopped.is_empty(),
        "the local allowance stopped methods: {:?}",
        sink.methods()
    );
    assert!(
        stopped.iter().all(|reason| matches!(
            reason,
            StopReason::Budget {
                dimension: CountedBudgetDimension::OutputBytes,
                ..
            }
        )),
        "every one of them states the dimension its own allowance refused: {stopped:?}"
    );
    assert!(
        report.summary.outcomes.not_produced > 0,
        "a method that stopped has no artifact: {:?}",
        report.summary
    );

    // And the operation itself was not stopped: the whole scope was walked, every declared method's
    // record was delivered, and no operation-level stop is recorded.
    assert_eq!(
        report.stop, None,
        "a method's local limit is not the operation's stop: {:?}",
        report.stop
    );
    assert!(report.summary.traversal_complete, "{:?}", report.summary);
    assert_eq!(
        report.summary.methods_delivered, report.summary.methods_declared,
        "every declared method of the scope was delivered: {:?}",
        report.summary
    );
    assert_eq!(report.summary.classes_refused, 0, "{:?}", report.summary);
}

/// The operation's own observation port sees what the workers configuration really did.
///
/// The figures are load-bearing in the one way a diagnostic can be: they are stated as *relations*
/// the run has to satisfy — one window call per produced record, one delivery per record the sink was
/// handed, no worker busier than its own lifetime — so a port whose call sites were removed, or an
/// operation that stopped coordinating the way it says it does, fails here instead of publishing a
/// smaller number. `optimize-demand-workloads` 4.1 is where these relations come from, and the
/// numbers the port reports are what the performance work reads.
#[test]
fn the_observation_port_sees_the_coordination_that_really_happened() {
    let probe = Arc::new(BulkProbe::new());
    let case = run_with(4, {
        let probe = probe.clone();
        move |request| request.with_probe(probe)
    });
    let report = &case.report;
    let reading = probe.reading();
    let row = |where_: &str, site: &str| -> u64 {
        match where_ {
            "ledger" => {
                reading
                    .ledger
                    .iter()
                    .find(|row| row.site == site)
                    .unwrap_or_else(|| panic!("{site} is a ledger site"))
                    .calls
            }
            _ => {
                reading
                    .window
                    .iter()
                    .find(|row| row.site == site)
                    .unwrap_or_else(|| panic!("{site} is a window site"))
                    .calls
            }
        }
    };

    assert_eq!(report.summary.status(), "complete", "{:?}", report.summary);
    assert_eq!(
        reading.records_delivered,
        case.sink.events.len() as u64,
        "every record the operation handed over is one delivery, and the sink saw exactly those"
    );
    assert_eq!(
        row("window", "place_control"),
        report.summary.classes_prepared,
        "one control record is handed to a slot per prepared class"
    );
    assert_eq!(
        row("window", "place_method"),
        report.summary.methods_executed,
        "one method record is handed to a slot per executed method"
    );
    assert!(
        row("window", "take_front") >= report.summary.methods_delivered,
        "the coordinator took the front of the window at least once per delivered method record"
    );
    assert_eq!(
        reading.class_tasks, report.summary.classes_seen,
        "one class task per class candidate the traversal yielded"
    );
    assert_eq!(
        reading.worker_threads, report.summary.limits.workers_effective as u64,
        "the workers the operation really created are the workers it published"
    );

    let charges: u64 = reading.ledger[..3].iter().map(|site| site.calls).sum();
    assert!(
        charges > 0,
        "the operation charged its total: {:?}",
        reading.ledger
    );
    assert!(
        row("ledger", "checkpoint") >= charges,
        "a charge on a budget that bills to an operation is preceded by that operation's checkpoint \
         ({} checkpoints for {charges} charges)",
        row("ledger", "checkpoint")
    );
    assert_eq!(
        reading.ledger.iter().map(|site| site.refusals).sum::<u64>(),
        0,
        "a complete run refuses nothing: {:?}",
        reading.ledger
    );

    assert!(reading.worker_thread_nanos > 0 && reading.worker_busy_nanos > 0);
    assert!(
        reading.worker_busy_nanos <= reading.worker_thread_nanos,
        "a worker cannot be busier than the life it lived: {reading:?}"
    );
    assert!(
        reading.class_task_max_nanos <= reading.class_task_nanos,
        "the longest class task is one of them: {reading:?}"
    );
    assert!(
        reading.declared_methods_max > 0,
        "the fixture's classes declare methods: {reading:?}"
    );
    assert!(reading.sink_nanos > 0, "the sink was called: {reading:?}");

    // The port is not an answer: nothing the operation publishes carries a figure from it.
    let published = serde_json::to_value(&report.summary).expect("a summary serializes");
    assert!(
        published.get("probe").is_none() && published.get("observation").is_none(),
        "an observation is not part of what the operation states: {published}"
    );
}
