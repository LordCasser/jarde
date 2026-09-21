//! The one place a recovery request states *which* optional evidence it wants delivered, and the one
//! place the report answers what it delivered (change `add-demand-driven-core-results`, D1).
//!
//! # Two axes, both already the caller's
//!
//! A recovery request already states its target (the payload it presents) and its rules (the
//! profile). This module adds the third statement the P3 report used to make for the caller: the
//! **evidence selection** — which categories of *optional* detail the run materializes — and the
//! optional **driver BCI range** those categories are restricted to. Neither axis decides whether a
//! rule runs: every premise, refusal and semantic check the presentation performs is performed for
//! every selection, and only the publication of the detail records is gated (see [`crate::report`]).
//!
//! # Essential is the empty selection, not a smaller algorithm
//!
//! [`RecoveryEvidenceRequest::essential`] is the empty kind set and no range: the ordinary recovery
//! a caller gets without asking for anything. It is *not* a shortcut through the recovery — the
//! regions, the naming, the rules and the emitter write exactly what they write for a full-evidence
//! request, and the only difference is that the owning records of the unselected categories are not
//! built, are not copied into the report and are not kept.
//!
//! [`RecoveryEvidenceRequest::all`] is the other end: every category **this entry materializes**.
//! `ReadDetails` is deliberately not one of them yet — the read evidence a presentation consumed is
//! published beside the report by the entry that performed the read, and its *expansion* into the
//! report is a later phase's work — so asking for it is refused explicitly ([`UNSUPPORTED_KIND_CODE`])
//! rather than answered with nothing.
//!
//! # The driver range selects evidence, never analysis
//!
//! A range selects the records that *intersect* it, together with the evidence each kept record
//! already carries: a record is kept or dropped as a unit, and the origins a record states for
//! itself are its closure. Three consequences are stated here because a caller has to be able to rely
//! on them:
//!
//! * a callee's own bytecode index is **not** a driver position: `access$100`'s field read at BCI 4
//!   does not put an accessor record into a selection of `[4, 5)` of the driver body, and a record
//!   the driver range does select keeps the callee's full identity as the proof of what it says;
//! * a record that states no bytecode index of its own — the member's declaration, the bridge
//!   verdict — is not a position record: the range neither selects nor drops it, and a request that
//!   selects such a category over a range gets that category whole;
//! * a category whose records are per **local slot** rather than per bytecode index (the names the
//!   presentation decided) is delivered whole for the same reason.
//!
//! The range is validated against the body the request really decoded: `[x, x)` at an instruction
//! boundary (the end of the code included) is a legal empty selection, and a reversed range, a
//! boundary the body does not have, or a range that reaches past the decoded instructions is refused
//! with a stable code instead of being widened to the whole method.
//!
//! # Four states, and no empty set standing in for one of them
//!
//! The report answers with a **fixed-size** selection/status list: one entry per category, in
//! [`RecoveryEvidenceKind::ALL`] order, whatever the selection was. [`EvidenceState`] states what
//! happened to that category in this run:
//!
//! | state | means |
//! | --- | --- |
//! | [`EvidenceState::NotRequested`] | the request did not select this category; its payload is empty *by selection* |
//! | [`EvidenceState::Complete`] | selected and delivered in full, and an empty payload is a legal empty result |
//! | [`EvidenceState::Partial`] | selected, and the delivery stopped with `delivered` records materialized |
//! | [`EvidenceState::NotPerformed`] | selected, and nothing of it was materialized: the run stopped before the evidence phase, or the selection was refused |
//!
//! `None`, an empty `Vec` or a zero count alone is therefore never the answer to "was this asked
//! for" — the status is. [`RecoveryReport::evidence`](crate::RecoveryReport::evidence) carries the
//! list beside the payload, and the report checks the two against each other before it is handed
//! out ([`RecoveryEvidence::agrees_with`]).

use std::collections::BTreeSet;

use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::MethodCodeFacts;
use serde::{Deserialize, Serialize};

/// The code of a refusal of a category this entry does not materialize.
pub const UNSUPPORTED_KIND_CODE: &str = "jre_evidence_kind_unsupported";
/// The code of a refusal of a driver range that is not a range of this body's instructions.
pub const RANGE_REFUSAL_CODE: &str = "jre_evidence_range_invalid";
/// The code of a refusal of a driver range that cannot be a selection at all — reversed, or stated
/// without any category to select.
pub const RANGE_SHAPE_CODE: &str = "jre_evidence_range_shape";

