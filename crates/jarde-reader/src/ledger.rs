//! One operation's total: the shared ledger every worker of a bulk operation bills its work to.
//!
//! `add-parallel-bulk-recovery` decision 4 asks for **one shared total and per-method local
//! limits**: N workers of one operation must not be N requests that each start from zero, and a
//! method that runs out of its own allowance must not take the rest of the operation with it. This
//! module is that total. It sits next to [`Budget`] because the budget is the one handle every read
//! path already threads (`&mut Budget` and nothing else), and because the dimensions it sums are
//! this crate's own counted set.
//!
//! ## One operation, one ledger, N workers
//!
//! An [`OperationLedger`] is built **from** the budget that opened the operation
//! ([`OperationLedger::new`]) and is then attached to every budget that does the operation's work
//! ([`Budget::with_ledger`]). It is `Clone + Send + Sync`, and cloning it shares one total rather
//! than copying one: it belongs to *this* operation and is reachable only through the handles the
//! operation handed out. Nothing here is a process-wide singleton, and nothing here is attached by
//! [`Budget::new`], which keeps the direct single-request path the shape it was.
//!
//! The entry budget's own usage is **folded in as the operation's starting point** and is never
//! reset: bytes that were read to open the input keep counting against the operation's totals, so a
//! bulk operation cannot begin with a second, fresh allowance. A method that wants its own local
//! limits gets a budget of its own ([`Budget::new`] over the change's `method_limits`) with this
//! same ledger attached; the local budget's counters are the method's, the ledger's are the
//! operation's.
//!
//! ## Admission before work
//!
//! Every charge on a budget that carries a ledger does the same four things in the same order, and
//! this module is the third and fourth of them:
//!
//! 1. the operation's deadline and cancellation are checked, then the budget's own cancellation and
//!    deadline ([`Budget::poll`], which calls [`OperationLedger::poll`] first so an operation-level
//!    stop is what a worker reports);
//! 2. the budget's own limit and the checked addition are verified, so a local limit stops the local
//!    work before the operation is asked for anything;
//! 3. the operation's total is taken **atomically** ([`OperationLedger::charge`]): the limit, the
//!    checked addition and the booking of the owner's share happen as one exchange on that
//!    dimension's own counter;
//! 4. the budget records the local usage.
//!
//! A charge that fails before step 3 started nothing and is billed nowhere, and a charge that took
//! its permit is billed even if a cancellation arrives immediately afterwards: work that already had
//! its permit is not refunded, and the next checkpoint stops it. This version takes a permit per
//! action and pre-borrows no bulk quota, so the totals always state what really ran.
//!
//! ## One counter per dimension, one record for the stop
//!
//! The totals are atomic counters rather than one locked state, and they are counted one dimension
//! at a time because that is the shape of an operation's work: a parse charges bytes, a method
//! charges items and steps, a delivery charges records, and a worker charging items must take *that*
//! counter's permit rather than queue behind a worker charging bytes. One operation's totals are
//! therefore as many independent cells as it has dimensions — each on its own cache line, since two
//! counters sharing one would contend for every charge of both.
//!
//! What the counters cannot state is the operation's single first stop, and that is what the one
//! remaining lock holds: a stop is written once ([`BulkStop`]) and read by whoever wants the reason,
//! and the lock is what makes "written once" true when two workers refuse the same exhausted
//! dimension at the same instant. Recording a stop also cancels the operation's token, so a worker
//! whose next charge would still fit another dimension cannot observe a quota another worker
//! exhausted, and a worker waiting for capacity cannot observe a consumer that stopped. A *local*
//! refusal — a method's own limit, or a depth a single container needed — is deliberately **not**
//! recorded here: it stops that work and leaves the rest of the operation alone.
//!
//! Neither the deadline nor the cancellation is read under a lock: a checkpoint is one clock reading
//! and one atomic load of the caller's own token, and the entry that finds either records the stop.
//!
//! ## What adds up
//!
//! `total usage = entry usage + discovery + methods + delivery`, with the counted dimensions summed
//! and the high-water dimensions (`NestedDepth`, `DependencyDepth`) taking the maximum of everything
//! accepted. `elapsed_millis` is the operation's wall clock, measured once from the entry budget's
//! start: it is not the sum of the workers' times, and it is not recomputed per worker. The totals
//! are read from the ledger ([`OperationLedger::usage`], [`OperationLedger::cumulative`]); a
//! [`Budget`]'s own [`Budget::usage`] stays what it always was — the usage of *that* budget's work.

use crate::budget::{
    Budget, BudgetDimension, CancellationToken, CountedBudgetDimension, Limits, UsageSnapshot,
};
use crate::error::{Error, Result};
use crate::model::TerminationReason;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Which part of one operation a charge belongs to.
///
/// The three are the work classes design decision 4 names: discovering and preparing the scope,
/// executing method recovery, and delivering finished results to the consumer. They are *actual
/// work* classes, not new budget dimensions: every dimension keeps its meaning, and the owner only
/// says which part of the operation spent it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageOwner {
    /// Walking the physical scope, reading containers and preparing classes.
    Discovery,
    /// Executing method recovery: the analysis and recovery passes over one method.
    Methods,
    /// Encoding and handing a finished result to the consumer.
    Delivery,
}

