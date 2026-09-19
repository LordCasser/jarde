//! The JVM analysis layers: resolution, the runtime environment, the raw CFG, the legacy call
//! contexts and the method-analysis driver that composes them.
//!
//! This crate owns everything a P2 request does above the reader's facts: the runtime
//! environment a request names, the header providers that read a definition under the declared
//! load domain and bind it to the identity it claims, the JVMS 5.4.3 member rules and the
//! open-world dispatch evidence the resolver answers definitions with, the raw CFG and the
//! `jsr`/`ret` call contexts built over a decoded body, the pass table and the ledger that
//! accounts for it, and the IR request/report schema those passes publish through.
//!
//! The dependency direction is one-way on purpose: this crate reads artifact facts through
//! `jarde-reader` and the declaration-reference candidates through `jarde-query`, and it never
//! depends on the facade above it. What a consumer outside can name is the request, the report,
//! the four entry points, and — since P3 1.1 — the read-only IR handoff (`method_ir`) that carries
//! one run's canonical graph, frames and SSA tables to the recovery layer above it, and — since P3
//! 3.2 — the on-demand read of a presented body's named callees (`callee`), which is the fact layer
//! answering "what is the member this call site reaches" without reading a body nothing asked for;
//! the mutable analysis internals — the header closure, the fact ledger, the analysis run, the pass
//! table and the call contexts — stay crate-private, so the facade cannot widen them back into the
//! product surface.
//!
//! `engine` is the one module allowed to call the resolver, the environment and the IR: it is
//! the P2 driver, and everything below it is reachable from outside only through the entry
//! points it publishes.

pub mod callee;
pub mod engine;
pub mod environment;
pub mod ir;
pub mod method_ir;
pub mod resolver;

mod call_context;
mod canonical;
mod cfg;
mod dispatch;
mod frame;
#[cfg(test)]
mod frame_oracle;
#[cfg(test)]
mod ir_audit;
mod members;
mod passes;
mod providers;
mod ssa;
#[cfg(test)]
mod ssa_oracle;
#[cfg(any(test, feature = "test-support"))]
mod test_fixtures;

pub use callee::{
    CalleeBody, CalleeCandidate, CalleeMember, CalleeReadReport, CalleeReadRequest, CalleeRefusal,
    read_callees,
};
pub use engine::{analyze_method, analyze_method_ir, declaration_references, resolve_symbol};
