//! D0 1.3's bounded counting port: what a demand path *did*, counted where it did it.
//!
//! # Why a port and not a field
//!
//! The demand-driven work this change owns is a property of a *run*, not of any result the run
//! publishes: "one class was prepared once", "no body was decoded", "no optional detail record was
//! built" are answers a report cannot state without becoming a second, competing contract — and a
//! report that stated them would also change the domain fingerprint every repeated-run gate compares
//! byte for byte. The parallel bulk operation's own observation port (`src/bulk/observation.rs`, a
//! test-support-only probe hung off its request) is the same port-shaped answer for *its*
//! coordination cost, and this module follows that precedent deliberately.
//!
//! So nothing here is part of [`crate::RecoveryReport`], of any other report, of a stop record or of
//! a fingerprint. The counters are compiled out of a normal build: every hook below is an empty
//! function there, no counter exists to be read, and the call sites stay the same one-line calls they
//! are in a test-support build. A build that has the port keeps exactly five `u64`s, whatever a run
//! does — the port is *bounded* by construction, and no counter ever retains a record of the work it
//! counted.
//!
//! # What each counter means, and where it is hooked
//!
//! Every counter names one event of the demand path the change owns, and is incremented at the one
//! site that performs it:
//!
//! | counter | one increment is | hooked at |
//! | --- | --- | --- |
//! | `class_materializations` | one class's bytes materialized as a trusted read of the definition an operation **selected** | `crate::facade`'s `bind_class` (the identity path's read) and `recover_own_read` (the read a recovery run performed itself) |
//! | `class_preparations` | one `PreparedClass::prepare` built over such a read | `crate::facade`'s `class_source`, `class_view` and `recovery_from` |
//! | `body_decodes` | one method body decoded on a demand path | `crate::facade`'s `body_result`, `recover_bound_method`, `recover_own_read` and `recover_prepared_member` |
//! | `recovery_runs` | one Java recovery presentation over one run's payload | `crate::facade`'s `recovery_presented` |
//! | `owned_records` | one owning result record this layer's own publication built | `crate::facade`'s `body_result` and `recovery_presented` |
//!
//! D2 (tasks 3.1–3.3) moved two of those sites without changing what they mean: a preparation is
//! now built over the read the binding performed — `read_prepared_definition`, which materialized
//! the definition a second time, is gone — and a class view and a direct recovery run prepare the
//! class they decode bodies from, so `class_preparations` is the figure that says "one preparation
//! served every body this operation decoded".
//!
//! # What is *not* hooked here, and why
//!
//! Three of the five events the D0 gate needs are performed inside layers this round freezes, so
//! their counts are read from an existing public surface instead of from a hook (see the change's
//! verification record):
//!
//! * **consumer work** happens in `jarde-query`'s consumers; the gate reads it as
//!   `QueryReport::coverage.scanned_items` and the usage deltas of the same request, both through
//!   [`crate::Engine::query`]'s own report;
//! * **the optional detail records of a recovery report** are constructed in `jarde-java`; the gate
//!   counts the owning records the report *published*, per category, until that layer's freeze is
//!   lifted and the D3 hook can be placed where the records are built. A published count is a lower
//!   bound of the constructions — a record built and then dropped before publication is invisible to
//!   it — which is exactly why D3 must place the real hook beside the construction;
//! * **release** is observed as the lifetime of the reader's own handle:
//!   `Arc::strong_count(PreparedClass::facts_handle())` falls back to its baseline once the
//!   consumer and the preparation are dropped, so no counter can be wrong about it.

/// The counter indices, in the order [`Counts`] states them.
///
/// They exist in every build: a hook call site names one, so the call site is the same line in a
/// build with the port and in one without it.
const CLASS_MATERIALIZATIONS: usize = 0;
const CLASS_PREPARATIONS: usize = 1;
const BODY_DECODES: usize = 2;
const RECOVERY_RUNS: usize = 3;
const OWNED_RECORDS: usize = 4;

/// How many counters this port keeps.
#[cfg(any(test, feature = "test-support"))]
const COUNTERS: usize = 5;

/// Every counter, in the order the indices above name them.
#[cfg(any(test, feature = "test-support"))]
static COUNTS: [std::sync::atomic::AtomicU64; COUNTERS] =
    [const { std::sync::atomic::AtomicU64::new(0) }; COUNTERS];