impl UsageOwner {
    /// Every owner, in declaration order.
    pub const ALL: [Self; 3] = [Self::Discovery, Self::Methods, Self::Delivery];

    /// The position of one owner in [`Self::ALL`], which is also its slot in the ledger's state.
    const fn index(self) -> usize {
        match self {
            Self::Discovery => 0,
            Self::Methods => 1,
            Self::Delivery => 2,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Observation: how much of a run the operation's own accounting took
// ---------------------------------------------------------------------------------------------

/// Where one entry into the operation's accounting happened.
///
/// The four sites are the four ways a part of the operation reaches its one total, and they are
/// kept apart because they cost the operation differently: a charge moves a counted dimension and
/// books it to an owner, the two read-only sites answer the deadline, the cancellation and a
/// "would this fit" question, and a depth observation moves a high-water dimension instead of a
/// counted one.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LedgerSite {
    /// One charge of one counted dimension, as one part of the operation's work.
    Charge(UsageOwner),
    /// One checkpoint: the operation's deadline and cancellation, taken before a unit of work.
    Checkpoint,
    /// One probe ([`OperationLedger::check`]): whether a dimension would still fit, taking nothing.
    Probe,
    /// One depth observation, which moves `NestedDepth`/`DependencyDepth` rather than a counter.
    Depth(UsageOwner),
}

impl LedgerSite {
    /// The part of the operation whose work the entry belongs to, when it belongs to one.
    pub const fn owner(self) -> Option<UsageOwner> {
        match self {
            Self::Charge(owner) | Self::Depth(owner) => Some(owner),
            Self::Checkpoint | Self::Probe => None,
        }
    }
}

/// What one observed entry cost the accounting that took it.
///
/// Two durations rather than one, because they answer different questions: `waited` is how long the
/// entry had to wait for the right to change the total (a contention figure — it is zero when the
/// entry took it immediately), and `held` is how long the entry then took to make its change (a
/// cost figure, the serial share of that entry).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LedgerEntryTiming {
    /// How long the entry waited for the right to change the total.
    pub waited: Duration,
    /// How long the entry then held that right.
    pub held: Duration,
}

/// How many entries pass between two entries whose timing an observer is told.
///
/// Timing every entry would cost the operation more than the accounting it measures: two clock
/// readings per charge are the same order as the charge itself, and a seam that changes the number
/// it reports is not a measurement. One entry in this stride carries a [`LedgerEntryTiming`]
/// instead, which leaves a figure an observer multiplies by the exact call count — an estimate, and
/// one the observer states as such.
pub const LEDGER_OBSERVATION_STRIDE: u64 = 64;

thread_local! {
    /// This thread's own stride counter, so the sampling decision costs no shared memory.
    ///
    /// One counter per thread rather than one for the operation: an atomic counter every worker
    /// incremented would be a second contended line on the very path this seam exists to measure.
    static ENTRY_TICK: Cell<u64> = const { Cell::new(0) };
}

/// Whether *this* entry is one the attached observer is told the timing of.
fn observing_now() -> bool {
    ENTRY_TICK.with(|tick| {
        let next = tick.get().wrapping_add(1);
        tick.set(next);
        next % LEDGER_OBSERVATION_STRIDE == 0
    })
}

/// One handle that sees every entry into an operation's accounting.
///
/// The total is the one thing every part of a parallel operation shares, so "how much of this run
/// was the total itself" is a question about a run rather than a term of any contract this crate
/// publishes. This seam is how that question is answered without the answer becoming part of an
/// answer: no report, no stop record and no usage figure carries an observation, nothing in this
/// crate attaches an observer, and an observer is attached by the caller of an operation
/// ([`OperationLedger::with_observer`]) rather than by [`OperationLedger::new`]. A ledger nobody
/// observes runs the same paths in the same order and pays one `Option` test per entry.
///
/// What an implementor may rely on:
///
/// * every entry is reported exactly once, on the thread that took it, before that entry returns;
/// * the counts are exact, and the durations are *sampled* — one entry in
///   [`LEDGER_OBSERVATION_STRIDE`] carries a [`LedgerEntryTiming`], the others carry `None`;
/// * an entry that prefers to change nothing (a `Probe`, an accepted high-water depth) is still an
///   entry, because it is still what the operation's other parts wait behind.
pub trait LedgerObserver: std::fmt::Debug + Send + Sync {
    /// One entry into the total, as it happened.
    fn entry(&self, site: LedgerSite, timing: Option<LedgerEntryTiming>);

