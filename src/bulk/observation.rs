//! One bulk operation's observation port: what a caller can read about how a run was coordinated.
//!
//! # Why a port rather than fields
//!
//! A bulk operation is one total, one window and N class tasks, and the question this port answers
//! — *how much of this run was the operation's own coordination rather than the work* — is a
//! question about a run, not a term of any contract the operation publishes. Nothing here is part of
//! [`BulkRecoveryReport`](crate::BulkRecoveryReport), of a stream event or of a stop record: the
//! report states what the operation did, this states what coordinating it cost. A caller that wants
//! the second asks for it ([`BulkRecoveryRequest::with_probe`](crate::BulkRecoveryRequest::with_probe))
//! and reads it afterwards ([`BulkProbe::reading`]).
//!
//! # What it costs the run it observes
//!
//! The port is compiled out of a normal build; in a build that has it, it is attached per operation,
//! so a run whose caller attached no probe takes the same paths with one `Option` test per
//! observation site. Where one is attached it is built to stay below the quantities it reports:
//!
//! * every ledger entry is counted **per thread** (the exact counts of [`LedgerObserver`]), because
//!   a shared counter on that path would be a second contended line on the very path being measured;
//! * only one entry in [`jarde_reader::ledger::LEDGER_OBSERVATION_STRIDE`] carries a timing, and a
//!   figure derived from those samples is an estimate the caller multiplies by the exact count;
//! * the window's waits are counted where they happen — the `wait_timeout` on the window's own
//!   signal — rather than by timing a whole call, so a coordinator's wait is a wait and not a
//!   delivery that happened to be slow.
//!
//! # What is exact and what is sampled
//!
//! Exact: every `calls` and `refusals` figure, every window wait figure, the class-task durations,
//! every worker's lifetime and busy time, and every delivery's duration. Sampled: `waited_nanos`,
//! `waited_max_nanos` and `held_nanos` of the ledger sites, one entry in
//! [`jarde_reader::ledger::LEDGER_OBSERVATION_STRIDE`] each. A sampled *sum* is
//! `sum(samples) / sampled × calls`, which is the estimate a reader of these numbers computes; the
//! port reports the sum over the samples and how many samples there were, so the estimate is the
//! caller's to make and to state.
//!
//! # Who flushes what
//!
//! The per-thread ledger tallies are added to the probe when the thread that took them says so
//! ([`BulkProbe::flush_this_thread`]), which the operation does for each worker as it returns and
//! for its coordinator before it ends. A thread that took entries and never flushed them — a worker
//! killed by a panic in the middle of a class task, say — is missing from the totals rather than
//! silently counted somewhere else.

use std::time::{Duration, Instant};

#[cfg(feature = "test-support")]
use std::sync::Arc;
#[cfg(feature = "test-support")]
use std::sync::atomic::{AtomicU64, Ordering};

/// Where a window wait happened: one class's own result slot, or the coordinator's own waits.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum WindowSite {
    /// The coordinator waiting on the window for the earliest active class: its next record, or the
    /// end of a class the operation closed on.
    TakeFront,
    /// A worker waiting for its own class's method-result slot to be taken.
    PlaceMethod,
    /// A worker waiting for its own class's control slot to be taken.
    PlaceControl,
    /// A worker waiting for the next class to be dispatched.
    TakeTask,
}

/// How many ledger sites the port keeps, in [`ledger_site_name`] order.
#[cfg(feature = "test-support")]
const LEDGER_SITES: usize = 6;

/// How many window sites the port keeps, in [`window_site_name`] order.
#[cfg(feature = "test-support")]
const WINDOW_SITES: usize = 4;

/// The name of one ledger site, in the order the port indexes them.
///
/// A name travels with every figure so a reader of the numbers does not have to know this module's
/// index arithmetic to say which counter it is reading.
#[cfg(feature = "test-support")]
fn ledger_site_name(index: usize) -> &'static str {
    [
        "charge_discovery",
        "charge_methods",
        "charge_delivery",
        "checkpoint",
        "probe",
        "depth",
    ][index]
}

