//! Bulk task 4.2 through the public entry: the worker lifecycle, exercised through the faults the
//! change's own request type carries.
//!
//! What this file pins:
//!
//! * at most `W` class tasks run at once and **two of them really ran at the same time**: the class
//!   tasks pass through a rendezvous the test holds, so "they overlapped" is an observation rather
//!   than an inference from a duration;
//! * no worker thread outlives the call, whatever ended it — the test's own watch counts the worker
//!   threads and reads zero after every one of these runs;
//! * a worker the operating system refuses to create closes the operation **before anything is
//!   dispatched**: the result is `Failed`, no class is recovered, and nothing falls back to the
//!   calling thread;
//! * a worker that panics is a `Failed` operation too: an unprepared class follows `Refused`, a
//!   prepared class keeps its `Prepared` handoff and states that it did not finish, records already
//!   delivered stand, and no second run is started.
//!
//! This target needs the `test-support` feature, because the faults it injects are compiled out of a
//! production build (the root manifest states that beside the feature).

#![cfg(feature = "test-support")]

mod bulk_support;

use bulk_support::{
    FLAT_PREFIXES, Recorder, container_roots, environment, flat_fixture, open, request, tree_scope,
};
use jarde::bulk::{BulkFaults, ClassGate, WorkerWatch};
use jarde::*;
use std::sync::Arc;
use std::time::Duration;

/// One run with the faults a case needs, and the watch that counts the worker threads.
struct Case {
    report: BulkRecoveryReport,
    sink: Recorder,
    watch: Arc<WorkerWatch>,
}

fn run(workers: usize, faults: BulkFaults, mut sink: Recorder) -> Case {
    let (snapshot, _opened) = open(flat_fixture());
    let content = vec![snapshot.clone()];
    let mut budget = Budget::new(bulk_support::limits());
    let roots = container_roots(&snapshot, &mut budget, &FLAT_PREFIXES);
    let environment = environment(&snapshot, tree_scope(), roots);
    let mut request = request(environment, workers);
    let watch = faults
        .worker_watch
        .clone()
        .unwrap_or_else(|| Arc::new(WorkerWatch::new()));
    request.faults = BulkFaults {
        worker_watch: Some(Arc::clone(&watch)),
        ..faults
    };
    let report = Engine::new()
        .recover_all(&content, &request, &mut budget, &mut sink)
        .expect("the operation returns a report whatever ended it");
    Case {
        report,
        sink,
        watch,
    }
}

#[test]
fn at_most_the_window_of_class_tasks_runs_at_once_and_two_of_them_overlap() {
    // The rendezvous opens when two class tasks are inside the class task at the same time, so a
    // runner that executed them one at a time would stall here until the gate's own timeout and
    // would never record a peak of two.
    let gate = Arc::new(ClassGate::new(2, Duration::from_secs(10)));
    let case = run(
        2,
        BulkFaults {
            class_gate: Some(Arc::clone(&gate)),
            ..BulkFaults::default()
        },
        Recorder::new(),
    );
    assert_eq!(case.report.summary.status(), "complete");
    assert_eq!(case.report.summary.methods_delivered, 18);
    assert!(
        gate.peak() >= 2,
        "two class tasks really stood inside the gate together: peak {}",
        gate.peak()
    );
    assert_eq!(
        case.watch.high_water(),
        2,
        "two worker threads were alive at once"
    );
    assert_eq!(case.watch.alive(), 0, "and none of them outlived the call");
    assert!(
        case.report.window.concurrent_classes_high_water == 2,
        "the operation's own count agrees: {:?}",
        case.report.window
    );
    assert!(
        case.report.window.concurrent_classes_high_water <= case.report.window.active_classes,
        "the declared window is the bound"
    );
}

