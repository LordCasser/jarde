//! Shared, synchronous contracts for bounded JVM artifact analysis.
//!
//! This crate owns bounded artifact I/O together with the stable identities and
//! result semantics consumed by later readers and thin adapters. It deliberately
//! does not depend on CLI, MCP, host protocols, JVM execution, or runtime integration.

pub mod artifact;
pub mod budget;
pub mod classfile;
pub mod engine;
pub mod error;
pub mod model;
pub mod query;
pub mod view;

pub use artifact::*;
pub use budget::{
    Budget, BudgetDimension, CancellationToken, CountedBudgetDimension, Limits, UsageSnapshot,
};
pub use classfile::*;
pub use engine::*;
pub use error::{Error, Result};
pub use model::*;
pub use query::*;
pub use view::*;
