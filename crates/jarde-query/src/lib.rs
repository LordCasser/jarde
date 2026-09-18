//! Bounded query execution and cross-reference scanning over reader-produced facts.
//!
//! This crate owns the query contract (request, report, cursor binding) and the X0/X1 scans
//! that answer it: candidate enumeration, consumer sub-scans, paging and coverage. It reads
//! artifact facts through `jarde-reader` and inherits their budgets, identities and execution
//! semantics; it never decodes a class file itself.
//!
//! The dependency direction is one-way on purpose: nothing here depends on the analysis layers
//! above it (the definition resolver, the CFG/SSA passes, the JVM runtime environment) nor on
//! the facade that composes them. What the resolver needs crosses as a candidate scan — the
//! items one candidate rule found, under the caller's own limit — not as an opened-up scanner:
//! [`query::execute`], [`xref::scan_candidates`] and the two candidate shapes of
//! [`xref::CandidateFilter`] are the whole seam, and the scanner's other filters stay internal.

pub mod query;
pub mod xref;