#[test]
fn a_worker_the_system_refuses_to_create_fails_the_operation_before_dispatching() {
    let case = run(
        4,
        BulkFaults {
            fail_worker_at: Some(1),
            ..BulkFaults::default()
        },
        Recorder::new(),
    );
    assert_eq!(
        case.report.summary.status(),
        "failed",
        "{:?}",
        case.report.summary
    );
    assert_eq!(
        case.report.summary.classes_seen, 0,
        "nothing was dispatched: no silent serial fallback"
    );
    assert_eq!(case.report.summary.methods_executed, 0);
    assert_eq!(case.sink.methods().len(), 0);
    assert!(
        case.report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "bulk_worker_spawn_failed"),
        "the failure is located: {:?}",
        case.report.diagnostics
    );
    assert!(matches!(
        case.report.stop.as_ref().map(|stop| stop.kind),
        Some(BulkStopKind::Infrastructure)
    ));
    assert_eq!(
        case.watch.alive(),
        0,
        "the workers that were created before the refusal were joined"
    );
    assert!(!case.report.final_delivered || case.sink.final_event().is_some());
}

#[test]
fn a_worker_that_panics_fails_the_operation_and_leaves_no_thread_behind() {
    // The fault panics inside the class task of ordinal 2 before it prepares anything, so the class
    // must use the unprepared Refused branch and cannot claim a method of its own. What is asserted
    // below is deterministic under any
    // scheduling: the failure itself, the absence of leftover threads, the terminal record the
    // consumer gets, and the agreement between every class end the sink saw and the records it was
    // really handed. Whether the window had already reached the panicking class when the operation
    // closed is a scheduling fact, so that class's own end is checked when it is there.
    let case = run(
        4,
        BulkFaults {
            panic_at_class_ordinal: Some(2),
            ..BulkFaults::default()
        },
        Recorder::new(),
    );
    assert_eq!(
        case.report.summary.status(),
        "failed",
        "{:?}",
        case.report.summary
    );
    assert_eq!(
        case.watch.alive(),
        0,
        "a panicking worker is joined like any other: none is left running"
    );
    assert!(
        case.watch.high_water() <= 4,
        "and never more workers than the effective count: {}",
        case.watch.high_water()
    );
    assert!(
        case.report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "bulk_worker_panicked"),
        "the panic is located in the report: {:?}",
        case.report.diagnostics
    );
    assert_eq!(
        case.report.stop.as_ref().map(|stop| stop.kind),
        Some(BulkStopKind::Infrastructure),
        "the operation names the failure as its own stop: {:?}",
        case.report.stop
    );
    let final_event = case
        .sink
        .final_event()
        .expect("the consumer is still writable, so it gets the terminal record");
    assert_ne!(
        final_event.summary.status(),
        "complete",
        "and that record states the failure rather than a completion"
    );
    assert!(
        matches!(
            &final_event.summary.execution,
            ExecutionReport::Failed {
                reason: TerminationReason::Error { code },
                ..
            } if code == "bulk_worker_panicked"
        ),
        "the final record preserves the worker panic as the operation's first cause: {:?}",
        final_event.summary.execution
    );
    // EOF and the worker panic can be observed in either order; traversal_complete describes
    // discovery reaching the scope end, not whether execution succeeded.

    // Every class end the sink saw is consistent with the records it was really handed: a class that
    // states it published N records was handed N of them, and a class that did not finish says so
    // with the code of the failure that ended the operation. The Prepared set comes from callbacks
    // that this sink confirmed; it is not inferred from the summary's prepared count.
    let delivered = case.sink.methods();
    let prepared_ordinals: Vec<u64> = case
        .sink
        .prepared()
        .into_iter()
        .map(|prepared| prepared.class_ordinal)
        .collect();
    for end in case.sink.class_ends() {
        let handed = delivered
            .iter()
            .filter(|method| method.class_ordinal == end.class_ordinal)
            .count() as u64;
        match &end.completion {
            ClassCompletion::Completed { methods } => assert_eq!(
                *methods, handed,
                "class {} states a completion only over the records it published",
                end.class_ordinal
            ),
            ClassCompletion::Stopped { methods, code } => {
                assert!(
                    *methods >= handed,
                    "class {} published at least what it delivered: {methods} against {handed}",
                    end.class_ordinal
                );
                assert_eq!(
                    code, "bulk_worker_panicked",
                    "the class that stopped names the failure that stopped it"
                );
            }
            ClassCompletion::Refused { .. } => assert!(
                !prepared_ordinals.contains(&end.class_ordinal),
                "class {} cannot be refused after its Prepared event was confirmed",
                end.class_ordinal
            ),
        }
        if !matches!(end.completion, ClassCompletion::Refused { .. }) {
            assert!(
                prepared_ordinals.contains(&end.class_ordinal),
                "non-Refused ClassEnd for ordinal {} requires a Prepared callback the sink confirmed; prepared ordinals: {:?}",
                end.class_ordinal,
                prepared_ordinals
            );
        }
    }
    let ordinals: Vec<u64> = delivered
        .iter()
        .map(|method| method.class_ordinal)
        .collect();
    let mut sorted = ordinals.clone();
    sorted.sort_unstable();
    assert_eq!(
        ordinals, sorted,
        "the delivered prefix stays in class order"
    );
    assert!(
        case.report.summary.methods_delivered <= case.report.summary.methods_executed,
        "delivered work is a prefix of executed work: {:?}",
        case.report.summary
    );
    // The panicking class, when the window reached it, states the panic and claims no method.
    if let Some(end) = case
        .sink
        .class_ends()
        .into_iter()
        .find(|end| end.class_ordinal == 2)
    {
        assert_eq!(
            end.completion,
            ClassCompletion::Refused {
                code: "bulk_worker_panicked".to_owned()
            },
            "the class that panicked before preparation takes the Refused branch and keeps its cause"
        );
    }
}