/// One half-open range of the **driver method's own** bytecode indexes, `[start, end)`.
///
/// The range is a selection of evidence and never a selection of analysis: a body is analysed and
/// recovered as a whole for every range, and the range decides which of the records the run produced
/// are materialized (see [`RecoveryEvidenceRequest`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BytecodeRange {
    start: u32,
    end: u32,
}

impl BytecodeRange {
    /// The range from `start` (inclusive) to `end` (exclusive).
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// The first bytecode index the range selects.
    pub fn start(&self) -> u32 {
        self.start
    }

    /// The first bytecode index the range does not select.
    pub fn end(&self) -> u32 {
        self.end
    }

    /// Whether the range selects no index at all. A legal empty selection: `[x, x)`.
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Whether one bytecode index is inside the range.
    pub fn contains(&self, bci: u32) -> bool {
        self.start <= bci && bci < self.end
    }

    /// Whether any index of `[from, to)` — one record's own span — is inside the range.
    pub fn intersects(&self, from: u32, to: u32) -> bool {
        from < self.end && self.start < to
    }

    /// Whether any index of `positions` is inside the range.
    pub fn intersects_any(&self, positions: impl IntoIterator<Item = u32>) -> bool {
        positions.into_iter().any(|bci| self.contains(bci))
    }
}

/// One category of optional recovery evidence.
///
/// The five are the categories the design of `add-demand-driven-core-results` fixes; four of them are
/// materialized by this layer's own report, and the fifth (a read detail) is the entry's read
/// evidence, whose expansion into a recovery report is a later phase's work.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryEvidenceKind {
    /// The segment table of the committed text: which byte range of the artifact came from where.
    SourceMap,
    /// The full region records, including the ones the artifact only quotes as fallbacks.
    RegionDetails,
    /// The rule records themselves: lambdas, concatenations, accessors, bridges, construction sites,
    /// field instructions, dispatch tables, constructor prologues and the member's declaration.
    RuleDetails,
    /// The names the presentation decided, with the spelling it could not write and the one it wrote
    /// instead.
    NameDetails,
    /// The read evidence this presentation consumed — not materialized by this entry yet.
    ReadDetails,
}

impl RecoveryEvidenceKind {
    /// Every category, in the order the report's status list states them.
    pub const ALL: [Self; 5] = [
        Self::SourceMap,
        Self::RegionDetails,
        Self::RuleDetails,
        Self::NameDetails,
        Self::ReadDetails,
    ];

    /// Every category this entry can materialize.
    pub const SUPPORTED: [Self; 4] = [
        Self::SourceMap,
        Self::RegionDetails,
        Self::RuleDetails,
        Self::NameDetails,
    ];

    /// The category's own name, as the request states it in a document.
    pub fn spell(self) -> &'static str {
        match self {
            Self::SourceMap => "source_map",
            Self::RegionDetails => "region_details",
            Self::RuleDetails => "rule_details",
            Self::NameDetails => "name_details",
            Self::ReadDetails => "read_details",
        }
    }

    /// The category one spelling names, or `None` for a spelling this vocabulary does not hold.
    ///
    /// The one place the names are read back, so an adapter cannot spell a category this layer does
    /// not know and cannot drop one it does.
    pub fn parse(spelling: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.spell() == spelling)
    }

    /// Its own index in [`Self::ALL`], which is also the index of its status entry.
    fn index(self) -> usize {
        match self {
            Self::SourceMap => 0,
            Self::RegionDetails => 1,
            Self::RuleDetails => 2,
            Self::NameDetails => 3,
            Self::ReadDetails => 4,
        }
    }
}

/// One recovery request's statement about which optional evidence it wants delivered.
///
/// The type is a plain value: an empty kind set with no range is [`Self::essential`], every kind this
/// entry materializes is [`Self::all`]. Nothing here is a per-rule filter — a request either wants a
/// category's records or does not, and no request can ask for a *decision* to be skipped.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryEvidenceRequest {
    /// The categories to materialize. Empty means none of them.
    #[serde(default)]
    kinds: BTreeSet<RecoveryEvidenceKind>,
    /// The driver bytecode range the positional categories are restricted to, when the request names
    /// one. `None` is the whole method.
    #[serde(default)]
    driver_bci_range: Option<BytecodeRange>,
}

impl RecoveryEvidenceRequest {
    /// The ordinary recovery request: no optional evidence at all.
    ///
    /// This is the default every entry of this crate presents under, and it is a *selection*: the run
    /// still performs every check and writes the same artifact it writes for [`Self::all`].
    pub fn essential() -> Self {
        Self::default()
    }