/// The name of one window site, in the order the port indexes them.
#[cfg(feature = "test-support")]
fn window_site_name(index: usize) -> &'static str {
    ["take_front", "place_method", "place_control", "take_task"][index]
}

/// The port's own handle on one operation, as the operation's parts hold it.
///
/// Always compiled: the coordinator, a worker's class task and the delivery of a record ask the same
/// questions in either build, and a build without the port answers "nothing is watching" to all of
/// them. That is the reason the wrapper exists — no `cfg` sits in the middle of a class task, and no
/// observation call site is a second code path the shipped operation takes.
#[derive(Clone, Default)]
pub(crate) struct Observation {
    /// The caller's probe, when this build can have one and the caller attached it.
    #[cfg(feature = "test-support")]
    probe: Option<Arc<BulkProbe>>,
}

impl Observation {
    /// The observation of one operation, from the request's own probe.
    #[cfg(feature = "test-support")]
    pub(crate) fn of(probe: Option<Arc<BulkProbe>>) -> Self {
        Self { probe }
    }

    /// The observation of one operation in a build that cannot be given a probe.
    #[cfg(not(feature = "test-support"))]
    pub(crate) fn of() -> Self {
        Self {}
    }

    /// One wait for the window's signal is about to happen: the instant, when anything is watching.
    ///
    /// The answer is what [`Observation::waited`] accounts the wait with, and it is `None` in a
    /// build without the port, so the caller reads the same either way.
    pub(crate) fn wait_started(&self) -> Option<Instant> {
        #[cfg(feature = "test-support")]
        {
            self.probe.as_ref().map(|_| Instant::now())
        }
        #[cfg(not(feature = "test-support"))]
        {
            None
        }
    }

    /// One wait for the window's signal has ended, after `started`.
    pub(crate) fn waited(&self, _site: WindowSite, started: Option<Instant>) {
        #[cfg(feature = "test-support")]
        if let (Some(probe), Some(started)) = (self.probe.as_ref(), started) {
            probe.window_wait(_site, started.elapsed());
        }
        #[cfg(not(feature = "test-support"))]
        {
            let _ = started;
        }
    }

    /// One class task is about to run: the instant, when anything is watching.
    pub(crate) fn class_task_started(&self) -> Option<Instant> {
        #[cfg(feature = "test-support")]
        {
            self.probe.as_ref().map(|_| Instant::now())
        }
        #[cfg(not(feature = "test-support"))]
        {
            None
        }
    }

    /// One class task has ended, `started` ago.
    pub(crate) fn class_task_ended(&self, started: Option<Instant>) {
        #[cfg(feature = "test-support")]
        if let (Some(probe), Some(started)) = (self.probe.as_ref(), started) {
            probe.class_task(started.elapsed());
        }
        #[cfg(not(feature = "test-support"))]
        {
            let _ = started;
        }
    }

    /// One window call is about to happen on this site, waiting or not.
    pub(crate) fn window_call(&self, _site: WindowSite) {
        #[cfg(feature = "test-support")]
        if let Some(probe) = self.probe.as_ref() {
            probe.window_call(_site);
        }
        #[cfg(not(feature = "test-support"))]
        {
            let _ = _site;
        }
    }

    /// One prepared class declared `declared` method records.
    pub(crate) fn class_declared(&self, _declared: u64) {
        #[cfg(feature = "test-support")]
        if let Some(probe) = self.probe.as_ref() {
            probe.declared(_declared);
        }
        #[cfg(not(feature = "test-support"))]
        {
            let _ = _declared;
        }
    }

    /// One worker thread is about to run its loop: its own start instant, when anything is watching.
    pub(crate) fn worker_started(&self) -> Option<Instant> {
        #[cfg(feature = "test-support")]
        {
            self.probe.as_ref().map(|_| Instant::now())
        }
        #[cfg(not(feature = "test-support"))]
        {
            None
        }
    }

