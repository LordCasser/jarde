//! Bulk recovery: one library operation over an explicit physical scope (change
//! `add-parallel-bulk-recovery`, tasks 3.1–3.3 and 4.1–4.5).
//!
//! [`Engine::recover_all`] answers one whole scope in one operation: every class candidate the
//! scope's own incremental traversal yields is prepared **once**, every method that class declares
//! is analysed and presented against that one preparation, and every result is handed to the
//! caller's sink as soon as it exists — in physical order, bounded in memory, and cancellable at
//! every wait.
//!
//! [`Engine::recover_all`]: crate::Engine::recover_all
//!
//! # One operation, two execution configurations
//!
//! `workers == 1` runs on the calling thread: the coordinator *is* the class task, and each result
//! is delivered the moment it is produced. `workers > 1` creates at most
//! `W = min(requested_workers, floor(max_buffered_result_weight / max_result_weight))` standard
//! library scoped threads, one class task each, and the calling thread becomes the coordinator that
//! delivers. Both configurations run the *same* class task and the same delivery code, so the
//! results a run publishes are a function of the input and the semantic configuration alone — never
//! of how many workers happened to be running (design decision 8).
//!
//! # The stream
//!
//! The sink sees `Header`, then per class `ClassPrepared → Method* → ClassEnd`, then `Final`. A
//! class that could not be prepared publishes a located `Diagnostic` and a `ClassEnd` that states
//! the refusal instead of a `ClassPrepared` record, because a class nobody read declares an
//! **unknown** number of methods and no record may state otherwise. `Diagnostic` events are
//! interleaved only where they have a physical position. Every callback runs on the coordinator
//! thread: the trait asks for no `Send` or `Sync` bound of its implementor, and no operation lock is
//! ever held across a callback.
//!
//! # The window and its backpressure
//!
//! At most `W` class tasks are active at once, each has **one** pending method-result slot and a
//! fixed small control slot for the `ClassPrepared` record, and the control slot carries no product
//! weight. A worker that produced a result waits for its own class's slot to be taken before it
//! starts the next method, so the retained product weight of the whole operation is bounded by
//! `W * max_result_weight`, which the effective worker count makes no larger than
//! `max_buffered_result_weight`. The coordinator always delivers from the **earliest** active class,
//! so a slow first class cannot be overtaken by a later fast one and a full window can never form a
//! capacity cycle: the earliest class's own slot is always free when the coordinator is waiting on
//! it.
//!
//! A single method result larger than the fixed `max_result_weight` stops **that method** locally,
//! with the weight and the limit it exceeded in its record: its Java text is discarded rather than
//! truncated, no success is claimed, and no worker waits for capacity that can never fit it.
//!
//! # The container a class task reads
//!
//! A candidate comes from the container the walk was inside when it yielded it, and that container
//! travels with the candidate: the slot keeps the walk's own active handle
//! ([`ContainerFactsHandle`]) from the dispatch until the worker takes the task, and the worker
//! holds it until that class's task ends. It is what makes the window a handover rather than a
//! gap — the walk opens one container at a time and leaves it behind as soon as its entries are
//! dispatched, so with more than one worker the ordinary case is a class task that reads its class
//! *after* the walk has moved on. Every read of that container — the class entry's own verified
//! read and each member's loader binding query — is then answered from the product the discovery
//! already paid for, whatever the caller's facts store retains ([`crate::FactsCache`]).
//!
//! Holding one is not retaining one: the handle is released with the class task that ends, and the
//! decision about a container nothing is using any more is the store's own, exactly as before. A
//! container whose classes are still dispatched is never "nothing is using it" — that is the one
//! case this handover decides.
//!
//! # One total, many workers
//!
//! Discovery, preparation, each method's execution and each delivery bill to **one**
//! [`OperationLedger`] built from the budget the caller opened the operation with. Per-method local
//! limits still apply on top (`method_limits`), and a method that runs out of its own allowance
//! stops locally without touching the operation's other classes. A refusal of the *operation's*
//! total closes dispatch, wakes every waiter and joins every worker; work that already took its
//! permit stays billed even when it was never delivered.
//!
//! The consumer's own bytes are part of that same total rather than a second allowance kept beside
//! it: the header hands the sink a [`DeliveryAccount`], and a record's bytes are charged to the
//! operation's `delivery` share **before** any of them is written. "Can this record be written" is
//! therefore answered by the declaration the operation's own work is answered by, and the two cannot
//! each stay inside one number while their sum exceeds it.
//!
//! # Cancellation
//!
//! Every wait is bounded and observes both the caller's cancellation token (which the ledger took
//! from the entry budget) and this operation's own close signal. Checkpoints sit before dispatching a
//! class, while waiting for a slot, while waiting for a worker's result, inside discovery between
//! entries and before every sink call. A `SinkControl::Stop`, a global budget stop or a caller
//! cancellation stops dispatch, wakes every waiter and joins every worker before this module
//! returns — no thread of it outlives the call, and no second run ever starts.
//!
//! # What this module does not do
//!
//! It does not decide what to recover: the scope, the environment, the workers and every capacity are
//! the caller's explicit declaration, and a request this module cannot serve (no worker can be
//! created for the stated window, an unusable value for a capacity) is refused **before** anything is
//! dispatched. It does not encode, write or flush anything: the sink owns its own I/O, and only a
//! callback that returned `Ok` counts as a delivery.

use crate::environment::ResolutionEnvironment;
use crate::ir::{AnalysisStage, MethodAnalysisReport, MethodAnalysisRequest, MethodBodyState};
use crate::{
    BudgetDimension, ClassBytesId, ContainerId, Coverage, CoverageDimension, CoverageRange,
    CoverageState, Diagnostic, DiagnosticSeverity, Error, ExecutionReport, FactsCapacity, Limits,
    PhysicalClassLocation, PhysicalDefinitionId, PhysicalMethodId, PhysicalView, Provenance,
    RecoveredMethod, Result, SnapshotId, TerminationReason, UsageSnapshot,
};
use jarde_reader::artifact::{ArtifactSnapshot, ContainerFactsHandle};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::ledger::{BulkStop, BulkStopKind, OperationLedger, UsageOwner};
use jarde_reader::model::{JvmBytes, PhysicalVariant, physical_variant_for_path};
use jarde_reader::prepared::{MethodOrdinal, PreparedClass, PreparedClassRead};
use jarde_reader::scope_cursor::ScopeClass;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::Duration;

mod observation;
#[cfg(feature = "test-support")]
pub use observation::{BulkProbe, BulkProbeReading, LedgerSiteReading, WindowSiteReading};
use observation::{Observation, WindowSite};

/// The default per-class preparation ceiling of a bulk request, in bytes.
///
/// A class whose own bytes are larger than this is a **refused** class: its method count stays
/// unknown and the operation continues with the classes beside it (design decision 6). The value is
/// the bounded default the CLI publishes in its `Header`, not a claim about any artifact.
pub const DEFAULT_MAX_CLASS_BYTES: u64 = 16 * 1024 * 1024;

/// The default fixed per-result retention ceiling, in bytes.
pub const DEFAULT_MAX_RESULT_WEIGHT: u64 = 2 * 1024 * 1024;

/// The default whole-window retention ceiling, in bytes: sixteen results of the default per-result
/// ceiling, so the default configuration can hold sixteen class tasks' pending results at once.
pub const DEFAULT_MAX_BUFFERED_RESULT_WEIGHT: u64 = 32 * 1024 * 1024;

/// One bulk recovery request: the scope, the environment and the explicit resources.
///
/// Every field is the caller's declaration. `workers` is an explicit positive number — `0` is an
/// input error, never "the machine's own count" — and the three capacities are explicit bounds,
/// because a library that guessed them would be a library whose memory a caller cannot predict.
/// [`BulkRecoveryRequest::for_scope`] builds the configuration the CLI and the tests share, and
/// [`BulkRecoveryRequest::with_capacities`] states the three bounds explicitly.
///
/// The request names its snapshot and scope twice on purpose: once as the physical view the
/// operation walks, and once inside the [`EnvironmentRequest`] the reads are performed under. The
/// two must agree (a mismatch is
/// `bulk_environment_view_mismatch`), so a caller cannot hand one scope to discovery and another to
/// the resolver by accident.
///
/// The type is `Clone + Debug` and serializable, but deliberately **not** `PartialEq`: a request is
/// a declaration a caller hands over, and the one field two callers could compare it by — the
/// test-only fault injection — is not part of what a request means.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkRecoveryRequest {
    /// The one snapshot this operation reads.
    pub snapshot: SnapshotId,
    /// The physical scope discovery walks, in the reader's own order.
    pub scope: crate::PhysicalScope,
    /// The environment every method of this operation is read and analysed under.
    pub environment: crate::EnvironmentRequest,
    /// How many class tasks may run at once, as the caller asks for it. `0` is an input error.
    pub workers: usize,
    /// The local limits **one method** runs under, from the start of its analysis to its presented
    /// result. Queueing time and shared preparation are not part of it.
    pub method_limits: Limits,
    /// The largest class this operation prepares; a larger one is a refused class. `0` is an input
    /// error.
    pub max_class_bytes: u64,
    /// The fixed retention ceiling of **one** method result. `0` is an input error, and the value
    /// does not change with the worker count.
    pub max_result_weight: u64,
    /// The retention ceiling of the whole result window. It must fit at least one
    /// `max_result_weight`; a smaller value is an input error.
    pub max_buffered_result_weight: u64,
    /// The optional evidence every method's presentation materializes (change
    /// `add-demand-driven-core-results`, D1).
    ///
    /// The operation is the *full export* of a scope — every method of every class, assembled into
    /// one stream — so its own constructor states [`crate::RecoveryEvidenceRequest::all`], which is
    /// what it has always delivered; a caller narrows it with
    /// [`BulkRecoveryRequest::with_evidence`]. A document that states no selection is read as the
    /// same full export, never as a third default.
    #[serde(default = "crate::RecoveryEvidenceRequest::all")]
    pub evidence: crate::RecoveryEvidenceRequest,
    /// Test-only fault injection for the worker lifecycle (task 4.2's evidence).
    #[cfg(feature = "test-support")]
    #[serde(skip)]
    pub faults: BulkFaults,
    /// Test-only observation of how this operation was coordinated, when a caller attaches one.
    #[cfg(feature = "test-support")]
    #[serde(skip)]
    pub probe: Option<std::sync::Arc<BulkProbe>>,
}

impl BulkRecoveryRequest {
    /// The request a caller that has one scope and one environment means: the published defaults for
    /// every capacity, and the worker count and method limits the caller states.
    ///
    /// The physical view is taken from the environment, so the two halves of the request cannot
    /// disagree about which snapshot and scope the operation is about.
    pub fn for_scope(
        environment: crate::EnvironmentRequest,
        workers: usize,
        method_limits: Limits,
    ) -> Self {
        Self {
            snapshot: environment.snapshot.clone(),
            scope: environment.scope.clone(),
            environment,
            workers,
            method_limits,
            max_class_bytes: DEFAULT_MAX_CLASS_BYTES,
            max_result_weight: DEFAULT_MAX_RESULT_WEIGHT,
            max_buffered_result_weight: DEFAULT_MAX_BUFFERED_RESULT_WEIGHT,
            evidence: crate::RecoveryEvidenceRequest::all(),
            #[cfg(feature = "test-support")]
            faults: BulkFaults::default(),
            #[cfg(feature = "test-support")]
            probe: None,
        }
    }

    /// The same request, observed by `probe`.
    ///
    /// The probe is not part of what a request *means* — no field of a report, a stream record or a
    /// stop states an observation — so it is attached here rather than declared in the request's own
    /// constructor. See [`BulkProbe`] for what it counts and what it costs.
    #[cfg(feature = "test-support")]
    pub fn with_probe(mut self, probe: std::sync::Arc<BulkProbe>) -> Self {
        self.probe = Some(probe);
        self
    }

    /// The same request with the evidence every method's presentation materializes stated
    /// explicitly.
    pub fn with_evidence(mut self, evidence: crate::RecoveryEvidenceRequest) -> Self {
        self.evidence = evidence;
        self
    }

    /// The same request with the three capacities stated explicitly.
    ///
    /// This is the one place they are set together, so a caller — or a CLI flag set — cannot state
    /// two of them and inherit the third by accident.
    pub fn with_capacities(
        mut self,
        max_class_bytes: u64,
        max_result_weight: u64,
        max_buffered_result_weight: u64,
    ) -> Self {
        self.max_class_bytes = max_class_bytes;
        self.max_result_weight = max_result_weight;
        self.max_buffered_result_weight = max_buffered_result_weight;
        self
    }
}

/// The effective configuration of one bulk operation: what the caller asked for, what the operation
/// actually uses, and why they differ.
///
/// The one difference a valid request can have is the worker count:
/// `workers_effective = min(workers_requested, floor(max_buffered_result_weight / max_result_weight))`
/// — a window that cannot hold `workers` results runs fewer class tasks rather than shrinking the
/// per-result ceiling, because the per-result ceiling is the caller's bound on one result and no
/// worker count may move it. Both values are published here, and the reduction is never silent.
///
/// `facts_capacity` is the retention capacity of the store the entry budget carries, or
/// [`FactsCapacity::none`] when the caller attached none: a zero capacity closes cross-consumption
/// retention and changes nothing about the facts the operation is holding right now.
///
/// `total` is the operation's own ceiling — the entry budget's limits, which every part of the
/// operation is admitted against — and `method` is the local limit of **one** method. The two are
/// published separately because they are separate declarations: a caller may open a whole package
/// under a ceiling that funds it and still hold each method to its own single-request allowance, and a
/// consumer that reads only this record has to be able to tell which number bounds which work.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkLimits {
    /// The worker count the request asked for.
    pub workers_requested: usize,
    /// The worker count this operation uses: `min(requested, window / per-result ceiling)`, at
    /// least one.
    pub workers_effective: usize,
    /// The operation's own total limits: the entry budget's, shared by discovery, every method and
    /// delivery. Never reset and never multiplied by the worker count.
    pub total: Limits,
    /// The local limits one method runs under, as the request declared them. They bound that method
    /// alone: a method that runs out of them stops without touching the operation's other classes.
    pub method: Limits,
    /// The per-class preparation ceiling, as the request declared it.
    pub max_class_bytes: u64,
    /// The fixed per-result retention ceiling, as the request declared it.
    pub max_result_weight: u64,
    /// The whole-window retention ceiling, as the request declared it.
    pub max_buffered_result_weight: u64,
    /// The retention capacity of the store this operation reads through.
    pub facts_capacity: FactsCapacity,
}

impl BulkLimits {
    /// The effective configuration of `request`, or the input error that makes it unservable.
    ///
    /// Four values are input errors and none of them is clamped: `workers == 0` (the caller has to
    /// say how many it wants), `max_result_weight == 0` (no result could ever fit),
    /// `max_buffered_result_weight < max_result_weight` (the window cannot hold even one result if
    /// that result is as large as the per-result ceiling allows) and `max_class_bytes == 0` (no
    /// class could ever be prepared). The window-derived worker reduction is the one adjustment a
    /// valid request may get, and it is published rather than hidden.
    pub fn effective_from(
        request: &BulkRecoveryRequest,
        total: &Limits,
        facts_capacity: FactsCapacity,
    ) -> Result<Self> {
        if request.workers == 0 {
            return Err(Error::invalid_input(
                "bulk_workers_zero",
                "a bulk operation needs an explicit positive worker count: `0` is not \
                 \"as many as this machine has\", and no worker count is inferred here",
            ));
        }
        if request.max_result_weight == 0 {
            return Err(Error::invalid_input(
                "bulk_max_result_weight_zero",
                "`max_result_weight` is a fixed per-result retention ceiling and `0` refuses every \
                 result; state a positive ceiling or do not run a bulk operation",
            ));
        }
        if request.max_buffered_result_weight < request.max_result_weight {
            return Err(Error::invalid_input(
                "bulk_window_too_small",
                format!(
                    "the result window holds {} bytes while one result may retain {}: a window that \
                     cannot hold a single result of the stated ceiling is refused rather than \
                     silently widened or narrowed",
                    request.max_buffered_result_weight, request.max_result_weight
                ),
            ));
        }
        if request.max_class_bytes == 0 {
            return Err(Error::invalid_input(
                "bulk_max_class_bytes_zero",
                "`max_class_bytes` is the preparation ceiling of one class and `0` refuses every \
                 class; state a positive ceiling or do not run a bulk operation",
            ));
        }
        let window = request.max_buffered_result_weight / request.max_result_weight;
        let effective = usize::try_from(window)
            .unwrap_or(usize::MAX)
            .min(request.workers)
            .max(1);
        Ok(Self {
            workers_requested: request.workers,
            workers_effective: effective,
            total: total.clone(),
            method: request.method_limits.clone(),
            max_class_bytes: request.max_class_bytes,
            max_result_weight: request.max_result_weight,
            max_buffered_result_weight: request.max_buffered_result_weight,
            facts_capacity,
        })
    }
}