    /// One entry the total **refused**: the entry took nothing.
    ///
    /// The first refusal of an operation is the one that records its stop ([`BulkStop`]) and the
    /// refusals after it are entries that observed the stop the first one recorded; both are
    /// reported, and neither replaces the other. `dimension` is the quota the entry needed, or
    /// `None` when it was the operation's cancellation rather than a dimension that refused it.
    fn refusal(&self, site: LedgerSite, dimension: Option<BudgetDimension>);
}

/// Why an operation stopped.
///
/// The kind says who decided to stop, not which terminal state the aggregate report publishes: the
/// report's own vocabulary ([`TerminationReason`], `ExecutionReport`) is built from this record plus
/// the method results, and the mapping from a kind to `Complete`/`Partial`/`Cancelled`/`Failed` is
/// the aggregate's, stated in the change's design decision 6.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkStopKind {
    /// The operation's total quota ran out, or its deadline expired.
    Budget,
    /// Cooperative cancellation: the caller's token, or [`OperationLedger::cancel`].
    Cancelled,
    /// The consumer stopped accepting records.
    Sink,
    /// Infrastructure the operation cannot continue past: a worker that could not be created or did
    /// not return, or a consumer call that failed.
    Infrastructure,
}

/// The first stop an operation observed: what happened, and where it was seen.
///
/// The record is published once and never overwritten, so it is the primary reason a report can
/// name even when many workers observe the same stop afterwards. It is serialized as it stands, for
/// the same reason every other record of this crate is: a report that names a reason has to be
/// readable by whoever consumed the run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BulkStop {
    /// The part of the operation that observed the stop, when owned work did: a charge or a depth
    /// check of one of the three owners. A stop the operation itself recorded — the entry's
    /// deadline, an explicit cancellation, a consumer that stopped — has none.
    pub owner: Option<UsageOwner>,
    pub kind: BulkStopKind,
    /// The dimension whose quota ran out. Only a budget stop names one: a quota that could not be
    /// taken names the dimension it needed (or `ElapsedMillis`, for the deadline).
    pub dimension: Option<BudgetDimension>,
}

impl BulkStop {
    /// The stop in the [`TerminationReason`] vocabulary an execution report carries.
    ///
    /// A budget stop is the one that is also a reason in that vocabulary. Cancellation and a
    /// consumer stop are stated by `ExecutionReport::Cancelled`, and an infrastructure failure
    /// carries the reason of the call that failed rather than of this record, so those return
    /// `None` here instead of inventing a code for a cause nobody reported.
    pub fn termination(&self) -> Option<TerminationReason> {
        match self.kind {
            BulkStopKind::Budget => self
                .dimension
                .map(|dimension| TerminationReason::BudgetExceeded { dimension }),
            BulkStopKind::Cancelled | BulkStopKind::Sink | BulkStopKind::Infrastructure => None,
        }
    }
}

/// How many counted dimensions one operation's total keeps a counter for.
const COUNTERS: usize = CountedBudgetDimension::ALL.len();

/// The position of one counted dimension in the operation's own counters.
///
/// Exhaustive, so a dimension added to [`CountedBudgetDimension`] does not compile until it has a
/// counter here, and `CountedBudgetDimension::ALL` is the same list in the same order (a unit test
/// in [`crate::budget`] pins that order for the crate's own tables).
const fn dimension_index(dimension: CountedBudgetDimension) -> usize {
    match dimension {
        CountedBudgetDimension::InputBytes => 0,
        CountedBudgetDimension::ArchiveEntries => 1,
        CountedBudgetDimension::EntryBytes => 2,
        CountedBudgetDimension::ReadBytes => 3,
        CountedBudgetDimension::ClassBytes => 4,
        CountedBudgetDimension::AttributeBytes => 5,
        CountedBudgetDimension::CodeBytes => 6,
        CountedBudgetDimension::ResultItems => 7,
        CountedBudgetDimension::OutputBytes => 8,
        CountedBudgetDimension::ClassHeaders => 9,
        CountedBudgetDimension::MethodBodies => 10,
        CountedBudgetDimension::IrItems => 11,
        CountedBudgetDimension::IrEdges => 12,
        CountedBudgetDimension::AnalysisSteps => 13,
        CountedBudgetDimension::NormalizationClones => 14,
    }
}

/// One counter, alone on a cache line.
///
/// The counted dimensions of a bulk operation are charged by different parts of it — bytes by
/// discovery, items and steps by every method — and two counters sharing a line would contend for
/// every one of those charges. Sixty counters of sixty-four bytes are the whole price of that, and
/// this is the one place in an operation where a shared line is the thing being avoided.
#[derive(Debug, Default)]
#[repr(align(64))]
struct Counter(AtomicU64);

impl Counter {
    /// A counter that already holds `value`: the operation's starting point for that dimension.
    fn holding(value: u64) -> Self {
        Self(AtomicU64::new(value))
    }

