//! The fixed pass table of the P2 method-analysis pipeline and its startup validation.
//!
//! Three things are fixed here and nowhere else:
//!
//! * the phase vocabulary ([`IrPhase`]) and the fact vocabulary ([`FactKind`]) of the method
//!   IR pipeline. [`IrPhase`] is 1:1 with the public [`AnalysisStage`] — same order, same
//!   names, `RawFacts = 1` — so a request never carries a second numbering;
//! * the pass table ([`PASSES`]) as a **static** constant in phase order. The engine runs
//!   exactly this order: there is no dynamic registration, no plugin interface and no runtime
//!   graph, and a later slice adds a pass by extending the constant;
//! * the startup validation of a requested stage set ([`validate_requested_stages`]), which
//!   projects the request onto the table prefix and refuses a schedule that cannot run
//!   *before* it executes anything: `ir_pass_order_invalid` (the table is not in ascending
//!   phase order), `ir_pass_graph_cycle` (the table's `requires`/`produces` relation is
//!   cyclic, including a pass that requires what it produces itself),
//!   `ir_pass_prerequisite_missing` (a scheduled pass needs a fact no earlier scheduled pass
//!   produces) and `ir_stale_fact` (a scheduled pass needs a fact an earlier pass invalidated
//!   and nothing recomputed).
//!
//! Invalidation is not a comment: [`FactLedger`] owns the "already produced facts" set, a pass
//! that declares `invalidates` leaves those facts stale until a pass publishes them again, and
//! requiring a stale fact is `ir_stale_fact` instead of a silently reused old analysis. The
//! same ledger is all-or-nothing — it checks every `requires` before it records anything —
//! which is what keeps a pass that cannot run from publishing half-initialized facts and from
//! advancing the last valid phase. The startup validation drives the scheduled prefix through
//! a throwaway ledger, so what a request is accepted with and what the pipeline checks while
//! it runs cannot drift apart.
//!
//! The table is a handful of passes over nine facts, so the checks in this module are direct
//! walks over the slices; `petgraph` is admitted for the raw CFG (3.1) and is not used here.

use crate::ir::AnalysisStage;
use jarde_reader::error::{Error, Result};

/// Code of a scheduled pass whose `requires` no earlier scheduled pass produces.
const IR_PASS_PREREQUISITE_MISSING: &str = "ir_pass_prerequisite_missing";

/// Code of a pass table that is not declared in ascending phase order.
const IR_PASS_ORDER_INVALID: &str = "ir_pass_order_invalid";

/// Code of a pass table whose `requires`/`produces` relation has a cycle.
const IR_PASS_GRAPH_CYCLE: &str = "ir_pass_graph_cycle";

/// Code of a pass that requires a fact an earlier pass invalidated and nothing recomputed.
const IR_STALE_FACT: &str = "ir_stale_fact";

/// Code of a scheduled pass this build does not implement yet.
///
/// The table is complete — its phases, facts and dependencies are 3.2's contract — but a build
/// implements the phases slice by slice (3.3 runs `raw_facts` and `raw_cfg`). A scheduled pass
/// the pipeline reached and cannot run is `Failed { code }` with this code and an `Unsupported`
/// termination, and the phases behind it stay `NotPerformed`: the request is answered honestly
/// instead of pretending the pipeline finished. 5.1 removes this code with the last phase.
pub(crate) const IR_PASS_NOT_IMPLEMENTED: &str = "ir_pass_not_implemented";

/// One phase of the fixed P2 IR pipeline; declaration order is the phase order.
///
/// The numbering is part of the contract (`RawFacts` starts at 1) and the name of a phase is
/// the snake_case name the same phase serializes to in a request ([`AnalysisStage`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum IrPhase {
    RawFacts = 1,
    RawCfg,
    LegacyNormalization,
    CanonicalCfg,
    Frame,
    Ssa,
}

impl IrPhase {
    /// The phase a request stage asks for; the mapping is 1:1 and order preserving.
    pub(crate) fn from_stage(stage: AnalysisStage) -> Self {
        match stage {
            AnalysisStage::RawFacts => Self::RawFacts,
            AnalysisStage::RawCfg => Self::RawCfg,
            AnalysisStage::LegacyNormalization => Self::LegacyNormalization,
            AnalysisStage::CanonicalCfg => Self::CanonicalCfg,
            AnalysisStage::Frame => Self::Frame,
            AnalysisStage::Ssa => Self::Ssa,
        }
    }

    /// Snake_case name of the phase, the name [`AnalysisStage`] serializes to.
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::RawFacts => "raw_facts",
            Self::RawCfg => "raw_cfg",
            Self::LegacyNormalization => "legacy_normalization",
            Self::CanonicalCfg => "canonical_cfg",
            Self::Frame => "frame",
            Self::Ssa => "ssa",
        }
    }

    /// The request stage this phase is: the inverse of [`IrPhase::from_stage`], used when a
    /// report has to list the stage of a pass.
    pub(crate) fn stage(self) -> AnalysisStage {
        match self {
            Self::RawFacts => AnalysisStage::RawFacts,
            Self::RawCfg => AnalysisStage::RawCfg,
            Self::LegacyNormalization => AnalysisStage::LegacyNormalization,
            Self::CanonicalCfg => AnalysisStage::CanonicalCfg,
            Self::Frame => AnalysisStage::Frame,
            Self::Ssa => AnalysisStage::Ssa,
        }
    }
}

/// Whether this build implements the pass of one phase.
///
/// The implemented phases are a **prefix** of the table — a later phase never runs while an
/// earlier one is missing — which is what makes a request stop at exactly one point instead of
/// skipping a hole in the pipeline. 3.3 implements `raw_facts` (the reader's own work, see the
/// table) and `raw_cfg`, 3.4 adds `legacy_normalization` (the call contexts, which is not the
/// cloning pass), 3.5 adds `canonical_cfg` (the bounded clone normalization that consumes
/// exactly those contexts), 4.1 adds `frame` (the descriptor-driven slot states over the
/// canonical graph) and 4.3 adds `ssa` (the stack/local names over those frames); 5.1 deletes
/// this list with the last phase.
pub(crate) fn implemented(phase: IrPhase) -> bool {
    matches!(
        phase,
        IrPhase::RawFacts
            | IrPhase::RawCfg
            | IrPhase::LegacyNormalization
            | IrPhase::CanonicalCfg
            | IrPhase::Frame
            | IrPhase::Ssa
    )
}

/// One fact of the method IR pipeline, as the pass table names it.
///
/// Facts are the unit of `requires`/`produces`/`invalidates`: a fact is published by a pass and
/// stays available to later passes until a pass invalidates it. These are the facts 3.x–5.x
/// publish; the IR payloads themselves stay crate-private.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum FactKind {
    /// Typed instruction facts of the method body (1.2).
    Instructions,
    /// The exception table as the class declares it.
    ExceptionTable,
    /// Instruction-level throw sites with the handlers that may catch them (3.3).
    ThrowSites,
    /// Raw CFG over the decoded instructions (3.3).
    RawCfg,
    /// The `jsr`/`ret` call contexts of one method (3.4).
    CallContexts,
    /// The CFG after bounded legacy normalization (3.5).
    CanonicalCfg,
    /// Frame state per block (4.x).
    Frames,
    /// Stack/local SSA values and phi inputs (4.x).
    Ssa,
    /// Per-instruction effect facts (3.3), tied to the CFG shape they were derived from.
    Effects,
}