// ---------------------------------------------------------------------------------------------
// The stream: what a caller's sink receives, in the order it receives it
// ---------------------------------------------------------------------------------------------

/// What the sink answered for one event.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SinkControl {
    /// Take the rest of the stream.
    Continue,
    /// Stop the operation: dispatch closes, every waiter wakes, every worker is joined, and the
    /// prefix the sink already confirmed stands.
    Stop,
}

/// The operation's own output account, handed to the sink with the stream's `Header`.
///
/// A consumer's records are bytes, and those bytes are charged to **the operation's** one total
/// (design decision 4: encoding for delivery is the `delivery` part of the ledger every read and
/// every method bills to) instead of to a second allowance an adapter keeps beside it. That is the
/// whole point of the handle: "may this record be written" is answered by the same declaration the
/// operation's own work is answered by, so the two cannot each stay inside a number and still exceed
/// their sum.
///
/// # What a consumer does with it
///
/// * ask [`DeliveryAccount::remaining_output_bytes`] for what the operation could still charge, and
///   hold no more than that while encoding;
/// * ask [`DeliveryAccount::charge_output_bytes`] for the permit of a record **before** writing any
///   byte of it. A refused permit is a refusal: not one byte of that record is written, the prefix
///   the consumer already confirmed stands, and the stop the ledger records names the dimension it
///   needed and the delivery owner that needed it;
/// * keep the handle for the rest of the stream, and for as long as the consumer's own bookkeeping
///   needs it. It is a handle on one total rather than a reading taken at the header, so what it
///   reports moves while the operation's other parts work.
///
/// # What it is not
///
/// The record weight itself is not the consumer's charge: the library bills one `ResultItems` per
/// record it hands over, before the callback. This handle is for the bytes the *consumer* owns.
///
/// A charge that took its permit stays charged. A cancellation that arrives after the permit does
/// not refund it, and a permitted record that then fails to be written is still billed: the permit
/// is for the work, not for its success. The handle locks the operation's total only for its own
/// charge — the library never holds that lock while it calls the sink, and a consumer needs no lock
/// of its own for its own bookkeeping.
#[derive(Clone, Debug)]
pub struct DeliveryAccount {
    /// The operation's total, shared with every part of the operation.
    ledger: OperationLedger,
    /// The `output_bytes` ceiling the operation was opened with: the entry budget's own total limit,
    /// which is the one any charge of this dimension is admitted against. It is read here once so a
    /// consumer never has to re-declare a number the operation already holds.
    output_bytes_limit: u64,
}

impl DeliveryAccount {
    /// The account of one operation: the ledger every part of it bills to, and the entry budget's
    /// own `output_bytes` ceiling.
    fn of(ledger: &OperationLedger, limits: &Limits) -> Self {
        Self {
            ledger: ledger.clone(),
            output_bytes_limit: limits.output_bytes,
        }
    }

    /// How many output bytes the operation could still charge, as its total stands right now.
    ///
    /// This is the reading of one total — `limit - usage` — and nothing else: it takes no permit and
    /// records no stop. A consumer that sizes its buffer by it can still be refused by
    /// [`DeliveryAccount::charge_output_bytes`] afterwards, because the operation's other parts
    /// charge the same dimension while the consumer encodes, and only the charge is a decision.
    pub fn remaining_output_bytes(&self) -> u64 {
        self.output_bytes_limit.saturating_sub(
            self.ledger
                .usage()
                .counted_usage(CountedBudgetDimension::OutputBytes),
        )
    }

    /// Takes the permit for `bytes` bytes of the consumer's own output, as the operation's
    /// **delivery** work.
    ///
    /// The charge is the operation's own: it is admitted against the total the caller declared and
    /// billed to [`UsageOwner::Delivery`], and a refusal is the operation's first stop — the
    /// dimension it needed and the owner that needed it travel with that record. `Err` means the
    /// bytes were **not** admitted: no byte of a record refused here may be written.
    pub fn charge_output_bytes(&self, bytes: u64) -> Result<()> {
        self.ledger.charge(
            UsageOwner::Delivery,
            CountedBudgetDimension::OutputBytes,
            bytes,
        )
    }
}

/// The typed consumer of one bulk operation's stream.
///
/// The callbacks run on the **coordinator thread only** — the calling thread of
/// [`crate::Engine::recover_all`] — so an implementor needs no `Send`, no `Sync` and no locking of
/// its own. Every callback returns [`SinkControl`], which is how a consumer that has seen enough
/// stops the operation cooperatively, and every callback returns a [`Result`], whose `Err` is an
/// I/O or consumer failure: it closes dispatch, joins every worker and ends the operation as
/// `Failed`, never as a silently short stream.
///
/// A record is **delivered** exactly when its callback returned `Ok`; the delivery counts a caller
/// reads are the confirmations, and an event whose callback was refused is not one of them.
pub trait RecoverySink {
    /// The view this operation covers, its effective configuration, and the operation's own output
    /// account. It is the first event of every stream, published before anything is discovered.
    ///
    /// The account travels with the header because the header is the one event a stream starts with:
    /// a consumer that charges its own bytes stores the handle here and keeps it for the rest of the
    /// stream ([`DeliveryAccount`]). A stream that published no header publishes nothing else either
    /// — an operation stopped before its first record delivers no record at all — so no consumer is
    /// ever asked to write a record without having been handed the account it may write it under.
    fn header(&mut self, event: &BulkHeaderEvent, delivery: DeliveryAccount)
    -> Result<SinkControl>;

    /// One class prepared: its identity, the shared read evidence of its one read and how many
    /// method records it declares. It carries no constant pool and no member table.
    fn class_prepared(&mut self, event: &ClassPreparedEvent) -> Result<SinkControl>;

    /// One method's record, in the physical and declaration order of its class.
    fn method(&mut self, event: &MethodResultEvent) -> Result<SinkControl>;

    /// The end of one class: what it declared, what it ended with, and the class's own execution
    /// plane.
    fn class_end(&mut self, event: &ClassEndEvent) -> Result<SinkControl>;

    /// One located diagnostic: a damaged container subtree, a refused class, or the stop that ended
    /// the operation. It is published where its physical position puts it in the stream.
    fn diagnostic(&mut self, event: &BulkDiagnosticEvent) -> Result<SinkControl>;

    /// The last event of a stream that reached its end. A stream without it is not a confirmed
    /// completion: a consumer that never saw a `Final` has a prefix, whatever the prefix holds.
    fn final_event(&mut self, event: &BulkFinalEvent) -> Result<SinkControl>;
}

/// The first event of every bulk stream: the view it covers and the configuration it runs under.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkHeaderEvent {
    pub view: PhysicalView,
    /// The effective configuration, both worker values included, so a caller reads what the
    /// operation really used rather than what it asked for.
    pub limits: BulkLimits,
}

/// One class prepared: identity, the evidence of its one read, and the count it declares.
///
/// The record is deliberately small — identity, the trusted class-bytes identity, the container it
/// lives in, its depth, the number of method records the class file declares and where the member
/// walk stopped, if it did. The constant pool and the member table stay where they are: a control
/// record is not a product, and it is excluded from the result window's weight.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassPreparedEvent {
    /// This class's position in the delivered stream, counted from zero in physical order.
    pub class_ordinal: u64,
    pub location: PhysicalClassLocation,
    /// The trusted identity of the class bytes this preparation read.
    pub class_bytes: ClassBytesId,
    /// The container the class entry lives in.
    pub container: ContainerId,
    /// How deep that container is: `0` for a snapshot's own root.
    pub depth: u64,
    /// How many method records the class file declares. With a member-table stop the records after
    /// it were never read, so more methods may be declared than the stream will state.
    pub declared_methods: u64,
    /// Where the member-table walk stopped, when it did not read to the declared end.
    pub member_table_stop: Option<jarde_reader::classfile::MemberTableStop>,
}

/// How one class's part of the stream ended.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClassCompletion {
    /// The class was prepared and every method record its own read reached was executed and handed
    /// over — each one with its own disposition, which may well be a refusal or a stop.
    Completed {
        /// How many method records this class published.
        methods: u64,
    },
    /// The class was **not** prepared: its declaration did not decode, or its bytes exceeded the
    /// preparation ceiling. The number of methods it declares is unknown, and this record states
    /// that rather than a number.
    Refused {
        /// The error code that refused it.
        code: String,
    },
    /// A class the operation closed on: its task did not reach the end of its member table, so the
    /// members after the last record it handed over are unknown. The records it did hand over stand,
    /// and the stop's own code says why it stopped there.
    Stopped {
        /// How many method records this class published before the operation closed.
        methods: u64,
        /// The code of the stop that closed it.
        code: String,
    },
}

/// The end of one class: what it declared and what its execution reached.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassEndEvent {
    /// The same ordinal the class's `ClassPrepared` (or its refusal) carried.
    pub class_ordinal: u64,
    pub location: PhysicalClassLocation,
    pub completion: ClassCompletion,
    /// The class's own execution plane: the strongest state its method records reached, restated
    /// under the operation's usage. A class whose every method completed is `Complete` here even
    /// when another class of the same operation did not.
    pub execution: ExecutionReport,
}

/// What one delivered method record holds.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum MethodDelivery {
    /// A run happened and its presentation is carried: the analysis report, the recovery report,
    /// the on-demand callee evidence and the facts, exactly as [`crate::Engine::recover_method`]
    /// hands them over for one method.
    Recovered(Box<RecoveredMethod>),
    /// The declaration states it has no body (`abstract`/`native`). No phase ran, so there is no
    /// artifact and no text — a declaration without a body is an answer, not a failure.
    NoBody { analysis: Box<MethodAnalysisReport> },
    /// The request was refused before any analysis ran, so nothing was produced. The stop and the
    /// diagnostic that state it travel with the record.
    Refused {
        execution: ExecutionReport,
        diagnostics: Vec<Diagnostic>,
    },
    /// The run's result was larger than the fixed per-result retention ceiling, so it is **not**
    /// delivered: its text is discarded rather than truncated and no success is claimed. The record
    /// stays small — the weight, the ceiling, the run's execution and the codes of the run's
    /// diagnostics — because a record that could not fit the window may not replace it.
    Oversized {
        /// The weight the produced result was accounted at.
        weight: u64,
        /// The fixed per-result ceiling it exceeded.
        limit: u64,
        execution: ExecutionReport,
        /// The run's own diagnostic codes, without their messages.
        diagnostic_codes: Vec<String>,
    },
}

impl MethodDelivery {
    /// The disposition bucket this record counts in.
    pub fn outcome(&self) -> BulkMethodOutcome {
        match self {
            Self::Recovered(recovered) => match recovered.recovery().content {
                jarde_java::RecoveryContent::ContainsStatements => BulkMethodOutcome::Produced,
                jarde_java::RecoveryContent::ExplanationOnly => BulkMethodOutcome::ExplanationOnly,
                jarde_java::RecoveryContent::NotProduced => BulkMethodOutcome::NotProduced,
            },
            Self::NoBody { .. } => BulkMethodOutcome::NoBody,
            Self::Refused { .. } => BulkMethodOutcome::Refused,
            Self::Oversized { .. } => BulkMethodOutcome::Oversized,
        }
    }

    /// Whether this method's execution ran to its end.
    ///
    /// A produced or explanation-only artifact whose run completed is a **complete** method: the
    /// operation's aggregate is not lowered by a result whose content is only an explanation (design
    /// decision 6). A refusal, a run that stopped, and a result the window refused are not.
    pub fn execution_incomplete(&self) -> bool {
        self.execution_planes().iter().any(|plane| !complete(plane))
            || matches!(self, Self::Refused { .. } | Self::Oversized { .. })
    }

    /// Every execution plane of this record: the run's own and the presentation's, in that order.
    fn execution_planes(&self) -> Vec<ExecutionReport> {
        match self {
            Self::Recovered(recovered) => vec![
                recovered.analysis().execution.clone(),
                recovered.recovery().execution.clone(),
            ],
            Self::NoBody { analysis } => vec![analysis.execution.clone()],
            Self::Refused { execution, .. } | Self::Oversized { execution, .. } => {
                vec![execution.clone()]
            }
        }
    }
}

/// One method's disposition, as the operation's per-outcome counts bucket it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkMethodOutcome {
    /// A run delivered an artifact holding at least one statement.
    Produced,
    /// A run delivered an artifact holding no statement: the envelope, its reasons and the quoted
    /// bytecode of the regions it refused.
    ExplanationOnly,
    /// A run stopped before an artifact was committed.
    NotProduced,
    /// The method was refused before any artifact could exist.
    Refused,
    /// The declaration states there is no body.
    NoBody,
    /// The run's result exceeded the fixed per-result ceiling, so it was not delivered.
    Oversized,
}

/// One method's record in the stream.
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MethodResultEvent {
    /// The delivering class's position in the stream.
    pub class_ordinal: u64,
    /// The member record's position inside that class's own declaration order.
    pub member_ordinal: u64,
    /// The physical identity of the member: its owner definition, its raw name and descriptor.
    pub method: PhysicalMethodId,
    pub delivery: MethodDelivery,
    /// The retention weight this record asked the window for. Zero for every record that retains no
    /// artifact.
    pub weight: u64,
}

impl MethodResultEvent {
    /// The disposition bucket this record counts in.
    pub fn outcome(&self) -> BulkMethodOutcome {
        self.delivery.outcome()
    }
}

/// One located diagnostic.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkDiagnosticEvent {
    /// The class the diagnostic is published at, when it belongs to one.
    pub class_ordinal: Option<u64>,
    /// The diagnostic itself, with the physical position it was found at.
    pub diagnostic: Diagnostic,
}

/// The last event of a stream that reached its end: the bounded counts and boundaries of the whole
/// operation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkFinalEvent {
    pub summary: BulkSummary,
}

// ---------------------------------------------------------------------------------------------
// The report: bounded counts, boundaries and the operation's own account
// ---------------------------------------------------------------------------------------------

/// How many methods of the operation ended in each disposition.
///
/// The first three are the design's outcome buckets beside `NoBody`; [`BulkMethodOutcome::Refused`]
/// and [`BulkMethodOutcome::Oversized`] are stated separately rather than folded together, because
/// "the request was refused before any analysis ran" and "the run completed and its result did not
/// fit the stated ceiling" are two different facts about the work that happened.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkOutcomeCounts {
    pub produced: u64,
    pub explanation_only: u64,
    pub not_produced: u64,
    pub refused: u64,
    pub no_body: u64,
    pub oversized: u64,
}

impl BulkOutcomeCounts {
    /// Counts one method record in its bucket.
    fn count(&mut self, outcome: BulkMethodOutcome) {
        let slot = match outcome {
            BulkMethodOutcome::Produced => &mut self.produced,
            BulkMethodOutcome::ExplanationOnly => &mut self.explanation_only,
            BulkMethodOutcome::NotProduced => &mut self.not_produced,
            BulkMethodOutcome::Refused => &mut self.refused,
            BulkMethodOutcome::NoBody => &mut self.no_body,
            BulkMethodOutcome::Oversized => &mut self.oversized,
        };
        *slot = slot.saturating_add(1);
    }

    /// Every method this operation recorded a disposition for: the method records it produced,
    /// whether or not they were delivered.
    pub fn total(&self) -> u64 {
        self.produced
            .saturating_add(self.explanation_only)
            .saturating_add(self.not_produced)
            .saturating_add(self.refused)
            .saturating_add(self.no_body)
            .saturating_add(self.oversized)
    }
}

