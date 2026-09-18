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

pub mod artifact;
pub mod budget;
pub mod classfile;
pub mod error;
pub mod inspect;
pub mod model;
pub mod multi_release;
#[cfg(any(test, feature = "test-support"))]
mod test_fixtures;
pub mod view;
