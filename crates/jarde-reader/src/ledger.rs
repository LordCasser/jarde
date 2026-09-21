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
//! 3. the operation's total is taken **atomically** ([`OperationLedger::charge`]): limit, checked
//!    addition and the booking of the owner's share happen under one short critical section;
//! 4. the budget records the local usage.
//!
//! A charge that fails before step 3 started nothing and is billed nowhere, and a charge that took
//! its permit is billed even if a cancellation arrives immediately afterwards: work that already had
//! its permit is not refunded, and the next checkpoint stops it. This version takes a permit per
//! action and pre-borrows no bulk quota, so the totals always state what really ran.
//!
//! ## Short critical sections
//!
//! The lock covers a handful of field reads and writes. It is **never** held across a parse, a
//! decompression, IR construction, recovery or a sink call, it is never held while a join or a sink
//! wait happens, and it is never taken while the facts store's own lock is held (the store polls the
//! budget, which may take this lock, *before* it takes its own).
//!
//! ## One stop, published once
//!
//! The operation's first observed stop is recorded ([`BulkStop`]) and later observations never
//! replace it, so a report can name the reason that really happened instead of whichever worker
//! raced last. Recording a stop also cancels the operation's token: a worker whose next charge would
//! still fit another dimension cannot observe a quota another worker exhausted, and a worker waiting
//! for capacity cannot observe a consumer that stopped. A *local* refusal — a method's own limit, or
//! a depth a single container needed — is deliberately **not** recorded here: it stops that work and
//! leaves the rest of the operation alone.
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
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

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

/// Everything the operation's total is made of.
///
/// One mutex holds all of it because the fields move together: an admission reads the total, writes
/// the total and books the owner's share in one critical section, and a stop is the operation's
/// single first observation. One lock per field could not state either of those.
#[derive(Debug)]
struct LedgerState {
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
    /// The operation's totals: [`Self::entry`] plus everything admitted since.
    total: UsageSnapshot,
    /// What each owner has been charged, in [`UsageOwner::ALL`] order, plus each owner's own
    /// depth high-water marks.
    owners: [UsageSnapshot; 3],
    /// The first stop anyone observed. Later observations do not replace it.
    stop: Option<BulkStop>,
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

/// Records `stop` when the operation has none yet, and makes the stop reach every worker.
///
/// The token is cancelled together with the record: a worker whose next charge is of another
/// dimension, or a worker waiting for capacity, has no other way to observe the stop. The first
/// record is what the report publishes, so the observations that follow it — every later checkpoint
/// sees the cancelled token — cannot move it.
fn record(state: &mut LedgerState, stop: BulkStop) {
    if state.stop.is_none() {
        state.stop = Some(stop);
        state.cancellation.cancel();
    }
}

/// The operation-level checks every entry point runs before it does anything: cancellation, then the
/// deadline. Both belong to the operation, and both are what a worker has to observe.
fn checkpoint(state: &mut LedgerState) -> Result<()> {
    if state.cancellation.is_cancelled() {
        record(
            state,
            BulkStop {
                owner: None,
                kind: BulkStopKind::Cancelled,
                dimension: None,
            },
        );
        return Err(cancelled());
    }
    let elapsed = elapsed_millis(state.started_at);
    if elapsed >= state.limits.elapsed_millis {
        record(
            state,
            BulkStop {
                owner: None,
                kind: BulkStopKind::Budget,
                dimension: Some(BudgetDimension::ElapsedMillis),
            },
        );
        return Err(Error::BudgetExceeded {
            dimension: BudgetDimension::ElapsedMillis,
            limit: state.limits.elapsed_millis,
            consumed: elapsed,
            requested: 0,
        });
    }
    Ok(())
}

/// One operation's total, shared by every worker of that operation.
///
/// Built once per operation from the budget that carries the operation's total limits
/// ([`OperationLedger::new`]) and attached to the budgets that do the work
/// ([`Budget::with_ledger`]). Cloning a handle shares the one total; see the module documentation
/// for what that total guarantees.
#[derive(Clone, Debug)]
pub struct OperationLedger {
    state: Arc<Mutex<LedgerState>>,
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
            Some(previous) => {
                let state = previous.shared();
                (
                    state.total.clone(),
                    state.started_at,
                    state.cancellation.clone(),
                )
            }
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
        Self {
            state: Arc::new(Mutex::new(LedgerState {
                limits: entry.limits().clone(),
                started_at,
                cancellation,
                entry: start.clone(),
                total: start,
                owners: std::array::from_fn(|_| UsageSnapshot::default()),
                stop: None,
            })),
        }
    }