    fn load(&self) -> u64 {
        self.0.load(Ordering::Acquire)
    }

    fn add(&self, requested: u64) -> u64 {
        self.0.fetch_add(requested, Ordering::AcqRel)
    }

    /// Adds `requested` when the total stays within `limit`, and answers whether it did.
    ///
    /// The exchange is the admission: the read, the checked addition, the limit and the write are
    /// one atomic step, so two workers adding to the same dimension cannot both see room for one
    /// unit of it. The answer is the value the counter had before the entry, which is what a refusal
    /// reports as `consumed` and what a success states it charged from.
    fn admit(&self, requested: u64, limit: u64) -> (bool, u64) {
        let mut current = self.load();
        loop {
            let admitted = match current.checked_add(requested) {
                Some(total) if total <= limit => total,
                _ => return (false, current),
            };
            match self.0.compare_exchange_weak(
                current,
                admitted,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return (true, current),
                Err(observed) => current = observed,
            }
        }
    }

    /// Keeps the deepest value this dimension reached.
    fn keep_deepest(&self, depth: u64) {
        self.0.fetch_max(depth, Ordering::AcqRel);
    }
}

/// Everything one operation's total is made of.
///
/// The counters are atomic and independent because that is what the operation charges: one counted
/// dimension at a time, by one part of the operation at a time, and a worker must take *that*
/// dimension's permit rather than queue behind a worker charging another one. What does not change
/// after the operation begins — its limits, its clock, the caller's token and the usage it started
/// from — is plain data beside them, read without a lock for the same reason.
///
/// One mutex remains, and it holds one thing: the operation's first stop. A stop is written once and
/// read by whatever wants the reason, and a lock is what makes "written once" true when two workers
/// refuse the same exhausted dimension at the same instant.
#[derive(Debug)]
struct LedgerShared {
    /// The operation's own limits, taken from the entry budget when the operation began. They are
    /// the *total* limits: a worker's own budget keeps its own local ones.
    limits: Limits,
    /// The instant the operation's clock started: the entry budget's own start, so the deadline
    /// covers discovery, waiting and delivery alike.
    started_at: Instant,
    /// The entry budget's cancellation token, so a caller that cancels the request stops every
    /// worker of the operation through the handle it already holds.
    cancellation: CancellationToken,
    /// The usage the operation started from: the entry budget's own usage when the ledger was
    /// built, or the total of the ledger that budget already billed to. Never reset.
    entry: UsageSnapshot,
    /// The operation's totals: [`Self::entry`] plus everything admitted since, one counter per
    /// counted dimension.
    totals: [Counter; COUNTERS],
    /// What each owner has been charged, in [`UsageOwner::ALL`] order, one counter per counted
    /// dimension, plus each owner's own depth high-water marks.
    owners: [[Counter; COUNTERS]; 3],
    /// The deepest container nesting the operation accepted, and the deepest the dependency closure
    /// reached. High-water dimensions, moved by a maximum rather than by an addition.
    nested_depth: Counter,
    dependency_depth: Counter,
    owners_nested_depth: [Counter; 3],
    owners_dependency_depth: [Counter; 3],
    /// The first stop anyone observed. Later observations do not replace it.
    stop: Mutex<Option<BulkStop>>,
}

impl LedgerShared {
    /// The operation's total of one counted dimension.
    fn total(&self, dimension: CountedBudgetDimension) -> u64 {
        self.totals[dimension_index(dimension)].load()
    }

    /// One owner's own share of one counted dimension.
    fn owner_total(&self, owner: UsageOwner, dimension: CountedBudgetDimension) -> u64 {
        self.owners[owner.index()][dimension_index(dimension)].load()
    }

    /// The operation's totals right now, with the depth high-water marks taken over all of them.
    ///
    /// The counted dimensions are read one at a time, so this is a reading of the counters as they
    /// were while it ran rather than a photograph of one instant: a charge in flight may be in the
    /// total and not yet in its owner's share. Every consumer of this figure reads it from a
    /// quiescent operation — the report, after every worker returned — where the two agree exactly.
    fn snapshot(&self) -> UsageSnapshot {
        let mut usage = UsageSnapshot::default();
        for dimension in CountedBudgetDimension::ALL {
            usage.set(dimension, self.total(dimension));
        }
        usage.nested_depth = self.nested_depth.load();
        usage.dependency_depth = self.dependency_depth.load();
        usage
    }

    /// One owner's own totals: what it was charged, and the deepest depths it accepted.
    fn owner_snapshot(&self, owner: UsageOwner) -> UsageSnapshot {
        let mut usage = UsageSnapshot::default();
        for dimension in CountedBudgetDimension::ALL {
            usage.set(dimension, self.owner_total(owner, dimension));
        }
        usage.nested_depth = self.owners_nested_depth[owner.index()].load();
        usage.dependency_depth = self.owners_dependency_depth[owner.index()].load();
        usage
    }

