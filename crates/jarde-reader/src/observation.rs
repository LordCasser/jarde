//! Test-support observations of reader work and live fact payloads.
//!
//! These values are process-wide test counters. They are not part of a reader result, budget, cache
//! report, or any persisted contract. File-read bytes count the snapshot file API; entry read-byte figures count the reader's wrapped
//! `Read` implementation (Deflate output for compressed entries); neither describes physical disk I/O. Payload counts describe object values,
//! not their heap size. Cloning an `Arc` does not create a payload; cloning a fact value does.

#[cfg(any(test, feature = "test-support"))]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(any(test, feature = "test-support"))]
static FILE_READ_ATTEMPTS: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static FILE_READ_BYTES: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static FILE_READ_EOFS: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static FILE_READ_ERRORS: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static BODY_DECODE_ATTEMPTS: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static READ_ATTEMPTS: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static READ_BYTES: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static READ_EOFS: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static READ_ERRORS: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static CLASS_FACTS_CONSTRUCTED: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static CLASS_FACTS_LIVE: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static CLASS_FACTS_PEAK: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static CONTAINER_FACTS_CONSTRUCTED: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static CONTAINER_FACTS_LIVE: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static CONTAINER_FACTS_PEAK: AtomicU64 = AtomicU64::new(0);

#[cfg(any(test, feature = "test-support"))]
static PREPARED_HANDLES_CONSTRUCTED: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static PREPARED_HANDLES_LIVE: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static PREPARED_HANDLES_PEAK: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static CONTAINER_HANDLES_CONSTRUCTED: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static CONTAINER_HANDLES_LIVE: AtomicU64 = AtomicU64::new(0);
#[cfg(any(test, feature = "test-support"))]
static CONTAINER_HANDLES_PEAK: AtomicU64 = AtomicU64::new(0);

#[cfg(any(test, feature = "test-support"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize)]
pub struct Snapshot {
    /// Artifact file reads, separate from entry wrapped-reader output below.
    pub file_read_attempts: u64,
    pub file_read_bytes: u64,
    pub file_read_eofs: u64,
    pub file_read_errors: u64,
    pub prepared_handles_constructed: u64,
    pub prepared_handles_live: u64,
    pub prepared_handles_peak: u64,
    pub container_handles_constructed: u64,
    pub container_handles_live: u64,
    pub container_handles_peak: u64,
    /// Entries into the shared Code decoder, including errors/stopped prefixes.
    pub body_decode_attempts: u64,
    /// Calls made to the wrapped reader, including EOF and error calls.
    pub read_attempts: u64,
    /// Bytes returned successfully by the wrapped reader.
    pub read_bytes: u64,
    /// Successful calls that returned zero bytes.
    pub read_eofs: u64,
    /// Calls for which the wrapped reader returned an error.
    pub read_errors: u64,
    /// ClassFacts values constructed, including value clones.
    pub class_facts_constructed: u64,
    /// ClassFacts values currently alive, counting values rather than Arc handles.
    pub class_facts_live: u64,
    /// Highest ClassFacts live value count since process start or the last peak reset.
    pub class_facts_peak: u64,
    /// ContainerFacts values constructed.
    pub container_facts_constructed: u64,
    /// ContainerFacts values currently alive, counting values rather than Arc handles.
    pub container_facts_live: u64,
    /// Highest ContainerFacts live value count since process start or the last peak reset.
    pub container_facts_peak: u64,
}

/// A value-level lifetime token for one fact payload.
///
/// The field on `ClassFacts` is public only in test-support builds because that type has public
/// fields. It is hidden from generated documentation and absent from normal builds.
#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
#[derive(Debug)]
pub struct PayloadToken {
    kind: PayloadKind,
}

#[cfg(any(test, feature = "test-support"))]
#[derive(Clone, Copy, Debug)]
enum PayloadKind {
    Class,
    Container,
    PreparedHandle,
    ContainerHandle,
}

#[cfg(any(test, feature = "test-support"))]
impl PayloadToken {
    pub(crate) fn prepared_handle() -> Self {
        Self::new(PayloadKind::PreparedHandle)
    }
    pub(crate) fn container_handle() -> Self {
        Self::new(PayloadKind::ContainerHandle)
    }
    pub(crate) fn class_facts() -> Self {
        Self::new(PayloadKind::Class)
    }

    pub(crate) fn container_facts() -> Self {
        Self::new(PayloadKind::Container)
    }

    fn new(kind: PayloadKind) -> Self {
        let (constructed, live, peak) = counters(kind);
        constructed.fetch_add(1, Ordering::Relaxed);
        let current = live.fetch_add(1, Ordering::Relaxed).saturating_add(1);
        raise_peak(peak, current);
        Self { kind }
    }
}

#[cfg(any(test, feature = "test-support"))]
impl Clone for PayloadToken {
    fn clone(&self) -> Self {
        Self::new(self.kind)
    }
}

