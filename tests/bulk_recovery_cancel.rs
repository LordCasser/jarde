//! Bulk tasks 4.4 and 4.5 through the public entry: cancellation, consumer stops and the account
//! they leave behind.
//!
//! What this file pins:
//!
//! * a cancellation is observed at **every** wait the design names — before the first dispatch, while
//!   a worker is waiting for its own result slot, while the coordinator is waiting for a record, and
//!   inside the traversal — and each of those sites ends the operation instead of waiting for work
//!   that will not come;
//! * a cancelled operation returns: no waiter is left behind, and the coordinator does not fall into
//!   an unbounded wait;
//! * work that was executed and never delivered is still **billed and counted as executed**: the
//!   report states execution holes rather than pretending the work never happened;
//! * the confirmed delivery prefix stands, and the aggregate is `Cancelled` (a consumer that stopped
//!   or a caller that cancelled) or `Partial` (a budget stop) — never `Complete`;
//! * a consumer that fails is infrastructure: `Failed`, no `Final`, and the prefix it confirmed
//!   before it failed is still exactly what it received.

mod bulk_support;

use bulk_support::{
    FLAT_PREFIXES, Recorded, Recorder, container_roots, environment, flat_fixture, open, request,
    tree_scope,
};
use jarde::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The fixture every case here reads, with the request built the way a caller would build it.
fn fixture(workers: usize) -> (Vec<ArtifactSnapshot>, BulkRecoveryRequest, Budget) {
    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    (content, request(environment, workers), budget)
}

#[test]
fn a_cancelled_request_publishes_nothing_and_dispatches_nothing() {
    let (content, request, mut budget) = fixture(4);
    budget.cancellation_token().cancel();
    let mut sink = Recorder::new();
    let started = std::time::Instant::now();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("a cancelled request is a report, not an input error");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "the operation returns instead of waiting for work it will never dispatch"
    );
    assert_eq!(report.summary.status(), "cancelled", "{:?}", report.summary);
    assert_eq!(report.summary.classes_seen, 0);
    assert_eq!(report.summary.methods_executed, 0);
    assert!(sink.events.is_empty(), "not even the header was published");
    assert!(
        matches!(
            report.stop.as_ref().map(|stop| stop.kind),
            Some(BulkStopKind::Cancelled)
        ),
        "the report names the stop that happened: {:?}",
        report.stop
    );
    assert_eq!(report.window.concurrent_classes_high_water, 0);
}

#[test]
fn a_cancellation_between_classes_ends_the_traversal_and_the_class_in_flight() {
    // One worker, so the run is the serial closed loop: the coordinator pulls one class, recovers it
    // and only then pulls the next one. Cancelling from inside the first delivered record therefore
    // ends the operation at the two checkpoints the design names beside the waits: the class in
    // flight stops before its next method is executed (the compute site), and the traversal is never
    // pulled again (the discovery site between entries) — so exactly one class is ever seen, exactly
    // one of its eight members is executed and delivered, and the rest are stated as holes rather
    // than run.
    let (content, request, mut budget) = fixture(1);
    let token = budget.cancellation_token();
    let cancelled = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&cancelled);
    let mut sink = Recorder::new();
    sink.hook = Some(Box::new(move |event: &Recorded| {
        if let Recorded::Method(_) = event
            && counter.fetch_add(1, Ordering::SeqCst) == 0
        {
            token.cancel();
        }
    }));
    let started = std::time::Instant::now();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("a cancelled operation returns its report");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "the class in flight and the traversal both stopped instead of waiting"
    );
    assert_eq!(
        cancelled.load(Ordering::SeqCst),
        1,
        "the hook cancelled at the first record"
    );
    assert_eq!(report.summary.status(), "cancelled", "{:?}", report.summary);
    assert_eq!(
        report.summary.classes_seen, 1,
        "the traversal was never pulled again: {:?}",
        report.summary
    );
    assert_eq!(sink.prepared().len(), 1);
    assert!(
        report.summary.methods_executed >= 1,
        "the record the hook cancelled on was really executed: {:?}",
        report.summary
    );
    assert!(
        report.summary.methods_executed < report.summary.methods_declared,
        "the class in flight stopped before its declaration ended: {:?}",
        report.summary
    );
    // The cancellation was recorded inside the first record's callback, so no later record may be
    // published: the checkpoint before every sink call is what makes the confirmed prefix exactly
    // the record that was in flight.
    assert_eq!(
        sink.methods().len(),
        1,
        "no record was published after the cancellation"
    );
    assert_eq!(report.summary.methods_delivered, 1);
    assert_eq!(
        report.summary.methods_not_executed,
        report
            .summary
            .methods_declared
            .saturating_sub(report.summary.methods_executed),
        "the members the cancellation cut off are stated as unexecuted: {:?}",
        report.summary
    );
    assert!(report.summary.methods_declared > report.summary.methods_executed);
    assert!(
        report.usage.method_bodies >= 1,
        "the work that was admitted is still billed: {:?}",
        report.usage
    );
    assert!(!report.final_delivered);
    assert!(
        matches!(
            report.stop.as_ref().map(|stop| stop.kind),
            Some(BulkStopKind::Cancelled)
        ),
        "the report names the cancellation: {:?}",
        report.stop
    );
}

