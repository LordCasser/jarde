//! 4.1: the operation's shared ledger, and the local limits a method keeps beside it.
//!
//! `add-parallel-bulk-recovery` decision 4 states one total and per-method allowances. This file
//! holds the half a design sentence cannot: that the total is really one total when two threads
//! race for it, that the entry budget's own usage keeps counting, that the three work classes are
//! separable and add up, that a local refusal is not the operation's stop, and that the first stop
//! the operation publishes is not the one that raced last.
//!
//! ```text
//! verify: cargo test --test p1_budget_ledger --locked
//! ```
//!
//! What the ledger is, and where its neighbours are
//! ------------------------------------------------
//!
//! [`OperationLedger`] is built from the budget that opened the operation and attached to every
//! budget that does its work ([`Budget::with_ledger`]). [`Budget`] keeps its own arithmetic — the
//! direct single-request path is untouched and is asserted here as well — and the ledger adds the
//! operation's totals, its deadline and its one published stop. `tests/p5_shared_payload.rs` holds
//! the other half of slice 2.2: the facts the operation shares between its workers.

use jarde::{
    Budget, BudgetDimension, CountedBudgetDimension, Error, Limits, TerminationReason,
    UsageSnapshot,
};
use jarde_reader::ledger::{
    BulkStop, BulkStopKind, LEDGER_OBSERVATION_STRIDE, LedgerEntryTiming, LedgerObserver,
    LedgerSite, OperationLedger, UsageOwner,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::time::Duration;

/// Headroom for one budget: every dimension the tests use, with the clock open.
///
/// The values are deliberately roomy. A case that wants a limit states it by replacing the one
/// dimension it means, so a refusal can only come from the limit the test set — from the operation's
/// or from the local budget's, whichever the case is about.
fn limits(value: u64) -> Limits {
    Limits {
        input_bytes: value,
        archive_entries: value,
        entry_bytes: value,
        read_bytes: value,
        class_bytes: value,
        attribute_bytes: value,
        code_bytes: value,
        result_items: value,
        output_bytes: value,
        class_headers: value,
        method_bodies: value,
        ir_items: value,
        ir_edges: value,
        analysis_steps: value,
        normalization_clones: value,
        nested_depth: value,
        dependency_depth: value,
        elapsed_millis: u64::MAX,
    }
}

/// One operation's ledger, and the three owners its work is booked under.
#[test]
fn one_ledger_is_one_total_for_every_worker() {
    fn assert_shareable<T: Clone + Send + Sync + 'static>() {}
    assert_shareable::<OperationLedger>();

    let mut entry = Budget::new(limits(1_000));
    // The bytes that opened the input were spent before the operation began, and they keep counting.
    entry
        .charge(CountedBudgetDimension::InputBytes, 40)
        .unwrap();
    entry.charge(CountedBudgetDimension::ReadBytes, 60).unwrap();
    assert!(
        entry.ledger().is_none(),
        "a budget carries no ledger until a caller attaches one"
    );

    let ledger = OperationLedger::new(&entry);
    assert_eq!(ledger.entry_usage().input_bytes, 40);
    assert_eq!(ledger.entry_usage().read_bytes, 60);
    assert_eq!(
        ledger.usage().read_bytes,
        60,
        "the operation starts from the entry usage instead of from zero"
    );

    let mut discovery = Budget::new(limits(1_000));
    discovery.with_ledger(ledger.clone(), UsageOwner::Discovery);
    let mut methods = Budget::new(limits(1_000));
    methods.with_ledger(ledger.clone(), UsageOwner::Methods);
    let mut delivery = Budget::new(limits(1_000));
    delivery.with_ledger(ledger.clone(), UsageOwner::Delivery);

    discovery
        .charge(CountedBudgetDimension::ArchiveEntries, 7)
        .unwrap();
    methods
        .charge(CountedBudgetDimension::AnalysisSteps, 11)
        .unwrap();
    delivery
        .charge(CountedBudgetDimension::OutputBytes, 13)
        .unwrap();

    // Each owner's share is its own: one class of work does not move another class's usage.
    assert_eq!(ledger.cumulative(UsageOwner::Discovery).archive_entries, 7);
    assert_eq!(ledger.cumulative(UsageOwner::Discovery).analysis_steps, 0);
    assert_eq!(ledger.cumulative(UsageOwner::Methods).analysis_steps, 11);
    assert_eq!(ledger.cumulative(UsageOwner::Methods).archive_entries, 0);
    assert_eq!(ledger.cumulative(UsageOwner::Delivery).output_bytes, 13);

    // And the operation's total is exactly the entry usage plus the three shares, dimension by
    // dimension: a total that dropped a share or counted one twice fails here.
    for dimension in CountedBudgetDimension::ALL {
        let owners: u64 = UsageOwner::ALL
            .iter()
            .map(|owner| ledger.cumulative(*owner).counted_usage(dimension))
            .sum();
        assert_eq!(
            owners + ledger.entry_usage().counted_usage(dimension),
            ledger.usage().counted_usage(dimension),
            "{dimension:?}: the totals are not entry + discovery + methods + delivery"
        );
    }
    assert_eq!(ledger.stop_reason(), None);
}

/// Two workers race for the last quota: one takes it, and the other is not billed for what it did
/// not get.
#[test]
fn two_workers_race_for_the_last_quota_and_the_refused_one_is_not_billed() {
    let mut operation = limits(8);
    operation.analysis_steps = 1;
    let entry = Budget::new(operation);
    let ledger = OperationLedger::new(&entry);

    let barrier = Barrier::new(2);
    let outcomes: Vec<(Option<Error>, UsageSnapshot)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let ledger = ledger.clone();
                let barrier = &barrier;
                scope.spawn(move || {
                    // The local budget has all the headroom in the world: what stops one of these
                    // two is the operation's one step, not a local limit.
                    let mut worker = Budget::new(limits(8));
                    worker.with_ledger(ledger, UsageOwner::Methods);
                    barrier.wait();
                    let outcome = worker.charge(CountedBudgetDimension::AnalysisSteps, 1);
                    (outcome.err(), worker.usage())
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("a worker thread returns"))
            .collect()
    });

    let refused: Vec<&(Option<Error>, UsageSnapshot)> = outcomes
        .iter()
        .filter(|(error, _)| error.is_some())
        .collect();
    assert_eq!(
        refused.len(),
        1,
        "exactly one of two workers may take an operation's last unit: {outcomes:?}"
    );
    assert_eq!(
        refused[0].0,
        Some(Error::BudgetExceeded {
            dimension: BudgetDimension::AnalysisSteps,
            limit: 1,
            consumed: 1,
            requested: 1,
        }),
        "the refused worker must be refused by the operation's total, with the total's numbers"
    );
    assert_eq!(
        refused[0].1.analysis_steps, 0,
        "a worker that took no permit was billed for work it did not start"
    );
    assert_eq!(
        ledger.usage().analysis_steps,
        1,
        "the operation's own total crossed its limit"
    );
    assert_eq!(ledger.cumulative(UsageOwner::Methods).analysis_steps, 1);
    assert_eq!(
        ledger.stop_reason(),
        Some(BulkStop {
            owner: Some(UsageOwner::Methods),
            kind: BulkStopKind::Budget,
            dimension: Some(BudgetDimension::AnalysisSteps),
        }),
        "the quota that ran out is the operation's stop, attributed to the work that needed it"
    );
}

