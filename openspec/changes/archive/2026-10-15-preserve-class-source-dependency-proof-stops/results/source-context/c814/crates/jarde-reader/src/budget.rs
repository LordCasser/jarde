use crate::error::{Error, Result};
use crate::facts_cache::FactsCache;
use crate::ledger::{OperationLedger, UsageOwner};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CountedBudgetDimension {
    InputBytes,
    /// Central-directory records processed by this request, including locator scans.
    ArchiveEntries,
    EntryBytes,
    ReadBytes,
    ClassBytes,
    AttributeBytes,
    CodeBytes,
    ResultItems,
    OutputBytes,
    /// One class Header **read attempt** (`P2`).
    ///
    /// Attempts are counted before the read; the caller deduplicates repeated reads of the
    /// same `(definition, loader)` binding inside one request, and a failed attempt still
    /// costs one. The budget layer counts what it is asked to count.
    ClassHeaders,
    /// One method Body (`Code`) **read attempt** (`P2`).
    ///
    /// Same rule as [`CountedBudgetDimension::ClassHeaders`]: attempts are charged before
    /// the read, deduplicated by the caller, and members without a Body are never attempted.
    MethodBodies,
    /// One derived storage item (`P2`): frame/local slots, SSA values, phi inputs and
    /// origin members each cost one, charged **before** the allocation or enqueue.
    IrItems,
    /// One derived edge (`P2`): CFG edges (exception edges included) and SSA def-use edges
    /// each cost one, charged **before** the edge is added.
    IrEdges,
    /// One worklist pop/processing step (`P2`). Repeated visits of the same node are
    /// counted, because they are real work; charged **before** the step runs.
    AnalysisSteps,
    /// One clone node produced by `jsr`/`ret` normalization (`P2`), charged **before** the
    /// clone is created.
    NormalizationClones,
}