    /// Takes `requested` units of `dimension` for `owner`'s work, or refuses without taking any.
    ///
    /// The operation-level checks run first, then the total's limit and checked addition are
    /// evaluated against the totals as they stand **inside one critical section**, and the owner's
    /// share is booked there too. A refusal is therefore a refusal of the operation's own quota —
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
        let mut state = self.shared();
        checkpoint(&mut state)?;
        let limit = state.limits.counted_limit(dimension);
        let consumed = state.total.counted_usage(dimension);
        let admitted = match consumed.checked_add(requested) {
            Some(total) if total <= limit => total,
            _ => {
                record(
                    &mut state,
                    BulkStop {
                        owner: Some(owner),
                        kind: BulkStopKind::Budget,
                        dimension: Some(dimension.into()),
                    },
                );
                return Err(Error::BudgetExceeded {
                    dimension: dimension.into(),
                    limit,
                    consumed,
                    requested,
                });
            }
        };
        let owner_consumed = state.owners[owner.index()].counted_usage(dimension);
        let Some(owner_total) = owner_consumed.checked_add(requested) else {
            // The owner's share is part of the total the admission above already bounded, so this
            // is the same overflow one level down: refused, never wrapped.
            record(
                &mut state,
                BulkStop {
                    owner: Some(owner),
                    kind: BulkStopKind::Budget,
                    dimension: Some(dimension.into()),
                },
            );
            return Err(Error::BudgetExceeded {
                dimension: dimension.into(),
                limit,
                consumed: owner_consumed,
                requested,
            });
        };
        state.total.set(dimension, admitted);
        state.owners[owner.index()].set(dimension, owner_total);
        Ok(())
    }

    /// Whether the operation could still take `requested` units of `dimension`, without taking them.
    ///
    /// The same arithmetic [`OperationLedger::charge`] performs, read-only and without recording
    /// anything: this is the "would this fit" question [`Budget::check`] asks, and a probe that
    /// refuses nothing is not a stop.
    pub(crate) fn check(&self, dimension: CountedBudgetDimension, requested: u64) -> Result<()> {
        let state = self.shared();
        if state.cancellation.is_cancelled() {
            return Err(cancelled());
        }
        let elapsed = elapsed_millis(state.started_at);
        if elapsed >= state.limits.elapsed_millis {
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
                limit: state.limits.elapsed_millis,
                consumed: elapsed,
                requested: 0,
            });
        }
        let limit = state.limits.counted_limit(dimension);
        let consumed = state.total.counted_usage(dimension);
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
        let mut state = self.shared();
        checkpoint(&mut state)?;
        let limit = state.limits.nested_depth;
        if depth > limit {
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::NestedDepth,
                limit,
                consumed: depth.saturating_sub(1),
                requested: 1,
            });
        }
        state.total.nested_depth = state.total.nested_depth.max(depth);
        let slot = &mut state.owners[owner.index()];
        slot.nested_depth = slot.nested_depth.max(depth);
        Ok(())
    }

    /// The dependency-closure depth the operation reached, enforcing the operation's
    /// `DependencyDepth`.
    ///
    /// Same high-water shape and the same refusal rule as
    /// [`OperationLedger::check_nested_depth`]: the deepest accepted depth is kept, a deeper one is
    /// refused, and neither dimension touches the other's value.
    pub fn observe_dependency_depth(&self, owner: UsageOwner, depth: u64) -> Result<()> {
        let mut state = self.shared();
        checkpoint(&mut state)?;
        let limit = state.limits.dependency_depth;
        if depth > limit {
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::DependencyDepth,
                limit,
                consumed: depth.saturating_sub(1),
                requested: 1,
            });
        }
        state.total.dependency_depth = state.total.dependency_depth.max(depth);
        let slot = &mut state.owners[owner.index()];
        slot.dependency_depth = slot.dependency_depth.max(depth);
        Ok(())
    }

    /// The operation's deadline and cancellation, without accounting anything.
    ///
    /// This is the operation-level half of a checkpoint: a worker calls it (through its own budget)
    /// before it starts a unit of work, and the first observation of an expired deadline or a
    /// cancellation is recorded as the operation's stop.
    pub fn poll(&self) -> Result<()> {
        let mut state = self.shared();
        checkpoint(&mut state)
    }

    /// The operation's totals right now: the entry usage plus every owner's work, with the depth
    /// high-water marks taken over all of them and the wall clock measured once from the start.
    pub fn usage(&self) -> UsageSnapshot {
        let state = self.shared();
        let mut usage = state.total.clone();
        usage.elapsed_millis = elapsed_millis(state.started_at);
        usage
    }

    /// The usage the operation started from: the entry budget's own usage when the ledger was built,
    /// or the total of the ledger that budget already billed to.
    ///
    /// Its `elapsed_millis` is zero on purpose: this is the point the operation started from, and
    /// the operation's clock is the one [`OperationLedger::usage`] reports.
    pub fn entry_usage(&self) -> UsageSnapshot {
        self.shared().entry.clone()
    }

    /// What one part of the operation has been charged.
    ///
    /// The counted dimensions are that owner's own — the three cumulative shares add up to the
    /// operation's totals beside the entry usage — and its depth marks are the deepest values *that
    /// owner* accepted. `elapsed_millis` is the operation's clock rather than a per-owner duration:
    /// there is one wall clock, and three readings of it are not three durations.
    pub fn cumulative(&self, owner: UsageOwner) -> UsageSnapshot {
        let state = self.shared();
        let mut usage = state.owners[owner.index()].clone();
        usage.elapsed_millis = elapsed_millis(state.started_at);
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
        let mut state = self.shared();
        record(
            &mut state,
            BulkStop {
                owner: None,
                kind: reason,
                dimension: None,
            },
        );
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
        let mut state = self.shared();
        if state.stop.is_none() && state.cancellation.is_cancelled() {
            record(
                &mut state,
                BulkStop {
                    owner: None,
                    kind: BulkStopKind::Cancelled,
                    dimension: None,
                },
            );
        }
        state.stop.clone()
    }

    /// A lock this accounting never leaves poisoned behind it.
    ///
    /// The critical sections are field reads and writes: no user code and no fallible allocation. If
    /// one were poisoned anyway, a ledger is not a reason to fail an operation — the totals are read
    /// as they stand, which is at worst the state of the last completed charge.
    fn shared(&self) -> MutexGuard<'_, LedgerState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