/// The entry usage is the operation's starting point, and the operation's limit is spent by it.
#[test]
fn the_entry_usage_counts_against_the_operation_and_is_never_reset() {
    let mut operation = limits(100);
    operation.read_bytes = 5;
    let mut entry = Budget::new(operation);
    entry.charge(CountedBudgetDimension::ReadBytes, 3).unwrap();

    let ledger = OperationLedger::new(&entry);
    let mut worker = Budget::new(limits(100));
    worker.with_ledger(ledger.clone(), UsageOwner::Methods);
    worker.charge(CountedBudgetDimension::ReadBytes, 2).unwrap();
    assert_eq!(ledger.entry_usage().read_bytes, 3);
    assert_eq!(ledger.usage().read_bytes, 5);

    // The operation has nothing left: the entry's three bytes and the worker's two are one total.
    assert_eq!(
        worker
            .charge(CountedBudgetDimension::ReadBytes, 1)
            .unwrap_err(),
        Error::BudgetExceeded {
            dimension: BudgetDimension::ReadBytes,
            limit: 5,
            consumed: 5,
            requested: 1,
        }
    );
    assert_eq!(
        worker.usage().read_bytes,
        2,
        "the refused unit was billed to the local budget"
    );
    assert_eq!(
        ledger.usage().read_bytes,
        5,
        "the refused unit was billed to the operation: the total may not move on a refusal"
    );
}