/// The bounded summary one bulk operation publishes, in its `Final` event and in its report.
///
/// Discovery, execution and delivery are counted **separately**, because they are three different
/// facts: a class the traversal yielded is *seen*; a class whose declaration decoded is *prepared*,
/// and only then does the operation know how many methods it declares; a method whose run started is
/// *executed*; a record the sink confirmed is *delivered*. A class the operation refused declares an
/// **unknown** number of methods, so it contributes nothing to `methods_declared` — the honest
/// reading of a class nobody read — and it is counted in `classes_refused` and lowers the aggregate
/// to at least `Partial`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkSummary {
    pub view: PhysicalView,
    pub limits: BulkLimits,
    /// Class candidates the traversal yielded.
    pub classes_seen: u64,
    /// Classes whose declaration was prepared. A class counted here contributes its declared method
    /// count to [`BulkSummary::methods_declared`], whether or not its `ClassPrepared` record reached
    /// the sink.
    pub classes_prepared: u64,
    /// Classes that were **not** prepared: an undecodable declaration, or bytes over the
    /// preparation ceiling. Their method counts are unknown.
    pub classes_refused: u64,
    /// Method records declared by the prepared classes. Refused classes are outside this
    /// denominator.
    pub methods_declared: u64,
    /// Methods a run was started for, whatever that run then produced, stopped at or was refused
    /// with.
    pub methods_executed: u64,
    /// Method records the sink confirmed with `Ok`.
    pub methods_delivered: u64,
    /// `methods_declared - methods_executed`: the declared methods no run was started for.
    pub methods_not_executed: u64,
    pub outcomes: BulkOutcomeCounts,
    /// Whether discovery reached the end of the scope **and** every container it holds was walked.
    /// A damaged subtree leaves this false: the classes it would have held are unknown, and no count
    /// above is a scope-wide denominator.
    pub traversal_complete: bool,
    /// The operation's aggregate state: infrastructure `Failed`, then a global `Cancelled`, then
    /// `Partial` (a budget stop, an incomplete method execution, a refused class or an unfinished
    /// traversal), and `Complete` only when the traversal reached the end, every required execution
    /// completed and every produced record was delivered.
    pub execution: ExecutionReport,
}

impl BulkSummary {
    /// The aggregate state names a caller can branch on without matching the usage figures.
    pub fn status(&self) -> &'static str {
        status_of(&self.execution)
    }
}

/// The retention window one operation kept, and what it observed while it ran.
///
/// The limits are the configuration's; the high-water marks are what this run really reached. Both
/// are published because "the window was bounded" is a claim about a run, not only about a
/// declaration. In the serial configuration the coordinator *is* the class task and hands each record
/// over as it produces it, so the weight figures are that one record's weight — the most product the
/// operation ever held at once — rather than a queue's depth.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkWindow {
    /// At most this many class tasks are active at once.
    pub active_classes: u64,
    /// The most class tasks that were executing at the same time.
    pub concurrent_classes_high_water: u64,
    /// The most window slots the operation held at once: one per class task dispatched and not yet
    /// ended. The serial configuration opens none — the coordinator *is* the class task there — so
    /// this is zero for it however many classes the scope holds, while a worker configuration holds
    /// at most [`BulkWindow::active_classes`] of them.
    pub window_slots_high_water: u64,
    /// The largest retained weight of pending method results this run reached.
    pub buffered_weight_high_water: u64,
    /// The largest weight one method result was accounted at.
    pub largest_result_weight: u64,
    /// The fixed per-result ceiling the run used.
    pub result_weight_limit: u64,
    /// The whole-window ceiling the run used.
    pub buffered_weight_limit: u64,
}

/// One bulk operation's report: the summary, the operation's own account and the boundaries it
/// reached.
///
/// The report holds no per-method results: those are the stream's, and this module never collects
/// them. It holds the bounded counts, the operation's usage (total, entry, and each of the three
/// attributable parts), the window, the coverage planes, the first stop and the located diagnostics
/// of the operation itself.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkRecoveryReport {
    pub summary: BulkSummary,
    /// The operation's totals: the entry budget's own usage plus every owner's work.
    pub usage: UsageSnapshot,
    /// The usage the operation started from, never reset.
    pub entry_usage: UsageSnapshot,
    /// What discovery and class preparation were charged.
    pub discovery_usage: UsageSnapshot,
    /// What method execution was charged, across every worker.
    pub method_usage: UsageSnapshot,
    /// What delivering records to the sink was charged.
    pub delivery_usage: UsageSnapshot,
    pub window: BulkWindow,
    pub coverage: Coverage,
    /// The first stop the operation observed: a budget stop, a cancellation, a consumer that
    /// stopped, or an infrastructure failure. Later observations never replace it.
    pub stop: Option<BulkStop>,
    /// The operation's own located diagnostics: damaged subtree, refused class, and the stop that
    /// ended it. Per-method dispositions are the stream's records, not this list's.
    pub diagnostics: Vec<Diagnostic>,
    /// Whether the sink confirmed the `Final` event. A stream whose `Final` was refused is a prefix,
    /// and no consumer may read its completion from the records it did see.
    pub final_delivered: bool,
}

/// Test-only fault injection for the worker lifecycle (task 4.2).
///
/// The shape exists so the lifecycle failures the design requires evidence for — a worker the
/// operating system refuses to create, a worker that panics, and two class tasks that really
/// overlap — can be exercised through the public entry point without a second code path: the faults
/// are read at the places the real failures would be observed, and the operation's behaviour behind
/// them is the behaviour a real failure meets.
#[cfg(feature = "test-support")]
#[derive(Clone, Debug, Default)]
pub struct BulkFaults {
    /// Refuse the creation of worker number `n` (zero-based), as the operating system would.
    pub fail_worker_at: Option<usize>,
    /// Panic inside the class task with this delivery ordinal, before it prepares anything.
    pub panic_at_class_ordinal: Option<u64>,
    /// Hold every class task for this long before it reads its class.
    ///
    /// The hold sits inside the task, on the worker that runs it, so the traversal keeps going while
    /// the reads wait: it is how a test states what a read sees when the walk has already left the
    /// container the candidate came from. It changes nothing else — no dispatch, window, delivery or
    /// cancellation semantics depend on it.
    pub class_delay: Option<Duration>,
    /// A rendezvous each class task passes through before it prepares. It records how many class
    /// tasks were inside it at once, which is the difference between "two classes were dispatched"
    /// and "two class tasks really ran at the same time".
    pub class_gate: Option<std::sync::Arc<ClassGate>>,
    /// Where the worker threads themselves are counted, so a test can state that none outlives the
    /// call rather than inferring it from the scope's own promise.
    pub worker_watch: Option<std::sync::Arc<WorkerWatch>>,
}

/// A count of the worker threads this operation has alive, for a test that wants to state "none is
/// left" as a measurement rather than as an inference from the scoped threads' own promise.
#[cfg(feature = "test-support")]
#[derive(Debug, Default)]
pub struct WorkerWatch {
    alive: std::sync::atomic::AtomicUsize,
    alive_high_water: std::sync::atomic::AtomicUsize,
}

#[cfg(feature = "test-support")]
impl WorkerWatch {
    pub fn new() -> Self {
        Self::default()
    }

    fn enter(&self) {
        let alive = self.alive.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        self.alive_high_water
            .fetch_max(alive, std::sync::atomic::Ordering::SeqCst);
    }

    fn exit(&self) {
        self.alive.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }

    /// How many worker threads are alive right now.
    pub fn alive(&self) -> usize {
        self.alive.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// The most worker threads that were alive at the same time.
    pub fn high_water(&self) -> usize {
        self.alive_high_water
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// A bounded rendezvous for class tasks, used as overlap evidence.
///
/// Each class task that arrives waits — for a bounded time, so a runner that never reaches the
/// expected number cannot hang — until `expected` tasks have arrived, and the gate records the most
/// tasks it ever held at once. A runner that executes class tasks one at a time can only ever reach a
/// peak of one, whatever its dispatch window says.
///
/// The gate opens for **every** task waiting in it, not only for the one that completed the count:
/// the last task to leave closes it again, so the next round of class tasks starts a fresh
/// rendezvous.
#[cfg(feature = "test-support")]
#[derive(Debug)]
pub struct ClassGate {
    expected: usize,
    timeout: Duration,
    state: Mutex<GateState>,
    changed: Condvar,
}

#[cfg(feature = "test-support")]
#[derive(Debug, Default)]
struct GateState {
    arrived: usize,
    peak: usize,
    opened: bool,
}

#[cfg(feature = "test-support")]
impl ClassGate {
    /// A gate that opens when `expected` class tasks have arrived, or after `timeout`.
    pub fn new(expected: usize, timeout: Duration) -> Self {
        Self {
            expected: expected.max(1),
            timeout,
            state: Mutex::new(GateState::default()),
            changed: Condvar::new(),
        }
    }

    /// Waits until `expected` class tasks are here, and records how many were.
    pub fn arrive(&self) {
        let mut state = self.lock();
        state.arrived = state.arrived.saturating_add(1);
        state.peak = state.peak.max(state.arrived);
        if state.arrived >= self.expected {
            state.opened = true;
        }
        self.changed.notify_all();
        let deadline = std::time::Instant::now() + self.timeout;
        while !state.opened {
            let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) else {
                break;
            };
            let (guard, _) = self
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = guard;
        }
        state.arrived = state.arrived.saturating_sub(1);
        if state.arrived == 0 {
            // Everyone that was let through has left: the next round starts closed.
            state.opened = false;
        }
        self.changed.notify_all();
    }

    /// The most class tasks this gate ever held at the same time.
    pub fn peak(&self) -> usize {
        self.lock().peak
    }

    fn lock(&self) -> MutexGuard<'_, GateState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

// ---------------------------------------------------------------------------------------------
// The operation: the state its coordinator and its workers share
// ---------------------------------------------------------------------------------------------

/// How long one wait may pass before it re-checks the close signal.
///
/// Long enough that an idle waiter is not a spin loop, short enough that a cancellation or a stop
/// raised somewhere else is noticed without any unbounded sleep.
const WAIT_SLICE: Duration = Duration::from_millis(20);

/// How long the coordinator waits for one already-dispatched class task to state where it stopped
/// when the operation closes. A task checks the operation's own close signal at every handover, so it
/// returns at once from a slot check and only a task in the middle of a method needs any of this.
const DRAIN_PER_CLASS: Duration = Duration::from_millis(500);

/// Why the operation is closing, and what the aggregate will be built from.
#[derive(Clone, Debug)]
enum Ending {
    /// The operation stopped: the ledger holds the reason, whether it was the operation's total, the
    /// caller's cancellation or a consumer that stopped accepting records.
    Stopped,
    /// Infrastructure the operation cannot continue past: a worker that was not created or did not
    /// return, or a sink callback that failed.
    Infrastructure { code: String, message: String },
    /// The operation reached its own end: every class it dispatched was delivered and the traversal
    /// either reached the end of the scope or stopped without a recorded stop.
    Reached,
}

/// Why one class task, or one delivery, could not go on.
#[derive(Clone, Debug)]
enum Closing {
    /// The operation stopped. The ledger already holds the reason; the error is what this site saw.
    Stopped(Error),
    /// The sink itself failed: an I/O or consumer failure, which is infrastructure.
    Consumer(Error),
}

impl Closing {
    /// The error this closing happened with.
    fn error(&self) -> &Error {
        match self {
            Self::Stopped(error) | Self::Consumer(error) => error,
        }
    }

    /// The code a diagnostic states for it.
    fn code(&self) -> String {
        crate::facade::error_code(self.error())
    }
}

/// How one class task ended.
///
/// The class's own execution plane travels with the ending — the strongest state its method records
/// reached — because the class task is the only place that sees every one of them as it is produced,
/// whatever the coordinator then does with them.
enum ClassEnding {
    /// The class was prepared and every member record its own read reached was handed over.
    Prepared {
        methods: u64,
        execution: Option<ExecutionReport>,
    },
    /// The class was never prepared.
    Refused(ClassRefusal),
    /// The operation closed before this class finished: the records already handed over stand, and
    /// the members after them are unknown.
    Closed {
        methods: u64,
        closing: Option<Closing>,
        execution: Option<ExecutionReport>,
    },
}

/// One class that could not be prepared, and why.
struct ClassRefusal {
    code: String,
    message: String,
    execution: ExecutionReport,
}

/// What one class's slot holds for the coordinator.
struct Slot {
    /// The class candidate this task is about, so the worker that takes it reads what the traversal
    /// yielded rather than a name derived from a path.
    class: ScopeClass,
    /// The container the walk was inside when it yielded this candidate, kept as that walk's own
    /// active handle ([`ContainerFactsHandle`]) until the worker takes the task.
    ///
    /// The walk holds the container it is inside and no other, and it leaves that container as soon
    /// as its entries are dispatched. Without this field a class task that runs after the walk moved
    /// on would find nothing holding the container it reads out of, and would parse the directory
    /// again per class — the container would be rebuilt while consumers of it were still waiting to
    /// read it, and the number of parses would grow with the class count instead of with the
    /// containers the scope holds. `None` is the one case that has no container to hand over: a
    /// standalone `CLASS` snapshot's root candidate.
    container: Option<ContainerFactsHandle>,
    /// The control record waiting for the coordinator, at most one (`ClassPrepared`).
    control: Option<ClassPreparedEvent>,
    /// The method record waiting for the coordinator, at most one.
    method: Option<Box<MethodResultEvent>>,
    /// The class task's own result, once it returned. Its presence makes the class "finished".
    ending: Option<ClassEnding>,
}

impl Slot {
    fn new(class: ScopeClass, container: Option<ContainerFactsHandle>) -> Self {
        Self {
            class,
            container,
            control: None,
            method: None,
            ending: None,
        }
    }
}

/// The bounded counts one operation keeps, whichever thread observed the fact.
#[derive(Clone, Default)]
struct Counts {
    classes_seen: u64,
    classes_prepared: u64,
    classes_refused: u64,
    methods_declared: u64,
    methods_executed: u64,
    methods_delivered: u64,
    /// Method records whose own execution did not complete.
    incomplete_methods: u64,
    outcomes: BulkOutcomeCounts,
}

/// Everything the coordinator and the workers of one operation share.
///
/// One mutex holds all of it because the fields move together: a class task finishing, the
/// coordinator taking its record and the window's weight all change in one step. The lock covers
/// field reads and writes only — never a read, a decode, a recovery, a sink callback or a join.
#[derive(Default)]
struct OperationState {
    counts: Counts,
    /// The class slots in dispatch order. The front is the earliest active class, and the
    /// coordinator never delivers past it.
    slots: VecDeque<Slot>,
    /// The delivery ordinal of the front slot: the class ordinal of `slots[i]` is `front + i`.
    front: u64,
    /// How many class tasks were dispatched; their ordinals are `0..dispatched`.
    dispatched: u64,
    /// The next dispatched task a worker may take.
    next_unstarted: u64,
    /// The traversal reached the end of the scope.
    traversal_done: bool,
    /// Class tasks executing right now, and the most that ever were.
    executing: u64,
    executing_high_water: u64,
    /// The most window slots this operation ever held at once: one per dispatched class the
    /// coordinator had not yet delivered the end of.
    slot_count_high_water: u64,
    /// The retained weight of the pending method records, and the most it ever was.
    retained_weight: u64,
    retained_weight_high_water: u64,
    largest_result_weight: u64,
    /// The operation's close signal. Once set it is never replaced, so the first reason is the one
    /// the report states.
    ending: Option<Ending>,
    /// The strongest execution state any method of this operation published, folded as the class
    /// tasks produced it. It is the *reason* a non-`Complete` aggregate states, so the report names
    /// the failure that really happened rather than a generic one.
    strongest: Option<ExecutionReport>,
}

struct Registry {
    state: Mutex<OperationState>,
    changed: Condvar,
    /// The operation's observation port: where a window wait is stated, when one is attached.
    observation: Observation,
}

impl Registry {
    fn new(observation: Observation) -> Self {
        Self {
            state: Mutex::new(OperationState::default()),
            changed: Condvar::new(),
            observation,
        }
    }

    fn lock(&self) -> MutexGuard<'_, OperationState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Waits on the window's own signal, stating the wait to the operation's observation port.
    ///
    /// One place, so "how long did a waiter sit on this window" is answered at the wait itself
    /// rather than by timing the call that contains it: a call that found its record waiting for it
    /// waited for nothing.
    fn wait_signal<'a>(
        &self,
        state: MutexGuard<'a, OperationState>,
        site: WindowSite,
        timeout: Duration,
    ) -> MutexGuard<'a, OperationState> {
        let started = self.observation.wait_started();
        let (guard, _) = self
            .changed
            .wait_timeout(state, timeout)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.observation.waited(site, started);
        guard
    }

    /// Closes the operation, keeping the first reason. Every waiter is woken.
    fn close(&self, ending: Ending) {
        let mut state = self.lock();
        if state.ending.is_none() {
            state.ending = Some(ending);
        }
        self.changed.notify_all();
    }

    /// The ending the operation already has, when it has one.
    fn ending(&self) -> Option<Ending> {
        self.lock().ending.clone()
    }

    /// Records one class candidate the traversal yielded.
    fn note_seen(&self) {
        let mut state = self.lock();
        state.counts.classes_seen = state.counts.classes_seen.saturating_add(1);
    }

    /// Publishes that the traversal reached the end of the scope.
    ///
    /// A worker waiting for work and the coordinator waiting for a record both read this flag, so it
    /// is the one fact that lets them return instead of waiting for a dispatch that will not come.
    fn note_traversal_done(&self) {
        let mut state = self.lock();
        state.traversal_done = true;
        self.changed.notify_all();
    }

    /// Records one class prepared, with the number of methods it declares.
    fn note_prepared(&self, declared: u64) {
        let mut state = self.lock();
        state.counts.classes_prepared = state.counts.classes_prepared.saturating_add(1);
        state.counts.methods_declared = state.counts.methods_declared.saturating_add(declared);
        self.observation.class_declared(declared);
    }

    /// Records one class that was not prepared: its method count stays unknown.
    fn note_refused(&self) {
        let mut state = self.lock();
        state.counts.classes_refused = state.counts.classes_refused.saturating_add(1);
    }

    /// Records one method record the task produced, wherever it is delivered afterwards.
    fn note_method(&self, event: &MethodResultEvent) {
        let mut state = self.lock();
        state.counts.methods_executed = state.counts.methods_executed.saturating_add(1);
        if event.delivery.execution_incomplete() {
            state.counts.incomplete_methods = state.counts.incomplete_methods.saturating_add(1);
        }
        let outcome = event.delivery.outcome();
        state.counts.outcomes.count(outcome);
        // The method's own execution planes are the operation's strongest: the class task sees them
        // as it produces them, whichever configuration runs it.
        let mut strongest = state.strongest.take();
        for plane in event.delivery.execution_planes() {
            match &mut strongest {
                Some(current) => crate::facade::merge_execution(current, plane),
                None => strongest = Some(plane),
            }
        }
        state.strongest = strongest;
    }

    /// Records one method record the sink confirmed.
    fn note_delivered(&self) {
        let mut state = self.lock();
        state.counts.methods_delivered = state.counts.methods_delivered.saturating_add(1);
    }

    /// Records one result the serial configuration produced and is about to hand over.
    ///
    /// The serial configuration retains a result for exactly as long as its own delivery takes — the
    /// coordinator *is* the class task and hands each record over as it produces it — so the weight
    /// it accounts is that one record's. The figure is the same statement the worker configuration's
    /// slots make: the most product this operation ever held at once, which for one class task at a
    /// time is one result.
    fn note_direct_result(&self, weight: u64) {
        let mut state = self.lock();
        state.retained_weight_high_water = state.retained_weight_high_water.max(weight);
        state.largest_result_weight = state.largest_result_weight.max(weight);
    }

    /// Counts one class task as executing, until the returned guard is dropped.
    fn enter(&self) -> Entered<'_> {
        let mut state = self.lock();
        state.executing = state.executing.saturating_add(1);
        state.executing_high_water = state.executing_high_water.max(state.executing);
        Entered { registry: self }
    }

