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
pub mod model;
pub mod modern;
pub mod multi_release;
pub mod release_registry;
pub mod runtime_matrix;
#[cfg(any(test, feature = "test-support"))]
mod test_fixtures;
pub mod view;