    /// One worker thread has returned, with how much of its life it spent inside class tasks.
    ///
    /// This is also where that thread's own ledger tallies are added: it is returning, and no other
    /// thread can see the counts it took.
    pub(crate) fn worker_ended(&self, started: Option<Instant>, busy: Duration) {
        #[cfg(feature = "test-support")]
        if let (Some(probe), Some(started)) = (self.probe.as_ref(), started) {
            probe.worker(started.elapsed(), busy);
            probe.flush_this_thread();
        }
        #[cfg(not(feature = "test-support"))]
        {
            let _ = (started, busy);
        }
    }

    /// One record is about to be handed over: the instant, when anything is watching.
    pub(crate) fn delivery_started(&self) -> Option<Instant> {
        #[cfg(feature = "test-support")]
        {
            self.probe.as_ref().map(|_| Instant::now())
        }
        #[cfg(not(feature = "test-support"))]
        {
            None
        }
    }

    /// One record was handled by the operation's delivery half: the charge, and the sink's answer.
    pub(crate) fn delivered(&self, started: Option<Instant>, in_sink: Option<Instant>) {
        #[cfg(feature = "test-support")]
        if let (Some(probe), Some(started), Some(in_sink)) = (self.probe.as_ref(), started, in_sink)
        {
            probe.delivery(started.elapsed(), in_sink.elapsed());
        }
        #[cfg(not(feature = "test-support"))]
        {
            let _ = (started, in_sink);
        }
    }