    /// Dispatches one class task into the window and answers the ordinal it delivers under.
    ///
    /// The container the walk handed over with this candidate enters the window with it: the walk may
    /// leave that container before any worker reaches this class, and the handle is what keeps the
    /// verified product — not merely its origin — alive for the task that will read it.
    fn dispatch_class(&self, class: ScopeClass, container: Option<ContainerFactsHandle>) -> u64 {
        let mut state = self.lock();
        let ordinal = state.dispatched;
        state.slots.push_back(Slot::new(class, container));
        state.dispatched = state.dispatched.saturating_add(1);
        state.slot_count_high_water = state
            .slot_count_high_water
            .max(u64::try_from(state.slots.len()).unwrap_or(u64::MAX));
        self.changed.notify_all();
        ordinal
    }

    /// Takes the next delivery ordinal for a class the coordinator runs **itself**, without opening a
    /// window slot for it.
    ///
    /// The serial configuration needs the ordinal of each class and nothing else: the coordinator *is*
    /// the class task, so every record goes straight to the sink where it is produced and the class's
    /// end is published at the same place. A slot opened for such a class would never be taken —
    /// nothing drains a window in that configuration — so it would stay in the window for the rest of
    /// the operation, grow with the class count, and leave the final drain waiting for an end that
    /// nobody owes it (a wait that costs the operation its deadline, and with it the `Final` record).
    fn next_ordinal(&self) -> u64 {
        let mut state = self.lock();
        let ordinal = state.dispatched;
        state.dispatched = state.dispatched.saturating_add(1);
        ordinal
    }

    /// Takes the next class task a worker may run, with the container the walk handed over for it,
    /// or `None` when this operation has no more work for it.
    ///
    /// The handle *leaves* the slot here instead of being cloned out of it: from this point the task
    /// holds its own container for as long as it runs, and the window keeps nothing of it — which is
    /// what makes the release a property of the task's lifetime rather than of the coordinator's
    /// delivery order.
    ///
    /// `None` is answered for a closed operation, and for one whose traversal ended with every
    /// dispatched task already taken. Anything else waits — bounded, and observing the close signal —
    /// for the coordinator to dispatch more.
    fn take_task(&self) -> Option<(u64, ScopeClass, Option<ContainerFactsHandle>)> {
        self.observation.window_call(WindowSite::TakeTask);
        let mut state = self.lock();
        loop {
            if state.ending.is_some() {
                return None;
            }
            if state.next_unstarted < state.dispatched {
                let index = state.next_unstarted;
                state.next_unstarted = state.next_unstarted.saturating_add(1);
                let position = usize::try_from(index.checked_sub(state.front)?).ok()?;
                let slot = state.slots.get_mut(position)?;
                let class = slot.class.clone();
                let container = slot.container.take();
                return Some((index, class, container));
            }
            if state.traversal_done {
                return None;
            }
            state = self.wait_signal(state, WindowSite::TakeTask, WAIT_SLICE);
        }
    }

    /// Hands one prepared-class control record to the class's slot, waiting — bounded — for the
    /// coordinator to take the record before it.
    ///
    /// The wait observes the operation's close signal every slice, so a consumer that stopped or a
    /// cancellation that recorded a stop releases a worker that is waiting here.
    fn place_control(
        &self,
        index: u64,
        event: ClassPreparedEvent,
    ) -> std::result::Result<bool, Closing> {
        self.observation.window_call(WindowSite::PlaceControl);
        let mut state = self.lock();
        loop {
            let Some(position) = position_of(&state, index) else {
                return Ok(false);
            };
            let free = state
                .slots
                .get(position)
                .is_some_and(|slot| slot.control.is_none());
            if free {
                if let Some(slot) = state.slots.get_mut(position) {
                    slot.control = Some(event);
                }
                self.changed.notify_all();
                return Ok(true);
            }
            state = self.wait_signal(state, WindowSite::PlaceControl, WAIT_SLICE);
        }
    }

    /// Hands one method record to the class's slot, waiting — bounded — for its own slot to be
    /// taken. The weight is accounted while the record waits, which is exactly what the window's
    /// ceiling bounds.
    fn place_method(
        &self,
        index: u64,
        event: MethodResultEvent,
    ) -> std::result::Result<bool, Closing> {
        self.observation.window_call(WindowSite::PlaceMethod);
        let mut state = self.lock();
        loop {
            let Some(position) = position_of(&state, index) else {
                return Ok(false);
            };
            let free = state
                .slots
                .get(position)
                .is_some_and(|slot| slot.method.is_none());
            if free {
                let weight = event.weight;
                state.retained_weight = state.retained_weight.saturating_add(weight);
                state.retained_weight_high_water =
                    state.retained_weight_high_water.max(state.retained_weight);
                state.largest_result_weight = state.largest_result_weight.max(weight);
                if let Some(slot) = state.slots.get_mut(position) {
                    slot.method = Some(Box::new(event));
                }
                self.changed.notify_all();
                return Ok(true);
            }
            state = self.wait_signal(state, WindowSite::PlaceMethod, WAIT_SLICE);
        }
    }

    /// Publishes the result of one class task, whatever it was.
    ///
    /// A class task's own result is published even after the operation closed: the slots are still
    /// there to state where each dispatched class stopped, and that is exactly what a closed stream
    /// owes its consumer.
    fn finish_task(&self, index: u64, ending: ClassEnding) {
        let mut state = self.lock();
        let Some(position) = slot_of(&state, index) else {
            return;
        };
        if let Some(slot) = state.slots.get_mut(position)
            && slot.ending.is_none()
        {
            slot.ending = Some(ending);
        }
        self.changed.notify_all();
    }
}

/// One class task's presence in the operation, as the overlap metric counts it.
struct Entered<'a> {
    registry: &'a Registry,
}

/// Where one dispatched class's slot sits inside the window, or `None` when the operation is ending
/// or the slot has already been delivered.
///
/// The window's front is the earliest active class: a slot's position is its ordinal minus that
/// front, and an ordinal behind the front is a class already delivered rather than a slot to write.
fn position_of(state: &OperationState, index: u64) -> Option<usize> {
    if state.ending.is_some() {
        return None;
    }
    slot_of(state, index)
}

/// Where one class's slot sits inside the window, whatever the operation's own state is.
///
/// A class task's *result* is published through this: the class really ran, and the record of where
/// it stopped is owed even when the operation closed underneath it.
fn slot_of(state: &OperationState, index: u64) -> Option<usize> {
    index
        .checked_sub(state.front)
        .and_then(|position| usize::try_from(position).ok())
}

impl Drop for Entered<'_> {
    fn drop(&mut self) {
        let mut state = self.registry.lock();
        state.executing = state.executing.saturating_sub(1);
    }
}

/// Where one class task hands its records as it produces them.
trait ResultConsumer {
    /// Hands over the prepared-class control record. `Ok(false)` means the operation is closing.
    fn prepared(&mut self, event: ClassPreparedEvent) -> std::result::Result<bool, Closing>;

    /// Hands over one method record. `Ok(false)` means the operation is closing.
    fn method(&mut self, event: MethodResultEvent) -> std::result::Result<bool, Closing>;
}

/// The operation-wide facts one class task runs with: what it reads, under which environment, to
/// which account.
struct Operation<'a> {
    content: &'a [ArtifactSnapshot],
    snapshot: &'a ArtifactSnapshot,
    environment: &'a ResolutionEnvironment,
    limits: &'a BulkLimits,
    ledger: &'a OperationLedger,
    stages: &'a [AnalysisStage],
    /// The evidence selection every method's presentation runs under (change
    /// `add-demand-driven-core-results`, D1): one operation states it once, and the presentation it
    /// hands to each method consumes it verbatim. Nothing else of this operation depends on it —
    /// the discovery, the class tasks, the window and the ledger are the same for every selection —
    /// which is why this is a field of the shared facts rather than a second mode of the operation.
    evidence: &'a crate::RecoveryEvidenceRequest,
    registry: &'a Registry,
    /// The store handle the entry budget carried, shared with every budget of this operation, so
    /// one operation consults one store whichever part of it is charging.
    store: crate::OperationStore,
    /// The local limits the entry budget declared, which bound the discovery half exactly as they
    /// bound it before this operation existed: one class read and one preparation are discovery
    /// work, and the operation's own total bounds them again beside this.
    discovery_limits: Limits,
    /// The operation's observation port, empty unless its caller attached a probe.
    observation: Observation,
    /// Test-only fault injection.
    faults: Faults,
}

impl Operation<'_> {
    /// Whether the operation itself has stopped: a caller's cancellation, or any stop the ledger
    /// recorded. A *local* refusal — this method's own limit, a damaged body — is deliberately not
    /// this: it stops the work it refused and leaves the rest of the operation alone.
    fn stopped(&self) -> bool {
        self.ledger.stop_reason().is_some()
    }

    /// One budget that does part of this operation's work, with the given local limits.
    ///
    /// Every budget of the operation bills to the same ledger, so no worker starts from a fresh
    /// allowance, and each carries the caller's store handle so one cache is consulted by all of
    /// them. The local limits are the *only* thing that differs between them: discovery runs under
    /// the entry limits, a method under `method_limits`, delivery under the entry limits again.
    fn budget_for(&self, limits: Limits, owner: UsageOwner) -> Budget {
        let mut budget = self.store.attached_to(Budget::new(limits));
        budget.with_ledger(self.ledger.clone(), owner);
        budget
    }
}

/// The fault injection one operation runs with, empty in a build without the test-support feature.
///
/// The wrapper exists so the production paths read the same questions in either build and answer
/// "no" when nothing was injected: no `cfg` sits in the middle of the class task or the worker loop,
/// and a build that cannot be asked for a fault carries no field, no gate and no branch.
#[derive(Clone, Default)]
struct Faults {
    #[cfg(feature = "test-support")]
    fail_worker_at: Option<usize>,
    #[cfg(feature = "test-support")]
    panic_at_class_ordinal: Option<u64>,
    #[cfg(feature = "test-support")]
    class_delay: Option<Duration>,
    #[cfg(feature = "test-support")]
    class_gate: Option<std::sync::Arc<ClassGate>>,
    #[cfg(feature = "test-support")]
    worker_watch: Option<std::sync::Arc<WorkerWatch>>,
}

impl Faults {
    /// The faults one request declares, in a build that can declare any.
    #[cfg(feature = "test-support")]
    fn from_request(request: &BulkRecoveryRequest) -> Self {
        Self {
            fail_worker_at: request.faults.fail_worker_at,
            panic_at_class_ordinal: request.faults.panic_at_class_ordinal,
            class_delay: request.faults.class_delay,
            class_gate: request.faults.class_gate.clone(),
            worker_watch: request.faults.worker_watch.clone(),
        }
    }

    /// The faults one request declares. A build without `test-support` cannot be asked for any, so
    /// the request carries none and this answers an empty set.
    #[cfg(not(feature = "test-support"))]
    fn from_request(_request: &BulkRecoveryRequest) -> Self {
        Self::default()
    }

    /// Whether the creation of worker number `index` is refused.
    fn creation_refused(&self, _index: usize) -> bool {
        #[cfg(feature = "test-support")]
        {
            self.fail_worker_at == Some(_index)
        }
        #[cfg(not(feature = "test-support"))]
        {
            false
        }
    }

    /// Whether the class task with this ordinal panics before it prepares anything.
    fn panics_at(&self, _class_ordinal: u64) -> bool {
        #[cfg(feature = "test-support")]
        {
            self.panic_at_class_ordinal == Some(_class_ordinal)
        }
        #[cfg(not(feature = "test-support"))]
        {
            false
        }
    }

    /// The rendezvous every class task passes through before it prepares, when a test asked for one.
    ///
    /// In a build that cannot ask for one this reads the same call site and does nothing, so the
    /// class task has one spelling of "a test may be watching how many of me run at once".
    fn arrive_at_gate(&self) {
        #[cfg(feature = "test-support")]
        if let Some(gate) = self.class_gate.as_ref() {
            gate.arrive();
        }
    }

    /// Holds this class task, when a test asked for a delay, before it reads its class.
    ///
    /// The hold is at the head of the task and nowhere else: the traversal, the window and the
    /// delivery keep running, so what the read below meets is the state a delayed worker really
    /// meets — the walk may have left the container this candidate came from. A build that cannot ask
    /// for a delay keeps the same call site and returns at once.
    fn delay_before_prepare(&self) {
        #[cfg(feature = "test-support")]
        if let Some(delay) = self.class_delay {
            std::thread::sleep(delay);
        }
    }

