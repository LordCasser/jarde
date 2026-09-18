//! Shared, synchronous contracts for bounded JVM artifact analysis.
//!
//! This crate owns bounded artifact I/O together with the stable identities and
//! result semantics consumed by later readers and thin adapters. It deliberately
//! does not depend on CLI, MCP, host protocols, JVM execution, or runtime integration.
//!
//! The input half of that contract — artifact snapshots, the class-file facts decoded from
//! them, the identities derived from those bytes, the budget that bounds a request, and the
//! inspection entry points — moved to `jarde-reader` in the layering work of P2 1.2, and is
//! re-exported here deliberately: the module paths (`jarde::artifact`, `jarde::classfile`,
//! …) and the inspection reports their consumers name are part of this facade's surface, so a
//! caller does not have to know which crate produces them. The query half — the request and
//! report schema, the cursor binding and the X0/X1 scans — moved to `jarde-query` in the same
//! work (P2 2.1) and is re-exported here for the same reason: `jarde::query` and `jarde::xref`
//! stay the paths their consumers name. What stays here is the analysis above those facts —
//! resolution, the JVM environment, CFG and the pass layers — plus the [`Engine`] entry that
//! delegates to each of them.

mod call_context;
mod cfg;
mod dispatch;
pub mod engine;
pub mod environment;
pub mod ir;
mod members;
mod passes;
mod providers;
pub mod resolver;
#[cfg(any(test, feature = "test-support"))]
mod test_fixtures;

/// The query layer's module paths, kept nameable through this facade.
pub use jarde_query::{query, xref};
/// The reader's inspection entry points: materializing one class and reporting on it.
pub use jarde_reader::inspect;
pub use jarde_reader::{artifact, budget, classfile, error, model, multi_release, view};

pub use artifact::*;
pub use budget::{
    Budget, BudgetDimension, CancellationToken, CountedBudgetDimension, Limits, UsageSnapshot,
};
pub use classfile::*;
pub use engine::*;
pub use environment::*;
pub use error::{Error, Result};
pub use inspect::{ClassSource, ClassTarget, EngineBytecodeReport, EngineHeaderReport};
pub use ir::*;
pub use jarde_query::query::*;
pub use jarde_query::xref::*;
pub use model::*;
pub use multi_release::*;
pub use resolver::*;
pub use view::*;