impl FactKind {
    /// Number of facts; this is the length of a ledger's state array.
    const COUNT: usize = 9;

    /// Snake_case name of the fact, used by the diagnostics of a schedule that cannot run.
    pub(crate) fn code(self) -> &'static str {
        match self {
            Self::Instructions => "instructions",
            Self::ExceptionTable => "exception_table",
            Self::ThrowSites => "throw_sites",
            Self::RawCfg => "raw_cfg",
            Self::CallContexts => "call_contexts",
            Self::CanonicalCfg => "canonical_cfg",
            Self::Frames => "frames",
            Self::Ssa => "ssa",
            Self::Effects => "effects",
        }
    }

    /// Position of this fact in a ledger's state array.
    fn index(self) -> usize {
        self as usize
    }
}

/// One counted dimension a pass bills before it allocates; a pass declares a *set* of them.
///
/// The class names the dimension: `Blocks` the IR storage items and edges
/// (`IrItems`/`IrEdges`), `Steps` the worklist steps (`AnalysisSteps`) and `Clones` the
/// `jsr`/`ret` clone nodes (`NormalizationClones`). A pass that bills nothing declares the
/// empty set, which is why there is no "no dimension" variant: an empty declaration and a
/// declared dimension are the two cases the check has to tell apart, and the `raw_facts` pass
/// is the first one.
///
/// A pass may bill several dimensions at once — the raw CFG produces blocks and edges *and*
/// iterates a worklist — so the declaration is a set. The declared set and the dimensions the
/// pass really charges must be equal entry by entry: charging an undeclared dimension and
/// leaving a declared one uncharged are both defects. The mapping itself belongs to the budget
/// layer, which this module does not depend on; a pass charges before it allocates, and a pass
/// boundary is a `poll()` checkpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum PassBudgetClass {
    Blocks,
    Steps,
    Clones,
}

/// One pass: the phase it belongs to, the facts it needs and publishes, and what it bills.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PassDescriptor {
    pub(crate) phase: IrPhase,
    /// Name of the pass, the same name as its phase while the table has one pass per phase.
    pub(crate) name: &'static str,
    /// Facts that must be live when the pass starts.
    pub(crate) requires: &'static [FactKind],
    /// Facts the pass publishes when it completes.
    pub(crate) produces: &'static [FactKind],
    /// Facts the pass makes obsolete; a pass that changes the CFG or its exception edges must
    /// declare the facts derived from the old shape here.
    pub(crate) invalidates: &'static [FactKind],
    /// The counted dimensions the pass bills before it allocates, as a set; the empty set is a
    /// pass that bills no counted dimension. See [`PassBudgetClass`].
    pub(crate) budget: &'static [PassBudgetClass],
}

/// The pass table: one descriptor per phase, in phase order, and the whole pipeline.
///
/// Every `requires` of a pass is produced by an earlier pass of this same table, which is what
/// the startup validation checks against the requested prefix. What each pass really reads is
/// the implementation contract of 3.x/4.x, so it is declared here, once, instead of leaving the
/// order of the passes to whatever the caller happens to ask for.
pub(crate) static PASSES: &[PassDescriptor] = &[
    // Decoding is the reader's own work (1.2) and its bytes are already charged as
    // `ClassBytes`/`AttributeBytes`/`CodeBytes`, so this pass projects charged reader output
    // into the fact set and bills no counted dimension of its own: the empty budget set.
    PassDescriptor {
        phase: IrPhase::RawFacts,
        name: "raw_facts",
        requires: &[],
        produces: &[FactKind::Instructions, FactKind::ExceptionTable],
        invalidates: &[],
        budget: &[],
    },
    // 3.3: blocks and edges over the raw facts, the throw site of every throwing instruction
    // with its handler list, and the effect facts of the raw graph — which stay the valid
    // effects while a legacy fallback (3.5) keeps the raw bytes instead of canonicalizing them.
    // The two dimensions this pass bills are its own: constructing a block or an edge is an IR
    // storage item (`Blocks`), and walking the worklist is analysis work counted per step
    // (`Steps`), so one pass declares both.
    PassDescriptor {
        phase: IrPhase::RawCfg,
        name: "raw_cfg",
        requires: &[FactKind::Instructions, FactKind::ExceptionTable],
        produces: &[FactKind::RawCfg, FactKind::ThrowSites, FactKind::Effects],
        invalidates: &[],
        budget: &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
    },
    // 3.4: one context per `jsr` site with its return point and affected locals, derived from
    // the raw graph, the exception coverage that crosses it and the **effect facts of that
    // graph** — which is why `Effects` is in `requires`: the pass reads `raw.effects`, and a
    // fact a pass consumes without naming is a fact the stale check cannot see (the
    // canonicalization invalidates the raw-graph effects, and only a declared consumer is
    // refused while they are stale). The contexts, the plans and the per-context local sets are
    // derived storage, billed per storage item (`Blocks`) — `IrEdges` included — and the walk
    // is billed as analysis steps, because a repeated visit of a block is real work. The pass
    // changes no fact: this is not the cloning pass, the clones belong to `canonical_cfg`
    // (3.5), which bills `Clones`.
    PassDescriptor {
        phase: IrPhase::LegacyNormalization,
        name: "legacy_normalization",
        requires: &[
            FactKind::Instructions,
            FactKind::ExceptionTable,
            FactKind::RawCfg,
            FactKind::ThrowSites,
            FactKind::Effects,
        ],
        produces: &[FactKind::CallContexts],
        invalidates: &[],
        budget: &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
    },
    // 3.5: the bounded clone normalization, billed per clone node before it is created, per
    // block and edge it rebuilds and per step of its clone worklist. It replaces the graph, so
    // every fact derived from the old blocks and exception edges — the raw-graph effects, the
    // frames, the SSA — is stale until a later pass recomputes it on the canonical graph: the
    // frames by the `frame` pass and the SSA, with the canonical graph's effects, by the `ssa`
    // pass, both below.
    PassDescriptor {
        phase: IrPhase::CanonicalCfg,
        name: "canonical_cfg",
        requires: &[
            FactKind::Instructions,
            FactKind::ExceptionTable,
            FactKind::RawCfg,
            FactKind::ThrowSites,
            FactKind::CallContexts,
        ],
        produces: &[FactKind::CanonicalCfg],
        invalidates: &[FactKind::Effects, FactKind::Frames, FactKind::Ssa],
        budget: &[
            PassBudgetClass::Blocks,
            PassBudgetClass::Steps,
            PassBudgetClass::Clones,
        ],
    },
    // 4.1/4.2: descriptor-driven frames over the canonical blocks (the stack shape comes from
    // the dense opcode table, not from `Effects`). Frame and local slots are IR storage items,
    // charged before they are allocated, and the fixpoint over the blocks walks a worklist
    // (4.2's handler entries included), so the pass bills the worklist steps as well.
    PassDescriptor {
        phase: IrPhase::Frame,
        name: "frame",
        requires: &[
            FactKind::Instructions,
            FactKind::ExceptionTable,
            FactKind::CanonicalCfg,
        ],
        produces: &[FactKind::Frames],
        invalidates: &[],
        budget: &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
    },
    // 4.3: stack/local SSA values and phi inputs over the canonical graph and its frames,
    // likewise charged as IR storage items and as the steps of its worklist walk. The effect
    // facts are republished here because their order belongs to this pass and because the
    // canonicalization dropped the raw-graph ones.
    PassDescriptor {
        phase: IrPhase::Ssa,
        name: "ssa",
        requires: &[
            FactKind::Instructions,
            FactKind::CanonicalCfg,
            FactKind::Frames,
        ],
        produces: &[FactKind::Ssa, FactKind::Effects],
        invalidates: &[],
        budget: &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
    },
];