    /// Marks one worker thread alive for as long as the returned guard lives.
    ///
    /// A build that cannot watch the workers gets a zero-sized guard, so the worker loop keeps one
    /// spelling of "this thread is one of the operation's workers".
    fn register_worker(&self) -> WorkerMark<'_> {
        #[cfg(feature = "test-support")]
        if let Some(watch) = self.worker_watch.as_ref() {
            watch.enter();
        }
        WorkerMark { faults: self }
    }
}

/// One worker thread's presence in the operation, as a test's own watch counts it.
///
/// The field is read only under `test-support`, where the watch exists; a production build keeps the
/// same call site with a zero-sized guard.
#[cfg_attr(not(feature = "test-support"), allow(dead_code))]
struct WorkerMark<'a> {
    faults: &'a Faults,
}

impl Drop for WorkerMark<'_> {
    fn drop(&mut self) {
        #[cfg(feature = "test-support")]
        if let Some(watch) = self.faults.worker_watch.as_ref() {
            watch.exit();
        }
    }
}

// ---------------------------------------------------------------------------------------------
// One class task: prepare once, execute every method, hand each record over as it exists
// ---------------------------------------------------------------------------------------------

/// Runs one class candidate.
///
/// Both configurations run exactly this function: the serial path calls it on the calling thread
/// with a consumer that delivers straight to the sink, and a worker calls it with a consumer that
/// hands each record to the class's own slot and waits for the coordinator to take it. Everything
/// the class task decides — one verified read, one preparation, one analysis and presentation per
/// method, the per-method weight and every count — is therefore the same in both.
fn run_class_task(
    operation: &Operation<'_>,
    class: &ScopeClass,
    class_ordinal: u64,
    budget: &mut Budget,
    out: &mut dyn ResultConsumer,
) -> ClassEnding {
    if operation.faults.panics_at(class_ordinal) {
        // A test-only fault: the class task panics where a real one would if something inside it
        // did, so the lifecycle around a panicking worker is exercised rather than described.
        panic!("bulk test fault: the class task for ordinal {class_ordinal} panics");
    }
    operation.faults.arrive_at_gate();
    operation.faults.delay_before_prepare();
    let _entered = operation.registry.enter();
    let mut methods = 0_u64;
    let mut class_execution: Option<ExecutionReport> = None;
    let read = match read_class(operation, class, budget) {
        Ok(read) => read,
        Err(error) => return class_failure(operation, error, budget, methods, class_execution),
    };
    let prepared = match PreparedClass::prepare(&read, budget) {
        Ok(prepared) => prepared,
        Err(error) => return class_failure(operation, error, budget, methods, class_execution),
    };
    let declared = prepared.method_count();
    let definition = definition_of(&read);
    // The class is prepared: it is counted here, before its control record is handed over, because
    // the count states the work that ran rather than the records that reached the sink.
    operation.registry.note_prepared(declared);
    let control = ClassPreparedEvent {
        class_ordinal,
        location: prepared.location().clone(),
        class_bytes: prepared.class_bytes().clone(),
        container: read.container.clone(),
        depth: read.depth,
        declared_methods: declared,
        member_table_stop: prepared.member_table_stop().cloned(),
    };
    match out.prepared(control) {
        Ok(true) => {}
        Ok(false) => return stopped_class(methods, None, class_execution),
        Err(closing) => return stopped_class(methods, Some(closing), class_execution),
    }
    for slot in prepared.method_slots() {
        let ordinal = slot.ordinal;
        let mut method_budget =
            operation.budget_for(operation.limits.method.clone(), UsageOwner::Methods);
        let run = match execute_method(
            operation,
            &prepared,
            &definition,
            ordinal,
            &mut method_budget,
        ) {
            Ok(run) => run,
            Err(closing) => return stopped_class(methods, Some(closing), class_execution),
        };
        let event = MethodResultEvent {
            class_ordinal,
            member_ordinal: u64::from(ordinal.0),
            method: method_identity(&definition, slot),
            delivery: run.delivery,
            weight: run.weight,
        };
        merge_class_execution(&mut class_execution, &event);
        operation.registry.note_method(&event);
        methods = methods.saturating_add(1);
        match out.method(event) {
            Ok(true) => {}
            Ok(false) => return stopped_class(methods, None, class_execution),
            Err(closing) => return stopped_class(methods, Some(closing), class_execution),
        }
    }
    ClassEnding::Prepared {
        methods,
        execution: class_execution,
    }
}

/// One class task that stopped before it finished: the records it already handed over stand, and
/// the members after them are unknown.
fn stopped_class(
    methods: u64,
    closing: Option<Closing>,
    execution: Option<ExecutionReport>,
) -> ClassEnding {
    ClassEnding::Closed {
        methods,
        closing,
        execution,
    }
}

/// One class-level failure: that class's refusal when the operation itself is fine, and the
/// operation's own end when it is not.
fn class_failure(
    operation: &Operation<'_>,
    error: Error,
    budget: &Budget,
    methods: u64,
    execution: Option<ExecutionReport>,
) -> ClassEnding {
    if operation.stopped() {
        return stopped_class(methods, Some(Closing::Stopped(error)), execution);
    }
    ClassEnding::Refused(ClassRefusal {
        code: crate::facade::error_code(&error),
        message: error.to_string(),
        execution: crate::facade::stop_execution(&error, budget),
    })
}

/// The one trusted read of one class candidate: the entry's verified bytes, or the snapshot's own
/// root when the candidate is a standalone CLASS.
///
/// # The preparation ceiling is enforced *while* the bytes are read
///
/// The read runs under a budget of its own whose `entry_bytes` and `class_bytes` limits are this
/// operation's [`BulkRecoveryRequest::max_class_bytes`] (when that is the tighter of the two), and it
/// bills to the same ledger as every other read of the operation. That is what makes the ceiling a
/// ceiling on **materialization** rather than on preparation alone: the entry's own trusted length —
/// the uncompressed size of the container's directory record, verified against the entry's local
/// header — is checked before the bytes are copied out, and a deflated entry is charged chunk by chunk
/// as it inflates. A class over the ceiling is therefore refused **before** it is read into memory,
/// and an entry that inflates past the length its record states cannot grow past the ceiling either.
///
/// The refusal is a *class-level* one: it is answered as [`BulkRecoveryRequest::max_class_bytes`]'
/// own refusal, the operation continues with the classes beside it, and the ceiling's dimension
/// (`entry_bytes`/`class_bytes`) is the one the caller declared in the entry budget.
fn read_class(
    operation: &Operation<'_>,
    class: &ScopeClass,
    budget: &mut Budget,
) -> Result<PreparedClassRead> {
    budget.set_owner(UsageOwner::Discovery);
    let mut limits = budget.limits().clone();
    limits.entry_bytes = limits.entry_bytes.min(operation.limits.max_class_bytes);
    limits.class_bytes = limits.class_bytes.min(operation.limits.max_class_bytes);
    let mut read = operation.budget_for(limits, UsageOwner::Discovery);
    let read = match &class.entry {
        Some(entry) => operation.snapshot.prepared_class(entry, &mut read),
        None => operation.snapshot.prepared_root_class(&mut read),
    };
    match read {
        Ok(read) => Ok(read),
        Err(error) => Err(ceiling_refusal(operation, &error).unwrap_or(error)),
    }
}

/// The operation's own preparation-ceiling refusal, when a read failed because of it.
///
/// A read that needs more than the ceiling is answered with
/// [`BulkRecoveryRequest::max_class_bytes`]' own error rather than the budget error of the local
/// limit that carried it: the caller declared a ceiling on one class, and "this class is larger than
/// the ceiling" is the fact a consumer of that record can act on. `None` leaves the read's own error
/// as it is — a damaged entry, a missing directory and an exhausted *quota* all keep their own codes.
fn ceiling_refusal(operation: &Operation<'_>, error: &Error) -> Option<Error> {
    let ceiling = operation.limits.max_class_bytes;
    match error {
        Error::BudgetExceeded {
            dimension: BudgetDimension::EntryBytes | BudgetDimension::ClassBytes,
            limit,
            requested,
            ..
        } if *limit == ceiling && *requested > ceiling => Some(Error::invalid_input(
            "bulk_class_too_large",
            format!(
                "the class entry needs {requested} byte(s) and this operation prepares at most \
                 {ceiling}: the class is refused before it is read into memory, and the methods it \
                 declares stay unknown rather than being counted or silently truncated"
            ),
        )),
        _ => None,
    }
}

/// The physical definition one prepared read is: its location, the trusted identity of its bytes and
/// the syntactic variant its own raw path derives — the same derivation every other read of the same
/// entry states, so the identity a method request names is the one a lookup would decide.
fn definition_of(read: &PreparedClassRead) -> PhysicalDefinitionId {
    let variant = match read.location.entry() {
        Some(entry) => physical_variant_for_path(&entry.raw_name.0),
        None => PhysicalVariant::Base,
    };
    PhysicalDefinitionId {
        location: read.location.clone(),
        class_bytes: read.class_bytes.clone(),
        variant,
    }
}

/// One member record's physical identity: its owner definition plus the raw name and descriptor the
/// preparation read.
fn method_identity(
    definition: &PhysicalDefinitionId,
    slot: &jarde_reader::prepared::MethodSlot,
) -> PhysicalMethodId {
    PhysicalMethodId {
        owner: definition.clone(),
        name: JvmBytes(slot.name.raw().0.clone()),
        descriptor: JvmBytes(slot.descriptor.raw().0.clone()),
    }
}

/// One method's record and the weight it asks the window for.
struct MethodRun {
    delivery: MethodDelivery,
    /// The retained weight of the record: zero for every record that retains no artifact, because a
    /// record the window could not hold may not be replaced by a smaller one that lies.
    weight: u64,
}

/// Executes one method of a prepared class and turns its run into a record.
///
/// The analysis is [`jarde_jvm::analyze_prepared_method_ir`] — the prepared half of the same
/// pipeline the single-method entry runs — and the presentation is
/// [`crate::facade::recovery_presented`], the same one [`crate::Engine::recover_method`] uses, so a
/// method recovered here and the same method recovered there cannot drift.
fn execute_method(
    operation: &Operation<'_>,
    prepared: &PreparedClass<'_>,
    definition: &PhysicalDefinitionId,
    ordinal: MethodOrdinal,
    budget: &mut Budget,
) -> std::result::Result<MethodRun, Closing> {
    budget.set_owner(UsageOwner::Methods);
    let Some(slot) = prepared.slot(ordinal) else {
        let error = Error::invalid_input(
            "classfile_method_not_found",
            format!(
                "this class declares no method record at ordinal {}",
                ordinal.0
            ),
        );
        return local_or_closing(operation, error, budget, definition);
    };
    let request = MethodAnalysisRequest {
        environment: operation.environment.clone(),
        method: method_identity(definition, slot),
        stages: operation.stages.to_vec(),
    };
    let analyzed = match jarde_jvm::analyze_prepared_method_ir(
        operation.content,
        prepared,
        &request,
        budget,
    ) {
        Ok(analyzed) => analyzed,
        Err(error) => return local_or_closing(operation, error, budget, definition),
    };
    if let MethodBodyState::DeclaredWithoutBody { .. } = analyzed.report().body {
        // A declaration that states it has no body is an answer, not a failure: no phase ran, so
        // there is no artifact to weigh and none to deliver.
        return Ok(MethodRun {
            delivery: MethodDelivery::NoBody {
                analysis: Box::new(analyzed.report().clone()),
            },
            weight: 0,
        });
    }
    match crate::facade::recovery_presented(
        operation.content,
        &request,
        analyzed,
        Some(prepared),
        operation.evidence,
        budget,
    ) {
        Ok(recovered) => {
            let weight = result_weight(&recovered);
            if weight > operation.limits.max_result_weight {
                // The result does not fit the fixed per-result ceiling. It is discarded rather than
                // truncated: the record states the weight, the ceiling and the codes, and never a
                // text that claims to be the artifact.
                let execution = ExecutionReport::Failed {
                    reason: TerminationReason::Error {
                        code: "bulk_result_weight_exceeds_limit".to_owned(),
                    },
                    usage: budget.usage(),
                };
                let diagnostic_codes = recovered
                    .recovery()
                    .diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.code.clone())
                    .collect();
                return Ok(MethodRun {
                    delivery: MethodDelivery::Oversized {
                        weight,
                        limit: operation.limits.max_result_weight,
                        execution,
                        diagnostic_codes,
                    },
                    weight: 0,
                });
            }
            Ok(MethodRun {
                delivery: MethodDelivery::Recovered(Box::new(recovered)),
                weight,
            })
        }
        Err(error) => local_or_closing(operation, error, budget, definition),
    }
}

/// A method-level failure: that method's own refusal when the operation is fine, and the operation's
/// end when it is not.
///
/// The distinction is the ledger's own: a *local* limit refusal records nothing (it stops the work
/// it refused), while a refusal of the operation's total, a cancellation or an expired deadline is
/// recorded and cancels the operation's token.
fn local_or_closing(
    operation: &Operation<'_>,
    error: Error,
    budget: &Budget,
    definition: &PhysicalDefinitionId,
) -> std::result::Result<MethodRun, Closing> {
    if operation.stopped() {
        return Err(Closing::Stopped(error));
    }
    let execution = crate::facade::stop_execution(&error, budget);
    let diagnostic = crate::facade::stop_diagnostic(
        &error,
        Some(crate::facade::definition_provenance(definition)),
    );
    Ok(MethodRun {
        delivery: MethodDelivery::Refused {
            execution,
            diagnostics: vec![diagnostic],
        },
        weight: 0,
    })
}