#[test]
fn a_cancellation_mid_run_keeps_the_prefix_and_bills_the_holes() {
    // The caller cancels from inside the sink, after three records: everything already confirmed
    // stands, the dispatch stops, and the work that was already admitted is still accounted.
    let (content, request, mut budget) = fixture(4);
    let token = budget.cancellation_token();
    let cancelled_at = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&cancelled_at);
    let mut sink = Recorder::new();
    sink.hook = Some(Box::new(move |event: &Recorded| {
        if let Recorded::Method(_) = event {
            let seen = counter.fetch_add(1, Ordering::SeqCst) + 1;
            if seen == 3 {
                token.cancel();
            }
        }
    }));
    let started = std::time::Instant::now();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("a cancelled operation returns its report");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "no wait is unbounded"
    );
    assert_eq!(
        cancelled_at.load(Ordering::SeqCst),
        3,
        "the sink really saw the cancellation point"
    );
    let delivered = sink.methods().len() as u64;
    assert!(
        delivered >= 3,
        "the confirmed prefix stands: {delivered} record(s) were delivered"
    );
    assert_eq!(
        report.summary.methods_delivered, delivered,
        "the delivered count is exactly what the sink confirmed"
    );
    assert!(
        report.summary.methods_executed >= report.summary.methods_delivered,
        "executed work is never less than delivered work: {} against {}",
        report.summary.methods_executed,
        report.summary.methods_delivered
    );
    assert_eq!(
        report.summary.status(),
        "cancelled",
        "a cancelled run states it: {:?}",
        report.summary
    );
    assert!(!report.final_delivered, "no `Final` is claimed for it");
    assert_eq!(
        report.summary.methods_not_executed,
        report
            .summary
            .methods_declared
            .saturating_sub(report.summary.methods_executed),
        "the holes are stated as unexecuted methods, not hidden"
    );
    let billed = report.usage.method_bodies + report.usage.code_bytes + report.usage.ir_items;
    assert!(
        billed > 0,
        "the work that really ran is still billed: {:?}",
        report.usage
    );
    // The delivery order is still the physical one: a cancelled prefix is a prefix, never a shuffle.
    let ordinals: Vec<(u64, u64)> = sink
        .methods()
        .iter()
        .map(|method| (method.class_ordinal, method.member_ordinal))
        .collect();
    let mut sorted = ordinals.clone();
    sorted.sort_unstable();
    assert_eq!(ordinals, sorted, "the prefix is in order");
    let expected: Vec<(u64, u64)> = vec![
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
    ];
    assert_eq!(
        ordinals,
        expected[..ordinals.len()].to_vec(),
        "and it is the prefix of the physical order"
    );
}

