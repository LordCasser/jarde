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
    RecoveryContent, RecoveryFacts, RecoveryOutcome, RecoveryProfile, RecoveryReport,
    RecoveryRequest, RegionRecord, RuleVersion, SourceMap, StopReason, recover,
};

pub mod bulk;
pub mod class_source;
pub mod facade;
// The D0 1.3 counting port (change `add-demand-driven-core-results`): a bounded, test-support-only
// count of what the demand paths did — class materializations, preparations, body decodes, recovery
// presentations and the owning records this facade built — beside the reader's `PreparedClass`
// lifetime. It is deliberately *not* a report field: nothing here reaches a report, a stop record or
// a fingerprint. A build without the feature keeps the same call sites and counts nothing, so the
// module is a private one there and has no reading API at all.
#[cfg(any(test, feature = "test-support"))]
pub mod d0_counts;
#[cfg(not(any(test, feature = "test-support")))]
mod d0_counts;

pub use artifact::*;
pub use budget::{
    Budget, BudgetDimension, CancellationToken, CountedBudgetDimension, Limits, UsageSnapshot,
};
// The bulk operation (change `add-parallel-bulk-recovery`, stream D): one entry over an explicit
// physical scope, its request/limits/events/report types, and the typed sink it streams to. It is
// exported the way `class_source` is — as a module path a caller may name, and as the product names
// re-exported here so a consumer imports one set of names.
pub use bulk::*;
// The operation ledger (bulk stream B), as the names this facade's own surface needs: the total a
// caller attaches to the budget of one operation, the three work classes a charge is attributed to,
// and the first stop the operation observed — the record `recover_all`'s report publishes. A caller
// that only runs operations the engine starts never names them; one that builds a budget for its own
// operation does, and it should not have to know which crate owns them.
pub use environment::*;
pub use error::{Error, Result};
pub use jarde_reader::ledger::{BulkStop, BulkStopKind, OperationLedger, UsageOwner};
// The class-source presentation (the shortcut 1.1's Non-Goal list excluded and the user asked for):
// the request, the report and the members' records, beside the assembly that spells one class as
// Java text. The module is published the way the facade is — as a path a caller may name — and its
// product types are re-exported here so a consumer of this crate imports one set of names.
pub use class_source::*;
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
    QueryBoundary, QueryCoverage, QueryCursor, QueryPage, QueryPosition, QueryRelation,
    QueryReport, QueryRequest, QueryResolution, QueryTarget, XrefCertainty, XrefDerivation,
    XrefEvidence, XrefItem, XrefOperation, XrefTarget,
};
pub use jarde_reader::facts_cache::{
    CONTAINER_FACTS_SCHEMA, FACTS_FORMAT, FactsCache, FactsCapacity, FactsIdentity, FactsReport,
};

/// The opt-in facts store one operation reads through, as the budget that opened the operation
/// carries it.
///
/// The store is *declared* in `jarde-reader` and *re-exported* two lines above, and this wrapper is
/// the one place the engine reads a caller's handle off a budget: an operation that spans many
/// budgets — the bulk operation builds one per class task and one per method — has to hand the same
/// handle to every part of itself, so that one operation's reads hit one store and a caller that
/// attached none reads exactly as every entry point always did. Keeping that single read here, beside
/// the names the store crosses this facade under, is what lets `tests/p5_benchmark.rs` hold its rule
/// that no second engine file grows a path into the cache
/// (`the_engine_has_one_disabled_facts_cache_and_no_index_or_scheduler`).
///
/// The wrapper constructs nothing and enables nothing: an operation that is handed a budget without
/// a store attaches none to the budgets it builds, and [`Budget::new`] keeps carrying none.
#[derive(Debug)]
pub(crate) struct OperationStore(Option<FactsCache>);

impl OperationStore {
    /// The store `budget` carries, if its caller attached one.
    pub(crate) fn of(budget: &Budget) -> Self {
        Self(budget.facts_cache().cloned())
    }

    /// The retention capacity of that store, or [`FactsCapacity::none`] when there is none.
    pub(crate) fn capacity(&self) -> FactsCapacity {
        match &self.0 {
            Some(store) => store.capacity(),
            None => FactsCapacity::none(),
        }
    }

    /// `budget` reading through this store. A budget with no store attached is handed back
    /// unchanged, so the shared handle never becomes a fresh one.
    pub(crate) fn attached_to(&self, budget: Budget) -> Budget {
        match &self.0 {
            Some(store) => budget.with_facts_cache(store.clone()),
            None => budget,
        }
    }
}
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
    MemberTablePhase, MemberTableStop, MethodCodeFacts, MethodSelector, ModernFeature,
    ModernOrigin, ModuleFacts, NestedAttributeFact, OutputLevel, OutputLevelConflict,
    OutputLevelStatus, PreviewMarker, ProvidesFacts, SwitchOperands, VerificationStatus,
    VersionCapability, VersionDialectSupport, VersionRuleStatus, attribute_content,
    attribute_facts, attribute_slice, bootstrap_methods, class_facts, code_nested_attributes,
    cp_class_name, cp_entry, cp_utf8, descriptor_types, entry_descriptor, inspect_header,
    inspect_method_bytecode, method_code_coverage, method_code_facts, push_unique,
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