/// The retained weight of one method result, in bytes: what the result **owns**.
///
/// # The model (design decision 5)
///
/// ```text
/// weight(result) =
///     size_of::<RecoveredMethod>()                             the held value's own frame
///   + text.capacity()                                          the artifact's text buffer
///   + source_map.segments().len() * size_of::<Segment>()        the segment table
///   + the recovery report:
///       method.capacity() + Σ aliased_names[i].capacity()       the strings it owns
///     + Σ diagnostics[i].{code,message}.capacity()              the diagnostics it owns
///     + capacities of rules/regions/lambdas/concats/accessors/bridges/news/fields/enum_switches/
///       fallbacks, each * size_of::<Element>()                  one allocation per table
///     + the strings of every LambdaRecord (and of its captures)
///   + the analysis report: the same two rules over
///       environment_problems (with their messages), requested_stages, stages, reads, diagnostics,
///       the member identity (its owner digest, its raw name and descriptor) and its origin set
///   + the callee read: its class name, its members' identities and its refusal table
///   + the facts: the declaration's own name and descriptor (by length: the surface hands them over
///       as `&str`), and the debug-local table's own buffer
/// ```
///
/// Every owned buffer is counted by its **capacity**, not by the length it currently uses: a `Vec` or
/// `String` this operation holds owns its whole allocation, and an allocator that grew it to twice
/// the length it needed is holding twice the memory the length suggests. Tables the surface hands over
/// as **slices** (`SourceMap::segments`, the callee read's members and refusals, the facts' debug
/// locals) are counted by length, because a slice carries no capacity — the number is then a lower
/// bound of that table's own allocation, and the tables this module owns itself are the ones counted
/// exactly.
///
/// # What this model is not
///
/// * **It is not RSS and not an allocator's view.** It counts the bytes this operation's own
///   structures own, with no allocator header, no fragmentation, no rounding and no copy an upper
///   layer made of a fact it read here. A process's real footprint is a different measurement.
/// * **No backing is shared inside a result.** Nothing a result holds is an `Arc`: the values above
///   are plain owned data, so there is no backing to deduplicate *within* one result's weight. What
///   *is* shared across the operation — the class's bytes and the container facts — belongs to the
///   preparation that read it and is charged there; counting it again per result would be exactly the
///   duplication the design's deduplication rule forbids.
/// * **Record elements are walked for the types this facade exposes** (`LambdaRecord` with its
///   captures, both reports' `Diagnostic`s, the analysis report's `EnvironmentProblem`s, the member
///   identities) and **not beyond that**: the strings *inside* a `RegionRecord`, a `ConcatRecord`, an
///   `AccessorRecord`, a `BridgeRecord`, a `NewRecord`, a `FieldRecord`, an `EnumSwitchRecord`, an
///   `InitRecord` or a `DeclarationRecord` are outside this model, because those element types are not
///   part of this facade's surface (only their tables' own allocations are counted). The same holds
///   for the facts' `DebugLocal` fields and for anything an `EnvironmentIdentity` or an `OriginSet`
///   holds beyond the parts listed above. A caller that needs a hard bound must therefore read this
///   figure as "at least what is counted here", which is why the per-item ceiling that refuses a
///   result is stated separately as [`BulkRecoveryRequest::max_result_weight`].
fn result_weight(recovered: &RecoveredMethod) -> u64 {
    let recovery = recovered.recovery();
    let analysis = recovered.analysis();
    let facts = recovered.facts();
    let mut weight = Weight::default();

    // The value itself, and the artifact it carries.
    weight.frame::<RecoveredMethod>();
    weight.text(&recovery.text);
    weight.slice(recovery.source_map.segments());

    // The recovery report's own strings and tables.
    weight.text(&recovery.method);
    weight.strings(&recovery.aliased_names);
    weight.diagnostics(&recovery.diagnostics);
    weight.buffer(&recovery.rules);
    weight.buffer(&recovery.regions);
    weight.buffer(&recovery.lambdas);
    weight.buffer(&recovery.concats);
    weight.buffer(&recovery.accessors);
    weight.buffer(&recovery.bridges);
    weight.buffer(&recovery.news);
    weight.buffer(&recovery.fields);
    weight.buffer(&recovery.enum_switches);
    weight.buffer(&recovery.fallbacks);
    for lambda in &recovery.lambdas {
        weight.optional_text(lambda.bootstrap.as_ref());
        weight.text(&lambda.sam_name);
        weight.text(&lambda.sam_descriptor);
        weight.optional_text(lambda.sam_method_type.as_ref());
        weight.optional_text(lambda.instantiated_method_type.as_ref());
        weight.optional_text(lambda.implementation.as_ref());
        weight.buffer(&lambda.captures);
    }

    // The analysis report: its own tables, the member it is about, and the origins it names.
    weight.identity(&analysis.method);
    weight.origin_set(&analysis.origin);
    weight.buffer(&analysis.environment_problems);
    for problem in &analysis.environment_problems {
        weight.text(&problem.message);
    }
    weight.buffer(&analysis.requested_stages);
    weight.buffer(&analysis.stages);
    weight.buffer(&analysis.reads);
    weight.diagnostics(&analysis.diagnostics);

    // The callee read, when this request made one.
    if let Some(read) = recovered.callees() {
        weight.borrowed(read.class());
        weight.slice(read.members());
        weight.slice(read.refusals());
        for member in read.members() {
            weight.identity(member.identity());
        }
    }

    // The facts the run was presented from: the declaration's own strings, and its debug-local table.
    weight.borrowed(facts.method().name());
    weight.borrowed(facts.method().descriptor());
    weight.slice(facts.debug_locals());

    weight.0
}

/// The accumulator one result's retained weight is summed in.
///
/// Every term is a **capacity** wherever the surface hands over a buffer this operation owns, and a
/// length wherever it hands over a slice (which owns nothing). All arithmetic saturates: a weight is a
/// bound used to refuse work, so it must never wrap into a small number and admit a result that does
/// not fit.
#[derive(Clone, Copy, Default)]
struct Weight(u64);

impl Weight {
    /// One value's own frame, without whatever it owns behind a pointer.
    fn frame<T>(&mut self) {
        self.value(u64::try_from(std::mem::size_of::<T>()).unwrap_or(u64::MAX));
    }

    /// One owned table's own allocation: `capacity` elements of `T`.
    fn buffer<T>(&mut self, values: &Vec<T>) {
        let elements = u64::try_from(values.capacity()).unwrap_or(u64::MAX);
        let each = u64::try_from(std::mem::size_of::<T>()).unwrap_or(u64::MAX);
        self.value(elements.saturating_mul(each));
    }

    /// One table the surface hands over as a slice: it owns nothing itself, so its length is the
    /// closest reading of the allocation behind it.
    fn slice<T>(&mut self, values: &[T]) {
        let elements = u64::try_from(values.len()).unwrap_or(u64::MAX);
        let each = u64::try_from(std::mem::size_of::<T>()).unwrap_or(u64::MAX);
        self.value(elements.saturating_mul(each));
    }

    /// One owned string's own allocation.
    fn text(&mut self, value: &String) {
        self.value(u64::try_from(value.capacity()).unwrap_or(u64::MAX));
    }

    /// One owned string that is not held, when it is held at all.
    fn optional_text(&mut self, value: Option<&String>) {
        if let Some(value) = value {
            self.text(value);
        }
    }

    /// One string the surface hands over as a borrowed `str`: a `&str` carries no capacity, so its
    /// length is all it states.
    fn borrowed(&mut self, value: &str) {
        self.value(u64::try_from(value.len()).unwrap_or(u64::MAX));
    }

    /// One owned list of strings: its own allocation, and the allocation of every string in it.
    fn strings(&mut self, values: &Vec<String>) {
        self.buffer(values);
        for value in values {
            self.text(value);
        }
    }

    /// One owned list of diagnostics: its own allocation, and the two strings each one owns.
    fn diagnostics(&mut self, values: &Vec<Diagnostic>) {
        self.buffer(values);
        for diagnostic in values {
            self.text(&diagnostic.code);
            self.text(&diagnostic.message);
        }
    }

    /// One member identity: the strings its definition carries and the raw name and descriptor bytes.
    fn identity(&mut self, identity: &PhysicalMethodId) {
        self.text(&identity.owner.class_bytes.digest.0);
        if let Some(entry) = identity.owner.location.entry() {
            self.buffer(&entry.raw_name.0);
        }
        self.buffer(&identity.name.0);
        self.buffer(&identity.descriptor.0);
    }

    /// One origin set: its own frame and the members it holds.
    fn origin_set(&mut self, origins: &jarde_reader::model::OriginSet) {
        self.frame::<jarde_reader::model::OriginSet>();
        self.buffer(&origins.members);
    }

    /// Adds one already counted number of bytes.
    fn value(&mut self, bytes: u64) {
        self.0 = self.0.saturating_add(bytes);
    }
}

// ---------------------------------------------------------------------------------------------
// Delivery: the coordinator's own half, and the two ways a class task is run
// ---------------------------------------------------------------------------------------------

/// One record the coordinator hands to the sink.
enum Record<'a> {
    Header(&'a BulkHeaderEvent),
    Prepared(&'a ClassPreparedEvent),
    Method(&'a MethodResultEvent),
    ClassEnd(&'a ClassEndEvent),
    Diagnostic(&'a BulkDiagnosticEvent),
    Final(&'a BulkFinalEvent),
}

/// The delivery side of one operation: the sink, the entry budget and the registry.
///
/// Only the coordinator holds one, which is the whole reason a sink implementation needs no `Send`
/// and no `Sync`: every callback of a stream runs on the thread that called
/// [`crate::Engine::recover_all`].
struct Delivery<'a> {
    sink: &'a mut dyn RecoverySink,
    budget: &'a mut Budget,
    /// The operation's one output account, handed to the sink with the header: the consumer's own
    /// bytes are charged to the same total every read and every method bills to.
    account: DeliveryAccount,
    /// The consumer refused (`Stop`) or failed: nothing further may be handed to it, including the
    /// records that would have stated why. What it confirmed stands.
    closed: bool,
    /// The operation's observation port, empty unless its caller attached a probe.
    observation: Observation,
}

impl Delivery<'_> {
    /// Hands one record to the sink, or answers why it was not handed over.
    ///
    /// The charge comes first and it is the operation's own: a record the total cannot pay for is
    /// not published, and the `Final` event is no exception — a stream is not completed by a record
    /// the quota refused, and a charge refusal is a stop like any other.
    ///
    /// The sink's callback is called with the operation's output account, and the call happens
    /// without the ledger's lock: taking a permit is the sink's own decision inside the callback.
    fn deliver(&mut self, record: Record<'_>) -> std::result::Result<SinkControl, Closing> {
        let started = self.observation.delivery_started();
        self.budget.set_owner(UsageOwner::Delivery);
        self.budget
            .charge(CountedBudgetDimension::ResultItems, 1)
            .map_err(Closing::Stopped)?;
        let in_sink = started.map(|_| std::time::Instant::now());
        let control = match record {
            Record::Header(event) => {
                let account = self.account.clone();
                self.sink.header(event, account)
            }
            Record::Prepared(event) => self.sink.class_prepared(event),
            Record::Method(event) => self.sink.method(event),
            Record::ClassEnd(event) => self.sink.class_end(event),
            Record::Diagnostic(event) => self.sink.diagnostic(event),
            Record::Final(event) => self.sink.final_event(event),
        }
        .map_err(Closing::Consumer)?;
        self.observation.delivered(started, in_sink);
        if control == SinkControl::Stop {
            self.closed = true;
        }
        Ok(control)
    }
}

/// Publishes one record, recording the stop when the consumer (or the operation) stopped the stream.
///
/// The answer is whether the operation may go on: a `Stop`, a refusal or a sink failure all end it.
fn publish(
    operation: &Operation<'_>,
    delivery: &mut Delivery<'_>,
    record: Record<'_>,
    delivered_method: Option<&MethodResultEvent>,
) -> bool {
    match delivery.deliver(record) {
        Ok(SinkControl::Continue) => {
            if delivered_method.is_some() {
                operation.registry.note_delivered();
            }
            true
        }
        Ok(SinkControl::Stop) => {
            operation.ledger.cancel(BulkStopKind::Sink);
            operation.registry.close(Ending::Stopped);
            false
        }
        Err(closing) => {
            // A consumer that failed is not asked again, whatever the rest of the stream would have
            // said: the failure itself is the operation's end.
            delivery.closed = true;
            operation.registry.close(closing_ending(&closing));
            false
        }
    }
}

/// The ending one closing leaves on the operation.
fn closing_ending(closing: &Closing) -> Ending {
    match closing {
        Closing::Consumer(_) => Ending::Infrastructure {
            code: closing.code(),
            message: closing.error().to_string(),
        },
        Closing::Stopped(_) => Ending::Stopped,
    }
}

/// The serial path's consumer: every record goes straight to the sink, where it was produced.
struct DirectConsumer<'a, 'o, 'd> {
    operation: &'a Operation<'o>,
    delivery: &'a mut Delivery<'d>,
}

impl ResultConsumer for DirectConsumer<'_, '_, '_> {
    fn prepared(&mut self, event: ClassPreparedEvent) -> std::result::Result<bool, Closing> {
        Ok(publish(
            self.operation,
            self.delivery,
            Record::Prepared(&event),
            None,
        ))
    }

    fn method(&mut self, event: MethodResultEvent) -> std::result::Result<bool, Closing> {
        self.operation.registry.note_direct_result(event.weight);
        Ok(publish(
            self.operation,
            self.delivery,
            Record::Method(&event),
            Some(&event),
        ))
    }
}

/// One worker's handover into its own class's slot.
struct SlotConsumer<'a> {
    registry: &'a Registry,
    index: u64,
}

impl ResultConsumer for SlotConsumer<'_> {
    fn prepared(&mut self, event: ClassPreparedEvent) -> std::result::Result<bool, Closing> {
        self.registry.place_control(self.index, event)
    }

    fn method(&mut self, event: MethodResultEvent) -> std::result::Result<bool, Closing> {
        self.registry.place_method(self.index, event)
    }
}

/// What the coordinator found at the front of the window.
enum Front {
    /// One control record to deliver.
    Control(Box<ClassPreparedEvent>),
    /// One method record to deliver.
    Method(Box<MethodResultEvent>),
    /// The front class task ended, and its `ClassEnd` is what the coordinator publishes next.
    Ended(Box<EndedClass>),
    /// The window is empty.
    Empty,
    /// The operation was already closing.
    Closed,
}

// ---------------------------------------------------------------------------------------------
// The coordinator
// ---------------------------------------------------------------------------------------------

/// How the classes of one operation are executed.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Dispatch {
    /// The coordinator runs each class task itself, on the calling thread.
    Serial,
    /// A worker takes each class task; the coordinator dispatches and delivers.
    Workers,
}

/// The end of one class's part of the stream, as the coordinator reads it back from the window.
struct EndedClass {
    /// The ordinal the class delivers under.
    ordinal: u64,
    ending: ClassEnding,
    class: ScopeClass,
}

/// What one operation's traversal and stream reached.
struct Stream {
    /// Discovery reached the end of the scope and every container it holds was walked.
    traversal_complete: bool,
}

/// Takes the next record of the earliest active class, waiting — bounded — for one to exist.
///
/// The wait observes both the operation's close signal and the caller's own cancellation token, and
/// it never holds the ledger's lock: a caller that cancels mid-operation is noticed by the token
/// this function reads directly, and the ledger records the stop at the next checkpoint that takes
/// its lock.
fn take_front(registry: &Registry, budget: &Budget) -> Front {
    registry.observation.window_call(WindowSite::TakeFront);
    let mut state = registry.lock();
    loop {
        if state.ending.is_some() {
            return Front::Closed;
        }
        // The front slot's control record is delivered before its method record, and taking one
        // never removes the other: a worker that produced its result while the coordinator had not
        // yet taken the class's `ClassPrepared` record has both pending, and the method record must
        // survive that delivery.
        let mut taken_weight = 0_u64;
        let taken = match state.slots.front_mut() {
            None => None,
            Some(slot) => match slot.control.take() {
                Some(control) => Some(Front::Control(Box::new(control))),
                None => match slot.method.take() {
                    Some(method) => {
                        taken_weight = method.weight;
                        Some(Front::Method(method))
                    }
                    None => None,
                },
            },
        };
        if taken_weight > 0 {
            state.retained_weight = state.retained_weight.saturating_sub(taken_weight);
        }
        if let Some(front) = taken {
            registry.changed.notify_all();
            return front;
        }
        let ended = state.slots.front_mut().and_then(|slot| {
            slot.ending
                .take()
                .map(|ending| (ending, slot.class.clone()))
        });
        if let Some((ending, class)) = ended {
            let ordinal = state.front;
            state.slots.pop_front();
            state.front = state.front.saturating_add(1);
            registry.changed.notify_all();
            return Front::Ended(Box::new(EndedClass {
                ordinal,
                ending,
                class,
            }));
        }
        if budget.cancellation_token().is_cancelled() {
            return Front::Closed;
        }
        if state.traversal_done && state.slots.is_empty() {
            return Front::Empty;
        }
        state = registry.wait_signal(state, WindowSite::TakeFront, WAIT_SLICE);
    }
}

/// The stop one operation reports: the first stop the ledger observed, or the operation's own
/// infrastructure ending when the failure never reached a charge.
///
/// A worker that could not be created or did not return stops the operation through its own close
/// signal rather than through the ledger's token — the charges of the classes still winding down stay
/// admitted, so the records the library owes for them are still published — and this is where that
/// ending becomes the report's `stop` record, exactly as a ledger stop would be.
fn stop_record(operation: &Operation<'_>, ending: Option<&Ending>) -> Option<BulkStop> {
    operation.ledger.stop_reason().or(match ending {
        Some(Ending::Infrastructure { .. }) => Some(BulkStop {
            owner: None,
            kind: BulkStopKind::Infrastructure,
            dimension: None,
        }),
        _ => None,
    })
}

/// The code a class's `Stopped` completion states: what the class task's own site saw, or what the
/// operation's own ending and the ledger recorded for it.
fn stop_code(operation: &Operation<'_>, closing: Option<&Closing>) -> String {
    if let Some(closing) = closing {
        return closing.code();
    }
    // The operation's own ending is what a class that never reached a closing of its own states.
    if let Some(Ending::Infrastructure { code, .. }) = operation.registry.ending().as_ref() {
        return code.clone();
    }
    match operation.ledger.stop_reason().map(|stop| stop.kind) {
        Some(BulkStopKind::Budget) => "bulk_budget_exhausted".to_owned(),
        Some(BulkStopKind::Cancelled) => "bulk_operation_cancelled".to_owned(),
        Some(BulkStopKind::Sink) => "bulk_sink_stopped".to_owned(),
        Some(BulkStopKind::Infrastructure) => "bulk_infrastructure_failed".to_owned(),
        None => "bulk_operation_stopping".to_owned(),
    }
}