    /// Records `stop` when the operation has none yet, and makes it reach every worker.
    ///
    /// The token is cancelled together with the record: a worker whose next charge is of another
    /// dimension, or a worker waiting for capacity, has no other way to observe the stop. The first
    /// record is what the report publishes, so the observations that follow it — every later
    /// checkpoint sees the cancelled token — cannot move it.
    fn record(&self, stop: BulkStop) {
        let mut held = self
            .stop
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if held.is_none() {
            *held = Some(stop);
            self.cancellation.cancel();
        }
    }

    /// The operation's first stop, when it has one.
    fn recorded(&self) -> Option<BulkStop> {
        self.stop
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

/// The elapsed wall clock since `started_at`, in whole milliseconds.
///
/// A clock that is read once per reading and never accumulated, which is what makes
/// `elapsed_millis` an operation duration instead of a sum of worker durations.
fn elapsed_millis(started_at: Instant) -> u64 {
    started_at.elapsed().as_millis().min(u64::MAX as u128) as u64
}

/// The error a cancelled operation reports to the work it stops.
///
/// The same wording [`Budget`] uses for its own token: a `Cancelled` error means one thing in this
/// crate, and the reason an operation stopped is [`OperationLedger::stop_reason`].
fn cancelled() -> Error {
    Error::Cancelled {
        reason: "cooperative cancellation requested".into(),
    }
}

/// Which of the two high-water dimensions a depth observation moves.
#[derive(Clone, Copy)]
enum HighWater {
    Nested,
    Dependency,
}

/// One operation's total, shared by every worker of that operation.
///
/// Built once per operation from the budget that carries the operation's total limits
/// ([`OperationLedger::new`]) and attached to the budgets that do the work
/// ([`Budget::with_ledger`]). Cloning a handle shares the one total; see the module documentation
/// for what that total guarantees.
#[derive(Clone, Debug)]
pub struct OperationLedger {
    shared: Arc<LedgerShared>,
    /// The caller's own observation handle, when it attached one. Absent on every path this crate
    /// builds; see [`OperationLedger::with_observer`].
    observer: Option<Arc<dyn LedgerObserver>>,
}

impl OperationLedger {
    /// The operation's ledger, starting from `entry`'s own usage and limits.
    ///
    /// The entry budget's usage becomes the operation's starting point — the bytes already read to
    /// open the input keep counting — and its clock becomes the operation's clock, so the deadline
    /// covers the whole operation rather than each worker's own share of it. Its cancellation token
    /// becomes the operation's, which is how a caller that cancels the request stops every worker.
    ///
    /// A budget that already bills to a ledger is a request whose operation is still running (the
    /// same `&mut Budget` is the operation's total). Building the next operation's ledger from it
    /// therefore starts from **that ledger's whole total**, not from the entry budget's own usage:
    /// otherwise a second operation on one request would be handed the same quota twice.
    pub fn new(entry: &Budget) -> Self {
        let (baseline, started_at, cancellation) = match entry.ledger() {
            Some(previous) => (
                previous.shared.snapshot(),
                previous.shared.started_at,
                previous.shared.cancellation.clone(),
            ),
            None => (
                entry.usage(),
                entry.started_at(),
                entry.cancellation_token(),
            ),
        };
        // `elapsed_millis` is a reading of the operation's clock, and the operation's clock is what
        // `usage`/`cumulative` report: the fold point itself carries no duration.
        let mut start = baseline;
        start.elapsed_millis = 0;
        let mut totals: [Counter; COUNTERS] = std::array::from_fn(|_| Counter::default());
        for dimension in CountedBudgetDimension::ALL {
            totals[dimension_index(dimension)] = Counter::holding(start.counted_usage(dimension));
        }
        Self {
            shared: Arc::new(LedgerShared {
                limits: entry.limits().clone(),
                started_at,
                cancellation,
                // The depth marks the operation starts from are the entry budget's own: a container
                // nesting the entry accepted is part of the total this operation reports, exactly as
                // its counted dimensions are.
                nested_depth: Counter::holding(start.nested_depth),
                dependency_depth: Counter::holding(start.dependency_depth),
                entry: start,
                totals,
                owners: std::array::from_fn(|_| std::array::from_fn(|_| Counter::default())),
                owners_nested_depth: std::array::from_fn(|_| Counter::default()),
                owners_dependency_depth: std::array::from_fn(|_| Counter::default()),
                stop: Mutex::new(None),
            }),
            observer: None,
        }
    }

    /// The same total, reporting every entry into it to `observer`.
    ///
    /// Opt-in, caller-owned and one-operation-scoped: nothing in this crate attaches an observer, a
    /// total nobody observes takes the same paths in the same order, and the observation is never
    /// part of a total, a stop record or a report. See [`LedgerObserver`] for what an implementor is
    /// told and what it may rely on.
    pub fn with_observer(mut self, observer: Arc<dyn LedgerObserver>) -> Self {
        self.observer = Some(observer);
        self
    }

    /// The operation-level checks every entry runs before it does anything: cancellation, then the
    /// deadline. Both belong to the operation, and both are what a worker has to observe.
    ///
    /// Neither is read under a lock: the cancellation is the caller's own token and the deadline is
    /// one clock reading, and a worker checking them is not a reason to queue behind another
    /// worker's charge. The entry that *finds* one records the stop, which is the one part that
    /// needs the operation's single record to be written once.
    fn checkpoint(&self) -> Result<()> {
        if self.shared.cancellation.is_cancelled() {
            self.shared.record(BulkStop {
                owner: None,
                kind: BulkStopKind::Cancelled,
                dimension: None,
            });
            return Err(cancelled());
        }
        let elapsed = elapsed_millis(self.shared.started_at);
        if elapsed >= self.shared.limits.elapsed_millis {
            self.shared.record(BulkStop {
                owner: None,
                kind: BulkStopKind::Budget,
                dimension: Some(BudgetDimension::ElapsedMillis),
            });
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
                limit: self.shared.limits.elapsed_millis,
                consumed: elapsed,
                requested: 0,
            });
        }
        Ok(())
    }