/// A second operation on one request starts where the first ended, so no quota is handed out twice.
#[test]
fn a_later_operation_on_one_request_folds_the_first_ones_total() {
    let mut operation = limits(100);
    operation.read_bytes = 5;
    let mut entry = Budget::new(operation);
    entry.charge(CountedBudgetDimension::ReadBytes, 3).unwrap();

    let ledger = OperationLedger::new(&entry);
    entry.with_ledger(ledger.clone(), UsageOwner::Delivery);
    entry
        .charge(CountedBudgetDimension::ResultItems, 4)
        .unwrap();
    let mut worker = Budget::new(limits(100));
    worker.with_ledger(ledger.clone(), UsageOwner::Methods);
    worker.charge(CountedBudgetDimension::ReadBytes, 2).unwrap();
    assert_eq!(ledger.usage().read_bytes, 5);

    let next = OperationLedger::new(&entry);
    assert_eq!(
        next.entry_usage().read_bytes,
        5,
        "a ledger built from a budget that already bills to one starts from that ledger's total"
    );
    assert_eq!(next.entry_usage().result_items, 4);
    assert_eq!(next.usage().read_bytes, 5);

    let mut next_worker = Budget::new(limits(100));
    next_worker.with_ledger(next.clone(), UsageOwner::Methods);
    assert_eq!(
        next_worker
            .charge(CountedBudgetDimension::ReadBytes, 1)
            .unwrap_err(),
        Error::BudgetExceeded {
            dimension: BudgetDimension::ReadBytes,
            limit: 5,
            consumed: 5,
            requested: 1,
        },
        "the second operation was handed the first one's quota again"
    );
}

/// A depth mark keeps the deepest accepted value, and each owner keeps its own.
#[test]
fn depth_marks_take_the_deepest_value_per_owner() {
    let mut operation = limits(100);
    operation.nested_depth = 8;
    operation.dependency_depth = 8;
    let mut entry = Budget::new(operation.clone());
    entry.check_nested_depth(1).unwrap();

    let ledger = OperationLedger::new(&entry);
    assert_eq!(ledger.entry_usage().nested_depth, 1);

    let mut discovery = Budget::new(operation.clone());
    discovery.with_ledger(ledger.clone(), UsageOwner::Discovery);
    // This worker's own allowance is wider than the operation's, so the refusal below can only be
    // the operation's.
    let mut wide = operation;
    wide.nested_depth = 10;
    let mut methods = Budget::new(wide);
    methods.with_ledger(ledger.clone(), UsageOwner::Methods);

    discovery.check_nested_depth(3).unwrap();
    methods.check_nested_depth(2).unwrap();
    discovery.observe_dependency_depth(4).unwrap();
    methods.observe_dependency_depth(5).unwrap();

    let usage = ledger.usage();
    assert_eq!(
        usage.nested_depth, 3,
        "the high-water mark is the maximum of everything accepted, not the last value"
    );
    assert_eq!(usage.dependency_depth, 5);
    assert_eq!(ledger.cumulative(UsageOwner::Discovery).nested_depth, 3);
    assert_eq!(ledger.cumulative(UsageOwner::Methods).nested_depth, 2);
    assert_eq!(ledger.cumulative(UsageOwner::Discovery).dependency_depth, 4);
    assert_eq!(ledger.cumulative(UsageOwner::Methods).dependency_depth, 5);

    // A container deeper than the operation allows is that container's refusal. It is recorded as
    // neither a stop nor a usage: discovery reports the subtree and keeps walking.
    assert_eq!(
        methods.check_nested_depth(9).unwrap_err(),
        Error::BudgetExceeded {
            dimension: BudgetDimension::NestedDepth,
            limit: 8,
            consumed: 8,
            requested: 1,
        }
    );
    assert_eq!(ledger.stop_reason(), None);
    assert_eq!(ledger.usage().nested_depth, 3);
}