impl CountedBudgetDimension {
    /// Every counted dimension in declaration order.
    ///
    /// Callers that must cover all counted dimensions (zero-usage assertions, result
    /// schema dumps) iterate this list instead of naming fields, so a new dimension is
    /// added in one place together with its `Limits`/`UsageSnapshot` field.
    pub const ALL: [Self; 15] = [
        Self::InputBytes,
        Self::ArchiveEntries,
        Self::EntryBytes,
        Self::ReadBytes,
        Self::ClassBytes,
        Self::AttributeBytes,
        Self::CodeBytes,
        Self::ResultItems,
        Self::OutputBytes,
        Self::ClassHeaders,
        Self::MethodBodies,
        Self::IrItems,
        Self::IrEdges,
        Self::AnalysisSteps,
        Self::NormalizationClones,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetDimension {
    InputBytes,
    ArchiveEntries,
    EntryBytes,
    ReadBytes,
    ClassBytes,
    AttributeBytes,
    CodeBytes,
    ResultItems,
    OutputBytes,
    ClassHeaders,
    MethodBodies,
    IrItems,
    IrEdges,
    AnalysisSteps,
    NormalizationClones,
    NestedDepth,
    /// High-water mark of the dependency-closure depth one request reached (`P2`).
    ///
    /// Like [`BudgetDimension::NestedDepth`] this dimension is not accumulated, but it
    /// measures a different thing and has its own limit and usage slot: `NestedDepth`
    /// counts how deep a container nested another container, `DependencyDepth` how deep the
    /// dependency closure reached. Exhausting one never reports or blocks the other.
    DependencyDepth,
    ElapsedMillis,
}

impl From<CountedBudgetDimension> for BudgetDimension {
    fn from(value: CountedBudgetDimension) -> Self {
        match value {
            CountedBudgetDimension::InputBytes => Self::InputBytes,
            CountedBudgetDimension::ArchiveEntries => Self::ArchiveEntries,
            CountedBudgetDimension::EntryBytes => Self::EntryBytes,
            CountedBudgetDimension::ReadBytes => Self::ReadBytes,
            CountedBudgetDimension::ClassBytes => Self::ClassBytes,
            CountedBudgetDimension::AttributeBytes => Self::AttributeBytes,
            CountedBudgetDimension::CodeBytes => Self::CodeBytes,
            CountedBudgetDimension::ResultItems => Self::ResultItems,
            CountedBudgetDimension::OutputBytes => Self::OutputBytes,
            CountedBudgetDimension::ClassHeaders => Self::ClassHeaders,
            CountedBudgetDimension::MethodBodies => Self::MethodBodies,
            CountedBudgetDimension::IrItems => Self::IrItems,
            CountedBudgetDimension::IrEdges => Self::IrEdges,
            CountedBudgetDimension::AnalysisSteps => Self::AnalysisSteps,
            CountedBudgetDimension::NormalizationClones => Self::NormalizationClones,
        }
    }
}

impl TryFrom<BudgetDimension> for CountedBudgetDimension {
    type Error = ();

    fn try_from(value: BudgetDimension) -> std::result::Result<Self, Self::Error> {
        match value {
            BudgetDimension::InputBytes => Ok(Self::InputBytes),
            BudgetDimension::ArchiveEntries => Ok(Self::ArchiveEntries),
            BudgetDimension::EntryBytes => Ok(Self::EntryBytes),
            BudgetDimension::ReadBytes => Ok(Self::ReadBytes),
            BudgetDimension::ClassBytes => Ok(Self::ClassBytes),
            BudgetDimension::AttributeBytes => Ok(Self::AttributeBytes),
            BudgetDimension::CodeBytes => Ok(Self::CodeBytes),
            BudgetDimension::ResultItems => Ok(Self::ResultItems),
            BudgetDimension::OutputBytes => Ok(Self::OutputBytes),
            BudgetDimension::ClassHeaders => Ok(Self::ClassHeaders),
            BudgetDimension::MethodBodies => Ok(Self::MethodBodies),
            BudgetDimension::IrItems => Ok(Self::IrItems),
            BudgetDimension::IrEdges => Ok(Self::IrEdges),
            BudgetDimension::AnalysisSteps => Ok(Self::AnalysisSteps),
            BudgetDimension::NormalizationClones => Ok(Self::NormalizationClones),
            BudgetDimension::NestedDepth
            | BudgetDimension::DependencyDepth
            | BudgetDimension::ElapsedMillis => Err(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Limits {
    pub input_bytes: u64,
    pub archive_entries: u64,
    pub entry_bytes: u64,
    pub read_bytes: u64,
    pub class_bytes: u64,
    pub attribute_bytes: u64,
    pub code_bytes: u64,
    pub result_items: u64,
    pub output_bytes: u64,
    /// Header read attempts; see [`CountedBudgetDimension::ClassHeaders`].
    pub class_headers: u64,
    /// Body read attempts; see [`CountedBudgetDimension::MethodBodies`].
    pub method_bodies: u64,
    /// Derived storage items; see [`CountedBudgetDimension::IrItems`].
    pub ir_items: u64,
    /// Derived edges; see [`CountedBudgetDimension::IrEdges`].
    pub ir_edges: u64,
    /// Worklist steps; see [`CountedBudgetDimension::AnalysisSteps`].
    pub analysis_steps: u64,
    /// `jsr`/`ret` normalization clones; see
    /// [`CountedBudgetDimension::NormalizationClones`].
    pub normalization_clones: u64,
    pub nested_depth: u64,
    /// Dependency-closure depth high-water mark; see [`BudgetDimension::DependencyDepth`].
    pub dependency_depth: u64,
    pub elapsed_millis: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub input_bytes: u64,
    pub archive_entries: u64,
    pub entry_bytes: u64,
    pub read_bytes: u64,
    pub class_bytes: u64,
    pub attribute_bytes: u64,
    pub code_bytes: u64,
    pub result_items: u64,
    pub output_bytes: u64,
    pub class_headers: u64,
    pub method_bodies: u64,
    pub ir_items: u64,
    pub ir_edges: u64,
    pub analysis_steps: u64,
    pub normalization_clones: u64,
    pub nested_depth: u64,
    pub dependency_depth: u64,
    pub elapsed_millis: u64,
}

/// Zero limits — a base for tests and tooling, **not** an implicit production budget.
///
/// Every dimension is `0`, which fails closed: a request that charges the first unit of any
/// counted dimension, reads the first byte, observes the first dependency level or polls the
/// clock is already over its limit. Callers that intend to run work must set the dimensions
/// they use explicitly (or start from a helper that does); nothing in the engine substitutes
/// this for a missing limit, and the CLI request schema keeps every field required for the
/// same reason. `Default` exists so a new dimension is added here and in
/// `Limits`/`UsageSnapshot` together while the literals that only care about the P0/P1
/// dimensions stay one line shorter (`..Limits::default()`), instead of every call site
/// restating values it does not use.
impl Default for Limits {
    fn default() -> Self {
        Self {
            input_bytes: 0,
            archive_entries: 0,
            entry_bytes: 0,
            read_bytes: 0,
            class_bytes: 0,
            attribute_bytes: 0,
            code_bytes: 0,
            result_items: 0,
            output_bytes: 0,
            class_headers: 0,
            method_bodies: 0,
            ir_items: 0,
            ir_edges: 0,
            analysis_steps: 0,
            normalization_clones: 0,
            nested_depth: 0,
            dependency_depth: 0,
            elapsed_millis: 0,
        }
    }
}

impl Limits {
    fn get(&self, dimension: CountedBudgetDimension) -> u64 {
        match dimension {
            CountedBudgetDimension::InputBytes => self.input_bytes,
            CountedBudgetDimension::ArchiveEntries => self.archive_entries,
            CountedBudgetDimension::EntryBytes => self.entry_bytes,
            CountedBudgetDimension::ReadBytes => self.read_bytes,
            CountedBudgetDimension::ClassBytes => self.class_bytes,
            CountedBudgetDimension::AttributeBytes => self.attribute_bytes,
            CountedBudgetDimension::CodeBytes => self.code_bytes,
            CountedBudgetDimension::ResultItems => self.result_items,
            CountedBudgetDimension::OutputBytes => self.output_bytes,
            CountedBudgetDimension::ClassHeaders => self.class_headers,
            CountedBudgetDimension::MethodBodies => self.method_bodies,
            CountedBudgetDimension::IrItems => self.ir_items,
            CountedBudgetDimension::IrEdges => self.ir_edges,
            CountedBudgetDimension::AnalysisSteps => self.analysis_steps,
            CountedBudgetDimension::NormalizationClones => self.normalization_clones,
        }
    }

    /// Limit of one counted dimension, addressed by value.
    ///
    /// Callers that need the whole table (CLI schema, budget checks over
    /// [`CountedBudgetDimension::ALL`]) read it through this accessor instead of matching
    /// fields themselves, so a new dimension only has to be added here.
    pub fn counted_limit(&self, dimension: CountedBudgetDimension) -> u64 {
        self.get(dimension)
    }
}

impl UsageSnapshot {
    /// Counted usage of one dimension, addressed by value.
    ///
    /// Crate-internal because the operation ledger reads and writes the same slots a `Budget` does:
    /// it is the store of one operation's totals, not a second public reading of them (that is
    /// [`Budget::usage`], [`UsageSnapshot::counted_usage`] and the ledger's own snapshot methods).
    pub(crate) fn get(&self, dimension: CountedBudgetDimension) -> u64 {
        match dimension {
            CountedBudgetDimension::InputBytes => self.input_bytes,
            CountedBudgetDimension::ArchiveEntries => self.archive_entries,
            CountedBudgetDimension::EntryBytes => self.entry_bytes,
            CountedBudgetDimension::ReadBytes => self.read_bytes,
            CountedBudgetDimension::ClassBytes => self.class_bytes,
            CountedBudgetDimension::AttributeBytes => self.attribute_bytes,
            CountedBudgetDimension::CodeBytes => self.code_bytes,
            CountedBudgetDimension::ResultItems => self.result_items,
            CountedBudgetDimension::OutputBytes => self.output_bytes,
            CountedBudgetDimension::ClassHeaders => self.class_headers,
            CountedBudgetDimension::MethodBodies => self.method_bodies,
            CountedBudgetDimension::IrItems => self.ir_items,
            CountedBudgetDimension::IrEdges => self.ir_edges,
            CountedBudgetDimension::AnalysisSteps => self.analysis_steps,
            CountedBudgetDimension::NormalizationClones => self.normalization_clones,
        }
    }

    /// Counted usage of one dimension, addressed by value.
    ///
    /// `NestedDepth`, `DependencyDepth` and `ElapsedMillis` are not counted dimensions: the
    /// high-water marks live in [`UsageSnapshot::nested_depth`] and
    /// [`UsageSnapshot::dependency_depth`], and the wall clock in
    /// [`UsageSnapshot::elapsed_millis`].
    pub fn counted_usage(&self, dimension: CountedBudgetDimension) -> u64 {
        self.get(dimension)
    }

    fn add(&mut self, dimension: CountedBudgetDimension, amount: u64) -> Option<()> {
        let slot = match dimension {
            CountedBudgetDimension::InputBytes => &mut self.input_bytes,
            CountedBudgetDimension::ArchiveEntries => &mut self.archive_entries,
            CountedBudgetDimension::EntryBytes => &mut self.entry_bytes,
            CountedBudgetDimension::ReadBytes => &mut self.read_bytes,
            CountedBudgetDimension::ClassBytes => &mut self.class_bytes,
            CountedBudgetDimension::AttributeBytes => &mut self.attribute_bytes,
            CountedBudgetDimension::CodeBytes => &mut self.code_bytes,
            CountedBudgetDimension::ResultItems => &mut self.result_items,
            CountedBudgetDimension::OutputBytes => &mut self.output_bytes,
            CountedBudgetDimension::ClassHeaders => &mut self.class_headers,
            CountedBudgetDimension::MethodBodies => &mut self.method_bodies,
            CountedBudgetDimension::IrItems => &mut self.ir_items,
            CountedBudgetDimension::IrEdges => &mut self.ir_edges,
            CountedBudgetDimension::AnalysisSteps => &mut self.analysis_steps,
            CountedBudgetDimension::NormalizationClones => &mut self.normalization_clones,
        };
        *slot = slot.checked_add(amount)?;
        Some(())
    }

    /// Writes one counted dimension's total, which the operation ledger has already admitted.
    ///
    /// Crate-internal for the same reason [`UsageSnapshot::get`] is, and checked by construction on
    /// the caller's side: the ledger computes `consumed + requested` with a checked addition and
    /// refuses the dimension before it writes anything, so the value handed here is one that passed
    /// both an overflow check and the operation's limit.
    pub(crate) fn set(&mut self, dimension: CountedBudgetDimension, consumed: u64) {
        let slot = match dimension {
            CountedBudgetDimension::InputBytes => &mut self.input_bytes,
            CountedBudgetDimension::ArchiveEntries => &mut self.archive_entries,
            CountedBudgetDimension::EntryBytes => &mut self.entry_bytes,
            CountedBudgetDimension::ReadBytes => &mut self.read_bytes,
            CountedBudgetDimension::ClassBytes => &mut self.class_bytes,
            CountedBudgetDimension::AttributeBytes => &mut self.attribute_bytes,
            CountedBudgetDimension::CodeBytes => &mut self.code_bytes,
            CountedBudgetDimension::ResultItems => &mut self.result_items,
            CountedBudgetDimension::OutputBytes => &mut self.output_bytes,
            CountedBudgetDimension::ClassHeaders => &mut self.class_headers,
            CountedBudgetDimension::MethodBodies => &mut self.method_bodies,
            CountedBudgetDimension::IrItems => &mut self.ir_items,
            CountedBudgetDimension::IrEdges => &mut self.ir_edges,
            CountedBudgetDimension::AnalysisSteps => &mut self.analysis_steps,
            CountedBudgetDimension::NormalizationClones => &mut self.normalization_clones,
        };
        *slot = consumed;
    }
}

#[derive(Clone, Debug)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Budget {
    limits: Limits,
    usage: UsageSnapshot,
    cancellation: CancellationToken,
    started_at: Instant,
    facts: Option<FactsCache>,
    /// The operation's shared total, when this budget is doing part of one, and the part of it this
    /// budget's work belongs to. Both are absent on the direct single-request path, which is what
    /// [`Budget::new`] builds: attaching nothing keeps every entry point's numbers exactly where
    /// they were.
    ledger: Option<LedgerTarget>,
}

/// The operation total a budget bills to, and the work class it bills as.
#[derive(Debug)]
struct LedgerTarget {
    ledger: OperationLedger,
    owner: UsageOwner,
}

impl Budget {
    pub fn new(limits: Limits) -> Self {
        Self::with_cancellation_token(limits, CancellationToken::new())
    }

    pub fn with_cancellation_token(limits: Limits, cancellation: CancellationToken) -> Self {
        Self {
            limits,
            usage: UsageSnapshot::default(),
            cancellation,
            started_at: Instant::now(),
            facts: None,
            ledger: None,
        }
    }

    /// The same lifecycle with a facts cache this request may consult.
    ///
    /// A budget is the one handle every read path already holds — the artifact reads, the query
    /// scan and the resolver all thread `&mut Budget` and nothing else — so it is where an opt-in
    /// cache is declared rather than a signature that every entry point and every caller would have
    /// to grow. `Budget::new` attaches none, so a caller that says nothing keeps today's byte-for-
    /// byte path; a caller that attaches one shares those facts with every other budget holding the
    /// same handle, which is what a cache is for.
    pub fn with_facts_cache(mut self, cache: FactsCache) -> Self {
        self.facts = Some(cache);
        self
    }

    /// The facts cache this request may consult, if a caller attached one.
    pub fn facts_cache(&self) -> Option<&FactsCache> {
        self.facts.as_ref()
    }

    /// Bills this budget's work to the operation's shared total, as `owner`'s part of it.
    ///
    /// A bulk operation is one operation with N workers, and this is how a worker's budget states
    /// that: every charge it makes is admitted against the operation's totals *and* against its own
    /// local limits, and the operation's deadline and cancellation are checked before each of them.
    /// The ledger is the one [`OperationLedger::new`] built from the budget that opened the
    /// operation, so the entry's usage keeps counting and nothing here resumes a fresh allowance.
    ///
    /// Unlike [`Budget::with_facts_cache`], which takes the budget it is called on, this is called
    /// on a budget the caller already owns — the entry budget a caller hands to an operation, or a
    /// worker's own per-class or per-method budget — so it attaches in place. A budget that already
    /// bills to a ledger is re-attached: the last ledger wins, and attaching the same ledger again
    /// with another owner is how a caller states that its next work belongs to another part of the
    /// operation.
    pub fn with_ledger(&mut self, ledger: OperationLedger, owner: UsageOwner) {
        self.ledger = Some(LedgerTarget { ledger, owner });
    }

    /// The operation's shared total this budget bills to, when it is doing part of one.
    pub fn ledger(&self) -> Option<&OperationLedger> {
        self.ledger
            .as_ref()
            .map(|target: &LedgerTarget| &target.ledger)
    }

    /// States which part of the operation this budget's work belongs to.
    ///
    /// Discovery/preparation, method execution and delivery are the three classes design decision 4
    /// separates in the operation's totals, and one worker does more than one of them: preparing a
    /// class is discovery, the methods it then executes are methods, and the encoded records it
    /// hands on are delivery. A budget with no ledger attached has no attribution to move, so this
    /// changes nothing there.
    pub fn set_owner(&mut self, owner: UsageOwner) {
        if let Some(target) = &mut self.ledger {
            target.owner = owner;
        }
    }

    /// The instant this request's clock started.
    ///
    /// The operation ledger reads it once so that an operation's `elapsed_millis` is one wall clock
    /// over all of its workers instead of a sum of their durations.
    pub(crate) fn started_at(&self) -> Instant {
        self.started_at
    }

    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    pub fn usage(&self) -> UsageSnapshot {
        let mut usage = self.usage.clone();
        usage.elapsed_millis = self.elapsed_millis();
        usage
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    /// The cooperative checkpoints: cancellation and the deadline.
    ///
    /// A budget doing part of an operation checks the **operation's** cancellation and deadline
    /// first ([`OperationLedger::poll`], which is also where the operation's first stop is recorded)
    /// and its own after them: a worker reports the reason that stops the whole operation rather
    /// than a local deadline that happened to expire in the same millisecond.
    pub fn poll(&self) -> Result<()> {
        if let Some(target) = &self.ledger {
            target.ledger.poll()?;
        }
        self.check_cancelled()?;
        self.check_elapsed()
    }

    /// Whether `requested` units of `dimension` would still fit, charging nothing.
    ///
    /// Both limits are asked: this budget's own, and — when it bills to an operation — the
    /// operation's total. A probe records no stop: it refuses no work, and the operation's stop is
    /// what a *refused charge* or an observed cancellation states.
    pub fn check(&self, dimension: CountedBudgetDimension, requested: u64) -> Result<()> {
        self.poll()?;
        self.ensure_within(dimension, requested)?;
        if let Some(target) = &self.ledger {
            target.ledger.check(dimension, requested)?;
        }
        Ok(())
    }

    /// Charges `requested` units of `dimension`.
    ///
    /// # Order of the checks (design decision 4)
    ///
    /// A budget doing part of an operation charges in this order, and the order is the semantics:
    ///
    /// 1. the checkpoints ([`Budget::poll`]): the operation's deadline and cancellation, then this
    ///    budget's;
    /// 2. this budget's own limit and checked addition, so a local limit stops the local work before
    ///    the operation is asked for anything;
    /// 3. the operation's total, taken atomically on that dimension's own counter, and the owner's
    ///    share of it booked immediately afterwards ([`OperationLedger::charge`]);
    /// 4. this budget's own usage.
    ///
    /// Work whose permit was refused started nothing and is billed nowhere. A cancellation that
    /// arrives *after* the permit was taken does not undo the charge: the work was admitted, the
    /// totals say so, and the next checkpoint stops it.
    pub fn charge(&mut self, dimension: CountedBudgetDimension, requested: u64) -> Result<()> {
        self.poll()?;
        self.ensure_within(dimension, requested)?;
        if let Some(target) = &self.ledger {
            target.ledger.charge(target.owner, dimension, requested)?;
        }
        if self.usage.add(dimension, requested).is_none() {
            return Err(self.exceeded(dimension, requested));
        }
        self.usage.elapsed_millis = self.elapsed_millis();
        Ok(())
    }

    /// Records the container nesting depth a request accepted and enforces `NestedDepth`.
    ///
    /// A high-water dimension: accepted depths keep the deepest value seen, and a depth over
    /// the limit is reported with `consumed = depth - 1` so the caller can see the deepest
    /// accepted depth. `NestedDepth` and [`Budget::observe_dependency_depth`] are independent
    /// — separate limits, separate usage slots, and neither touches the other's value.
    pub fn check_nested_depth(&mut self, depth: u64) -> Result<()> {
        self.poll()?;
        if depth > self.limits.nested_depth {
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::NestedDepth,
                limit: self.limits.nested_depth,
                consumed: depth.saturating_sub(1),
                requested: 1,
            });
        }
        if let Some(target) = &self.ledger {
            target.ledger.check_nested_depth(target.owner, depth)?;
        }
        self.usage.nested_depth = self.usage.nested_depth.max(depth);
        self.usage.elapsed_millis = self.elapsed_millis();
        Ok(())
    }

    /// Records the dependency-closure depth a request reached and enforces
    /// `DependencyDepth`.
    ///
    /// Same high-water shape as [`Budget::check_nested_depth`], for the second and otherwise
    /// independent depth dimension: the closure depth is compared **before** the closure is
    /// expanded, accepted depths keep the deepest value in
    /// [`UsageSnapshot::dependency_depth`], and a depth over the limit reports
    /// [`BudgetDimension::DependencyDepth`] with `consumed = depth - 1`. Exhausting this
    /// dimension does not mark `NestedDepth` as exceeded, and vice versa.
    pub fn observe_dependency_depth(&mut self, depth: u64) -> Result<()> {
        self.poll()?;
        if depth > self.limits.dependency_depth {
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::DependencyDepth,
                limit: self.limits.dependency_depth,
                consumed: depth.saturating_sub(1),
                requested: 1,
            });
        }
        if let Some(target) = &self.ledger {
            target
                .ledger
                .observe_dependency_depth(target.owner, depth)?;
        }
        self.usage.dependency_depth = self.usage.dependency_depth.max(depth);
        self.usage.elapsed_millis = self.elapsed_millis();
        Ok(())
    }

    fn elapsed_millis(&self) -> u64 {
        self.started_at.elapsed().as_millis().min(u64::MAX as u128) as u64
    }

    fn check_cancelled(&self) -> Result<()> {
        if self.cancellation.is_cancelled() {
            Err(Error::Cancelled {
                reason: "cooperative cancellation requested".into(),
            })
        } else {
            Ok(())
        }
    }

    fn check_elapsed(&self) -> Result<()> {
        let elapsed = self.elapsed_millis();
        if elapsed >= self.limits.elapsed_millis {
            return Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
                limit: self.limits.elapsed_millis,
                consumed: elapsed,
                requested: 0,
            });
        }
        Ok(())
    }