/// The physical position of one class candidate, for a diagnostic that has no definition to point
/// at: the read that would have established one is exactly what failed.
fn class_provenance(class: &ScopeClass) -> Option<Provenance> {
    let entry = class.entry.as_ref()?;
    Some(crate::facade::entry_provenance(entry, 0))
}

/// Publishes the end of one class: the located diagnostic of a refusal, and the class's `ClassEnd`.
///
/// The class's own execution plane is the strongest state its delivered method records reached —
/// `Complete` for a class whose methods all completed, whatever another class of the same operation
/// did — restated under the operation's usage.
fn publish_class_end(
    operation: &Operation<'_>,
    delivery: &mut Delivery<'_>,
    class_ordinal: u64,
    class: &ScopeClass,
    ending: ClassEnding,
) -> bool {
    let location = class.location.clone();
    let mut executions: Vec<ExecutionReport> = Vec::new();
    let completion = match ending {
        ClassEnding::Prepared { methods, execution } => {
            executions.extend(execution);
            ClassCompletion::Completed { methods }
        }
        ClassEnding::Refused(refusal) => {
            operation.registry.note_refused();
            let diagnostic = Diagnostic {
                code: refusal.code.clone(),
                severity: DiagnosticSeverity::Error,
                message: refusal.message,
                provenance: class_provenance(class),
            };
            let event = BulkDiagnosticEvent {
                class_ordinal: Some(class_ordinal),
                diagnostic,
            };
            if !publish(operation, delivery, Record::Diagnostic(&event), None) {
                return false;
            }
            executions.push(refusal.execution);
            ClassCompletion::Refused { code: refusal.code }
        }
        ClassEnding::Closed {
            methods,
            closing,
            execution,
        } => {
            executions.extend(execution);
            ClassCompletion::Stopped {
                methods,
                code: stop_code(operation, closing.as_ref()),
            }
        }
    };
    let mut class_execution: Option<ExecutionReport> = None;
    for incoming in executions {
        match &mut class_execution {
            Some(current) => crate::facade::merge_execution(current, incoming),
            None => class_execution = Some(incoming),
        }
    }
    let execution = class_execution.unwrap_or(ExecutionReport::Complete {
        usage: delivery.budget.usage(),
    });
    let event = ClassEndEvent {
        class_ordinal,
        location,
        completion,
        execution: jarde_reader::accounting::with_usage(execution, delivery.budget.usage()),
    };
    publish(operation, delivery, Record::ClassEnd(&event), None)
}

/// Publishes any diagnostic the traversal recorded since it was last read.
fn publish_traversal_diagnostics(
    operation: &Operation<'_>,
    delivery: &mut Delivery<'_>,
    cursor: &jarde_reader::scope_cursor::ScopeCursor,
    published: &mut usize,
) -> bool {
    let diagnostics = cursor.diagnostics();
    while *published < diagnostics.len() {
        let event = BulkDiagnosticEvent {
            class_ordinal: None,
            diagnostic: diagnostics[*published].clone(),
        };
        *published = published.saturating_add(1);
        if !publish(operation, delivery, Record::Diagnostic(&event), None) {
            return false;
        }
    }
    true
}

/// Runs the traversal and the stream: the one loop both configurations share.
///
/// The classes are pulled from the scope's own incremental cursor, one at a time, and delivered in
/// the order the cursor yields them. In the serial configuration the coordinator runs each class
/// task where it stands; in the worker configuration it dispatches the task into the window and
/// delivers from the earliest active class until that class ends, then extends the window again. No
/// step of either configuration collects a package-wide class list, a class header batch or a method
/// batch: the only state that grows is the window itself, which is bounded by
/// [`BulkLimits::workers_effective`].
fn coordinate(
    operation: &Operation<'_>,
    cursor: &mut jarde_reader::scope_cursor::ScopeCursor,
    delivery: &mut Delivery<'_>,
    dispatch: Dispatch,
) -> Stream {
    let window = operation.limits.workers_effective as u64;
    let mut diagnostics_published = 0_usize;
    let mut traversal_done = false;

    loop {
        // The checkpoint before anything is dispatched: a cancellation, a stop the ledger recorded
        // or a consumer that refused ends the operation here, and closing the registry is what wakes
        // every worker waiting for work or for a slot.
        if let Some(ending) = stop_ending(operation, delivery) {
            operation.registry.close(ending);
            break;
        }
        match dispatch {
            Dispatch::Serial => {
                // The coordinator is the class task: it pulls one class, runs it where it stands —
                // every record reaching the sink as it is produced — and goes on to the next.
                let mut discovery =
                    operation.budget_for(operation.discovery_limits.clone(), UsageOwner::Discovery);
                let class = match next_class(
                    operation,
                    cursor,
                    &mut discovery,
                    delivery,
                    &mut diagnostics_published,
                ) {
                    Next::Class(class) => *class,
                    Next::Done => {
                        traversal_done = true;
                        operation.registry.note_traversal_done();
                        break;
                    }
                    Next::Stopped => break,
                };
                let ordinal = operation.registry.next_ordinal();
                let mut consumer = DirectConsumer {
                    operation,
                    delivery,
                };
                let started = operation.observation.class_task_started();
                let ending =
                    run_class_task(operation, &class, ordinal, &mut discovery, &mut consumer);
                operation.observation.class_task_ended(started);
                if !publish_class_end(operation, delivery, ordinal, &class, ending) {
                    break;
                }
            }
            Dispatch::Workers => {
                // Extend the window while it has room, then deliver from the earliest active class.
                // Filling never blocks on a worker: a full window is consumed before more is
                // discovered, which is what keeps the traversal's memory bounded.
                let mut stop = false;
                while !stop {
                    let open = operation.registry.lock().slots.len() as u64;
                    if open >= window {
                        break;
                    }
                    let mut discovery = operation
                        .budget_for(operation.discovery_limits.clone(), UsageOwner::Discovery);
                    match next_class(
                        operation,
                        cursor,
                        &mut discovery,
                        delivery,
                        &mut diagnostics_published,
                    ) {
                        Next::Class(class) => {
                            // The walk's own container travels with the candidate it just yielded.
                            // The walk is inside this container now and will have left it by the
                            // time the task reads the class; the handle is what keeps the product
                            // that read needs alive until then.
                            let container = cursor.container_facts();
                            operation.registry.dispatch_class(*class, container);
                        }
                        Next::Done => {
                            traversal_done = true;
                            operation.registry.note_traversal_done();
                            stop = true;
                        }
                        Next::Stopped => stop = true,
                    }
                }
                if let Some(ending) = stop_ending(operation, delivery) {
                    operation.registry.close(ending);
                    break;
                }
                match take_front(operation.registry, delivery.budget) {
                    Front::Closed => break,
                    // A window with no active class and an unfinished traversal cannot happen: the
                    // fill above either dispatched a class or found the end of the scope. It is
                    // re-checked rather than assumed.
                    Front::Empty => {
                        if traversal_done {
                            break;
                        }
                        continue;
                    }
                    Front::Control(control) => {
                        if !publish(operation, delivery, Record::Prepared(&control), None) {
                            break;
                        }
                    }
                    Front::Method(method) => {
                        if !publish(operation, delivery, Record::Method(&method), Some(&method)) {
                            break;
                        }
                    }
                    Front::Ended(ended) => {
                        let EndedClass {
                            ordinal,
                            ending,
                            class,
                        } = *ended;
                        if !publish_class_end(operation, delivery, ordinal, &class, ending) {
                            break;
                        }
                    }
                }
            }
        }
    }

    let traversal_complete =
        traversal_done && cursor.coverage_state() == CoverageState::CompleteWithinSchema;
    // A class whose task already ended states where it stopped, even though the stream is closing:
    // that record is what makes "this class did not finish" readable rather than silent. A consumer
    // that stopped or failed is not asked again.
    let _ = drain_ended_classes(operation, delivery);
    // The traversal's own diagnostics are published as long as the consumer still takes records:
    // they are what makes a short denominator readable. A consumer that refused is not asked again.
    if !delivery.closed {
        let _ =
            publish_traversal_diagnostics(operation, delivery, cursor, &mut diagnostics_published);
    }
    Stream { traversal_complete }
}

/// Publishes the `ClassEnd` of every class whose task already ended, in delivery order.
///
/// A stream that closes before the coordinator reached a class still owes that class a statement of
/// where it stopped: this drains the window from the front, one ended class at a time, and stops at
/// the first class that has not ended (its task is still working, and no record of it may claim
/// otherwise). Nothing else is delivered: the method records still waiting in those slots were never
/// confirmed by the consumer, so they stay executed-but-undelivered.
///
/// The wait for a class that is still winding down is bounded per class: a task whose next checkpoint
/// is one slot check returns at once, and one that is in the middle of a method states nothing here
/// rather than holding the call open.
fn drain_ended_classes(operation: &Operation<'_>, delivery: &mut Delivery<'_>) -> bool {
    loop {
        if delivery.closed {
            return false;
        }
        let deadline = std::time::Instant::now() + DRAIN_PER_CLASS;
        let taken = loop {
            let mut state = operation.registry.lock();
            match state.slots.front_mut() {
                None => break None,
                Some(slot) => {
                    if let Some(ending) = slot.ending.take() {
                        let class = slot.class.clone();
                        let ordinal = state.front;
                        state.slots.pop_front();
                        state.front = state.front.saturating_add(1);
                        operation.registry.changed.notify_all();
                        break Some((ordinal, class, ending));
                    }
                }
            }
            let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) else {
                break None;
            };
            let guard = operation.registry.wait_signal(
                state,
                WindowSite::TakeFront,
                remaining.min(WAIT_SLICE),
            );
            drop(guard);
        };
        let Some((ordinal, class, ending)) = taken else {
            return true;
        };
        if !publish_class_end(operation, delivery, ordinal, &class, ending) {
            return false;
        }
    }
}

/// Why the operation cannot go on, when it cannot: a stop the ledger recorded, or a stop the caller
/// asked for.
///
/// The caller's cancellation is read directly from the entry budget's own token as well as through
/// the ledger, so a caller that cancels between two charges is obeyed at the next checkpoint rather
/// than at the next charge.
fn stop_ending(operation: &Operation<'_>, delivery: &Delivery<'_>) -> Option<Ending> {
    if delivery.closed {
        return Some(Ending::Stopped);
    }
    // A worker that did not return, or a sink callback that failed, closes the registry before the
    // coordinator reaches its next checkpoint; that ending is the operation's own.
    if let Some(ending) = operation.registry.ending() {
        return Some(ending);
    }
    if operation.stopped() {
        return Some(Ending::Stopped);
    }
    if delivery.budget.cancellation_token().is_cancelled() {
        operation.ledger.cancel(BulkStopKind::Cancelled);
        return Some(Ending::Stopped);
    }
    None
}

/// What one step of the traversal yielded.
enum Next {
    /// One class candidate, which the traversal has seen and counted.
    Class(Box<ScopeClass>),
    /// The traversal reached the end of the scope.
    Done,
    /// The traversal could not go on: the operation is ending.
    Stopped,
}

/// Pulls the next class candidate from the traversal, publishing whatever the cursor recorded while
/// it looked.
fn next_class(
    operation: &Operation<'_>,
    cursor: &mut jarde_reader::scope_cursor::ScopeCursor,
    discovery: &mut Budget,
    delivery: &mut Delivery<'_>,
    published: &mut usize,
) -> Next {
    let step = cursor.next_class(discovery);
    let _ = publish_traversal_diagnostics(operation, delivery, cursor, published);
    if delivery.closed {
        return Next::Stopped;
    }
    match step {
        Ok(Some(class)) => {
            operation.registry.note_seen();
            Next::Class(Box::new(class))
        }
        Ok(None) => Next::Done,
        Err(error) => {
            // A stop, a cancellation or a root this scope cannot read: the traversal ends here, and
            // the operation's own stop record says why. A root that failed is infrastructure — the
            // scope has no readable prefix at all — while a budget or a cancellation is the stop the
            // ledger already holds.
            if !operation.stopped() {
                operation.registry.close(match &error {
                    Error::Cancelled { .. } | Error::BudgetExceeded { .. } => Ending::Stopped,
                    _ => {
                        operation.ledger.cancel(BulkStopKind::Infrastructure);
                        Ending::Infrastructure {
                            code: crate::facade::error_code(&error),
                            message: error.to_string(),
                        }
                    }
                });
            }
            Next::Stopped
        }
    }
}

/// Folds one delivered method's execution planes into its class's own.
fn merge_class_execution(class_execution: &mut Option<ExecutionReport>, event: &MethodResultEvent) {
    let mut incoming: Vec<ExecutionReport> = Vec::new();
    match &event.delivery {
        MethodDelivery::Recovered(recovered) => {
            incoming.push(recovered.analysis().execution.clone());
            incoming.push(recovered.recovery().execution.clone());
        }
        MethodDelivery::NoBody { analysis } => incoming.push(analysis.execution.clone()),
        MethodDelivery::Refused { execution, .. } | MethodDelivery::Oversized { execution, .. } => {
            incoming.push(execution.clone());
        }
    }
    for execution in incoming {
        match class_execution {
            Some(current) => crate::facade::merge_execution(current, execution),
            None => *class_execution = Some(execution),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The workers: bounded, scoped to this call, joined before it returns
// ---------------------------------------------------------------------------------------------

/// One worker's loop: take the next dispatched class until the operation has none left.
///
/// The loop owns no class list and no result: it takes one class at a time from the shared registry,
/// runs the same class task the serial configuration runs, and hands each record to that class's own
/// slot. It returns when the operation closes or when the traversal ended with every dispatched
/// class taken, which is what makes the coordinator's join prompt.
fn worker_loop(operation: &Operation<'_>) {
    let _alive = operation.faults.register_worker();
    let lived = operation.observation.worker_started();
    let mut busy = Duration::ZERO;
    while let Some((index, class, container)) = operation.registry.take_task() {
        // The container the walk handed over with this class, held for exactly as long as this class
        // task runs: the read below and every member's binding query are answered from that verified
        // product instead of parsing the directory again, however far the walk has moved on in the
        // meantime and whatever the caller's facts store decided to keep. Dropping it when the task
        // returns is the release, so the decision about a container no class task is reading any
        // more belongs to the store alone.
        let _container = container;
        let guard = TaskGuard { operation, index };
        let mut budget =
            operation.budget_for(operation.discovery_limits.clone(), UsageOwner::Discovery);
        let mut consumer = SlotConsumer {
            registry: operation.registry,
            index,
        };
        let started = operation.observation.class_task_started();
        let ending = run_class_task(operation, &class, index, &mut budget, &mut consumer);
        operation.observation.class_task_ended(started);
        if let Some(started) = started {
            busy = busy.saturating_add(started.elapsed());
        }
        drop(guard);
        operation.registry.finish_task(index, ending);
    }
    operation.observation.worker_ended(lived, busy);
}

/// A class task's presence in the window while it runs, as the panic path needs it.
///
/// A worker that panics never reaches its own `finish_task`, and a coordinator waiting for that
/// class's slot would wait forever. The guard is what makes an observed panic a *bounded* failure: it
/// publishes the class as closed and closes the operation with an infrastructure ending, so the
/// coordinator wakes, stops dispatching and every worker returns.
struct TaskGuard<'a, 'o> {
    operation: &'a Operation<'o>,
    index: u64,
}

impl Drop for TaskGuard<'_, '_> {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            return;
        }
        self.operation.registry.finish_task(
            self.index,
            ClassEnding::Closed {
                methods: 0,
                closing: Some(Closing::Stopped(Error::invalid_input(
                    "bulk_worker_panicked",
                    format!(
                        "the class task of class ordinal {} did not return",
                        self.index
                    ),
                ))),
                execution: None,
            },
        );
        self.operation.registry.close(Ending::Infrastructure {
            code: "bulk_worker_panicked".to_owned(),
            message: format!(
                "the class task of class ordinal {} did not return: the operation ends rather than \
                 retrying the class or silently running it on this thread",
                self.index
            ),
        });
    }
}

/// What running the workers left behind.
struct WorkerOutcome {
    stream: Stream,
    /// The infrastructure failure the worker lifecycle itself produced: a thread that could not be
    /// created, or one that did not return.
    failure: Option<String>,
}

/// Runs one operation's classes on workers scoped to this call.
///
/// The threads are created by [`std::thread::Builder::spawn_scoped`], so a creation failure is
/// observable rather than fatal, and every one of them is joined before this function returns: no
/// thread of this operation outlives it, and no second run ever starts. A creation failure closes
/// the operation before anything is dispatched — the operation does not fall back to the calling
/// thread — and a worker that did not return is reported as the infrastructure failure it is.
fn run_workers(
    operation: &Operation<'_>,
    cursor: &mut jarde_reader::scope_cursor::ScopeCursor,
    delivery: &mut Delivery<'_>,
) -> WorkerOutcome {
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        let mut failure = None;
        for index in 0..operation.limits.workers_effective {
            if operation.faults.creation_refused(index) {
                failure = Some("bulk_worker_spawn_failed".to_owned());
                break;
            }
            let created = std::thread::Builder::new()
                .name(format!("jarde-bulk-{index}"))
                .spawn_scoped(scope, || worker_loop(operation));
            match created {
                Ok(handle) => handles.push(handle),
                Err(error) => {
                    failure = Some("bulk_worker_spawn_failed".to_owned());
                    let _ = error;
                    break;
                }
            }
        }
        let stream = if failure.is_some() {
            // A worker the operating system refused is the operation's end: nothing is dispatched,
            // nothing is recovered on the calling thread, and the report states the failure.
            operation.registry.close(Ending::Infrastructure {
                code: "bulk_worker_spawn_failed".to_owned(),
                message: format!(
                    "this operation asked for {} worker(s) and could not create all of them, so it \
                     stops before dispatching any class rather than running some of them or falling \
                     back to the calling thread",
                    operation.limits.workers_effective
                ),
            });
            Stream {
                traversal_complete: false,
            }
        } else {
            coordinate(operation, cursor, delivery, Dispatch::Workers)
        };
        // Every worker is joined before this call returns, whatever the coordinator reached. The
        // operation is closed first so a worker waiting for work or for a slot can return instead of
        // waiting for a dispatch that will not come.
        operation.registry.close(Ending::Reached);
        let mut panicked = false;
        for handle in handles {
            if handle.join().is_err() {
                panicked = true;
            }
        }
        if panicked {
            operation.registry.close(Ending::Infrastructure {
                code: "bulk_worker_panicked".to_owned(),
                message: "a worker did not return: the operation ends rather than retrying it"
                    .to_owned(),
            });
            failure = Some("bulk_worker_panicked".to_owned());
        }
        WorkerOutcome { stream, failure }
    })
}