/// The operation's elapsed time is one wall clock, not the sum of its workers' time.
#[test]
fn the_elapsed_clock_is_the_operations_and_not_the_sum_of_worker_time() {
    const SLEEP: Duration = Duration::from_millis(200);
    let entry = Budget::new(limits(8));
    let ledger = OperationLedger::new(&entry);

    let barrier = Barrier::new(2);
    std::thread::scope(|scope| {
        for _ in 0..2 {
            let ledger = ledger.clone();
            let barrier = &barrier;
            scope.spawn(move || {
                let mut worker = Budget::new(limits(8));
                worker.with_ledger(ledger, UsageOwner::Methods);
                barrier.wait();
                std::thread::sleep(SLEEP);
                worker
                    .charge(CountedBudgetDimension::AnalysisSteps, 1)
                    .unwrap();
            });
        }
    });

    let elapsed = ledger.usage().elapsed_millis;
    assert!(
        elapsed >= 200,
        "the operation's clock did not cover its workers' work: {elapsed} ms"
    );
    assert!(
        elapsed < 400,
        "the operation's elapsed ({elapsed} ms) is the sum of two concurrent 200 ms waits, so the \
         clock is measured per worker instead of once for the operation"
    );
    let per_owner = ledger.cumulative(UsageOwner::Methods).elapsed_millis;
    assert!(
        (200..400).contains(&per_owner),
        "an owner's reading is the operation's clock, not a duration of its own: {per_owner} ms"
    );
    assert_eq!(ledger.usage().analysis_steps, 2);
}

/// A method's own allowance stops that method, and nothing else.
#[test]
fn a_local_limit_stops_that_work_and_leaves_the_operation_alone() {
    let mut operation = limits(32);
    operation.analysis_steps = 10;
    let entry = Budget::new(operation);
    let ledger = OperationLedger::new(&entry);

    let mut method_limits = limits(32);
    method_limits.analysis_steps = 1;
    let mut local = Budget::new(method_limits.clone());
    local.with_ledger(ledger.clone(), UsageOwner::Methods);
    local
        .charge(CountedBudgetDimension::AnalysisSteps, 1)
        .unwrap();
    assert_eq!(
        local
            .charge(CountedBudgetDimension::AnalysisSteps, 1)
            .unwrap_err(),
        Error::BudgetExceeded {
            dimension: BudgetDimension::AnalysisSteps,
            limit: 1,
            consumed: 1,
            requested: 1,
        },
        "the second step has to be refused by the method's own limit"
    );

    assert_eq!(
        ledger.stop_reason(),
        None,
        "a local refusal is not the operation's stop"
    );
    assert_eq!(ledger.usage().analysis_steps, 1);
    assert_eq!(ledger.cumulative(UsageOwner::Methods).analysis_steps, 1);
    assert_eq!(
        local.usage().analysis_steps,
        1,
        "the refused step was billed to the method"
    );

    let mut other = Budget::new(method_limits);
    other.with_ledger(ledger.clone(), UsageOwner::Methods);
    other
        .charge(CountedBudgetDimension::AnalysisSteps, 1)
        .unwrap();
    assert_eq!(
        ledger.usage().analysis_steps,
        2,
        "one method's local limit stopped the rest of the operation"
    );
}

/// One worker doing two parts of the operation books them separately.
#[test]
fn a_worker_can_state_which_part_of_the_operation_its_next_work_is() {
    let entry = Budget::new(limits(64));
    let ledger = OperationLedger::new(&entry);
    let mut worker = Budget::new(limits(64));
    worker.with_ledger(ledger.clone(), UsageOwner::Discovery);

    worker.charge(CountedBudgetDimension::ReadBytes, 5).unwrap();
    worker.set_owner(UsageOwner::Methods);
    worker
        .charge(CountedBudgetDimension::AnalysisSteps, 3)
        .unwrap();
    worker.set_owner(UsageOwner::Delivery);
    worker
        .charge(CountedBudgetDimension::OutputBytes, 2)
        .unwrap();

    assert_eq!(ledger.cumulative(UsageOwner::Discovery).read_bytes, 5);
    assert_eq!(ledger.cumulative(UsageOwner::Methods).analysis_steps, 3);
    assert_eq!(ledger.cumulative(UsageOwner::Delivery).output_bytes, 2);
    assert_eq!(ledger.cumulative(UsageOwner::Discovery).analysis_steps, 0);
    assert_eq!(ledger.usage().read_bytes, 5);
    assert_eq!(ledger.usage().analysis_steps, 3);
    assert_eq!(ledger.usage().output_bytes, 2);
    assert_eq!(
        worker.usage().analysis_steps,
        3,
        "the local budget keeps everything it spent, whatever part it spent it as"
    );
}

