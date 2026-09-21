//! Why a recovery run stopped before it produced an artifact, and the billing helper every stage
//! stops through.
//!
//! # Why a stop is a value and not an error
//!
//! The P2 pipeline states a stop in the run's execution plane, and the recovery layer does the same
//! for the same reason: a budget that ran out is a *result* about the run — with the bytes written
//! and the BCI it stopped at — not an exception the caller has to reconstruct from an error type.
//! The one property this vocabulary exists to keep is that a stopped run cannot be mistaken for a
//! produced one: the report of a stop carries no text, no segments and an execution plane that says
//! `partial` or `cancelled`, and the caller has to read that plane to learn the difference.
//!
//! # Billing before the work
//!
//! Every charge happens *before* the work it pays for, and the graph algorithms bill their whole
//! input up front: petgraph's dominator and SCC walks have no interruption hook (the raw CFG
//! documents the same boundary), so a run that cannot afford the walk must refuse to enter it
//! rather than expect to be stopped inside. That is why [`charge`] is the only way these stages
//! bill: there is no path here that runs work first and discovers the limit afterwards.

use jarde_reader::budget::{Budget, CountedBudgetDimension};
use serde::Serialize;

/// Why a recovery run stopped.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// The payload has no such table, so the phase validity of the run does not reach the recovery
    /// layer and nothing can be presented. Never an empty artifact with a success state.
    IrTableMissing { table: &'static str },
    /// A counted bound refused a charge before the work it pays for.
    Budget {
        dimension: CountedBudgetDimension,
        /// Bytes of text written before the refusal; `0` when the refusal happened before the
        /// emitter ran.
        written: u64,
        /// The bound that refused, as the run's limits state it.
        limit: u64,
        /// The node the emitter was writing when it refused, when it was writing one.
        at: Option<u32>,
    },
    /// The run's cancellation token was set.
    Cancelled { at: Option<u32> },
    /// The budget refused to continue for a reason that is neither the output bound nor a flagged
    /// cancellation (elapsed time, for instance), or the recovery recursion refused to descend any
    /// further: `code` is the stable name of what interrupted the run — `jre_budget_interrupted`,
    /// [`RECURSION_BOUND_CODE`] or [`RECURSION_REENTRY_CODE`] — and `at` is the node it stopped on.
    Interrupted { code: &'static str, at: Option<u32> },
    /// The request's own evidence selection cannot be answered for this body (change
    /// `add-demand-driven-core-results`, D1): a category this entry does not materialize, a driver
    /// range stated without a category, or a range this body's decoded instructions cannot support.
    /// Nothing was presented — the refusal is the request's fact and not the body's — and the run
    /// never widens an unsupported selection into a full-evidence delivery.
    EvidenceRefused {
        /// The stable name of the refusal ([`crate::evidence::UNSUPPORTED_KIND_CODE`],
        /// [`crate::evidence::RANGE_SHAPE_CODE`] or [`crate::evidence::RANGE_REFUSAL_CODE`]).
        code: &'static str,
        /// The instruction index the refusal is about, when it is about one.
        at: Option<u32>,
        /// One sentence stating what could not be applied.
        message: String,
    },
}

/// The code of a run a budget poll interrupted without a flagged cancellation: the elapsed bound
/// running out, for instance. It keeps its own code and its own wording — [`RECURSION_BOUND_CODE`]
/// and [`RECURSION_REENTRY_CODE`] are different reasons for the same plane.
pub(crate) const BUDGET_INTERRUPTED_CODE: &str = "jre_budget_interrupted";

/// The code of a run stopped because the recovery recursion reached its explicit depth bound
/// (`region.rs`'s `MAX_REGION_DEPTH`).
///
/// The bound is checked before the walk descends, never after: the process stack cannot be
/// recovered once it is gone, so the run has to refuse to go deeper while it still can.
pub(crate) const RECURSION_BOUND_CODE: &str = "jre_recursion_bound";

/// The code of a run stopped because the recovery recursion re-entered a state this run had already
/// entered — the same block of the same structure — and so cannot prove that recursion completes.
///
/// This is deliberately not [`RECURSION_BOUND_CODE`]: "the input nests too deeply" and "the walk is
/// back inside a structure it is already building" are different facts about the run, and a
/// diagnosis that called one the other would name a cause that did not stop it.
pub(crate) const RECURSION_REENTRY_CODE: &str = "jre_recursion_reentry";

impl StopReason {
    /// The node the run stopped at, when it stopped inside one.
    pub fn at(&self) -> Option<u32> {
        match self {
            Self::IrTableMissing { .. } => None,
            Self::Budget { at, .. } | Self::Cancelled { at } | Self::Interrupted { at, .. } => *at,
            Self::EvidenceRefused { at, .. } => *at,
        }
    }

    /// Whether the stop was the caller's cancellation rather than a bound.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled { .. })
    }
}

/// The one way a stage of the recovery layer bills work.
///
/// `at` is the node the stage was working on, which is what makes a stopped run's report say *where*
/// it stopped rather than only that it did.
pub(crate) fn charge(
    budget: &mut Budget,
    dimension: CountedBudgetDimension,
    requested: u64,
    at: Option<u32>,
) -> Result<(), StopReason> {
    if budget.cancellation_token().is_cancelled() {
        // A cancellation outranks a bound: the caller asked for the run to end, so reporting a
        // budget would name a limit that did not stop it.
        return Err(StopReason::Cancelled { at });
    }
    if budget.charge(dimension, requested).is_err() {
        let limit = budget.limits().counted_limit(dimension);
        return Err(StopReason::Budget {
            dimension,
            written: budget.usage().output_bytes,
            limit,
            at,
        });
    }
    Ok(())
}

/// The poll every stage runs before starting a step, so that a cancellation is noticed even where no
/// charge is due.
pub(crate) fn poll(budget: &Budget, at: Option<u32>) -> Result<(), StopReason> {
    if budget.cancellation_token().is_cancelled() {
        return Err(StopReason::Cancelled { at });
    }
    if budget.poll().is_err() {
        return Err(StopReason::Interrupted {
            code: BUDGET_INTERRUPTED_CODE,
            at,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarde_reader::budget::{CancellationToken, Limits};

    #[test]
    fn a_refused_charge_names_the_bound_it_refused_on() {
        let limits = Limits {
            ir_items: 1,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        };
        let mut budget = Budget::new(limits);
        assert_eq!(
            charge(&mut budget, CountedBudgetDimension::IrItems, 1, Some(7)),
            Ok(())
        );
        let stop = charge(&mut budget, CountedBudgetDimension::IrItems, 1, Some(9))
            .expect_err("the second item is over the bound");
        assert_eq!(
            stop,
            StopReason::Budget {
                dimension: CountedBudgetDimension::IrItems,
                written: 0,
                limit: 1,
                at: Some(9),
            }
        );
        assert!(!stop.is_cancelled());
        assert_eq!(stop.at(), Some(9));
    }

    #[test]
    fn a_cancellation_outranks_a_bound_and_says_so() {
        let token = CancellationToken::new();
        token.cancel();
        let limits = Limits {
            ir_items: 1,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        };
        let mut budget = Budget::with_cancellation_token(limits, token);
        assert_eq!(
            charge(
                &mut budget,
                CountedBudgetDimension::IrItems,
                1 << 20,
                Some(3)
            ),
            Err(StopReason::Cancelled { at: Some(3) }),
            "the caller's cancellation is what stopped the run"
        );
        assert!(poll(&budget, None).unwrap_err().is_cancelled());
    }
}
