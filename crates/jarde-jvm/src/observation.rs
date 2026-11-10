//! Fixed-size S0 test observations. Counts are process-wide events, never report fields.
//! One isolated process owns a sample; concurrent workers use the same atomic counters.
//! A stage attempt is counted immediately before its implementation is called. A published
//! table is counted after the driver has accepted it; a returned payload may still be partial.

#[cfg(feature = "test-support")]
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) const PIPELINE: usize = 0;
pub(crate) const RAW_CFG: usize = 1;
pub(crate) const CONTEXTS: usize = 2;
pub(crate) const CANONICAL: usize = 3;
pub(crate) const FRAMES: usize = 4;
pub(crate) const SSA: usize = 5;

#[cfg(feature = "test-support")]
static STARTS: [AtomicU64; 6] = [const { AtomicU64::new(0) }; 6];
#[cfg(feature = "test-support")]
static RETURNS: [AtomicU64; 6] = [const { AtomicU64::new(0) }; 6];

#[inline]
pub(crate) fn started(stage: usize) {
    #[cfg(feature = "test-support")]
    STARTS[stage].fetch_add(1, Ordering::Relaxed);
    #[cfg(not(feature = "test-support"))]
    let _ = stage;
}

#[inline]
pub(crate) fn published(stage: usize) {
    #[cfg(feature = "test-support")]
    RETURNS[stage].fetch_add(1, Ordering::Relaxed);
    #[cfg(not(feature = "test-support"))]
    let _ = stage;
}

/// Index order: pipeline, raw CFG, call contexts, canonical CFG, frames, SSA.
/// Pipeline completion means returned payload, including refused/stopped prefixes;
/// the other completions mean accepted table publication, not whole-method success.
#[cfg(feature = "test-support")]
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct Counts {
    pub starts: [u64; 6],
    pub completions: [u64; 6],
}

#[cfg(feature = "test-support")]
pub fn snapshot() -> Counts {
    Counts {
        starts: std::array::from_fn(|i| STARTS[i].load(Ordering::Relaxed)),
        completions: std::array::from_fn(|i| RETURNS[i].load(Ordering::Relaxed)),
    }
}

#[cfg(feature = "test-support")]
impl Counts {
    pub fn since(self, later: Self) -> Self {
        Self {
            starts: std::array::from_fn(|i| later.starts[i] - self.starts[i]),
            completions: std::array::from_fn(|i| later.completions[i] - self.completions[i]),
        }
    }
}

/// Actual MethodIr owners alive in this process. Moves keep one owner; no heap-byte claim.
/// A lifetime sample requires an isolated test process or serial execution.
#[cfg(feature = "test-support")]
static METHOD_IR_LIVE: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "test-support")]
#[derive(Debug)]
pub(crate) struct LiveMethodIr;

#[cfg(feature = "test-support")]
impl LiveMethodIr {
    pub(crate) fn new() -> Self {
        METHOD_IR_LIVE.fetch_add(1, Ordering::Relaxed);
        Self
    }
}

#[cfg(feature = "test-support")]
impl Drop for LiveMethodIr {
    fn drop(&mut self) {
        METHOD_IR_LIVE.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(feature = "test-support")]
pub fn live_method_irs() -> u64 {
    METHOD_IR_LIVE.load(Ordering::Relaxed)
}
