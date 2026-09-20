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

pub use jarde_jvm::{environment, ir, reflection, resolver};
// The on-demand callee read (P3 3.2) crosses as its report's own types: [`RecoveredMethod::callees`]
// hands one over and an adapter serializes it, so a consumer needs the names to hold. The entry that
// performs the read (`callee::read_callees`) and the candidate type it takes stay below — a consumer
// reaches this read through [`Engine::recover_method`], which enumerates the candidates from the
// body's own decode, and never by naming call sites of its own.
pub use jarde_jvm::callee::{CalleeBody, CalleeMember, CalleeReadReport, CalleeRefusal};
pub use jarde_reader::inspect;
// `classfile` is deliberately absent: it is re-exported as the names below, not as a module
// path, so that the reader's `test-support` builder (`classfile::test_class`) and everything
// else the module holds beyond that list stay unreachable through this crate.
pub use jarde_reader::{artifact, budget, error, model, multi_release, runtime_matrix, view};

// The recovery layer (P3 1.3), narrowed deliberately. What crosses is the *request*, the *report*
// and the read-only vocabulary either one names — never `jarde-java`'s modules, and with them
// never its region tree, its AST, its statement builder, its emitter or its naming table. The
// layer's own read surface for one request is what a consumer needs; the machinery that decides a
// shape is the layer's to keep, so it stays where it is.
//
// `RecoveryRequest`/`recover` are here for a caller that holds a payload of its own (the 1.1
// handoff is `jarde_jvm::method_ir`, which this facade does not publish); a caller that only wants
// a presented method calls [`Engine::recover_method`], which performs the run and hands the same
// run's report to the presentation. `SourceMap` crosses because the map is a first-class output of
// this layer (P3 1.2 decision 2) and 3.2 grows on it; the `Segment`/`Origin`/`OriginSet` types a
// segment is read through stay below, because the map's own read surface answers the questions
// 1.3 poses ("which text came from this BCI", "what covers this byte") without naming them.
pub use jarde_java::{
    IrTable, LambdaCapture, LambdaForm, LambdaRecord, LambdaRefusal, MethodFacts, Precondition,
    RecoveryFacts, RecoveryOutcome, RecoveryProfile, RecoveryReport, RecoveryRequest, RegionRecord,
    RuleVersion, SourceMap, StopReason, recover,
};

pub mod facade;

pub use artifact::*;
pub use budget::{
    Budget, BudgetDimension, CancellationToken, CountedBudgetDimension, Limits, UsageSnapshot,
};
pub use environment::*;
pub use error::{Error, Result};
pub use facade::*;
pub use inspect::{ClassSource, ClassTarget, EngineBytecodeReport, EngineHeaderReport};
// The facts cache (P5 2.3) crosses as the three names a caller needs to switch it on and read what
// it did: the handle a budget is given, the identity a store is read under, and the report that says
// how many lookups were answered, discarded or refused. It is disabled by default and nothing in the
// engine constructs one — `crates/jarde-reader/src/facts_cache.rs` owns the whole mechanism, and the
// guard in `tests/p5_benchmark.rs` holds that split.
pub use ir::*;
pub use jarde_query::query::{
    BootstrapVia, ConsumerKind, ConsumerSchema, LiteralValue, QUERY_ENGINE_SCHEMA, QueryAnalysis,
    QueryBoundary, QueryCoverage, QueryCursor, QueryPage, QueryRelation, QueryReport, QueryRequest,
    QueryResolution, QueryTarget, XrefCertainty, XrefDerivation, XrefEvidence, XrefItem,
    XrefOperation, XrefTarget,
};
pub use jarde_reader::facts_cache::{FACTS_FORMAT, FactsCache, FactsIdentity, FactsReport};
// The versioned plugin plane (P4 3.1) crosses the same way the query layer does: as its product
// types and its registry functions, never as the query layer's module path — the entry point that
// performs a request (`plugin::execute`) stays below and is reached through
// [`Engine::plugins`], exactly like `query::execute` is reached through [`Engine::query`].
pub use jarde_query::plugin::{
    PLUGIN_TRUST_DOMAIN, PLUGINS, PluginAnalysis, PluginBudget, PluginConfigPath,
    PluginInputCategory, PluginItem, PluginOutputSchema, PluginReport, PluginRequest, PluginRule,
    PluginRuleAnalysis, PluginRuleReport, PluginRuleVersion, PluginSelection, PluginSupport,
    PluginValue, plugin_for, plugins,
};
pub use jarde_reader::classfile::{
    AttributeFacts, AttributeShell, BootstrapMethodFacts, BytecodeInspection, BytecodeStop,
    BytecodeStopPhase, ClassFacts, ClassHeader, ClassfileVersion, ControlFlowTarget,
    ControlFlowTargetKind, CpEntryFacts, CpEntryKind, CpIndexOf, DescriptorKind,
    DialectValidationScope, EnclosingMethodFacts, EntryDescriptor, ExceptionHandlerFact,
    HeaderInspection, HeaderStructuralRead, ImmediateValue, InnerClassFacts, InspectionMode,
    InstructionFact, InstructionOperands, Java8RuntimeCompatibility, LocalOperand, MemberHeader,
    MethodCodeFacts, MethodSelector, ModernFeature, ModernOrigin, ModuleFacts, NestedAttributeFact,
    OutputLevel, OutputLevelConflict, OutputLevelStatus, PreviewMarker, ProvidesFacts,
    SwitchOperands, VerificationStatus, VersionCapability, VersionDialectSupport,
    VersionRuleStatus, attribute_content, attribute_facts, attribute_slice, bootstrap_methods,
    class_facts, code_nested_attributes, cp_class_name, cp_entry, cp_utf8, descriptor_types,
    entry_descriptor, inspect_header, inspect_method_bytecode, method_code_coverage,
    method_code_facts, push_unique,
};
// The modern structural facts (P4 1.2) cross the same way `classfile`'s names do: the fact types and
// the one entry point that reads them, never the module path. What a caller gets is record
// components, the deferred constant-dynamic graph, the modern concat sites and the output-level
// answer over them, each carrying the class-file origin it was read from.
pub use jarde_reader::modern::{
    ConcatSite, ConcatStrategy, CondyBudget, CondyCycle, CondyEdge, CondyEdgeKind, CondyGraph,
    CondyNode, CondyNodeKind, CondyNodeRef, CondyReach, CondyStop, CondyUseSite,
    ModernAttributePlacement, ModernFacts, PermittedSubclassesFacts, RecordComponentFacts,
    RecordFacts, modern_facts,
};
pub use jarde_reader::release_registry::{
    AttributePlacement, AttributeRule, ClassfileLocation, ConstantPoolTagRule,
    ConstantPoolTagStatus, FeatureRegistry, FlagPlacement, FlagRule, HIGHEST_REGISTERED_MAJOR,
    IntroducedConstraints, Java8RuntimeRule, MinorForm, OpcodeConstraint, OpcodeRule, OpcodeStatus,
    Placement, PreviewRule, ReleaseBand, ReleaseLookup, ReleaseRecord, ReleaseRegistration,
    feature_registry,
};
pub use model::*;
pub use multi_release::*;
pub use resolver::*;
pub use runtime_matrix::*;
pub use view::*;