/// One increment of one counter, in a build that has the port.
#[cfg(any(test, feature = "test-support"))]
#[inline]
fn bump(counter: usize) {
    COUNTS[counter].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// The same increment, in a build without the port: nothing is counted and nothing is kept.
#[cfg(not(any(test, feature = "test-support")))]
#[inline]
fn bump(_counter: usize) {}

/// `count` increments at once, or none at all in a build without the port.
#[cfg(any(test, feature = "test-support"))]
#[inline]
fn bump_by(counter: usize, count: u64) {
    if count > 0 {
        COUNTS[counter].fetch_add(count, std::sync::atomic::Ordering::Relaxed);
    }
}

/// The same, in a build without the port.
#[cfg(not(any(test, feature = "test-support")))]
#[inline]
fn bump_by(_counter: usize, _count: u64) {}

/// One class's bytes were materialized as a trusted read, to be prepared.
pub(crate) fn class_materialized() {
    bump(CLASS_MATERIALIZATIONS);
}

/// One prepared class was built over such a read.
pub(crate) fn class_prepared() {
    bump(CLASS_PREPARATIONS);
}

/// One method body was decoded on a demand path.
pub(crate) fn body_decoded() {
    bump(BODY_DECODES);
}

/// One recovery presentation ran over one analysis run's payload.
pub(crate) fn recovery_presented_run() {
    bump(RECOVERY_RUNS);
}

/// `built` owning result records were constructed by one publication of this layer.
pub(crate) fn owned_records(built: u64) {
    bump_by(OWNED_RECORDS, built);
}

/// What a demand path counted, as plain numbers.
///
/// A reading is a value, not a live view: a caller takes one before and one after the work it is
/// asking about and reads [`Counts::since`], so a count taken by a test running beside it cannot be
/// mistaken for its own.
#[cfg(any(test, feature = "test-support"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counts {
    /// Class bytes materialized as a trusted read for a preparation.
    pub class_materializations: u64,
    /// Prepared classes built.
    pub class_preparations: u64,
    /// Method bodies decoded on a demand path.
    pub body_decodes: u64,
    /// Recovery presentations run.
    pub recovery_runs: u64,
    /// Owning result records built by this layer's own publications.
    pub owned_records: u64,
}

#[cfg(any(test, feature = "test-support"))]
impl Counts {
    /// The work these counters did between `self` and `later`: one field per counter, `later` minus
    /// `self`.
    ///
    /// The difference is the demand path's own, so a reading of the whole process — which other
    /// tests running in parallel also move — is still a usable baseline.
    pub fn since(self, later: Counts) -> Counts {
        Counts {
            class_materializations: later.class_materializations - self.class_materializations,
            class_preparations: later.class_preparations - self.class_preparations,
            body_decodes: later.body_decodes - self.body_decodes,
            recovery_runs: later.recovery_runs - self.recovery_runs,
            owned_records: later.owned_records - self.owned_records,
        }
    }

    /// Whether no counter moved at all.
    pub fn is_silent(&self) -> bool {
        *self == Counts::default()
    }

    /// The sum of every counter: one figure for "did anything of this run's own work happen".
    pub fn total(&self) -> u64 {
        self.class_materializations
            + self.class_preparations
            + self.body_decodes
            + self.recovery_runs
            + self.owned_records
    }
}

/// Every counter, as one reading.
#[cfg(any(test, feature = "test-support"))]
pub fn snapshot() -> Counts {
    let read = |counter: usize| COUNTS[counter].load(std::sync::atomic::Ordering::Relaxed);
    Counts {
        class_materializations: read(CLASS_MATERIALIZATIONS),
        class_preparations: read(CLASS_PREPARATIONS),
        body_decodes: read(BODY_DECODES),
        recovery_runs: read(RECOVERY_RUNS),
        owned_records: read(OWNED_RECORDS),
    }
}

/// Every counter back to zero.
///
/// A gate that takes a baseline with [`snapshot`] and reads a difference does not need this; it
/// exists so a long test binary can be read from a known origin.
#[cfg(any(test, feature = "test-support"))]
pub fn reset() {
    for counter in &COUNTS {
        counter.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}
