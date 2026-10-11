# Private budget-test candidate v3

This is a private, read-only candidate for `tests/p3_nested_loop_side_exits.rs`.
It renames the existing ForConditionEffects late-stop/precancel case to
`a_for_condition_public_late_stop_reports_dimension_usage_and_location`, logs the
actual public execution report, usage, physical method identity, stop reason and BCI when available,
and retains the existing `assert_late_stop` and `assert_no_partial_body` checks.

For precancellation it compares the execution usage and, for a performed class
report, the report usage after zeroing only `elapsed_millis` against
`UsageSnapshot::default()`. A performed canceled report must have no declaration,
fields, methods, or assembled text. Existing per-body stopped-artifact checks now
also assert `RecoveryContent::NotProduced` alongside empty text and source map.
No budget value or stopped BCI is assumed by the candidate; those are surfaced by
runtime logging. The existing analysis-step dimension check is retained because
this test deliberately limits `analysis_steps`.

Relevant API facts checked read-only:
- `ExecutionReport::{Partial,Cancelled}` each carry a `UsageSnapshot`.
- `RecoveryOutcome::stop()` exposes the `StopReason`; `StopReason::at()` exposes
the optional stopped BCI.
- `ClassSourceReport` carries `usage`, `execution`, `diagnostics`, text, and
physical method outcomes; `OperationOutcome::Incomplete` carries candidates,
execution, and diagnostics but no class report.
- `UsageSnapshot` includes `elapsed_millis`; it is excluded from the zero-count
assertion as requested.

No Rust/Cargo, Java, JADX, or CLI command was run. This candidate has not been
compiled or tested.