    /// One charge: the total's own admission of `dimension`, then the owner's share of it.
    ///
    /// The admission is one atomic exchange on that dimension's counter, so two workers charging it
    /// at the same instant cannot both see room for the same unit. The owner's share is booked after
    /// it and cannot wrap: an owner's share is never above the total's — every charge does both in
    /// this order and nothing else moves either counter — and the total was just bounded by the
    /// limit, which is a `u64`.
    fn charge_entry(
        &self,
        owner: UsageOwner,
        dimension: CountedBudgetDimension,
        requested: u64,
    ) -> Result<()> {
        self.checkpoint()?;
        let limit = self.shared.limits.counted_limit(dimension);
        let (admitted, consumed) =
            self.shared.totals[dimension_index(dimension)].admit(requested, limit);
        if !admitted {
            // A refusal of the operation's own quota is the operation's stop: work that could not
            // take its permit neither starts nor is billed, and the first refusal is what later
            // observation cannot replace.
            self.shared.record(BulkStop {
                owner: Some(owner),
                kind: BulkStopKind::Budget,
                dimension: Some(dimension.into()),
            });
            return Err(Error::BudgetExceeded {
                dimension: dimension.into(),
                limit,
                consumed,
                requested,
            });
        }
        self.shared.owners[owner.index()][dimension_index(dimension)].add(requested);
        Ok(())
    }

    /// One depth observation.
    ///
    /// A refusal here records nothing: a container deeper than the operation allows fails *that
    /// container's* traversal, so recording it would publish a stop the operation did not take.
    fn depth_entry(&self, owner: UsageOwner, depth: u64, high_water: HighWater) -> Result<()> {
        self.checkpoint()?;
        let (limit, dimension) = match high_water {
            HighWater::Nested => (
                self.shared.limits.nested_depth,
                BudgetDimension::NestedDepth,
            ),
            HighWater::Dependency => (
                self.shared.limits.dependency_depth,
                BudgetDimension::DependencyDepth,
            ),
        };
        if depth > limit {
            return Err(Error::BudgetExceeded {
                dimension,
                limit,
                consumed: depth.saturating_sub(1),
                requested: 1,
            });
        }
        let (total, owner_slot) = match high_water {
            HighWater::Nested => (
                &self.shared.nested_depth,
                &self.shared.owners_nested_depth[owner.index()],
            ),
            HighWater::Dependency => (
                &self.shared.dependency_depth,
                &self.shared.owners_dependency_depth[owner.index()],
            ),
        };
        total.keep_deepest(depth);
        owner_slot.keep_deepest(depth);
        Ok(())
    }