/// Whether one fact is available to a pass of the current run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FactState {
    /// No pass produced it; invalidating an absent fact is a no-op, so it stays uninvolved and
    /// a later `requires` reports it as `ir_pass_prerequisite_missing` rather than stale.
    NotProduced,
    /// The newest value of the fact is available to the next pass.
    Live,
    /// A pass declared the fact in `invalidates` and no pass published it again since.
    Stale,
}

/// Which facts one pipeline run has produced, invalidated and recomputed.
///
/// The ledger is what keeps invalidation from being a comment: a fact an `invalidates` entry
/// names leaves `Live` and comes back only when a pass publishes it again, and a pass is
/// checked — all of its `requires`, before anything is recorded — so a pass that cannot run
/// publishes no fact and does not advance the last valid phase. Only declared dependencies are
/// visible here, which is why a pass that consumes a fact has to name it in its `requires`.
/// The startup validation drives the scheduled prefix through a throwaway ledger; the runtime
/// slices apply each pass as they run it, which is why a schedule and a run are checked by the
/// same code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FactLedger {
    facts: [FactState; FactKind::COUNT],
    /// Highest phase a pass of this run completed; a stopped pass leaves it where it was, and
    /// re-applying an earlier phase (3.5 retrying the canonicalization with a larger clone
    /// budget) never lowers it, so the value means "how far the run got" and not "which pass
    /// was applied last".
    last_completed: Option<IrPhase>,
}

impl FactLedger {
    /// A ledger of a run in which no pass has run yet.
    pub(crate) fn new() -> Self {
        Self {
            facts: [FactState::NotProduced; FactKind::COUNT],
            last_completed: None,
        }
    }

    /// Applies one pass: checks every `requires`, then records the invalidations and the facts
    /// the pass publishes.
    ///
    /// Failure isolation is the point of the first step: a pass that cannot obtain a fact is
    /// rejected before it publishes anything, so a stopped run keeps exactly the facts of its
    /// last valid phase. Invalidations are recorded before the productions, so a pass that
    /// rebuilds a fact it invalidated leaves it live.
    pub(crate) fn apply(&mut self, pass: &PassDescriptor) -> Result<()> {
        for fact in pass.requires {
            match self.state(*fact) {
                FactState::Live => {}
                FactState::Stale => {
                    return Err(Error::invalid_input(
                        IR_STALE_FACT,
                        format!(
                            "the pass `{}` (phase `{}`) requires the fact `{}`, which an earlier \
                             pass invalidated and no pass recomputed since ({})",
                            pass.name,
                            pass.phase.code(),
                            fact.code(),
                            self.progress(),
                        ),
                    ));
                }
                FactState::NotProduced => {
                    return Err(Error::invalid_input(
                        IR_PASS_PREREQUISITE_MISSING,
                        format!(
                            "the pass `{}` (phase `{}`) requires the fact `{}`, which no earlier \
                             scheduled pass produced ({})",
                            pass.name,
                            pass.phase.code(),
                            fact.code(),
                            self.progress(),
                        ),
                    ));
                }
            }
        }
        for fact in pass.invalidates {
            self.invalidate(*fact);
        }
        for fact in pass.produces {
            self.publish(*fact);
        }
        // The highest completed phase, not the last applied one: re-entering an earlier phase
        // (3.5 retrying the canonicalization) must not report a run that got less far than it
        // really did, because 5.1 reads this value when it assembles `stages`.
        self.last_completed = self.last_completed.max(Some(pass.phase));
        Ok(())
    }

    /// State of one fact; `Live` is the only state a pass may require.
    pub(crate) fn state(&self, fact: FactKind) -> FactState {
        self.facts[fact.index()]
    }

    /// Publishes a fact, which also clears a stale one: this is what recomputation looks like.
    fn publish(&mut self, fact: FactKind) {
        self.facts[fact.index()] = FactState::Live;
    }

    /// Drops a fact that was produced; a fact no pass produced stays uninvolved.
    fn invalidate(&mut self, fact: FactKind) {
        if self.facts[fact.index()] == FactState::Live {
            self.facts[fact.index()] = FactState::Stale;
        }
    }

    /// How far the run got, for the diagnostic of a pass that cannot run.
    fn progress(&self) -> String {
        match self.last_completed {
            Some(phase) => format!("last completed phase: {}", phase.code()),
            None => "no phase completed yet".to_string(),
        }
    }
}

/// Startup validation of a request: the passes its stage set schedules, or a structured error.
///
/// The requested stages map 1:1 onto [`IrPhase`]s and the scheduled passes are the table prefix
/// up to the last requested phase — the same prefix rule, over the same phase order, that the
/// report's scheduled stages use, so the phases of the returned passes are exactly the stages a
/// report lists for this request. Before the prefix is returned:
///
/// * the table is checked as a whole: ascending phase order (`ir_pass_order_invalid`) and an
///   acyclic `requires`/`produces` relation (`ir_pass_graph_cycle`);
/// * the prefix is driven through a throwaway [`FactLedger`], so a pass whose `requires` no
///   earlier scheduled pass produced is `ir_pass_prerequisite_missing` and one whose fact an
///   earlier pass invalidated and nothing recomputed is `ir_stale_fact`.
///
/// Nothing runs and nothing is published when this returns an error; the caller gets an input
/// error instead of a half-initialized pipeline. A request with an empty stage set is rejected
/// earlier by [`crate::ir::validate_request`] (`analysis_no_stages`), so this function has no
/// schedule to check and returns the empty prefix rather than owning that code twice.
pub(crate) fn validate_requested_stages(
    stages: &[AnalysisStage],
) -> Result<&'static [PassDescriptor]> {
    let Some(last) = stages.iter().copied().map(IrPhase::from_stage).max() else {
        return Ok(&[]);
    };
    validate_schedule(PASSES, last)
}

/// Table-parameterized core of [`validate_requested_stages`]: checks the whole table, then the
/// scheduled prefix up to and including `last`.
fn validate_schedule(table: &[PassDescriptor], last: IrPhase) -> Result<&[PassDescriptor]> {
    validate_phase_order(table)?;
    validate_acyclic(table)?;
    // The table is in ascending phase order, so the passes up to `last` are a prefix of it.
    let count = table.iter().take_while(|pass| pass.phase <= last).count();
    let scheduled = &table[..count];
    let mut ledger = FactLedger::new();
    for pass in scheduled {
        ledger.apply(pass)?;
    }
    Ok(scheduled)
}