#[test]
fn a_cancellation_while_the_window_is_full_wakes_every_waiter() {
    // Two class tasks and a window that holds exactly their two slots: the sink is slow enough that
    // the second class's worker has already put its result in its own slot and is waiting for the
    // coordinator to take it when the caller cancels. A library that waited for a slot without
    // observing the cancellation would never return here.
    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let ceiling = jarde::DEFAULT_MAX_RESULT_WEIGHT / 4;
    let request = request(environment, 2).with_capacities(
        jarde::DEFAULT_MAX_CLASS_BYTES,
        ceiling,
        2 * ceiling,
    );
    let token = budget.cancellation_token();
    let seen = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&seen);
    let mut sink = Recorder::new();
    sink.hook = Some(Box::new(move |event: &Recorded| {
        if let Recorded::Method(method) = event
            && method.class_ordinal == 0
            && method.member_ordinal == 0
        {
            // While the coordinator is inside this callback, the class behind the front one fills
            // its own slot and starts waiting for it to be taken.
            std::thread::sleep(std::time::Duration::from_millis(150));
            counter.fetch_add(1, Ordering::SeqCst);
            token.cancel();
        }
    }));
    let started = std::time::Instant::now();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("a cancelled operation returns its report");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(10),
        "every waiter woke up: the call returned in {:?}",
        started.elapsed()
    );
    assert_eq!(seen.load(Ordering::SeqCst), 1, "the hook really ran");
    assert_eq!(
        report.summary.limits.workers_effective, 2,
        "two class tasks were really running: {:?}",
        report.summary.limits
    );
    assert!(
        report.summary.status() == "cancelled" || report.summary.status() == "partial",
        "the run states a stop rather than a completion: {:?}",
        report.summary
    );
    assert!(
        report.summary.methods_executed >= report.summary.methods_delivered,
        "the account still adds up: {:?}",
        report.summary
    );
    assert!(
        report.summary.classes_seen <= 4 && report.summary.methods_executed <= 18,
        "the operation never states more work than the scope holds: {:?}",
        report.summary
    );
    assert!(
        report.window.concurrent_classes_high_water == 2,
        "both class tasks were live at the same time: {:?}",
        report.window
    );
}

#[test]
fn a_consumer_that_stops_the_stream_keeps_its_prefix() {
    let (content, request, mut budget) = fixture(4);
    let mut sink = Recorder::new();
    sink.method_behaviour = bulk_support::Behaviour::StopAt(2);
    let started = std::time::Instant::now();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("a stopped stream returns its report");
    assert!(started.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(report.summary.status(), "cancelled", "{:?}", report.summary);
    assert_eq!(
        report.summary.methods_delivered, 2,
        "the consumer confirmed two records, and exactly those are counted: {:?}",
        report.summary
    );
    assert!(sink.methods().len() >= 2);
    assert!(sink.final_event().is_none() || !report.final_delivered);
    assert!(
        matches!(
            report.stop.as_ref().map(|stop| stop.kind),
            Some(BulkStopKind::Sink)
        ),
        "the report names the consumer as the stop: {:?}",
        report.stop
    );
    assert_eq!(
        report.summary.methods_not_executed,
        report
            .summary
            .methods_declared
            .saturating_sub(report.summary.methods_executed),
        "the suffix it never ran is stated as unexecuted"
    );
    assert!(
        report.usage.method_bodies > 0,
        "the work before the stop is billed"
    );
}

#[test]
fn a_consumer_that_fails_is_infrastructure() {
    let (content, request, mut budget) = fixture(4);
    let mut sink = Recorder::new();
    sink.method_behaviour = bulk_support::Behaviour::FailAt(2);
    let started = std::time::Instant::now();
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("a failed consumer still leaves a report");
    assert!(started.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(report.summary.status(), "failed", "{:?}", report.summary);
    assert!(!report.final_delivered, "a failed consumer gets no `Final`");
    assert_eq!(
        report.summary.methods_delivered, 2,
        "the two records it confirmed before failing are the delivered ones"
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "bulk_test_sink_failure"),
        "the failure is located in the report: {:?}",
        report.diagnostics
    );
    assert!(
        report.summary.classes_refused == 0 && report.summary.classes_prepared > 0,
        "the classes that ran before the failure keep their disposition: {:?}",
        report.summary
    );
}