/// The operation's first stop is the one it publishes, and later observations do not replace it.
#[test]
fn the_first_stop_is_published_and_later_observations_do_not_replace_it() {
    let entry = Budget::new(limits(4));
    let ledger = OperationLedger::new(&entry);
    assert_eq!(ledger.stop_reason(), None);

    ledger.cancel(BulkStopKind::Sink);
    assert_eq!(
        ledger.stop_reason(),
        Some(BulkStop {
            owner: None,
            kind: BulkStopKind::Sink,
            dimension: None,
        })
    );

    let mut worker = Budget::new(limits(4));
    worker.with_ledger(ledger.clone(), UsageOwner::Methods);
    assert!(matches!(worker.poll(), Err(Error::Cancelled { .. })));
    assert!(matches!(
        worker.charge(CountedBudgetDimension::AnalysisSteps, 1),
        Err(Error::Cancelled { .. })
    ));
    assert_eq!(
        ledger.usage().analysis_steps,
        0,
        "work after the stop was started and billed"
    );

    ledger.cancel(BulkStopKind::Budget);
    ledger.cancel(BulkStopKind::Infrastructure);
    let stop = ledger.stop_reason().expect("the stop is published");
    assert_eq!(stop.kind, BulkStopKind::Sink);
    assert_eq!(stop.owner, None);
    assert_eq!(
        stop.termination(),
        None,
        "a consumer stop is stated by the aggregate's Cancelled, not by a budget reason"
    );
}

/// A quota the operation could not take is its stop, and the first one is what stays.
#[test]
fn a_quota_refusal_is_the_operations_stop() {
    let mut operation = limits(4);
    operation.code_bytes = 0;
    let entry = Budget::new(operation);
    let ledger = OperationLedger::new(&entry);

    // The local budget could afford the byte; the operation cannot.
    let mut worker = Budget::new(limits(4));
    worker.with_ledger(ledger.clone(), UsageOwner::Discovery);
    assert_eq!(
        worker
            .charge(CountedBudgetDimension::CodeBytes, 1)
            .unwrap_err(),
        Error::BudgetExceeded {
            dimension: BudgetDimension::CodeBytes,
            limit: 0,
            consumed: 0,
            requested: 1,
        }
    );
    assert_eq!(
        ledger.stop_reason(),
        Some(BulkStop {
            owner: Some(UsageOwner::Discovery),
            kind: BulkStopKind::Budget,
            dimension: Some(BudgetDimension::CodeBytes),
        })
    );
    assert_eq!(
        ledger.stop_reason().expect("a stop").termination(),
        Some(TerminationReason::BudgetExceeded {
            dimension: BudgetDimension::CodeBytes,
        }),
        "a budget stop is also a reason in the execution report's vocabulary"
    );
    assert_eq!(ledger.usage().code_bytes, 0);
    assert_eq!(worker.usage().code_bytes, 0);

    // A later, different reason does not move it.
    ledger.cancel(BulkStopKind::Cancelled);
    assert_eq!(
        ledger.stop_reason().expect("a stop").dimension,
        Some(BudgetDimension::CodeBytes)
    );
}

/// An overflowing request is refused with the numbers that produced it, never wrapped.
#[test]
fn an_overflowing_request_is_refused_rather_than_wrapped() {
    let operation = limits(u64::MAX);
    let mut entry = Budget::new(operation.clone());
    entry
        .charge(CountedBudgetDimension::InputBytes, u64::MAX)
        .unwrap();
    let ledger = OperationLedger::new(&entry);

    let mut worker = Budget::new(operation);
    worker.with_ledger(ledger.clone(), UsageOwner::Methods);
    assert_eq!(
        worker
            .charge(CountedBudgetDimension::InputBytes, 1)
            .unwrap_err(),
        Error::BudgetExceeded {
            dimension: BudgetDimension::InputBytes,
            limit: u64::MAX,
            consumed: u64::MAX,
            requested: 1,
        }
    );
    assert_eq!(
        ledger.usage().input_bytes,
        u64::MAX,
        "the total wrapped around instead of refusing"
    );
    assert_eq!(
        worker.usage().input_bytes,
        0,
        "the refused unit was billed to the worker"
    );
}