// ---------------------------------------------------------------------------------------------
// The operation's own account: usage, window, coverage and the aggregate
// ---------------------------------------------------------------------------------------------

/// Whether one execution plane ran to its end.
fn complete(execution: &ExecutionReport) -> bool {
    matches!(execution, ExecutionReport::Complete { .. })
}

/// The name of one aggregate state, as a caller branches on it without matching usage figures.
fn status_of(execution: &ExecutionReport) -> &'static str {
    match execution {
        ExecutionReport::Complete { .. } => "complete",
        ExecutionReport::Partial { .. } => "partial",
        ExecutionReport::Cancelled { .. } => "cancelled",
        ExecutionReport::Failed { .. } => "failed",
    }
}

/// The coverage of one bulk operation.
///
/// The structural plane states the classes this operation accounted for: the candidates the
/// traversal yielded are the scanned range, and the plane is complete only when the traversal
/// reached the end of an undamaged scope, no class was refused and every declared method of every
/// prepared class was executed. That is the honest reading of "the scope was covered": the range is
/// a count of classes, not a claim about a total this engine never established.
fn coverage_of(traversal_complete: bool, counts: &Counts, execution: &ExecutionReport) -> Coverage {
    let complete = complete(execution) && traversal_complete && counts.classes_refused == 0;
    Coverage {
        artifact_structural: CoverageDimension {
            state: if complete {
                CoverageState::CompleteWithinSchema
            } else {
                CoverageState::Partial
            },
            scanned: vec![CoverageRange {
                label: "bulk_classes".to_owned(),
                start: 0,
                end: counts.classes_seen,
            }],
            skipped: Vec::new(),
            uninterpreted_extensions: Vec::new(),
        },
        runtime_resolution: CoverageDimension::not_requested(),
        dynamic_analysis: CoverageDimension::not_requested(),
    }
}

/// The operation's aggregate state, in the priority the design fixes.
///
/// Infrastructure `Failed` first, then a global `Cancelled`, then `Partial` — a budget stop, an
/// incomplete method execution, a refused class or a damaged traversal — and `Complete` only when the
/// traversal reached the end, every required execution completed and every produced record was
/// delivered. `final_confirmed` is false for the report of a stream whose `Final` event was not
/// confirmed, which is a prefix however complete the records before it looked.
fn aggregate(
    operation: &Operation<'_>,
    state: &OperationState,
    traversal_complete: bool,
    failure: Option<&str>,
    final_confirmed: bool,
) -> ExecutionReport {
    let usage = operation.ledger.usage();
    let stop = operation.ledger.stop_reason();
    if let Some(code) = failure {
        return ExecutionReport::Failed {
            reason: TerminationReason::Error {
                code: code.to_owned(),
            },
            usage,
        };
    }
    match state.ending.as_ref() {
        Some(Ending::Infrastructure { code, .. }) => {
            return ExecutionReport::Failed {
                reason: TerminationReason::Error { code: code.clone() },
                usage,
            };
        }
        Some(Ending::Reached) | Some(Ending::Stopped) | None => {}
    }
    match stop.as_ref().map(|stop| stop.kind) {
        Some(BulkStopKind::Infrastructure) => {
            return ExecutionReport::Failed {
                reason: TerminationReason::Error {
                    code: "bulk_infrastructure_failed".to_owned(),
                },
                usage,
            };
        }
        Some(BulkStopKind::Cancelled) | Some(BulkStopKind::Sink) => {
            return ExecutionReport::Cancelled { usage };
        }
        Some(BulkStopKind::Budget) => {
            let dimension = stop
                .and_then(|stop| stop.dimension)
                .unwrap_or(crate::BudgetDimension::ResultItems);
            return ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded { dimension },
                usage,
            };
        }
        None => {}
    }
    let counts = &state.counts;
    let reason = incomplete_reason(state);
    if counts.classes_refused > 0
        || counts.incomplete_methods > 0
        || counts.methods_delivered < counts.methods_executed
        || !traversal_complete
        || !final_confirmed
    {
        return ExecutionReport::Partial { reason, usage };
    }
    ExecutionReport::Complete { usage }
}

/// Why an operation that is not `Complete` is `Partial`.
///
/// The reason is the strongest stop the operation's own classes and methods published, so a report
/// names the failure that really happened (a refused class's code, a method's exhausted dimension)
/// instead of a generic "incomplete"; an operation whose only hole is an unconfirmed `Final` states
/// that code instead.
fn incomplete_reason(state: &OperationState) -> TerminationReason {
    match state.strongest.as_ref() {
        Some(ExecutionReport::Partial { reason, .. })
        | Some(ExecutionReport::Failed { reason, .. }) => reason.clone(),
        _ => TerminationReason::Error {
            code: "bulk_operation_incomplete".to_owned(),
        },
    }
}

/// The retention window this run reached, from the registry's own high-water marks.
fn window_of(state: &OperationState, limits: &BulkLimits) -> BulkWindow {
    BulkWindow {
        active_classes: limits.workers_effective as u64,
        concurrent_classes_high_water: state.executing_high_water,
        window_slots_high_water: state.slot_count_high_water,
        buffered_weight_high_water: state.retained_weight_high_water,
        largest_result_weight: state.largest_result_weight,
        result_weight_limit: limits.max_result_weight,
        buffered_weight_limit: limits.max_buffered_result_weight,
    }
}

// ---------------------------------------------------------------------------------------------
// The operation
// ---------------------------------------------------------------------------------------------

/// Recovers every method of one physical scope in one operation.
///
/// This is the entry the facade delegates to ([`crate::Engine::recover_all`]); see the module
/// documentation for what the operation guarantees. Three things are decided here, once, before
/// anything is read or dispatched:
///
/// * the request's own shape — the snapshot has to be provided, the environment it declares has to
///   name the same view the traversal walks, and the environment has to be one this engine can serve
///   (`EnvironmentRequest::build` plus the JVM layer's own validation, performed **once** for the
///   whole operation rather than once per method);
/// * the effective configuration ([`BulkLimits::effective_from`]), which refuses a request that is
///   not a configuration at all — no worker, a per-result ceiling of zero, a window that cannot hold
///   one result, a preparation ceiling of zero — and publishes the window-derived worker count the
///   operation really runs;
/// * the operation's total: the ledger folds the entry budget's own usage in and every worker bills
///   to it, so a bulk operation cannot start from a fresh allowance or multiply its quota by its
///   worker count.
///
/// An input error is a `Result` `Err` and happens before anything is read. Everything after it
/// returns a report: a stream that stops because of the operation's own quota, the caller's
/// cancellation, a consumer that stopped accepting records, a worker that could not be created or a
/// sink that failed is an operation the report accounts for, with the first stop it observed, the
/// delivered prefix and the real execution holes beside it.
pub fn recover_all(
    content: &[ArtifactSnapshot],
    request: &BulkRecoveryRequest,
    budget: &mut Budget,
    sink: &mut dyn RecoverySink,
) -> Result<BulkRecoveryReport> {
    let store = crate::OperationStore::of(budget);
    let total = budget.limits().clone();
    let limits = BulkLimits::effective_from(request, &total, store.capacity())?;
    if request.environment.snapshot != request.snapshot
        || request.environment.scope != request.scope
    {
        return Err(Error::invalid_input(
            "bulk_environment_view_mismatch",
            "the environment declares another physical view than the request's own snapshot and \
             scope: one operation walks one view, and a second declaration of it is a request this \
             engine refuses to reconcile",
        ));
    }
    let Some(snapshot) = content
        .iter()
        .find(|candidate| candidate.id() == &request.snapshot)
    else {
        return Err(crate::facade::snapshot_not_provided(&request.snapshot));
    };
    let environment = request.environment.build(content)?;
    let (problems, _identity) = crate::environment::validate_environment(content, &environment);
    if !problems.is_empty() {
        let codes: Vec<&'static str> = problems
            .iter()
            .map(|problem| problem.code.as_str())
            .collect();
        return Err(Error::invalid_input(
            "bulk_environment_rejected",
            format!(
                "this operation's environment is one the engine will not serve ({codes:?}), so no \
                 method of the scope could be analysed under it: a rejected declaration is the \
                 request's, not a per-method result, and it is refused once instead of being \
                 published identically for every method of every class"
            ),
        ));
    }
    let mut cursor = snapshot.scope_cursor(&request.scope)?;
    let ledger = OperationLedger::new(budget);
    #[cfg(feature = "test-support")]
    let ledger = match request.probe.as_ref() {
        // The probe sees the operation's own total as a caller's observer: nothing else in the run
        // reads it, and a request without one leaves the total unobserved.
        Some(probe) => ledger.with_observer(probe.clone()),
        None => ledger,
    };
    budget.with_ledger(ledger.clone(), UsageOwner::Discovery);
    #[cfg(feature = "test-support")]
    let observation = Observation::of(request.probe.clone());
    #[cfg(not(feature = "test-support"))]
    let observation = Observation::of();
    let registry = Registry::new(observation.clone());
    let operation = Operation {
        content,
        snapshot,
        environment: &environment,
        limits: &limits,
        ledger: &ledger,
        stages: crate::facade::MethodOperation::Recovery.stages(),
        evidence: &request.evidence,
        registry: &registry,
        store,
        discovery_limits: budget.limits().clone(),
        observation: observation.clone(),
        faults: Faults::from_request(request),
    };
    let account = DeliveryAccount::of(&ledger, budget.limits());
    let mut delivery = Delivery {
        sink,
        budget,
        account,
        closed: false,
        observation: observation.clone(),
    };
    let header = BulkHeaderEvent {
        view: PhysicalView {
            snapshot: request.snapshot.clone(),
            scope: request.scope.clone(),
        },
        limits: limits.clone(),
    };
    let mut dispatch = true;
    if stop_ending(&operation, &delivery).is_none() {
        if !publish(&operation, &mut delivery, Record::Header(&header), None) {
            dispatch = false;
            registry.close(Ending::Stopped);
        }
    } else {
        dispatch = false;
    }
    let mut failure: Option<String> = None;
    let stream = if dispatch {
        if limits.workers_effective == 1 {
            coordinate(&operation, &mut cursor, &mut delivery, Dispatch::Serial)
        } else {
            let outcome = run_workers(&operation, &mut cursor, &mut delivery);
            failure = outcome.failure;
            outcome.stream
        }
    } else {
        Stream {
            traversal_complete: false,
        }
    };
    // The operation is over. Everything below reads it: no worker is still running, and the closing
    // record below is the last thing any waiter can observe.
    registry.close(Ending::Reached);
    let (counts, window, strongest, ending) = {
        let state = registry.lock();
        (
            state.counts.clone(),
            window_of(&state, &limits),
            state.strongest.clone(),
            state.ending.clone(),
        )
    };
    let _ = strongest;
    if let Some(ending) = ending.as_ref()
        && let Ending::Infrastructure { code, .. } = ending
    {
        failure = Some(code.clone());
    }
    let provisional = {
        let state = registry.lock();
        aggregate(
            &operation,
            &state,
            stream.traversal_complete,
            failure.as_deref(),
            true,
        )
    };
    let summary = BulkSummary {
        view: header.view.clone(),
        limits: limits.clone(),
        classes_seen: counts.classes_seen,
        classes_prepared: counts.classes_prepared,
        classes_refused: counts.classes_refused,
        methods_declared: counts.methods_declared,
        methods_executed: counts.methods_executed,
        methods_delivered: counts.methods_delivered,
        methods_not_executed: counts
            .methods_declared
            .saturating_sub(counts.methods_executed),
        outcomes: counts.outcomes,
        traversal_complete: stream.traversal_complete,
        execution: jarde_reader::accounting::with_usage(
            provisional.clone(),
            operation.ledger.usage(),
        ),
    };
    let final_event = BulkFinalEvent {
        summary: summary.clone(),
    };
    let final_delivered = if delivery.closed {
        false
    } else {
        publish(&operation, &mut delivery, Record::Final(&final_event), None)
    };
    let execution = if final_delivered {
        provisional
    } else {
        let state = registry.lock();
        aggregate(
            &operation,
            &state,
            stream.traversal_complete,
            failure.as_deref(),
            false,
        )
    };
    let coverage = coverage_of(stream.traversal_complete, &counts, &execution);
    let mut diagnostics = cursor.diagnostics().to_vec();
    if let Some(ending) = ending.as_ref()
        && let Ending::Infrastructure { code, message } = ending
    {
        diagnostics.push(Diagnostic {
            code: code.clone(),
            severity: DiagnosticSeverity::Error,
            message: message.clone(),
            provenance: None,
        });
    }
    let usage = operation.ledger.usage();
    // Every thread that entered the operation's accounting has returned by now, so the coordinator's
    // own tallies are added here: a caller that reads the probe after this call reads the whole run.
    observation.flush();
    Ok(BulkRecoveryReport {
        summary: BulkSummary {
            execution: jarde_reader::accounting::with_usage(execution, usage.clone()),
            ..summary
        },
        usage: usage.clone(),
        entry_usage: operation.ledger.entry_usage(),
        discovery_usage: operation.ledger.cumulative(UsageOwner::Discovery),
        method_usage: operation.ledger.cumulative(UsageOwner::Methods),
        delivery_usage: operation.ledger.cumulative(UsageOwner::Delivery),
        window,
        coverage,
        stop: stop_record(&operation, ending.as_ref()),
        diagnostics,
        final_delivered,
    })
}