    /// Every category this entry materializes, over the whole method.
    pub fn all() -> Self {
        Self {
            kinds: RecoveryEvidenceKind::SUPPORTED.into_iter().collect(),
            driver_bci_range: None,
        }
    }

    /// The same request, asking for one more category.
    pub fn with_kind(mut self, kind: RecoveryEvidenceKind) -> Self {
        self.kinds.insert(kind);
        self
    }

    /// The same request, asking for exactly `kinds`.
    pub fn with_kinds(mut self, kinds: impl IntoIterator<Item = RecoveryEvidenceKind>) -> Self {
        self.kinds.extend(kinds);
        self
    }

    /// The same request, restricted to one driver bytecode range.
    pub fn with_driver_bci_range(mut self, range: BytecodeRange) -> Self {
        self.driver_bci_range = Some(range);
        self
    }

    /// The categories this request selects.
    pub fn kinds(&self) -> impl Iterator<Item = RecoveryEvidenceKind> + '_ {
        self.kinds.iter().copied()
    }

    /// Whether this request selects one category.
    pub fn requests(&self, kind: RecoveryEvidenceKind) -> bool {
        self.kinds.contains(&kind)
    }

    /// Whether this request selects nothing optional — the ordinary recovery.
    pub fn is_essential(&self) -> bool {
        self.kinds.is_empty() && self.driver_bci_range.is_none()
    }

    /// The driver range this request restricts its positional evidence to, when it states one.
    pub fn driver_bci_range(&self) -> Option<BytecodeRange> {
        self.driver_bci_range
    }

    /// Whether this request states a selection at all, which is what makes it worth validating
    /// against a body.
    pub fn is_stated(&self) -> bool {
        !self.is_essential()
    }

    /// Whether this selection can be answered for one decoded body.
    ///
    /// The three refusals it states are the request's own: a category this entry does not
    /// materialize, a range stated without any category to select, and a range this body cannot
    /// support. `code` is the body the request really decoded: a range is only readable against the
    /// instructions that decode produced, so a range that needs the bytes is refused rather than
    /// widened — the read that produced them is part of this request either way.
    pub(crate) fn check(&self, code: &MethodCodeFacts) -> Result<(), EvidenceRefusal> {
        if let Some(kind) = self
            .kinds
            .iter()
            .find(|kind| !RecoveryEvidenceKind::SUPPORTED.contains(kind))
        {
            return Err(EvidenceRefusal {
                code: UNSUPPORTED_KIND_CODE,
                at: None,
                message: format!(
                    "the recovery request asks for `{}`, a category this entry does not materialize: \
                     the read evidence a presentation consumed is published beside this report by the \
                     entry that performed the read",
                    kind.spell()
                ),
            });
        }
        let Some(range) = self.driver_bci_range else {
            return Ok(());
        };
        if self.kinds.is_empty() {
            return Err(EvidenceRefusal {
                code: RANGE_SHAPE_CODE,
                at: Some(range.start()),
                message: format!(
                    "the request states the driver range {}..{} without selecting any evidence \
                     category, so the range would select nothing",
                    range.start(),
                    range.end()
                ),
            });
        }
        if range.start() > range.end() {
            return Err(EvidenceRefusal {
                code: RANGE_SHAPE_CODE,
                at: Some(range.start()),
                message: format!(
                    "the driver range {}..{} is reversed: a range is [start, end) and its end is \
                     never before its start",
                    range.start(),
                    range.end()
                ),
            });
        }
        let boundaries = instruction_boundaries(code);
        for bound in [range.start(), range.end()] {
            if !boundaries.contains(&bound) {
                return Err(EvidenceRefusal {
                    code: RANGE_REFUSAL_CODE,
                    at: Some(bound),
                    message: format!(
                        "the driver range {}..{} names BCI {bound}, which is not an instruction \
                         boundary of this body's decoded instructions (the body ends at BCI {})",
                        range.start(),
                        range.end(),
                        boundaries.iter().next_back().copied().unwrap_or(0)
                    ),
                });
            }
        }
        Ok(())
    }
}

/// Every BCI a driver range may start or end at: the start of each decoded instruction, and the end
/// of the last one.
///
/// The end of the decoded body is a boundary — `[x, x)` there is the empty selection a caller states
/// to ask "no positions" — and so is the last instruction's own end, which *is* the end of the code
/// when the decode read the whole body.
fn instruction_boundaries(code: &MethodCodeFacts) -> BTreeSet<u32> {
    let mut boundaries: BTreeSet<u32> = code
        .instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect();
    if let Some(last) = code.instructions.last() {
        boundaries.insert(last.bci.saturating_add(last.width));
    }
    boundaries
}

