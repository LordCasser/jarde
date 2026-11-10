//! Test-support-only live object counts for recovery intermediates.
//!
//! These counters state how many instrumented values are currently owned. They intentionally do
//! not estimate allocated bytes or count shared `Arc` handles as payloads.
//!
//! Candidate payload coverage includes generic return/constructor results; class initializer and
//! enum-constructor results; array-constructor, lambda-helper, bridge and enum-switch candidates;
//! anonymous-allocation scans and candidates; retained class-source method AST payloads; and S1
//! same-run projection proof payloads. The listed payloads are counted at construction, including
//! rejected temporaries. Individual projection segments are not separately counted.
//! Vector allocations, other local proof records and `Region` trees are not counted. These are selected payload counters, not a census of the crate's heap.

use std::sync::atomic::{AtomicU64, Ordering};

const EXPRS: usize = 0;
const STMTS: usize = 1;
const CANDIDATES: usize = 2;
const METHOD_AST_SOURCES: usize = 3;
const KINDS: usize = 4;

static LIVE: [AtomicU64; KINDS] = [const { AtomicU64::new(0) }; KINDS];
static PEAK: [AtomicU64; KINDS] = [const { AtomicU64::new(0) }; KINDS];

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Expr,
    Stmt,
    Candidate,
    MethodAstSource,
}

impl Kind {
    const fn index(self) -> usize {
        match self {
            Self::Expr => EXPRS,
            Self::Stmt => STMTS,
            Self::Candidate => CANDIDATES,
            Self::MethodAstSource => METHOD_AST_SOURCES,
        }
    }
}

/// One owned instrumented payload. Cloning it represents a new owned value; cloning an `Arc`
/// containing one only changes the Arc count and does not clone this token.
#[doc(hidden)]
pub struct Token(Kind);

impl Token {
    /// Instrument an externally constructed test candidate, without asserting recovery proof.
    #[doc(hidden)]
    pub fn candidate() -> Self {
        Self::new(Kind::Candidate)
    }
    pub(crate) fn new(kind: Kind) -> Self {
        increment(kind.index());
        Self(kind)
    }
}

impl Clone for Token {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl Drop for Token {
    fn drop(&mut self) {
        LIVE[self.0.index()].fetch_sub(1, Ordering::Relaxed);
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Token")
    }
}

impl PartialEq for Token {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for Token {}

fn increment(index: usize) {
    let live = LIVE[index].fetch_add(1, Ordering::Relaxed) + 1;
    let mut peak = PEAK[index].load(Ordering::Relaxed);
    while peak < live {
        match PEAK[index].compare_exchange_weak(peak, live, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(actual) => peak = actual,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize)]
pub struct Snapshot {
    pub expr_live: u64,
    pub expr_peak: u64,
    pub stmt_live: u64,
    pub stmt_peak: u64,
    pub class_candidate_payloads_live: u64,
    pub class_candidate_payloads_peak: u64,
    pub method_ast_sources_live: u64,
    pub method_ast_sources_peak: u64,
}

/// Reads current live counts and the highest simultaneous live count since the last reset.
pub fn snapshot() -> Snapshot {
    Snapshot {
        expr_live: LIVE[EXPRS].load(Ordering::Relaxed),
        expr_peak: PEAK[EXPRS].load(Ordering::Relaxed),
        stmt_live: LIVE[STMTS].load(Ordering::Relaxed),
        stmt_peak: PEAK[STMTS].load(Ordering::Relaxed),
        class_candidate_payloads_live: LIVE[CANDIDATES].load(Ordering::Relaxed),
        class_candidate_payloads_peak: PEAK[CANDIDATES].load(Ordering::Relaxed),
        method_ast_sources_live: LIVE[METHOD_AST_SOURCES].load(Ordering::Relaxed),
        method_ast_sources_peak: PEAK[METHOD_AST_SOURCES].load(Ordering::Relaxed),
    }
}

/// Resets peak readings to the current live counts without changing ownership counts.
pub fn reset_peaks() {
    for index in 0..KINDS {
        PEAK[index].store(LIVE[index].load(Ordering::Relaxed), Ordering::Relaxed);
    }
}