/// Rejects a table whose phases go backwards, the first fault checked because such a table has
/// no prefix that means anything.
///
/// The order rule is the contract's: the table is declared in ascending `IrPhase` order, so a
/// later pass may not run in an earlier phase. Two passes of one phase are *not* a fault — a
/// phase that takes more than one step is ordered by its own dependencies instead — and the
/// prefix walk still sees the passes of that phase in table order.
///
/// Order is a property of the *whole table*, not of the scheduled prefix: a table that goes
/// backwards anywhere is rejected even when the offending pass is beyond the request's prefix.
fn validate_phase_order(table: &[PassDescriptor]) -> Result<()> {
    let mut previous: Option<IrPhase> = None;
    for pass in table {
        if let Some(previous_phase) = previous
            && pass.phase < previous_phase
        {
            return Err(Error::invalid_input(
                IR_PASS_ORDER_INVALID,
                format!(
                    "the pass table is not in ascending phase order: the pass `{}` (phase `{}`) \
                     follows the phase `{}`",
                    pass.name,
                    pass.phase.code(),
                    previous_phase.code(),
                ),
            ));
        }
        previous = Some(pass.phase);
    }
    Ok(())
}

/// Rejects a table whose `requires`/`produces` relation has a cycle — including the
/// `requires`/`produces` conflict of a single pass, which is the smallest such cycle.
///
/// What the graph answers is whether the declared order can satisfy every `requires`, so the
/// edges are built with that question in mind, fact by fact and consumer by consumer:
///
/// * a `require` of a fact an **earlier** pass already produces is satisfiable in declaration
///   order, and a producer *after* the consumer adds no edge to it. A fact may have more than
///   one producer because its meaning follows the graph it was derived from — `Effects` are the
///   raw graph's while `raw_cfg` produced them and the canonical graph's after `ssa` recomputed
///   them (3.5 invalidates them in between) — so a consumer that names such a fact is served by
///   the producer before it, and the later producer is not a second prerequisite;
/// * a `require` no earlier pass produces keeps every producer as an edge, including a later
///   one. A backward edge is then not a cycle by itself: that is `ir_pass_prerequisite_missing`,
///   decided where the schedule is walked, because the fault is that the fact is not there *when
///   the pass runs* — and the prefix walk judges only the scheduled passes, so a dangling
///   `requires` outside the prefix is no error;
/// * a pass that requires what it produces itself (the smallest cycle) and two passes that each
///   need what the other publishes keep their cycle: neither `require` has an earlier producer,
///   so both directions are edges.
///
/// The check is Kahn's algorithm over those edges; a table without a cycle drains completely.
/// Like the phase order, a cycle is a property of the *whole table*: it is rejected even when
/// the cycle lies beyond the scheduled prefix.
fn validate_acyclic(table: &[PassDescriptor]) -> Result<()> {
    let mut indegree = vec![0usize; table.len()];
    let mut consumers: Vec<Vec<usize>> = vec![Vec::new(); table.len()];
    for (consumer, pass) in table.iter().enumerate() {
        for fact in pass.requires {
            // Produced strictly before the consumer, so the fact is available to it in
            // declaration order whatever the later producers of the same fact do.
            let satisfied_earlier = table[..consumer]
                .iter()
                .any(|candidate| candidate.produces.contains(fact));
            for (producer, candidate) in table.iter().enumerate() {
                if candidate.produces.contains(fact) && !(satisfied_earlier && producer > consumer)
                {
                    consumers[producer].push(consumer);
                    indegree[consumer] += 1;
                }
            }
        }
    }
    let mut ready: Vec<usize> = (0..table.len())
        .filter(|index| indegree[*index] == 0)
        .collect();
    let mut drained = 0usize;
    while let Some(node) = ready.pop() {
        drained += 1;
        for &next in &consumers[node] {
            indegree[next] -= 1;
            if indegree[next] == 0 {
                ready.push(next);
            }
        }
    }
    if drained < table.len() {
        let stuck = table
            .iter()
            .zip(indegree.iter())
            .find(|(_, degree)| **degree > 0)
            .map(|(pass, _)| pass)
            .expect("a table that did not drain has a pass left with a prerequisite");
        return Err(Error::invalid_input(
            IR_PASS_GRAPH_CYCLE,
            format!(
                "the pass table has a dependency cycle: the pass `{}` (phase `{}`) is part of a \
                 cycle or depends on one, so its `requires` can never all be available",
                stuck.name,
                stuck.phase.code(),
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every fact with the wire name it must have, in ledger order: a new fact has to be added
    /// here, which is where its name and its index are pinned against the exhaustive
    /// `fact_index` match below.
    const FACTS: [(FactKind, &str); 9] = [
        (FactKind::Instructions, "instructions"),
        (FactKind::ExceptionTable, "exception_table"),
        (FactKind::ThrowSites, "throw_sites"),
        (FactKind::RawCfg, "raw_cfg"),
        (FactKind::CallContexts, "call_contexts"),
        (FactKind::CanonicalCfg, "canonical_cfg"),
        (FactKind::Frames, "frames"),
        (FactKind::Ssa, "ssa"),
        (FactKind::Effects, "effects"),
    ];

    /// Position of one phase in the phase order; exhaustive, so a new phase has to be given a
    /// position here instead of silently shifting the others.
    fn phase_index(phase: IrPhase) -> usize {
        match phase {
            IrPhase::RawFacts => 0,
            IrPhase::RawCfg => 1,
            IrPhase::LegacyNormalization => 2,
            IrPhase::CanonicalCfg => 3,
            IrPhase::Frame => 4,
            IrPhase::Ssa => 5,
        }
    }

    /// Position of one fact in the ledger; exhaustive for the same reason as [`phase_index`].
    fn fact_index(fact: FactKind) -> usize {
        match fact {
            FactKind::Instructions => 0,
            FactKind::ExceptionTable => 1,
            FactKind::ThrowSites => 2,
            FactKind::RawCfg => 3,
            FactKind::CallContexts => 4,
            FactKind::CanonicalCfg => 5,
            FactKind::Frames => 6,
            FactKind::Ssa => 7,
            FactKind::Effects => 8,
        }
    }

    /// Position of one budget class in a declaration; exhaustive for the same reason as
    /// [`phase_index`], so a new counted dimension cannot be added without saying where it
    /// belongs.
    fn class_index(class: PassBudgetClass) -> usize {
        match class {
            PassBudgetClass::Blocks => 0,
            PassBudgetClass::Steps => 1,
            PassBudgetClass::Clones => 2,
        }
    }

    /// The single pass of one phase of the real table.
    fn pass_of(phase: IrPhase) -> &'static PassDescriptor {
        PASSES
            .iter()
            .find(|pass| pass.phase == phase)
            .expect("the table defines every phase")
    }

    /// One pass of a throwaway table; only the fields the checks read are parameterized.
    fn descriptor(
        phase: IrPhase,
        name: &'static str,
        requires: &'static [FactKind],
        produces: &'static [FactKind],
        invalidates: &'static [FactKind],
    ) -> PassDescriptor {
        PassDescriptor {
            phase,
            name,
            requires,
            produces,
            invalidates,
            budget: &[PassBudgetClass::Steps],
        }
    }

    fn invalid_input_code(error: &Error) -> Option<&str> {
        match error {
            Error::InvalidInput { code, .. } => Some(code),
            _ => None,
        }
    }

    fn invalid_input_message(error: &Error) -> &str {
        match error {
            Error::InvalidInput { message, .. } => message,
            _ => "",
        }
    }

    #[test]
    fn the_fact_vocabulary_owns_one_ledger_index_and_one_name_each() {
        assert_eq!(FACTS.len(), FactKind::COUNT);
        let mut indexes = Vec::new();
        for (fact, code) in FACTS {
            assert_eq!(fact.code(), code, "{code} names itself");
            assert_eq!(fact.index(), fact_index(fact), "{code} is indexed here");
            indexes.push(fact.index());
        }
        indexes.sort_unstable();
        assert_eq!(
            indexes,
            (0..FactKind::COUNT).collect::<Vec<_>>(),
            "every ledger slot is the slot of exactly one fact"
        );
    }

    /// One row of the expected table: the fields 3.2 fixes for a pass.
    struct ExpectedPass {
        name: &'static str,
        budget: &'static [PassBudgetClass],
        requires: &'static [FactKind],
        produces: &'static [FactKind],
        invalidates: &'static [FactKind],
    }

    #[test]
    fn the_table_declares_the_fixed_dependencies_of_every_phase() {
        // The table is 3.2's contract: one pass per phase, in phase order, and each pass's
        // `requires`/`produces`/`invalidates`/budget as the design fixes them.
        let expected = [
            ExpectedPass {
                name: "raw_facts",
                budget: &[],
                requires: &[],
                produces: &[FactKind::Instructions, FactKind::ExceptionTable],
                invalidates: &[],
            },
            ExpectedPass {
                name: "raw_cfg",
                budget: &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
                requires: &[FactKind::Instructions, FactKind::ExceptionTable],
                produces: &[FactKind::RawCfg, FactKind::ThrowSites, FactKind::Effects],
                invalidates: &[],
            },
            ExpectedPass {
                name: "legacy_normalization",
                budget: &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
                requires: &[
                    FactKind::Instructions,
                    FactKind::ExceptionTable,
                    FactKind::RawCfg,
                    FactKind::ThrowSites,
                    FactKind::Effects,
                ],
                produces: &[FactKind::CallContexts],
                invalidates: &[],
            },
            ExpectedPass {
                name: "canonical_cfg",
                budget: &[
                    PassBudgetClass::Blocks,
                    PassBudgetClass::Steps,
                    PassBudgetClass::Clones,
                ],
                requires: &[
                    FactKind::Instructions,
                    FactKind::ExceptionTable,
                    FactKind::RawCfg,
                    FactKind::ThrowSites,
                    FactKind::CallContexts,
                ],
                produces: &[FactKind::CanonicalCfg],
                invalidates: &[FactKind::Effects, FactKind::Frames, FactKind::Ssa],
            },
            ExpectedPass {
                name: "frame",
                budget: &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
                requires: &[
                    FactKind::Instructions,
                    FactKind::ExceptionTable,
                    FactKind::CanonicalCfg,
                ],
                produces: &[FactKind::Frames],
                invalidates: &[],
            },
            ExpectedPass {
                name: "ssa",
                budget: &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
                requires: &[
                    FactKind::Instructions,
                    FactKind::CanonicalCfg,
                    FactKind::Frames,
                ],
                produces: &[FactKind::Ssa, FactKind::Effects],
                invalidates: &[],
            },
        ];
        assert_eq!(PASSES.len(), expected.len());
        for (pass, row) in PASSES.iter().zip(expected) {
            let name = row.name;
            assert_eq!(pass.name, name);
            assert_eq!(
                pass.phase.code(),
                name,
                "a pass runs in the phase it is named for"
            );
            assert_eq!(
                pass.budget, row.budget,
                "{name} declares the dimensions it bills"
            );
            assert_eq!(pass.requires, row.requires, "{name} requires");
            assert_eq!(pass.produces, row.produces, "{name} produces");
            assert_eq!(pass.invalidates, row.invalidates, "{name} invalidates");
        }
        // Phases ascend and carry exactly one pass each, so a request's phase prefix and the
        // passes it schedules are the same thing — which is what lets the report (1.1) and the
        // startup validation (3.2) agree without a second numbering.
        assert_eq!(
            PASSES
                .iter()
                .map(|pass| phase_index(pass.phase))
                .collect::<Vec<_>>(),
            (0..AnalysisStage::ALL.len()).collect::<Vec<_>>(),
        );
        // The vocabulary is the table's: no fact is decorative.
        for (fact, code) in FACTS {
            assert!(
                PASSES
                    .iter()
                    .any(|pass| pass.produces.contains(&fact) || pass.requires.contains(&fact)),
                "{code} is neither produced nor required by any pass"
            );
        }
    }

    /// The counted dimensions 3.2 fixes for each phase.
    ///
    /// `Blocks` is `IrItems` + `IrEdges`, `Steps` is `AnalysisSteps` and `Clones` is
    /// `NormalizationClones`. This slice runs no pass, so the declaration is what can be
    /// checked here; the entry-by-entry equality with the dimensions a pass really charges is
    /// each pass's own case from 3.3 on, and a declared dimension it never charges (or a
    /// dimension it charges undeclared) is a defect there.
    const DECLARED_BUDGETS: [(IrPhase, &[PassBudgetClass]); 6] = [
        // The reader already charged `ClassBytes`/`AttributeBytes`/`CodeBytes` for the bytes
        // this pass projects into the fact set, so its set is empty.
        (IrPhase::RawFacts, &[]),
        // Blocks and edges are IR storage items and the worklist walk is analysis work: one
        // pass billing two dimensions is why the field is a set and not a single class.
        (
            IrPhase::RawCfg,
            &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
        ),
        (
            IrPhase::LegacyNormalization,
            &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
        ),
        (
            IrPhase::CanonicalCfg,
            &[
                PassBudgetClass::Blocks,
                PassBudgetClass::Steps,
                PassBudgetClass::Clones,
            ],
        ),
        (
            IrPhase::Frame,
            &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
        ),
        (
            IrPhase::Ssa,
            &[PassBudgetClass::Blocks, PassBudgetClass::Steps],
        ),
    ];

    #[test]
    fn every_pass_declares_exactly_the_dimensions_it_bills() {
        // The declaration is a set of dimensions: listing one twice would be a duplicated
        // charge, and the order the classes are written in carries no meaning.
        for pass in PASSES {
            let mut declared: Vec<usize> = pass.budget.iter().copied().map(class_index).collect();
            declared.sort_unstable();
            declared.dedup();
            assert_eq!(
                declared.len(),
                pass.budget.len(),
                "{} declares each counted dimension once",
                pass.name
            );
        }
        assert_eq!(PASSES.len(), DECLARED_BUDGETS.len());
        for (pass, (phase, budget)) in PASSES.iter().zip(DECLARED_BUDGETS) {
            assert_eq!(pass.phase, phase, "the table declares {phase:?} here");
            assert_eq!(
                pass.budget, budget,
                "{} declares exactly the dimensions it bills",
                pass.name
            );
        }
    }

    /// The phases this build implements: 3.3 runs `raw_facts` and `raw_cfg`, 3.4 adds the call
    /// contexts of `legacy_normalization`, 3.5 the canonical CFG of `canonical_cfg`, 4.1 the
    /// frames of `frame`, and 4.3 the names of `ssa`.
    const IMPLEMENTED_PHASES: [IrPhase; 6] = [
        IrPhase::RawFacts,
        IrPhase::RawCfg,
        IrPhase::LegacyNormalization,
        IrPhase::CanonicalCfg,
        IrPhase::Frame,
        IrPhase::Ssa,
    ];

    #[test]
    fn the_implemented_phases_are_a_prefix_of_the_table() {
        // The property the pipeline relies on: a later phase never runs while an earlier one
        // is missing, so a request stops at exactly one point. A build that implemented
        // `frame` but not `canonical_cfg` would fail here.
        for pass in PASSES {
            assert_eq!(
                implemented(pass.phase),
                IMPLEMENTED_PHASES.contains(&pass.phase),
                "{} is implemented in this build",
                pass.name
            );
        }
        let implemented_count = PASSES
            .iter()
            .take_while(|pass| implemented(pass.phase))
            .count();
        assert_eq!(implemented_count, IMPLEMENTED_PHASES.len());
        assert!(
            PASSES[implemented_count..]
                .iter()
                .all(|pass| !implemented(pass.phase)),
            "the implemented phases are a prefix, not a set with holes"
        );
    }

    #[test]
    fn every_stage_maps_to_its_own_phase_in_order() {
        for (index, stage) in AnalysisStage::ALL.into_iter().enumerate() {
            let phase = IrPhase::from_stage(stage);
            assert_eq!(phase.stage(), stage, "{stage:?} round-trips");
            assert_eq!(
                phase_index(phase),
                index,
                "{stage:?} is phase number {index}"
            );
            // The contract numbers the phases from 1 upwards, so a renumbering fails here.
            assert_eq!(
                phase as usize,
                index + 1,
                "{stage:?} keeps its phase number"
            );
            // The phase name is the name the request serializes: one vocabulary, not two.
            assert_eq!(
                serde_json::to_string(&stage).expect("a stage serializes"),
                format!("\"{}\"", phase.code()),
            );
        }
        let mut mapped: Vec<usize> = AnalysisStage::ALL
            .into_iter()
            .map(|stage| phase_index(IrPhase::from_stage(stage)))
            .collect();
        mapped.sort_unstable();
        assert_eq!(
            mapped,
            (0..AnalysisStage::ALL.len()).collect::<Vec<_>>(),
            "every phase is the phase of exactly one stage"
        );
    }

    #[test]
    fn every_non_empty_stage_set_schedules_the_table_prefix_of_its_last_phase() {
        for mask in 1u32..(1u32 << AnalysisStage::ALL.len()) {
            let stages: Vec<AnalysisStage> = AnalysisStage::ALL
                .into_iter()
                .enumerate()
                .filter(|(index, _)| mask & (1 << index) != 0)
                .map(|(_, stage)| stage)
                .collect();
            let last = stages
                .iter()
                .copied()
                .map(IrPhase::from_stage)
                .max()
                .expect("a non-empty mask has a stage");
            let scheduled =
                validate_requested_stages(&stages).expect("a non-empty stage set is valid");
            assert_eq!(
                scheduled,
                &PASSES[..=phase_index(last)],
                "{stages:?} schedules its own prerequisites"
            );
            // The scheduled phases are the phases of the requested prefix of the phase order,
            // which is the rule the report lists as scheduled stages.
            assert_eq!(
                scheduled
                    .iter()
                    .map(|pass| pass.phase.code())
                    .collect::<Vec<_>>(),
                AnalysisStage::ALL[..=phase_index(last)]
                    .iter()
                    .map(|stage| IrPhase::from_stage(*stage).code())
                    .collect::<Vec<_>>(),
                "{stages:?} ends at its last requested phase"
            );
        }
        // An empty stage set is not a run; `crate::ir::validate_request` owns that code and
        // this function only has no schedule to check.
        assert_eq!(validate_requested_stages(&[]).expect("no stages"), &[]);
    }

    #[test]
    fn a_fact_produced_twice_does_not_cycle_when_the_earlier_producer_serves_it() {
        // `Effects` have two producers on purpose: the raw graph's by `raw_cfg` and, after the
        // canonicalization invalidated them, the canonical graph's by `ssa`. The consumer between
        // the two — `legacy_normalization` — is served by the earlier one, so the later producer
        // is not a second prerequisite and the table must not be rejected: the graph answers
        // whether the declared order can satisfy every `require`, and here it can.
        let twice_produced = vec![
            descriptor(
                IrPhase::RawFacts,
                "p1",
                &[],
                &[FactKind::Instructions, FactKind::Effects],
                &[],
            ),
            descriptor(
                IrPhase::RawCfg,
                "c",
                &[FactKind::Instructions, FactKind::Effects],
                &[FactKind::RawCfg],
                &[],
            ),
            descriptor(
                IrPhase::Ssa,
                "p2",
                &[FactKind::RawCfg],
                &[FactKind::Effects],
                &[],
            ),
        ];
        assert_eq!(
            validate_schedule(&twice_produced, IrPhase::Ssa)
                .expect("the earlier producer serves the consumer")
                .len(),
            3
        );
        // The real table is that shape, and this is the property a table-wide cycle rejection
        // must not take away: `Effects` has a producer on either side of its consumer, and the
        // consumer is served by the earlier one.
        let producers: Vec<usize> = PASSES
            .iter()
            .enumerate()
            .filter(|(_, pass)| pass.produces.contains(&FactKind::Effects))
            .map(|(index, _)| index)
            .collect();
        let consumer = PASSES
            .iter()
            .position(|pass| pass.name == "legacy_normalization")
            .expect("the real table declares the consumer");
        assert_eq!(
            producers.len(),
            2,
            "the raw graph's effects and the canonical graph's are produced by two passes"
        );
        assert!(
            producers[0] < consumer && consumer < producers[1],
            "the consumer sits between the two producers of `effects`"
        );

        // Take the earlier producer away and the same `require` is no longer satisfiable in
        // declaration order: the later producer becomes the backward edge again, and the fault is
        // the missing prerequisite at the moment the pass runs — not a cycle, because nothing in
        // the table needs what the consumer publishes in return.
        let only_later_producer = vec![
            descriptor(IrPhase::RawFacts, "a", &[], &[FactKind::Instructions], &[]),
            descriptor(IrPhase::RawCfg, "c", &[FactKind::Effects], &[], &[]),
            descriptor(IrPhase::Ssa, "p2", &[], &[FactKind::Effects], &[]),
        ];
        let error = validate_schedule(&only_later_producer, IrPhase::Ssa)
            .expect_err("only a later pass produces `effects`");
        assert_eq!(
            invalid_input_code(&error),
            Some(IR_PASS_PREREQUISITE_MISSING),
            "a backward edge is a missing prerequisite, not a cycle"
        );
        assert!(invalid_input_message(&error).contains("`c`"), "{error}");

        // …while a genuine mutual dependency keeps its cycle: `c` needs what `d` publishes and
        // `d` needs what `c` publishes, so neither `require` has an earlier producer and both
        // directions stay edges — no declaration order satisfies them.
        let mutual = vec![
            descriptor(
                IrPhase::RawFacts,
                "c",
                &[FactKind::Effects],
                &[FactKind::RawCfg],
                &[],
            ),
            descriptor(
                IrPhase::RawCfg,
                "d",
                &[FactKind::RawCfg],
                &[FactKind::Effects],
                &[],
            ),
        ];
        let error = validate_schedule(&mutual, IrPhase::RawCfg).expect_err("a real cycle");
        assert_eq!(invalid_input_code(&error), Some(IR_PASS_GRAPH_CYCLE));
    }

    #[test]
    fn a_scheduled_pass_without_its_prerequisite_is_rejected() {
        // `b` requires a fact only the later `c` publishes. The dependency graph has no cycle
        // (a backward edge is not a cycle), so the fault is the missing prerequisite at the
        // moment `b` would run — in any prefix that schedules it.
        let table = vec![
            descriptor(IrPhase::RawFacts, "a", &[], &[FactKind::Instructions], &[]),
            descriptor(
                IrPhase::RawCfg,
                "b",
                &[FactKind::Instructions, FactKind::Effects],
                &[FactKind::RawCfg],
                &[],
            ),
            descriptor(IrPhase::Ssa, "c", &[], &[FactKind::Effects], &[]),
        ];
        for last in [IrPhase::RawCfg, IrPhase::Ssa] {
            let error = validate_schedule(&table, last).expect_err("`effects` is not there yet");
            assert_eq!(
                invalid_input_code(&error),
                Some(IR_PASS_PREREQUISITE_MISSING)
            );
            let message = invalid_input_message(&error);
            assert!(message.contains("`b`"), "{message}");
            assert!(message.contains("`effects`"), "{message}");
        }
        // The diagnostic states how far the run got: that is what a stopped request reports as
        // the last valid phase instead of publishing half-initialized facts.
        let error = validate_schedule(&table, IrPhase::Ssa).expect_err("still missing");
        assert!(
            invalid_input_message(&error).contains("last completed phase: raw_facts"),
            "{error}"
        );

        // The real table has no such gap: every phase prefix of it validates.
        for last in [
            IrPhase::RawFacts,
            IrPhase::RawCfg,
            IrPhase::LegacyNormalization,
            IrPhase::CanonicalCfg,
            IrPhase::Frame,
            IrPhase::Ssa,
        ] {
            assert!(
                validate_schedule(PASSES, last).is_ok(),
                "the real table schedules up to {last:?}"
            );
        }
    }

    #[test]
    fn a_pass_that_requires_effects_no_pass_produces_is_rejected() {
        // Half of the consumer rule of 3.2: a pass that reads a fact must name it in `requires`,
        // and naming a fact no pass of the schedule publishes is the missing prerequisite — not
        // a silent read of an absent fact. An unproduced fact stays `NotProduced` (an
        // `invalidates` entry for it would be a no-op), which is why the fault is named for what
        // it is instead of being reported as a stale fact.
        let table = vec![descriptor(
            IrPhase::RawFacts,
            "a",
            &[FactKind::Effects],
            &[FactKind::Instructions],
            &[],
        )];
        let error = validate_schedule(&table, IrPhase::RawFacts).expect_err("nothing produces it");
        assert_eq!(
            invalid_input_code(&error),
            Some(IR_PASS_PREREQUISITE_MISSING),
            "an absent fact is a missing prerequisite, not a stale one"
        );
        let message = invalid_input_message(&error);
        assert!(message.contains("`a`"), "{message}");
        assert!(message.contains("`effects`"), "{message}");
        assert!(message.contains("no phase completed yet"), "{message}");
    }

    #[test]
    fn the_effects_consumer_is_refused_while_the_effects_are_stale() {
        // The other half, on the real table: `Effects` are the effect facts of the graph they
        // were derived from — the raw graph's while `raw_cfg` produced them, the canonical
        // graph's after `ssa` recomputed them — and `legacy_normalization` reads the raw ones,
        // so the pass declares `Effects`. 3.5 re-enters the canonicalization with a larger clone
        // budget (a pass of an earlier phase applied again, which 3.2 admits), and every fact
        // derived from the replaced graph goes stale. The consumer must then be refused while the
        // effects are stale instead of reading an input that describes a graph that is gone —
        // which is what declaring them buys: a consumer the ledger cannot see is a consumer
        // invalidation does not protect.
        let mut ledger = FactLedger::new();
        for pass in validate_requested_stages(&AnalysisStage::ALL).expect("the pipeline schedules")
        {
            ledger.apply(pass).expect("the real table runs in order");
        }

        let legacy = pass_of(IrPhase::LegacyNormalization);
        ledger
            .apply(pass_of(IrPhase::CanonicalCfg))
            .expect("the canonicalization still has its inputs");
        assert_eq!(ledger.state(FactKind::Effects), FactState::Stale);
        // The state after the retry, which a refused pass must leave untouched.
        let after_the_retry = ledger.clone();

        let error = ledger
            .apply(legacy)
            .expect_err("the effects of the replaced graph are not the pass's input");
        assert_eq!(
            invalid_input_code(&error),
            Some(IR_STALE_FACT),
            "a stale fact is the stale code, not the missing prerequisite"
        );
        let message = invalid_input_message(&error);
        assert!(message.contains("`legacy_normalization`"), "{message}");
        assert!(message.contains("`effects`"), "{message}");
        // The refused pass published nothing: no call contexts of an input it could not obtain,
        // and no change to the run's progress either. (`CallContexts` stays live from the first
        // run — the refused pass did not get to publish a new value — and the equality is what
        // says so, fact by fact and phase by phase.)
        assert_eq!(ledger.state(FactKind::CallContexts), FactState::Live);
        assert_eq!(
            ledger, after_the_retry,
            "a refused pass changes nothing at all"
        );

        // Recomputation is the way back, and the recomputer is the pass 3.2 names: the SSA
        // republishes the effect facts of the canonical graph…
        ledger
            .apply(pass_of(IrPhase::Frame))
            .expect("the frames of the canonical graph");
        ledger
            .apply(pass_of(IrPhase::Ssa))
            .expect("the SSA republishes the canonical effects");
        assert_eq!(ledger.state(FactKind::Effects), FactState::Live);
        // …and only then does the consumer of the fact have its input again.
        ledger
            .apply(legacy)
            .expect("the recomputed effects are the consumer's input");
        assert_eq!(ledger.state(FactKind::CallContexts), FactState::Live);
    }

    #[test]
    fn a_table_out_of_phase_order_is_rejected() {
        let reversed = vec![
            descriptor(IrPhase::RawCfg, "b", &[], &[FactKind::RawCfg], &[]),
            descriptor(IrPhase::RawFacts, "a", &[], &[FactKind::Instructions], &[]),
        ];
        let error = validate_schedule(&reversed, IrPhase::RawCfg).expect_err("b follows a");
        assert_eq!(invalid_input_code(&error), Some(IR_PASS_ORDER_INVALID));
        // The diagnostic names the pass that breaks the order and the phase it follows.
        let message = invalid_input_message(&error);
        assert!(message.contains("`a`"), "{message}");
        assert!(message.contains("phase `raw_facts`"), "{message}");
        assert!(message.contains("the phase `raw_cfg`"), "{message}");

        // A phase that takes two passes is not a phase that goes backwards: the table is
        // ordered by phase, and the two passes of one phase are ordered by their own
        // dependency — which the prefix walk checks like any other prerequisite.
        let two_steps = vec![
            descriptor(IrPhase::RawFacts, "a", &[], &[FactKind::Instructions], &[]),
            descriptor(
                IrPhase::RawFacts,
                "b",
                &[FactKind::Instructions],
                &[FactKind::ExceptionTable],
                &[],
            ),
        ];
        assert_eq!(
            validate_schedule(&two_steps, IrPhase::RawFacts)
                .expect("two passes of one phase are legal")
                .len(),
            2
        );
        // …and the same second pass, moved in front of its producer, is a missing prerequisite
        // rather than a silent reordering.
        let inverted = vec![
            descriptor(
                IrPhase::RawFacts,
                "b",
                &[FactKind::Instructions],
                &[FactKind::ExceptionTable],
                &[],
            ),
            descriptor(IrPhase::RawFacts, "a", &[], &[FactKind::Instructions], &[]),
        ];
        let error = validate_schedule(&inverted, IrPhase::RawFacts)
            .expect_err("b requires what a publishes");
        assert_eq!(
            invalid_input_code(&error),
            Some(IR_PASS_PREREQUISITE_MISSING)
        );
    }

    #[test]
    fn a_table_that_cycles_is_rejected() {
        // Two passes that each need what the other publishes: neither can run first.
        let mutual = vec![
            descriptor(
                IrPhase::RawFacts,
                "a",
                &[FactKind::CallContexts],
                &[FactKind::Instructions],
                &[],
            ),
            descriptor(
                IrPhase::RawCfg,
                "b",
                &[FactKind::Instructions],
                &[FactKind::CallContexts],
                &[],
            ),
        ];
        let error = validate_schedule(&mutual, IrPhase::RawCfg).expect_err("a cycle");
        assert_eq!(invalid_input_code(&error), Some(IR_PASS_GRAPH_CYCLE));
        let message = invalid_input_message(&error);
        assert!(message.contains("`a`"), "{message}");
        assert!(message.contains("`raw_facts`"), "{message}");

        // The `requires`/`produces` conflict of one pass is the smallest cycle of the same kind.
        let conflicted = vec![descriptor(
            IrPhase::RawFacts,
            "a",
            &[FactKind::RawCfg],
            &[FactKind::RawCfg],
            &[],
        )];
        let error = validate_schedule(&conflicted, IrPhase::RawFacts).expect_err("a self cycle");
        assert_eq!(invalid_input_code(&error), Some(IR_PASS_GRAPH_CYCLE));
    }

    #[test]
    fn a_cfg_change_invalidates_the_facts_derived_from_the_old_graph() {
        // The whole pipeline runs: every fact of the vocabulary is live.
        let mut ledger = FactLedger::new();
        for pass in validate_requested_stages(&AnalysisStage::ALL).expect("the pipeline schedules")
        {
            ledger.apply(pass).expect("the real table runs in order");
        }
        for (fact, code) in FACTS {
            assert_eq!(ledger.state(fact), FactState::Live, "{code} is live");
        }

        // 3.5 re-enters the canonicalization (a bounded attempt retried with more clone budget
        // rebuilds the blocks and the exception edges), so every fact derived from the old
        // graph goes stale while the canonical CFG itself is the newest value.
        let canonical = pass_of(IrPhase::CanonicalCfg);
        ledger
            .apply(canonical)
            .expect("the canonicalization still has its inputs");
        assert_eq!(ledger.state(FactKind::Effects), FactState::Stale);
        assert_eq!(ledger.state(FactKind::Frames), FactState::Stale);
        assert_eq!(ledger.state(FactKind::Ssa), FactState::Stale);
        assert_eq!(ledger.state(FactKind::CanonicalCfg), FactState::Live);

        // Using the frames of the old graph is a structured error, not a silent reuse…
        let ssa = pass_of(IrPhase::Ssa);
        let error = ledger
            .apply(ssa)
            .expect_err("the old frames are not the SSA's input");
        assert_eq!(invalid_input_code(&error), Some(IR_STALE_FACT));
        let message = invalid_input_message(&error);
        assert!(message.contains("`ssa`"), "{message}");
        assert!(message.contains("`frames`"), "{message}");
        // …and the rejected pass published nothing: the SSA stays stale rather than becoming an
        // old value that is presented as the current analysis.
        assert_eq!(ledger.state(FactKind::Ssa), FactState::Stale);

        // Recomputation is the way back: the frames for the new graph…
        ledger
            .apply(pass_of(IrPhase::Frame))
            .expect("the frames are recomputed");
        assert_eq!(ledger.state(FactKind::Frames), FactState::Live);
        // …and then the SSA, which republishes the effect facts of the canonical graph.
        ledger.apply(ssa).expect("the SSA is recomputed");
        assert_eq!(ledger.state(FactKind::Ssa), FactState::Live);
        assert_eq!(ledger.state(FactKind::Effects), FactState::Live);
    }

    #[test]
    fn re_entering_an_earlier_phase_never_lowers_the_last_completed_phase() {
        // 3.5 retries the canonicalization with a larger clone budget, so a pass of an earlier
        // phase is applied again after the pipeline ran. The ledger must still report the
        // highest phase the run completed — 5.1 reads it when it assembles the stage results —
        // and not whichever pass happened to be applied last.
        let mut ledger = FactLedger::new();
        for pass in validate_requested_stages(&AnalysisStage::ALL).expect("the pipeline schedules")
        {
            ledger.apply(pass).expect("the real table runs in order");
        }
        let pipeline_end = ledger.last_completed;
        assert_eq!(
            pipeline_end,
            Some(IrPhase::Ssa),
            "the whole pipeline completed"
        );

        // The retry of the canonicalization really is a pass of the pipeline: its
        // invalidations take effect as always…
        ledger
            .apply(pass_of(IrPhase::CanonicalCfg))
            .expect("the canonicalization still has its inputs");
        assert_eq!(ledger.state(FactKind::Frames), FactState::Stale);
        assert_eq!(
            ledger.last_completed, pipeline_end,
            "retrying an earlier phase does not lower the last completed phase"
        );

        // …and neither does finishing a later pass of a phase the run already passed: only a
        // phase beyond the highest completed one moves the value.
        ledger
            .apply(pass_of(IrPhase::Frame))
            .expect("the frames are recomputed");
        assert_eq!(
            ledger.last_completed, pipeline_end,
            "recomputing inside the run does not lower the last completed phase"
        );
    }

    #[test]
    fn a_pass_that_cannot_run_publishes_nothing_and_keeps_the_last_valid_phase() {
        // 3.2 has no pass to run yet, so the isolation rule is pinned where it is decided: the
        // ledger. A pass that cannot obtain its inputs neither publishes a fact nor advances
        // the last valid phase — the equality below covers both, because the ledger carries the
        // phase of its last completed pass.
        let mut ledger = FactLedger::new();
        ledger
            .apply(pass_of(IrPhase::RawFacts))
            .expect("the first pass has no prerequisite");
        let after_first = ledger.clone();

        let error = ledger
            .apply(pass_of(IrPhase::CanonicalCfg))
            .expect_err("the canonical CFG is not there yet");
        assert_eq!(
            invalid_input_code(&error),
            Some(IR_PASS_PREREQUISITE_MISSING)
        );
        let message = invalid_input_message(&error);
        assert!(message.contains("`canonical_cfg`"), "{message}");
        assert!(
            message.contains("last completed phase: raw_facts"),
            "{message}"
        );
        assert_eq!(
            ledger, after_first,
            "a stopped pass publishes nothing and does not advance the last valid phase"
        );

        // The ledger is not poisoned by the stop: a pass whose inputs are there still runs.
        ledger
            .apply(pass_of(IrPhase::RawCfg))
            .expect("the raw CFG still has its inputs");
        assert_eq!(ledger.state(FactKind::RawCfg), FactState::Live);
    }
}