    /// One "would this fit" probe: the same arithmetic a charge runs, read-only.
    ///
    /// A probe refuses no work, so it records no stop and changes nothing.
    fn probe_entry(&self, dimension: CountedBudgetDimension, requested: u64) -> Result<()> {
        if self.shared.cancellation.is_cancelled() {
            return Err(cancelled());
        }
        let elapsed = elapsed_millis(self.shared.started_at);
        if elapsed >= self.shared.limits.elapsed_millis {
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
                limit: self.shared.limits.elapsed_millis,
                consumed: elapsed,
                requested: 0,
            });
        }
        let limit = self.shared.limits.counted_limit(dimension);
        let consumed = self.shared.total(dimension);
        match consumed.checked_add(requested) {
            Some(total) if total <= limit => Ok(()),
            _ => Err(Error::BudgetExceeded {
                dimension: dimension.into(),
                limit,
                consumed,
                requested,
            }),
        }
    }

    /// Runs one entry into the total, reporting it to the observer this ledger was given.
    ///
    /// Every entry is reported, which is what makes an observer's counts exact; the timing is one
    /// entry in [`LEDGER_OBSERVATION_STRIDE`]. The timing is the **whole** entry — this
    /// implementation's exclusion is the exchange itself, so an entry does not wait for the total and
    /// then hold it, it takes it once — which is why `waited` is zero here and `held` carries the
    /// entry's whole cost.
    fn observed<T>(&self, site: LedgerSite, work: impl FnOnce() -> T) -> T {
        if self.observer.is_none() {
            return work();
        }
        if !observing_now() {
            let outcome = work();
            self.report(site, None);
            return outcome;
        }
        let started = Instant::now();
        let outcome = work();
        let held = started.elapsed();
        self.report(
            site,
            Some(LedgerEntryTiming {
                waited: Duration::ZERO,
                held,
            }),
        );
        outcome
    }

    /// Reports one entry to the attached observer, with its timing when it was a sampled one.
    fn report(&self, site: LedgerSite, timing: Option<LedgerEntryTiming>) {
        if let Some(observer) = self.observer.as_ref() {
            observer.entry(site, timing);
        }
    }

    /// Reports one entry the total refused, with the quota it needed.
    fn refuse(&self, site: LedgerSite, outcome: &Result<()>) {
        let (Some(observer), Err(error)) = (self.observer.as_ref(), outcome) else {
            return;
        };
        let dimension = match error {
            Error::BudgetExceeded { dimension, .. } => Some(*dimension),
            _ => None,
        };
        observer.refusal(site, dimension);
    }

    /// Takes `requested` units of `dimension` for `owner`'s work, or refuses without taking any.
    ///
    /// The operation-level checks run first, then the total's limit and checked addition are
    /// evaluated against that dimension's total **as one atomic step**, and the owner's share is
    /// booked immediately after it. A refusal is therefore a refusal of the operation's own quota —
    /// and it is the operation's stop: work that could not take its permit neither starts nor is
    /// billed, and the first refused charge is what later observation cannot replace.
    ///
    /// An overflow is reported as an exceeded limit rather than wrapped: a `consumed + requested`
    /// that does not fit in `u64` is refused with the numbers that produced it.
    pub fn charge(
        &self,
        owner: UsageOwner,
        dimension: CountedBudgetDimension,
        requested: u64,
    ) -> Result<()> {
        let site = LedgerSite::Charge(owner);
        let outcome = self.observed(site, || self.charge_entry(owner, dimension, requested));
        self.refuse(site, &outcome);
        outcome
    }

    /// Whether the operation could still take `requested` units of `dimension`, without taking them.
    ///
    /// The same arithmetic [`OperationLedger::charge`] performs, read-only and without recording
    /// anything: this is the "would this fit" question [`Budget::check`] asks, and a probe that
    /// refuses nothing is not a stop.
    pub(crate) fn check(&self, dimension: CountedBudgetDimension, requested: u64) -> Result<()> {
        let site = LedgerSite::Probe;
        let outcome = self.observed(site, || self.probe_entry(dimension, requested));
        self.refuse(site, &outcome);
        outcome
    }

    /// The container nesting depth the operation accepted, enforcing the operation's `NestedDepth`.
    ///
    /// A high-water dimension, exactly as [`Budget::check_nested_depth`] states it: the deepest
    /// accepted depth is kept, and a deeper one is refused with `consumed = depth - 1`.
    ///
    /// A refusal here is deliberately **not** the operation's stop. A container deeper than the
    /// operation allows fails *that container's* traversal — discovery records it, keeps the
    /// remainder of the scope unknown and continues — so recording it would publish a stop the
    /// operation did not take. The operation's totals still take the deepest depth accepted.
    pub fn check_nested_depth(&self, owner: UsageOwner, depth: u64) -> Result<()> {
        let site = LedgerSite::Depth(owner);
        let outcome = self.observed(site, || self.depth_entry(owner, depth, HighWater::Nested));
        self.refuse(site, &outcome);
        outcome
    }

    /// The dependency-closure depth the operation reached, enforcing the operation's
    /// `DependencyDepth`.
    ///
    /// Same high-water shape and the same refusal rule as
    /// [`OperationLedger::check_nested_depth`]: the deepest accepted depth is kept, a deeper one is
    /// refused, and neither dimension touches the other's value.
    pub fn observe_dependency_depth(&self, owner: UsageOwner, depth: u64) -> Result<()> {
        let site = LedgerSite::Depth(owner);
        let outcome = self.observed(site, || {
            self.depth_entry(owner, depth, HighWater::Dependency)
        });
        self.refuse(site, &outcome);
        outcome
    }

    /// The operation's deadline and cancellation, without accounting anything.
    ///
    /// This is the operation-level half of a checkpoint: a worker calls it (through its own budget)
    /// before it starts a unit of work, and the first observation of an expired deadline or a
    /// cancellation is recorded as the operation's stop.
    pub fn poll(&self) -> Result<()> {
        let site = LedgerSite::Checkpoint;
        let outcome = self.observed(site, || self.checkpoint());
        self.refuse(site, &outcome);
        outcome
    }

    /// The operation's totals right now: the entry usage plus every owner's work, with the depth
    /// high-water marks taken over all of them and the wall clock measured once from the start.
    pub fn usage(&self) -> UsageSnapshot {
        let mut usage = self.shared.snapshot();
        usage.elapsed_millis = elapsed_millis(self.shared.started_at);
        usage
    }

    /// The usage the operation started from: the entry budget's own usage when the ledger was built,
    /// or the total of the ledger that budget already billed to.
    ///
    /// Its `elapsed_millis` is zero on purpose: this is the point the operation started from, and
    /// the operation's clock is the one [`OperationLedger::usage`] reports.
    pub fn entry_usage(&self) -> UsageSnapshot {
        self.shared.entry.clone()
    }

    /// What one part of the operation has been charged.
    ///
    /// The counted dimensions are that owner's own — the three cumulative shares add up to the
    /// operation's totals beside the entry usage — and its depth marks are the deepest values *that
    /// owner* accepted. `elapsed_millis` is the operation's clock rather than a per-owner duration:
    /// there is one wall clock, and three readings of it are not three durations.
    pub fn cumulative(&self, owner: UsageOwner) -> UsageSnapshot {
        let mut usage = self.shared.owner_snapshot(owner);
        usage.elapsed_millis = elapsed_millis(self.shared.started_at);
        usage
    }

    /// Records that the operation is stopping, and stops every worker of it.
    ///
    /// The first recorded stop is kept and later ones are ignored, so the report names what really
    /// happened first. Cancelling the operation's token is part of the record: a worker waiting for
    /// capacity, or a worker whose next charge is of another dimension, learns to stop from the
    /// token rather than from a check it may never reach. Work that already took its permit is still
    /// billed — a stop does not refund it.
    pub fn cancel(&self, reason: BulkStopKind) {
        self.shared.record(BulkStop {
            owner: None,
            kind: reason,
            dimension: None,
        });
    }

    /// The operation's first stop, when it has one.
    ///
    /// A cancellation the operation's token already carries is recorded here when no checkpoint has
    /// reached it yet: the token is the operation's own sticky state, and a report that read "no
    /// stop" while the caller had cancelled the request would claim a completion that did not
    /// happen. The record is written before the read, so this cannot be overtaken by a later
    /// observation — the deadline, by contrast, is a clock reading rather than a state, and an
    /// operation that finished its work before that reading is not retroactively stopped by it.
    pub fn stop_reason(&self) -> Option<BulkStop> {
        if self.shared.recorded().is_none() && self.shared.cancellation.is_cancelled() {
            self.shared.record(BulkStop {
                owner: None,
                kind: BulkStopKind::Cancelled,
                dimension: None,
            });
        }
        self.shared.recorded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three owners are the declared list, in the declared order, and each has its own slot.
    ///
    /// The ledger's state is indexed by the owner's position in this list, so an owner added without
    /// a slot — or two owners sharing one — would misattribute every charge after it.
    #[test]
    fn the_owner_list_is_the_declared_three_in_order() {
        assert_eq!(
            UsageOwner::ALL,
            [
                UsageOwner::Discovery,
                UsageOwner::Methods,
                UsageOwner::Delivery
            ]
        );
        let mut indexes = Vec::new();
        for owner in UsageOwner::ALL {
            let index = owner.index();
            assert_eq!(UsageOwner::ALL[index], owner);
            indexes.push(index);
        }
        indexes.sort_unstable();
        assert_eq!(indexes, (0..UsageOwner::ALL.len()).collect::<Vec<_>>());
    }

    /// The wire shape the reports and the streamed records carry, pinned once.
    #[test]
    fn the_stop_serializes_as_the_report_spelling() {
        let stop = BulkStop {
            owner: Some(UsageOwner::Methods),
            kind: BulkStopKind::Budget,
            dimension: Some(CountedBudgetDimension::AnalysisSteps.into()),
        };
        assert_eq!(
            serde_json::to_value(&stop).expect("a stop serializes"),
            serde_json::json!({
                "owner": "methods",
                "kind": "budget",
                "dimension": "analysis_steps",
            })
        );
        assert_eq!(
            stop.termination(),
            Some(TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
            })
        );

        // A stop the operation itself recorded names no owner and no dimension, and it is not a
        // `TerminationReason`: the aggregate states those with `ExecutionReport::Cancelled`.
        let cancelled = BulkStop {
            owner: None,
            kind: BulkStopKind::Sink,
            dimension: None,
        };
        assert_eq!(cancelled.termination(), None);
        assert_eq!(
            serde_json::to_value(&cancelled).expect("a stop serializes"),
            serde_json::json!({"owner": null, "kind": "sink", "dimension": null})
        );
    }
}