#[test]
fn a_worker_that_panics_after_a_method_keeps_the_prepared_prefix_count() {
    let case = run(
        4,
        BulkFaults {
            panic_after_method: Some((0, 0)),
            ..BulkFaults::default()
        },
        Recorder::new(),
    );

    assert_eq!(case.watch.alive(), 0, "all workers are joined after panic");
    assert_eq!(
        case.report.stop.as_ref().map(|stop| stop.kind),
        Some(BulkStopKind::Infrastructure)
    );
    assert!(matches!(
        &case.report.summary.execution,
        ExecutionReport::Failed {
            reason: TerminationReason::Error { code },
            ..
        } if code == "bulk_worker_panicked"
    ));

    let prepared = case
        .sink
        .prepared()
        .into_iter()
        .find(|event| event.class_ordinal == 0)
        .expect("the panicking class's Prepared event was confirmed");
    let methods: Vec<_> = case
        .sink
        .methods()
        .into_iter()
        .filter(|event| event.class_ordinal == 0)
        .collect();
    let end = case
        .sink
        .class_ends()
        .into_iter()
        .find(|event| event.class_ordinal == 0)
        .expect("the panicking class has a terminal record");

    assert_eq!(prepared.class_ordinal, end.class_ordinal);
    assert_eq!(
        methods.len(),
        1,
        "the injected panic follows one produced method"
    );
    assert!(matches!(
        end.completion,
        ClassCompletion::Stopped { methods: 1, ref code }
            if code == "bulk_worker_panicked"
    ));
    assert!(
        case.report.summary.methods_executed >= 1,
        "the operation records the method work that preceded the panic"
    );
}

#[test]
fn a_run_without_faults_still_reports_its_own_lifecycle() {
    let case = run(4, BulkFaults::default(), Recorder::new());
    assert_eq!(case.report.summary.status(), "complete");
    assert_eq!(case.watch.alive(), 0);
    assert!(
        case.watch.high_water() >= 2,
        "{} workers ran",
        case.watch.high_water()
    );
    assert!(
        case.watch.high_water() <= case.report.summary.limits.workers_effective,
        "and never more than the effective count: {} against {}",
        case.watch.high_water(),
        case.report.summary.limits.workers_effective
    );
}