/// A permit taken before a cancellation is still billed, and the next checkpoint stops the work.
#[test]
fn a_permit_taken_before_a_cancellation_is_still_billed() {
    let entry = Budget::new(limits(8));
    let token = entry.cancellation_token();
    let ledger = OperationLedger::new(&entry);
    let mut worker = Budget::new(limits(8));
    worker.with_ledger(ledger.clone(), UsageOwner::Methods);

    worker
        .charge(CountedBudgetDimension::ResultItems, 3)
        .unwrap();
    token.cancel();
    assert_eq!(
        ledger.stop_reason(),
        Some(BulkStop {
            owner: None,
            kind: BulkStopKind::Cancelled,
            dimension: None,
        })
    );
    assert_eq!(
        ledger.usage().result_items,
        3,
        "work a permit admitted is not refunded by a later cancellation"
    );
    assert_eq!(ledger.cumulative(UsageOwner::Methods).result_items, 3);

    assert!(matches!(worker.poll(), Err(Error::Cancelled { .. })));
    assert!(matches!(
        worker.charge(CountedBudgetDimension::ResultItems, 1),
        Err(Error::Cancelled { .. })
    ));
    assert_eq!(
        ledger.usage().result_items,
        3,
        "the checkpoint billed work it refused"
    );
    assert_eq!(worker.usage().result_items, 3);
}

/// The direct single-request path is what it was: no ledger, no cache, the same numbers.
#[test]
fn a_budget_without_a_ledger_keeps_the_direct_path() {
    let mut budget = Budget::new(limits(3));
    assert!(budget.ledger().is_none());
    assert!(budget.facts_cache().is_none());

    budget.charge(CountedBudgetDimension::CodeBytes, 3).unwrap();
    assert_eq!(
        budget
            .charge(CountedBudgetDimension::CodeBytes, 1)
            .unwrap_err(),
        Error::BudgetExceeded {
            dimension: BudgetDimension::CodeBytes,
            limit: 3,
            consumed: 3,
            requested: 1,
        }
    );
    assert_eq!(budget.usage().code_bytes, 3);
    assert!(budget.poll().is_ok());
    assert!(
        budget.check(CountedBudgetDimension::CodeBytes, 0).is_ok(),
        "a probe that fits must still pass without a ledger"
    );

    // The owner a budget states is only meaningful while it bills to an operation, and stating one
    // there changes nothing.
    budget.set_owner(UsageOwner::Delivery);
    budget.charge(CountedBudgetDimension::CodeBytes, 0).unwrap();
    assert_eq!(budget.usage().code_bytes, 3);
}

/// One observer a test can hold: every entry and every refusal the total reported, in order.
///
/// This is what a caller of an operation can write against the seam, and it is how this file states
/// that the port sees what really happened rather than what a run happens to make convenient.
#[derive(Debug, Default)]
struct Census {
    entries: Mutex<Vec<(LedgerSite, bool)>>,
    refusals: Mutex<Vec<(LedgerSite, Option<BudgetDimension>)>>,
}

impl Census {
    fn entries(&self) -> Vec<(LedgerSite, bool)> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn refusals(&self) -> Vec<(LedgerSite, Option<BudgetDimension>)> {
        self.refusals
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    fn sampled(&self) -> usize {
        self.entries()
            .into_iter()
            .filter(|(_, sampled)| *sampled)
            .count()
    }
}

impl LedgerObserver for Census {
    fn entry(&self, site: LedgerSite, timing: Option<LedgerEntryTiming>) {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push((site, timing.is_some()));
    }