/// A request-level refusal: the selection cannot be answered for this body.
///
/// It is the request's own fact and not the body's — the body decoded, the run could have presented
/// it — which is why it is stated as a stop of its own ([`crate::StopReason::EvidenceRefused`]) and
/// never as an empty delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvidenceRefusal {
    pub(crate) code: &'static str,
    pub(crate) at: Option<u32>,
    pub(crate) message: String,
}

/// What happened to one evidence category in one run.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum EvidenceState {
    /// The request did not select this category. Its payload in this report is empty *because* of
    /// that, and an empty payload alone never says which of the two it is.
    NotRequested,
    /// Selected, and delivered in full. An empty payload here is a legal empty result — the body
    /// really has no record of this category — and not a missing answer.
    Complete,
    /// Selected, and the delivery stopped: `delivered` owning records were materialized before the
    /// run stopped, and the rest were not.
    Partial {
        /// How many owning records of this category the report holds.
        delivered: u64,
    },
    /// Selected, and nothing of it was materialized: the run stopped before the evidence phase (or
    /// the selection itself was refused). Never a claim that the body has no such evidence.
    NotPerformed,
}

/// One category's entry in the report's status list.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct EvidenceCategory {
    /// The category this entry is about.
    pub kind: RecoveryEvidenceKind,
    /// What happened to it in this run.
    pub state: EvidenceState,
}

/// The report's own answer to "what did this run deliver": the effective selection, and one status
/// per category.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveryEvidence {
    /// The selection this run was presented under, echoed from the request.
    requested: RecoveryEvidenceRequest,
    /// One entry per category, always [`RecoveryEvidenceKind::ALL`]'s five, in that order.
    categories: [EvidenceCategory; 5],
}

impl RecoveryEvidence {
    /// The status list of a run that has not materialized anything yet: every category the request
    /// did not select is [`EvidenceState::NotRequested`], and every category it did select is
    /// [`EvidenceState::NotPerformed`] until it really is materialized.
    pub(crate) fn pending(requested: &RecoveryEvidenceRequest) -> Self {
        Self {
            requested: requested.clone(),
            categories: RecoveryEvidenceKind::ALL.map(|kind| EvidenceCategory {
                kind,
                state: if requested.requests(kind) {
                    EvidenceState::NotPerformed
                } else {
                    EvidenceState::NotRequested
                },
            }),
        }
    }

    /// The selection this run was presented under.
    pub fn requested(&self) -> &RecoveryEvidenceRequest {
        &self.requested
    }

    /// One category's status.
    pub fn state(&self, kind: RecoveryEvidenceKind) -> EvidenceState {
        self.categories[kind.index()].state
    }

    /// Every entry, in [`RecoveryEvidenceKind::ALL`] order.
    pub fn categories(&self) -> &[EvidenceCategory; 5] {
        &self.categories
    }

    /// Marks one category delivered in full, which is [`EvidenceState::Complete`] — an empty payload
    /// included.
    pub(crate) fn delivered(&mut self, kind: RecoveryEvidenceKind) {
        self.set(kind, EvidenceState::Complete);
    }

    /// Marks one category whose materialization stopped after `delivered` records.
    pub(crate) fn stopped(&mut self, kind: RecoveryEvidenceKind, delivered: u64) {
        self.set(kind, EvidenceState::Partial { delivered });
    }

    fn set(&mut self, kind: RecoveryEvidenceKind, state: EvidenceState) {
        self.categories[kind.index()] = EvidenceCategory { kind, state };
    }

    /// Whether the status list says what the report's own payload holds.
    ///
    /// The one consistency rule this module exists to keep: a category the request did not select
    /// holds nothing, a category the run did not perform holds nothing, and a category it delivered
    /// holds exactly `delivered` owning records when it is partial (a complete delivery holds
    /// whatever the body really has). The report checks this before it is handed out.
    pub(crate) fn agrees_with(&self, payload: &dyn EvidencePayload) -> bool {
        RecoveryEvidenceKind::ALL.into_iter().all(|kind| {
            let held = payload.owning_records(kind);
            match self.state(kind) {
                EvidenceState::NotRequested | EvidenceState::NotPerformed => held == 0,
                EvidenceState::Complete => true,
                EvidenceState::Partial { delivered } => held == delivered,
            }
        })
    }
}

/// How many owning records one category's payload holds, for the status/payload check.
pub(crate) trait EvidencePayload {
    /// The number of owning records of one category that are in the report right now.
    fn owning_records(&self, kind: RecoveryEvidenceKind) -> u64;
}