    fn ensure_within(&self, dimension: CountedBudgetDimension, requested: u64) -> Result<()> {
        let consumed = self.usage.get(dimension);
        let limit = self.limits.get(dimension);
        match consumed.checked_add(requested) {
            Some(total) if total <= limit => Ok(()),
            _ => Err(self.exceeded(dimension, requested)),
        }
    }

    fn exceeded(&self, dimension: CountedBudgetDimension, requested: u64) -> Error {
        Error::BudgetExceeded {
            dimension: dimension.into(),
            limit: self.limits.get(dimension),
            consumed: self.usage.get(dimension),
            requested,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            nested_depth: value,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        }
    }

    /// [`limits`] with the P2 dimensions opened too: every counted dimension and both
    /// high-water depths hold `value`, so a test can name the one dimension it means to
    /// exhaust while the others keep exactly the same headroom.
    fn counted_limits(value: u64) -> Limits {
        Limits {
            class_headers: value,
            method_bodies: value,
            ir_items: value,
            ir_edges: value,
            analysis_steps: value,
            normalization_clones: value,
            dependency_depth: value,
            ..limits(value)
        }
    }

    /// The six counted dimensions this slice adds, with the unit each one counts.
    ///
    /// Kept as a list so the exact-limit, overflow and cancellation cases below cannot
    /// quietly cover five of six dimensions.
    const P2_COUNTED: [(CountedBudgetDimension, &str); 6] = [
        (CountedBudgetDimension::ClassHeaders, "header read attempts"),
        (CountedBudgetDimension::MethodBodies, "body read attempts"),
        (CountedBudgetDimension::IrItems, "derived storage items"),
        (CountedBudgetDimension::IrEdges, "derived edges"),
        (CountedBudgetDimension::AnalysisSteps, "worklist steps"),
        (
            CountedBudgetDimension::NormalizationClones,
            "normalization clones",
        ),
    ];

