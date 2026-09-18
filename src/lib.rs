//! Shared, synchronous contracts for bounded JVM artifact analysis.
//!
//! This crate owns bounded artifact I/O together with the stable identities and result
//! semantics consumed by later readers and thin adapters. It deliberately does not depend on
//! CLI, MCP, host protocols, JVM execution, or runtime integration.
//!
//! The three layers below it are re-exported here deliberately, so that a caller does not have
//! to know which crate produces what it names:
//!
//! * `jarde-reader` (P2 1.2) is the input half: artifact snapshots, the class-file facts decoded
//!   from them, the identities derived from those bytes, the budget that bounds a request, and
//!   the inspection entry points. Its module paths (`jarde::artifact`, `jarde::budget`, …) and
//!   the reports their consumers name stay part of this facade's surface — with `classfile` as
//!   the one exception: that layer crosses as the names listed below rather than as a module
//!   path, because the reader's test-only class builder lives below it behind `test-support`,
//!   and a module path would put `jarde::classfile::test_class` back in the surface of every
//!   build that enables that feature (`--all-features`, as CI runs).
//! * `jarde-query` (P2 2.1) is the query half: the request and report schema, the cursor
//!   binding and the X0/X1 scans. It crosses as its product types rather than as a module path:
//!   a nameable `jarde::query`/`jarde::xref` would reach the layer's own cross-crate seams
//!   (`execute` and the candidate scan) that only `jarde-jvm` uses.
//! * `jarde-jvm` (P2 2.2) is the analysis half: the runtime environment, the header providers
//!   and the resolver, the raw CFG, the legacy call contexts and the pass table, and the
//!   method-analysis driver that composes them.
//!
//! What this facade adds on top is [`Engine`], and its entries are one-line delegations to the
//! layer that owns the work. Nothing else crosses it: the layers' own cross-crate seams — the
//! candidate scan (`scan_candidates`, `CandidateScan`, `CandidateFilter`) and the query entry
//! (`execute`) that `jarde-jvm` reaches — are not product types a consumer of this crate names,
//! so the query layer is re-exported as its product types rather than as a module path a caller
//! could reach `execute` and the scanner through. Neither are the mutable analysis internals
//! (`HeaderClosure`, `FactLedger`, `AnalysisRun`, `IrPhase`, `CallContexts`, …) the driver keeps
//! to itself.

pub use jarde_jvm::{environment, ir, resolver};
pub use jarde_reader::inspect;
// `classfile` is deliberately absent: it is re-exported as the names below, not as a module
// path, so that the reader's `test-support` builder (`classfile::test_class`) and everything
// else the module holds beyond that list stay unreachable through this crate.
pub use jarde_reader::{artifact, budget, error, model, multi_release, view};

pub mod facade;

pub use artifact::*;
pub use budget::{
    Budget, BudgetDimension, CancellationToken, CountedBudgetDimension, Limits, UsageSnapshot,
};
pub use environment::*;
pub use error::{Error, Result};
pub use facade::*;
pub use inspect::{ClassSource, ClassTarget, EngineBytecodeReport, EngineHeaderReport};
pub use ir::*;
pub use jarde_query::query::{
    BootstrapVia, ConsumerKind, ConsumerSchema, LiteralValue, QUERY_ENGINE_SCHEMA, QueryAnalysis,
    QueryBoundary, QueryCoverage, QueryCursor, QueryPage, QueryRelation, QueryReport, QueryRequest,
    QueryResolution, QueryTarget, XrefCertainty, XrefDerivation, XrefEvidence, XrefItem,
    XrefOperation, XrefTarget,
};
pub use jarde_reader::classfile::{
    AttributeFacts, AttributeShell, BootstrapMethodFacts, BytecodeInspection, BytecodeStop,
    BytecodeStopPhase, ClassFacts, ClassHeader, ClassfileVersion, ControlFlowTarget,
    ControlFlowTargetKind, CpEntryFacts, CpEntryKind, CpIndexOf, DescriptorKind,
    DialectValidationScope, EnclosingMethodFacts, EntryDescriptor, ExceptionHandlerFact,
    HeaderInspection, HeaderStructuralRead, ImmediateValue, InnerClassFacts, InspectionMode,
    InstructionFact, InstructionOperands, Java8RuntimeCompatibility, LocalOperand, MemberHeader,
    MethodCodeFacts, MethodSelector, ModuleFacts, NestedAttributeFact, ProvidesFacts,
    SwitchOperands, VerificationStatus, VersionCapability, VersionDialectSupport,
    VersionRuleStatus, attribute_content, attribute_facts, attribute_slice, bootstrap_methods,
    class_facts, code_nested_attributes, cp_class_name, cp_entry, cp_utf8, descriptor_types,
    entry_descriptor, inspect_header, inspect_method_bytecode, method_code_coverage,
    method_code_facts, push_unique,
};
pub use model::*;
pub use multi_release::*;
pub use resolver::*;
pub use view::*;