    /// Adds this thread's own ledger tallies to the probe's totals.
    pub(crate) fn flush(&self) {
        #[cfg(feature = "test-support")]
        if let Some(probe) = self.probe.as_ref() {
            probe.flush_this_thread();
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The probe itself: only compiled where a caller can be given one
// ---------------------------------------------------------------------------------------------

#[cfg(feature = "test-support")]
use crate::BudgetDimension;
#[cfg(feature = "test-support")]
use jarde_reader::ledger::{LedgerEntryTiming, LedgerObserver, LedgerSite, UsageOwner};

/// One operation's observation handle.
///
/// Built by the caller before the operation ([`BulkProbe::new`]), attached to the request
/// ([`BulkRecoveryRequest::with_probe`](crate::BulkRecoveryRequest::with_probe)) and read after it
/// ([`BulkProbe::reading`]). Reading one while its operation runs states what has been counted so
/// far and what the threads that already returned have flushed — it is not a snapshot of a run in
/// progress.
#[cfg(feature = "test-support")]
#[derive(Debug, Default)]
pub struct BulkProbe {
    ledger: [LedgerCounters; LEDGER_SITES],
    window: [WindowCounters; WINDOW_SITES],
    class_tasks: AtomicU64,
    class_task_nanos: AtomicU64,
    class_task_max_nanos: AtomicU64,
    declared_methods_max: AtomicU64,
    worker_threads: AtomicU64,
    worker_thread_nanos: AtomicU64,
    worker_busy_nanos: AtomicU64,
    records_delivered: AtomicU64,
    sink_nanos: AtomicU64,
    sink_max_nanos: AtomicU64,
    deliver_nanos: AtomicU64,
}

/// One run's ledger-side figures for one site.
///
/// `sampled` counts the entries of this site that carried a timing, and `waited_nanos` is the sum
/// over those samples: the estimate of the site's whole wait is `waited_nanos / sampled × calls`,
/// and a high-water figure read here is the largest wait *among the samples*.
#[cfg(feature = "test-support")]
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct LedgerSiteReading {
    /// The site's name, as this port indexes it.
    pub site: &'static str,
    /// Entries this site saw. Exact, however long the run.
    pub calls: u64,
    /// Entries that carried a timing.
    pub sampled: u64,
    /// Sampled entries that had to wait for the total at all.
    pub waited_calls: u64,
    /// Sum of the waits of the sampled entries.
    pub waited_nanos: u64,
    /// Largest wait among the sampled entries.
    pub waited_max_nanos: u64,
    /// Sum of the holds of the sampled entries: how long those entries took once they had the
    /// total.
    pub held_nanos: u64,
    /// Entries this site was refused.
    pub refusals: u64,
}

/// One run's window-side figures for one site.
///
/// The wait figures are the `wait_timeout` calls on the window's own signal and nothing else: a call
/// that found its record already waiting for it is a call and not a wait, and one call may wait
/// several times before its condition holds.
#[cfg(feature = "test-support")]
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct WindowSiteReading {
    /// The site's name, as this port indexes it.
    pub site: &'static str,
    /// Calls this site saw. Exact.
    pub calls: u64,
    /// Waits for the signal those calls made.
    pub waits: u64,
    /// Sum of the waits.
    pub wait_nanos: u64,
    /// Largest single wait.
    pub wait_max_nanos: u64,
}

/// Everything one operation's probe counted, as plain numbers.
#[cfg(feature = "test-support")]
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct BulkProbeReading {
    /// The operation's total, one row per ledger site.
    pub ledger: Vec<LedgerSiteReading>,
    /// The window and its waits, one row per site.
    pub window: Vec<WindowSiteReading>,
    /// Class tasks the operation ran, in either configuration.
    pub class_tasks: u64,
    /// Sum of the class tasks' durations.
    pub class_task_nanos: u64,
    /// The longest single class task.
    pub class_task_max_nanos: u64,
    /// The most method records any one prepared class declared: the run's own class skew.
    pub declared_methods_max: u64,
    /// Worker threads the operation created: zero for the serial configuration, which runs its class
    /// tasks on the calling thread.
    pub worker_threads: u64,
    /// Sum of the workers' lifetimes, from their own start to their return.
    pub worker_thread_nanos: u64,
    /// Sum of the workers' time *inside* class tasks. `worker_thread_nanos - worker_busy_nanos` is
    /// what the workers spent waiting for work or for a slot.
    pub worker_busy_nanos: u64,
    /// Records the operation handed to its sink, whatever the sink then did with them.
    pub records_delivered: u64,
    /// Sum of the sink callbacks' own durations.
    pub sink_nanos: u64,
    /// The longest single sink callback.
    pub sink_max_nanos: u64,
    /// Sum of the coordinator's whole delivery half: the record's charge on the total, and the sink
    /// callback.
    pub deliver_nanos: u64,
}

#[cfg(feature = "test-support")]
#[derive(Debug, Default)]
struct LedgerCounters {
    calls: AtomicU64,
    sampled: AtomicU64,
    waited_calls: AtomicU64,
    waited_nanos: AtomicU64,
    waited_max_nanos: AtomicU64,
    held_nanos: AtomicU64,
    refusals: AtomicU64,
}

#[cfg(feature = "test-support")]
#[derive(Debug, Default)]
struct WindowCounters {
    calls: AtomicU64,
    waits: AtomicU64,
    wait_nanos: AtomicU64,
    wait_max_nanos: AtomicU64,
}

/// One thread's ledger tallies, added to the probe when that thread flushes them.
///
/// `calls` and `refusals` are what make a site's counts exact: counting them in shared memory would
/// put one contended line per site on the path that runs once per charged item, which is the traffic
/// this port exists to measure rather than to add.
#[cfg(feature = "test-support")]
#[derive(Clone, Copy, Default)]
struct ThreadTally {
    calls: u64,
    sampled: u64,
    waited_calls: u64,
    waited_nanos: u64,
    waited_max_nanos: u64,
    held_nanos: u64,
    refusals: u64,
}

/// One thread's tallies with nothing in them, as a `const` so the thread local needs no runtime
/// initialization at all.
#[cfg(feature = "test-support")]
const EMPTY_TALLY: ThreadTally = ThreadTally {
    calls: 0,
    sampled: 0,
    waited_calls: 0,
    waited_nanos: 0,
    waited_max_nanos: 0,
    held_nanos: 0,
    refusals: 0,
};

#[cfg(feature = "test-support")]
thread_local! {
    static TALLY: std::cell::Cell<[ThreadTally; LEDGER_SITES]> =
        const { std::cell::Cell::new([EMPTY_TALLY; LEDGER_SITES]) };
}

/// The whole nanoseconds of one duration, saturating: a figure is a bound, not a wrap.
#[cfg(feature = "test-support")]
fn nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

/// Where one ledger site's figures are kept.
#[cfg(feature = "test-support")]
fn ledger_bucket(site: LedgerSite) -> usize {
    match site {
        LedgerSite::Charge(UsageOwner::Discovery) => 0,
        LedgerSite::Charge(UsageOwner::Methods) => 1,
        LedgerSite::Charge(UsageOwner::Delivery) => 2,
        LedgerSite::Checkpoint => 3,
        LedgerSite::Probe => 4,
        LedgerSite::Depth(_) => 5,
    }
}

/// Where one window site's figures are kept.
#[cfg(feature = "test-support")]
fn window_bucket(site: WindowSite) -> usize {
    match site {
        WindowSite::TakeFront => 0,
        WindowSite::PlaceMethod => 1,
        WindowSite::PlaceControl => 2,
        WindowSite::TakeTask => 3,
    }
}

#[cfg(feature = "test-support")]
impl BulkProbe {
    /// A probe with nothing counted yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds this thread's own ledger tallies to the probe's totals, and clears them.
    ///
    /// The operation calls this for every worker that returns and for its coordinator before it
    /// ends; a caller of a library operation never has to call it, and a stream that closes through
    /// a panic loses the tallies of the thread that panicked.
    pub fn flush_this_thread(&self) {
        TALLY.with(|tally| {
            let all = tally.replace([EMPTY_TALLY; LEDGER_SITES]);
            for (index, thread) in all.iter().enumerate() {
                let site = &self.ledger[index];
                site.calls.fetch_add(thread.calls, Ordering::Relaxed);
                site.sampled.fetch_add(thread.sampled, Ordering::Relaxed);
                site.waited_calls
                    .fetch_add(thread.waited_calls, Ordering::Relaxed);
                site.waited_nanos
                    .fetch_add(thread.waited_nanos, Ordering::Relaxed);
                site.waited_max_nanos
                    .fetch_max(thread.waited_max_nanos, Ordering::Relaxed);
                site.held_nanos
                    .fetch_add(thread.held_nanos, Ordering::Relaxed);
                site.refusals.fetch_add(thread.refusals, Ordering::Relaxed);
            }
        });
    }

    /// Everything this probe has counted.
    ///
    /// The counts of threads that have not flushed what they took are not in it: see the module
    /// documentation for who flushes when.
    pub fn reading(&self) -> BulkProbeReading {
        BulkProbeReading {
            ledger: (0..LEDGER_SITES)
                .map(|index| {
                    let site = &self.ledger[index];
                    LedgerSiteReading {
                        site: ledger_site_name(index),
                        calls: site.calls.load(Ordering::Relaxed),
                        sampled: site.sampled.load(Ordering::Relaxed),
                        waited_calls: site.waited_calls.load(Ordering::Relaxed),
                        waited_nanos: site.waited_nanos.load(Ordering::Relaxed),
                        waited_max_nanos: site.waited_max_nanos.load(Ordering::Relaxed),
                        held_nanos: site.held_nanos.load(Ordering::Relaxed),
                        refusals: site.refusals.load(Ordering::Relaxed),
                    }
                })
                .collect(),
            window: (0..WINDOW_SITES)
                .map(|index| {
                    let site = &self.window[index];
                    WindowSiteReading {
                        site: window_site_name(index),
                        calls: site.calls.load(Ordering::Relaxed),
                        waits: site.waits.load(Ordering::Relaxed),
                        wait_nanos: site.wait_nanos.load(Ordering::Relaxed),
                        wait_max_nanos: site.wait_max_nanos.load(Ordering::Relaxed),
                    }
                })
                .collect(),
            class_tasks: self.class_tasks.load(Ordering::Relaxed),
            class_task_nanos: self.class_task_nanos.load(Ordering::Relaxed),
            class_task_max_nanos: self.class_task_max_nanos.load(Ordering::Relaxed),
            declared_methods_max: self.declared_methods_max.load(Ordering::Relaxed),
            worker_threads: self.worker_threads.load(Ordering::Relaxed),
            worker_thread_nanos: self.worker_thread_nanos.load(Ordering::Relaxed),
            worker_busy_nanos: self.worker_busy_nanos.load(Ordering::Relaxed),
            records_delivered: self.records_delivered.load(Ordering::Relaxed),
            sink_nanos: self.sink_nanos.load(Ordering::Relaxed),
            sink_max_nanos: self.sink_max_nanos.load(Ordering::Relaxed),
            deliver_nanos: self.deliver_nanos.load(Ordering::Relaxed),
        }
    }

    /// One wait on the window's own signal, made by the call whose site this is.
    fn window_wait(&self, site: WindowSite, waited: Duration) {
        let counters = &self.window[window_bucket(site)];
        let waited = nanos(waited);
        counters.waits.fetch_add(1, Ordering::Relaxed);
        counters.wait_nanos.fetch_add(waited, Ordering::Relaxed);
        counters.wait_max_nanos.fetch_max(waited, Ordering::Relaxed);
    }

    /// One call to a window site, waiting or not.
    fn window_call(&self, site: WindowSite) {
        self.window[window_bucket(site)]
            .calls
            .fetch_add(1, Ordering::Relaxed);
    }

    /// One class task, with how long it ran.
    fn class_task(&self, ran: Duration) {
        let ran = nanos(ran);
        self.class_tasks.fetch_add(1, Ordering::Relaxed);
        self.class_task_nanos.fetch_add(ran, Ordering::Relaxed);
        self.class_task_max_nanos.fetch_max(ran, Ordering::Relaxed);
    }

    /// One prepared class's declared method count, for the run's own class skew.
    fn declared(&self, declared: u64) {
        self.declared_methods_max
            .fetch_max(declared, Ordering::Relaxed);
    }

    /// One worker thread's lifetime, and the part of it that ran class tasks.
    fn worker(&self, lived: Duration, busy: Duration) {
        self.worker_threads.fetch_add(1, Ordering::Relaxed);
        self.worker_thread_nanos
            .fetch_add(nanos(lived), Ordering::Relaxed);
        self.worker_busy_nanos
            .fetch_add(nanos(busy), Ordering::Relaxed);
    }

    /// One record's delivery: the charge and the sink callback together, and the callback alone.
    fn delivery(&self, whole: Duration, in_sink: Duration) {
        let sink = nanos(in_sink);
        self.records_delivered.fetch_add(1, Ordering::Relaxed);
        self.sink_nanos.fetch_add(sink, Ordering::Relaxed);
        self.sink_max_nanos.fetch_max(sink, Ordering::Relaxed);
        self.deliver_nanos
            .fetch_add(nanos(whole), Ordering::Relaxed);
    }
}

#[cfg(feature = "test-support")]
impl LedgerObserver for BulkProbe {
    fn entry(&self, site: LedgerSite, timing: Option<LedgerEntryTiming>) {
        let bucket = ledger_bucket(site);
        TALLY.with(|tally| {
            let mut all = tally.get();
            let thread = &mut all[bucket];
            thread.calls = thread.calls.saturating_add(1);
            if let Some(timing) = timing {
                let waited = nanos(timing.waited);
                thread.sampled = thread.sampled.saturating_add(1);
                thread.waited_calls = thread.waited_calls.saturating_add(u64::from(waited > 0));
                thread.waited_nanos = thread.waited_nanos.saturating_add(waited);
                thread.waited_max_nanos = thread.waited_max_nanos.max(waited);
                thread.held_nanos = thread.held_nanos.saturating_add(nanos(timing.held));
            }
            tally.set(all);
        });
    }

    fn refusal(&self, site: LedgerSite, _dimension: Option<BudgetDimension>) {
        let bucket = ledger_bucket(site);
        TALLY.with(|tally| {
            let mut all = tally.get();
            let thread = &mut all[bucket];
            thread.refusals = thread.refusals.saturating_add(1);
            tally.set(all);
        });
    }
}

#[cfg(all(test, feature = "test-support"))]
mod tests {
    use super::*;

    /// The site tables are the declared lists, and a reordered one would misattribute every figure.
    #[test]
    fn the_site_names_and_indexes_are_the_declared_order() {
        let ledger: Vec<&str> = (0..LEDGER_SITES).map(ledger_site_name).collect();
        assert_eq!(
            ledger,
            [
                "charge_discovery",
                "charge_methods",
                "charge_delivery",
                "checkpoint",
                "probe",
                "depth"
            ]
        );
        let window: Vec<&str> = (0..WINDOW_SITES).map(window_site_name).collect();
        assert_eq!(
            window,
            ["take_front", "place_method", "place_control", "take_task"]
        );
        // Every site the operation can report maps to one bucket, and the three owners keep their
        // own charge bucket rather than sharing one.
        let mut buckets = Vec::new();
        for owner in UsageOwner::ALL {
            buckets.push(ledger_bucket(LedgerSite::Charge(owner)));
        }
        buckets.push(ledger_bucket(LedgerSite::Checkpoint));
        buckets.push(ledger_bucket(LedgerSite::Probe));
        buckets.push(ledger_bucket(LedgerSite::Depth(UsageOwner::Methods)));
        buckets.sort_unstable();
        assert_eq!(buckets, (0..LEDGER_SITES).collect::<Vec<_>>());
        assert_eq!(
            [
                window_bucket(WindowSite::TakeFront),
                window_bucket(WindowSite::PlaceMethod),
                window_bucket(WindowSite::PlaceControl),
                window_bucket(WindowSite::TakeTask),
            ],
            [0, 1, 2, 3]
        );
    }

    /// A thread's tallies are added once, and a second flush adds nothing.
    #[test]
    fn a_threads_tallies_are_flushed_once() {
        let probe = BulkProbe::new();
        probe.entry(
            LedgerSite::Charge(UsageOwner::Methods),
            Some(LedgerEntryTiming {
                waited: Duration::from_nanos(30),
                held: Duration::from_nanos(40),
            }),
        );
        probe.entry(LedgerSite::Charge(UsageOwner::Methods), None);
        probe.refusal(LedgerSite::Charge(UsageOwner::Methods), None);
        assert_eq!(
            probe.reading().ledger[1].calls,
            0,
            "nothing is flushed before the thread says so"
        );
        probe.flush_this_thread();
        let reading = probe.reading();
        assert_eq!(reading.ledger[1].calls, 2);
        assert_eq!(reading.ledger[1].sampled, 1);
        assert_eq!(reading.ledger[1].waited_calls, 1);
        assert_eq!(reading.ledger[1].waited_nanos, 30);
        assert_eq!(reading.ledger[1].held_nanos, 40);
        assert_eq!(reading.ledger[1].refusals, 1);
        probe.flush_this_thread();
        assert_eq!(probe.reading().ledger[1].calls, 2);
    }

    /// The reading carries its site names, so a tool reading the numbers does not have to know this
    /// module's index arithmetic.
    #[test]
    fn the_reading_carries_its_site_names() {
        let probe = BulkProbe::new();
        probe.window_call(WindowSite::TakeFront);
        probe.window_wait(WindowSite::TakeFront, Duration::from_millis(2));
        let reading = probe.reading();
        assert_eq!(reading.window[0].site, "take_front");
        assert_eq!(reading.window[0].calls, 1);
        assert_eq!(reading.window[0].waits, 1);
        assert_eq!(reading.window[0].wait_nanos, 2_000_000);
        let json = serde_json::to_value(&reading).expect("a reading serializes");
        assert_eq!(json["window"][0]["site"], "take_front");
        assert_eq!(json["ledger"][0]["site"], "charge_discovery");
    }
}
