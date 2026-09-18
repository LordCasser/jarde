//! Attaching an observed usage to a report, which is an operation and not part of the model.
//!
//! It lives apart from the report types on purpose. The facade re-exports the model wholesale —
//! that is how the pre-layering public surface survives — so anything public in `model` becomes
//! public at the crate root as well, and a function that stamps arbitrary accounting onto any
//! report is not a product export: it would hand every consumer a way to state usage the reader
//! never measured, which is exactly what the reader's own contract forbids.
//!
//! Cross-package callers reach it by name, `jarde_reader::accounting::with_usage`, which the
//! facade does not re-export.

use crate::budget::UsageSnapshot;
use crate::model::ExecutionReport;

/// The same execution, carrying the usage the caller observed.
///
/// Every variant keeps its reason and takes the usage verbatim: a run that stopped early is
/// still a run that stopped early, and the counts beside it are what the caller measured.
pub fn with_usage(execution: ExecutionReport, usage: UsageSnapshot) -> ExecutionReport {
    match execution {
        ExecutionReport::Complete { .. } => ExecutionReport::Complete { usage },
        ExecutionReport::Partial { reason, .. } => ExecutionReport::Partial { reason, usage },
        ExecutionReport::Cancelled { .. } => ExecutionReport::Cancelled { usage },
        ExecutionReport::Failed { reason, .. } => ExecutionReport::Failed { reason, usage },
    }
}
