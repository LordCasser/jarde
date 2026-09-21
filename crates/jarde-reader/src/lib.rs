//! Bounded JVM artifact and class-file reading, with no analysis layer above it.
//!
//! This crate owns the input facts and the execution base contracts every later layer consumes:
//! artifact snapshots and their bounded reads, the class-file and method-body facts decoded from
//! them, the identities and result semantics derived from those bytes, the budget that bounds a
//! request, and the inspection entry points that materialize one class inside a snapshot and
//! report on it.
//!
//! The dependency direction is one-way on purpose: nothing here may depend on a consumer of these
//! facts — not on the query layer, not on JVM analysis, and not on the facade that composes them.
//! The public surface is the producer's side of those seams: read-only access to facts this crate
//! produced, and no exported constructor for a state it has not established.

pub mod accounting;
pub mod artifact;
pub mod budget;
pub mod classfile;
pub mod error;
// The facts cache (P5 2.3) is opt-in state a caller attaches to a budget; it is a module of its own
// because it owns an identity, a store and a report, and because the entry points that consult it
// (`classfile::class_facts`, `classfile::inspect_header`) must not be the place that decides what a
// cache is keyed by.
pub mod facts_cache;
pub mod inspect;
// The operation ledger (add-parallel-bulk-recovery 4.1) is the shared total a bulk operation's
// workers bill their work to: one operation, N workers, one set of totals and one published stop,
// with each worker's own budget keeping its local limits. It is a module of its own because it owns
// that state, the stop vocabulary and the arithmetic that folds an entry budget into an operation.
pub mod ledger;
pub mod model;
pub mod modern;
pub mod multi_release;
// The prepared class view (bulk task 2.1) is a lifecycle of one trusted read, not a cache layer: a
// class task holds one for as long as it decodes that class's methods, and every method consumer of
// that class shares it by reference. It is a module of its own because it owns the multi-valued
// method locator and the "one walk, one decoder" invariant the bulk operation's class granularity
// rests on.
pub mod prepared;
pub mod release_registry;
pub mod runtime_matrix;
// The incremental physical traversal cursor (bulk task 3.1) is a discovery handover, not a
// report: it walks the containers a scope holds one entry at a time and hands over one class
// candidate per pull, so a coordinator can keep at most one dispatch window outstanding. It is a
// module of its own because it owns the walk order, the per-subtree "unknown" boundary and the
// cancellation checks, none of which belong to the snapshot's materializing reads.
pub mod scope_cursor;
#[cfg(any(test, feature = "test-support"))]
mod test_fixtures;
pub mod view;