/// The gated half of one selection the emitter reads: which spans of the text it anchors.
///
/// The emitter is the one stage that produces the segment table, so the source-map gate and the
/// driver range meet here: a run that did not select the map anchors nothing, and a run that
/// selected a range anchors only the segments one of whose own anchors the range holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SegmentPublication {
    /// The request did not select the source map: no segment is built.
    NotSelected,
    /// The whole method's segments are recorded.
    Whole,
    /// Only the segments that intersect this driver range are recorded.
    Range(BytecodeRange),
}

impl SegmentPublication {
    /// The source-map half of `selection`.
    pub(crate) fn of(selection: &RecoveryEvidenceRequest) -> Self {
        if !selection.requests(RecoveryEvidenceKind::SourceMap) {
            return Self::NotSelected;
        }
        match selection.driver_bci_range() {
            Some(range) => Self::Range(range),
            None => Self::Whole,
        }
    }

    /// Whether a span anchored at `positions` is recorded.
    pub(crate) fn records(&self, positions: impl IntoIterator<Item = u32>) -> bool {
        match self {
            Self::NotSelected => false,
            Self::Whole => true,
            Self::Range(range) => range.intersects_any(positions),
        }
    }
}

/// The gated half of one selection, as the rule plans read it.
///
/// A rule module never sees the whole request: what it needs is whether its own owning records are
/// published at all, and which driver positions they are restricted to. The two questions are asked
/// through [`Self::publishes`], which is the *only* place the rule-evidence gate and the range filter
/// meet — a plan that consulted either one on its own would publish a record the other excluded.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Publication {
    rules: bool,
    range: Option<BytecodeRange>,
}

impl Publication {
    /// The rule-evidence half of `selection`.
    pub(crate) fn of(selection: &RecoveryEvidenceRequest) -> Self {
        Self {
            rules: selection.requests(RecoveryEvidenceKind::RuleDetails),
            range: selection.driver_bci_range(),
        }
    }

    /// Whether a record of the selected positions is published.
    ///
    /// A record that states **no** driver position of its own is not selectable by a range — the
    /// member's declaration and the bridge verdict are verdicts about the body rather than about a
    /// position — so an empty `positions` is published with its category whatever range the request
    /// states, while a positioned record is published only when one of its positions is inside the
    /// range. No record is ever built and then dropped: the caller asks before it constructs.
    pub(crate) fn publishes(&self, positions: &[u32]) -> bool {
        self.rules
            && (positions.is_empty()
                || self
                    .range
                    .is_none_or(|range| range.intersects_any(positions.iter().copied())))
    }
}

/// The one stage of a run that materializes optional evidence, and what it is allowed to spend.
///
/// The phase is a stage of the same request as everything before it and is answered by the same
/// budget: one owning record of a selected category is one `IrItems` charge, charged before the
/// record is built, and the phase never resets a limit and never continues after a refusal. So a
/// request whose allowance is spent delivers a real prefix and says so, and a request that selected
/// nothing enters no phase at all. The artifact a previous stage committed is not touched by this
/// phase: what a refusal ends is the *materialization*, not the run's text.
pub(crate) struct EvidencePhase {
    materialized: u64,
    stopped: bool,
}

impl EvidencePhase {
    /// A phase that has not materialized anything yet.
    pub(crate) fn new() -> Self {
        Self {
            materialized: 0,
            stopped: false,
        }
    }

    /// Whether the phase may materialize one more owning record, charging it to the run's budget.
    ///
    /// A phase that stops here has materialized exactly its `materialized` prefix, and the report
    /// states that prefix and a real stop: it never deletes what a previous stage committed and never
    /// claims the categories it did not reach.
    pub(crate) fn may_continue(&mut self, budget: &mut Budget) -> bool {
        if self.stopped {
            return false;
        }
        match budget.charge(CountedBudgetDimension::IrItems, 1) {
            Ok(()) => {
                self.materialized += 1;
                true
            }
            Err(_) => {
                self.stopped = true;
                false
            }
        }
    }

    /// Whether the phase stopped before it could materialize everything the request selected.
    pub(crate) fn stopped(&self) -> bool {
        self.stopped
    }