    fn refusal(&self, site: LedgerSite, dimension: Option<BudgetDimension>) {
        self.refusals
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push((site, dimension));
    }
}

/// The operation's total reports **every** entry into it, exactly once and in order, and an entry
/// carries a timing once in [`LEDGER_OBSERVATION_STRIDE`].
///
/// The counts are the exact ones a reader multiplies the sampled durations by, so a port that lost
/// its call sites — a `charge` that stopped reporting, a checkpoint that never reached the observer —
/// fails here rather than quietly publishing a smaller figure.
#[test]
fn the_total_reports_every_entry_and_the_observation_moves_no_number() {
    let mut headroom = limits(8);
    headroom.analysis_steps = 4_096;
    let entry = Budget::new(headroom);

    // The same work, once observed and once not: the observation is not part of any total.
    let unobserved = OperationLedger::new(&entry);
    for _ in 0..3 {
        unobserved
            .charge(
                UsageOwner::Methods,
                CountedBudgetDimension::AnalysisSteps,
                2,
            )
            .unwrap();
        unobserved.poll().unwrap();
    }
    unobserved
        .check_nested_depth(UsageOwner::Discovery, 2)
        .unwrap();

    let census = Arc::new(Census::default());
    let ledger = OperationLedger::new(&entry).with_observer(census.clone());
    for _ in 0..3 {
        ledger
            .charge(
                UsageOwner::Methods,
                CountedBudgetDimension::AnalysisSteps,
                2,
            )
            .unwrap();
        ledger.poll().unwrap();
    }
    ledger.check_nested_depth(UsageOwner::Discovery, 2).unwrap();
    assert_eq!(
        ledger.usage(),
        unobserved.usage(),
        "an observed total is the same total"
    );
    assert_eq!(
        ledger.cumulative(UsageOwner::Methods),
        unobserved.cumulative(UsageOwner::Methods)
    );

    let observed = census.entries();
    assert_eq!(
        observed.len(),
        7,
        "three charges, three checkpoints and one depth observation: {observed:?}"
    );
    assert_eq!(
        observed,
        vec![
            (LedgerSite::Charge(UsageOwner::Methods), false),
            (LedgerSite::Checkpoint, false),
            (LedgerSite::Charge(UsageOwner::Methods), false),
            (LedgerSite::Checkpoint, false),
            (LedgerSite::Charge(UsageOwner::Methods), false),
            (LedgerSite::Checkpoint, false),
            (LedgerSite::Depth(UsageOwner::Discovery), false),
        ],
        "every entry is reported once, in the order it happened, with its own site"
    );
    assert!(census.refusals().is_empty());

    // The stride is a stride: seven entries are not yet a sample, and the sixty-fourth is.
    let sampling_start = observed.len() as u64;
    for _ in sampling_start..LEDGER_OBSERVATION_STRIDE {
        ledger
            .charge(
                UsageOwner::Methods,
                CountedBudgetDimension::AnalysisSteps,
                1,
            )
            .unwrap();
    }
    assert_eq!(
        census.sampled(),
        1,
        "entry {} carries the first timing",
        LEDGER_OBSERVATION_STRIDE
    );
    for _ in LEDGER_OBSERVATION_STRIDE..LEDGER_OBSERVATION_STRIDE * 2 {
        ledger
            .charge(
                UsageOwner::Methods,
                CountedBudgetDimension::AnalysisSteps,
                1,
            )
            .unwrap();
    }
    assert_eq!(
        census.sampled(),
        2,
        "one timing per stride, not one per entry and not one per run"
    );
    assert_eq!(
        census.entries().len() as u64,
        LEDGER_OBSERVATION_STRIDE * 2,
        "the counts stay exact however few entries are timed"
    );
}

/// An entry budget's own depth high-water mark is part of the operation's total, exactly as its
/// counted dimensions are.
///
/// The refusals a per-dimension counter does not need are the two high-water marks: they are moved by
/// a maximum rather than by an addition, so an operation whose starting point already reached a depth
/// is the deepest of the entry and its own work — a fold that a total built from zero counters would
/// silently drop.
#[test]
fn the_entry_depths_are_part_of_the_operation_and_its_own_work_can_exceed_them() {
    let mut headroom = limits(8);
    headroom.nested_depth = 4;
    headroom.dependency_depth = 6;
    let mut entry = Budget::new(headroom);
    entry.check_nested_depth(4).unwrap();
    entry.observe_dependency_depth(6).unwrap();
    assert_eq!(entry.usage().nested_depth, 4);
    assert_eq!(entry.usage().dependency_depth, 6);

    let ledger = OperationLedger::new(&entry);
    assert_eq!(
        ledger.usage().nested_depth,
        4,
        "the operation starts from the depth the entry already reached"
    );
    assert_eq!(ledger.usage().dependency_depth, 6);
    assert_eq!(ledger.entry_usage().nested_depth, 4);

    // Its own work can reach deeper, and a shallower observation never lowers the mark.
    ledger.check_nested_depth(UsageOwner::Methods, 4).unwrap();
    ledger.check_nested_depth(UsageOwner::Discovery, 3).unwrap();
    assert_eq!(ledger.usage().nested_depth, 4);
    assert_eq!(
        ledger.cumulative(UsageOwner::Methods).nested_depth,
        4,
        "the owner's own mark is the deepest *it* accepted"
    );
    assert_eq!(ledger.cumulative(UsageOwner::Discovery).nested_depth, 3);
    // The two high-water dimensions are independent: neither observation touches the other.
    assert_eq!(ledger.usage().dependency_depth, 6);
}

/// A total's admission is atomic: eight workers hammer one exhausted dimension and exactly the
/// limit's worth of units is admitted, never one more.
///
/// The lock this total used to hold made that true by construction; a per-dimension counter has to
/// make it true by exchanging, and "two workers race for the last unit" is too narrow a window to
/// catch a lost exchange (`optimize-demand-workloads` 4.2 measured 10/10 passes against a
/// read-then-write admission). This test is written so that a lost exchange is very likely rather
/// than merely possible: eight threads each ask for one unit a thousand times against a limit of a
/// thousand, so every oversell the total allows is one the total states.
#[test]
fn a_hammered_dimension_admits_exactly_its_limit_and_never_one_more() {
    const WORKERS: usize = 8;
    const EACH: u64 = 1_000;
    const LIMIT: u64 = 1_000;
    let mut operation = limits(1 << 20);
    operation.analysis_steps = LIMIT;
    let entry = Budget::new(operation);
    let ledger = OperationLedger::new(&entry);

    let admitted = AtomicU64::new(0);
    let barrier = Barrier::new(WORKERS);
    std::thread::scope(|scope| {
        for _ in 0..WORKERS {
            let ledger = ledger.clone();
            let admitted = &admitted;
            let barrier = &barrier;
            scope.spawn(move || {
                let mut worker = Budget::new(limits(1 << 20));
                worker.with_ledger(ledger, UsageOwner::Methods);
                barrier.wait();
                for _ in 0..EACH {
                    if worker
                        .charge(CountedBudgetDimension::AnalysisSteps, 1)
                        .is_ok()
                    {
                        admitted.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
    });

    assert_eq!(
        admitted.load(Ordering::Relaxed),
        LIMIT,
        "a total admits exactly its limit of a dimension, whatever its workers do"
    );
    assert_eq!(
        ledger.usage().analysis_steps,
        LIMIT,
        "and states exactly that: {WORKERS} workers asked {EACH} times each for one unit of a \
         {LIMIT}-unit quota"
    );
    assert_eq!(
        ledger.cumulative(UsageOwner::Methods).analysis_steps,
        LIMIT,
        "every admitted unit belongs to the work that took it"
    );
}

/// An entry that could not take its permit is reported as the refusal it was, with the quota it
/// needed — and the entry that recorded the operation's stop is the first one that did.
#[test]
fn a_refused_entry_is_reported_with_the_quota_it_needed() {
    let mut operation = limits(64);
    operation.analysis_steps = 1;
    operation.result_items = 0;
    let entry = Budget::new(operation);
    let census = Arc::new(Census::default());
    let ledger = OperationLedger::new(&entry).with_observer(census.clone());

    ledger
        .charge(
            UsageOwner::Methods,
            CountedBudgetDimension::AnalysisSteps,
            1,
        )
        .unwrap();
    assert!(
        census.refusals().is_empty(),
        "an admitted entry is no refusal"
    );
    let refused = ledger
        .charge(
            UsageOwner::Methods,
            CountedBudgetDimension::AnalysisSteps,
            1,
        )
        .unwrap_err();
    assert!(matches!(
        refused,
        Error::BudgetExceeded {
            dimension: BudgetDimension::AnalysisSteps,
            ..
        }
    ));
    assert_eq!(
        census.refusals(),
        vec![(
            LedgerSite::Charge(UsageOwner::Methods),
            Some(BudgetDimension::AnalysisSteps)
        )]
    );
    assert_eq!(
        ledger.stop_reason().map(|stop| stop.kind),
        Some(BulkStopKind::Budget),
        "the refusal that was reported is the one that stopped the operation"
    );

    // The operation is stopped: the entries after it observe the stop the first refusal recorded,
    // and the report still names that first one.
    let _ = ledger.charge(UsageOwner::Delivery, CountedBudgetDimension::ResultItems, 1);
    assert_eq!(
        ledger.stop_reason().and_then(|stop| stop.owner),
        Some(UsageOwner::Methods),
        "a later observation does not replace the stop the first refusal recorded"
    );
}