    /// Position of one counted dimension in [`CountedBudgetDimension::ALL`].
    ///
    /// A test-time exhaustive `match`, the same anchor `tests/p2_contracts.rs` keeps on the
    /// public side: a dimension added to the enum without being added to `ALL` does not
    /// compile here. Without it, a new dimension could stay out of the set every
    /// zero-usage assertion, schema dump and fuzz bound iterates, and the omission would be
    /// invisible.
    fn counted_dimension_index(dimension: CountedBudgetDimension) -> usize {
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

    /// Position of one dimension in the declaration order of [`BudgetDimension`].
    ///
    /// Exhaustive `match` with no wildcard arm, covering the counted set and the three
    /// non-counted dimensions alike: a variant added to `BudgetDimension` does not compile
    /// until it is classified here, which is what keeps
    /// [`diagnostic_codes_are_the_serde_names_of_every_dimension`] exhaustive over the whole
    /// enum instead of only over the counted part.
    fn budget_dimension_index(dimension: BudgetDimension) -> usize {
        match dimension {
            BudgetDimension::InputBytes => 0,
            BudgetDimension::ArchiveEntries => 1,
            BudgetDimension::EntryBytes => 2,
            BudgetDimension::ReadBytes => 3,
            BudgetDimension::ClassBytes => 4,
            BudgetDimension::AttributeBytes => 5,
            BudgetDimension::CodeBytes => 6,
            BudgetDimension::ResultItems => 7,
            BudgetDimension::OutputBytes => 8,
            BudgetDimension::ClassHeaders => 9,
            BudgetDimension::MethodBodies => 10,
            BudgetDimension::IrItems => 11,
            BudgetDimension::IrEdges => 12,
            BudgetDimension::AnalysisSteps => 13,
            BudgetDimension::NormalizationClones => 14,
            BudgetDimension::NestedDepth => 15,
            BudgetDimension::DependencyDepth => 16,
            BudgetDimension::ElapsedMillis => 17,
        }
    }

    #[test]
    fn diagnostic_codes_are_the_serde_names_of_every_dimension() {
        // A report names one dimension twice: as the `BudgetDimension` variant inside
        // `BudgetExceeded`, and as the `budget_exceeded_<code>` diagnostic suffix built from
        // `artifact::budget_dimension_code`. P1 anchors its eleven codes through end-to-end
        // diagnostics; this is the table-level anchor for all eighteen, so a mis-typed
        // suffix (an `ir_edge` next to an `ir_edges` variant, say) fails here instead of
        // silently publishing a second, undocumented code.
        let mut dimensions = CountedBudgetDimension::ALL
            .iter()
            .map(|dimension| BudgetDimension::from(*dimension))
            .collect::<Vec<_>>();
        dimensions.push(BudgetDimension::NestedDepth);
        dimensions.push(BudgetDimension::DependencyDepth);
        dimensions.push(BudgetDimension::ElapsedMillis);
        assert_eq!(
            dimensions.len(),
            18,
            "eighteen dimensions: the counted fifteen plus two depths and the clock"
        );

        // The list is the declaration order, not merely a set of eighteen names: the position
        // the exhaustive match hands out has to be the position the list holds, which fails
        // if a variant is missing from, duplicated in or reordered within the list.
        for (index, dimension) in dimensions.iter().enumerate() {
            assert_eq!(
                budget_dimension_index(*dimension),
                index,
                "{dimension:?} is not at its declared position"
            );
            let serde_name = serde_json::to_string(dimension).expect("a dimension serializes");
            assert_eq!(
                crate::artifact::budget_dimension_code(*dimension),
                serde_name.trim_matches('"'),
                "{dimension:?} must keep its serde name as the diagnostic suffix"
            );
        }
    }

    #[test]
    fn exact_limit_succeeds_and_next_charge_reports_counts() {
        let mut budget = Budget::new(limits(3));
        budget.charge(CountedBudgetDimension::CodeBytes, 3).unwrap();
        let error = budget
            .charge(CountedBudgetDimension::CodeBytes, 1)
            .unwrap_err();
        assert_eq!(
            error,
            Error::BudgetExceeded {
                dimension: BudgetDimension::CodeBytes,
                limit: 3,
                consumed: 3,
                requested: 1
            }
        );
        assert_eq!(budget.usage().code_bytes, 3);
    }

    #[test]
    fn overflow_is_rejected_without_mutating_usage() {
        let mut budget = Budget::new(limits(u64::MAX));
        budget
            .charge(CountedBudgetDimension::InputBytes, u64::MAX - 1)
            .unwrap();
        let before = budget.usage().input_bytes;
        assert!(
            budget
                .charge(CountedBudgetDimension::InputBytes, 2)
                .is_err()
        );
        assert_eq!(budget.usage().input_bytes, before);
    }

    #[test]
    fn cancellation_applies_before_and_after_a_charge() {
        let mut budget = Budget::new(limits(4));
        let token = budget.cancellation_token();
        token.cancel();
        assert!(matches!(
            budget.check(CountedBudgetDimension::CodeBytes, 1),
            Err(Error::Cancelled { .. })
        ));
        assert!(matches!(
            budget.charge(CountedBudgetDimension::CodeBytes, 1),
            Err(Error::Cancelled { .. })
        ));
        assert_eq!(budget.usage().code_bytes, 0);
    }

    #[test]
    fn usage_snapshot_is_json_serializable() {
        let mut budget = Budget::new(limits(5));
        budget
            .charge(CountedBudgetDimension::ResultItems, 2)
            .unwrap();
        let mut snapshot = budget.usage();
        let json = serde_json::to_string(&snapshot).unwrap();
        let round_trip: UsageSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(
            round_trip, snapshot,
            "the snapshot survives a JSON round trip"
        );

        // A second read has to agree on everything the budget counted. `elapsed_millis` is
        // recomputed from the wall clock on every read, so it is the one field two reads may
        // differ by — zeroing it is this repository's convention for comparing a snapshot read
        // twice, rather than asserting that no millisecond can pass between two calls.
        let mut later = budget.usage();
        assert!(
            later.elapsed_millis >= snapshot.elapsed_millis,
            "the wall clock does not run backwards"
        );
        later.elapsed_millis = 0;
        snapshot.elapsed_millis = 0;
        assert_eq!(
            later, snapshot,
            "a second read agrees on every counted dimension"
        );
    }

    #[test]
    fn successful_charge_is_retained_when_cancelled_afterward() {
        let mut budget = Budget::new(limits(5));
        let token = budget.cancellation_token();
        budget.charge(CountedBudgetDimension::ReadBytes, 2).unwrap();
        token.cancel();
        assert!(matches!(budget.poll(), Err(Error::Cancelled { .. })));
        assert_eq!(budget.usage().read_bytes, 2);
    }

    #[test]
    fn elapsed_limit_is_cooperative_and_reported_in_usage() {
        let mut configured = limits(10);
        configured.elapsed_millis = 1;
        let budget = Budget::new(configured);
        std::thread::sleep(std::time::Duration::from_millis(3));
        assert!(matches!(
            budget.poll(),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
                ..
            })
        ));
        assert!(budget.usage().elapsed_millis >= 1);
    }

    #[test]
    fn nested_depth_is_a_non_counted_high_water_mark() {
        let mut configured = limits(10);
        configured.nested_depth = 1;
        let mut budget = Budget::new(configured);
        budget.check_nested_depth(0).unwrap();
        budget.check_nested_depth(1).unwrap();
        assert_eq!(budget.usage().nested_depth, 1);
        assert_eq!(
            budget.check_nested_depth(2).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::NestedDepth,
                limit: 1,
                consumed: 1,
                requested: 1,
            }
        );
        assert_eq!(budget.usage().nested_depth, 1);
        assert!(CountedBudgetDimension::try_from(BudgetDimension::NestedDepth).is_err());
    }

    #[test]
    fn elapsed_is_not_a_chargeable_dimension() {
        assert!(CountedBudgetDimension::try_from(BudgetDimension::ElapsedMillis).is_err());
        // The second high-water dimension is not chargeable either: its limit is compared
        // against a depth, never accumulated by `charge`.
        assert!(CountedBudgetDimension::try_from(BudgetDimension::DependencyDepth).is_err());
        assert_eq!(
            BudgetDimension::from(CountedBudgetDimension::ReadBytes),
            BudgetDimension::ReadBytes
        );
    }

    #[test]
    fn default_limits_are_all_zero_and_fail_closed() {
        let base = Limits::default();
        for dimension in CountedBudgetDimension::ALL {
            assert_eq!(base.counted_limit(dimension), 0, "{dimension:?}");
        }
        assert_eq!(base.nested_depth, 0);
        assert_eq!(base.dependency_depth, 0);
        assert_eq!(base.elapsed_millis, 0);

        // The base cannot run work. With the clock left at zero nothing polls through at
        // all; opening only the clock makes the first unit of every dimension exceed it.
        let stopped = Budget::new(Limits::default());
        assert!(matches!(
            stopped.poll(),
            Err(Error::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
                ..
            })
        ));

        let mut budget = Budget::new(Limits {
            elapsed_millis: u64::MAX,
            ..Limits::default()
        });
        for dimension in CountedBudgetDimension::ALL {
            assert_eq!(
                budget.charge(dimension, 1).unwrap_err(),
                Error::BudgetExceeded {
                    dimension: dimension.into(),
                    limit: 0,
                    consumed: 0,
                    requested: 1,
                },
                "{dimension:?} must not be chargeable from the zero base"
            );
        }
        assert_eq!(
            budget.observe_dependency_depth(1).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::DependencyDepth,
                limit: 0,
                consumed: 0,
                requested: 1,
            }
        );
        assert_eq!(
            budget.check_nested_depth(1).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::NestedDepth,
                limit: 0,
                consumed: 0,
                requested: 1,
            }
        );
        let usage = budget.usage();
        for dimension in CountedBudgetDimension::ALL {
            assert_eq!(usage.counted_usage(dimension), 0, "{dimension:?}");
        }
        assert_eq!(usage.dependency_depth, 0);
        assert_eq!(usage.nested_depth, 0);
    }

    #[test]
    fn counted_set_is_the_declared_dimensions_in_order() {
        assert_eq!(CountedBudgetDimension::ALL.len(), 15);
        let mut positions = Vec::new();
        for dimension in CountedBudgetDimension::ALL {
            let index = counted_dimension_index(dimension);
            assert_eq!(CountedBudgetDimension::ALL[index], dimension);
            positions.push(index);
        }
        positions.sort_unstable();
        assert_eq!(
            positions,
            (0..CountedBudgetDimension::ALL.len()).collect::<Vec<_>>(),
            "each counted dimension owns exactly one position in ALL"
        );
    }

    #[test]
    fn every_new_counted_dimension_accepts_its_exact_limit_and_rejects_the_next_unit() {
        for (dimension, unit) in P2_COUNTED {
            let mut budget = Budget::new(counted_limits(3));
            budget.check(dimension, 3).unwrap();
            assert_eq!(
                budget.usage().counted_usage(dimension),
                0,
                "`check` must not consume anything for {dimension:?} ({unit})"
            );
            budget.charge(dimension, 3).unwrap();
            assert_eq!(budget.usage().counted_usage(dimension), 3, "{dimension:?}");
            assert_eq!(
                budget.charge(dimension, 1).unwrap_err(),
                Error::BudgetExceeded {
                    dimension: dimension.into(),
                    limit: 3,
                    consumed: 3,
                    requested: 1,
                },
                "{dimension:?} ({unit})"
            );
            let usage = budget.usage();
            assert_eq!(usage.counted_usage(dimension), 3, "{dimension:?}");
            for other in CountedBudgetDimension::ALL {
                if other != dimension {
                    assert_eq!(
                        usage.counted_usage(other),
                        0,
                        "exhausting {dimension:?} moved {other:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn zero_units_and_zero_depth_are_accepted_at_a_zero_limit() {
        for (dimension, unit) in P2_COUNTED {
            let mut budget = Budget::new(counted_limits(0));
            // Zero work is within every limit, including zero; the first real unit is not.
            budget.charge(dimension, 0).unwrap();
            assert_eq!(
                budget.usage().counted_usage(dimension),
                0,
                "{dimension:?} ({unit})"
            );
            assert_eq!(
                budget.charge(dimension, 1).unwrap_err(),
                Error::BudgetExceeded {
                    dimension: dimension.into(),
                    limit: 0,
                    consumed: 0,
                    requested: 1,
                },
                "{dimension:?} ({unit})"
            );
        }

        let mut budget = Budget::new(counted_limits(0));
        budget.observe_dependency_depth(0).unwrap();
        budget.check_nested_depth(0).unwrap();
        let usage = budget.usage();
        assert_eq!(usage.dependency_depth, 0);
        assert_eq!(usage.nested_depth, 0);
        assert_eq!(
            budget.observe_dependency_depth(1).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::DependencyDepth,
                limit: 0,
                consumed: 0,
                requested: 1,
            }
        );
        assert_eq!(
            budget.check_nested_depth(1).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::NestedDepth,
                limit: 0,
                consumed: 0,
                requested: 1,
            }
        );
    }

    #[test]
    fn ir_storage_units_accumulate_nodes_slots_edges_and_clones() {
        let mut budget = Budget::new(counted_limits(u64::MAX));

        // One method's derived storage, charged item by item the way 3.x/4.x will: 3
        // frame/local slots, 4 SSA values, 2 phi inputs, 3 origin members.
        for items in [3, 4, 2, 3] {
            budget
                .charge(CountedBudgetDimension::IrItems, items)
                .unwrap();
        }
        assert_eq!(budget.usage().ir_items, 12);

        // Edges are a separate dimension: 2 CFG edges, 1 exception edge, 5 def-use edges.
        for edges in [2, 1, 5] {
            budget
                .charge(CountedBudgetDimension::IrEdges, edges)
                .unwrap();
        }
        assert_eq!(budget.usage().ir_edges, 8);

        // A `jsr`/`ret` normalization announces its clones before creating them.
        budget
            .charge(CountedBudgetDimension::NormalizationClones, 2)
            .unwrap();
        assert_eq!(budget.usage().normalization_clones, 2);

        // Header/Body attempts are their own dimensions and stay untouched by IR work.
        assert_eq!(budget.usage().class_headers, 0);
        assert_eq!(budget.usage().method_bodies, 0);
        assert_eq!(budget.usage().analysis_steps, 0);
    }

    #[test]
    fn new_counted_dimensions_reject_the_overflowing_unit() {
        for (dimension, unit) in P2_COUNTED {
            let mut budget = Budget::new(counted_limits(u64::MAX));
            budget.charge(dimension, u64::MAX).unwrap();
            assert_eq!(
                budget.usage().counted_usage(dimension),
                u64::MAX,
                "{dimension:?} ({unit})"
            );
            // `u64::MAX + 1` overflows: the checked add must report the dimension instead of
            // wrapping to zero or panicking, and must not consume the rejected unit.
            assert_eq!(
                budget.charge(dimension, 1).unwrap_err(),
                Error::BudgetExceeded {
                    dimension: dimension.into(),
                    limit: u64::MAX,
                    consumed: u64::MAX,
                    requested: 1,
                },
                "{dimension:?} ({unit})"
            );
            assert_eq!(budget.usage().counted_usage(dimension), u64::MAX);
        }
    }

    #[test]
    fn exhausted_analysis_steps_stop_the_next_step_on_the_steps_dimension() {
        let mut budget = Budget::new(counted_limits(2));
        budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)
            .unwrap();
        budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)
            .unwrap();
        assert_eq!(
            budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)
                .unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
                limit: 2,
                consumed: 2,
                requested: 1,
            }
        );
        // The stopped worklist must not be reported as a storage or edge stop: those
        // dimensions still have their full headroom.
        budget.charge(CountedBudgetDimension::IrItems, 2).unwrap();
        budget.charge(CountedBudgetDimension::IrEdges, 2).unwrap();
        assert_eq!(budget.usage().analysis_steps, 2);
    }

    #[test]
    fn cancellation_blocks_new_charges_and_both_depth_observations() {
        let mut budget = Budget::new(counted_limits(7));
        let token = budget.cancellation_token();
        token.cancel();
        for dimension in CountedBudgetDimension::ALL {
            assert!(
                matches!(budget.check(dimension, 1), Err(Error::Cancelled { .. })),
                "{dimension:?}"
            );
            assert!(
                matches!(budget.charge(dimension, 1), Err(Error::Cancelled { .. })),
                "{dimension:?}"
            );
        }
        assert!(matches!(
            budget.observe_dependency_depth(1),
            Err(Error::Cancelled { .. })
        ));
        assert!(matches!(
            budget.check_nested_depth(1),
            Err(Error::Cancelled { .. })
        ));
        let usage = budget.usage();
        for dimension in CountedBudgetDimension::ALL {
            assert_eq!(usage.counted_usage(dimension), 0, "{dimension:?}");
        }
        assert_eq!(usage.dependency_depth, 0);
        assert_eq!(usage.nested_depth, 0);
    }

    #[test]
    fn an_exhausted_dimension_stays_exhausted_for_the_request() {
        // 3.5/5.1 fall back on the facts already read instead of resetting the budget and
        // re-reading the Body; this is the budget side of that rule.
        let mut budget = Budget::new(counted_limits(2));
        budget
            .charge(CountedBudgetDimension::AnalysisSteps, 2)
            .unwrap();
        for _ in 0..3 {
            assert_eq!(
                budget
                    .charge(CountedBudgetDimension::AnalysisSteps, 1)
                    .unwrap_err(),
                Error::BudgetExceeded {
                    dimension: BudgetDimension::AnalysisSteps,
                    limit: 2,
                    consumed: 2,
                    requested: 1,
                }
            );
        }
        assert_eq!(budget.usage().analysis_steps, 2);

        let mut depth_budget = Budget::new(counted_limits(1));
        depth_budget.observe_dependency_depth(1).unwrap();
        for _ in 0..2 {
            assert_eq!(
                depth_budget.observe_dependency_depth(2).unwrap_err(),
                Error::BudgetExceeded {
                    dimension: BudgetDimension::DependencyDepth,
                    limit: 1,
                    consumed: 1,
                    requested: 1,
                }
            );
        }
        assert_eq!(depth_budget.usage().dependency_depth, 1);

        // Only a new `Budget` is a new lifecycle.
        let fresh = Budget::new(counted_limits(2));
        assert_eq!(fresh.usage().analysis_steps, 0);
        assert_eq!(fresh.usage().dependency_depth, 0);
    }

    #[test]
    fn dependency_depth_keeps_the_deepest_accepted_value() {
        let mut budget = Budget::new(counted_limits(4));
        budget.observe_dependency_depth(0).unwrap();
        assert_eq!(budget.usage().dependency_depth, 0);
        budget.observe_dependency_depth(2).unwrap();
        budget.observe_dependency_depth(1).unwrap();
        assert_eq!(
            budget.usage().dependency_depth,
            2,
            "a shallower closure must not lower the high-water mark"
        );
        budget.observe_dependency_depth(4).unwrap();
        assert_eq!(budget.usage().dependency_depth, 4);
        assert_eq!(
            budget.observe_dependency_depth(5).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::DependencyDepth,
                limit: 4,
                consumed: 4,
                requested: 1,
            }
        );
        assert_eq!(budget.usage().dependency_depth, 4);
    }

    #[test]
    fn the_two_depth_dimensions_are_independent() {
        let mut configured = counted_limits(10);
        configured.nested_depth = 1;
        configured.dependency_depth = 3;
        let mut budget = Budget::new(configured);

        budget.check_nested_depth(1).unwrap();
        budget.observe_dependency_depth(3).unwrap();
        let usage = budget.usage();
        assert_eq!(usage.nested_depth, 1);
        assert_eq!(usage.dependency_depth, 3);

        // The container depth is exhausted: the dependency depth still accepts the depth it
        // was given, and its own usage is untouched.
        assert_eq!(
            budget.check_nested_depth(2).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::NestedDepth,
                limit: 1,
                consumed: 1,
                requested: 1,
            }
        );
        budget.observe_dependency_depth(3).unwrap();
        assert_eq!(budget.usage().nested_depth, 1);
        assert_eq!(budget.usage().dependency_depth, 3);

        // ... and the other way around: the dependency closure stops at depth 4 while the
        // accepted container depth (1) stays accepted and reported as its own dimension.
        assert_eq!(
            budget.observe_dependency_depth(4).unwrap_err(),
            Error::BudgetExceeded {
                dimension: BudgetDimension::DependencyDepth,
                limit: 3,
                consumed: 3,
                requested: 1,
            }
        );
        budget.check_nested_depth(1).unwrap();
        let usage = budget.usage();
        assert_eq!(usage.nested_depth, 1);
        assert_eq!(usage.dependency_depth, 3);
        for dimension in CountedBudgetDimension::ALL {
            assert_eq!(
                usage.counted_usage(dimension),
                0,
                "{dimension:?} is not a depth dimension and must stay untouched"
            );
        }
    }

    #[test]
    fn counted_dimensions_are_addressed_by_value() {
        let mut budget = Budget::new(counted_limits(7));
        budget
            .charge(CountedBudgetDimension::ResultItems, 2)
            .unwrap();
        let usage = budget.usage();

        // `ALL` is the counted set itself, not a subset of it, and it keeps the
        // declaration order the usage table uses.
        let counted = CountedBudgetDimension::ALL
            .iter()
            .map(|dimension| BudgetDimension::from(*dimension))
            .collect::<Vec<_>>();
        assert_eq!(
            counted,
            vec![
                BudgetDimension::InputBytes,
                BudgetDimension::ArchiveEntries,
                BudgetDimension::EntryBytes,
                BudgetDimension::ReadBytes,
                BudgetDimension::ClassBytes,
                BudgetDimension::AttributeBytes,
                BudgetDimension::CodeBytes,
                BudgetDimension::ResultItems,
                BudgetDimension::OutputBytes,
                BudgetDimension::ClassHeaders,
                BudgetDimension::MethodBodies,
                BudgetDimension::IrItems,
                BudgetDimension::IrEdges,
                BudgetDimension::AnalysisSteps,
                BudgetDimension::NormalizationClones,
            ]
        );

        for dimension in CountedBudgetDimension::ALL {
            let expected = if dimension == CountedBudgetDimension::ResultItems {
                2
            } else {
                0
            };
            assert_eq!(usage.counted_usage(dimension), expected, "{dimension:?}");
            assert_eq!(budget.limits().counted_limit(dimension), 7, "{dimension:?}");
        }
    }
}