    /// The stop the phase hit, stated in the same vocabulary the rest of the run stops in.
    pub(crate) fn reason(&self, budget: &Budget) -> crate::StopReason {
        if budget.cancellation_token().is_cancelled() {
            return crate::StopReason::Cancelled { at: None };
        }
        let usage = budget.usage();
        crate::StopReason::Budget {
            dimension: CountedBudgetDimension::IrItems,
            written: usage.counted_usage(CountedBudgetDimension::IrItems),
            limit: budget
                .limits()
                .counted_limit(CountedBudgetDimension::IrItems),
            at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{RecoveryReport, recover};
    use crate::{MethodFacts, RecoveryFacts, RecoveryRequest};
    use jarde_reader::artifact::{ArtifactInput, ArtifactSnapshot};
    use jarde_reader::budget::Limits;
    use jarde_reader::classfile::test_class;
    use jarde_reader::model::{
        ClassBytesId, Digest, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId,
        PhysicalMethodId, PhysicalVariant,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, ModuleMode, MultiReleasePolicy,
        PhysicalScope, PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
    };

    fn limits() -> Limits {
        Limits {
            input_bytes: 1 << 20,
            archive_entries: 1_000,
            entry_bytes: 1 << 20,
            read_bytes: 1 << 20,
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            code_bytes: 1 << 20,
            result_items: 1 << 20,
            output_bytes: 1 << 20,
            class_headers: 10,
            method_bodies: 10,
            ir_items: 1 << 20,
            ir_edges: 1 << 20,
            analysis_steps: 1 << 20,
            normalization_clones: 1 << 20,
            nested_depth: 8,
            dependency_depth: 4,
            elapsed_millis: u64::MAX,
        }
    }

    /// One assembled body's own `method()V`, analyzed through the real engine entry — the same route
    /// the crate's own end-to-end harness takes (`crate::oracle`), so the fixture is real bytes.
    fn analyze(code: &[u8], max_locals: u16) -> jarde_jvm::method_ir::MethodIrAnalysis {
        let class = test_class::single_method(52, 4, max_locals, code);
        let mut budget = jarde_reader::budget::Budget::new(limits());
        let snapshot = ArtifactSnapshot::open(ArtifactInput::bytes(class.clone()), &mut budget)
            .expect("the fixture opens as a standalone CLASS");
        let definition = PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                // The reader's own identity for an assembled fixture: the digest of the bytes the run
                // is about to read (the same route `crate::oracle`'s cases take).
                digest: Digest(blake3::hash(&class).to_hex().to_string()),
                length: u64::try_from(class.len()).expect("the fixture length fits u64"),
            },
            variant: PhysicalVariant::Base,
        };
        let method = PhysicalMethodId {
            owner: definition,
            name: JvmBytes(b"method".to_vec()),
            descriptor: JvmBytes(b"()V".to_vec()),
        };
        let domain = LoadDomain {
            loader: jarde_reader::view::LoaderId("app".to_string()),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots: vec![LoadRoot::StandaloneClass {
                snapshot: snapshot.id().clone(),
            }],
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        };
        let environment = jarde_jvm::environment::ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: snapshot.id().clone(),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        };
        let request = jarde_jvm::ir::MethodAnalysisRequest {
            environment,
            method,
            stages: jarde_jvm::ir::AnalysisStage::ALL.to_vec(),
        };
        jarde_jvm::engine::analyze_method_ir(&[snapshot], &request, &mut budget)
            .expect("the analysis of an assembled body runs")
    }

    /// The report of one fixture under `selection`, with the debug names a case states (none for
    /// every case but the name one) and no member table.
    fn report_of(
        analysis: &jarde_jvm::method_ir::MethodIrAnalysis,
        selection: RecoveryEvidenceRequest,
    ) -> RecoveryReport {
        report_of_with(analysis, selection, Vec::new())
    }

    /// The same, with the raw debug names the caller states for the body's slots.
    fn report_of_with(
        analysis: &jarde_jvm::method_ir::MethodIrAnalysis,
        selection: RecoveryEvidenceRequest,
        debug: Vec<crate::DebugLocal>,
    ) -> RecoveryReport {
        let facts =
            RecoveryFacts::new(MethodFacts::new("method", "()V", 0)).with_debug_locals(debug);
        let mut budget = jarde_reader::budget::Budget::new(limits());
        recover(
            &RecoveryRequest::new(analysis.ir(), &facts, crate::pass::JAVA_8)
                .with_evidence(selection),
            &mut budget,
        )
    }

    /// The three decisions a selection may not change: what the artifact is, what it holds, and
    /// which fallbacks it had to keep.
    fn decisions(report: &RecoveryReport) -> String {
        format!(
            "{:?}/{:?}/{:?}/{:?}/{:?}",
            report.representation,
            report.quality,
            report.content,
            report.fallbacks,
            report
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.as_str())
                .collect::<Vec<_>>(),
        )
    }

    /// The essential selection builds **no** owning record of an unselected category, and the full
    /// one builds them: the same bytes, the same text, the same decisions.
    ///
    /// This is the gate the port exists for (`crate::demand_counts`): the count is taken where the
    /// records are built, so an implementation that built the whole table and then dropped it out of
    /// the payload reports a non-zero count here and fails, while the status list it hands out looks
    /// exactly the same.
    /// The request type's own default is the ordinary recovery: a caller that states no selection
    /// asks for the necessary results, and the entry points that *do* state one state it explicitly.
    #[test]
    fn the_request_type_defaults_to_the_essential_selection() {
        let analysis = analyze(&[0xb1], 0);
        let facts = RecoveryFacts::new(MethodFacts::new("method", "()V", 0));
        let request = RecoveryRequest::new(analysis.ir(), &facts, crate::pass::JAVA_8);
        assert!(
            request.evidence.is_essential(),
            "the ordinary request selects nothing optional: {:?}",
            request.evidence
        );
    }

    #[test]
    fn the_essential_selection_builds_only_what_it_asked_for() {
        // `iconst_0; istore_1; iload_1; ifeq +4; return; return`: two regions, a branch and a
        // return, so region details, a source map and rule records all have something to hold.
        const BODY: &[u8] = &[
            0x03, // 0: iconst_0
            0x3c, // 1: istore_1
            0x1b, // 2: iload_1
            0x99, 0x00, 0x04, // 3: ifeq 7
            0xb1, // 6: return
            0xb1, // 7: return
        ];
        let analysis = analyze(BODY, 2);
        let before = crate::demand_counts::snapshot();
        let essential = report_of(&analysis, RecoveryEvidenceRequest::essential());
        let essential_built = before.since(crate::demand_counts::snapshot());
        assert_eq!(
            essential_built,
            crate::demand_counts::Built::default(),
            "the ordinary recovery constructs no owning record of any optional category"
        );
        for kind in RecoveryEvidenceKind::SUPPORTED {
            assert_eq!(
                essential.evidence.state(kind),
                EvidenceState::NotRequested,
                "{kind:?}"
            );
            assert_eq!(essential.owning_records(kind), 0, "{kind:?}");
            assert_eq!(essential_built.of(kind), 0, "{kind:?}");
        }
        assert!(essential.evidence.agrees_with(&essential));
        assert!(essential.produced(), "{:?}", essential.outcome);

        let before = crate::demand_counts::snapshot();
        let all = report_of(&analysis, RecoveryEvidenceRequest::all());
        let all_built = before.since(crate::demand_counts::snapshot());
        assert!(
            all_built.region_details > 0 && all_built.source_map > 0 && all_built.rule_details > 0,
            "the full selection really builds the records it asks for: {all_built:?}"
        );

        // Same bytes, same run, same artifact.
        assert_eq!(all.text, essential.text, "the artifact is the same text");
        assert_eq!(decisions(&all), decisions(&essential));
        for kind in RecoveryEvidenceKind::SUPPORTED {
            assert_eq!(
                all.evidence.state(kind),
                EvidenceState::Complete,
                "{kind:?}"
            );
        }
        assert!(all.evidence.agrees_with(&all));
        assert_eq!(
            all.evidence.requested().kinds().collect::<Vec<_>>(),
            RecoveryEvidenceKind::SUPPORTED.to_vec(),
            "the report echoes the selection it was presented under"
        );
    }

    /// The four states are four statements, and each one is reachable.
    ///
    /// `NotRequested` is the empty selection, `Complete` an empty-or-not full delivery,
    /// `NotPerformed` a selected category the run never materialized, and `Partial` a delivery that
    /// stopped after a prefix. The last two need a budget that refuses, and the same fixture answers
    /// all four.
    #[test]
    fn the_status_list_distinguishes_not_requested_complete_partial_and_not_performed() {
        // `if (x) { return; } if (y) { return; } return;`: two branches in sequence, so the structure
        // layer recovers more than one region and the evidence phase has more than one record to
        // materialize — which is what makes a *prefix* of it a different state from "nothing".
        const BODY: &[u8] = &[
            0x03, // 0: iconst_0
            0x3d, // 1: istore_2
            0x1c, // 2: iload_2
            0x99, 0x00, 0x04, // 3: ifeq 7
            0xb1, // 6: return
            0x03, // 7: iconst_0
            0x3e, // 8: istore_3
            0x1d, // 9: iload_3
            0x99, 0x00, 0x04, // 10: ifeq 14
            0xb1, // 13: return
            0xb1, // 14: return
        ];
        let analysis = analyze(BODY, 4);

        // NotRequested, and Complete with a legal empty payload: a legal empty driver range selects
        // no position, so the region category is delivered — and empty.
        let request = RecoveryEvidenceRequest::essential()
            .with_kind(RecoveryEvidenceKind::RegionDetails)
            .with_driver_bci_range(BytecodeRange::new(0, 0));
        let report = report_of(&analysis, request);
        assert_eq!(
            report.evidence.state(RecoveryEvidenceKind::RegionDetails),
            EvidenceState::Complete
        );
        assert!(report.regions.is_empty(), "an empty range selects nothing");
        assert_eq!(
            report.evidence.state(RecoveryEvidenceKind::RuleDetails),
            EvidenceState::NotRequested
        );
        assert!(report.evidence.agrees_with(&report));

        // NotPerformed: the same category, selected, in a run that stopped before the evidence
        // phase. The report says so instead of pretending the body has no such evidence.
        let facts = RecoveryFacts::new(MethodFacts::new("method", "()V", 0));
        let mut budget = jarde_reader::budget::Budget::new(Limits {
            output_bytes: 0,
            ..limits()
        });
        let stopped = recover(
            &RecoveryRequest::new(analysis.ir(), &facts, crate::pass::JAVA_8)
                .with_evidence(RecoveryEvidenceRequest::all()),
            &mut budget,
        );
        assert!(!stopped.produced(), "{:?}", stopped.outcome);
        for kind in RecoveryEvidenceKind::SUPPORTED {
            assert_eq!(stopped.evidence.state(kind), EvidenceState::NotPerformed);
            assert_eq!(stopped.owning_records(kind), 0, "{kind:?}");
        }
        assert!(stopped.evidence.agrees_with(&stopped));

        // Partial: the evidence phase materializes within the request's own remaining allowance, so
        // a bound that funds the artifact and all but one record of the selection stops the phase
        // with the prefix it built — and the artifact stays, which is what `Partial` beside a
        // produced report means.
        //
        // The fixture's slots are spelled `int` and `class` by the request's own facts (the 1.1
        // payload holds no debug table), so the presentation replaces both: the phase has one region
        // record and two name records to materialize, and the bound below is one charge short of the
        // whole run.
        let names = vec![
            crate::DebugLocal::named(0, "int"),
            crate::DebugLocal::named(2, "class"),
        ];
        let selection = RecoveryEvidenceRequest::all();
        let measuring = {
            let mut budget = jarde_reader::budget::Budget::new(limits());
            let facts = RecoveryFacts::new(MethodFacts::new("method", "()V", 0))
                .with_debug_locals(names.clone());
            let report = recover(
                &RecoveryRequest::new(analysis.ir(), &facts, crate::pass::JAVA_8)
                    .with_evidence(selection.clone()),
                &mut budget,
            );
            assert!(report.evidence.agrees_with(&report));
            assert!(
                !report.regions.is_empty() && report.aliased_names.len() >= 2,
                "the fixture has records for the phase to materialize: {} region(s), {} name(s)",
                report.regions.len(),
                report.aliased_names.len()
            );
            budget
        };
        let used = measuring
            .usage()
            .counted_usage(jarde_reader::budget::CountedBudgetDimension::IrItems);
        let mut budget = jarde_reader::budget::Budget::new(Limits {
            ir_items: used.saturating_sub(1),
            ..limits()
        });
        let facts =
            RecoveryFacts::new(MethodFacts::new("method", "()V", 0)).with_debug_locals(names);
        let partial = recover(
            &RecoveryRequest::new(analysis.ir(), &facts, crate::pass::JAVA_8)
                .with_evidence(selection),
            &mut budget,
        );
        assert!(partial.produced(), "the committed artifact is kept");
        assert!(
            !partial.text.is_empty(),
            "the text the evidence phase does not touch is delivered"
        );
        assert_eq!(
            partial.evidence.state(RecoveryEvidenceKind::RegionDetails),
            EvidenceState::Complete,
            "the category the phase reached first is complete"
        );
        let state = partial.evidence.state(RecoveryEvidenceKind::NameDetails);
        assert!(
            matches!(state, EvidenceState::Partial { delivered } if delivered >= 1),
            "the category the phase stopped inside states the prefix it delivered: {state:?}"
        );
        assert_eq!(
            u64::try_from(partial.aliased_names.len()).expect("a small count"),
            match state {
                EvidenceState::Partial { delivered } => delivered,
                other => panic!("{other:?}"),
            },
            "and the prefix it states is the payload it really holds"
        );
        assert!(
            !matches!(
                partial.execution,
                jarde_reader::model::ExecutionReport::Complete { .. }
            ),
            "the run states the stop that ended its evidence phase"
        );
        assert!(partial.evidence.agrees_with(&partial));
    }
}