#[cfg(any(test, feature = "test-support"))]
impl Drop for PayloadToken {
    fn drop(&mut self) {
        let (_, live, _) = counters(self.kind);
        live.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(any(test, feature = "test-support"))]
impl PartialEq for PayloadToken {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[cfg(any(test, feature = "test-support"))]
impl Eq for PayloadToken {}

#[inline(always)]
pub(crate) fn body_decode_started() {
    #[cfg(any(test, feature = "test-support"))]
    BODY_DECODE_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
}

#[inline(always)]
pub(crate) fn read_started() {
    #[cfg(any(test, feature = "test-support"))]
    READ_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
}

#[inline(always)]
pub(crate) fn read_finished(result: &std::io::Result<usize>) {
    #[cfg(any(test, feature = "test-support"))]
    match result {
        Ok(0) => {
            READ_EOFS.fetch_add(1, Ordering::Relaxed);
        }
        Ok(bytes) => {
            READ_BYTES.fetch_add(*bytes as u64, Ordering::Relaxed);
        }
        Err(_) => {
            READ_ERRORS.fetch_add(1, Ordering::Relaxed);
        }
    }
    #[cfg(not(any(test, feature = "test-support")))]
    let _ = result;
}

#[inline(always)]
pub(crate) fn file_read_started() {
    #[cfg(any(test, feature = "test-support"))]
    FILE_READ_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
}

#[inline(always)]
pub(crate) fn file_read_finished(result: &std::io::Result<usize>) {
    #[cfg(any(test, feature = "test-support"))]
    match result {
        Ok(0) => {
            FILE_READ_EOFS.fetch_add(1, Ordering::Relaxed);
        }
        Ok(bytes) => {
            FILE_READ_BYTES.fetch_add(*bytes as u64, Ordering::Relaxed);
        }
        Err(_) => {
            FILE_READ_ERRORS.fetch_add(1, Ordering::Relaxed);
        }
    }
    #[cfg(not(any(test, feature = "test-support")))]
    let _ = result;
}

/// Reads the current process-wide observation counters.
#[cfg(any(test, feature = "test-support"))]
pub fn snapshot() -> Snapshot {
    Snapshot {
        file_read_attempts: FILE_READ_ATTEMPTS.load(Ordering::Relaxed),
        file_read_bytes: FILE_READ_BYTES.load(Ordering::Relaxed),
        file_read_eofs: FILE_READ_EOFS.load(Ordering::Relaxed),
        file_read_errors: FILE_READ_ERRORS.load(Ordering::Relaxed),
        prepared_handles_constructed: PREPARED_HANDLES_CONSTRUCTED.load(Ordering::Relaxed),
        prepared_handles_live: PREPARED_HANDLES_LIVE.load(Ordering::Relaxed),
        prepared_handles_peak: PREPARED_HANDLES_PEAK.load(Ordering::Relaxed),
        container_handles_constructed: CONTAINER_HANDLES_CONSTRUCTED.load(Ordering::Relaxed),
        container_handles_live: CONTAINER_HANDLES_LIVE.load(Ordering::Relaxed),
        container_handles_peak: CONTAINER_HANDLES_PEAK.load(Ordering::Relaxed),
        body_decode_attempts: BODY_DECODE_ATTEMPTS.load(Ordering::Relaxed),
        read_attempts: READ_ATTEMPTS.load(Ordering::Relaxed),
        read_bytes: READ_BYTES.load(Ordering::Relaxed),
        read_eofs: READ_EOFS.load(Ordering::Relaxed),
        read_errors: READ_ERRORS.load(Ordering::Relaxed),
        class_facts_constructed: CLASS_FACTS_CONSTRUCTED.load(Ordering::Relaxed),
        class_facts_live: CLASS_FACTS_LIVE.load(Ordering::Relaxed),
        class_facts_peak: CLASS_FACTS_PEAK.load(Ordering::Relaxed),
        container_facts_constructed: CONTAINER_FACTS_CONSTRUCTED.load(Ordering::Relaxed),
        container_facts_live: CONTAINER_FACTS_LIVE.load(Ordering::Relaxed),
        container_facts_peak: CONTAINER_FACTS_PEAK.load(Ordering::Relaxed),
    }
}

/// Re-baselines live-payload peaks to the current live counts.
///
/// Call after warmup while the measured process is quiescent. Cumulative read and construction
/// counters are intentionally preserved, so callers can take before/after snapshots for deltas.
#[cfg(any(test, feature = "test-support"))]
pub fn reset_peaks() {
    PREPARED_HANDLES_PEAK.store(
        PREPARED_HANDLES_LIVE.load(Ordering::Relaxed),
        Ordering::Relaxed,
    );
    CONTAINER_HANDLES_PEAK.store(
        CONTAINER_HANDLES_LIVE.load(Ordering::Relaxed),
        Ordering::Relaxed,
    );
    CLASS_FACTS_PEAK.store(CLASS_FACTS_LIVE.load(Ordering::Relaxed), Ordering::Relaxed);
    CONTAINER_FACTS_PEAK.store(
        CONTAINER_FACTS_LIVE.load(Ordering::Relaxed),
        Ordering::Relaxed,
    );
}

#[cfg(any(test, feature = "test-support"))]
fn counters(kind: PayloadKind) -> (&'static AtomicU64, &'static AtomicU64, &'static AtomicU64) {
    match kind {
        PayloadKind::PreparedHandle => (
            &PREPARED_HANDLES_CONSTRUCTED,
            &PREPARED_HANDLES_LIVE,
            &PREPARED_HANDLES_PEAK,
        ),
        PayloadKind::ContainerHandle => (
            &CONTAINER_HANDLES_CONSTRUCTED,
            &CONTAINER_HANDLES_LIVE,
            &CONTAINER_HANDLES_PEAK,
        ),
        PayloadKind::Class => (
            &CLASS_FACTS_CONSTRUCTED,
            &CLASS_FACTS_LIVE,
            &CLASS_FACTS_PEAK,
        ),
        PayloadKind::Container => (
            &CONTAINER_FACTS_CONSTRUCTED,
            &CONTAINER_FACTS_LIVE,
            &CONTAINER_FACTS_PEAK,
        ),
    }
}

#[cfg(any(test, feature = "test-support"))]
fn raise_peak(peak: &AtomicU64, candidate: u64) {
    let mut current = peak.load(Ordering::Relaxed);
    while candidate > current {
        match peak.compare_exchange_weak(current, candidate, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return,
            Err(observed) => current = observed,
        }
    }
}
