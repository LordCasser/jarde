//! Shared, synchronous contracts for bounded JVM artifact analysis.
//!
//! This crate owns bounded artifact I/O together with the stable identities and
//! result semantics consumed by later readers and thin adapters. It deliberately
//! does not depend on CLI, MCP, host protocols, JVM execution, or runtime integration.

pub mod artifact;
pub mod budget;
pub mod classfile;
mod dispatch;
pub mod engine;
pub mod environment;
pub mod error;
pub mod ir;
mod members;
pub mod model;
pub mod multi_release;
mod passes;
mod providers;
pub mod query;
pub mod resolver;
pub mod view;
pub mod xref;

pub use artifact::*;
pub use budget::{
    Budget, BudgetDimension, CancellationToken, CountedBudgetDimension, Limits, UsageSnapshot,
};
pub use classfile::*;
pub use engine::*;
pub use environment::*;
pub use error::{Error, Result};
pub use ir::*;
pub use model::*;
pub use multi_release::*;
pub use query::*;
pub use resolver::*;
pub use view::*;
