//! The facade's [`Engine`]: one line per entry point, delegating to the layer that owns the
//! work.
//!
//! Artifact I/O, enumeration and inspection are the reader's (P2 1.2); the query entry is
//! `jarde-query`'s (P2 2.1); resolution, declaration references and method analysis are
//! `jarde-jvm`'s (P2 2.2). Nothing here implements analysis — the module exists so that a
//! caller has one type to hold and one set of names to import, and so this crate can publish
//! that type without publishing the layers behind it or the mutable internals they keep to
//! themselves.

use crate::class_source::{
    self, ClassSourceDeclaration, ClassSourceField, ClassSourceInitializerField,
    ClassSourceInitializerProof, ClassSourceMethod, ClassSourceReport, ClassSourceRequest,
    ClassSourceRunFacts,
};
use crate::environment::{EnvironmentIdentity, EnvironmentProblem, ResolutionEnvironment};
use crate::ir::{AnalysisStage, NoBodyKind, Quality};
use crate::resolver::{
    DeclarationRefItem, DeclarationRefReport, HeaderRead, ReferenceUse, ResolutionAnalysis,
    ResolutionRequest, ResolutionState, ResolvedMemberRef,
};
use jarde_java::ast::{Expr, ExprKind, Type as JavaType};
use jarde_java::{
    RecoveryContent, RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryReport, StopReason,
    type_of_component,
};
// The artifact binding (change `add-demand-driven-core-results`, D3') crosses the facade here, the
// way the recovery layer's other product names do: a caller reads it off the report it already
// holds — `RecoveredMethod::recovery().artifact()` — and hands the same value back as the
// `expected_artifact` of a later request, so the names have to be nameable without naming
// `jarde-java`'s modules.
pub use jarde_java::{
    ARTIFACT_MISMATCH_CODE, ARTIFACT_SCHEMA, ARTIFACT_UNVERIFIABLE_CODE, ArtifactAgreement,
    ArtifactBinding, ArtifactDimension, ArtifactMismatch, ArtifactSubject, RecoveryArtifact,
    TEXT_DIGEST,
};
// The one reader identity that vocabulary names: a binding states the member **record** the read
// established, and a caller reading it back has to be able to hold that value.
use jarde_query::query::{
    ConsumerKind, ConsumerSchema, QueryAnalysis, QueryCoverage, QueryPage, QueryRelation,
    QueryReport, QueryRequest, XrefDerivation, XrefItem,
};
use jarde_reader::accounting::with_usage;
use jarde_reader::artifact::{
    ArtifactInput, ArtifactKind, ArtifactSnapshot, ArtifactTreeReport, EnumerationReport,
    PhysicalEntry, budget_dimension_code,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension, Limits, UsageSnapshot};
use jarde_reader::classfile::{
    AttributeShell, BytecodeStop, BytecodeStopPhase, ClassMemberFacts, CpEntryFacts, CpEntryKind,
    DescriptorKind, ExceptionHandlerFact, InspectionMode, InstructionFact, MemberHeader,
    MemberTablePhase, MemberTableStop, MethodSelector, attribute_facts, class_constant_pool,
    class_member_facts, descriptor_facts, method_code_coverage,
};
use jarde_reader::error::{Error, Result};
use jarde_reader::inspect::{
    ClassSource, ClassTarget, EngineBytecodeReport, EngineHeaderReport, materialize_definition,
    materialize_root,
};
use jarde_reader::model::{
    ArchiveNameBytes, ByteSpan, ClassBytesId, ContainerId, ContainerOrigin, Coverage,
    CoverageDimension, CoverageRange, CoverageState, Diagnostic, DiagnosticSeverity,
    ExecutionReport, JvmBytes, JvmString, Location, MemberKey, OriginMember, OriginSet,
    PhysicalClassLocation, PhysicalDefinitionId, PhysicalEntryId, PhysicalMemberId,
    PhysicalMethodId, PhysicalVariant, Provenance, SnapshotId, TerminationReason,
    physical_variant_for_path,
};
pub use jarde_reader::prepared::MethodOrdinal;
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, PhysicalScope,
    PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};
use serde::{Deserialize, Serialize};

const ACC_INTERFACE: u16 = 0x0200;
const ACC_ANNOTATION: u16 = 0x2000;
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Default)]
pub struct Engine;

impl Engine {
    pub const fn new() -> Self {
        Self
    }

    pub fn open(&self, input: ArtifactInput, budget: &mut Budget) -> Result<ArtifactSnapshot> {
        ArtifactSnapshot::open(input, budget)
    }

    pub fn enumerate(
        &self,
        snapshot: &ArtifactSnapshot,
        budget: &mut Budget,
    ) -> Result<EnumerationReport> {
        snapshot.enumerate(budget)
    }

    pub fn enumerate_artifact_tree(
        &self,
        snapshot: &ArtifactSnapshot,
        budget: &mut Budget,
    ) -> Result<ArtifactTreeReport> {
        snapshot.enumerate_artifact_tree(budget)
    }

    pub fn select_multi_release(
        &self,
        snapshot: &ArtifactSnapshot,
        view: &crate::view::RuntimeView,
        budget: &mut Budget,
    ) -> Result<crate::multi_release::MultiReleaseViewReport> {
        crate::multi_release::select(snapshot, view, budget)
    }

    /// Compares several runtime profiles over **one** physical scan of the same snapshot.
    ///
    /// The scan, the per-profile selection and the compression all belong to the reader; this is
    /// the facade's one-line delegation to them, so a caller through `Engine` and a caller through
    /// `jarde_reader::runtime_matrix` read the same bytes under the same accounting.
    pub fn runtime_matrix(
        &self,
        snapshot: &ArtifactSnapshot,
        request: &crate::runtime_matrix::RuntimeMatrixRequest,
        budget: &mut Budget,
    ) -> Result<crate::runtime_matrix::RuntimeMatrix> {
        crate::runtime_matrix::build(snapshot, request, budget)
    }

    pub fn query(
        &self,
        snapshot: &ArtifactSnapshot,
        request: &QueryRequest,
        budget: &mut Budget,
    ) -> Result<QueryReport> {
        jarde_query::query::execute(snapshot, request, budget)
    }

    /// Materializes the target class and inspects its header (reader entry point).
    ///
    /// The bounded read and the header inspection are the reader's own steps; this is the
    /// facade's one-line delegation to them, so a caller through `Engine` and a caller through
    /// `jarde_reader::inspect` read the same bytes under the same accounting.
    pub fn inspect_header(
        &self,
        snapshot: &ArtifactSnapshot,
        target: ClassTarget<'_>,
        budget: &mut Budget,
        mode: InspectionMode,
    ) -> Result<EngineHeaderReport> {
        crate::inspect::inspect_header(snapshot, target, budget, mode)
    }

    /// Materializes the target class and inspects the selected method's body (reader entry point).
    ///
    /// Delegated to the reader exactly as [`Engine::inspect_header`] is.
    pub fn inspect_method_bytecode(
        &self,
        snapshot: &ArtifactSnapshot,
        target: ClassTarget<'_>,
        selector: MethodSelector,
        budget: &mut Budget,
    ) -> Result<EngineBytecodeReport> {
        crate::inspect::inspect_method_bytecode(snapshot, target, selector, budget)
    }

    /// Demand-bound symbol resolution under an explicit environment (P2 entry point).
    ///
    /// The request shape is checked first: a snapshot the content does not provide, a target
    /// whose kind contradicts the reference use, or a dispatch range whose tree root cannot
    /// describe this snapshot is an input error (`resolution_snapshot_mismatch`,
    /// `resolution_target_use_mismatch`, `query_artifact_tree_root_mismatch`). Environment
    /// problems are not an error; they are part of the report.
    ///
    /// A class symbol is looked up by name in the declared search order and the selected
    /// definition is reported as `Resolved` / `Missing` / `Ambiguous` (2.1); a member symbol is
    /// resolved by the JVMS 5.4.3 member rules under the invocation-kind and access rules (2.3);
    /// a request that also names a dispatch range (`request.dispatch`) enumerates the known
    /// candidates of that range with their open-world evidence once its member declaration
    /// resolved (2.5) — never a unique runtime target. A request whose environment the validator
    /// rejected keeps the honest unavailable state, because a rejected environment never yields
    /// a definition.
    ///
    /// A member whose hierarchy needs a class this snapshot's order does not provide is
    /// `UnresolvedDependency` rather than `Missing`, and the classes it could not read are
    /// published by name in `unresolved_dependencies`: reading nothing for a name is not the
    /// statement that the name does not exist (2.2, A11).
    pub fn resolve_symbol(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::resolver::ResolutionRequest,
        budget: &mut Budget,
    ) -> Result<crate::resolver::ResolutionReport> {
        jarde_jvm::resolve_symbol(content, request, budget)
    }

    /// Declaration-reference scan under an explicit environment (P2 entry point).
    ///
    /// Same request-level check as [`Engine::resolve_symbol`]. The query scans the explicit
    /// scope for candidate use sites with the structure consumers, resolves every candidate's
    /// owner, and publishes only the candidates that resolve to the requested declaration;
    /// candidates no search could decide are reported as unresolved instead of excluded, and a
    /// rejected environment keeps the honest unavailable state.
    pub fn declaration_references(
        &self,
        content: &[ArtifactSnapshot],
        query: &crate::resolver::DeclarationRefQuery,
        budget: &mut Budget,
    ) -> Result<crate::resolver::DeclarationRefReport> {
        jarde_jvm::declaration_references(content, query, budget)
    }

    /// Bounded reflection and `ServiceLoader` pattern scan (P4 2.3 entry point).
    ///
    /// Same request-level checks as [`Engine::resolve_symbol`]. The scan enumerates the classes of
    /// the explicit scope, analyses the bodies that name a registered overload, and answers one
    /// site per call site: a site whose target inputs are constants this engine proves under the
    /// bounded value flow of its own body is `pattern_inferred_target` with the odd overload, the
    /// constant input, the propagation scope, the loader assumption and the rule version, while
    /// every other input is `Unknown` with the definition that kept it from being a constant.
    ///
    /// Nothing is executed, loaded or initialized, and the target is never decoded or resolved as
    /// a member, so a name the snapshot does not even hold is still inferred — and published with
    /// the snapshot's own separate answer beside it, so "what the call site asks for" and "what
    /// this snapshot provides" are never one claim. A refused charge, a stopped listing and range
    /// the scan could not read all end it with the prefix it published (`has_more`), never with a
    /// completed answer over range it never searched.
    pub fn reflection_patterns(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::reflection::ReflectionPatternRequest,
        budget: &mut Budget,
    ) -> Result<crate::reflection::ReflectionPatternReport> {
        jarde_jvm::reflection_patterns(content, request, budget)
    }

    /// Versioned framework/resource plugin scan (P4 3.1/3.2 entry point).
    ///
    /// Same request-level shape checks as the query entry: a request that enables no rule
    /// (`plugin_no_rules`), a physical view of another snapshot (`plugin_snapshot_mismatch`) or a
    /// tree root this snapshot cannot have (`plugin_artifact_tree_root_mismatch`) is an input error
    /// before anything is read.
    ///
    /// Every enabled rule is answered **by id and version** against the registered table
    /// ([`crate::plugins`]): a configuration this registry does not hold — at the id or at the
    /// version — is `Unsupported` with the code that says which, never an empty item list, because
    /// "this engine does not read that configuration" and "that configuration declares nothing" are
    /// different statements. A performed rule publishes derived facts, each stamped with the rule,
    /// the rule version, the entry it was read from and the range inside that entry, beside the
    /// rule's own coverage; a budget refusal keeps that prefix, names the dimension that refused and
    /// the entries it did not read, and never becomes a wrong answer. The P1/P2 structural facts are
    /// untouched by all of it: a plugin has no `QueryRelation`, no `XrefItem` and no X1 edge, and
    /// the generic structural scan stays target-driven and cursor-bound next to it (P4 decision 4).
    ///
    /// Nothing is executed, loaded or fetched: a plugin reads the authorized snapshot input through
    /// this engine's budgeted read face and reports the names a configuration spells — a name this
    /// snapshot does not even hold is still published as declared. A plugin is ordinary in-process
    /// code and this entry point claims no sandbox for it ([`crate::PLUGIN_TRUST_DOMAIN`]): an
    /// untrusted extension would need a separate process or Wasm, under its own change.
    pub fn plugins(
        &self,
        snapshot: &ArtifactSnapshot,
        request: &crate::PluginRequest,
        budget: &mut Budget,
    ) -> Result<crate::PluginReport> {
        jarde_query::plugin::execute(snapshot, request, budget)
    }

    /// Method IR analysis under an explicit environment (P2 entry point).
    ///
    /// An empty stage set is an input error (`analysis_no_stages`); every other mismatch
    /// is checked like [`Engine::resolve_symbol`]. The requested stages are then validated
    /// against the fixed pass table *before* anything runs: a schedule the table cannot
    /// serve is an input error (`ir_pass_prerequisite_missing`, `ir_pass_order_invalid`,
    /// `ir_pass_graph_cycle`, `ir_stale_fact`) rather than a half-initialized pipeline.
    ///
    /// The scheduled passes of 3.x–4.x then really run, in table order and through the ledger:
    /// `raw_facts` reads the driver method's class header and decodes its body (one
    /// `ClassHeaders` and one `MethodBodies` attempt, recorded under `DriverMethodBody`),
    /// `raw_cfg` builds the raw graph, its throw sites and its effect facts over those decoded
    /// facts, `legacy_normalization` builds the `jsr`/`ret` call contexts over that graph,
    /// `canonical_cfg` normalizes the graph under exactly those contexts, `frame` derives the
    /// operand stack and local state of every block it reaches, and `ssa` names the stack and
    /// local slots of those frames through one explicit worklist. Every phase of the fixed table
    /// is implemented, so a legal request is answered with the whole pipeline performed; a
    /// stopped pass keeps every stage result it had already produced, and a phase this build does
    /// not implement is `Failed { ir_pass_not_implemented }` wherever the pipeline reaches it —
    /// which no phase of today's table is.
    pub fn analyze_method(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::ir::MethodAnalysisRequest,
        budget: &mut Budget,
    ) -> Result<crate::ir::MethodAnalysisReport> {
        jarde_jvm::analyze_method(content, request, budget)
    }

    /// One recovery request: the method analysis of `request`, performed **once**, and the
    /// presentation of the payload of that same run (P3 1.3).
    ///
    /// This is the one place a P3 consumer outside `jarde-java` reaches recovery, and it is why the
    /// entry exists here rather than as a second library call: [`jarde_jvm::analyze_method_ir`] runs
    /// the pipeline and hands over the tables it published *beside* that run's own report, so the
    /// presentation reads the very decode the run performed instead of a second analysis of the same
    /// bytes. Nothing is run twice, and no table is rebuilt from the report.
    ///
    /// The request is the P2 one — environment, member and stages — and the recovery profile is
    /// *not* a second field: it is the environment's own runtime profile
    /// (`environment.runtime.profile`), which is the one fact the gate reads
    /// ([`jarde_java::pass::Pass::admits`]). A second field naming a profile would be a second
    /// source for one fact.
    ///
    /// The facts the recovery layer needs and the payload does not carry are derived here, and where
    /// the run's own evidence does not reach, this entry states *nothing* rather than a guess:
    ///
    /// * the member's **declaration** — its access flags, its raw name and descriptor, and how many
    ///   local slots its parameters occupy (`this` included for an instance member, a `long`/`double`
    ///   counting two) — is read from the payload's own [`jarde_jvm::method_ir::MethodDeclaration`],
    ///   which the `raw_facts` pass filled from the **same header read** that decoded the body. No
    ///   second read of the class is performed and no table is rebuilt from the report (P3 3.1);
    /// * when the run located no member header at all (it stopped before `raw_facts`, or the member
    ///   declares no body), the only facts left are the identity the request named: the member's name
    ///   and descriptor as the request spells them, and **no** parameter-slot count — the honest
    ///   reading of a run that read no declaration, and the one case where a slot below the
    ///   signature's is named by its ordinal as a local.
    ///
    /// The member's **debug names** travel the same way: they are the `LocalVariableTable` of the
    /// `Code` entry the run already decoded (P3 3.1), so `jarde-reader` reads it inside that one
    /// decode — no second slice of the bytes, no second `AttributeBytes` charge. A slot the table
    /// names once takes that name; a slot it names twice (two variables sharing one reused slot, in
    /// disjoint scopes) takes none, because one storage location has one name and neither record's
    /// name is the whole truth about it.
    ///
    /// A body with no debug metadata is presented with deterministic ordinal names (A10) rather than
    /// refused: nothing is invented for it, and the ordinals it gets are the ones the declaration's
    /// parameter slots and the body's own slots state.
    ///
    /// **One class, one read, one preparation** (D2 3.1/3.3). The definition this request names is
    /// read once — by [`jarde_jvm::read_method_class`], which performs exactly the read
    /// [`jarde_jvm::analyze_method_ir`] performs before it runs, or adopts the read a caller's own
    /// binding already performed — and prepared once, and that one preparation is what the run
    /// ([`jarde_jvm::analyze_prepared_method_ir`]) and the presentation's callee read consume. A
    /// body whose call sites name members of the same class therefore costs no second class read,
    /// and the loader's binding check is *not* skipped: the prepared run performs it over the
    /// declared order, from the facts the read established.
    pub fn recover_method(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::ir::MethodAnalysisRequest,
        budget: &mut Budget,
    ) -> Result<RecoveredMethod> {
        self.recover_bound_method(
            content,
            request,
            None,
            &RecoveryEvidenceRequest::essential(),
            budget,
        )
    }

    /// The same request as [`Engine::recover_method`], with the optional evidence the caller wants
    /// delivered stated explicitly (change `add-demand-driven-core-results`, D1).
    ///
    /// This is the *same* entry — one binding, one run, one presentation — with the selection the
    /// caller states instead of the ordinary default: the run performs every check and writes the
    /// same artifact whatever is selected, and the selection decides which optional detail records
    /// are materialized, which of them a driver BCI range restricts, and what
    /// [`RecoveredMethod::recovery`]'s evidence status list then states.
    pub fn recover_method_with_evidence(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::ir::MethodAnalysisRequest,
        evidence: &RecoveryEvidenceRequest,
        budget: &mut Budget,
    ) -> Result<RecoveredMethod> {
        self.recover_bound_method(content, request, None, evidence, budget)
    }

    /// One recovery request whose target binding already read the class it selected (D2 3.1/3.3).
    ///
    /// `binding` is the trusted read the caller's own binding performed for this request's own
    /// definition, when it performed one: a name search reads the definitions it examines, so the
    /// definition it elects has already been read, and this is that read. The operation prepares the
    /// class **once** over it and runs over that preparation, which the same-class callee read then
    /// consumes too — so the selected definition is materialized once, prepared once, and read by no
    /// consumer of this request again.
    ///
    /// A caller with no binding read (the identity path, which binds from the caller's own identity)
    /// lets the run perform the request's own read
    /// ([`jarde_jvm::analyze_method_ir_owning_the_read`]) and hands *that* read to the callee
    /// consumer, which prepares it exactly when the presented body really names members of the same
    /// class. The loader binding check, the profile decision and every stop stay exactly where they
    /// were: they are the run's own, decided over the declared order from the facts the read
    /// established, and no path skips them.
    fn recover_bound_method(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::ir::MethodAnalysisRequest,
        binding: Option<&ConfirmedRead>,
        evidence: &RecoveryEvidenceRequest,
        budget: &mut Budget,
    ) -> Result<RecoveredMethod> {
        // The binding's own read, stated once more in the shape a class task consumes: adopting it
        // is what keeps this operation from reading the definition its binding selected again.
        let bound_read = match binding {
            Some(bound) => match definition_snapshot(content, &request.method.owner) {
                Some(snapshot) => Some(bound.prepared_read(
                    snapshot,
                    jarde_reader::prepared::ContainerHandover::Keep,
                    budget,
                )?),
                // The content the binding read from is not provided to this request: the run states
                // that, in the vocabulary its own read uses for it.
                None => return self.recover_own_read(content, request, evidence, budget),
            },
            None => None,
        };
        let Some(read) = bound_read else {
            return self.recover_own_read(content, request, evidence, budget);
        };
        // One prepared class over one materialization (`crate::d0_counts`): the run and the callee
        // read below consume this one, and the definition the binding selected is read by neither.
        crate::d0_counts::class_prepared();
        let prepared = jarde_reader::prepared::PreparedClass::prepare(&read, budget)?;
        let analyzed = jarde_jvm::analyze_prepared_method_ir(content, &prepared, request, budget)?;
        if analyzed.ir().code().is_some() {
            // A decode was published, so this demand path really decoded one body
            // (`crate::d0_counts`): counted after the run, so a stop before `raw_facts` counts none.
            crate::d0_counts::body_decoded();
        }
        recovery_presented(
            content,
            request,
            analyzed,
            Some(&prepared),
            evidence,
            budget,
        )
    }

    /// One recovery request whose class the run reads itself, with that read handed to the
    /// presentation (D2 3.1/3.3).
    ///
    /// [`jarde_jvm::analyze_method_ir_owning_the_read`] is [`Engine::recover_method`]'s own run —
    /// the same validation, the same environment check, the same passes, the same report and the
    /// same charges as [`jarde_jvm::analyze_method_ir`] — returning the trusted read the run's
    /// driver class was read as. The presentation's same-class callee read prepares *that* read when
    /// the presented body names members of the same class, so the class is read once for the whole
    /// request and a body that names no such member prepares nothing at all.
    fn recover_own_read(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::ir::MethodAnalysisRequest,
        evidence: &RecoveryEvidenceRequest,
        budget: &mut Budget,
    ) -> Result<RecoveredMethod> {
        let run = jarde_jvm::analyze_method_ir_owning_the_read(content, request, budget)?;
        let (analyzed, read) = run.into_parts();
        if analyzed.ir().code().is_some() {
            crate::d0_counts::body_decoded();
        }
        if read.as_ref().is_some_and(|read| !read.retained()) {
            // One class materialization for this operation's own selected definition
            // (`crate::d0_counts`): the run above performed it, and the presentation below consumes
            // it rather than reading the definition again. A read the request's store answered from
            // retention is not a materialization this request performed.
            crate::d0_counts::class_materialized();
        }
        recovery_read(content, request, analyzed, read, evidence, budget)
    }

    /// One whole physical scope recovered in one operation: every class it holds prepared once,
    /// every method of every prepared class recovered, in physical order (change
    /// `add-parallel-bulk-recovery`, tasks 3.x and 4.x).
    ///
    /// The work is [`crate::bulk::recover_all`]'s; this is the facade's one-line delegation to it,
    /// exactly as the other entries delegate to the layer that owns their work. The request, the
    /// limits, the streamed events and the report are [`crate::bulk`]'s own types.
    ///
    /// Nothing here is reachable from the single-method entries, and none of them reaches it: this
    /// is the one entry that starts workers, and it starts them only when the request asks for more
    /// than one.
    pub fn recover_all(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::bulk::BulkRecoveryRequest,
        budget: &mut Budget,
        sink: &mut dyn crate::bulk::RecoverySink,
    ) -> Result<crate::bulk::BulkRecoveryReport> {
        crate::bulk::recover_all(content, request, budget, sink)
    }

    /// Lists the class **candidates** a physical scope holds, and the ordinary resources beside them.
    ///
    /// The first, non-reading evidence level of class navigation. The scope's entries are enumerated
    /// (the same enumeration `Engine::enumerate` and `Engine::enumerate_artifact_tree` perform, under
    /// the same accounting) and partitioned by one raw-name rule: an entry whose raw name ends in a
    /// case-sensitive `.class` is a [`ClassListingItem::ClassCandidate`], and every other stored entry
    /// is a [`ClassListingItem::Resource`]. A standalone `CLASS` snapshot has no entries at all, so its
    /// single item is the standalone root itself and the resource side is empty.
    ///
    /// **This listing reads no class header** and therefore charges no `class_headers`: it states what
    /// the *names* in the scope look like, which is exactly why it must not be read as "this scope
    /// holds these classes". A path is not a declaration — a `.class` entry whose bytes are not a
    /// class file is still a candidate here, and only the header-confirmed listing
    /// ([`Engine::list_class_declarations`]) reads it and reports what it really holds. `resource` is
    /// the physical remainder of the same partition ("a stored entry no class-name rule presents as a
    /// class candidate" — a resource, a directory, a nested library, a manifest) and claims nothing
    /// about what that entry holds: this change does not parse a resource, does not interpret it and
    /// does not claim it was understood.
    ///
    /// A stop — the enumeration itself stopping on the budget, on a cancellation or on a damaged
    /// container, or this listing's own item charge being refused — leaves every item published
    /// before it, a diagnostic that names the failure, ranges that mark the unscanned remainder and a
    /// non-`Complete` execution.
    pub fn list_class_candidates(
        &self,
        snapshot: &ArtifactSnapshot,
        scope: &PhysicalScope,
        budget: &mut Budget,
    ) -> Result<ClassCandidateListing> {
        let scan = scan_scope(snapshot, scope, budget)?;
        let partition = scope_partition(snapshot.kind(), snapshot.id(), &scan.entries)?;
        let total = to_u64(partition.len())?;
        let mut items = Vec::new();
        let mut diagnostics = scan.diagnostics;
        let mut execution = scan.execution;
        for item in partition {
            if let Err(error) = charge_item(budget) {
                let provenance = item_provenance(&item, &scan.entries);
                merge_execution(&mut execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, provenance));
                break;
            }
            items.push(item);
        }
        let published = to_u64(items.len())?;
        let complete = matches!(execution, ExecutionReport::Complete { .. });
        Ok(ClassCandidateListing {
            view: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: scope.clone(),
            },
            items,
            coverage: listing_coverage(
                &scan.physical_coverage,
                "class_candidates",
                published,
                total,
                complete,
            ),
            execution: with_usage(execution, budget.usage()),
            diagnostics,
        })
    }

    /// Lists the class **declarations** a physical scope holds, reading each candidate's own header.
    ///
    /// The second, reading evidence level: every candidate of [`Engine::list_class_candidates`] is
    /// really read — one `class_headers` attempt and one materialized class per candidate, the
    /// attempt charged before the read — and every item carries the facts *that read* established:
    /// the definition the bytes are (location, class bytes digest and length, syntactic variant), the
    /// class's own `this_class` and access flags, its superclass and interfaces, whether the entry's
    /// raw path and the declared name agree, and where the same read's member walk stopped when a
    /// member record did not decode ([`ClassDeclarationItem::member_table`]). A candidate whose
    /// declaration was not read never appears in [`ClassDeclarationListing::items`]: the listing
    /// publishes the candidates it confirmed and names every candidate it did not in
    /// [`ClassDeclarationListing::unconfirmed`].
    ///
    /// One failed read ends the scan, as one damaged candidate ends the query scan: a candidate whose
    /// bytes do not decode, a refused charge or a cancellation publishes the prefix confirmed before
    /// it, a diagnostic carrying that candidate's physical origin, the unconfirmed remainder, ranges
    /// that mark the candidates never reached and a non-`Complete` execution (`Failed` for a structure
    /// this read could not read, `Partial` for an exhausted dimension, `Cancelled` for a
    /// cancellation).
    ///
    /// Two findings are *not* failures, because neither makes the class unreadable:
    ///
    /// * a damaged **member** record stops that class's member walk only — the class is confirmed, the
    ///   stop is recorded on the item and reported as a diagnostic, and the scan continues, because a
    ///   damaged member does not erase a class (task 2.1's own rule);
    /// * a path whose stated internal name and the header's own `this_class` disagree keeps both
    ///   names side by side ([`ClassNameBinding`]) with a `navigation_path_name_mismatch` diagnostic,
    ///   and the scan continues, because which name is "right" is not this layer's to decide.
    ///
    /// Nothing here reads a method body, runs the version gate or builds an analysis fact: for the
    /// version and dialect planes of these same bytes, call [`Engine::inspect_header`].
    pub fn list_class_declarations(
        &self,
        snapshot: &ArtifactSnapshot,
        scope: &PhysicalScope,
        budget: &mut Budget,
    ) -> Result<ClassDeclarationListing> {
        let scan = scan_scope(snapshot, scope, budget)?;
        let candidates = scope_class_candidates(snapshot.kind(), snapshot.id(), &scan.entries);
        let total = to_u64(candidates.len())?;
        let mut items = Vec::new();
        let mut diagnostics = scan.diagnostics;
        let mut execution = scan.execution;
        let mut attempted = 0_u64;
        for candidate in &candidates {
            let provenance = candidate_provenance(candidate);
            if let Err(error) = budget.charge(CountedBudgetDimension::ClassHeaders, 1) {
                merge_execution(&mut execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, provenance));
                break;
            }
            attempted = attempted.saturating_add(1);
            match read_class_declaration(snapshot, candidate, budget) {
                Ok(read) => {
                    let ClassContentItem::ClassDeclaration(class) = read.class else {
                        unreachable!("a class declaration read publishes a class declaration item")
                    };
                    if let Err(error) = charge_item(budget) {
                        merge_execution(&mut execution, stop_execution(&error, budget));
                        diagnostics.push(stop_diagnostic(&error, provenance));
                        break;
                    }
                    let mismatch = item_path_mismatch_diagnostic(&class);
                    items.push(class);
                    match publish_diagnostics(
                        mismatch.into_iter().chain(read.diagnostics).collect(),
                        &mut diagnostics,
                        budget,
                    ) {
                        Ok(()) => {}
                        Err(error) => {
                            merge_execution(&mut execution, stop_execution(&error, budget));
                            diagnostics.push(stop_diagnostic(&error, provenance));
                            break;
                        }
                    }
                }
                Err(error) => {
                    merge_execution(&mut execution, stop_execution(&error, budget));
                    diagnostics.push(stop_diagnostic(&error, provenance));
                    break;
                }
            }
        }
        let complete = matches!(execution, ExecutionReport::Complete { .. });
        let unconfirmed = candidates[items.len()..]
            .iter()
            .map(|candidate| candidate.location.clone())
            .collect();
        Ok(ClassDeclarationListing {
            view: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: scope.clone(),
            },
            candidates: total,
            items,
            unconfirmed,
            coverage: listing_coverage(
                &scan.physical_coverage,
                "class_declarations",
                attempted,
                total,
                complete,
            ),
            execution: with_usage(execution, budget.usage()),
            diagnostics,
        })
    }

    /// Lists one class's own members from **one** bounded read of the definition the caller holds.
    ///
    /// The class is addressed by the physical identity a confirmed listing handed back — the
    /// [`PhysicalDefinitionId`] of [`ClassDeclarationItem::definition`] — and this entry reads exactly
    /// those bytes: the definition's location, the entry's own coordinates and derived variant, and
    /// the class bytes' digest are all verified before anything is parsed, so a listed identity is
    /// directly usable and never needs reassembly by the caller. One header read attempt — charged
    /// as `class_headers` before the read, like every other class read — then materializes those
    /// bytes, and one bounded read (`jarde_reader::classfile::class_member_facts`) states the class's
    /// own declaration and walks its field and method tables, publishing
    ///
    /// * the class-level facts of that same read, as [`ClassContentItem::ClassDeclaration`];
    /// * one [`ClassContentItem::Field`] per field record — its position in that table, its own raw
    ///   name and descriptor as the read decoded them, its access flags and a [`PhysicalMemberId`]
    ///   whose owner is the definition read;
    /// * one [`ClassContentItem::Method`] per method record — the same facts plus a
    ///   [`PhysicalMethodId`] a method request consumes directly, and what the member's own
    ///   declaration says about its body: [`MemberBodyEvidence::CodeAttribute`] when the declaration
    ///   carries a `Code` shell (whose **content was not read**) or
    ///   [`MemberBodyEvidence::NoCodeAttribute`], which is how an `abstract` or `native` member is
    ///   stated and never given an empty body.
    ///
    /// **No body is read and no analysis runs here**: no attribute content is decoded, `code_bytes`
    /// and `method_bodies` stay at zero, and no resolver, CFG, SSA, region or Java AST is constructed.
    /// A member record that does not decode does not erase the class: the read stops there, the
    /// members before it stay usable, [`MemberListing::stopped_at`] states where it stopped, coverage
    /// marks the members it never reached and a diagnostic carries the class's physical origin. The
    /// class-level item is published first and charged like any other item; when its own raw path does
    /// not state the name the class declares, that finding travels with it as a
    /// `navigation_path_name_mismatch` diagnostic, exactly as the confirmed listing states it. A
    /// refusal of the read itself — an exhausted budget, a cancellation, a definition this snapshot
    /// does not hold or whose bytes are not the ones it names — is an `Err`, because nothing was
    /// established to publish.
    pub fn list_members(
        &self,
        snapshot: &ArtifactSnapshot,
        definition: &PhysicalDefinitionId,
        budget: &mut Budget,
    ) -> Result<MemberListing> {
        // One header read attempt, charged before the read like every other class read in this
        // engine: the dimension counts what it is asked to count, and a refused charge means the read
        // never happened, which is an `Err` here rather than a prefix.
        budget.charge(CountedBudgetDimension::ClassHeaders, 1)?;
        let (bytes, source, _retained) = materialize_definition(snapshot, definition, budget)?;
        let facts = class_member_facts(&bytes, budget)?;
        let provenance = Some(definition_provenance(definition));
        let mut items = Vec::new();
        let mut diagnostics = Vec::new();
        let mut execution: Option<ExecutionReport> = None;
        let class_item = ClassContentItem::ClassDeclaration(ClassDeclarationItem {
            definition: definition.clone(),
            declaration: declaration_facts(
                &facts.this_class,
                facts.access_flags,
                &facts.super_class,
                &facts.interfaces,
            ),
            binding: class_name_binding(&source, &facts.this_class),
            member_table: facts.stopped_at.clone(),
        });
        match charge_item(budget) {
            Ok(()) => {
                if let ClassContentItem::ClassDeclaration(class) = &class_item
                    && let Some(diagnostic) = item_path_mismatch_diagnostic(class)
                {
                    diagnostics.push(diagnostic);
                }
                items.push(class_item);
            }
            Err(error) => {
                merge_execution_option(&mut execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, provenance.clone()));
            }
        }
        if execution.is_none() {
            for (index, field) in facts.fields.iter().enumerate() {
                match charge_item(budget) {
                    Ok(()) => items.push(field_item(definition, index, field)?),
                    Err(error) => {
                        merge_execution_option(&mut execution, stop_execution(&error, budget));
                        diagnostics.push(stop_diagnostic(&error, provenance.clone()));
                        break;
                    }
                }
            }
        }
        if execution.is_none() {
            for (index, method) in facts.methods.iter().enumerate() {
                match charge_item(budget) {
                    Ok(()) => items.push(method_item(definition, index, method)?),
                    Err(error) => {
                        merge_execution_option(&mut execution, stop_execution(&error, budget));
                        diagnostics.push(stop_diagnostic(&error, provenance.clone()));
                        break;
                    }
                }
            }
        }
        if let Some(stop) = &facts.stopped_at {
            merge_execution_option(&mut execution, member_stop_execution(stop, budget));
            diagnostics.push(member_stop_diagnostic(definition, stop));
        }
        let execution = execution.unwrap_or(ExecutionReport::Complete {
            usage: budget.usage(),
        });
        let complete = matches!(execution, ExecutionReport::Complete { .. });
        Ok(MemberListing {
            definition: definition.clone(),
            items,
            stopped_at: facts.stopped_at.clone(),
            coverage: member_coverage(&facts, complete),
            execution: with_usage(execution, budget.usage()),
            diagnostics,
        })
    }

    /// Answers one navigation name search over a physical scope, in the caller's own spelling.
    ///
    /// The one entry that turns a *friendly* name into a physical identity. The request names a class
    /// in either accepted spelling ([`ClassNameQuery`]) and may narrow the answer to one member name,
    /// descriptor and kind ([`MemberQuery`]). The scope's entries whose raw path states the requested
    /// internal name at a `/` boundary — the path without its `.class` suffix *is* that name or ends
    /// with `/<name>`, so a class under any container prefix is found without this layer claiming a
    /// prefix or a root — are then read, with the same one `class_headers` attempt per candidate and
    /// the same read as the confirmed listing. A candidate is a [`ClassContentItem::ClassDeclaration`]
    /// only when the header's own `this_class` states the requested name; a candidate that declares
    /// another name is not a binding for it, and becomes a `navigation_path_name_mismatch` diagnostic
    /// (carrying that entry's physical origin, the name its path states and the name it declares)
    /// instead, because a path is not a declaration.
    ///
    /// **Several definitions of one name, or several overloads of one member, are all returned.** Each
    /// item carries its own physical identity — the definition's location and class bytes for a class,
    /// the owner definition plus the declared raw name and descriptor for a member — and that identity,
    /// never the order the scan walked or the spelling the caller typed, is the basis for choosing
    /// between them. Feeding a chosen identity back ([`Engine::list_members`], a method request) reads
    /// that definition and nothing else, wherever the artifact stores it.
    ///
    /// No match is an answer, not a failure: the candidate list is empty, the coverage states the range
    /// that was searched, the execution is `Complete` when the whole scope was searched, and no
    /// definition, member or diagnostic is invented. Two cases end the search early, with the items it
    /// published before it and a non-`Complete` execution: a candidate whose read *fails* (a diagnostic
    /// carrying that entry's origin), and — for a query that names a member — a candidate whose own
    /// member table stopped, whose records past the stop were never read.
    pub fn find_targets(
        &self,
        snapshot: &ArtifactSnapshot,
        scope: &PhysicalScope,
        query: &NavigationQuery,
        budget: &mut Budget,
    ) -> Result<NavigationReport> {
        let search = search_targets(snapshot, scope, query, budget)?;
        Ok(NavigationReport {
            view: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: scope.clone(),
            },
            query: query.clone(),
            candidates: search.items,
            coverage: search.coverage,
            execution: search.execution,
            diagnostics: search.diagnostics,
        })
    }

    /// Answers the class view of one artifact: the class's declaration, its fields and its methods
    /// from **one** read, and the method bodies the request asked for, on demand (task 3.1).
    ///
    /// The class is addressed the task way ([`ClassRef`]): a friendly name searched with the
    /// navigation rules over this scope, or a physical definition used exactly as given. The read
    /// that answers a friendly name is the same read the view publishes — one `class_headers`
    /// attempt, one member walk — and the bodies are decoded out of the very bytes that read
    /// materialized, one `method_bodies` attempt each, so a view of a class with N requested bodies
    /// charges one class header, one member listing and N bodies and never re-reads the class per
    /// method. A member that declares no `Code` is stated as such and charges no body attempt.
    ///
    /// Nothing runs the analysis pipeline: no resolver, CFG, SSA, region or Java AST is built, and
    /// `ir_items`/`ir_edges`/`analysis_steps`/`normalization_clones` stay at zero. What a body read
    /// publishes is the reader's own decode of that body — its two phases, the instructions, the
    /// handlers, the coverage of that decode and that decode's own stop.
    ///
    /// A request whose name matches more than one physical definition — a class name held at
    /// several origins, or a body name with several declared descriptors — is answered with
    /// [`OperationOutcome::Ambiguous`]: every candidate keeps its own physical identity and no body
    /// is decoded. A friendly name whose *search* did not finish — a candidate that did not read,
    /// an exhausted dimension, a cancellation — is answered with
    /// [`OperationOutcome::Incomplete`]: the confirmed prefix (one candidate, or none), the search's
    /// own planes and nothing decoded, because an unfinished search has shown neither a unique
    /// definition nor a missing one. A body identity that belongs to another class, or a name the
    /// class does not declare, is an input error (see [`ClassViewReport`]'s own codes).
    ///
    /// The bodies are independent results of one request: a body whose own decode stopped, or that
    /// the member walk never reached, keeps its own result, coverage and diagnostics while the view
    /// goes on to the bodies the caller asked for beside it. The one thing that ends the request
    /// rather than the value is the shared budget: a cancellation or an exhausted dimension stops
    /// the view from starting any further body, and the prefix it published stays as it is.
    pub fn class_view(
        &self,
        snapshot: &ArtifactSnapshot,
        scope: &PhysicalScope,
        request: &ClassViewRequest,
        budget: &mut Budget,
    ) -> Result<OperationOutcome<ClassViewReport>> {
        let view = PhysicalView {
            snapshot: snapshot.id().clone(),
            scope: scope.clone(),
        };
        let mut execution = ExecutionReport::Complete {
            usage: budget.usage(),
        };
        let mut diagnostics = Vec::new();
        let (read, search_coverage, class_item) = match bind_class(
            snapshot,
            scope,
            &request.class,
            &mut execution,
            &mut diagnostics,
            budget,
        )? {
            ClassBinding::Bound(bound) => (bound.read, bound.search_coverage, bound.class_item),
            ClassBinding::Ambiguous(candidates) => {
                return Ok(OperationOutcome::Ambiguous(candidates));
            }
            ClassBinding::Incomplete(candidates) => {
                return Ok(OperationOutcome::Incomplete(candidates));
            }
        };
        let ClassContentItem::ClassDeclaration(declaration) = &read.class else {
            unreachable!("a class read publishes a class declaration item")
        };
        let definition = declaration.definition.clone();
        let mut items = Vec::new();
        if let Some(class_item) = class_item {
            items.push(class_item);
        } else {
            // The search confirmed the class but could not publish its own item (a refused
            // `result_items` charge): the view keeps the identity and the stop, and publishes no
            // member either, exactly like the listing entries whose item charge was refused.
            return Ok(OperationOutcome::Performed(ClassViewReport {
                view,
                class: definition,
                items,
                bodies: Vec::new(),
                limits: budget.limits().clone(),
                usage: budget.usage(),
                coverage: class_view_coverage(search_coverage.as_ref(), &read.facts, false),
                execution: with_usage(execution, budget.usage()),
                diagnostics,
            }));
        }
        let class_provenance = Some(definition_provenance(&definition));
        // The member items of the same read: fields in declaration order, then methods, each
        // charged as the item it is, exactly like the member listing that publishes them.
        let mut refused = false;
        for (index, field) in read.facts.fields.iter().enumerate() {
            if let Err(error) = charge_item(budget) {
                merge_execution(&mut execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                refused = true;
                break;
            }
            items.push(field_item(&definition, index, field)?);
        }
        if !refused {
            for (index, method) in read.facts.methods.iter().enumerate() {
                if let Err(error) = charge_item(budget) {
                    merge_execution(&mut execution, stop_execution(&error, budget));
                    diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    refused = true;
                    break;
                }
                items.push(method_item(&definition, index, method)?);
            }
        }
        // The class and member planes are complete on their own evidence: the search and the class
        // read reached their ends and the member table was read to its declared end. Taken here,
        // before any body runs, so that a body which stopped below cannot rewrite what the member
        // table really was (A13/A14).
        let structure_complete = matches!(execution, ExecutionReport::Complete { .. })
            && read.facts.stopped_at.is_none();
        // The bodies the request asked for: every reference is resolved against this one listing
        // before any body is decoded, so a name that matches several descriptors answers with the
        // candidates and reads nothing, and the published results keep the request's own order.
        let mut bodies = Vec::new();
        if !refused {
            let mut resolutions = Vec::new();
            for body in &request.bodies {
                match resolve_body_ref(&read, body)? {
                    BodyResolution::Method(method, ordinal, member) => {
                        resolutions.push(BodyResolution::Method(method, ordinal, member));
                    }
                    BodyResolution::NotReached(stop) => {
                        resolutions.push(BodyResolution::NotReached(stop));
                    }
                    BodyResolution::Ambiguous { query, candidates } => {
                        return Ok(OperationOutcome::Ambiguous(Box::new(TargetCandidates {
                            query,
                            candidates,
                            limits: budget.limits().clone(),
                            coverage: class_view_coverage(
                                search_coverage.as_ref(),
                                &read.facts,
                                false,
                            ),
                            execution: with_usage(execution, budget.usage()),
                            diagnostics,
                        })));
                    }
                }
            }
            // The one preparation every requested body is decoded against (D2 3.2): the class's
            // declaration, constant pool, member table and locator are read **once** — out of the
            // very bytes the binding read (task 3.1), never by reading the definition again — and
            // every body the request asked for is located and decoded against them, so a body does
            // not parse the class a second time.
            //
            // It is made exactly when this view really decodes a body: a body the class declares
            // without a `Code` entry, a reference the member table never reached and a body that
            // stopped are not decodes of this class, and preparing it for them would charge a read
            // nothing consumes. A preparation that fails — the reader's strict structure read
            // refusing bytes the tolerant class read accepted — does not erase the class either:
            // the declaration, the fields and every member stay published, and each body that would
            // have been decoded states that failure as its own refusal.
            let requested_bodies = resolutions.iter().any(|resolution| match resolution {
                BodyResolution::Method(_, _, member) => code_shell(member).is_some(),
                BodyResolution::NotReached(_) | BodyResolution::Ambiguous { .. } => false,
            });
            let mut prepared_read = None;
            let mut preparation_failure = None;
            if requested_bodies {
                match read.prepared_read(
                    snapshot,
                    jarde_reader::prepared::ContainerHandover::NotNeeded,
                    budget,
                ) {
                    Ok(value) => prepared_read = Some(value),
                    Err(error) => preparation_failure = Some(error),
                }
            }
            let prepared = match &prepared_read {
                Some(value) => {
                    // One prepared class over one materialization (`crate::d0_counts`): a view that
                    // prepared the class per body would prepare N.
                    crate::d0_counts::class_prepared();
                    match jarde_reader::prepared::PreparedClass::prepare(value, budget) {
                        Ok(class) => Some(class),
                        Err(error) => {
                            preparation_failure = Some(error);
                            None
                        }
                    }
                }
                None => None,
            };
            let bodies_state = match (&prepared, preparation_failure) {
                (Some(prepared), _) => ViewBodies::Prepared(prepared),
                (None, Some(error)) => ViewBodies::Refused(error),
                // Unreachable by construction: `requested_bodies` is the one predicate that decides
                // whether this view prepares anything and the one that sends a member here, so a
                // member with a `Code` entry in this loop belongs to a class it prepared.
                (None, None) => ViewBodies::NotNeeded,
            };
            for (position, resolution) in resolutions.into_iter().enumerate() {
                let reference = request
                    .bodies
                    .get(position)
                    .expect("one resolution per requested body");
                let body = match resolution {
                    BodyResolution::Method(method, ordinal, member) => {
                        match body_result(
                            &definition,
                            &bodies_state,
                            ordinal,
                            &member,
                            &method,
                            reference,
                            budget,
                        ) {
                            Ok(body) => body,
                            Err(error) => {
                                merge_execution(&mut execution, stop_execution(&error, budget));
                                diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                                break;
                            }
                        }
                    }
                    BodyResolution::NotReached(stop) => ClassViewBody::Refused {
                        reference: reference.clone(),
                        method: None,
                        execution: member_stop_execution(&stop, budget),
                        diagnostics: vec![member_stop_diagnostic(&definition, &stop)],
                    },
                    BodyResolution::Ambiguous { .. } => {
                        unreachable!("an ambiguity returned before any body was published")
                    }
                };
                if let Err(error) = charge_item(budget) {
                    merge_execution(&mut execution, stop_execution(&error, budget));
                    diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    break;
                }
                // The body's own plane joins the view's: a body that stopped makes the view's
                // execution non-`Complete` while the body keeps its own result, coverage and
                // diagnostics. A no-body declaration is not a stop and contributes nothing.
                let stop_ahead = match &body {
                    ClassViewBody::Read {
                        execution: body_execution,
                        ..
                    }
                    | ClassViewBody::Refused {
                        execution: body_execution,
                        ..
                    } => {
                        let stop_ahead = ends_the_request(body_execution);
                        merge_execution(&mut execution, body_execution.clone());
                        stop_ahead
                    }
                    ClassViewBody::NotDeclared { .. } => false,
                };
                bodies.push(body);
                // A cancellation or an exhausted shared dimension is the request ending: the bodies
                // after it are never begun, and the prefixes published stay as they are.
                if stop_ahead {
                    break;
                }
            }
        }
        if let Some(stop) = &read.facts.stopped_at {
            merge_execution(&mut execution, member_stop_execution(stop, budget));
        }
        Ok(OperationOutcome::Performed(ClassViewReport {
            view,
            class: definition,
            items,
            bodies,
            limits: budget.limits().clone(),
            usage: budget.usage(),
            coverage: class_view_coverage(
                search_coverage.as_ref(),
                &read.facts,
                structure_complete,
            ),
            execution: with_usage(execution, budget.usage()),
            diagnostics,
        }))
    }

    /// Analyzes one method the task way (task 1.2): the target and the environment are the
    /// request's, the stage set is the operation's own table, and the report publishes both.
    ///
    /// The one physical identity is bound by [`bind_method`] — a friendly name searched with the
    /// navigation rules, or an identity used exactly as given — and the run is
    /// [`Engine::analyze_method`] with the operation's stage list: the same validation, the same
    /// schedule, the same stop semantics. The report publishes the stage list it passed, so a
    /// caller can reproduce the schedule with an explicit
    /// [`crate::ir::MethodAnalysisRequest`] and read the same stage results.
    pub fn analyze_target(
        &self,
        content: &[ArtifactSnapshot],
        request: &MethodOperationRequest,
        budget: &mut Budget,
    ) -> Result<OperationOutcome<MethodOperationReport>> {
        let operation = MethodOperation::Analysis;
        let bound = match bind_method(content, request, budget)? {
            MethodBinding::Bound(bound) => bound,
            MethodBinding::Ambiguous(candidates) => {
                return Ok(OperationOutcome::Ambiguous(candidates));
            }
            MethodBinding::Incomplete(candidates) => {
                return Ok(OperationOutcome::Incomplete(candidates));
            }
        };
        // The analysis-only operation has no consumer beside its run, so the read the search
        // performed is the search's own and the run reads the class as every analysis request does.
        let BoundMethod {
            method,
            environment,
            read: _,
        } = *bound;
        let stages = operation.stages().to_vec();
        let analysis = jarde_jvm::analyze_method(
            content,
            &crate::ir::MethodAnalysisRequest {
                environment,
                method: method.clone(),
                stages: stages.clone(),
            },
            budget,
        )?;
        Ok(OperationOutcome::Performed(MethodOperationReport {
            operation,
            method,
            stages,
            limits: budget.limits().clone(),
            usage: budget.usage(),
            analysis,
        }))
    }

    /// Recovers one method the task way (task 1.2, task 3.4): the target and the environment are the
    /// request's, the stage set is the operation's own table, and the same run's report is presented
    /// content-first.
    ///
    /// The run is [`Engine::recover_method`] under the operation's stage list — one analysis run and
    /// the presentation of that run's own payload — so the analysis report, the recovery report, the
    /// on-demand callee evidence and the [`RecoveryPresentation`] all describe one request. The
    /// report publishes the stage list and the effective configuration beside them.
    pub fn recover_target(
        &self,
        content: &[ArtifactSnapshot],
        request: &MethodOperationRequest,
        budget: &mut Budget,
    ) -> Result<OperationOutcome<MethodRecoveryReport>> {
        self.recover_target_with_evidence(
            content,
            request,
            &RecoveryEvidenceRequest::essential(),
            budget,
        )
    }

    /// The same operation as [`Engine::recover_target`], with the optional evidence the caller wants
    /// delivered stated explicitly (change `add-demand-driven-core-results`, D1).
    pub fn recover_target_with_evidence(
        &self,
        content: &[ArtifactSnapshot],
        request: &MethodOperationRequest,
        evidence: &RecoveryEvidenceRequest,
        budget: &mut Budget,
    ) -> Result<OperationOutcome<MethodRecoveryReport>> {
        let operation = MethodOperation::Recovery;
        let bound = match bind_method(content, request, budget)? {
            MethodBinding::Bound(bound) => bound,
            MethodBinding::Ambiguous(candidates) => {
                return Ok(OperationOutcome::Ambiguous(candidates));
            }
            MethodBinding::Incomplete(candidates) => {
                return Ok(OperationOutcome::Incomplete(candidates));
            }
        };
        let BoundMethod {
            method,
            environment,
            read,
        } = *bound;
        let stages = operation.stages().to_vec();
        let recovered = self.recover_bound_method(
            content,
            &crate::ir::MethodAnalysisRequest {
                environment,
                method: method.clone(),
                stages: stages.clone(),
            },
            read.as_ref(),
            evidence,
            budget,
        )?;
        let presentation = RecoveryPresentation::of(recovered.recovery());
        Ok(OperationOutcome::Performed(MethodRecoveryReport {
            operation,
            method,
            stages,
            limits: budget.limits().clone(),
            usage: budget.usage(),
            recovered,
            presentation,
        }))
    }

    /// Presents one class as Java source: its declaration, its fields and every member's body.
    ///
    /// The class is bound the task way, by the same rules and with the same checks the method
    /// operations apply ([`bind_class`]): a friendly name searched with the navigation rules over the
    /// environment's scope, or a physical definition used exactly as given — and a definition of
    /// another artifact is `operation_target_snapshot_mismatch` before the environment is built,
    /// never a same-named substitute of this snapshot. A name that several definitions answer to is
    /// [`OperationOutcome::Ambiguous`] and presents nothing; a search that did not finish is
    /// [`OperationOutcome::Incomplete`].
    ///
    /// **Nothing here reads or recovers on its own.** The class's declaration and member tables are
    /// [`Engine::class_view`]'s own read — one `class_headers` attempt and one member walk over one
    /// materialized definition — and every member body is one analysis run under
    /// [`MethodOperation::Recovery`]'s stage table with the presentation of that same run's payload,
    /// charged in the dimensions that run charges. What this entry adds is the assembly of those
    /// results into one class text ([`crate::class_source`]) and the report that publishes the facts
    /// beside the spelling.
    ///
    /// **One preparation, however many members the class has** (task 7.3, D2 3.2). The class is
    /// prepared once — [`jarde_reader::prepared::PreparedClass::prepare`], the reader's own once-read
    /// class task — over the read the binding performed, and every member body is decoded against
    /// that one preparation, through [`jarde_jvm::analyze_prepared_method_ir`] and
    /// [`jarde_jvm::callee::read_prepared_callees`], which is the prepared half of exactly the run
    /// [`Engine::recover_method`] performs. A member's run therefore charges no class header and no
    /// class bytes of its own: it charges its `method_bodies` attempt and the decode dimensions, and
    /// the callee evidence its call sites justify comes from the same prepared class.
    ///
    /// **What one request costs.** One class header and one member walk for the class-binding read,
    /// one preparation *over that same read* — made exactly when the class declares at least one
    /// member this presentation would run a body for, and never a second read of the definition
    /// (D2 task 3.2) — and then one body attempt per member that declares one. Neither
    /// `class_headers` nor the class's bytes are read twice: a class with `N` bodies costs one
    /// materialization, one preparation and `N` decodes. A member that declares no body charges
    /// nothing, a member this presentation cannot spell is never run, and no class is read for a
    /// member that is not presented.
    ///
    /// A preparation that fails — the reader's own strict structure read refusing bytes the tolerant
    /// class read accepted — does not erase the class: the declaration, the fields and every member
    /// declaration stay presented, and each member that declares a body states that failure as its
    /// own refusal, under the reader's own code. That is the same answer such a member's own run gives
    /// today (the run's read is the same strict read), stated once per member instead of once per run.
    ///
    /// **A member's failure is that member's.** A run that stops, is refused, or produces no
    /// artifact leaves its own result in [`ClassSourceReport::methods`] and the members beside it are
    /// still presented (A13), exactly as a class view keeps the bodies beside a stopped one. What it
    /// cannot do is let the report claim to be `Complete`: every stop a member's run published is
    /// merged into [`ClassSourceReport::execution`]. A stop that is the *request's* — a cancellation
    /// or an exhausted dimension — ends it, and the members never reached are stated by the report's
    /// planes (its coverage's skipped range, its execution and its diagnostics) rather than
    /// presented.
    ///
    /// The text is a presentation and not a claim of compilability: [`ClassSourceReport::text`] is
    /// deterministic for one request and one budget, carries no timestamp, and marks every member it
    /// could not present in full (see [`crate::class_source`]).
    pub fn class_source(
        &self,
        content: &[ArtifactSnapshot],
        request: &ClassSourceRequest,
        budget: &mut Budget,
    ) -> Result<OperationOutcome<ClassSourceReport>> {
        self.class_source_with_evidence(
            content,
            request,
            &RecoveryEvidenceRequest::essential(),
            budget,
        )
    }

    /// The same presentation as [`Engine::class_source`], with the optional evidence the caller
    /// wants delivered stated explicitly (change `add-demand-driven-core-results`, D1).
    ///
    /// The assembled class text does not depend on the selection: what changes is which optional
    /// detail records each member's own recovery run materializes, and what the evidence status list
    /// of that run states about them.
    pub fn class_source_with_evidence(
        &self,
        content: &[ArtifactSnapshot],
        request: &ClassSourceRequest,
        evidence: &RecoveryEvidenceRequest,
        budget: &mut Budget,
    ) -> Result<OperationOutcome<ClassSourceReport>> {
        // The identity the caller gave is checked against the request's own physical view before the
        // environment is built, exactly as `bind_method` checks it: a foreign identity is an input
        // error of the request and must not be hidden behind a policy problem of a declaration the
        // caller would then fix for nothing.
        let snapshot_id = request.environment.snapshot.clone();
        if let ClassRef::Definition { definition } = &request.class
            && definition.snapshot() != &snapshot_id
        {
            return Err(Error::invalid_input(
                "operation_target_snapshot_mismatch",
                format!(
                    "the definition names snapshot `{}` while this request reads `{}`; an identity \
                     of another artifact is never replaced by a same-named definition of this one",
                    definition.snapshot().0,
                    snapshot_id.0
                ),
            ));
        }
        let environment = request.environment.build(content)?;
        let Some(snapshot) = content
            .iter()
            .find(|candidate| candidate.id() == &snapshot_id)
        else {
            return Err(snapshot_not_provided(&snapshot_id));
        };
        let view = PhysicalView {
            snapshot: snapshot_id.clone(),
            scope: environment.runtime.physical.scope.clone(),
        };
        let stages = MethodOperation::Recovery.stages().to_vec();
        let mut execution = ExecutionReport::Complete {
            usage: budget.usage(),
        };
        let mut diagnostics = Vec::new();
        let bound = match bind_class(
            snapshot,
            &view.scope,
            &request.class,
            &mut execution,
            &mut diagnostics,
            budget,
        )? {
            ClassBinding::Bound(bound) => bound,
            ClassBinding::Ambiguous(candidates) => {
                return Ok(OperationOutcome::Ambiguous(candidates));
            }
            ClassBinding::Incomplete(candidates) => {
                return Ok(OperationOutcome::Incomplete(candidates));
            }
        };
        let (mut report, family_scan) = self.prepare_physical_class_source(
            content,
            request,
            evidence,
            &environment,
            snapshot,
            view,
            stages,
            *bound,
            execution,
            diagnostics,
            budget,
        )?;
        report.member_family = match family_scan {
            crate::member_inner::FamilyRootScan::Absent => {
                class_source::ClassSourceMemberFamily::Absent
            }
            crate::member_inner::FamilyRootScan::Refused(reason) => {
                class_source::ClassSourceMemberFamily::Refused {
                    reason,
                    child: None,
                }
            }
            crate::member_inner::FamilyRootScan::Candidate(candidate) => {
                let root_name = report
                    .declaration
                    .as_ref()
                    .expect("candidate requires a published root declaration")
                    .item
                    .declaration
                    .this_class
                    .raw()
                    .0
                    .clone();
                match self.prepare_class_source_member_family(
                    content,
                    request,
                    evidence,
                    &environment,
                    snapshot,
                    &report,
                    &root_name,
                    &candidate,
                    budget,
                ) {
                    Ok((family, family_execution)) => {
                        merge_execution(&mut report.execution, family_execution);
                        family
                    }
                    Err(error) => {
                        merge_execution(&mut report.execution, stop_execution(&error, budget));
                        report.diagnostics.push(stop_diagnostic(
                            &error,
                            Some(definition_provenance(&report.class)),
                        ));
                        class_source::ClassSourceMemberFamily::Refused {
                            reason: "member family preparation stopped".to_owned(),
                            child: None,
                        }
                    }
                }
            }
        };
        if matches!(
            report.member_family,
            class_source::ClassSourceMemberFamily::Prepared { .. }
        ) {
            let mut projection_execution = ExecutionReport::Complete {
                usage: budget.usage(),
            };
            let projected = project_class_source_member_family(
                content,
                &environment,
                &report,
                &mut projection_execution,
                budget,
            );
            merge_execution(&mut report.execution, projection_execution);
            match projected {
                Ok(Ok((text, derived))) => {
                    report.text = text;
                    if let class_source::ClassSourceMemberFamily::Prepared { projection, .. } =
                        &mut report.member_family
                    {
                        *projection =
                            class_source::ClassSourceMemberProjection::Projected { derived };
                    }
                }
                Ok(Err(reason)) => {
                    if let class_source::ClassSourceMemberFamily::Prepared { projection, .. } =
                        &mut report.member_family
                    {
                        *projection = class_source::ClassSourceMemberProjection::Refused { reason };
                    }
                }
                Err(error) => {
                    merge_execution(&mut report.execution, stop_execution(&error, budget));
                    report.diagnostics.push(stop_diagnostic(
                        &error,
                        Some(definition_provenance(&report.class)),
                    ));
                    if let class_source::ClassSourceMemberFamily::Prepared { projection, .. } =
                        &mut report.member_family
                    {
                        *projection = class_source::ClassSourceMemberProjection::Refused {
                            reason: format!("family source projection stopped: {error}"),
                        };
                    }
                }
            }
        }
        report.usage = budget.usage();
        report.execution = with_usage(report.execution, budget.usage());
        Ok(OperationOutcome::Performed(report))
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_class_source_member_family(
        &self,
        content: &[ArtifactSnapshot],
        request: &ClassSourceRequest,
        evidence: &RecoveryEvidenceRequest,
        environment: &ResolutionEnvironment,
        snapshot: &ArtifactSnapshot,
        root_report: &ClassSourceReport,
        root_name: &[u8],
        candidate: &crate::member_inner::FamilyRootCandidate,
        budget: &mut Budget,
    ) -> Result<(class_source::ClassSourceMemberFamily, ExecutionReport)> {
        use class_source::ClassSourceMemberFamily as Family;
        let mut child_execution = ExecutionReport::Complete {
            usage: budget.usage(),
        };
        let Some((child_definition, child_read)) = resolve_class_source_dependency_read_raw(
            content,
            environment,
            None,
            &candidate.child_name,
            &mut child_execution,
            budget,
        )?
        else {
            return Ok((
                Family::Refused {
                    reason: "selected environment did not uniquely resolve the member definition"
                        .to_owned(),
                    child: None,
                },
                child_execution,
            ));
        };
        let relation = (|| -> Result<bool> {
            let pool = class_constant_pool(&child_read.bytes, budget)?;
            let shells: Vec<_> = child_read
                .facts
                .attributes
                .iter()
                .filter(|attribute| {
                    matches!(
                        attribute.name.raw().0.as_slice(),
                        b"InnerClasses" | b"EnclosingMethod"
                    )
                })
                .cloned()
                .collect();
            let nesting = class_source::read_class_source_assembly_context(
                &child_read.bytes,
                &shells,
                &pool,
                budget,
            )?;
            crate::member_inner::child_relation_agrees(
                root_name,
                candidate,
                &child_read.facts,
                &nesting,
                &pool,
                budget,
            )
        })();
        if let Err(error) = &relation {
            merge_execution(&mut child_execution, stop_execution(error, budget));
        }
        let child_class_item = match charge_item(budget) {
            Ok(()) => Some(child_read.class.clone()),
            Err(error) => {
                merge_execution(&mut child_execution, stop_execution(&error, budget));
                None
            }
        };
        let mut child_diagnostics = Vec::new();
        if let Err(error) = publish_diagnostics(
            child_read.diagnostics.clone(),
            &mut child_diagnostics,
            budget,
        ) {
            merge_execution(&mut child_execution, stop_execution(&error, budget));
            child_diagnostics.push(stop_diagnostic(
                &error,
                Some(definition_provenance(&child_definition)),
            ));
        }
        let child_facts = child_read.facts.clone();
        let child_request = ClassSourceRequest {
            class: ClassRef::Definition {
                definition: child_definition.clone(),
            },
            environment: request.environment.clone(),
        };
        let (child, _) = self.prepare_physical_class_source(
            content,
            &child_request,
            evidence,
            environment,
            snapshot,
            root_report.view.clone(),
            root_report.stages.clone(),
            BoundClass {
                read: child_read,
                search_coverage: None,
                class_item: child_class_item,
            },
            child_execution,
            child_diagnostics,
            budget,
        )?;
        let child = Box::new(child);
        let physically_complete = matches!(root_report.execution, ExecutionReport::Complete { .. })
            && matches!(child.execution, ExecutionReport::Complete { .. });
        let mut capture_execution = ExecutionReport::Complete {
            usage: budget.usage(),
        };
        let capture = if matches!(relation, Ok(true)) && physically_complete {
            match prove_class_source_member_capture(
                content,
                environment,
                &child_definition,
                root_name,
                &child_facts,
                &mut capture_execution,
                budget,
            ) {
                Ok(capture) => capture,
                Err(error) => {
                    merge_execution(&mut capture_execution, stop_execution(&error, budget));
                    class_source::ClassSourceMemberCapture::Refused {
                        reason: "capture proof stopped".to_owned(),
                    }
                }
            }
        } else {
            class_source::ClassSourceMemberCapture::Refused {
                reason: "physical family relation or preparation is incomplete".to_owned(),
            }
        };
        let calls = match (&relation, &capture) {
            (Ok(true), class_source::ClassSourceMemberCapture::Proved { proof })
                if physically_complete =>
            {
                match prove_class_source_member_calls(
                    content,
                    environment,
                    root_report,
                    &child,
                    root_name,
                    candidate,
                    proof,
                    &mut capture_execution,
                    budget,
                ) {
                    Ok(calls) => calls,
                    Err(error) => {
                        merge_execution(&mut capture_execution, stop_execution(&error, budget));
                        class_source::ClassSourceMemberCalls::Refused {
                            reason: "member call proof stopped".to_owned(),
                            sites: Vec::new(),
                            refusals: Vec::new(),
                        }
                    }
                }
            }
            _ => class_source::ClassSourceMemberCalls::Refused {
                reason: "family relation or capture proof is incomplete".to_owned(),
                sites: Vec::new(),
                refusals: Vec::new(),
            },
        };
        let family = match relation {
            Ok(true) if physically_complete => Family::Prepared {
                relation: class_source::ClassSourceMemberRelation {
                    root: root_report.class.clone(),
                    child: child_definition,
                    simple_name: candidate.simple_name.clone(),
                    access_flags: candidate.access_flags,
                },
                child,
                capture,
                calls,
                projection: class_source::ClassSourceMemberProjection::Refused {
                    reason: "family source projection has not completed".to_owned(),
                },
            },
            Ok(true) => Family::Refused {
                reason: "root or child physical preparation did not complete".to_owned(),
                child: Some(child),
            },
            Ok(false) => Family::Refused {
                reason: "selected child has no unique matching InnerClasses self row or has EnclosingMethod identity".to_owned(),
                child: Some(child),
            },
            Err(error) => Family::Refused {
                reason: match error {
                    Error::BudgetExceeded { .. } | Error::Cancelled { .. } => {
                        "selected child relation proof stopped"
                    }
                    _ => "selected child nesting attributes could not be read",
                }
                .to_owned(),
                child: Some(child),
            },
        };
        let execution = match &family {
            Family::Prepared { child, .. }
            | Family::Refused {
                child: Some(child), ..
            } => child.execution.clone(),
            _ => unreachable!("resolved child remains in family result"),
        };
        let mut execution = execution;
        merge_execution(&mut execution, capture_execution);
        Ok((family, execution))
    }

    /// Prepare one already bound physical definition. Family assembly calls this sequentially for
    /// each selected definition with the same request budget; no public operation is re-entered.
    #[allow(clippy::too_many_arguments)]
    fn prepare_physical_class_source(
        &self,
        content: &[ArtifactSnapshot],
        request: &ClassSourceRequest,
        evidence: &RecoveryEvidenceRequest,
        environment: &ResolutionEnvironment,
        snapshot: &ArtifactSnapshot,
        view: PhysicalView,
        stages: Vec<AnalysisStage>,
        bound: BoundClass,
        mut execution: ExecutionReport,
        mut diagnostics: Vec<Diagnostic>,
        budget: &mut Budget,
    ) -> Result<(ClassSourceReport, crate::member_inner::FamilyRootScan)> {
        let BoundClass {
            read,
            search_coverage,
            class_item,
        } = bound;
        let ClassContentItem::ClassDeclaration(item) = read.class.clone() else {
            unreachable!("a class read publishes a class declaration item")
        };
        let definition = item.definition.clone();
        let mut declaration = ClassSourceDeclaration::of(item);
        let annotation_shells: Vec<AttributeShell> = read
            .facts
            .attributes
            .iter()
            .filter(|attribute| {
                matches!(
                    attribute.name.raw().0.as_slice(),
                    b"RuntimeVisibleAnnotations" | b"RuntimeInvisibleAnnotations"
                )
            })
            .cloned()
            .collect();
        // The class and member planes are complete on their own evidence, taken before any member is
        // run, so that a member which stopped below cannot rewrite what the member table really was
        // (A13/A14) — the same rule the class view applies to its bodies.
        let structure_complete = matches!(execution, ExecutionReport::Complete { .. })
            && read.facts.stopped_at.is_none();
        if class_item.is_none() {
            // The class confirmed its own item and could not publish it (a refused `result_items`
            // charge): the report keeps the identity and the stop and presents nothing at all, which
            // is what a report that could not pay for its own declaration may say.
            return Ok((
                ClassSourceReport {
                    view,
                    class: definition,
                    declaration: None,
                    stages,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    member_family: class_source::ClassSourceMemberFamily::Absent,
                    bridge_proofs: Vec::new(),
                    enum_switch_proofs: Vec::new(),
                    initializer_proof: if read.facts.access_flags & ACC_INTERFACE != 0
                        && read.facts.access_flags & ACC_ANNOTATION == 0
                    {
                        ClassSourceInitializerProof::Refused {
                        reason: "the class declaration was not published, so its interface initializer group cannot be proved".to_owned(),
                    }
                    } else {
                        ClassSourceInitializerProof::NotApplicable
                    },
                    enum_constant_proof:
                        crate::enum_constants::ClassSourceEnumConstantProof::NotApplicable,
                    enum_constant_body_relations: Vec::new(),
                    text: String::new(),
                    limits: budget.limits().clone(),
                    usage: budget.usage(),
                    coverage: class_view_coverage(search_coverage.as_ref(), &read.facts, false),
                    execution: with_usage(execution, budget.usage()),
                    diagnostics,
                },
                crate::member_inner::FamilyRootScan::Absent,
            ));
        }
        let class_provenance = Some(definition_provenance(&definition));
        // How many members this presentation could run a body for at all: one that declares a `Code`
        // attribute and whose descriptor this presentation can read. A member outside that set is a
        // declaration without a body — a declaration, never work left undone — or a member whose
        // bytes are not a method descriptor at all (see [`ClassSourceReport::coverage`]).
        let declared_bodies = to_u64(
            read.facts
                .methods
                .iter()
                .filter(|member| class_source_runs_body(member))
                .count(),
        )?;
        // The one preparation every member body is decoded against (task 7.3, D2 3.2): the class's
        // declaration, constant pool, member table and locator read once, by the reader's own
        // prepared-class lifecycle, over the very bytes the class binding read — the read this
        // request already performed, handed over as the read a class task consumes (task 3.1), so
        // the selected definition is materialized once and not once per consumer. It is made
        // exactly when this presentation would run some member's body — a class whose members all
        // declare no body is presented without one.
        //
        // A preparation that could not be made is not the request ending: the failure is kept and
        // becomes the refusal of every member that declares a body, so the class, its fields and its
        // member declarations are still presented beside a reader code that says why no body of it
        // could be decoded.
        let mut preparation: Option<Error> = None;
        let prepared_read = if declared_bodies > 0 {
            // The container is kept: every member's own run reads it again for the loader's binding
            // query, and this read is what keeps that directory from being parsed per member.
            match read.prepared_read(
                snapshot,
                jarde_reader::prepared::ContainerHandover::Keep,
                budget,
            ) {
                Ok(read) => Some(read),
                Err(error) => {
                    preparation = Some(error);
                    None
                }
            }
        } else {
            None
        };
        let prepared = match &prepared_read {
            Some(read) => {
                // One prepared class over one materialization (`crate::d0_counts`). The gate that
                // holds "one preparation per presentation" reads this count: a path that prepares
                // the same read twice would prepare two.
                crate::d0_counts::class_prepared();
                match jarde_reader::prepared::PreparedClass::prepare(read, budget) {
                    Ok(prepared) => Some(prepared),
                    Err(error) => {
                        preparation = Some(error);
                        None
                    }
                }
            }
            None => None,
        };
        let bodies = match (&prepared, preparation) {
            (Some(prepared), _) => ClassBodies::Prepared(prepared),
            (None, Some(error)) => {
                let stop = stop_execution(&error, budget);
                ClassBodies::Refused(Box::new(ClassBodyRefusal {
                    ends: ends_the_request(&stop),
                    diagnostic: stop_diagnostic(&error, class_provenance.clone()),
                    stop,
                }))
            }
            (None, None) => ClassBodies::NotNeeded,
        };
        let capture_enum_group_code = match &bodies {
            ClassBodies::Prepared(prepared) => crate::enum_constants::may_capture_group_code(
                prepared.class_facts(),
                prepared.member_table_stop().is_none(),
            ),
            ClassBodies::Refused(_) | ClassBodies::NotNeeded => false,
        };
        let mut constructor_count = 0_usize;
        let mut has_no_arg_enum_constructor = false;
        let mut has_int_enum_constructor = false;
        if capture_enum_group_code {
            // This is an eligibility check over the one prepared member table already read for the
            // class. The class-header read billed every physical member record; the enum proof
            // below bills its complete table walk. Poll each record here, and charge the two
            // selected constructor identities only when this exact pair enables AST retention.
            for method in &read.facts.methods {
                budget.poll()?;
                if method.name.raw().0 == b"<init>" {
                    constructor_count += 1;
                    has_no_arg_enum_constructor |=
                        method.descriptor.raw().0 == b"(Ljava/lang/String;I)V";
                    has_int_enum_constructor |=
                        method.descriptor.raw().0 == b"(Ljava/lang/String;II)V";
                }
            }
        }
        let capture_enum_constructor_ast = capture_enum_group_code
            && constructor_count == 2
            && has_no_arg_enum_constructor
            && has_int_enum_constructor;
        if capture_enum_constructor_ast {
            budget.charge(CountedBudgetDimension::IrItems, 2)?;
        }
        let mut attempted = 0_u64;
        let mut methods = Vec::new();
        // Same-run method candidates stay private to this assembly pass until the complete
        // class-level proof selects the unique `<clinit>()V` result. Other member bodies also
        // carry this sidecar handoff, so the physical method identity is the selector.
        let mut initializer_candidate_runs = Vec::new();
        let mut enum_constructor_candidate_runs = Vec::new();
        let mut enum_code_candidates = Vec::new();
        let mut enum_switch_candidate_runs = Vec::new();
        let mut array_constructor_candidate_runs = Vec::new();
        let mut array_helper_use_runs = Vec::new();
        // This only avoids scanning unrelated classes. A header-level synthetic lambda helper is
        // not proof of a projection; all eligibility still comes from same-run Code/CP/AST facts.
        let array_helper_census_needed = read.facts.methods.iter().any(|member| {
            member.name.raw().0.starts_with(b"lambda$")
                && member.access_flags & (0x0002 | 0x0008 | 0x1000) == (0x0002 | 0x0008 | 0x1000)
                && code_shell(member).is_some()
        });
        let mut enum_switch_field_use_runs = Vec::new();
        let mut enum_switch_scanned_members = Vec::new();
        // The anonymous-body follow-up consumes these method-scoped scans from this same assembly.
        // A missing entry remains distinct from a completed empty scan.
        let mut anonymous_allocation_scans: Vec<(
            PhysicalMethodId,
            Option<jarde_java::report::AnonymousAllocationScan>,
        )> = Vec::new();
        // Bridge decisions are also kept from the same member runs, independently of whether
        // RuleDetails were requested. They remain associated with physical member identity and
        // are not derived again from the serialized recovery report.
        let mut _bridge_candidate_runs = Vec::new();
        // The constant pool every declaration attribute this presentation spells is resolved
        // against — class/member Runtime*Annotations and Runtime*TypeAnnotations, parameter
        // annotations, a field's `ConstantValue` (JVMS 4.7.2) or `Signature` (JVMS 4.7.9),
        // a member's `Exceptions` (JVMS 4.7.4) or `AnnotationDefault` (JVMS 4.7.22) — read only
        // when one of those shells exists:
        // a class without them reads no pool for declarations. When there is a
        // preparation it holds that pool already — the same bytes, read once for the whole
        // presentation — and a class that is never prepared (one whose members all declare no body,
        // which is every annotation type) has its own resolved here. Neither read is charged: the
        // binding read paid for the class's bytes, and this is another view of them.
        let pool: Cow<'_, [CpEntryFacts]> = match (
            &prepared,
            read.facts
                .fields
                .iter()
                .any(class_source::declares_constant_value)
                || read
                    .facts
                    .fields
                    .iter()
                    .any(class_source::declares_signature)
                || read
                    .facts
                    .fields
                    .iter()
                    .any(class_source::declares_member_annotations)
                || read.facts.methods.iter().any(|member| {
                    class_source::declares_member_annotations(member)
                        || class_source::declares_parameter_annotations(member)
                })
                || !annotation_shells.is_empty()
                || read.facts.attributes.iter().any(|attribute| {
                    matches!(
                        attribute.name.raw().0.as_slice(),
                        b"Signature" | b"InnerClasses" | b"EnclosingMethod"
                    )
                })
                || read.facts.methods.iter().any(|member| {
                    class_source::declares_annotation_default(member)
                        || class_source::declares_exceptions(member)
                        || class_source::declares_signature(member)
                }),
        ) {
            (Some(prepared), _) => Cow::Borrowed(prepared.class_facts().constant_pool.as_slice()),
            (None, true) => Cow::Owned(class_constant_pool(&read.bytes, budget)?),
            (None, false) => Cow::Borrowed(&[]),
        };
        // Class nesting is an assembly fact, not method IR. Decode the two already recognized
        // class attributes once from the selected read and keep them in a private handoff for the
        // source assembler. In particular, do not re-slice these attributes from a sibling class
        // or infer their meaning from `$` names.
        let nesting_shells: Vec<AttributeShell> = read
            .facts
            .attributes
            .iter()
            .filter(|attribute| {
                matches!(
                    attribute.name.raw().0.as_slice(),
                    b"InnerClasses" | b"EnclosingMethod"
                )
            })
            .cloned()
            .collect();
        let mut assembly_context = class_source::ClassSourceAssemblyContext::default();
        let mut family_scan = crate::member_inner::FamilyRootScan::Absent;
        if !nesting_shells.is_empty() {
            match class_source::read_class_source_assembly_context(
                &read.bytes,
                &nesting_shells,
                &pool,
                budget,
            ) {
                Ok(context) => {
                    assembly_context = context;
                    match crate::member_inner::scan_family_root(
                        &read.facts.this_class.raw().0,
                        &assembly_context,
                        &pool,
                        budget,
                    ) {
                        Ok(scan) => family_scan = scan,
                        Err(error) => {
                            merge_execution(&mut execution, stop_execution(&error, budget));
                            diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                            family_scan = crate::member_inner::FamilyRootScan::Refused(
                                "member relation scan stopped".to_owned(),
                            );
                        }
                    }
                }
                Err(error) => {
                    merge_execution(&mut execution, stop_execution(&error, budget));
                    diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    family_scan = crate::member_inner::FamilyRootScan::Refused(
                        "typed nesting attributes could not be read".to_owned(),
                    );
                }
            }
        }
        let mut class_scope = None;
        let class_signature_present = read
            .facts
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Signature");
        let mut class_signature_stop = false;
        match declaration.project_generic_signature(
            &read.bytes,
            &read.facts.attributes,
            &pool,
            budget,
        ) {
            Ok(proof) => class_scope = proof,
            Err(error) => {
                merge_execution(&mut execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                class_signature_stop = true;
            }
        }
        let physical_interfaces_raw: Vec<Vec<u8>> = read
            .facts
            .interfaces
            .iter()
            .map(|name| name.raw().0.clone())
            .collect();
        if !class_signature_stop && !annotation_shells.is_empty() {
            match attribute_facts(&read.bytes, &annotation_shells, &pool, budget) {
                Ok(facts) => {
                    let attributes = annotation_shells
                        .iter()
                        .map(|attribute| {
                            let annotations = match attribute.name.raw().0.as_slice() {
                                b"RuntimeVisibleAnnotations" => {
                                    facts.runtime_visible_annotations.clone()
                                }
                                b"RuntimeInvisibleAnnotations" => {
                                    facts.runtime_invisible_annotations.clone()
                                }
                                _ => unreachable!("only annotation shells were selected"),
                            };
                            class_source::ClassAnnotationAttribute {
                                attribute: attribute.clone(),
                                annotations,
                            }
                        })
                        .collect();
                    declaration = declaration.with_annotations(attributes, &pool);
                }
                Err(error) => {
                    merge_execution(&mut execution, stop_execution(&error, budget));
                    diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    declaration
                        .annotation_refusals
                        .push(format!("annotation attribute read stopped: {}", error));
                    declaration.annotation_attributes = annotation_shells
                        .into_iter()
                        .map(|attribute| class_source::ClassAnnotationAttribute {
                            attribute,
                            annotations: Vec::new(),
                        })
                        .collect();
                }
            }
        }
        // The fields of the same read, in declaration order, each charged as the item it is and each
        // spelled from its own `ConstantValue` when it declares one. They are read after the pool
        // above because that attribute's own index resolves through it. This loop spells only that
        // attribute's value; the separate private `<clinit>` candidate handoff does not change field
        // declarations in this task.
        let mut fields = Vec::new();
        let mut constant_value_spellable = Vec::new();
        let mut ended = class_signature_stop;
        for (index, field) in read.facts.fields.iter().enumerate() {
            if ended {
                break;
            }
            if let Err(error) = charge_item(budget) {
                merge_execution(&mut execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                ended = true;
                break;
            }
            let ClassContentItem::Field(item) = field_item(&definition, index, field)? else {
                unreachable!("a field record publishes a field item")
            };
            // The field's own `ConstantValue`, read **once** here: the declaration below is written
            // from this one value, so no second reading can state a second initializer. A value this
            // presentation has no literal for — a `float`/`double` constant — is no initializer at
            // all, which is the declaration the bytes state.
            let constant =
                match class_source::declared_constant_value(&read.bytes, field, &pool, budget) {
                    Ok(constant) => constant,
                    Err(error) => {
                        // A field record states no refusal of its own in this report, so a field whose
                        // attribute could not be read stops the presentation where that read happened
                        // rather than being published as a declaration without its initializer: which
                        // constants a field declares is exactly what these bytes did not establish.
                        // The fields before it keep their spellings and the reader's own code states
                        // why it stopped.
                        merge_execution(&mut execution, stop_execution(&error, budget));
                        diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                        ended = true;
                        break;
                    }
                };
            constant_value_spellable.push(constant.is_some());
            let annotation_read =
                class_source::declared_member_annotations(&read.bytes, field, &pool, budget);
            let mut annotation_stop = None;
            for error in &annotation_read.errors {
                let stop = stop_execution(error, budget);
                merge_execution(&mut execution, stop.clone());
                diagnostics.push(stop_diagnostic(error, class_provenance.clone()));
                if ends_the_request(&stop) && annotation_stop.is_none() {
                    annotation_stop = Some(error.clone());
                }
            }
            let mut source_field =
                ClassSourceField::of(item, constant.as_ref(), annotation_read.facts, &pool);
            if annotation_stop.is_some() {
                fields.push(source_field);
                ended = true;
                break;
            }
            if let Err(error) = class_source::project_field_signature(
                &mut source_field,
                field,
                &read.bytes,
                &pool,
                &declaration.item.declaration.this_class.raw().0,
                declaration.item.declaration.access_flags,
                class_scope
                    .as_ref()
                    .map(|proof| proof.type_parameters.as_slice())
                    .unwrap_or(&[]),
                constant.as_ref(),
                budget,
            ) {
                merge_execution(&mut execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                fields.push(source_field);
                ended = true;
                break;
            }
            fields.push(source_field);
        }
        for (index, member) in read.facts.methods.iter().enumerate() {
            if ended {
                break;
            }
            let ClassContentItem::Method(item) = method_item(&definition, index, member)? else {
                unreachable!("a method record publishes a method item")
            };
            let annotation_read =
                class_source::declared_member_annotations(&read.bytes, member, &pool, budget);
            let mut annotation_stop = None;
            for error in &annotation_read.errors {
                let stop = stop_execution(error, budget);
                merge_execution(&mut execution, stop.clone());
                diagnostics.push(stop_diagnostic(error, class_provenance.clone()));
                if ends_the_request(&stop) && annotation_stop.is_none() {
                    annotation_stop = Some(error.clone());
                }
            }
            if let Some(error) = annotation_stop {
                let stop = stop_execution(&error, budget);
                let spelled = class_source::spell_method(
                    &item,
                    None,
                    &declaration.name,
                    read.facts.access_flags,
                    None,
                    &annotation_read.facts,
                    &pool,
                );
                let record = ClassSourceMethod::refused(
                    item,
                    spelled,
                    stop.clone(),
                    vec![stop_diagnostic(&error, class_provenance.clone())],
                );
                if let Err(charge) = charge_item(budget) {
                    merge_execution(&mut execution, stop_execution(&charge, budget));
                    diagnostics.push(stop_diagnostic(&charge, class_provenance.clone()));
                    break;
                }
                methods.push(record);
                break;
            }
            // The member's own declaration attributes, read **once** here: the `AnnotationDefault`
            // (JVMS 4.7.22) the declaration's `default` is written from and the `Exceptions`
            // (JVMS 4.7.4) its `throws` clause is written from. Both spellings below — before and
            // after the member's run — are written from these two values, so a second reading cannot
            // state a second clause. A request-ending failure (a budget, a cancellation) is this
            // request's stop, exactly like a refused item charge above; damage inside one member's
            // attribute is that member's read failing, and the member is still published with the
            // declaration the flags and the descriptor state — neither attribute was read to write
            // either clause from — while the engine keeps the reader's own code for it.
            let attributes = match class_source::declared_member_attributes(
                &read.bytes,
                member,
                &pool,
                budget,
            ) {
                Ok(attributes) => attributes,
                Err(error) => {
                    let stop = stop_execution(&error, budget);
                    let ends = ends_the_request(&stop);
                    let spelled = class_source::spell_method(
                        &item,
                        None,
                        &declaration.name,
                        read.facts.access_flags,
                        None,
                        &annotation_read.facts,
                        &pool,
                    );
                    let record = ClassSourceMethod::refused(
                        item,
                        spelled,
                        stop.clone(),
                        vec![stop_diagnostic(&error, class_provenance.clone())],
                    );
                    merge_execution(&mut execution, stop);
                    // The member is published, so it is charged as the item it is, exactly as the
                    // members below are: a refusal that could not pay for its own record is the
                    // request's stop and the record is not written.
                    if let Err(charge) = charge_item(budget) {
                        merge_execution(&mut execution, stop_execution(&charge, budget));
                        diagnostics.push(stop_diagnostic(&charge, class_provenance.clone()));
                        break;
                    }
                    methods.push(record);
                    if ends {
                        break;
                    }
                    continue;
                }
            };
            let spelled = class_source::spell_method(
                &item,
                None,
                &declaration.name,
                read.facts.access_flags,
                Some(&attributes),
                &annotation_read.facts,
                &pool,
            );
            let (record, stops, ends) =
                if !class_source::spellable_descriptor(&item.descriptor.raw().0) {
                    // A member this presentation cannot spell: no run is performed for it, because the
                    // artifact of such a run would have no declaration to be written under.
                    (
                        ClassSourceMethod::unspelled(item, spelled),
                        Vec::new(),
                        false,
                    )
                } else if !class_source_runs_body(member) {
                    let mut record = ClassSourceMethod::no_body(
                        item,
                        no_body_kind(member.access_flags),
                        spelled,
                    );
                    let mut stops = Vec::new();
                    if let Err(error) = class_source::project_method_signature(
                        &mut record,
                        member,
                        &attributes,
                        None,
                        None,
                        &read.bytes,
                        &pool,
                        &read.facts.this_class.raw().0,
                        read.facts.access_flags,
                        read.facts
                            .super_class
                            .as_ref()
                            .map(|name| name.raw().0.as_slice()),
                        &physical_interfaces_raw,
                        class_scope
                            .as_ref()
                            .map(|proof| proof.type_parameters.as_slice())
                            .unwrap_or(&[]),
                        class_signature_present,
                        budget,
                    ) {
                        stops.push(stop_execution(&error, budget));
                        diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    }
                    let ends = stops.iter().any(ends_the_request);
                    (record, stops, ends)
                } else {
                    let request = crate::ir::MethodAnalysisRequest {
                        environment: environment.clone(),
                        method: item.identity.clone(),
                        stages: stages.clone(),
                    };
                    match &bodies {
                        ClassBodies::Prepared(prepared) => {
                            // One run entered: this is the coordinate this presentation's body plane
                            // counts, so a member whose run was never entered — because the class could
                            // not be prepared — stays in that plane's skipped range.
                            attempted = attempted.saturating_add(1);
                            match recover_prepared_member(
                                content,
                                &request,
                                prepared,
                                &assembly_context,
                                item.index,
                                evidence,
                                PreparedMemberOptions {
                                    prove_generic_return: member
                                        .attributes
                                        .iter()
                                        .any(|attribute| attribute.name.raw().0 == b"Signature"),
                                    capture_enum_group_code,
                                    capture_enum_constructor_ast,
                                    array_helper_census_needed,
                                    capture_array_helper_use_table: array_helper_use_runs
                                        .is_empty(),
                                },
                                budget,
                            ) {
                                Ok(PreparedMemberRecovery {
                                    recovered,
                                    initializer: candidates_for_member,
                                    enum_constructor: enum_constructor_candidates,
                                    bridge: bridge_candidate,
                                    enum_switches: enum_switch_candidates,
                                    array_constructors: array_constructor_candidates,
                                    array_helper_uses,
                                    enum_switch_field_uses,
                                    generic_return,
                                    generic_constructor,
                                    anonymous_allocations,
                                    enum_code: enum_code_candidate,
                                }) => {
                                    if let Some(candidates) = candidates_for_member {
                                        initializer_candidate_runs.push(candidates);
                                    }
                                    if let Some(candidate) = enum_constructor_candidates {
                                        enum_constructor_candidate_runs.push(candidate);
                                    }
                                    if let Some(candidate) = enum_code_candidate {
                                        enum_code_candidates.push(candidate);
                                    }
                                    if let Some(candidate) = bridge_candidate {
                                        _bridge_candidate_runs.push(candidate);
                                    }
                                    if let Some(candidates) = enum_switch_candidates {
                                        enum_switch_candidate_runs.extend(candidates);
                                    }
                                    if let Some(candidates) = array_constructor_candidates {
                                        array_constructor_candidate_runs.extend(candidates);
                                    }
                                    if let Some(scan) = array_helper_uses {
                                        array_helper_use_runs.push(scan);
                                    }
                                    if let Some(uses) = enum_switch_field_uses {
                                        enum_switch_scanned_members.push(item.identity.clone());
                                        enum_switch_field_use_runs.extend(uses);
                                    }
                                    anonymous_allocation_scans
                                        .push((item.identity.clone(), anonymous_allocations));
                                    let analysis = ClassSourceRunFacts {
                                        execution: recovered.analysis().execution.clone(),
                                        diagnostics: to_u64(
                                            recovered.analysis().diagnostics.len(),
                                        )?,
                                    };
                                    // The declaration is spelled again with the run's own facts (the
                                    // parameter names the body's statements use); the declared
                                    // attributes are the ones already resolved for this member, so
                                    // the two spellings cannot state two different clauses.
                                    let spelled = class_source::spell_method(
                                        &item,
                                        Some(recovered.facts()),
                                        &declaration.name,
                                        read.facts.access_flags,
                                        Some(&attributes),
                                        &annotation_read.facts,
                                        &pool,
                                    );
                                    let (_, report, _) = recovered.into_parts();
                                    let mut stops =
                                        vec![analysis.execution.clone(), report.execution.clone()];
                                    let mut record = ClassSourceMethod::recovered(
                                        item,
                                        spelled,
                                        Box::new(report),
                                        analysis,
                                    );
                                    if let Err(error) = class_source::project_method_signature(
                                        &mut record,
                                        member,
                                        &attributes,
                                        generic_return.as_ref(),
                                        generic_constructor.as_ref(),
                                        &read.bytes,
                                        &pool,
                                        &read.facts.this_class.raw().0,
                                        read.facts.access_flags,
                                        read.facts
                                            .super_class
                                            .as_ref()
                                            .map(|name| name.raw().0.as_slice()),
                                        &physical_interfaces_raw,
                                        class_scope
                                            .as_ref()
                                            .map(|proof| proof.type_parameters.as_slice())
                                            .unwrap_or(&[]),
                                        class_signature_present,
                                        budget,
                                    ) {
                                        stops.push(stop_execution(&error, budget));
                                        diagnostics.push(stop_diagnostic(
                                            &error,
                                            class_provenance.clone(),
                                        ));
                                    }
                                    let ends = stops.iter().any(ends_the_request);
                                    (record, stops, ends)
                                }
                                Err(error) => {
                                    anonymous_allocation_scans.push((item.identity.clone(), None));
                                    let stop = stop_execution(&error, budget);
                                    let ends = ends_the_request(&stop);
                                    let record = ClassSourceMethod::refused(
                                        item,
                                        spelled,
                                        stop.clone(),
                                        vec![stop_diagnostic(&error, class_provenance.clone())],
                                    );
                                    (record, vec![stop], ends)
                                }
                            }
                        }
                        // No body of this class can be decoded, and the one attempt to prepare it is why:
                        // the member keeps that failure as its own refusal, and the members beside it are
                        // presented exactly as usual.
                        ClassBodies::Refused(refusal) => (
                            ClassSourceMethod::refused(
                                item,
                                spelled,
                                refusal.stop.clone(),
                                vec![refusal.diagnostic.clone()],
                            ),
                            vec![refusal.stop.clone()],
                            refusal.ends,
                        ),
                        // Unreachable by construction: `class_source_runs_body` is the one predicate that
                        // counts the bodies a class has and the one that sends a member here, so a member
                        // in this arm is a member of a class the preparation above was made for.
                        ClassBodies::NotNeeded => unreachable!(
                            "a member that runs a body belongs to a class the preparation counted"
                        ),
                    }
                };
            for stop in stops {
                merge_execution(&mut execution, stop);
            }
            if let Err(error) = charge_item(budget) {
                merge_execution(&mut execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                break;
            }
            methods.push(record);
            if ends {
                ended = true;
            }
        }
        let mut enum_switch_proofs = Vec::new();
        let mut array_projection_method_texts: Vec<(u64, String)> = Vec::new();
        let mut array_helper_method_indices: Vec<u64> = Vec::new();
        let mut array_projection_markers = Vec::new();
        let mut array_projection_members: Vec<(u64, Vec<u64>)> = Vec::new();
        let mut array_original_member_texts: Vec<(u64, String)> = Vec::new();
        // Keep the same-run census local until its class-level proof is implemented. Methods whose
        // preparation or body run never happened are absent; they are not empty scans.
        let enum_projection_complete = structure_complete
            && !ended
            && methods.len() == read.facts.methods.len()
            && read
                .facts
                .methods
                .iter()
                .zip(&methods)
                .all(|(header, method)| {
                    if !class_source_runs_body(header) {
                        return true;
                    }
                    enum_switch_scanned_members.contains(&method.item.identity)
                        && matches!(
                            &method.outcome,
                            class_source::ClassSourceOutcome::Recovered { analysis, .. }
                                if matches!(&analysis.execution, ExecutionReport::Complete { .. })
                        )
                });
        let array_projection_complete = structure_complete
            && !ended
            && methods.len() == read.facts.methods.len()
            && read
                .facts
                .methods
                .iter()
                .zip(&methods)
                .all(|(header, method)| {
                    if code_shell(header).is_none() {
                        return true;
                    }
                    class_source_runs_body(header)
                        && array_helper_use_runs.iter().any(|scan| {
                            scan.complete && scan.member.as_ref() == Some(&method.item.identity)
                        })
                        && matches!(
                            &method.outcome,
                            class_source::ClassSourceOutcome::Recovered { analysis, .. }
                                if matches!(&analysis.execution, ExecutionReport::Complete { .. })
                        )
                });
        let mut staged_enum_projections: Vec<(usize, String, String, usize)> = Vec::new();
        let mut enum_projection_stopped = false;
        if enum_projection_complete && !enum_switch_candidate_runs.is_empty() {
            let mut by_method: Vec<(
                PhysicalMethodId,
                Vec<jarde_java::enumswitch::ClassSourceEnumSwitchCandidate>,
            )> = Vec::new();
            for candidate in enum_switch_candidate_runs.iter().cloned() {
                if let Some(member) = &candidate.member {
                    if let Some((_, candidates)) =
                        by_method.iter_mut().find(|(known, _)| known == member)
                    {
                        candidates.push(candidate);
                    } else {
                        by_method.push((member.clone(), vec![candidate]));
                    }
                }
            }
            for (member, candidates) in by_method {
                // Until the emitter can stage several switch sites in one copied AST, a method
                // with multiple candidates stays entirely on its original integer path.
                if candidates.len() != 1 {
                    for candidate in candidates {
                        if let Some(member) = candidate.member.clone() {
                            enum_switch_proofs.push(class_source::ClassSourceEnumSwitchProof {
                                member,
                                switch_bci: candidate.switch_bci,
                                read_bci: candidate.read_bci,
                                table_owner: candidate.table.owner,
                                table_name: candidate.table.name,
                                helper: None,
                                enum_definition: None,
                                entries: Vec::new(),
                                projected: false,
                                refusal: Some("the method has multiple enum switch sites and grouped AST projection is not available".to_owned()),
                            });
                        }
                    }
                    continue;
                }
                let Some(method_index) = methods
                    .iter()
                    .position(|method| method.item.identity == member)
                else {
                    continue;
                };
                let method_complete = matches!(
                    &methods[method_index].outcome,
                    class_source::ClassSourceOutcome::Recovered { report, analysis }
                        if report.produced()
                            && report.quality == Quality::Structured
                            && report.fallbacks.is_empty()
                            && matches!(&report.execution, ExecutionReport::Complete { .. })
                            && matches!(&analysis.execution, ExecutionReport::Complete { .. })
                );
                if !method_complete {
                    let candidate = &candidates[0];
                    enum_switch_proofs.push(class_source::ClassSourceEnumSwitchProof {
                        member: member.clone(),
                        switch_bci: candidate.switch_bci,
                        read_bci: candidate.read_bci,
                        table_owner: candidate.table.owner.clone(),
                        table_name: candidate.table.name.clone(),
                        helper: None,
                        enum_definition: None,
                        entries: Vec::new(),
                        projected: false,
                        refusal: Some(
                            "the candidate method did not recover completely as structured Java"
                                .to_owned(),
                        ),
                    });
                    continue;
                }
                let candidate = &candidates[0];
                let extra_table_use = enum_switch_field_use_runs.iter().find(|use_site| {
                    use_site.owner == candidate.table.owner
                        && use_site.name == candidate.table.name
                        && use_site.descriptor == candidate.table.descriptor
                        && !(use_site.member.as_ref() == Some(&member)
                            && use_site.bci == candidate.table.bci
                            && use_site.is_static
                            && !use_site.write)
                });
                if let Some(use_site) = extra_table_use {
                    enum_switch_proofs.push(class_source::ClassSourceEnumSwitchProof {
                        member: member.clone(),
                        switch_bci: candidate.switch_bci,
                        read_bci: candidate.read_bci,
                        table_owner: candidate.table.owner.clone(),
                        table_name: candidate.table.name.clone(),
                        helper: None,
                        enum_definition: None,
                        entries: Vec::new(),
                        projected: false,
                        refusal: Some(format!(
                            "visible method writes or reads the selected table outside the proved candidate at BCI {}",
                            use_site.bci,
                        )),
                    });
                    continue;
                }
                let mut method_staged = Vec::new();
                let mut method_proved = true;
                for candidate in &candidates {
                    let map = match prove_class_source_enum_switch(
                        content,
                        &environment,
                        &request.environment.policy,
                        &definition,
                        candidate,
                        &mut execution,
                        budget,
                    ) {
                        Ok(Ok(map)) => map,
                        Ok(Err(reason)) => {
                            enum_switch_proofs.push(class_source::ClassSourceEnumSwitchProof {
                                member: member.clone(),
                                switch_bci: candidate.switch_bci,
                                read_bci: candidate.read_bci,
                                table_owner: candidate.table.owner.clone(),
                                table_name: candidate.table.name.clone(),
                                helper: None,
                                enum_definition: None,
                                entries: Vec::new(),
                                projected: false,
                                refusal: Some(reason),
                            });
                            method_proved = false;
                            break;
                        }
                        Err(error) => {
                            enum_switch_proofs.push(class_source::ClassSourceEnumSwitchProof {
                                member: member.clone(),
                                switch_bci: candidate.switch_bci,
                                read_bci: candidate.read_bci,
                                table_owner: candidate.table.owner.clone(),
                                table_name: candidate.table.name.clone(),
                                helper: None,
                                enum_definition: None,
                                entries: Vec::new(),
                                projected: false,
                                refusal: Some(format!("cross-class proof stopped: {error}")),
                            });
                            let stop = stop_execution(&error, budget);
                            merge_execution(&mut execution, stop);
                            diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                            enum_projection_stopped = true;
                            method_proved = false;
                            break;
                        }
                    };
                    let recovery_text = match jarde_java::report::emit_class_source_enum_switch(
                        candidate,
                        &map.labels,
                        budget,
                    ) {
                        Ok(Some(text)) => text,
                        Ok(None) => {
                            enum_switch_proofs.push(class_source::ClassSourceEnumSwitchProof {
                                member: member.clone(), switch_bci: candidate.switch_bci,
                                read_bci: candidate.read_bci, table_owner: candidate.table.owner.clone(),
                                table_name: candidate.table.name.clone(), helper: Some(map.helper.clone()),
                                enum_definition: Some(map.enum_definition.clone()), entries: Vec::new(),
                                projected: false, refusal: Some("the same-run AST did not contain the matching enum switch shape".to_owned()),
                            });
                            method_proved = false;
                            break;
                        }
                        Err(stop) => {
                            let error = enum_projection_stop_error(
                                stop,
                                "enum switch projection",
                                "enum_switch_ir_missing",
                            );
                            enum_switch_proofs.push(class_source::ClassSourceEnumSwitchProof {
                                member: member.clone(),
                                switch_bci: candidate.switch_bci,
                                read_bci: candidate.read_bci,
                                table_owner: candidate.table.owner.clone(),
                                table_name: candidate.table.name.clone(),
                                helper: Some(map.helper.clone()),
                                enum_definition: Some(map.enum_definition.clone()),
                                entries: Vec::new(),
                                projected: false,
                                refusal: Some(format!("projection emission stopped: {error}")),
                            });
                            let execution_stop = stop_execution(&error, budget);
                            merge_execution(&mut execution, execution_stop);
                            diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                            enum_projection_stopped = true;
                            method_proved = false;
                            break;
                        }
                    };
                    let map_text = map
                        .stores
                        .iter()
                        .map(|entry| {
                            format!(
                                "{}={}@{}[field {}, ordinal {}, read {}, handler {}:{}]",
                                entry.key,
                                String::from_utf8_lossy(&entry.constant),
                                entry.store_bci,
                                entry.constant_field_bci,
                                entry.ordinal_bci,
                                entry.table_read_bci,
                                entry.handler_ordinal,
                                entry.handler_bci,
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    let marker = format!(
                        "// jarde: enum switch projected at BCI {} from {:?}.{} via {:?}; initializer stores [{}]",
                        candidate.switch_bci,
                        map.helper,
                        candidate.table.name,
                        map.enum_definition,
                        map_text,
                    );
                    if let Err(error) = budget.charge(
                        CountedBudgetDimension::OutputBytes,
                        u64::try_from(marker.len()).unwrap_or(u64::MAX),
                    ) {
                        let stop = stop_execution(&error, budget);
                        enum_switch_proofs.push(class_source::ClassSourceEnumSwitchProof {
                            member: member.clone(),
                            switch_bci: candidate.switch_bci,
                            read_bci: candidate.read_bci,
                            table_owner: candidate.table.owner.clone(),
                            table_name: candidate.table.name.clone(),
                            helper: Some(map.helper.clone()),
                            enum_definition: Some(map.enum_definition.clone()),
                            entries: Vec::new(),
                            projected: false,
                            refusal: Some(format!("projection marker output was stopped: {error}")),
                        });
                        merge_execution(&mut execution, stop);
                        diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                        enum_projection_stopped = true;
                        method_proved = false;
                        break;
                    }
                    let proof_index = enum_switch_proofs.len();
                    enum_switch_proofs.push(class_source::ClassSourceEnumSwitchProof {
                        member: member.clone(),
                        switch_bci: candidate.switch_bci,
                        read_bci: candidate.read_bci,
                        table_owner: candidate.table.owner.clone(),
                        table_name: candidate.table.name.clone(),
                        helper: Some(map.helper.clone()),
                        enum_definition: Some(map.enum_definition.clone()),
                        entries: map
                            .stores
                            .iter()
                            .map(|entry| class_source::ClassSourceEnumSwitchEntry {
                                key: entry.key,
                                constant: entry.constant.clone(),
                                constant_field_bci: entry.constant_field_bci,
                                ordinal_bci: entry.ordinal_bci,
                                table_read_bci: entry.table_read_bci,
                                store_bci: entry.store_bci,
                                handler_ordinal: entry.handler_ordinal,
                                handler_bci: entry.handler_bci,
                            })
                            .collect(),
                        projected: false,
                        refusal: None,
                    });
                    method_staged.push((recovery_text, marker, proof_index));
                }
                if method_proved {
                    for (recovery_text, marker, proof_index) in method_staged {
                        staged_enum_projections.push((
                            method_index,
                            recovery_text,
                            marker,
                            proof_index,
                        ));
                    }
                }
                if enum_projection_stopped {
                    break;
                }
            }
        }
        if !enum_projection_stopped {
            let mut projected_methods = methods.clone();
            let mut projected_proof_indices = Vec::new();
            for (method_index, recovery_text, marker, proof_index) in staged_enum_projections {
                if !projected_methods[method_index].project_enum_switch(&recovery_text, marker) {
                    enum_projection_stopped = true;
                    enum_switch_proofs[proof_index].refusal = Some(
                        "the emitted method could not be atomically placed in its physical member"
                            .to_owned(),
                    );
                    break;
                }
                projected_proof_indices.push(proof_index);
            }
            if !enum_projection_stopped {
                methods = projected_methods;
                for proof_index in projected_proof_indices {
                    enum_switch_proofs[proof_index].projected = true;
                }
            }
        }
        if enum_projection_stopped {
            for proof in &mut enum_switch_proofs {
                if proof.refusal.is_none() && !proof.projected {
                    proof.refusal = Some(
                        "the class-source enum projection batch stopped before commit".to_owned(),
                    );
                }
            }
        }
        let mut array_projection_stopped = false;
        if array_projection_complete && !array_constructor_candidate_runs.is_empty() {
            let mut by_helper: Vec<(
                PhysicalMethodId,
                Vec<jarde_java::report::ClassSourceArrayConstructorCandidate>,
            )> = Vec::new();
            for candidate in array_constructor_candidate_runs.iter().cloned() {
                if let Some((_, sites)) = by_helper
                    .iter_mut()
                    .find(|(helper, _)| *helper == candidate.helper)
                {
                    sites.push(candidate);
                } else {
                    by_helper.push((candidate.helper.clone(), vec![candidate]));
                }
            }
            for (helper, candidates) in by_helper {
                if array_projection_stopped {
                    break;
                }
                let census = array_helper_census_refusal(
                    &candidates[0],
                    &candidates,
                    &array_helper_use_runs,
                    budget,
                );
                let refusal = match census {
                    Ok(refusal) => refusal,
                    Err(error) => {
                        merge_execution(&mut execution, stop_execution(&error, budget));
                        diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                        break;
                    }
                };
                if refusal.is_none() {
                    let collides_with_enum_switch = candidates.iter().any(|candidate| {
                        enum_switch_proofs
                            .iter()
                            .any(|proof| proof.projected && proof.member == candidate.member)
                    });
                    let collides_with_bridge = candidates.iter().any(|candidate| {
                        _bridge_candidate_runs.iter().any(|bridge| {
                            bridge.presented && bridge.member.as_ref() == Some(&candidate.member)
                        })
                    });
                    if collides_with_enum_switch || collides_with_bridge {
                        continue;
                    }
                    let mut helper_members = methods
                        .iter()
                        .filter(|method| method.item.identity == helper);
                    let Some(helper_method) = helper_members.next() else {
                        continue;
                    };
                    if helper_members.next().is_some() {
                        continue;
                    }
                    let mut candidate_by_member: Vec<(
                        PhysicalMethodId,
                        jarde_java::report::ClassSourceArrayConstructorCandidate,
                    )> = Vec::new();
                    for candidate in candidates {
                        if let Some((_, grouped)) = candidate_by_member
                            .iter_mut()
                            .find(|(member, _)| *member == candidate.member)
                        {
                            grouped.sites.extend(candidate.sites);
                        } else {
                            candidate_by_member.push((candidate.member.clone(), candidate));
                        }
                    }
                    let mut staged = Vec::new();
                    let mut accepted = true;
                    for (member, candidate) in candidate_by_member {
                        let mut member_matches = methods
                            .iter()
                            .enumerate()
                            .filter(|(_, method)| method.item.identity == member);
                        let Some((method_index, method)) = member_matches.next() else {
                            accepted = false;
                            break;
                        };
                        if member_matches.next().is_some() {
                            accepted = false;
                            break;
                        }
                        if !array_target_declaration_matches(&candidate, method) {
                            accepted = false;
                            break;
                        }
                        let class_source::ClassSourceOutcome::Recovered { report, analysis } =
                            &method.outcome
                        else {
                            accepted = false;
                            break;
                        };
                        if !matches!(analysis.execution, ExecutionReport::Complete { .. })
                            || !report.outcome.produced()
                            || report.quality != jarde_jvm::ir::Quality::Structured
                            || !report.fallbacks.is_empty()
                        {
                            accepted = false;
                            break;
                        }
                        let projected_body =
                            match jarde_java::report::emit_class_source_array_constructors(
                                &candidate, budget,
                            ) {
                                Ok(Some(text)) => text,
                                Ok(None) => {
                                    accepted = false;
                                    break;
                                }
                                Err(stop) => {
                                    let error = enum_projection_stop_error(
                                        stop,
                                        "array-constructor projection",
                                        "array_constructor_projection_stopped",
                                    );
                                    merge_execution(&mut execution, stop_execution(&error, budget));
                                    diagnostics
                                        .push(stop_diagnostic(&error, class_provenance.clone()));
                                    accepted = false;
                                    array_projection_stopped = true;
                                    break;
                                }
                            };
                        let marker = format!(
                            "// jarde: projected {} array-constructor site(s) at {:?} from exact helper {:?}",
                            candidate.sites.len(),
                            candidate
                                .sites
                                .iter()
                                .map(|site| site.use_site)
                                .collect::<Vec<_>>(),
                            helper.name,
                        );
                        let Some(full_text) = method.array_projection_text(&projected_body, marker)
                        else {
                            accepted = false;
                            break;
                        };
                        if let Err(error) = budget.charge(
                            CountedBudgetDimension::OutputBytes,
                            u64::try_from(full_text.len()).unwrap_or(u64::MAX),
                        ) {
                            merge_execution(&mut execution, stop_execution(&error, budget));
                            diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                            accepted = false;
                            array_projection_stopped = true;
                            break;
                        }
                        staged.push((method_index, method.item.index, full_text));
                    }
                    let overlaps_existing = staged.iter().any(|(_, member_index, _)| {
                        array_projection_members
                            .iter()
                            .any(|(_, members)| members.contains(member_index))
                    });
                    if accepted && !staged.is_empty() && !overlaps_existing {
                        let marker = format!(
                            "// jarde: omitted physical helper {:?} after proving all same-class uses are projected",
                            helper.name,
                        );
                        if let Err(error) = budget.charge(
                            CountedBudgetDimension::OutputBytes,
                            u64::try_from(marker.len()).unwrap_or(u64::MAX),
                        ) {
                            merge_execution(&mut execution, stop_execution(&error, budget));
                            diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                            array_projection_stopped = true;
                        } else {
                            array_projection_method_texts.extend(
                                staged.iter().map(|(_, index, text)| (*index, text.clone())),
                            );
                            array_helper_method_indices.push(helper_method.item.index);
                            array_projection_markers.push(marker);
                            let member_indices = staged
                                .iter()
                                .map(|(method_index, member_index, _)| {
                                    array_original_member_texts
                                        .push((*member_index, methods[*method_index].text.clone()));
                                    *member_index
                                })
                                .collect();
                            array_projection_members
                                .push((helper_method.item.index, member_indices));
                        }
                    }
                    continue;
                }
            }
        }
        let mut bridge_proofs = prove_class_source_bridges(
            content,
            &environment,
            &request.environment.policy,
            &definition,
            &read.facts,
            &methods,
            &_bridge_candidate_runs,
            &mut execution,
            budget,
        );
        // Stage every proof note and pay for all of its output before changing a member. If a
        // later note cannot be funded (or cancellation is observed by the charge), the original
        // method texts remain together as a complete, unprojected prefix.
        let staged_bridge_projections =
            stage_class_source_bridge_projections(&bridge_proofs, &methods);
        let mut bridge_projection_ready = staged_bridge_projections.is_some();
        if let Some(staged) = &staged_bridge_projections {
            for (_, marker) in staged {
                if let Err(error) = budget.charge(
                    CountedBudgetDimension::OutputBytes,
                    u64::try_from(marker.len()).unwrap_or(u64::MAX),
                ) {
                    merge_execution(&mut execution, stop_execution(&error, budget));
                    diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    bridge_projection_ready = false;
                    break;
                }
            }
        }
        if bridge_projection_ready {
            if let Some(staged) = staged_bridge_projections {
                for (method_index, marker) in staged {
                    methods[method_index].project_bridge(marker);
                }
                for proof in &mut bridge_proofs {
                    proof.projected = proof.admitted;
                }
            }
        }
        if let Some(stop) = &read.facts.stopped_at {
            merge_execution(&mut execution, member_stop_execution(stop, budget));
        }
        let mut initializer_proof = if read.facts.access_flags & ACC_INTERFACE != 0
            && read.facts.access_flags & ACC_ANNOTATION == 0
        {
            match prove_interface_initializer_group(
                &declaration,
                &read.facts.fields,
                read.facts.field_count,
                read.facts.stopped_at.is_none(),
                &fields,
                &constant_value_spellable,
                &read.facts.methods,
                read.facts.method_count,
                read.facts.stopped_at.is_none(),
                &methods,
                &initializer_candidate_runs,
                budget,
            ) {
                Ok(proof) => proof,
                Err(error) => {
                    let stop = stop_execution(&error, budget);
                    merge_execution(&mut execution, stop);
                    diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    ClassSourceInitializerProof::Refused {
                        reason: format!("initializer proof stopped: {error}"),
                    }
                }
            }
        } else {
            ClassSourceInitializerProof::NotApplicable
        };
        let initializer_field_order = match project_interface_initializer_group(
            &initializer_proof,
            &mut fields,
            &methods,
            &initializer_candidate_runs,
            budget,
        ) {
            Ok(order) => order,
            Err(InitializerProjectionFailure::Refused(reason)) => {
                initializer_proof = ClassSourceInitializerProof::Refused { reason };
                None
            }
            Err(InitializerProjectionFailure::Stopped(stop)) => {
                let (stop, diagnostic) =
                    initializer_projection_stop(&stop, budget, class_provenance.clone());
                merge_execution(&mut execution, stop);
                diagnostics.push(diagnostic);
                None
            }
        };
        let mut enum_constant_proof = if read.facts.stopped_at.is_some()
            || !matches!(&execution, ExecutionReport::Complete { .. })
        {
            crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                reason: "the class-source member run stopped before the complete enum proof"
                    .to_owned(),
            }
        } else {
            match crate::enum_constants::prove_group(
                &declaration,
                prepared.as_ref().map_or((0, 0), |prepared| {
                    (
                        prepared.class_facts().major_version,
                        prepared.class_facts().minor_version,
                    )
                }),
                &read.facts.fields,
                read.facts.field_count,
                read.facts.stopped_at.is_none(),
                &fields,
                &read.facts.methods,
                read.facts.method_count,
                read.facts.stopped_at.is_none(),
                &methods,
                &enum_code_candidates,
                &initializer_candidate_runs,
                &enum_constructor_candidate_runs,
                budget,
            ) {
                Ok(proof) => proof,
                Err(error) => {
                    let stop = stop_execution(&error, budget);
                    merge_execution(&mut execution, stop);
                    diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                        reason: format!("enum constant proof stopped: {error}"),
                    }
                }
            }
        };
        let mut enum_constant_body_relations = if capture_enum_group_code
            && matches!(
                &enum_constant_proof,
                crate::enum_constants::ClassSourceEnumConstantProof::Refused { .. }
            )
            && structure_complete
            && matches!(&execution, ExecutionReport::Complete { .. })
        {
            match resolve_enum_constant_body_relations(
                content,
                &environment,
                &assembly_context,
                &pool,
                &read.facts.this_class.raw().0,
                read.facts.access_flags,
                &read.facts.fields,
                &read.facts.methods,
                &enum_code_candidates,
                &anonymous_allocation_scans,
                &mut execution,
                budget,
            ) {
                Ok(mut relations) => {
                    match census_enum_constant_body_uses(
                        content,
                        &environment,
                        &definition,
                        &mut relations,
                        &mut execution,
                        budget,
                    ) {
                        Ok(()) => {
                            if relations.iter().any(|relation| {
                                relation.use_census.scans.iter().any(|scan| {
                                    !matches!(&scan.execution, ExecutionReport::Complete { .. })
                                })
                            }) {
                                for relation in &mut relations {
                                    relation.use_census.exclusive = false;
                                    relation.use_census.refusal =
                                        Some("the group owner census stopped".to_owned());
                                }
                                enum_constant_proof =
                                    crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                                        reason: "enum constant subclass use census stopped"
                                            .to_owned(),
                                    };
                            }
                            relations
                        }
                        Err(error) => {
                            for relation in &mut relations {
                                relation.use_census.exclusive = false;
                                relation.use_census.refusal =
                                    Some("the group owner census stopped".to_owned());
                            }
                            let stop = stop_execution(&error, budget);
                            merge_execution(&mut execution, stop);
                            diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                            enum_constant_proof =
                                crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                                    reason: format!(
                                        "enum constant subclass use census stopped: {error}"
                                    ),
                                };
                            relations
                        }
                    }
                }
                Err(error) => {
                    let stop = stop_execution(&error, budget);
                    merge_execution(&mut execution, stop.clone());
                    diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    enum_constant_proof =
                        crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                            reason: format!("enum constant subclass relation stopped: {error}"),
                        };
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        if matches!(
            &enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Refused { .. }
        ) {
            for relation in &mut enum_constant_body_relations {
                if !relation.use_census.exclusive || relation.constructor_bridge.is_err() {
                    continue;
                }
                match prove_enum_constant_child_body(
                    self,
                    content,
                    request,
                    relation,
                    &read.facts.methods,
                    budget,
                ) {
                    Ok((proof, child_execution)) => {
                        merge_execution(&mut execution, child_execution);
                        relation.body_proof = Some(proof.map(std::sync::Arc::from));
                    }
                    Err(error) => {
                        let stop = stop_execution(&error, budget);
                        merge_execution(&mut execution, stop);
                        diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                        enum_constant_proof =
                            crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                                reason: format!(
                                    "enum constant subclass body proof stopped: {error}"
                                ),
                            };
                        break;
                    }
                }
                if !matches!(&execution, ExecutionReport::Complete { .. }) {
                    enum_constant_proof =
                        crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                            reason: "enum constant subclass body recovery stopped".to_owned(),
                        };
                    break;
                }
            }
        }
        if matches!(
            &enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Refused { .. }
        ) && !enum_constant_body_relations.is_empty()
            && matches!(&execution, ExecutionReport::Complete { .. })
        {
            enum_constant_proof = match prove_enum_constant_body_group(
                &declaration,
                &fields,
                &methods,
                &enum_code_candidates,
                &initializer_candidate_runs,
                &enum_constant_body_relations,
                budget,
            ) {
                Ok(Ok(group)) => crate::enum_constants::ClassSourceEnumConstantProof::Proved(
                    crate::enum_constants::ProvedEnumConstantGroup::Body(group),
                ),
                Ok(Err(reason)) => {
                    crate::enum_constants::ClassSourceEnumConstantProof::Refused { reason }
                }
                Err(error) => {
                    let stop = stop_execution(&error, budget);
                    merge_execution(&mut execution, stop);
                    diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                        reason: format!("enum constant body group proof stopped: {error}"),
                    }
                }
            };
        }
        let mut projection_tail = None;
        if let crate::enum_constants::ClassSourceEnumConstantProof::Proved(
            crate::enum_constants::ProvedEnumConstantGroup::Ordinary(group),
        ) = &enum_constant_proof
        {
            if crate::enum_constants::has_only_terminal_initializer_return(
                group,
                &enum_code_candidates,
                &initializer_candidate_runs,
                &methods,
            ) {
                projection_tail = Some((group.clone(), None));
            } else {
                match crate::enum_constants::prove_static_assignment_suffix(
                    crate::enum_constants::EnumStaticAssignmentInput {
                        group,
                        owner: &declaration.item.declaration.this_class.raw().0,
                        field_headers: &read.facts.fields,
                        source_fields: &fields,
                        method_headers: &read.facts.methods,
                        source_methods: &methods,
                        code_candidates: &enum_code_candidates,
                        initializer_candidates: &initializer_candidate_runs,
                        budget,
                    },
                ) {
                    Ok(Some(suffix)) => {
                        match jarde_java::report::emit_class_initializer_value(
                            &suffix.value,
                            &suffix.initializer_member,
                            budget,
                        ) {
                            Ok(fragment) => {
                                let initializer_text = format!(
                                    "    static {{\n        {}{};\n    }}\n",
                                    suffix.field_name, fragment
                                );
                                projection_tail = Some((
                                    group.clone(),
                                    Some((suffix.field_index, initializer_text)),
                                ));
                            }
                            Err(stop) => {
                                let (stop_execution_report, diagnostic) =
                                    initializer_projection_stop(
                                        &stop,
                                        budget,
                                        class_provenance.clone(),
                                    );
                                merge_execution(&mut execution, stop_execution_report);
                                diagnostics.push(diagnostic);
                                enum_constant_proof =
                                    crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                                        reason: format!(
                                            "enum static suffix expression stopped: {stop:?}"
                                        ),
                                    };
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        let stop = stop_execution(&error, budget);
                        merge_execution(&mut execution, stop);
                        diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                        enum_constant_proof =
                            crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                                reason: format!("enum static suffix proof stopped: {error}"),
                            };
                    }
                }
            }
        }
        let mut enum_projection = match projection_tail {
            Some((group, initializer)) => {
                let mut terminal_constructor_body = None;
                let mut terminal_emission_error = None;
                if let Some(body) = group.constructor_body.as_deref()
                    && let Ok(index) = usize::try_from(group.constructor_method_index)
                    && let Some(method) = methods.get(index)
                {
                    match jarde_java::report::emit_class_enum_constructor_body(
                        &body.candidate,
                        &method.item.identity,
                        budget,
                    ) {
                        Ok(text) => terminal_constructor_body = text,
                        Err(stop) => {
                            terminal_emission_error = Some(enum_projection_stop_error(
                                stop,
                                "enum constructor source emission",
                                "enum_constructor_ir_missing",
                            ));
                        }
                    }
                }
                if let Some(error) = terminal_emission_error {
                    let stop = stop_execution(&error, budget);
                    merge_execution(&mut execution, stop);
                    diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                    enum_constant_proof =
                        crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                            reason: format!("enum constructor source emission stopped: {error}"),
                        };
                    None
                } else {
                    match class_source::prepare_enum_constant_source_projection(
                        &declaration,
                        &fields,
                        &methods,
                        &group,
                        terminal_constructor_body,
                        initializer,
                        budget,
                    ) {
                        Ok(projection) => projection,
                        Err(error) => {
                            let stop = stop_execution(&error, budget);
                            merge_execution(&mut execution, stop);
                            diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                            enum_constant_proof =
                                crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                                    reason: format!("enum source projection stopped: {error}"),
                                };
                            None
                        }
                    }
                }
            }
            None => None,
        };
        if let crate::enum_constants::ClassSourceEnumConstantProof::Proved(
            crate::enum_constants::ProvedEnumConstantGroup::Body(group),
        ) = &enum_constant_proof
        {
            match enum_constant_body_relations
                .first()
                .map(|relation| &relation.group_shape)
            {
                Some(shape) => match class_source::prepare_enum_constant_body_source_projection(
                    &declaration,
                    &fields,
                    &methods,
                    group,
                    shape,
                    budget,
                ) {
                    Ok(Some(projection)) => enum_projection = Some(projection),
                    Ok(None) => {
                        enum_constant_proof =
                            crate::enum_constants::ClassSourceEnumConstantProof::Refused {
                                reason: "a proved enum child body could not be emitted as source"
                                    .to_owned(),
                            };
                    }
                    Err(error) => {
                        let stop = stop_execution(&error, budget);
                        merge_execution(&mut execution, stop);
                        diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                        enum_constant_proof =
                            crate::enum_constants::ClassSourceEnumConstantProof::Stopped {
                                reason: format!("enum child body emission stopped: {error}"),
                            };
                    }
                },
                None => {
                    enum_constant_proof =
                        crate::enum_constants::ClassSourceEnumConstantProof::Refused {
                            reason: "a proved enum body group has no selected relation".to_owned(),
                        };
                }
            }
        }
        let invalid_array_helpers: std::collections::BTreeSet<u64> = array_projection_members
            .iter()
            .filter_map(|(helper_index, member_indices)| {
                let changed = member_indices.iter().any(|member_index| {
                    let originals = array_original_member_texts
                        .iter()
                        .filter(|(index, _)| index == member_index)
                        .collect::<Vec<_>>();
                    let current = methods
                        .iter()
                        .filter(|method| method.item.index == *member_index)
                        .collect::<Vec<_>>();
                    match (originals.as_slice(), current.as_slice()) {
                        ([(_, original)], [method]) => method.text != *original,
                        _ => true,
                    }
                });
                changed.then_some(*helper_index)
            })
            .collect();
        if !invalid_array_helpers.is_empty() {
            let rejected_members: std::collections::BTreeSet<u64> = array_projection_members
                .iter()
                .filter(|(helper_index, _)| invalid_array_helpers.contains(helper_index))
                .flat_map(|(_, members)| members.iter().copied())
                .collect();
            array_helper_method_indices.retain(|index| !invalid_array_helpers.contains(index));
            array_projection_markers = array_helper_method_indices
                .iter()
                .filter_map(|index| {
                    array_projection_members
                        .iter()
                        .position(|(helper_index, _)| helper_index == index)
                        .and_then(|position| array_projection_markers.get(position).cloned())
                })
                .collect();
            array_projection_method_texts.retain(|(index, _)| !rejected_members.contains(index));
            array_projection_members
                .retain(|(helper_index, _)| !invalid_array_helpers.contains(helper_index));
        }
        let text_context = class_source::ClassSourceTextContext {
            initializer_field_order: initializer_field_order.as_deref(),
            declared_methods: read.facts.method_count,
            member_table: read.facts.stopped_at.as_ref(),
            execution: &execution,
            enum_projection: enum_projection.as_ref(),
            array_helper_indices: Some(&array_helper_method_indices),
            array_method_texts: Some(&array_projection_method_texts),
            array_helper_markers: Some(&array_projection_markers),
        };
        let text = class_source::source_text(&declaration, &fields, &methods, &text_context);
        let coverage = class_source_coverage(
            class_view_coverage(search_coverage.as_ref(), &read.facts, structure_complete),
            attempted,
            declared_bodies,
            attempted == declared_bodies,
        );
        Ok((
            ClassSourceReport {
                view,
                class: definition,
                declaration: Some(declaration),
                stages,
                fields,
                methods,
                member_family: class_source::ClassSourceMemberFamily::Absent,
                bridge_proofs,
                enum_switch_proofs,
                initializer_proof,
                enum_constant_proof,
                enum_constant_body_relations,
                text,
                limits: budget.limits().clone(),
                usage: budget.usage(),
                coverage,
                execution: with_usage(execution, budget.usage()),
                diagnostics,
            },
            family_scan,
        ))
    }
}

/// One physical interface field and the facts needed to join it to a same-run `<clinit>` write.
struct InterfaceInitializerField {
    index: u64,
    name: String,
    ty: JavaType,
    has_constant_value: bool,
}

/// The all-or-nothing structural proof for the ordinary-interface field initializer group.
///
/// This consumes the existing member read, source field records and same-run AST sidecars. It
/// performs no class read or recovery, and never consults either report text. The result is only a
/// verdict and origin mapping; source emission remains a later step.
#[allow(clippy::too_many_arguments)]
fn prove_interface_initializer_group(
    declaration: &ClassSourceDeclaration,
    field_headers: &[MemberHeader],
    field_count: u64,
    fields_complete: bool,
    source_fields: &[ClassSourceField],
    constant_value_spellable: &[bool],
    method_headers: &[MemberHeader],
    method_count: u64,
    methods_complete: bool,
    source_methods: &[ClassSourceMethod],
    initializer_candidates: &[jarde_java::report::ClassInitializerCandidates],
    budget: &mut Budget,
) -> Result<ClassSourceInitializerProof> {
    let class_facts = &declaration.item.declaration;
    let class_name = String::from_utf16(class_facts.this_class.utf16()).ok();
    let Some(class_name) = class_name else {
        return Ok(initializer_refused(
            "the interface's internal name is not a Unicode Java name",
        ));
    };
    if !fields_complete
        || !methods_complete
        || u64::try_from(field_headers.len()).unwrap_or(u64::MAX) != field_count
        || source_fields.len() != field_headers.len()
        || constant_value_spellable.len() != field_headers.len()
        || u64::try_from(method_headers.len()).unwrap_or(u64::MAX) != method_count
        || source_methods.len() != method_headers.len()
    {
        return Ok(initializer_refused(
            "the class member table or its published declarations are incomplete",
        ));
    }

    let mut fields = Vec::with_capacity(field_headers.len());
    let mut field_by_identity = std::collections::BTreeMap::<(String, Vec<u8>), usize>::new();
    let mut field_names = std::collections::BTreeSet::<String>::new();
    for (index, (header, source)) in field_headers.iter().zip(source_fields).enumerate() {
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        let Some(name) = String::from_utf16(header.name.utf16()).ok() else {
            return Ok(initializer_refused(format!(
                "field at physical index {index} has no Java name"
            )));
        };
        if !jarde_java::is_java_identifier(&name) || source.declaration.is_none() {
            return Ok(initializer_refused(format!(
                "field `{name}` at physical index {index} has no faithful Java declaration"
            )));
        }
        if !field_names.insert(name.clone()) {
            return Ok(initializer_refused(format!(
                "interface field name `{name}` is ambiguous in Java source"
            )));
        }
        let required_flags = 0x0001 | 0x0008 | 0x0010;
        let forbidden_flags = 0x0002 | 0x0004 | 0x0040 | 0x0080 | 0x4000;
        if header.access_flags & required_flags != required_flags
            || header.access_flags & forbidden_flags != 0
        {
            return Ok(initializer_refused(format!(
                "interface field `{name}` does not have an unambiguous public static final declaration"
            )));
        }
        if source.item.index != u64::try_from(index).unwrap_or(u64::MAX)
            || source.item.name != header.name
            || source.item.descriptor != header.descriptor
            || source.item.access_flags != header.access_flags
        {
            return Ok(initializer_refused(format!(
                "field `{name}` at physical index {index} does not match the published field identity"
            )));
        }
        let descriptor = header.descriptor.raw().0.clone();
        let Some(ty) = interface_field_type(&descriptor) else {
            return Ok(initializer_refused(format!(
                "field `{name}` has a descriptor with no Java source type"
            )));
        };
        let constant_value_count = header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"ConstantValue")
            .count();
        if constant_value_count > 1 {
            return Ok(initializer_refused(format!(
                "field `{name}` has more than one ConstantValue attribute"
            )));
        }
        let has_constant_value = constant_value_count == 1;
        if has_constant_value && !constant_value_spellable[index] {
            return Ok(initializer_refused(format!(
                "field `{name}` declares a ConstantValue that has no complete Java spelling"
            )));
        }
        let key = (name.clone(), descriptor.clone());
        let field_index = fields.len();
        if field_by_identity.insert(key, field_index).is_some() {
            return Ok(initializer_refused(format!(
                "field `{name}` and its descriptor are duplicated"
            )));
        }
        fields.push(InterfaceInitializerField {
            index: source.item.index,
            name,
            ty,
            has_constant_value,
        });
    }

    let clinit_positions: Vec<usize> = method_headers
        .iter()
        .enumerate()
        .filter_map(|(index, method)| (method.name.raw().0 == b"<clinit>").then_some(index))
        .collect();
    if clinit_positions
        .iter()
        .any(|index| method_headers[*index].descriptor.raw().0 != b"()V")
        || clinit_positions.len() > 1
    {
        return Ok(initializer_refused(
            "the class does not have one unique `<clinit>()V` member",
        ));
    }

    let runtime_field_count = fields
        .iter()
        .filter(|field| !field.has_constant_value)
        .count();
    let Some(&clinit_index) = clinit_positions.first() else {
        if runtime_field_count == 0 {
            return Ok(ClassSourceInitializerProof::Proved { fields: Vec::new() });
        }
        return Ok(initializer_refused(
            "the interface has runtime-initialized fields but no `<clinit>()V` member",
        ));
    };
    let clinit_header = &method_headers[clinit_index];
    if clinit_header.access_flags & 0x0008 == 0
        || clinit_header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"Code")
            .count()
            != 1
    {
        return Ok(initializer_refused(
            "the unique `<clinit>()V` is not one complete static Code member",
        ));
    }
    let Some(clinit_method) = source_methods.get(clinit_index) else {
        return Ok(initializer_refused(
            "the `<clinit>()V` declaration was not published",
        ));
    };
    let expected_clinit_name = jarde_reader::model::JvmBytes(b"<clinit>".to_vec());
    let expected_clinit_descriptor = jarde_reader::model::JvmBytes(b"()V".to_vec());
    if clinit_method.item.identity.name != expected_clinit_name
        || clinit_method.item.identity.descriptor != expected_clinit_descriptor
    {
        return Ok(initializer_refused(
            "the published initializer identity does not match `<clinit>()V`",
        ));
    }
    let crate::class_source::ClassSourceOutcome::Recovered { report, analysis } =
        &clinit_method.outcome
    else {
        return Ok(initializer_refused(
            "the `<clinit>()V` body has no completed recovery",
        ));
    };
    if !report.produced()
        || !matches!(analysis.execution, ExecutionReport::Complete { .. })
        || !matches!(report.execution, ExecutionReport::Complete { .. })
        || report.quality != Quality::Structured
        || !report.fallbacks.is_empty()
    {
        return Ok(initializer_refused(
            "the `<clinit>()V` analysis or recovery did not complete without fallback",
        ));
    }
    let matching_candidates: Vec<_> = initializer_candidates
        .iter()
        .filter(|candidates| candidates.member.as_ref() == Some(&clinit_method.item.identity))
        .collect();
    let [candidates] = matching_candidates.as_slice() else {
        return Ok(initializer_refused(
            "the completed `<clinit>()V` run has no unique same-run candidate sequence",
        ));
    };
    if candidates.has_exception_handlers {
        return Ok(initializer_refused(
            "the `<clinit>()V` Code has an exception-table edge or an incomplete handler read",
        ));
    }

    let mut writes_by_field = vec![None::<(usize, u32)>; fields.len()];
    let mut ordered_writes = Vec::<(usize, &jarde_java::report::ClassInitializerFieldWrite)>::new();
    let mut last_write_bci = None;
    let mut saw_return = false;
    for (position, step) in candidates.steps.iter().enumerate() {
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        match step {
            jarde_java::report::ClassInitializerStep::FieldWrite(write) => {
                if write.order != position
                    || write.source.primary().bci() != write.bci
                    || last_write_bci.is_some_and(|previous| previous >= write.bci)
                {
                    return Ok(initializer_refused(
                        "the `<clinit>()V` candidate order does not match its write origins",
                    ));
                }
                last_write_bci = Some(write.bci);
                if write.owner != class_name || !write.is_static {
                    return Ok(initializer_refused(format!(
                        "write at BCI {} does not target a static field of this interface",
                        write.bci
                    )));
                }
                let key = (write.name.clone(), write.descriptor.as_bytes().to_vec());
                let Some(&field_index) = field_by_identity.get(&key) else {
                    return Ok(initializer_refused(format!(
                        "write at BCI {} does not name one field in the complete interface field table",
                        write.bci
                    )));
                };
                let field = &fields[field_index];
                if field.has_constant_value {
                    return Ok(initializer_refused(format!(
                        "write at BCI {} duplicates the ConstantValue initialization of field `{}`",
                        write.bci, field.name
                    )));
                }
                if write.spelled_name != field.name || write.op != jarde_java::ast::AssignOp::Assign
                {
                    return Ok(initializer_refused(format!(
                        "write at BCI {} is not a simple assignment to field `{}`",
                        write.bci, field.name
                    )));
                }
                if writes_by_field[field_index]
                    .replace((write.order, write.bci))
                    .is_some()
                {
                    return Ok(initializer_refused(format!(
                        "field `{}` is written more than once by `<clinit>()V`",
                        field.name
                    )));
                }
                ordered_writes.push((field_index, write));
            }
            jarde_java::report::ClassInitializerStep::Other {
                order,
                kind: jarde_java::report::ClassInitializerStatementKind::Return,
                ..
            } if *order == position && position + 1 == candidates.steps.len() && !saw_return => {
                saw_return = true;
            }
            jarde_java::report::ClassInitializerStep::Other { kind, .. } => {
                return Ok(initializer_refused(format!(
                    "`<clinit>()V` contains an unclaimed top-level effect ({kind:?})"
                )));
            }
        }
    }
    if !saw_return {
        return Ok(initializer_refused(
            "the `<clinit>()V` candidate sequence has no unique trailing normal return",
        ));
    }
    for (index, field) in fields.iter().enumerate() {
        if field.has_constant_value {
            if writes_by_field[index].is_some() {
                return Ok(initializer_refused(format!(
                    "ConstantValue field `{}` also has a `<clinit>()V` write",
                    field.name
                )));
            }
        } else if writes_by_field[index].is_none() {
            return Ok(initializer_refused(format!(
                "runtime field `{}` has no unique `<clinit>()V` write",
                field.name
            )));
        }
    }

    let mut proof_fields = Vec::with_capacity(runtime_field_count);
    for (field_index, write) in ordered_writes {
        let field = &fields[field_index];
        if write.field_reads.is_none() {
            return Ok(initializer_refused(format!(
                "field read identity inside RHS of `{}` is unproved",
                field.name
            )));
        }
        let type_matches = match write.value.presented.as_ref() {
            Some(actual_type) => actual_type == &field.ty,
            None => {
                matches!(write.value.kind, ExprKind::Null)
                    && matches!(field.ty, JavaType::Reference(_))
            }
        };
        if !type_matches {
            let actual = write
                .value
                .presented
                .as_ref()
                .map(JavaType::spell)
                .unwrap_or("unknown");
            return Ok(initializer_refused(format!(
                "RHS of field `{}` is presented as `{}` but its descriptor requires `{}`",
                field.name,
                actual,
                field.ty.spell()
            )));
        }
        let Some(reads) = write.field_reads.as_ref() else {
            unreachable!("the sidecar read identity was checked above")
        };
        let mut read_claims = std::collections::BTreeMap::<
            u32,
            Vec<&jarde_java::report::ClassInitializerFieldRead>,
        >::new();
        for read in reads {
            read_claims.entry(read.bci).or_default().push(read);
        }
        let mut consumed_reads = std::collections::BTreeSet::new();
        let mut own_runtime_reads = Vec::<(u32, usize)>::new();
        let mut expression_state = InitializerExpressionState::default();
        if let Some(reason) = validate_initializer_expression(
            &write.value,
            write.bci,
            &class_name,
            &fields,
            &field_by_identity,
            &read_claims,
            &mut consumed_reads,
            &mut own_runtime_reads,
            &mut expression_state,
            budget,
        )? {
            return Ok(initializer_refused(reason));
        }
        if consumed_reads.len() != reads.len() {
            return Ok(initializer_refused(format!(
                "the AST and field@1 read claims inside RHS of `{}` are not one-to-one",
                field.name
            )));
        }
        if expression_state.has_unknown_static_field {
            return Ok(initializer_refused(format!(
                "RHS of field `{}` reads an external static field whose ConstantValue and source-level initialization behavior are unknown",
                field.name
            )));
        }
        if !expression_state.has_nonconstant_shape {
            return Ok(initializer_refused(format!(
                "RHS of field `{}` is a Java constant expression and would change initialization phase",
                field.name
            )));
        }
        for (read_bci, target_index) in own_runtime_reads {
            let Some((target_order, target_write_bci)) = writes_by_field[target_index] else {
                return Ok(initializer_refused(format!(
                    "read at BCI {read_bci} names a runtime field with no proved write"
                )));
            };
            let source_read_precedes_target_write = write.order <= target_order;
            let bytecode_read_precedes_target_write = read_bci < target_write_bci;
            if source_read_precedes_target_write != bytecode_read_precedes_target_write {
                return Ok(initializer_refused(format!(
                    "read at BCI {read_bci} would observe a different initialization phase after source-order projection"
                )));
            }
        }
        proof_fields.push(ClassSourceInitializerField {
            field_index: field.index,
            write_order: write.order,
            write_bci: write.bci,
        });
    }
    Ok(ClassSourceInitializerProof::Proved {
        fields: proof_fields,
    })
}

enum InitializerProjectionFailure {
    Refused(String),
    Stopped(StopReason),
}

/// Commits an admitted runtime initializer group only after every RHS fragment has been emitted.
/// The returned indices are source order; the `fields` vector itself stays in classfile order.
fn project_interface_initializer_group(
    proof: &ClassSourceInitializerProof,
    fields: &mut [ClassSourceField],
    methods: &[ClassSourceMethod],
    candidate_runs: &[jarde_java::report::ClassInitializerCandidates],
    budget: &mut Budget,
) -> std::result::Result<Option<Vec<usize>>, InitializerProjectionFailure> {
    let ClassSourceInitializerProof::Proved {
        fields: proved_fields,
    } = proof
    else {
        return Ok(None);
    };

    let clinit_methods: Vec<_> = methods
        .iter()
        .filter(|method| {
            method.item.identity.name.0 == b"<clinit>"
                && method.item.identity.descriptor.0 == b"()V"
        })
        .collect();
    let clinit_method = match clinit_methods.as_slice() {
        [method] => Some(*method),
        [] if proved_fields.is_empty() => None,
        [] => {
            return Err(InitializerProjectionFailure::Refused(
                "the proved initializer writes have no retained `<clinit>()V` member".to_owned(),
            ));
        }
        _ => {
            return Err(InitializerProjectionFailure::Refused(
                "the proved initializer group has more than one `<clinit>()V` member".to_owned(),
            ));
        }
    };
    if proved_fields.is_empty() {
        return Ok(Some((0..fields.len()).collect()));
    }
    let Some(clinit_method) = clinit_method else {
        return Err(InitializerProjectionFailure::Refused(
            "the proved initializer writes have no retained `<clinit>()V` member".to_owned(),
        ));
    };

    let matching: Vec<_> = candidate_runs
        .iter()
        .filter(|candidates| candidates.member.as_ref() == Some(&clinit_method.item.identity))
        .collect();
    let candidates = match matching.as_slice() {
        [candidates] => *candidates,
        _ => {
            return Err(InitializerProjectionFailure::Refused(
                "the proved initializer group has no unique retained `<clinit>()V` candidate"
                    .to_owned(),
            ));
        }
    };

    let mut field_order = Vec::with_capacity(fields.len());
    let mut projected_fields = std::collections::BTreeSet::new();
    let mut staged_initializers = Vec::with_capacity(proved_fields.len());
    let mut previous_write_order = None;
    for proved in proved_fields {
        if previous_write_order.is_some_and(|previous| previous >= proved.write_order) {
            return Err(InitializerProjectionFailure::Refused(
                "the proved field list is not in strict `<clinit>()V` write order".to_owned(),
            ));
        }
        previous_write_order = Some(proved.write_order);
        let matching_fields: Vec<_> = fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| (field.item.index == proved.field_index).then_some(index))
            .collect();
        let [physical_index] = matching_fields.as_slice() else {
            return Err(InitializerProjectionFailure::Refused(format!(
                "proved physical field index {} is absent or ambiguous during projection",
                proved.field_index
            )));
        };
        let physical_index = *physical_index;
        if !projected_fields.insert(physical_index) {
            return Err(InitializerProjectionFailure::Refused(format!(
                "physical field index {} occurs more than once in the projection",
                proved.field_index
            )));
        }
        field_order.push(physical_index);

        let Some(jarde_java::report::ClassInitializerStep::FieldWrite(write)) =
            candidates.steps.get(proved.write_order)
        else {
            return Err(InitializerProjectionFailure::Refused(format!(
                "proved write position {} is absent from the retained `<clinit>()V` candidate",
                proved.write_order
            )));
        };
        if write.order != proved.write_order || write.bci != proved.write_bci {
            return Err(InitializerProjectionFailure::Refused(format!(
                "proved field index {} no longer matches write position {} at BCI {}",
                proved.field_index, proved.write_order, proved.write_bci
            )));
        }
        if fields[physical_index].declaration.is_none() {
            return Err(InitializerProjectionFailure::Refused(format!(
                "proved field index {} has no retained declaration",
                proved.field_index
            )));
        }
        let fragment = jarde_java::report::emit_class_initializer_value(
            &write.value,
            &clinit_method.item.identity,
            budget,
        )
        .map_err(InitializerProjectionFailure::Stopped)?;
        staged_initializers.push((physical_index, fragment));
    }

    for (physical_index, _) in fields.iter().enumerate() {
        if projected_fields.insert(physical_index) {
            field_order.push(physical_index);
        }
    }
    if field_order.len() != fields.len() {
        return Err(InitializerProjectionFailure::Refused(
            "the projected source field order is not a complete physical-field permutation"
                .to_owned(),
        ));
    }

    // The plan is complete and every output byte was paid for; only now publish any initializer.
    for (physical_index, fragment) in staged_initializers {
        fields[physical_index]
            .declaration
            .as_mut()
            .expect("the staged declaration was checked")
            .push_str(&fragment);
    }
    Ok(Some(field_order))
}

/// Maps an expression formatter stop into the class-source request's execution and diagnostic
/// planes. The original proof remains available, while the absence of a source order keeps the
/// untouched `<clinit>` presentation in place.
fn initializer_projection_stop(
    stop: &StopReason,
    budget: &Budget,
    provenance: Option<Provenance>,
) -> (ExecutionReport, Diagnostic) {
    let usage = budget.usage();
    let (execution, code, severity, message) = match stop {
        StopReason::Budget {
            dimension,
            written,
            limit,
            at,
        } => {
            let dimension = jarde_reader::budget::BudgetDimension::from(*dimension);
            (
                ExecutionReport::Partial {
                    reason: TerminationReason::BudgetExceeded { dimension },
                    usage,
                },
                format!("budget_exceeded_{}", budget_dimension_code(dimension)),
                DiagnosticSeverity::Error,
                format!(
                    "initializer projection stopped on {dimension:?} after {written} byte(s) of {limit}, at {}",
                    at.map_or("no node".to_owned(), |bci| format!("BCI {bci}"))
                ),
            )
        }
        StopReason::Cancelled { at } => (
            ExecutionReport::Cancelled { usage },
            "jre_cancelled".to_owned(),
            DiagnosticSeverity::Warning,
            format!(
                "initializer projection was cancelled{}",
                at.map_or(String::new(), |bci| format!(" at BCI {bci}"))
            ),
        ),
        StopReason::Interrupted { code, at } => (
            ExecutionReport::Partial {
                reason: TerminationReason::Error {
                    code: (*code).to_owned(),
                },
                usage,
            },
            (*code).to_owned(),
            DiagnosticSeverity::Error,
            format!(
                "initializer projection stopped ({code}){}",
                at.map_or(String::new(), |bci| format!(" at BCI {bci}"))
            ),
        ),
        StopReason::EvidenceRefused { code, at, message } => (
            ExecutionReport::Partial {
                reason: TerminationReason::Unsupported {
                    code: (*code).to_owned(),
                },
                usage,
            },
            (*code).to_owned(),
            DiagnosticSeverity::Error,
            format!(
                "initializer projection was refused: {message}{}",
                at.map_or(String::new(), |bci| format!(" at BCI {bci}"))
            ),
        ),
        StopReason::IrTableMissing { table } => (
            ExecutionReport::Partial {
                reason: TerminationReason::Unsupported {
                    code: "jre_ir_table_missing".to_owned(),
                },
                usage,
            },
            "jre_ir_table_missing".to_owned(),
            DiagnosticSeverity::Error,
            format!(
                "initializer projection has no {table} table, so its expression cannot be emitted"
            ),
        ),
    };
    (
        execution,
        Diagnostic {
            code,
            severity,
            message,
            provenance,
        },
    )
}

/// Returns a reason in the report's existing refusal vocabulary without making a partial proof.
fn initializer_refused(reason: impl Into<String>) -> ClassSourceInitializerProof {
    ClassSourceInitializerProof::Refused {
        reason: reason.into(),
    }
}

fn interface_field_type(descriptor: &[u8]) -> Option<JavaType> {
    let facts = descriptor_facts(descriptor, DescriptorKind::Field).ok()?;
    type_of_component(facts.single()?)
}

fn is_java_constant_variable_type(ty: &JavaType) -> bool {
    matches!(
        ty,
        JavaType::Boolean
            | JavaType::Byte
            | JavaType::Char
            | JavaType::Short
            | JavaType::Int
            | JavaType::Long
            | JavaType::Float
            | JavaType::Double
    ) || matches!(ty, JavaType::Reference(name) if name == "java.lang.String")
}

#[derive(Default)]
struct InitializerExpressionState {
    has_nonconstant_shape: bool,
    has_unknown_static_field: bool,
    effect_bcis: std::collections::BTreeSet<u32>,
}

/// Walks one RHS AST once, matching every field node to its exact sidecar read and finding whether
/// the emitted source could be a Java constant expression. The explicit stack keeps a deep but
/// already-bounded expression from adding recursion to class-source assembly.
#[allow(clippy::too_many_arguments)]
fn validate_initializer_expression(
    root: &Expr,
    write_bci: u32,
    class_name: &str,
    fields: &[InterfaceInitializerField],
    field_by_identity: &std::collections::BTreeMap<(String, Vec<u8>), usize>,
    read_claims: &std::collections::BTreeMap<
        u32,
        Vec<&jarde_java::report::ClassInitializerFieldRead>,
    >,
    consumed_reads: &mut std::collections::BTreeSet<u32>,
    own_runtime_reads: &mut Vec<(u32, usize)>,
    state: &mut InitializerExpressionState,
    budget: &mut Budget,
) -> Result<Option<String>> {
    let mut pending = vec![root];
    while let Some(expression) = pending.pop() {
        budget.charge(CountedBudgetDimension::IrItems, 1)?;
        match &expression.kind {
            ExprKind::Field { receiver, name } => {
                let read_bci = expression.origin.primary().bci();
                if read_bci >= write_bci {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} is not before its owning putstatic at BCI {write_bci}"
                    )));
                }
                let Some([claim]) = read_claims.get(&read_bci).map(Vec::as_slice) else {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} has no unique field@1 claim"
                    )));
                };
                if !consumed_reads.insert(read_bci)
                    || claim.name != *name
                    || !claim.is_static && claim.owner == class_name
                {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} does not match its unique claim"
                    )));
                }
                let Some(claim_type) = interface_field_type(claim.descriptor.as_bytes()) else {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} has an unspellable claimed descriptor"
                    )));
                };
                if expression.presented.as_ref() != Some(&claim_type) {
                    return Ok(Some(format!(
                        "field read at BCI {read_bci} does not have its claimed descriptor type"
                    )));
                }
                let expected_owner_path = claim.owner.replace('/', ".");
                if claim.is_static {
                    if !matches!(&receiver.kind, ExprKind::Path(path) if *path == expected_owner_path)
                    {
                        return Ok(Some(format!(
                            "static field read at BCI {read_bci} is not qualified by its proven owner `{expected_owner_path}`"
                        )));
                    }
                } else {
                    let expected_receiver = JavaType::Reference(expected_owner_path);
                    if receiver.presented.as_ref() != Some(&expected_receiver) {
                        return Ok(Some(format!(
                            "instance field read at BCI {read_bci} has no receiver of its proven owner type"
                        )));
                    }
                    pending.push(receiver);
                }
                if claim.owner == class_name {
                    let key = (claim.name.clone(), claim.descriptor.as_bytes().to_vec());
                    let Some(&target_index) = field_by_identity.get(&key) else {
                        return Ok(Some(format!(
                            "same-interface field read at BCI {read_bci} does not resolve to one declared field"
                        )));
                    };
                    let field = &fields[target_index];
                    if field.has_constant_value {
                        // A ConstantValue with a primitive or String type is a Java constant
                        // variable. Other reference-typed ConstantValue attributes (legal as class
                        // file data but not Java constant variables) still require a field read.
                        if !is_java_constant_variable_type(&field.ty) {
                            state.has_nonconstant_shape = true;
                        }
                    } else {
                        own_runtime_reads.push((read_bci, target_index));
                        state.has_nonconstant_shape = true;
                    }
                } else if !claim.is_static {
                    state.has_nonconstant_shape = true;
                } else {
                    state.has_unknown_static_field = true;
                }
            }
            ExprKind::Call { receiver, args, .. } => {
                if let Some(reason) = record_initializer_effect(expression, write_bci, state) {
                    return Ok(Some(reason));
                }
                state.has_nonconstant_shape = true;
                for argument in args.iter().rev() {
                    pending.push(argument);
                }
                if let Some(receiver) = receiver {
                    pending.push(receiver);
                }
            }
            ExprKind::New {
                qualifier, args, ..
            } => {
                if let Some(reason) = record_initializer_effect(expression, write_bci, state) {
                    return Ok(Some(reason));
                }
                state.has_nonconstant_shape = true;
                for argument in args.iter().rev() {
                    pending.push(argument);
                }
                if let Some(qualifier) = qualifier {
                    pending.push(qualifier);
                }
            }
            ExprKind::NewArray {
                lengths,
                initializers,
                ..
            } => {
                if let Some(reason) = record_initializer_effect(expression, write_bci, state) {
                    return Ok(Some(reason));
                }
                state.has_nonconstant_shape = true;
                if let Some(initializers) = initializers {
                    pending.extend(initializers.iter().rev());
                }
                pending.extend(lengths.iter().rev());
            }
            ExprKind::Index { array, index } => {
                if let Some(reason) = record_initializer_effect(expression, write_bci, state) {
                    return Ok(Some(reason));
                }
                state.has_nonconstant_shape = true;
                pending.push(index);
                pending.push(array);
            }
            ExprKind::PostIncrement { .. } => {
                return Ok(Some(
                    "a postfix update has no interface-initializer evaluation proof".to_owned(),
                ));
            }
            ExprKind::ArrayLength { array } => {
                if let Some(reason) = record_initializer_effect(expression, write_bci, state) {
                    return Ok(Some(reason));
                }
                state.has_nonconstant_shape = true;
                pending.push(array);
            }
            ExprKind::Cast { ty, value } => {
                if !matches!(
                    ty,
                    JavaType::Boolean
                        | JavaType::Byte
                        | JavaType::Char
                        | JavaType::Short
                        | JavaType::Int
                        | JavaType::Long
                        | JavaType::Float
                        | JavaType::Double
                ) && !matches!(ty, JavaType::Reference(name) if name == "java.lang.String")
                {
                    state.has_nonconstant_shape = true;
                }
                pending.push(value);
            }
            ExprKind::Not { value } | ExprKind::Neg { value } => pending.push(value),
            ExprKind::InstanceOf { value, .. } => {
                state.has_nonconstant_shape = true;
                pending.push(value);
            }
            ExprKind::Binary { left, right, .. } => {
                pending.push(right);
                pending.push(left);
            }
            ExprKind::Concat { parts } => {
                pending.extend(parts.iter().rev().map(|part| &part.value));
            }
            ExprKind::Lambda { .. } | ExprKind::MethodReference { .. } => {
                return Ok(Some(
                    "a lambda or method reference has no complete initializer type and evaluation proof".to_owned(),
                ));
            }
            ExprKind::Conditional { .. } => {
                return Ok(Some(
                    "conditional expression constant-expression phase proof is unavailable"
                        .to_owned(),
                ));
            }
            ExprKind::Local(name) => {
                return Ok(Some(format!("RHS refers to unscoped local `{name}`")));
            }
            ExprKind::QualifiedThis { .. } => {
                return Ok(Some("RHS refers to a lexical outer instance".to_owned()));
            }
            ExprKind::Null | ExprKind::ClassLiteral { .. } => {
                state.has_nonconstant_shape = true;
            }
            ExprKind::Integer(_)
            | ExprKind::Boolean(_)
            | ExprKind::Long(_)
            | ExprKind::Float(_)
            | ExprKind::Double(_)
            | ExprKind::Str(_)
            | ExprKind::Path(_)
            | ExprKind::Super { .. } => {}
        }
    }
    Ok(None)
}

fn record_initializer_effect(
    expression: &Expr,
    write_bci: u32,
    state: &mut InitializerExpressionState,
) -> Option<String> {
    let bci = expression.origin.primary().bci();
    if bci >= write_bci {
        return Some(format!(
            "RHS effect at BCI {bci} is not before its owning putstatic at BCI {write_bci}"
        ));
    }
    if !state.effect_bcis.insert(bci) {
        return Some(format!(
            "RHS effect at BCI {bci} would be evaluated more than once"
        ));
    }
    None
}

/// Whether this presentation runs a body for one member record: it declares a `Code` attribute whose
/// content can be decoded and its descriptor is one this presentation can read.
///
/// This is the one predicate behind two decisions of [`Engine::class_source`] — how many members the
/// class has that need the preparation, and which members are sent to the prepared class for a run —
/// so the count a report publishes for its body plane and the runs it really performs cannot drift
/// apart.
fn class_source_runs_body(member: &MemberHeader) -> bool {
    code_shell(member).is_some() && class_source::spellable_descriptor(&member.descriptor.raw().0)
}

fn scan_array_helper_uses(
    ir: &jarde_jvm::method_ir::MethodIr,
    capture_bootstrap_table: bool,
    budget: &mut Budget,
) -> Result<Option<ArrayHelperUseScan>> {
    use jarde_reader::classfile::CpEntryKind as K;

    let Some(code) = ir.code() else {
        return Ok(None);
    };
    let Some(declaration) = ir.declaration() else {
        return Ok(Some(ArrayHelperUseScan::default()));
    };
    let instruction_cost = u64::try_from(code.instructions.len()).unwrap_or(u64::MAX);
    let bootstrap_cost = if capture_bootstrap_table {
        ir.bootstrap_methods().iter().fold(0_u64, |sum, bootstrap| {
            sum.saturating_add(1 + u64::try_from(bootstrap.arguments.len()).unwrap_or(u64::MAX))
        })
    } else {
        0
    };
    budget.charge(
        CountedBudgetDimension::IrItems,
        instruction_cost.saturating_add(bootstrap_cost),
    )?;
    let pool = ir.constant_pool();
    let identity = declaration.identity().clone();
    let mut scan = ArrayHelperUseScan {
        complete: code.stopped_at.is_none(),
        member: Some(identity),
        ..ArrayHelperUseScan::default()
    };
    for (instruction, operands) in code.instructions.iter().zip(code.operands()) {
        let opcode = operands.effective_opcode;
        let Some(index) = operands.constant_pool_index else {
            continue;
        };
        let entry = match jarde_reader::classfile::cp_entry(pool, index) {
            Ok(entry) => entry,
            Err(_) => {
                scan.complete = false;
                continue;
            }
        };
        if (0xb6..=0xb9).contains(&opcode) {
            if let Some(target) = method_reference_identity(pool, index) {
                scan.direct_calls.push(target);
            } else {
                scan.complete = false;
            }
        } else if matches!(opcode, 0x12..=0x14) {
            match &entry.kind {
                K::MethodHandle { .. } => {
                    if let Some(target) = method_handle_identity(pool, index) {
                        scan.ldc_handles.push(target);
                    } else {
                        scan.complete = false;
                    }
                }
                K::Dynamic {
                    bootstrap_method_attr_index,
                    ..
                } => {
                    scan.dynamic_bootstrap_indices
                        .push(*bootstrap_method_attr_index);
                }
                _ => {}
            }
        } else if opcode == 0xba {
            match &entry.kind {
                K::InvokeDynamic {
                    bootstrap_method_attr_index,
                    ..
                } => scan.invokedynamic_sites.push((
                    instruction.bci,
                    index,
                    *bootstrap_method_attr_index,
                )),
                _ => scan.complete = false,
            }
        }
    }
    if capture_bootstrap_table {
        for (bootstrap_index, bootstrap) in ir.bootstrap_methods().iter().enumerate() {
            let bootstrap_index = u16::try_from(bootstrap_index).unwrap_or(u16::MAX);
            let bootstrap_handle = method_handle_identity(pool, bootstrap.method_ref);
            if bootstrap_handle.is_none() {
                scan.complete = false;
            }
            let mut arguments = Vec::with_capacity(bootstrap.arguments.len());
            for cp_index in bootstrap.arguments.iter().copied() {
                match jarde_reader::classfile::cp_entry(pool, cp_index).map(|entry| &entry.kind) {
                    Ok(K::MethodHandle { .. }) => {
                        if let Some(target) = method_handle_identity(pool, cp_index) {
                            arguments
                                .push(ArrayBootstrapArgument::MethodHandle { cp_index, target });
                        } else {
                            scan.complete = false;
                        }
                    }
                    Ok(K::Dynamic {
                        bootstrap_method_attr_index,
                        ..
                    }) => arguments.push(ArrayBootstrapArgument::Dynamic {
                        bootstrap_index: *bootstrap_method_attr_index,
                    }),
                    Ok(_) => arguments.push(ArrayBootstrapArgument::Other),
                    Err(_) => scan.complete = false,
                }
            }
            scan.bootstrap_rows.push(ArrayBootstrapRow {
                index: bootstrap_index,
                bootstrap: bootstrap_handle,
                arguments,
            });
        }
    }
    Ok(Some(scan))
}

fn method_reference_identity(
    pool: &[jarde_reader::classfile::CpEntryFacts],
    index: u16,
) -> Option<RawMethodReference> {
    use jarde_reader::classfile::CpEntryKind as K;
    match &jarde_reader::classfile::cp_entry(pool, index).ok()?.kind {
        K::MethodRef {
            owner,
            name,
            descriptor,
            ..
        }
        | K::InterfaceMethodRef {
            owner,
            name,
            descriptor,
            ..
        } => Some(RawMethodReference {
            owner: owner.clone(),
            name: name.clone(),
            descriptor: descriptor.clone(),
        }),
        _ => None,
    }
}

fn method_handle_identity(
    pool: &[jarde_reader::classfile::CpEntryFacts],
    index: u16,
) -> Option<RawMethodReference> {
    use jarde_reader::classfile::CpEntryKind as K;
    let K::MethodHandle {
        reference_index, ..
    } = &jarde_reader::classfile::cp_entry(pool, index).ok()?.kind
    else {
        return None;
    };
    method_reference_identity(pool, *reference_index)
}

fn same_raw_method(
    reference: &RawMethodReference,
    candidate: &jarde_java::report::ClassSourceArrayConstructorCandidate,
) -> bool {
    same_raw_target(
        reference,
        &RawMethodReference {
            owner: candidate.helper_owner.clone(),
            name: candidate.helper.name.clone(),
            descriptor: candidate.helper.descriptor.clone(),
        },
    )
}

fn same_raw_target(reference: &RawMethodReference, helper: &RawMethodReference) -> bool {
    reference.owner == helper.owner
        && reference.name == helper.name
        && reference.descriptor == helper.descriptor
}

fn array_helper_has_direct_use(helper: &RawMethodReference, scans: &[ArrayHelperUseScan]) -> bool {
    scans.iter().any(|scan| {
        scan.direct_calls
            .iter()
            .chain(&scan.ldc_handles)
            .any(|reference| same_raw_target(reference, helper))
    })
}

#[cfg(test)]
mod array_helper_census_tests {
    use super::*;

    fn method_reference(owner: &[u8], name: &[u8], descriptor: &[u8]) -> RawMethodReference {
        RawMethodReference {
            owner: jarde_reader::model::JvmBytes(owner.to_vec()),
            name: jarde_reader::model::JvmBytes(name.to_vec()),
            descriptor: jarde_reader::model::JvmBytes(descriptor.to_vec()),
        }
    }

    #[test]
    fn direct_invocation_and_ldc_handle_refuse_the_exact_physical_helper() {
        let helper = method_reference(b"p/Subject", b"lambda$arrayCtor$0", b"(I)[I");
        let mut scan = ArrayHelperUseScan::default();
        scan.direct_calls.push(helper.clone());
        assert!(array_helper_has_direct_use(
            &helper,
            std::slice::from_ref(&scan)
        ));

        scan.direct_calls.clear();
        scan.ldc_handles.push(helper.clone());
        assert!(array_helper_has_direct_use(
            &helper,
            std::slice::from_ref(&scan)
        ));

        scan.ldc_handles[0].descriptor = jarde_reader::model::JvmBytes(b"(J)[I".to_vec());
        assert!(!array_helper_has_direct_use(
            &helper,
            std::slice::from_ref(&scan)
        ));
    }
}

fn array_helper_census_refusal(
    helper: &jarde_java::report::ClassSourceArrayConstructorCandidate,
    candidates: &[jarde_java::report::ClassSourceArrayConstructorCandidate],
    scans: &[ArrayHelperUseScan],
    budget: &mut Budget,
) -> Result<Option<String>> {
    if candidates.is_empty() || scans.is_empty() {
        return Ok(Some("the same-run class use census is absent".to_owned()));
    }
    let Some(first) = scans.first() else {
        return Ok(Some("the same-run class use census is absent".to_owned()));
    };
    let setup_cost = u64::try_from(scans.len())
        .unwrap_or(u64::MAX)
        .saturating_add(u64::try_from(first.bootstrap_rows.len()).unwrap_or(u64::MAX))
        .saturating_add(u64::try_from(candidates.len()).unwrap_or(u64::MAX));
    budget.charge(CountedBudgetDimension::IrItems, setup_cost)?;
    let row_count = u64::try_from(first.bootstrap_rows.len()).unwrap_or(u64::MAX);
    let argument_count = first.bootstrap_rows.iter().fold(0_u64, |sum, row| {
        sum.saturating_add(u64::try_from(row.arguments.len()).unwrap_or(u64::MAX))
    });
    let scan_count = u64::try_from(scans.len()).unwrap_or(u64::MAX);
    let reference_count = scans.iter().fold(0_u64, |sum, scan| {
        sum.saturating_add(u64::try_from(scan.direct_calls.len()).unwrap_or(u64::MAX))
            .saturating_add(u64::try_from(scan.ldc_handles.len()).unwrap_or(u64::MAX))
    });
    let indy_count = scans.iter().fold(0_u64, |sum, scan| {
        sum.saturating_add(u64::try_from(scan.invokedynamic_sites.len()).unwrap_or(u64::MAX))
    });
    let dynamic_count = scans.iter().fold(0_u64, |sum, scan| {
        sum.saturating_add(u64::try_from(scan.dynamic_bootstrap_indices.len()).unwrap_or(u64::MAX))
    });
    let candidate_site_count = candidates.iter().fold(0_u64, |sum, candidate| {
        sum.saturating_add(u64::try_from(candidate.sites.len()).unwrap_or(u64::MAX))
    });
    let candidate_count = u64::try_from(candidates.len()).unwrap_or(u64::MAX);
    let work = row_count
        .saturating_mul(row_count)
        .saturating_mul(2)
        .saturating_add(argument_count)
        .saturating_add(reference_count)
        .saturating_add(scan_count.saturating_mul(row_count.saturating_add(argument_count)))
        .saturating_add(indy_count)
        .saturating_add(dynamic_count)
        .saturating_add(
            candidate_site_count.saturating_mul(
                scan_count
                    .saturating_add(indy_count)
                    .saturating_add(row_count)
                    .saturating_add(1),
            ),
        )
        .saturating_add(
            argument_count
                .saturating_mul(scan_count)
                .saturating_mul(indy_count)
                .saturating_mul(candidate_site_count.saturating_add(1)),
        )
        .saturating_add(argument_count.saturating_mul(candidate_count))
        .saturating_add(candidate_count);
    budget.charge(CountedBudgetDimension::IrItems, work)?;

    let helper_target = RawMethodReference {
        owner: helper.helper_owner.clone(),
        name: helper.helper.name.clone(),
        descriptor: helper.helper.descriptor.clone(),
    };
    for scan in scans {
        budget.poll()?;
        if !scan.complete || scan.member.is_none() {
            return Ok(Some(
                "at least one same-run method Code/use scan is incomplete".to_owned(),
            ));
        }
        if array_helper_has_direct_use(&helper_target, std::slice::from_ref(scan)) {
            return Ok(Some(
                "the helper has a direct invocation or ldc MethodHandle use".to_owned(),
            ));
        }
    }
    for scan in scans {
        budget.poll()?;
        if !scan.bootstrap_rows.is_empty() && scan.bootstrap_rows != first.bootstrap_rows {
            return Ok(Some(
                "same-run method payloads disagree about the class bootstrap table".to_owned(),
            ));
        }
    }

    let mut reachable = std::collections::BTreeSet::new();
    for scan in scans {
        budget.poll()?;
        for (_, _, bootstrap_index) in &scan.invokedynamic_sites {
            budget.poll()?;
            reachable.insert(*bootstrap_index);
        }
        reachable.extend(scan.dynamic_bootstrap_indices.iter().copied());
    }
    let mut queue = reachable
        .iter()
        .copied()
        .collect::<std::collections::VecDeque<_>>();
    while let Some(index) = queue.pop_front() {
        budget.poll()?;
        if let Some(row) = first.bootstrap_rows.iter().find(|row| row.index == index) {
            for argument in &row.arguments {
                budget.poll()?;
                if let ArrayBootstrapArgument::Dynamic { bootstrap_index } = argument
                    && reachable.insert(*bootstrap_index)
                {
                    queue.push_back(*bootstrap_index);
                }
            }
        }
    }
    if reachable
        .iter()
        .any(|index| !first.bootstrap_rows.iter().any(|row| row.index == *index))
    {
        return Ok(Some(
            "a reachable dynamic site names a missing bootstrap row".to_owned(),
        ));
    }

    let mut accepted = Vec::new();
    for candidate in candidates {
        budget.poll()?;
        if candidate.helper != helper.helper || candidate.helper_owner != helper.helper_owner {
            return Ok(Some(
                "the grouped sites do not name one exact physical helper".to_owned(),
            ));
        }
        for site in &candidate.sites {
            budget.poll()?;
            let Some(scan) = scans
                .iter()
                .find(|scan| scan.member.as_ref() == Some(&candidate.member))
            else {
                return Ok(Some(
                    "a projected site has no complete same-run method scan".to_owned(),
                ));
            };
            if !scan.invokedynamic_sites.contains(&(
                site.use_site,
                site.site_cp,
                candidate.bootstrap_index,
            )) {
                return Ok(Some(
                    "a projected site does not match its exact BCI/CP/bootstrap tuple".to_owned(),
                ));
            }
            let Some(row) = first
                .bootstrap_rows
                .iter()
                .find(|row| row.index == candidate.bootstrap_index)
            else {
                return Ok(Some("a projected site has no bootstrap row".to_owned()));
            };
            if !matches!(
                row.arguments.get(1),
                Some(ArrayBootstrapArgument::MethodHandle { cp_index, target })
                    if *cp_index == candidate.implementation_index
                        && same_raw_method(target, helper)
            ) {
                return Ok(Some(
                    "the exact bootstrap implementation argument does not name the helper"
                        .to_owned(),
                ));
            }
            accepted.push((
                candidate.member.clone(),
                site.use_site,
                site.site_cp,
                candidate.bootstrap_index,
                candidate.implementation_index,
            ));
        }
    }

    for bootstrap_index in &reachable {
        budget.poll()?;
        let Some(row) = first
            .bootstrap_rows
            .iter()
            .find(|row| row.index == *bootstrap_index)
        else {
            return Ok(Some("a reachable bootstrap row is absent".to_owned()));
        };
        if row
            .bootstrap
            .as_ref()
            .is_some_and(|reference| same_raw_method(reference, helper))
        {
            return Ok(Some(
                "a reachable bootstrap method handle names the helper".to_owned(),
            ));
        }
        for (argument_index, argument) in row.arguments.iter().enumerate() {
            budget.poll()?;
            if let ArrayBootstrapArgument::MethodHandle { cp_index, target } = argument
                && same_raw_method(target, helper)
            {
                if argument_index != 1
                    || !candidates.iter().any(|candidate| {
                        candidate.bootstrap_index == *bootstrap_index
                            && candidate.implementation_index == *cp_index
                    })
                {
                    return Ok(Some(
                        "a reachable bootstrap row has an additional helper handle use".to_owned(),
                    ));
                }
                for scan in scans {
                    budget.poll()?;
                    for (bci, site_cp, site_bootstrap) in &scan.invokedynamic_sites {
                        budget.poll()?;
                        let Some(member) = scan.member.clone() else {
                            return Ok(Some("a method scan has no physical identity".to_owned()));
                        };
                        if site_bootstrap == bootstrap_index
                            && !accepted.contains(&(
                                member,
                                *bci,
                                *site_cp,
                                *site_bootstrap,
                                *cp_index,
                            ))
                        {
                            return Ok(Some(
                                "a bootstrap row also serves an unprojected invokedynamic site"
                                    .to_owned(),
                            ));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

fn array_target_declaration_matches(
    candidate: &jarde_java::report::ClassSourceArrayConstructorCandidate,
    method: &ClassSourceMethod,
) -> bool {
    let (Some(declaration), Ok(method_name)) = (
        method.declaration.as_deref(),
        std::str::from_utf8(&method.item.name.raw().0),
    ) else {
        return false;
    };
    if declaration.contains('<') || declaration.contains('>') {
        return false;
    }
    let Some(header) = declaration.split('(').next() else {
        return false;
    };
    let Some((return_prefix, _)) = header.rsplit_once(method_name) else {
        return false;
    };
    let Some(spelled_type) = return_prefix.split_whitespace().last() else {
        return false;
    };
    candidate.sites.iter().all(|site| {
        let target_simple = site
            .target_type
            .rsplit('.')
            .next()
            .unwrap_or(&site.target_type);
        let descriptor_return = format!("L{};", site.target_type.replace('.', "/"));
        let descriptor_matches = method
            .item
            .descriptor
            .raw()
            .0
            .split(|byte| *byte == b')')
            .next_back()
            .is_some_and(|result| result == descriptor_return.as_bytes());
        descriptor_matches && (spelled_type == site.target_type || spelled_type == target_simple)
    })
}

/// How one class-source request produces the bodies of the members that declare one (task 7.3).
///
/// Exactly one of these is decided per request, before the member loop: the class is prepared once
/// and every body is decoded against that preparation, or the one attempt to read it failed and that
/// failure is the refusal each of those members states.
enum ClassBodies<'a> {
    /// The class was prepared: [`recover_prepared_member`] runs each member body against it, and the
    /// member's own identity decides which record of that class that is.
    Prepared(&'a jarde_reader::prepared::PreparedClass<'a>),
    /// The class could not be prepared, and this is why: the reader's own stop, the diagnostic that
    /// names it, and whether it ends the request. Boxed because one request builds one of these and
    /// an outcome of a prepared class is a borrow.
    Refused(Box<ClassBodyRefusal>),
    /// No member of this class declares a body this presentation would run, so nothing was prepared
    /// and no member reaches a body run.
    NotNeeded,
}

/// The refusal of one class's preparation, as the members that declare a body state it.
struct ClassBodyRefusal {
    /// The reader's own stop, in the vocabulary every report of this engine states a stop in.
    stop: ExecutionReport,
    /// The diagnostic that names the failure, published inside each member that states it.
    diagnostic: Diagnostic,
    /// Whether the failure ends the request: a cancellation or an exhausted shared dimension does,
    /// and a class the reader's strict structure read refuses does not.
    ends: bool,
}

/// One member body recovered from the class this request prepared (task 7.3).
///
/// This is [`Engine::recover_method`] with its driver read supplied by the caller's preparation: the
/// same one analysis run ([`jarde_jvm::analyze_prepared_method_ir`] is the prepared half of
/// [`jarde_jvm::analyze_method_ir`]) and the same presentation ([`recovery_presented`] with the
/// prepared class), over the same stages. What the preparation changes is what that run reads — the
/// class's declaration, pool and member table were read once for the whole presentation, so this run
/// charges its body attempt and its decode and no class read at all — and where its on-demand callee
/// evidence comes from: [`jarde_jvm::callee::read_prepared_callees`], which reads no class either.
/// A member recovered here and the same member recovered through [`Engine::recover_method`] share
/// the selected facts and decoder. Class-source retains a proved array helper's ordinary lambda
/// call until class-level source projection can also omit its declaration; method-only recovery may
/// write `T[]::new` from the same proof.
struct PreparedMemberRecovery {
    recovered: RecoveredMethod,
    initializer: Option<jarde_java::report::ClassInitializerCandidates>,
    enum_constructor: Option<jarde_java::report::ClassEnumConstructorCandidates>,
    bridge: Option<jarde_java::bridge::ClassSourceBridgeCandidate>,
    enum_switches: Option<Vec<jarde_java::enumswitch::ClassSourceEnumSwitchCandidate>>,
    array_constructors: Option<Vec<jarde_java::report::ClassSourceArrayConstructorCandidate>>,
    array_helper_uses: Option<ArrayHelperUseScan>,
    enum_switch_field_uses: Option<Vec<jarde_java::report::ClassSourceEnumSwitchFieldUse>>,
    generic_return: Option<jarde_java::report::GenericReturnCandidate>,
    generic_constructor: Option<jarde_java::report::GenericConstructorCandidate>,
    anonymous_allocations: Option<jarde_java::report::AnonymousAllocationScan>,
    enum_code: Option<crate::enum_constants::EnumMethodCodeCandidate>,
}

#[derive(Clone, Debug, Default)]
struct ArrayHelperUseScan {
    complete: bool,
    member: Option<PhysicalMethodId>,
    direct_calls: Vec<RawMethodReference>,
    ldc_handles: Vec<RawMethodReference>,
    bootstrap_rows: Vec<ArrayBootstrapRow>,
    invokedynamic_sites: Vec<(u32, u16, u16)>,
    dynamic_bootstrap_indices: Vec<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RawMethodReference {
    owner: jarde_reader::model::JvmBytes,
    name: jarde_reader::model::JvmBytes,
    descriptor: jarde_reader::model::JvmBytes,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ArrayBootstrapRow {
    index: u16,
    bootstrap: Option<RawMethodReference>,
    arguments: Vec<ArrayBootstrapArgument>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ArrayBootstrapArgument {
    MethodHandle {
        cp_index: u16,
        target: RawMethodReference,
    },
    Dynamic {
        bootstrap_index: u16,
    },
    Other,
}

struct PreparedMemberOptions {
    prove_generic_return: bool,
    capture_enum_group_code: bool,
    capture_enum_constructor_ast: bool,
    array_helper_census_needed: bool,
    capture_array_helper_use_table: bool,
}

fn recover_prepared_member(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    prepared: &jarde_reader::prepared::PreparedClass<'_>,
    assembly_context: &class_source::ClassSourceAssemblyContext,
    method_index: u64,
    evidence: &RecoveryEvidenceRequest,
    options: PreparedMemberOptions,
    budget: &mut Budget,
) -> Result<PreparedMemberRecovery> {
    let analyzed = jarde_jvm::analyze_prepared_method_ir(content, prepared, request, budget)?;
    let array_helper_uses = if options.array_helper_census_needed {
        scan_array_helper_uses(
            analyzed.ir(),
            options.capture_array_helper_use_table,
            budget,
        )?
    } else {
        None
    };
    let enum_code_candidate = if options.capture_enum_group_code {
        crate::enum_constants::capture_method_code(
            method_index,
            &request.method,
            analyzed.ir(),
            budget,
        )?
    } else {
        None
    };
    if analyzed.ir().code().is_some() {
        // The prepared half of the same demand-path decode (`crate::d0_counts`): one count per
        // member body this presentation really decoded.
        crate::d0_counts::body_decoded();
    }
    let (
        recovered,
        initializer,
        enum_constructor,
        bridge,
        enum_switches,
        array_constructors,
        enum_switch_field_uses,
        generic_return,
        generic_constructor,
        anonymous_allocations,
    ) = recovery_presented_for_class_source(
        content,
        request,
        analyzed,
        prepared,
        assembly_context,
        evidence,
        options.prove_generic_return,
        options.capture_enum_constructor_ast,
        budget,
    )?;
    // The constructor AST is an evidence handoff from this exact run. Keep it only when both the
    // analysis and the class-source presentation completed and the latter committed its report;
    // otherwise a budget stop could leave an apparently usable partial candidate beside a refused
    // member result.
    let enum_constructor = enum_constructor.filter(|_| {
        matches!(
            &recovered.analysis().execution,
            ExecutionReport::Complete { .. }
        ) && recovered.recovery().produced()
            && matches!(
                &recovered.recovery().execution,
                ExecutionReport::Complete { .. }
            )
    });
    Ok(PreparedMemberRecovery {
        recovered,
        initializer,
        enum_constructor,
        bridge,
        enum_switches,
        array_constructors,
        array_helper_uses,
        enum_switch_field_uses,
        generic_return,
        generic_constructor,
        anonymous_allocations,
        enum_code: enum_code_candidate,
    })
}

/// The class view's own coverage plus this presentation's one plane: the members a body run was
/// attempted for, in the method table's own coordinates.
///
/// The range counts the members this presentation can run a body for — one that declares a `Code`
/// attribute and one whose descriptor it can read — scanned up to the runs it attempted and skipped
/// from there to the end. A member that declares no body is outside it (the same rule the class view
/// applies: a declaration is not a body attempt), and so is a member whose bytes are not a method
/// descriptor: neither is work this presentation left undone.
fn class_source_coverage(
    base: Coverage,
    attempted: u64,
    declared: u64,
    complete: bool,
) -> Coverage {
    let mut coverage = base;
    if attempted > 0 {
        coverage.artifact_structural.scanned.push(CoverageRange {
            label: "class_source_bodies".to_owned(),
            start: 0,
            end: attempted,
        });
    }
    if attempted < declared {
        coverage.artifact_structural.skipped.push(CoverageRange {
            label: "class_source_bodies".to_owned(),
            start: attempted,
            end: declared,
        });
    }
    if !complete || coverage.artifact_structural.state != CoverageState::CompleteWithinSchema {
        coverage.artifact_structural.state = CoverageState::Partial;
    }
    coverage
}

struct ProvedEnumSwitchMap {
    labels: std::collections::BTreeMap<i64, String>,
    stores: Vec<jarde_java::enumswitch::EnumSwitchMapEntry>,
    helper: PhysicalDefinitionId,
    enum_definition: PhysicalDefinitionId,
}

fn enum_projection_stop_error(
    stop: jarde_java::StopReason,
    operation: &str,
    missing_table_code: &'static str,
) -> Error {
    match stop {
        jarde_java::StopReason::Budget {
            dimension,
            limit,
            written,
            ..
        } => Error::BudgetExceeded {
            dimension: dimension.into(),
            limit,
            consumed: written,
            requested: 1,
        },
        jarde_java::StopReason::Cancelled { .. } => Error::Cancelled {
            reason: format!("{operation} was cancelled"),
        },
        jarde_java::StopReason::Interrupted { code, .. } => {
            Error::unsupported(code, format!("{operation} was interrupted"))
        }
        jarde_java::StopReason::IrTableMissing { table } => Error::unsupported(
            missing_table_code,
            format!("required IR table `{table}` is missing"),
        ),
        jarde_java::StopReason::EvidenceRefused { code, message, .. } => {
            Error::unsupported(code, message)
        }
    }
}

fn prove_class_source_enum_switch(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    policy: &EnvironmentPolicy,
    target: &PhysicalDefinitionId,
    candidate: &jarde_java::enumswitch::ClassSourceEnumSwitchCandidate,
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<std::result::Result<ProvedEnumSwitchMap, String>> {
    use std::collections::BTreeMap;

    if matches!(policy, EnvironmentPolicy::SingleClass) {
        return Ok(Err(
            "the SingleClass environment does not provide the helper and enum dependencies"
                .to_owned(),
        ));
    }
    if candidate
        .member
        .as_ref()
        .is_none_or(|member| &member.owner != target)
    {
        return Ok(Err(
            "the candidate does not belong to the class-source physical definition".to_owned(),
        ));
    }
    let Some(receiver) = candidate.selector_receiver_type.as_deref() else {
        return Ok(Err(
            "the selector receiver has no exact verifier class type".to_owned(),
        ));
    };
    let Some(enum_owner) = receiver
        .strip_prefix(b"L")
        .and_then(|name| name.strip_suffix(b";"))
    else {
        return Ok(Err(
            "the selector receiver verifier type is not one exact object class".to_owned(),
        ));
    };
    let enum_owner_text = match std::str::from_utf8(enum_owner) {
        Ok(name) if !name.is_empty() => name,
        _ => {
            return Ok(Err(
                "the selector receiver class name is not valid UTF-8".to_owned()
            ));
        }
    };
    if candidate.index.name != "ordinal"
        || candidate.index.descriptor != "()I"
        || (candidate.index.owner != enum_owner_text && candidate.index.owner != "java/lang/Enum")
    {
        return Ok(Err(
            "the table index call is not the receiver enum's inherited ordinal()I".to_owned(),
        ));
    }

    let Some((helper_definition, helper_facts)) = resolve_class_source_dependency(
        content,
        environment,
        candidate.member.as_ref(),
        &candidate.table.owner,
        execution,
        budget,
    )?
    else {
        return Ok(Err(
            "the selected environment did not uniquely resolve the helper class".to_owned(),
        ));
    };
    let Some((enum_definition, enum_facts)) = resolve_class_source_dependency(
        content,
        environment,
        candidate.member.as_ref(),
        enum_owner_text,
        execution,
        budget,
    )?
    else {
        return Ok(Err(
            "the selected environment did not uniquely resolve the enum class".to_owned(),
        ));
    };
    if helper_facts.stopped_at.is_some()
        || helper_facts.this_class.raw().0.as_slice() != candidate.table.owner.as_bytes()
        || helper_facts.access_flags & 0x1000 == 0
    {
        return Ok(Err(
            "the selected helper definition is incomplete, mismatched, or not synthetic".to_owned(),
        ));
    }
    if enum_facts.stopped_at.is_some()
        || enum_facts.this_class.raw().0.as_slice() != enum_owner
        || enum_facts.access_flags & 0x4000 == 0
        || enum_facts
            .super_class
            .as_ref()
            .map(|name| name.raw().0.as_slice())
            != Some(b"java/lang/Enum".as_slice())
    {
        return Ok(Err("the selected verifier receiver definition is not a complete ACC_ENUM class extending java/lang/Enum".to_owned()));
    }
    let helper_tables: Vec<_> = helper_facts
        .fields
        .iter()
        .filter(|field| {
            field.name.raw().0.as_slice() == candidate.table.name.as_bytes()
                && field.descriptor.raw().0.as_slice() == b"[I"
        })
        .collect();
    if helper_tables.len() != 1
        || helper_tables[0].access_flags & (0x0008 | 0x0010 | 0x1000) != (0x0008 | 0x0010 | 0x1000)
    {
        return Ok(Err(
            "the helper does not declare one unique static final synthetic int[] table field"
                .to_owned(),
        ));
    }
    let values_descriptor = format!("()[L{enum_owner_text};");
    let values_methods: Vec<_> = enum_facts
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0.as_slice() == b"values"
                && method.descriptor.raw().0.as_slice() == values_descriptor.as_bytes()
                && method.access_flags & (0x0001 | 0x0008) == (0x0001 | 0x0008)
        })
        .collect();
    let overrides_ordinal = enum_facts.methods.iter().any(|method| {
        method.name.raw().0.as_slice() == b"ordinal"
            && method.descriptor.raw().0.as_slice() == b"()I"
    });
    if values_methods.len() != 1 || overrides_ordinal {
        return Ok(Err(
            "the enum lacks one real static values() method or declares its own ordinal()I"
                .to_owned(),
        ));
    }
    let initializer_headers: Vec<_> = helper_facts
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0.as_slice() == b"<clinit>"
                && method.descriptor.raw().0.as_slice() == b"()V"
        })
        .collect();
    if initializer_headers.len() != 1 || helper_facts.methods.len() != 1 {
        return Ok(Err("the helper has missing/duplicate <clinit> or another method that could expose a table alias".to_owned()));
    }
    let constants: Vec<Vec<u8>> = enum_facts
        .fields
        .iter()
        .filter(|field| {
            field.access_flags & 0x4000 != 0
                && field.access_flags & 0x0008 != 0
                && field.access_flags & 0x0010 != 0
                && field.descriptor.raw().0.as_slice() == format!("L{enum_owner_text};").as_bytes()
        })
        .map(|field| field.name.raw().0.clone())
        .collect();
    if constants.is_empty()
        || constants
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != constants.len()
        || constants.iter().any(|constant| {
            std::str::from_utf8(constant)
                .ok()
                .is_none_or(|name| !jarde_java::names::is_java_identifier(name))
        })
    {
        return Ok(Err(
            "the enum has no uniquely spellable ACC_ENUM constant fields".to_owned(),
        ));
    }

    let enum_clinit_headers: Vec<_> = enum_facts
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0.as_slice() == b"<clinit>"
                && method.descriptor.raw().0.as_slice() == b"()V"
        })
        .collect();
    let enum_constructor_headers: Vec<_> = enum_facts
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0.as_slice() == b"<init>"
                && method.descriptor.raw().0.as_slice() == b"(Ljava/lang/String;I)V"
        })
        .collect();
    let values_field_descriptor = format!("[L{enum_owner_text};");
    let values_fields: Vec<_> = enum_facts
        .fields
        .iter()
        .filter(|field| {
            field.access_flags & (0x0008 | 0x0010 | 0x1000) == (0x0008 | 0x0010 | 0x1000)
                && field.descriptor.raw().0.as_slice() == values_field_descriptor.as_bytes()
        })
        .collect();
    if enum_clinit_headers.len() != 1
        || enum_constructor_headers.len() != 1
        || values_fields.len() != 1
    {
        return Ok(Err(
            "the enum has no unique <clinit> and (String,int) constructor for constant-identity proof".to_owned(),
        ));
    }
    let enum_clinit = PhysicalMethodId {
        owner: enum_definition.clone(),
        name: JvmBytes(b"<clinit>".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    };
    let enum_constructor = PhysicalMethodId {
        owner: enum_definition.clone(),
        name: JvmBytes(b"<init>".to_vec()),
        descriptor: JvmBytes(b"(Ljava/lang/String;I)V".to_vec()),
    };
    let values_field_name = std::str::from_utf8(&values_fields[0].name.raw().0).map_err(|_| {
        Error::unsupported(
            "enum_values_field_name",
            "the enum values field name is not UTF-8",
        )
    })?;
    let enum_clinit_analysis = jarde_jvm::analyze_method_ir(
        content,
        &crate::ir::MethodAnalysisRequest {
            environment: environment.clone(),
            method: enum_clinit.clone(),
            stages: MethodOperation::Analysis.stages().to_vec(),
        },
        budget,
    )?;
    merge_execution(execution, enum_clinit_analysis.report().execution.clone());
    if enum_clinit_analysis.report().method != enum_clinit
        || !matches!(
            enum_clinit_analysis.report().execution,
            ExecutionReport::Complete { .. }
        )
    {
        return Ok(Err(
            "the enum <clinit> IR did not complete for the selected physical definition".to_owned(),
        ));
    }
    let factory_name = match jarde_java::enumswitch::enum_values_factory_name(
        enum_clinit_analysis.ir(),
        enum_owner_text,
        values_field_name,
        budget,
    ) {
        Ok(Ok(name)) => name,
        Ok(Err(reason)) => return Ok(Err(reason)),
        Err(stop) => {
            return Err(enum_projection_stop_error(
                stop,
                "enum switch projection",
                "enum_switch_ir_missing",
            ));
        }
    };
    let factory_headers: Vec<_> = enum_facts
        .methods
        .iter()
        .filter(|method| {
            method.name.raw().0.as_slice() == factory_name.as_bytes()
                && method.descriptor.raw().0.as_slice() == values_descriptor.as_bytes()
                && method.access_flags & (0x0002 | 0x0008 | 0x1000) == (0x0002 | 0x0008 | 0x1000)
        })
        .collect();
    if factory_headers.len() != 1 || factory_name == "values" {
        return Ok(Err(
            "the enum <clinit> target is not one private static synthetic values-array factory"
                .to_owned(),
        ));
    }
    let enum_values_factory = PhysicalMethodId {
        owner: enum_definition.clone(),
        name: JvmBytes(factory_name.as_bytes().to_vec()),
        descriptor: JvmBytes(values_descriptor.as_bytes().to_vec()),
    };
    let enum_constructor_analysis = jarde_jvm::analyze_method_ir(
        content,
        &crate::ir::MethodAnalysisRequest {
            environment: environment.clone(),
            method: enum_constructor.clone(),
            stages: MethodOperation::Analysis.stages().to_vec(),
        },
        budget,
    )?;
    merge_execution(
        execution,
        enum_constructor_analysis.report().execution.clone(),
    );
    if enum_constructor_analysis.report().method != enum_constructor
        || !matches!(
            enum_constructor_analysis.report().execution,
            ExecutionReport::Complete { .. }
        )
    {
        return Ok(Err(
            "the enum constructor IR did not complete for the selected physical definition"
                .to_owned(),
        ));
    }
    let enum_factory_analysis = jarde_jvm::analyze_method_ir(
        content,
        &crate::ir::MethodAnalysisRequest {
            environment: environment.clone(),
            method: enum_values_factory.clone(),
            stages: MethodOperation::Analysis.stages().to_vec(),
        },
        budget,
    )?;
    merge_execution(execution, enum_factory_analysis.report().execution.clone());
    if enum_factory_analysis.report().method != enum_values_factory
        || !matches!(
            enum_factory_analysis.report().execution,
            ExecutionReport::Complete { .. }
        )
    {
        return Ok(Err(
            "the enum values-array factory IR did not complete for the selected physical definition".to_owned(),
        ));
    }
    let enum_values_method = PhysicalMethodId {
        owner: enum_definition.clone(),
        name: JvmBytes(b"values".to_vec()),
        descriptor: JvmBytes(values_descriptor.as_bytes().to_vec()),
    };
    let enum_values_analysis = jarde_jvm::analyze_method_ir(
        content,
        &crate::ir::MethodAnalysisRequest {
            environment: environment.clone(),
            method: enum_values_method.clone(),
            stages: MethodOperation::Analysis.stages().to_vec(),
        },
        budget,
    )?;
    merge_execution(execution, enum_values_analysis.report().execution.clone());
    if enum_values_analysis.report().method != enum_values_method
        || !matches!(
            enum_values_analysis.report().execution,
            ExecutionReport::Complete { .. }
        )
    {
        return Ok(Err(
            "the enum public values() IR did not complete for the selected physical definition"
                .to_owned(),
        ));
    }
    match jarde_java::enumswitch::prove_enum_constant_ordinals(
        enum_clinit_analysis.ir(),
        enum_constructor_analysis.ir(),
        (enum_factory_analysis.ir(), enum_values_analysis.ir()),
        enum_owner_text,
        &constants,
        values_field_name,
        budget,
    ) {
        Ok(Ok(())) => {}
        Ok(Err(reason)) => return Ok(Err(reason)),
        Err(stop) => {
            return Err(match stop {
                jarde_java::StopReason::Budget {
                    dimension, limit, ..
                } => Error::BudgetExceeded {
                    dimension: dimension.into(),
                    limit,
                    consumed: budget.usage().counted_usage(dimension),
                    requested: 1,
                },
                jarde_java::StopReason::Cancelled { .. } => Error::Cancelled {
                    reason: "enum constant identity proof was cancelled".to_owned(),
                },
                jarde_java::StopReason::Interrupted { code, .. } => {
                    Error::unsupported(code, "enum constant identity proof was interrupted")
                }
                jarde_java::StopReason::IrTableMissing { table } => Error::unsupported(
                    "enum_switch_ir_missing",
                    format!("required IR table `{table}` is missing"),
                ),
                jarde_java::StopReason::EvidenceRefused { code, message, .. } => {
                    Error::unsupported(code, message)
                }
            });
        }
    }

    let initializer = PhysicalMethodId {
        owner: helper_definition.clone(),
        name: JvmBytes(b"<clinit>".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    };
    let analysis = jarde_jvm::analyze_method_ir(
        content,
        &crate::ir::MethodAnalysisRequest {
            environment: environment.clone(),
            method: initializer.clone(),
            stages: MethodOperation::Analysis.stages().to_vec(),
        },
        budget,
    )?;
    if analysis.report().method != initializer
        || !matches!(
            analysis.report().execution,
            ExecutionReport::Complete { .. }
        )
    {
        return Ok(Err(
            "the helper <clinit> IR did not complete for the selected physical method".to_owned(),
        ));
    }
    let proof = match jarde_java::enumswitch::prove_enum_switch_map_initializer(
        analysis.ir(),
        &candidate.table.owner,
        &candidate.table.name,
        enum_owner_text,
        &constants,
        &candidate.keys,
        budget,
    ) {
        Ok(Ok(proof)) => proof,
        Ok(Err(reason)) => return Ok(Err(reason)),
        Err(stop) => {
            return Err(match stop {
                jarde_java::StopReason::Budget {
                    dimension, limit, ..
                } => Error::BudgetExceeded {
                    dimension: dimension.into(),
                    limit,
                    consumed: budget.usage().counted_usage(dimension),
                    requested: 1,
                },
                jarde_java::StopReason::Cancelled { .. } => Error::Cancelled {
                    reason: "enum switch proof was cancelled".to_owned(),
                },
                jarde_java::StopReason::Interrupted { code, .. } => {
                    Error::unsupported(code, "enum switch proof was interrupted")
                }
                jarde_java::StopReason::IrTableMissing { table } => Error::unsupported(
                    "enum_switch_ir_missing",
                    format!("required IR table `{table}` is missing"),
                ),
                jarde_java::StopReason::EvidenceRefused { code, message, .. } => {
                    Error::unsupported(code, message)
                }
            });
        }
    };
    if proof.initializer.as_ref() != Some(&initializer) {
        return Ok(Err(
            "initializer proof identity differs from the selected physical <clinit>".to_owned(),
        ));
    }
    let mut labels = BTreeMap::new();
    let mut stores = Vec::new();
    for entry in proof.entries {
        let Ok(label) = std::str::from_utf8(&entry.constant) else {
            return Ok(Err("an enum constant name is not UTF-8".to_owned()));
        };
        if !jarde_java::names::is_java_identifier(label)
            || labels.insert(entry.key, label.to_owned()).is_some()
        {
            return Ok(Err(
                "the proven enum map has an invalid or repeated key/label".to_owned(),
            ));
        }
        stores.push(entry);
    }
    if candidate.keys.iter().any(|key| !labels.contains_key(key)) {
        return Ok(Err(
            "the proven initializer map does not cover every switch integer key".to_owned(),
        ));
    }
    Ok(Ok(ProvedEnumSwitchMap {
        labels,
        stores,
        helper: helper_definition,
        enum_definition,
    }))
}

fn resolve_class_source_dependency(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    enclosing: Option<&PhysicalMethodId>,
    owner: &str,
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<Option<(PhysicalDefinitionId, ClassMemberFacts)>> {
    Ok(resolve_class_source_dependency_read(
        content,
        environment,
        enclosing,
        owner,
        execution,
        budget,
    )?
    .map(|(definition, read)| (definition, read.facts)))
}

/// The same selected-definition chain used by class-source's enum proof, retaining the bytes for
/// a member constructor's target-only attribute and prologue proof.
fn resolve_class_source_dependency_read(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    enclosing: Option<&PhysicalMethodId>,
    owner: &str,
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<Option<(PhysicalDefinitionId, ConfirmedRead)>> {
    resolve_class_source_dependency_read_raw(
        content,
        environment,
        enclosing,
        owner.as_bytes(),
        execution,
        budget,
    )
}

/// The same selected-definition read for a JVM internal name retained in its original bytes.
fn resolve_class_source_dependency_read_raw(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    enclosing: Option<&PhysicalMethodId>,
    owner: &[u8],
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<Option<(PhysicalDefinitionId, ConfirmedRead)>> {
    let resolution = jarde_jvm::resolve_symbol(
        content,
        &ResolutionRequest {
            environment: environment.clone(),
            target: jarde_reader::model::SymbolRef::Class {
                owner: JvmBytes(owner.to_vec()),
            },
            use_kind: ReferenceUse::ClassReference,
            caller: jarde_jvm::environment::CallerContext {
                loader: environment.runtime.load_domain.loader.clone(),
                enclosing: enclosing.cloned(),
            },
            dispatch: None,
        },
        budget,
    )?;
    merge_execution(execution, resolution.execution.clone());
    if resolution.state != Some(ResolutionState::Resolved)
        || !matches!(resolution.execution, ExecutionReport::Complete { .. })
        || !resolution.environment_problems.is_empty()
        || !resolution.unresolved_dependencies.is_empty()
        || !resolution.candidates.is_empty()
    {
        return Ok(None);
    }
    let Some(resolved) = resolution.resolved else {
        return Ok(None);
    };
    if !matches!(&resolved.member, jarde_reader::model::SymbolRef::Class { owner: name } if name.0 == owner)
        || resolution.reads.len() != 1
        || resolution.reads[0].definition != resolved.definition
        || resolution.reads[0].reason != jarde_jvm::resolver::ReadReason::RequestedDefinition
    {
        return Ok(None);
    }
    let Some(snapshot) = content
        .iter()
        .find(|snapshot| snapshot.id() == resolved.definition.snapshot())
    else {
        return Ok(None);
    };
    let (read, _) = read_definition(snapshot, &resolved.definition, budget)?;
    if read.facts.stopped_at.is_some() || read.facts.this_class.raw().0.as_slice() != owner {
        return Ok(None);
    }
    Ok(Some((resolved.definition, read)))
}

/// Resolve only the subclass named by a verified allocation retained from this class-source
/// run. These are private candidates for the later exclusivity/body proof; they never authorize a
/// source projection by themselves.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingEnumConstantBodyRelation {
    pub(crate) field_index: u64,
    pub(crate) allocation_bci: u32,
    pub(crate) constructor_bci: u32,
    pub(crate) constructor_descriptor: Vec<u8>,
    pub(crate) subclass_owner: Vec<u8>,
    pub(crate) subclass: PhysicalDefinitionId,
    /// Anonymous-shaped typed InnerClasses rows in this selected child's own class file.
    /// The census only uses rows naming another selected child of this same constant group.
    pub(crate) anonymous_inner_owners: Vec<Vec<u8>>,
    /// Exhaustive, selected-input owner references. `exclusive` is only a use-site conclusion;
    /// constructor arguments, stack aliases and source projection remain unproved.
    pub(crate) use_census: PendingEnumConstantBodyUseCensus,
    /// Same-run physical facts that establish the narrow zero-source-argument, ordered
    /// two-constant enum shape. This remains private pending evidence and grants no projection.
    pub(crate) group_shape: PendingEnumConstantBodyGroupShape,
    /// Exact selected child Code edge to the already identified access constructor.
    pub(crate) constructor_bridge: std::result::Result<PendingEnumConstructorEdge, String>,
    /// Typed Code evidence for each potential override, including the exception table. The
    /// ordinary recovery report can omit this physical fact under an evidence selection.
    pub(crate) child_code_evidence: std::result::Result<(), String>,
    /// The selected child's complete physical method results, admitted only after every
    /// source-visible member has passed the bounded body and declaration proof.
    pub(crate) body_proof: Option<std::result::Result<std::sync::Arc<[ClassSourceMethod]>, String>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingEnumConstructorEdge {
    pub(crate) caller: PhysicalMethodId,
    pub(crate) call_bci: u32,
    pub(crate) target_owner: Vec<u8>,
    pub(crate) target_descriptor: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PendingEnumConstantBodyUseCensus {
    pub(crate) scans: Vec<PendingEnumConstantBodyUseScan>,
    pub(crate) exclusive: bool,
    pub(crate) refusal: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingEnumConstantBodyUseScan {
    pub(crate) snapshot: SnapshotId,
    pub(crate) scope: PhysicalScope,
    pub(crate) items: Vec<XrefItem>,
    pub(crate) has_more: bool,
    pub(crate) coverage: QueryCoverage,
    pub(crate) execution: ExecutionReport,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingEnumConstantBodyGroupShape {
    pub(crate) enum_access_flags: u16,
    pub(crate) constants: Vec<PendingEnumConstantBodyConstant>,
    pub(crate) implicit_members: Vec<PendingEnumConstantBodyMember>,
    pub(crate) abstract_methods: Vec<PendingEnumConstantBodyMember>,
    pub(crate) constructors: Vec<PendingEnumConstantBodyMember>,
    /// The private constructor reaches java/lang/Enum unchanged; the optional synthetic
    /// access constructor reaches that private constructor unchanged and ignores its marker.
    pub(crate) constructor_chain: std::result::Result<Vec<PendingEnumConstructorEdge>, String>,
    /// Ordinary constants use their own physical constructor without an access bridge.
    pub(crate) direct_constant_bcis: std::result::Result<Vec<u32>, String>,
    /// Exact same-run `<clinit>` prefix evidence; a refusal leaves the physical relations intact.
    pub(crate) initializer_prefix:
        std::result::Result<PendingEnumConstantBodyInitializerPrefix, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingEnumConstantBodyInitializerPrefix {
    pub(crate) constant_field_write_bcis: Vec<u32>,
    pub(crate) values_factory_element_bcis: Vec<u32>,
    pub(crate) values_factory_call_bci: u32,
    pub(crate) values_field_write_bci: u32,
    pub(crate) prefix_end_bci: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingEnumConstantBodyConstant {
    pub(crate) field_index: u64,
    pub(crate) field_name: Vec<u8>,
    pub(crate) expected_ordinal: u32,
    pub(crate) allocation_bci: u32,
    pub(crate) constructor_bci: u32,
    pub(crate) allocation_owner: Vec<u8>,
    pub(crate) constructor_owner: Vec<u8>,
    pub(crate) constructor_descriptor: Vec<u8>,
    pub(crate) descriptor_source_argument_count: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PendingEnumConstantBodyMember {
    pub(crate) table_index: u64,
    pub(crate) name: Vec<u8>,
    pub(crate) descriptor: Vec<u8>,
    pub(crate) access_flags: u16,
    pub(crate) has_code: bool,
    /// The raw object type in a javac enum access-constructor descriptor, when this is that record.
    pub(crate) access_marker_owner: Option<Vec<u8>>,
}

/// Check the entire straight-line constructor body, not just its `invokespecial`. The
/// positional loads establish that neither name nor ordinal was replaced; the optional
/// `aconst_null` is the only value allowed in the marker slot. No other effect can hide here.
fn prove_enum_constructor_instructions(
    instructions: &[crate::enum_constants::EnumCodeInstruction],
    expected_owner: &[u8],
    expected_descriptor: &[u8],
    marker: bool,
) -> std::result::Result<u32, String> {
    use crate::enum_constants::EnumCodeReference;

    let refusal =
        || "the constructor does not forward unchanged name/ordinal on a pure edge".to_owned();
    let opcodes: &[u8] = if marker {
        &[0x2a, 0x2b, 0x1c, 0x01, 0xb7, 0xb1]
    } else {
        &[0x2a, 0x2b, 0x1c, 0xb7, 0xb1]
    };
    if instructions.len() != opcodes.len() {
        return Err(refusal());
    }
    let mut next_bci = 0;
    for (position, instruction) in instructions.iter().enumerate() {
        if instruction.bci != next_bci
            || instruction.opcode != opcodes[position]
            || instruction.width != if instruction.opcode == 0xb7 { 3 } else { 1 }
            || instruction.immediate.is_some()
            || (instruction.opcode != 0xb7 && instruction.reference.is_some())
            || (instruction.opcode == 0xb7 && instruction.local.is_some())
            || (instruction.opcode != 0xb7
                && instruction.local
                    != match position {
                        0 => Some(0),
                        1 => Some(1),
                        2 => Some(2),
                        _ => None,
                    })
        {
            return Err(refusal());
        }
        next_bci = next_bci
            .checked_add(instruction.width)
            .ok_or_else(refusal)?;
    }
    let call = &instructions[instructions.len() - 2];
    if !matches!(&call.reference,
        Some(EnumCodeReference::Method { owner, name, descriptor, interface: false })
            if owner == expected_owner && name == b"<init>" && descriptor == expected_descriptor)
    {
        return Err(refusal());
    }
    Ok(call.bci)
}

fn prove_enum_constructor_candidate(
    candidates: &[crate::enum_constants::EnumMethodCodeCandidate],
    member: &PendingEnumConstantBodyMember,
    identity: &PhysicalMethodId,
    target_owner: &[u8],
    target_descriptor: &[u8],
) -> std::result::Result<PendingEnumConstructorEdge, String> {
    let matching: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.table_index == member.table_index)
        .collect();
    let [code] = matching.as_slice() else {
        return Err("the selected main constructor has no unique same-run Code".to_owned());
    };
    if code.member.as_ref() != Some(identity)
        || !member.has_code
        || !code.complete
        || code.exception_handler_count != 0
    {
        return Err("the selected main constructor Code is incomplete".to_owned());
    }
    let call_bci = prove_enum_constructor_instructions(
        &code.instructions,
        target_owner,
        target_descriptor,
        false,
    )?;
    Ok(PendingEnumConstructorEdge {
        caller: identity.clone(),
        call_bci,
        target_owner: target_owner.to_vec(),
        target_descriptor: target_descriptor.to_vec(),
    })
}

fn prove_enum_physical_constructor(
    bytes: &[u8],
    header: &jarde_reader::classfile::MemberHeader,
    identity: PhysicalMethodId,
    pool: &[jarde_reader::classfile::CpEntryFacts],
    enum_owner: &[u8],
    access_descriptor: &[u8],
    marker: bool,
    budget: &mut Budget,
) -> Result<std::result::Result<PendingEnumConstructorEdge, String>> {
    use jarde_reader::classfile::{CpEntryKind, cp_entry, method_code_facts};

    if !header
        .attributes
        .iter()
        .any(|attribute| attribute.name.raw().0 == b"Code")
    {
        return Ok(Err("the selected child constructor has no Code".to_owned()));
    }
    budget.charge(CountedBudgetDimension::MethodBodies, 1)?;
    let code = match method_code_facts(bytes, header, budget) {
        Ok(code) => code,
        Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => return Err(error),
        Err(_) => {
            return Ok(Err(
                "the selected child constructor Code cannot be read".to_owned()
            ));
        }
    };
    match &code.execution {
        ExecutionReport::Cancelled { .. } => {
            return Err(Error::Cancelled {
                reason: "the selected child constructor Code read was cancelled".to_owned(),
            });
        }
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded { dimension },
            ..
        } => {
            let counted = CountedBudgetDimension::try_from(*dimension).map_err(|()| {
                Error::unsupported(
                    "enum_constructor_code_stop",
                    "the child constructor Code read stopped on an uncounted dimension",
                )
            })?;
            let limit = budget.limits().counted_limit(counted);
            let consumed = match counted {
                CountedBudgetDimension::CodeBytes => budget.usage().code_bytes,
                CountedBudgetDimension::ResultItems => budget.usage().result_items,
                _ => 0,
            };
            return Err(Error::BudgetExceeded {
                dimension: *dimension,
                limit,
                consumed,
                requested: limit.saturating_sub(consumed).saturating_add(1),
            });
        }
        _ => {}
    }
    if code.stopped_at.is_some()
        || !matches!(code.execution, ExecutionReport::Complete { .. })
        || code.exception_handler_count != 0
        || !code.exception_handlers.is_empty()
        || code.instructions.len() != code.operands().len()
        || code
            .instructions
            .last()
            .is_none_or(|last| u64::from(last.bci) + u64::from(last.width) != code.code_span.length)
    {
        return Ok(Err(
            "the selected child constructor Code is incomplete".to_owned()
        ));
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
    )?;
    budget.charge(
        CountedBudgetDimension::IrItems,
        u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
    )?;
    let mut instructions = Vec::with_capacity(code.instructions.len());
    for (instruction, operands) in code.instructions.iter().zip(code.operands()) {
        budget.poll()?;
        let reference = if let Some(index) = instruction.constant_pool_index {
            match cp_entry(pool, index).map(|entry| &entry.kind) {
                Ok(CpEntryKind::MethodRef {
                    owner,
                    name,
                    descriptor,
                    ..
                }) => Some(crate::enum_constants::EnumCodeReference::Method {
                    owner: owner.0.clone(),
                    name: name.0.clone(),
                    descriptor: descriptor.0.clone(),
                    interface: false,
                }),
                _ => Some(crate::enum_constants::EnumCodeReference::Other),
            }
        } else {
            None
        };
        instructions.push(crate::enum_constants::EnumCodeInstruction {
            bci: instruction.bci,
            width: instruction.width,
            opcode: instruction.opcode,
            immediate: operands.immediate,
            local: operands.local.map(|local| local.index),
            reference,
        });
    }
    let result =
        prove_enum_constructor_instructions(&instructions, enum_owner, access_descriptor, marker)
            .map(|call_bci| PendingEnumConstructorEdge {
                caller: identity,
                call_bci,
                target_owner: enum_owner.to_vec(),
                target_descriptor: access_descriptor.to_vec(),
            });
    Ok(result)
}

fn prove_enum_body_values_factory(
    code: &crate::enum_constants::EnumMethodCodeCandidate,
    owner: &[u8],
    constants: &[PendingEnumConstantBodyConstant],
) -> std::result::Result<Vec<u32>, String> {
    use crate::enum_constants::EnumCodeReference;

    let refused = || "$values() does not return the two constructed constants in order".to_owned();
    let instructions = &code.instructions;
    if !code.complete || code.exception_handler_count != 0 || instructions.len() != 11 {
        return Err(refused());
    }
    let mut next_bci = 0;
    for instruction in instructions {
        if instruction.bci != next_bci || instruction.width == 0 {
            return Err(refused());
        }
        next_bci = instruction
            .bci
            .checked_add(instruction.width)
            .ok_or_else(refused)?;
    }
    if instructions[0].opcode != 0x05
        || !matches!(&instructions[1].reference,
            Some(EnumCodeReference::Class(actual))
                if instructions[1].opcode == 0xbd && actual == owner)
        || instructions[10].opcode != 0xb0
    {
        return Err(refused());
    }
    let mut field_read_bcis = Vec::with_capacity(2);
    for (ordinal, constant) in constants.iter().enumerate() {
        let start = 2 + ordinal * 4;
        if instructions[start].opcode != 0x59
            || instructions[start + 1].opcode != 0x03 + ordinal as u8
            || !matches!(&instructions[start + 2].reference,
                Some(EnumCodeReference::Field { owner: field_owner, name, descriptor })
                    if instructions[start + 2].opcode == 0xb2
                        && field_owner == owner
                        && name == &constant.field_name
                        && descriptor == &[b"L".as_slice(), owner, b";"].concat())
            || instructions[start + 3].opcode != 0x53
        {
            return Err(refused());
        }
        field_read_bcis.push(instructions[start + 2].bci);
    }
    Ok(field_read_bcis)
}

/// The fixed Java 8 constant prefix leaves exactly one allocation reference after each
/// constructor call. The adjacent `putstatic` consumes it; no other opcode can copy or store it.
fn prove_enum_body_initializer_prefix(
    code: &crate::enum_constants::EnumMethodCodeCandidate,
    factory: Option<&crate::enum_constants::EnumMethodCodeCandidate>,
    owner: &[u8],
    constants: &[PendingEnumConstantBodyConstant],
    values_field: &PendingEnumConstantBodyMember,
) -> std::result::Result<PendingEnumConstantBodyInitializerPrefix, String> {
    use crate::enum_constants::{EnumCodeInstruction, EnumCodeReference};

    let refused = || "the same-run <clinit> constant prefix is not exact".to_owned();
    if !code.complete || code.exception_handler_count != 0 || constants.len() != 2 {
        return Err(refused());
    }
    let instructions = &code.instructions;
    let mut next_bci = 0;
    for instruction in instructions {
        if instruction.bci != next_bci || instruction.width == 0 {
            return Err("the <clinit> Code has a gap or overlapping instruction".to_owned());
        }
        next_bci = instruction
            .bci
            .checked_add(instruction.width)
            .ok_or_else(refused)?;
    }
    let plain = |instruction: Option<&EnumCodeInstruction>, opcode| {
        instruction.is_some_and(|instruction| {
            instruction.opcode == opcode
                && instruction.reference.is_none()
                && instruction.immediate.is_none()
                && instruction.local.is_none()
        })
    };
    let field = |instruction: Option<&EnumCodeInstruction>, name: &[u8], descriptor: &[u8]| {
        instruction.is_some_and(|instruction| {
            instruction.opcode == 0xb3
                && matches!(&instruction.reference,
                    Some(EnumCodeReference::Field { owner: actual_owner, name: actual_name, descriptor: actual_descriptor })
                        if actual_owner == owner && actual_name == name && actual_descriptor == descriptor)
        })
    };
    let ordinal = |instruction: Option<&EnumCodeInstruction>, expected: u32| {
        let Some(instruction) = instruction else {
            return false;
        };
        let actual = match instruction.opcode {
            0x02..=0x08 => Some(i32::from(instruction.opcode) - 3),
            0x10 | 0x11 => match instruction.immediate {
                Some(jarde_reader::classfile::ImmediateValue::Int(value)) => Some(value),
                _ => None,
            },
            0x12 | 0x13 => match instruction.reference {
                Some(EnumCodeReference::Integer(value)) => Some(value),
                _ => None,
            },
            _ => None,
        };
        actual == i32::try_from(expected).ok()
    };
    let mut cursor = 0;
    let mut field_write_bcis = Vec::with_capacity(2);
    let enum_descriptor = [b"L".as_slice(), owner, b";"].concat();
    for constant in constants {
        let part = instructions.get(cursor..cursor + 6).ok_or_else(refused)?;
        if part[0].bci != constant.allocation_bci
            || !matches!(&part[0].reference, Some(EnumCodeReference::Class(actual))
                if part[0].opcode == 0xbb && actual == &constant.allocation_owner)
            || !plain(Some(&part[1]), 0x59)
            || !matches!(&part[2].reference, Some(EnumCodeReference::String(actual))
                if matches!(part[2].opcode, 0x12 | 0x13) && actual == &constant.field_name)
            || !ordinal(Some(&part[3]), constant.expected_ordinal)
            || part[4].bci != constant.constructor_bci
            || !matches!(&part[4].reference,
                Some(EnumCodeReference::Method { owner: actual_owner, name, descriptor, interface: false })
                    if part[4].opcode == 0xb7
                        && actual_owner == &constant.constructor_owner
                        && name == b"<init>"
                        && descriptor == &constant.constructor_descriptor)
            || !field(Some(&part[5]), &constant.field_name, &enum_descriptor)
        {
            return Err(refused());
        }
        field_write_bcis.push(part[5].bci);
        cursor += 6;
    }
    let factory_call = instructions.get(cursor).ok_or_else(refused)?;
    let values_descriptor = [b"()[L".as_slice(), owner, b";"].concat();
    if !matches!(&factory_call.reference,
        Some(EnumCodeReference::Method { owner: actual_owner, name, descriptor, interface: false })
            if factory_call.opcode == 0xb8 && actual_owner == owner && name == b"$values"
                && descriptor == &values_descriptor)
        || !field(
            instructions.get(cursor + 1),
            &values_field.name,
            &values_field.descriptor,
        )
    {
        return Err(refused());
    }
    let store = &instructions[cursor + 1];
    let prefix_end_bci = store.bci.checked_add(store.width).ok_or_else(refused)?;
    let factory = factory.ok_or_else(|| "the same-run $values() Code is absent".to_owned())?;
    let values_factory_element_bcis = prove_enum_body_values_factory(factory, owner, constants)?;
    let suffix = instructions.get(cursor + 2..).ok_or_else(refused)?;
    if !matches!(suffix.last(), Some(last) if plain(Some(last), 0xb1))
        || suffix.iter().any(|instruction| {
            // A transfer back into the prefix (or an early exit) would invalidate its
            // single-execution, empty-stack boundary. Re-reading a constant or `$VALUES`
            // could create another alias or store it elsewhere. Other suffix effects remain
            // available for the later whole-initializer proof.
            matches!(
                instruction.opcode,
                0x99..=0xa9 | 0xaa | 0xab | 0xbf | 0xc6 | 0xc7 | 0xc8 | 0xc9
            ) || (instruction.opcode == 0xb1 && instruction.bci != suffix.last().unwrap().bci)
                || matches!(&instruction.reference,
                    Some(EnumCodeReference::Field { owner: field_owner, name, .. })
                        if field_owner == owner
                            && (name == &values_field.name
                                || constants.iter().any(|constant| name == &constant.field_name)))
        })
    {
        return Err("the <clinit> suffix does not close after the $VALUES prefix".to_owned());
    }
    Ok(PendingEnumConstantBodyInitializerPrefix {
        constant_field_write_bcis: field_write_bcis,
        values_factory_element_bcis,
        values_factory_call_bci: factory_call.bci,
        values_field_write_bci: store.bci,
        prefix_end_bci,
    })
}

#[allow(clippy::too_many_arguments)]
fn resolve_enum_constant_body_relations(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    enum_nesting: &class_source::ClassSourceAssemblyContext,
    enum_pool: &[jarde_reader::classfile::CpEntryFacts],
    enum_owner: &[u8],
    enum_access_flags: u16,
    fields: &[jarde_reader::classfile::MemberHeader],
    methods: &[jarde_reader::classfile::MemberHeader],
    code_candidates: &[crate::enum_constants::EnumMethodCodeCandidate],
    allocation_scans: &[(
        PhysicalMethodId,
        Option<jarde_java::report::AnonymousAllocationScan>,
    )],
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<Vec<PendingEnumConstantBodyRelation>> {
    use crate::enum_constants::EnumCodeReference;

    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_FINAL: u16 = 0x0010;
    const ACC_SYNTHETIC: u16 = 0x1000;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_ENUM: u16 = 0x4000;
    const BASE_CTOR: &[u8] = b"(Ljava/lang/String;I)V";

    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(methods.len()).unwrap_or(u64::MAX),
    )?;
    let clinit_headers: Vec<_> = methods
        .iter()
        .enumerate()
        .filter(|(_, method)| {
            method.name.raw().0 == b"<clinit>" && method.descriptor.raw().0 == b"()V"
        })
        .collect();
    let [(clinit_index, _clinit_header)] = clinit_headers.as_slice() else {
        return Ok(Vec::new());
    };
    let matching_code: Vec<_> = code_candidates
        .iter()
        .filter(|candidate| candidate.table_index == *clinit_index as u64)
        .collect();
    let [code] = matching_code.as_slice() else {
        return Ok(Vec::new());
    };
    let Some((clinit_identity, Some(scan))) = allocation_scans
        .iter()
        .find(|(identity, _)| identity.name.0 == b"<clinit>" && identity.descriptor.0 == b"()V")
    else {
        return Ok(Vec::new());
    };
    if code.member.as_ref() != Some(clinit_identity) || !code.complete || !scan.complete {
        return Ok(Vec::new());
    }

    let allocations: Vec<_> = scan
        .allocations
        .iter()
        .filter(|allocation| allocation.member == *clinit_identity && allocation.verified)
        .collect();
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(allocations.len()).unwrap_or(u64::MAX),
    )?;
    if allocations.len() != 2
        || !allocations
            .iter()
            .any(|allocation| allocation.class.as_bytes() != enum_owner)
    {
        return Ok(Vec::new());
    }
    let owner_text = std::str::from_utf8(enum_owner).ok();
    let Some(owner_text) = owner_text else {
        return Ok(Vec::new());
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(fields.len()).unwrap_or(u64::MAX),
    )?;
    let constants: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| {
            field.access_flags & 0x4000 != 0
                && field.descriptor.raw().0 == format!("L{owner_text};").as_bytes()
        })
        .collect();
    if constants.len() != 2
        || fields
            .iter()
            .filter(|field| field.access_flags & ACC_ENUM != 0)
            .count()
            != 2
        || constants.iter().any(|(_, field)| {
            field.access_flags != (0x0001 | ACC_STATIC | ACC_FINAL | ACC_ENUM)
                || !field.attributes.is_empty()
        })
    {
        return Ok(Vec::new());
    }

    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(code.instructions.len()).unwrap_or(u64::MAX),
    )?;
    if allocations.len() != constants.len() {
        return Ok(Vec::new());
    }
    let mut constructions = Vec::with_capacity(constants.len());
    for allocation in allocations {
        budget.poll()?;
        let Some(constructor_bci) = allocation.constructor_bci else {
            return Ok(Vec::new());
        };
        if !code.instructions.iter().any(|instruction| {
            instruction.bci == allocation.head_bci
                && instruction.opcode == 0xbb
                && instruction.reference
                    == Some(EnumCodeReference::Class(
                        allocation.class.as_bytes().to_vec(),
                    ))
        }) {
            return Ok(Vec::new());
        }
        let Some(constructor_position) = code
            .instructions
            .iter()
            .position(|instruction| instruction.bci == constructor_bci)
        else {
            return Ok(Vec::new());
        };
        let Some(EnumCodeReference::Method {
            owner: constructor_owner,
            name,
            descriptor,
            interface: false,
        }) = code.instructions[constructor_position].reference.as_ref()
        else {
            return Ok(Vec::new());
        };
        if !enum_constant_constructor_matches(
            constructor_owner,
            name,
            descriptor,
            allocation.class.as_bytes(),
        ) {
            return Ok(Vec::new());
        }
        // `invokespecial` must be followed by the one physical enum-field write. This makes
        // constructor BCI and field BCI a one-to-one pair before any child definition is read.
        let Some(field_write) = code.instructions.get(constructor_position + 1) else {
            return Ok(Vec::new());
        };
        let Some((field_index, _)) = constants.iter().find(|(_, field)| {
            matches!(&field_write.reference,
                Some(EnumCodeReference::Field { owner, name, descriptor })
                    if owner.as_slice() == enum_owner
                        && name.as_slice() == field.name.raw().0.as_slice()
                        && descriptor.as_slice() == field.descriptor.raw().0.as_slice())
        }) else {
            return Ok(Vec::new());
        };
        if field_write.opcode != 0xb3 {
            return Ok(Vec::new());
        }
        constructions.push((
            *field_index,
            allocation,
            constructor_bci,
            descriptor.clone(),
        ));
    }
    constructions.sort_by_key(|(_, allocation, _, _)| allocation.head_bci);
    if constructions
        .iter()
        .map(|(field_index, _, _, _)| *field_index)
        .ne(constants.iter().map(|(field_index, _)| *field_index))
    {
        return Ok(Vec::new());
    }

    // Preserve the entire ordered initializer shape and the physical implicit-member records
    // before resolving child definitions. These facts are intentionally only pending evidence:
    // the ordinary enum proof above still refuses zero-source-argument anonymous-owner groups.
    let mut constant_shape = Vec::with_capacity(constants.len());
    for (
        ordinal,
        ((field_index, field), (constructed_field_index, allocation, constructor_bci, descriptor)),
    ) in constants.iter().zip(&constructions).enumerate()
    {
        if *field_index != *constructed_field_index || descriptor.as_slice() != BASE_CTOR {
            return Ok(Vec::new());
        }
        let Some(constructor_owner) = code
            .instructions
            .iter()
            .find(|instruction| instruction.bci == *constructor_bci)
            .and_then(|instruction| instruction.reference.as_ref())
            .and_then(|reference| match reference {
                EnumCodeReference::Method { owner, .. } => Some(owner.clone()),
                _ => None,
            })
        else {
            return Ok(Vec::new());
        };
        if constructor_owner != allocation.class.as_bytes() {
            return Ok(Vec::new());
        }
        constant_shape.push(PendingEnumConstantBodyConstant {
            field_index: u64::try_from(*field_index).unwrap_or(u64::MAX),
            field_name: field.name.raw().0.clone(),
            expected_ordinal: u32::try_from(ordinal).unwrap_or(u32::MAX),
            allocation_bci: allocation.head_bci,
            constructor_bci: *constructor_bci,
            allocation_owner: allocation.class.as_bytes().to_vec(),
            constructor_owner,
            constructor_descriptor: descriptor.clone(),
            // This count describes only the constructor descriptor after the VM-injected
            // name/ordinal pair; it does not prove the values passed at this call site.
            descriptor_source_argument_count: 0,
        });
    }

    let physical_member = |index: usize, header: &jarde_reader::classfile::MemberHeader| {
        PendingEnumConstantBodyMember {
            table_index: u64::try_from(index).unwrap_or(u64::MAX),
            name: header.name.raw().0.clone(),
            descriptor: header.descriptor.raw().0.clone(),
            access_flags: header.access_flags,
            has_code: header
                .attributes
                .iter()
                .any(|attribute| attribute.name.raw().0 == b"Code"),
            access_marker_owner: None,
        }
    };
    let unique_method_record = |name: &[u8], descriptor: &[u8]| {
        let matches: Vec<_> = methods
            .iter()
            .enumerate()
            .filter(|(_, method)| {
                method.name.raw().0 == name && method.descriptor.raw().0 == descriptor
            })
            .collect();
        match matches.as_slice() {
            [(index, method)] => Some(physical_member(*index, method)),
            _ => None,
        }
    };
    let array_descriptor = format!("()[L{};", owner_text).into_bytes();
    let value_of_descriptor = format!("(Ljava/lang/String;)L{};", owner_text).into_bytes();
    let values_field: Vec<_> = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| field.name.raw().0 == b"$VALUES")
        .collect();
    let [(values_field_index, values_field)] = values_field.as_slice() else {
        return Ok(Vec::new());
    };
    if values_field.descriptor.raw().0 != format!("[L{};", owner_text).as_bytes()
        || values_field.access_flags != (ACC_PRIVATE | ACC_STATIC | ACC_FINAL | ACC_SYNTHETIC)
        || !values_field.attributes.is_empty()
    {
        return Ok(Vec::new());
    }
    let mut implicit_members = vec![PendingEnumConstantBodyMember {
        table_index: u64::try_from(*values_field_index).unwrap_or(u64::MAX),
        name: values_field.name.raw().0.clone(),
        descriptor: values_field.descriptor.raw().0.clone(),
        access_flags: values_field.access_flags,
        has_code: false,
        access_marker_owner: None,
    }];
    for (name, descriptor, flags) in [
        (b"<clinit>".as_slice(), b"()V".as_slice(), ACC_STATIC),
        (
            b"values".as_slice(),
            array_descriptor.as_slice(),
            0x0001 | ACC_STATIC,
        ),
        (
            b"valueOf".as_slice(),
            value_of_descriptor.as_slice(),
            0x0001 | ACC_STATIC,
        ),
        (
            b"$values".as_slice(),
            array_descriptor.as_slice(),
            ACC_PRIVATE | ACC_STATIC | ACC_SYNTHETIC,
        ),
        (b"<init>".as_slice(), BASE_CTOR, ACC_PRIVATE),
    ] {
        let Some(member) = unique_method_record(name, descriptor) else {
            return Ok(Vec::new());
        };
        if member.access_flags != flags || !member.has_code {
            return Ok(Vec::new());
        }
        implicit_members.push(member);
    }
    let mut constructors: Vec<_> = methods
        .iter()
        .enumerate()
        .filter(|(_, method)| method.name.raw().0 == b"<init>")
        .map(|(index, method)| physical_member(index, method))
        .collect();
    let bridge_indexes: Vec<_> = constructors
        .iter()
        .filter_map(|method| {
            if method.access_flags == ACC_SYNTHETIC {
                enum_access_constructor_marker_owner(&method.descriptor)
                    .map(|owner| (method.table_index, owner))
            } else {
                None
            }
        })
        .collect();
    let needs_bridge = constant_shape
        .iter()
        .any(|constant| constant.allocation_owner != enum_owner);
    if constructors
        .iter()
        .filter(|method| method.descriptor == BASE_CTOR)
        .count()
        != 1
        || bridge_indexes.len() != usize::from(needs_bridge)
        || bridge_indexes.iter().any(|(table_index, owner)| {
            !constructors
                .iter()
                .any(|constructor| constructor.table_index == *table_index && constructor.has_code)
                || !constant_shape.iter().any(|constant| {
                    constant.allocation_owner == *owner && constant.allocation_owner != enum_owner
                })
        })
        || constructors.iter().any(|constructor| {
            constructor.descriptor != BASE_CTOR
                && !bridge_indexes
                    .iter()
                    .any(|(table_index, _)| *table_index == constructor.table_index)
        })
    {
        return Ok(Vec::new());
    }
    for (table_index, owner) in bridge_indexes {
        if let Some(constructor) = constructors
            .iter_mut()
            .find(|constructor| constructor.table_index == table_index)
        {
            constructor.access_marker_owner = Some(owner);
        }
    }
    let abstract_methods: Vec<_> = methods
        .iter()
        .enumerate()
        .filter(|(_, method)| method.access_flags & ACC_ABSTRACT != 0)
        .map(|(index, method)| physical_member(index, method))
        .collect();
    if (enum_access_flags & ACC_ABSTRACT != 0) != !abstract_methods.is_empty()
        || abstract_methods.iter().any(|method| method.has_code)
        || methods.iter().enumerate().any(|(index, method)| {
            !physical_member(index, method).has_code && method.access_flags & ACC_ABSTRACT == 0
        })
    {
        return Ok(Vec::new());
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(code_candidates.len()).unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    let factory_codes: Vec<_> = code_candidates
        .iter()
        .filter(|candidate| {
            candidate.table_index == implicit_members[4].table_index
                && candidate.member.as_ref().is_some_and(|member| {
                    member.owner == clinit_identity.owner
                        && member.name.0 == b"$values"
                        && member.descriptor.0 == implicit_members[4].descriptor
                })
        })
        .collect();
    let factory_code = match factory_codes.as_slice() {
        [factory] => Some(*factory),
        _ => None,
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(
            code.instructions.len() + factory_code.map_or(0, |factory| factory.instructions.len()),
        )
        .unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    let initializer_prefix = prove_enum_body_initializer_prefix(
        code,
        factory_code,
        enum_owner,
        &constant_shape,
        &implicit_members[0],
    );
    let constructor_proof_work = code_candidates
        .len()
        .saturating_mul(constructors.len())
        .saturating_add(
            code_candidates
                .iter()
                .filter(|candidate| {
                    constructors
                        .iter()
                        .any(|constructor| constructor.table_index == candidate.table_index)
                })
                .map(|candidate| candidate.instructions.len())
                .sum::<usize>(),
        )
        .saturating_add(constant_shape.len());
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(constructor_proof_work).unwrap_or(u64::MAX),
    )?;
    budget.poll()?;
    let constructor_identity = |member: &PendingEnumConstantBodyMember| PhysicalMethodId {
        owner: clinit_identity.owner.clone(),
        name: JvmBytes(member.name.clone()),
        descriptor: JvmBytes(member.descriptor.clone()),
    };
    let constructor_chain = (|| {
        let private = constructors
            .iter()
            .find(|member| member.descriptor == BASE_CTOR)
            .ok_or_else(|| "the unique private enum constructor is absent".to_owned())?;
        let private_edge = prove_enum_constructor_candidate(
            code_candidates,
            private,
            &constructor_identity(private),
            b"java/lang/Enum",
            BASE_CTOR,
        )?;
        let mut edges = vec![private_edge];
        if needs_bridge {
            let access = constructors
                .iter()
                .find(|member| member.access_marker_owner.is_some())
                .ok_or_else(|| "the unique synthetic access constructor is absent".to_owned())?;
            edges.push(prove_enum_constructor_candidate(
                code_candidates,
                access,
                &constructor_identity(access),
                enum_owner,
                BASE_CTOR,
            )?);
        }
        Ok(edges)
    })();
    let direct_constant_bcis = (|| {
        initializer_prefix.as_ref().map_err(Clone::clone)?;
        constructor_chain.as_ref().map_err(Clone::clone)?;
        Ok(constant_shape
            .iter()
            .filter(|constant| constant.constructor_owner == enum_owner)
            .map(|constant| constant.constructor_bci)
            .collect())
    })();
    let group_shape = PendingEnumConstantBodyGroupShape {
        enum_access_flags,
        constants: constant_shape,
        implicit_members,
        abstract_methods,
        constructors,
        constructor_chain,
        direct_constant_bcis,
        initializer_prefix,
    };

    let mut relations = Vec::new();
    for (field_index, allocation, constructor_bci, descriptor) in constructions {
        if allocation.class.as_bytes() == enum_owner {
            continue;
        }
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let Some((definition, child_read)) = resolve_class_source_dependency_read_raw(
            content,
            environment,
            Some(clinit_identity),
            allocation.class.as_bytes(),
            execution,
            budget,
        )?
        else {
            continue;
        };
        if child_read.facts.stopped_at.is_some()
            || child_read.facts.this_class.raw().0.as_slice() != allocation.class.as_bytes()
            || child_read
                .facts
                .super_class
                .as_ref()
                .map(|name| name.raw().0.as_slice())
                != Some(enum_owner)
        {
            continue;
        }
        let nesting_shells: Vec<_> = child_read
            .facts
            .attributes
            .iter()
            .filter(|attribute| {
                matches!(
                    attribute.name.raw().0.as_slice(),
                    b"InnerClasses" | b"EnclosingMethod"
                )
            })
            .cloned()
            .collect();
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(nesting_shells.len()).unwrap_or(u64::MAX),
        )?;
        let child_pool = match class_constant_pool(&child_read.bytes, budget) {
            Ok(pool) => pool,
            Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => {
                return Err(error);
            }
            Err(_) => continue,
        };
        let nesting = match class_source::read_class_source_assembly_context(
            &child_read.bytes,
            &nesting_shells,
            &child_pool,
            budget,
        ) {
            Ok(nesting) => nesting,
            Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => {
                return Err(error);
            }
            Err(_) => continue,
        };
        let inner_matches: Vec<_> = nesting
            .inner_classes
            .iter()
            .filter(|inner| {
                jarde_reader::classfile::cp_class_name(&child_pool, inner.class_index)
                    .is_ok_and(|name| name.0 == allocation.class.as_bytes())
            })
            .collect();
        let [inner] = inner_matches.as_slice() else {
            continue;
        };
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(nesting.inner_classes.len()).unwrap_or(u64::MAX),
        )?;
        let mut inner_rows = std::collections::BTreeMap::<Vec<u8>, (usize, bool)>::new();
        for row in &nesting.inner_classes {
            budget.poll()?;
            if let Ok(name) = jarde_reader::classfile::cp_class_name(&child_pool, row.class_index) {
                let entry = inner_rows.entry(name.0).or_insert((0, false));
                entry.0 += 1;
                entry.1 = row.outer_class_index == 0 && row.inner_name.is_none();
            }
        }
        let anonymous_inner_owners = inner_rows
            .into_iter()
            .filter_map(|(owner, (count, anonymous))| (count == 1 && anonymous).then_some(owner))
            .collect();
        let enum_inner_rows: Vec<_> = enum_nesting
            .inner_classes
            .iter()
            .filter(|inner| {
                jarde_reader::classfile::cp_class_name(enum_pool, inner.class_index)
                    .is_ok_and(|name| name.0 == allocation.class.as_bytes())
            })
            .collect();
        let [enum_inner] = enum_inner_rows.as_slice() else {
            continue;
        };
        let Some(enclosing) = nesting.enclosing_method.as_ref() else {
            continue;
        };
        if inner.outer_class_index != 0
            || inner.inner_name.is_some()
            || enum_inner.outer_class_index != 0
            || enum_inner.inner_name.is_some()
            || jarde_reader::classfile::cp_class_name(&child_pool, enclosing.class_index)
                .map_or(true, |name| name.0 != enum_owner)
            || enclosing.method_index != 0
        {
            continue;
        }
        let child_constructors: Vec<_> = child_read
            .facts
            .methods
            .iter()
            .filter(|method| method.name.raw().0 == b"<init>")
            .collect();
        budget.charge(
            CountedBudgetDimension::AnalysisSteps,
            u64::try_from(child_read.facts.methods.len()).unwrap_or(u64::MAX),
        )?;
        let matching_constructors: Vec<_> = child_constructors
            .iter()
            .filter(|method| method.descriptor.raw().0 == descriptor.as_slice())
            .collect();
        let [child_constructor] = matching_constructors.as_slice() else {
            continue;
        };
        if child_constructors.len() != 1 {
            continue;
        }
        let constructor_bridge = if let Some(access) = group_shape
            .constructors
            .iter()
            .find(|member| member.access_marker_owner.is_some())
        {
            prove_enum_physical_constructor(
                &child_read.bytes,
                child_constructor,
                member_identity(&definition, child_constructor),
                &child_pool,
                enum_owner,
                &access.descriptor,
                true,
                budget,
            )?
        } else {
            Err("the selected child has no unique access constructor target".to_owned())
        };
        let mut child_code_evidence = Ok(());
        for method in &child_read.facts.methods {
            budget.poll()?;
            if method.name.raw().0 == b"<init>" || method.name.raw().0 == b"<clinit>" {
                continue;
            }
            let code =
                match jarde_reader::classfile::method_code_facts(&child_read.bytes, method, budget)
                {
                    Ok(code) => code,
                    Err(error @ (Error::BudgetExceeded { .. } | Error::Cancelled { .. })) => {
                        return Err(error);
                    }
                    Err(error) => {
                        child_code_evidence =
                            Err(format!("child override Code could not be read: {error}"));
                        break;
                    }
                };
            if code.stopped_at.is_some()
                || !matches!(code.execution, ExecutionReport::Complete { .. })
                || code.exception_handler_count != 0
                || !code.exception_handlers.is_empty()
            {
                child_code_evidence =
                    Err("child override has exceptional or incomplete Code".to_owned());
                break;
            }
        }
        relations.push(PendingEnumConstantBodyRelation {
            field_index: u64::try_from(field_index).unwrap_or(u64::MAX),
            allocation_bci: allocation.head_bci,
            constructor_bci,
            constructor_descriptor: descriptor.clone(),
            subclass_owner: allocation.class.as_bytes().to_vec(),
            subclass: definition,
            anonymous_inner_owners,
            use_census: PendingEnumConstantBodyUseCensus::default(),
            group_shape: group_shape.clone(),
            constructor_bridge,
            child_code_evidence,
            body_proof: None,
        });
    }
    Ok(relations)
}

/// A child is read through the same class-source member path as an independent request. That
/// path prepares its selected definition once and recovers each Code member once; retaining its
/// records here lets the later projection consume the exact declaration and source map already
/// paid for. A body verdict is private and cannot turn the parent group into `Proved` by itself.
fn prove_enum_constant_child_body(
    engine: &Engine,
    content: &[ArtifactSnapshot],
    request: &ClassSourceRequest,
    relation: &PendingEnumConstantBodyRelation,
    enum_methods: &[MemberHeader],
    budget: &mut Budget,
) -> Result<(
    std::result::Result<Vec<ClassSourceMethod>, String>,
    ExecutionReport,
)> {
    let refuse = |reason: &str| Err(reason.to_owned());
    if let Err(reason) = &relation.child_code_evidence {
        return Ok((
            Err(reason.clone()),
            ExecutionReport::Complete {
                usage: budget.usage(),
            },
        ));
    }
    let child_request = ClassSourceRequest {
        class: ClassRef::Definition {
            definition: relation.subclass.clone(),
        },
        environment: request.environment.clone(),
    };
    // The proof needs region, declaration and source-map details even when the caller asks for
    // essential evidence; optional public evidence selection cannot decide proof admission.
    let OperationOutcome::Performed(child) = engine.class_source_with_evidence(
        content,
        &child_request,
        &RecoveryEvidenceRequest::all(),
        budget,
    )?
    else {
        return Ok((
            refuse("the selected child source is not a unique physical class"),
            ExecutionReport::Complete {
                usage: budget.usage(),
            },
        ));
    };
    let execution = child.execution.clone();
    if !matches!(&execution, ExecutionReport::Complete { .. }) {
        return Ok((
            refuse("the selected child source did not complete"),
            execution,
        ));
    }
    if child.class != relation.subclass || child.declaration.is_none() {
        return Ok((
            refuse("the selected child source changed physical identity"),
            execution,
        ));
    }
    if !child.fields.is_empty() {
        return Ok((
            refuse("the selected child has an instance or static field"),
            execution,
        ));
    }
    let code_count = child
        .methods
        .iter()
        .filter(|method| {
            matches!(
                method.item.body,
                crate::MemberBodyEvidence::CodeAttribute { .. }
            )
        })
        .count();
    let body_runs: Vec<_> = child
        .coverage
        .artifact_structural
        .scanned
        .iter()
        .filter(|range| range.label == "class_source_bodies")
        .collect();
    if !matches!(body_runs.as_slice(), [range] if range.start == 0 && range.end == code_count as u64)
        || child.methods.iter().any(|method| {
            !matches!(
                &method.outcome,
                class_source::ClassSourceOutcome::Recovered { .. }
            )
        })
    {
        return Ok((
            refuse("the selected child did not complete one run per Code member"),
            execution,
        ));
    }
    let expected_constructor = PhysicalMethodId {
        owner: relation.subclass.clone(),
        name: JvmBytes(b"<init>".to_vec()),
        descriptor: JvmBytes(relation.constructor_descriptor.clone()),
    };
    let mut constructors = 0;
    let mut overrides = Vec::new();
    for method in &child.methods {
        budget.poll()?;
        let name = method.item.name.raw().0.as_slice();
        if name == b"<init>" {
            constructors += 1;
            if method.item.identity != expected_constructor
                || method.item.access_flags != 0
                || !matches!(
                    &method.outcome,
                    class_source::ClassSourceOutcome::Recovered { .. }
                )
            {
                return Ok((
                    refuse("the child constructor is not the unique compiler constructor"),
                    execution,
                ));
            }
            continue;
        }
        if name == b"<clinit>" {
            return Ok((
                refuse("the selected child has class initialization"),
                execution,
            ));
        }
        let Some(source_name) = std::str::from_utf8(name).ok() else {
            return Ok((
                refuse("the selected child method name is not Java text"),
                execution,
            ));
        };
        let flags = method.item.access_flags;
        if !jarde_java::is_java_identifier(source_name)
            || flags & (0x0002 | 0x0008 | 0x0040 | 0x0100 | 0x0400 | 0x1000) != 0
        {
            return Ok((
                refuse("the selected child has a non-source override member"),
                execution,
            ));
        }
        let matching_base: Vec<_> = enum_methods
            .iter()
            .filter(|base| {
                base.name.raw().0 == name
                    && base.descriptor.raw().0 == method.item.descriptor.raw().0
            })
            .collect();
        let [base] = matching_base.as_slice() else {
            return Ok((
                refuse("the selected child method has no unique base declaration"),
                execution,
            ));
        };
        if base.access_flags & (0x0002 | 0x0008 | 0x0010) != 0
            || (base.access_flags & 0x0001 != 0 && flags & 0x0001 == 0)
            || (base.access_flags & 0x0004 != 0 && flags & (0x0001 | 0x0004) == 0)
        {
            return Ok((
                refuse("the child method cannot override its base declaration"),
                execution,
            ));
        }
        if !complete_enum_child_override(method) {
            return Ok((
                refuse("the child override body or declaration is not fully presentable"),
                execution,
            ));
        }
        overrides.push(method.clone());
    }
    if constructors != 1 || overrides.is_empty() {
        return Ok((
            refuse("the child does not have one constructor and source overrides"),
            execution,
        ));
    }
    Ok((Ok(overrides), execution))
}

fn complete_enum_child_override(method: &ClassSourceMethod) -> bool {
    let class_source::ClassSourceOutcome::Recovered { report, analysis } = &method.outcome else {
        return false;
    };
    matches!(&analysis.execution, ExecutionReport::Complete { .. })
        && matches!(&report.execution, ExecutionReport::Complete { .. })
        && report.produced()
        && report.representation == crate::ir::Representation::Java
        && report.quality == Quality::Structured
        && report.syntax_status != crate::ir::SyntaxStatus::NotJava
        && report.content == RecoveryContent::ContainsStatements
        && report.fallbacks.is_empty()
        && [
            RecoveryEvidenceKind::SourceMap,
            RecoveryEvidenceKind::RegionDetails,
            RecoveryEvidenceKind::RuleDetails,
        ]
        .into_iter()
        .all(|kind| report.evidence.state(kind) == jarde_java::EvidenceState::Complete)
        && !report.regions.is_empty()
        && report.regions.iter().all(|region| region.structured)
        && report
            .source_map
            .segments()
            .iter()
            .any(|segment| segment.origin().primary().method() == Some(&method.item.identity))
        && report
            .declaration
            .as_ref()
            .is_some_and(|declaration| declaration.presented())
        && method.declaration.is_some()
        && method.markers.is_empty()
}

/// Join the already-frozen main-class shape with the selected child proofs. This stage checks
/// the generated API and terminal initializer, which the pending relation did not prove.
fn prove_enum_constant_body_group(
    declaration: &class_source::ClassSourceDeclaration,
    source_fields: &[ClassSourceField],
    source_methods: &[ClassSourceMethod],
    codes: &[crate::enum_constants::EnumMethodCodeCandidate],
    initializers: &[jarde_java::report::ClassInitializerCandidates],
    relations: &[PendingEnumConstantBodyRelation],
    budget: &mut Budget,
) -> Result<std::result::Result<crate::enum_constants::ProvedEnumConstantBodyGroup, String>> {
    use crate::enum_constants::{
        EnumCodeReference, ProvedEnumBodyConstant, ProvedEnumConstantBodyGroup,
    };
    let refuse = |reason: &str| Err(reason.to_owned());
    let Some(shape) = relations.first().map(|relation| &relation.group_shape) else {
        return Ok(refuse("the body group has no selected child"));
    };
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(source_methods.len() + relations.len()).unwrap_or(u64::MAX),
    )?;
    let owner = declaration.item.declaration.this_class.raw().0.as_slice();
    let (Ok(prefix), Ok(chain), Ok(direct_bcis)) = (
        &shape.initializer_prefix,
        &shape.constructor_chain,
        &shape.direct_constant_bcis,
    ) else {
        return Ok(refuse(
            "the pending initializer or constructor chain refused",
        ));
    };
    if shape.constants.len() != 2
        || shape.implicit_members.len() != 6
        || chain.is_empty()
        || prefix.constant_field_write_bcis.len() != 2
        || relations
            .iter()
            .any(|relation| relation.group_shape != *shape)
    {
        return Ok(refuse("the pending two-constant group is incomplete"));
    }

    let code_at = |index: u64| -> Option<&crate::enum_constants::EnumMethodCodeCandidate> {
        let matching: Vec<_> = codes
            .iter()
            .filter(|code| code.table_index == index)
            .collect();
        let [code] = matching.as_slice() else {
            return None;
        };
        let source = source_methods.get(usize::try_from(index).ok()?)?;
        (code.complete && code.member.as_ref() == Some(&source.item.identity)).then_some(*code)
    };
    for method in source_methods {
        budget.poll()?;
        if matches!(
            method.item.body,
            crate::MemberBodyEvidence::CodeAttribute { .. }
        ) && code_at(method.item.index).is_none()
        {
            return Ok(refuse(
                "a main enum Code member has no complete same-run candidate",
            ));
        }
    }
    let [
        backing,
        initializer,
        values,
        value_of,
        factory,
        private_constructor,
    ] = shape.implicit_members.as_slice()
    else {
        return Ok(refuse("the implicit member set is incomplete"));
    };
    if backing.name != b"$VALUES"
        || initializer.name != b"<clinit>"
        || values.name != b"values"
        || value_of.name != b"valueOf"
        || factory.name != b"$values"
        || private_constructor.name != b"<init>"
    {
        return Ok(refuse("the pending implicit members changed"));
    }
    let Some(values_code) = code_at(values.table_index) else {
        return Ok(refuse("values Code is absent"));
    };
    let Some(value_of_code) = code_at(value_of.table_index) else {
        return Ok(refuse("valueOf Code is absent"));
    };
    if values_code.exception_handler_count != 0
        || value_of_code.exception_handler_count != 0
        || !crate::enum_constants::prove_values(values_code, owner, b"$VALUES")
        || !crate::enum_constants::prove_value_of(value_of_code, owner)
    {
        return Ok(refuse(
            "the generated enum API does not preserve its standard behavior",
        ));
    }
    let Some(clinit) = code_at(initializer.table_index) else {
        return Ok(refuse("initializer Code is absent"));
    };
    let suffix: Vec<_> = clinit
        .instructions
        .iter()
        .filter(|instruction| instruction.bci >= prefix.prefix_end_bci)
        .collect();
    if clinit.exception_handler_count != 0
        || !matches!(suffix.as_slice(), [end]
            if end.bci == prefix.prefix_end_bci && end.width == 1 && end.opcode == 0xb1
                && end.reference.is_none() && end.immediate.is_none() && end.local.is_none())
    {
        return Ok(refuse(
            "the initializer has effects after the proved constant prefix",
        ));
    }
    let Some(clinit_source) = usize::try_from(initializer.table_index)
        .ok()
        .and_then(|index| source_methods.get(index))
    else {
        return Ok(refuse("the initializer has no physical method record"));
    };
    if !matches!(&clinit_source.outcome,
        class_source::ClassSourceOutcome::Recovered { report, analysis }
            if report.produced() && report.quality == Quality::Structured
                && report.fallbacks.is_empty()
                && matches!(report.execution, ExecutionReport::Complete { .. })
                && matches!(analysis.execution, ExecutionReport::Complete { .. }))
    {
        return Ok(refuse(
            "the initializer has no complete structured recovery",
        ));
    }
    let matching: Vec<_> = initializers
        .iter()
        .filter(|candidate| candidate.member.as_ref() == Some(&clinit_source.item.identity))
        .collect();
    let [sidecar] = matching.as_slice() else {
        return Ok(refuse("the initializer has no unique structured sidecar"));
    };
    if sidecar.has_exception_handlers
        || sidecar.steps.len() != 4
        || !matches!(&sidecar.steps[3],
            jarde_java::report::ClassInitializerStep::Other {
                order: 3, bci,
                kind: jarde_java::report::ClassInitializerStatementKind::Return,
            } if *bci == prefix.prefix_end_bci)
    {
        return Ok(refuse(
            "the initializer sidecar does not close after three writes",
        ));
    }
    for (order, bci) in prefix
        .constant_field_write_bcis
        .iter()
        .copied()
        .chain(std::iter::once(prefix.values_field_write_bci))
        .enumerate()
    {
        let jarde_java::report::ClassInitializerStep::FieldWrite(write) = &sidecar.steps[order]
        else {
            return Ok(refuse("the initializer sidecar omits a field write"));
        };
        let name = if order < 2 {
            &shape.constants[order].field_name
        } else {
            &backing.name
        };
        let descriptor = if order < 2 {
            [b"L".as_slice(), owner, b";"].concat()
        } else {
            [b"[L".as_slice(), owner, b";"].concat()
        };
        if write.order != order
            || write.bci != bci
            || !write.is_static
            || write.owner.as_bytes() != owner
            || write.name.as_bytes() != name
            || write.descriptor.as_bytes() != descriptor
            || write.op != jarde_java::ast::AssignOp::Assign
            || write
                .field_reads
                .as_ref()
                .is_none_or(|reads| !reads.is_empty())
        {
            return Ok(refuse(
                "an initializer write has unproved structured effects",
            ));
        }
    }
    for abstract_member in &shape.abstract_methods {
        let Some(source) = usize::try_from(abstract_member.table_index)
            .ok()
            .and_then(|index| source_methods.get(index))
        else {
            return Ok(refuse("an abstract enum declaration is absent"));
        };
        if source.item.name.raw().0 != abstract_member.name
            || source.item.descriptor.raw().0 != abstract_member.descriptor
            || abstract_member.access_flags & (0x0002 | 0x0008 | 0x0010 | 0x0040 | 0x0100 | 0x1000)
                != 0
            || abstract_member.has_code
            || !matches!(source.no_body_kind, Some(NoBodyKind::Abstract))
            || !matches!(source.outcome, class_source::ClassSourceOutcome::NoBody)
            || source.declaration.is_none()
        {
            return Ok(refuse("an abstract enum declaration is not complete"));
        }
    }
    for code in codes {
        if shape
            .implicit_members
            .iter()
            .any(|member| member.table_index == code.table_index)
        {
            continue;
        }
        if code
            .member_uses
            .iter()
            .any(|use_site| match &use_site.reference {
                EnumCodeReference::Field {
                    owner: use_owner,
                    name,
                    ..
                } => use_owner == owner && name == b"$VALUES",
                EnumCodeReference::Method {
                    owner: use_owner,
                    name,
                    ..
                } => use_owner == owner && name == b"$values",
                _ => false,
            })
        {
            return Ok(refuse("a user method uses an implicit enum member"));
        }
    }

    let mut proved = Vec::with_capacity(2);
    let mut selected_children = std::collections::HashSet::new();
    for (ordinal, constant) in shape.constants.iter().enumerate() {
        budget.poll()?;
        let Some(field) = usize::try_from(constant.field_index)
            .ok()
            .and_then(|index| source_fields.get(index))
        else {
            return Ok(refuse("an enum constant field is absent"));
        };
        let Some(name) = String::from_utf16(field.item.name.utf16()).ok() else {
            return Ok(refuse("an enum constant name is not Java text"));
        };
        if field.item.identity.owner != declaration.item.definition
            || field.item.name.raw().0 != constant.field_name
            || field.declaration.is_none()
            || !field.markers.is_empty()
            || !jarde_java::is_java_identifier(&name)
            || constant.expected_ordinal != ordinal as u32
            || constant.descriptor_source_argument_count != 0
            || constant.constructor_descriptor != b"(Ljava/lang/String;I)V"
            || constant.constructor_owner != constant.allocation_owner
        {
            return Ok(refuse(
                "an ordered zero-argument constant is not presentable",
            ));
        }
        let matching: Vec<_> = relations
            .iter()
            .filter(|relation| relation.field_index == constant.field_index)
            .collect();
        let (subclass, methods) = if constant.allocation_owner == owner {
            if !matching.is_empty() || !direct_bcis.contains(&constant.constructor_bci) {
                return Ok(refuse("a direct constant has an unexpected child relation"));
            }
            (None, None)
        } else {
            let [relation] = matching.as_slice() else {
                return Ok(refuse("an anonymous constant has no unique selected child"));
            };
            if relation.allocation_bci != constant.allocation_bci
                || relation.constructor_bci != constant.constructor_bci
                || relation.constructor_descriptor != constant.constructor_descriptor
                || relation.subclass_owner != constant.allocation_owner
                || !relation.use_census.exclusive
                || relation.use_census.refusal.is_some()
                || relation.constructor_bridge.is_err()
            {
                return Ok(refuse(
                    "the selected child use or constructor proof is incomplete",
                ));
            }
            let Some(Ok(methods)) = relation.body_proof.as_ref() else {
                return Ok(refuse("a selected child lacks complete override bodies"));
            };
            if shape.abstract_methods.iter().any(|abstract_member| {
                methods
                    .iter()
                    .filter(|method| {
                        method.item.name.raw().0 == abstract_member.name
                            && method.item.descriptor.raw().0 == abstract_member.descriptor
                            && complete_enum_child_override(method)
                    })
                    .count()
                    != 1
            }) {
                return Ok(refuse(
                    "a constant body does not implement every abstract enum method",
                ));
            }
            if !selected_children.insert(&relation.subclass) {
                return Ok(refuse("one selected child is assigned to two constants"));
            }
            (Some(relation.subclass.clone()), Some(methods.clone()))
        };
        if subclass.is_none() && !shape.abstract_methods.is_empty() {
            return Ok(refuse("an abstract enum has a constant without a body"));
        }
        proved.push(ProvedEnumBodyConstant {
            field_index: constant.field_index,
            allocation_bci: constant.allocation_bci,
            constructor_bci: constant.constructor_bci,
            field_write_bci: prefix.constant_field_write_bcis[ordinal],
            subclass,
            methods,
        });
    }
    if relations.len() != selected_children.len() {
        return Ok(refuse(
            "the child relations do not cover exactly the anonymous constants",
        ));
    }
    Ok(Ok(ProvedEnumConstantBodyGroup { constants: proved }))
}

/// Census each selected input through P1's physical scanner. A relation is exclusive only when
/// every selected range was fully read and its entire owner-reference set has an explanation.
/// This says nothing about the constructed object's stack value or constructor arguments.
fn census_enum_constant_body_uses(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    enum_definition: &PhysicalDefinitionId,
    relations: &mut [PendingEnumConstantBodyRelation],
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<()> {
    use jarde_query::query::{XrefCertainty, XrefOperation, XrefTarget};
    use jarde_query::xref::{CandidateFilter, scan_candidates};
    use jarde_reader::model::SymbolRef;

    if relations.is_empty() {
        return Ok(());
    }
    budget.charge(
        CountedBudgetDimension::AnalysisSteps,
        u64::try_from(environment.runtime.load_domain.roots.len() + relations.len())
            .unwrap_or(u64::MAX),
    )?;
    let mut ranges = Vec::<(SnapshotId, PhysicalScope)>::new();
    let mut unscannable_root = false;
    for root in &environment.runtime.load_domain.roots {
        let range = match root {
            LoadRoot::StandaloneClass { snapshot } => {
                Some((snapshot.clone(), PhysicalScope::SnapshotAll))
            }
            LoadRoot::Container { origin, prefix } => {
                // P1 scopes a container tree, not a load-root prefix or one nested path. A
                // wider scan may expose real extra uses, but cannot certify the narrower root.
                if !prefix.0.is_empty() || !origin.steps.is_empty() {
                    unscannable_root = true;
                }
                Some((
                    origin.snapshot.clone(),
                    PhysicalScope::ArtifactTree {
                        root_container: origin.root_container.clone(),
                    },
                ))
            }
            LoadRoot::External { .. } => None,
        };
        if let Some(range) = range {
            if !ranges.contains(&range) {
                ranges.push(range);
            }
        } else {
            unscannable_root = true;
        }
    }
    let consumers = ConsumerSchema::new(
        1,
        [
            ConsumerKind::Invocation,
            ConsumerKind::Field,
            ConsumerKind::Type,
            ConsumerKind::Constant,
            ConsumerKind::Exception,
            ConsumerKind::Signature,
            ConsumerKind::Annotation,
            ConsumerKind::InnerNest,
            ConsumerKind::Module,
            ConsumerKind::Bootstrap,
            ConsumerKind::Resource,
        ],
    );
    let clinit = PhysicalMethodId {
        owner: enum_definition.clone(),
        name: JvmBytes(b"<clinit>".to_vec()),
        descriptor: JvmBytes(b"()V".to_vec()),
    };
    let selected_children: Vec<_> = relations
        .iter()
        .map(|relation| {
            (
                relation.subclass.clone(),
                relation.subclass_owner.clone(),
                relation.anonymous_inner_owners.clone(),
            )
        })
        .collect();
    for relation in relations {
        let census = &mut relation.use_census;
        let mut new_count = 0;
        let mut constructor_count = 0;
        let mut access_descriptor_count = 0;
        if ranges.is_empty() || unscannable_root {
            census.refusal.get_or_insert_with(|| {
                "a selected input root cannot be completely scanned".to_owned()
            });
        }
        for (snapshot_id, scope) in &ranges {
            budget.poll()?;
            let Some(snapshot) = content
                .iter()
                .find(|candidate| candidate.id() == snapshot_id)
            else {
                census
                    .refusal
                    .get_or_insert_with(|| "a selected input snapshot was not provided".to_owned());
                continue;
            };
            let scan = scan_candidates(
                snapshot,
                scope,
                &consumers,
                CandidateFilter::Owner {
                    owner: JvmBytes(relation.subclass_owner.clone()),
                },
                0,
                budget,
            )?;
            let complete = !scan.has_more
                && matches!(&scan.execution, ExecutionReport::Complete { .. })
                && scan.coverage.dimensions.artifact_structural.state
                    == CoverageState::CompleteWithinSchema
                && scan.coverage.unknown_candidates == 0
                && scan.coverage.unsupported_categories.is_empty();
            if !complete {
                census.refusal.get_or_insert_with(|| {
                    "the selected input owner scan is incomplete".to_owned()
                });
            }
            for item in &scan.items {
                let allowed_code = match (&item.source.location, &item.target) {
                    (
                        Location::Code { method, bci },
                        XrefTarget::Symbol {
                            value: SymbolRef::Class { owner },
                        },
                    ) if method == &clinit
                        && *bci == relation.allocation_bci
                        && owner.0 == relation.subclass_owner
                        && item.operation == XrefOperation::New
                        && item.consumer == Some(ConsumerKind::Type)
                        && item.evidence.bci == Some(*bci)
                        && item.evidence.opcode == Some(0xbb) =>
                    {
                        new_count += 1;
                        true
                    }
                    (
                        Location::Code { method, bci },
                        XrefTarget::Symbol {
                            value:
                                SymbolRef::Method {
                                    owner,
                                    name,
                                    descriptor,
                                },
                        },
                    ) if method == &clinit
                        && *bci == relation.constructor_bci
                        && owner.0 == relation.subclass_owner
                        && name.0 == b"<init>"
                        && descriptor.0 == relation.constructor_descriptor
                        && item.operation == XrefOperation::InvokeSpecial
                        && item.consumer == Some(ConsumerKind::Invocation)
                        && item.evidence.bci == Some(*bci)
                        && item.evidence.opcode == Some(0xb7) =>
                    {
                        constructor_count += 1;
                        true
                    }
                    _ => false,
                };
                // 2.1b checked unique typed InnerClasses self rows in both the enum and the
                // child. Those are declaration metadata, not a second object use. Any other
                // class, metadata operation, method, field or handle is additional use.
                let allowed_nesting = matches!(
                    (&item.source.location, &item.target),
                    (
                        Location::ClassOffset { definition, .. },
                        XrefTarget::Symbol {
                            value: SymbolRef::Class { owner },
                        },
                    ) if owner.0 == relation.subclass_owner
                        && item.operation == XrefOperation::InnerClass
                        && item.consumer == Some(ConsumerKind::InnerNest)
                        && (definition == enum_definition
                            || definition == &relation.subclass
                            || selected_children.iter().any(|(selected, _, rows)| {
                                selected == definition
                                    && rows.contains(&relation.subclass_owner)
                            }))
                );
                // 2.1c recorded the one synthetic access constructor and its exact marker
                // owner. P1 reports descriptor types at the UTF8/class coordinate, so require
                // exactly one such item across the census: a second declaration with that
                // descriptor cannot be attributed to the verified bridge.
                let allowed_access_descriptor = matches!(
                    (&item.source.location, &item.target),
                    (
                        Location::ClassOffset { definition, .. },
                        XrefTarget::Symbol {
                            value: SymbolRef::Class { owner },
                        },
                    ) if definition == enum_definition
                        && owner.0 == relation.subclass_owner
                        && item.operation == XrefOperation::MethodDescriptor
                        && item.consumer == Some(ConsumerKind::Type)
                        && relation.group_shape.constructors.iter().any(|constructor| {
                            constructor.access_marker_owner.as_deref()
                                == Some(relation.subclass_owner.as_slice())
                        })
                );
                if allowed_access_descriptor {
                    access_descriptor_count += 1;
                }
                if item.certainty != XrefCertainty::Exact
                    || !(allowed_code || allowed_nesting || allowed_access_descriptor)
                {
                    census.refusal.get_or_insert_with(|| {
                        "the subclass has an additional owner use".to_owned()
                    });
                }
            }
            merge_execution(execution, scan.execution.clone());
            census.scans.push(PendingEnumConstantBodyUseScan {
                snapshot: snapshot_id.clone(),
                scope: scope.clone(),
                items: scan.items,
                has_more: scan.has_more,
                coverage: scan.coverage,
                execution: scan.execution,
                diagnostics: scan.diagnostics,
            });
        }
        if new_count != 1 || constructor_count != 1 {
            census.refusal.get_or_insert_with(|| {
                "the exact enum construction uses were not unique".to_owned()
            });
        }
        let has_access_marker = relation.group_shape.constructors.iter().any(|constructor| {
            constructor.access_marker_owner.as_deref() == Some(relation.subclass_owner.as_slice())
        });
        if access_descriptor_count != usize::from(has_access_marker) {
            census.refusal.get_or_insert_with(|| {
                "the access constructor descriptor use is not unique".to_owned()
            });
        }
        census.exclusive = census.refusal.is_none();
    }
    Ok(())
}

fn enum_access_constructor_marker_owner(descriptor: &[u8]) -> Option<Vec<u8>> {
    let Some(parameter) = descriptor
        .strip_prefix(b"(Ljava/lang/String;I")
        .and_then(|tail| tail.strip_suffix(b";)V"))
    else {
        return None;
    };
    let owner = parameter.strip_prefix(b"L")?;
    if owner.is_empty() || owner.contains(&b';') || owner.contains(&b'[') {
        return None;
    }
    Some(owner.to_vec())
}

fn enum_constant_constructor_matches(
    constructor_owner: &[u8],
    constructor_name: &[u8],
    constructor_descriptor: &[u8],
    allocation_owner: &[u8],
) -> bool {
    constructor_owner == allocation_owner
        && constructor_name == b"<init>"
        && constructor_descriptor == b"(Ljava/lang/String;I)V"
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct InterfaceSuperTarget {
    owner: String,
    name: String,
    descriptor: String,
}

struct SelectedInterfaceNode {
    definition: PhysicalDefinitionId,
    facts: ClassMemberFacts,
}

/// Selected definitions shared by every interface-super candidate in one method request.
struct InterfaceSuperReads<'a> {
    content: &'a [ArtifactSnapshot],
    environment: &'a ResolutionEnvironment,
    enclosing: &'a PhysicalMethodId,
    budget: &'a mut Budget,
    execution: ExecutionReport,
    attempted: std::collections::BTreeSet<Vec<u8>>,
    by_name: std::collections::BTreeMap<Vec<u8>, usize>,
    nodes: Vec<SelectedInterfaceNode>,
}

impl InterfaceSuperReads<'_> {
    fn node(&mut self, owner: &[u8], depth: u64) -> Result<Option<usize>> {
        self.budget.observe_dependency_depth(depth)?;
        if let Some(index) = self.by_name.get(owner) {
            return Ok(Some(*index));
        }
        if !self.attempted.insert(owner.to_vec()) {
            return Ok(None);
        }
        self.budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let Some((definition, read)) = resolve_class_source_dependency_read_raw(
            self.content,
            self.environment,
            Some(self.enclosing),
            owner,
            &mut self.execution,
            self.budget,
        )?
        else {
            member_inner_resolution_stop(&self.execution, self.budget)?;
            return Ok(None);
        };
        member_inner_resolution_stop(&self.execution, self.budget)?;
        if read.facts.stopped_at.is_some() || read.facts.this_class.raw().0.as_slice() != owner {
            return Ok(None);
        }
        if let Some(index) = self
            .nodes
            .iter()
            .position(|node| node.definition == definition)
        {
            self.by_name.insert(owner.to_vec(), index);
            return Ok(Some(index));
        }
        let index = self.nodes.len();
        self.nodes.push(SelectedInterfaceNode {
            definition,
            facts: read.facts,
        });
        self.by_name.insert(owner.to_vec(), index);
        Ok(Some(index))
    }
}

/// Add a fully selected interface closure to this request's small graph. The height memo checks
/// the dependency-depth limit even when a diamond reuses a completed node.
fn ensure_interface_closure(
    owner: &[u8],
    depth: u64,
    reads: &mut InterfaceSuperReads<'_>,
    graph: &mut std::collections::BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
    heights: &mut std::collections::BTreeMap<Vec<u8>, u64>,
    active: &mut Vec<Vec<u8>>,
) -> Result<bool> {
    reads.budget.poll()?;
    if active.iter().any(|ancestor| ancestor.as_slice() == owner) {
        return Ok(false);
    }
    if let Some(height) = heights.get(owner) {
        reads
            .budget
            .observe_dependency_depth(depth.saturating_add(*height))?;
        return Ok(true);
    }
    reads.budget.observe_dependency_depth(depth)?;
    let Some(index) = reads.node(owner, depth)? else {
        return Ok(false);
    };
    let facts = &reads.nodes[index].facts;
    if facts.access_flags & ACC_INTERFACE == 0 {
        return Ok(false);
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut parents = Vec::with_capacity(facts.interfaces.len());
    for parent in &facts.interfaces {
        reads
            .budget
            .charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let raw = parent.raw().0.clone();
        if !seen.insert(raw.clone()) {
            return Ok(false);
        }
        parents.push(raw);
    }
    active.push(owner.to_vec());
    let mut height = 0_u64;
    for parent in &parents {
        let child_depth = depth.saturating_add(1);
        if !ensure_interface_closure(parent, child_depth, reads, graph, heights, active)? {
            active.pop();
            return Ok(false);
        }
        height = height.max(heights.get(parent).copied().unwrap_or(0).saturating_add(1));
    }
    active.pop();
    graph.insert(owner.to_vec(), parents);
    heights.insert(owner.to_vec(), height);
    reads
        .budget
        .observe_dependency_depth(depth.saturating_add(height))?;
    Ok(true)
}

fn interface_reaches(
    start: &[u8],
    target: &[u8],
    graph: &std::collections::BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
    budget: &mut Budget,
) -> Result<Option<bool>> {
    let mut pending = vec![start.to_vec()];
    let mut visited = std::collections::BTreeSet::new();
    while let Some(owner) = pending.pop() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if owner.as_slice() == target {
            return Ok(Some(true));
        }
        if !visited.insert(owner.clone()) {
            continue;
        }
        let Some(parents) = graph.get(&owner) else {
            return Ok(None);
        };
        pending.extend(parents.iter().cloned());
    }
    Ok(Some(false))
}

fn interface_source_type_accessible(current: &[u8], owner: &[u8], flags: u16) -> bool {
    // Nested source names need InnerClasses evidence; this first slice leaves them mapped to the
    // existing refusal path instead of treating a binary `$` name as a source qualifier.
    !owner.contains(&b'$') && (flags & 0x0001 != 0 || package_name(current) == package_name(owner))
}

fn package_name(owner: &[u8]) -> &[u8] {
    owner
        .iter()
        .rposition(|byte| *byte == b'/')
        .map_or(&[], |slash| &owner[..slash])
}

fn prove_interface_super_calls(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    ir: &jarde_jvm::method_ir::MethodIr,
    budget: &mut Budget,
) -> Result<Vec<jarde_java::report::ProvedInterfaceSuperCall>> {
    use std::collections::{BTreeMap, BTreeSet};

    let Some(code) = ir.code() else {
        return Ok(Vec::new());
    };
    let mut candidates = BTreeSet::new();
    for instruction in &code.instructions {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if instruction.opcode != 0xb7 {
            continue;
        }
        let Some(index) = instruction.constant_pool_index else {
            continue;
        };
        let Ok(entry) = jarde_reader::classfile::cp_entry(ir.constant_pool(), index) else {
            continue;
        };
        let CpEntryKind::InterfaceMethodRef {
            owner,
            name,
            descriptor,
            ..
        } = &entry.kind
        else {
            continue;
        };
        if ir
            .direct_interfaces()
            .iter()
            .filter(|interface| interface.raw().0 == owner.0)
            .count()
            != 1
        {
            continue;
        }
        let (Ok(owner), Ok(name), Ok(descriptor)) = (
            std::str::from_utf8(&owner.0),
            std::str::from_utf8(&name.0),
            std::str::from_utf8(&descriptor.0),
        ) else {
            continue;
        };
        candidates.insert(InterfaceSuperTarget {
            owner: owner.to_owned(),
            name: name.to_owned(),
            descriptor: descriptor.to_owned(),
        });
    }
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let Some(declaration) = ir.declaration() else {
        return Ok(Vec::new());
    };
    let current_class = declaration.class_name().0.as_slice();
    let class_is_interface = declaration.class_access_flags() & ACC_INTERFACE != 0;
    let direct_interfaces = ir
        .direct_interfaces()
        .iter()
        .map(|name| name.raw().0.clone())
        .collect::<Vec<_>>();
    let distinct_direct_interfaces = direct_interfaces.iter().collect::<BTreeSet<_>>();
    if distinct_direct_interfaces.len() != direct_interfaces.len() {
        return Ok(Vec::new());
    }
    let initial_usage = budget.usage();
    let mut reads = InterfaceSuperReads {
        content,
        environment: &request.environment,
        enclosing: &request.method,
        budget,
        execution: ExecutionReport::Complete {
            usage: initial_usage,
        },
        attempted: BTreeSet::new(),
        by_name: BTreeMap::new(),
        nodes: Vec::new(),
    };
    let mut graph = BTreeMap::new();
    let mut heights = BTreeMap::new();
    let mut active = Vec::new();
    let mut proved = Vec::new();
    for candidate in candidates {
        let owner = candidate.owner.as_bytes();
        let Some(owner_index) = reads.node(owner, 0)? else {
            continue;
        };
        let owner_flags = reads.nodes[owner_index].facts.access_flags;
        if owner_flags & ACC_INTERFACE == 0
            || !interface_source_type_accessible(current_class, owner, owner_flags)
            || !ensure_interface_closure(
                owner,
                0,
                &mut reads,
                &mut graph,
                &mut heights,
                &mut active,
            )?
        {
            continue;
        }
        let mut nonredundant = true;
        for direct in &direct_interfaces {
            reads
                .budget
                .charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if direct.as_slice() == owner {
                continue;
            }
            if !ensure_interface_closure(
                direct,
                0,
                &mut reads,
                &mut graph,
                &mut heights,
                &mut active,
            )? || interface_reaches(direct, owner, &graph, reads.budget)? != Some(false)
            {
                nonredundant = false;
                break;
            }
        }
        if !nonredundant {
            continue;
        }
        if !class_is_interface {
            let mut parent = ir.direct_super_class().map(|name| name.0.clone());
            let mut seen_classes = BTreeSet::new();
            let mut depth = 0_u64;
            if parent.is_none() && current_class != b"java/lang/Object" {
                continue;
            }
            let mut class_chain_clear = true;
            while let Some(class_name) = parent {
                reads.budget.poll()?;
                if class_name == b"java/lang/Object" {
                    // `java/lang/Object` is the JVM's fixed root: no host JDK bytes are needed.
                    break;
                }
                if !seen_classes.insert(class_name.clone()) {
                    class_chain_clear = false;
                    break;
                }
                depth = depth.saturating_add(1);
                let Some(class_index) = reads.node(&class_name, depth)? else {
                    class_chain_clear = false;
                    break;
                };
                let facts = &reads.nodes[class_index].facts;
                if facts.access_flags & ACC_INTERFACE != 0 {
                    class_chain_clear = false;
                    break;
                }
                let implemented = facts
                    .interfaces
                    .iter()
                    .map(|name| name.raw().0.clone())
                    .collect::<Vec<_>>();
                let next_parent = facts.super_class.as_ref().map(|name| name.raw().0.clone());
                let mut seen_implemented = BTreeSet::new();
                for interface in &implemented {
                    reads
                        .budget
                        .charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                    if !seen_implemented.insert(interface.clone())
                        || !ensure_interface_closure(
                            interface,
                            depth,
                            &mut reads,
                            &mut graph,
                            &mut heights,
                            &mut active,
                        )?
                        || interface_reaches(interface, owner, &graph, reads.budget)? != Some(false)
                    {
                        class_chain_clear = false;
                        break;
                    }
                }
                if !class_chain_clear {
                    break;
                }
                if next_parent.is_none() {
                    class_chain_clear = false;
                    break;
                }
                parent = next_parent;
            }
            if !class_chain_clear {
                continue;
            }
        }
        let (nodes, by_name, proof_budget) = (&reads.nodes, &reads.by_name, &mut *reads.budget);
        if unique_source_default(
            owner,
            candidate.name.as_bytes(),
            candidate.descriptor.as_bytes(),
            nodes,
            by_name,
            &graph,
            proof_budget,
        )? {
            proved.push(jarde_java::report::ProvedInterfaceSuperCall {
                owner: candidate.owner,
                name: candidate.name,
                descriptor: candidate.descriptor,
            });
        }
    }
    Ok(proved)
}

/// The interface-super proof as the presentation consumes it.
///
/// The proof is optional evidence of what the artifact may spell, and a run the budget or the
/// caller's cancellation stopped cannot spend more work on it. It degrades here — to its
/// conservative answer, nothing proved, so no `interface`-qualified super call is spelled — and
/// the stop is stated where every other stop is: by the presentation's own execution plane.
/// Raising a stop in the middle of the presentation path would answer a legal request with an
/// error instead of the partial or cancelled run it can still publish.
fn interface_super_calls_presented(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    ir: &jarde_jvm::method_ir::MethodIr,
    budget: &mut Budget,
) -> Result<Vec<jarde_java::report::ProvedInterfaceSuperCall>> {
    match prove_interface_super_calls(content, request, ir, budget) {
        Ok(proved) => Ok(proved),
        Err(Error::BudgetExceeded { .. } | Error::Cancelled { .. }) => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

fn unique_source_default(
    root: &[u8],
    name: &[u8],
    descriptor: &[u8],
    nodes: &[SelectedInterfaceNode],
    by_name: &std::collections::BTreeMap<Vec<u8>, usize>,
    graph: &std::collections::BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
    budget: &mut Budget,
) -> Result<bool> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_BRIDGE: u16 = 0x0040;
    const ACC_NATIVE: u16 = 0x0100;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_SYNTHETIC: u16 = 0x1000;

    let mut pending = vec![root.to_vec()];
    let mut closure = std::collections::BTreeSet::new();
    while let Some(owner) = pending.pop() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if !closure.insert(owner.clone()) {
            continue;
        }
        let Some(parents) = graph.get(&owner) else {
            return Ok(false);
        };
        pending.extend(parents.iter().cloned());
    }
    let mut declarations: Vec<(Vec<u8>, &MemberHeader)> = Vec::new();
    for owner in &closure {
        budget.poll()?;
        let Some(index) = by_name.get(owner) else {
            return Ok(false);
        };
        let mut exact = 0_u32;
        for method in &nodes[*index].facts.methods {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if method.name.raw().0.as_slice() != name {
                continue;
            }
            if method.descriptor.raw().0.as_slice() != descriptor {
                // This first slice cannot establish which same-named overload Java would bind.
                return Ok(false);
            }
            exact = exact.saturating_add(1);
            if exact > 1
                || method.access_flags & (ACC_BRIDGE | ACC_SYNTHETIC) != 0
                || method
                    .attributes
                    .iter()
                    .any(|attribute| attribute.name.raw().0.as_slice() == b"Signature")
            {
                return Ok(false);
            }
            declarations.push((owner.clone(), method));
        }
    }
    if declarations.is_empty() {
        return Ok(false);
    }
    let mut maximal = Vec::new();
    'candidate: for (owner, method) in &declarations {
        for (other_owner, _) in &declarations {
            if owner == other_owner {
                continue;
            }
            if interface_reaches(other_owner, owner, graph, budget)? != Some(false) {
                continue 'candidate;
            }
        }
        maximal.push(method);
    }
    let [method] = maximal.as_slice() else {
        return Ok(false);
    };
    let flags = method.access_flags;
    let code_count = method
        .attributes
        .iter()
        .filter(|attribute| attribute.name.raw().0.as_slice() == b"Code")
        .count();
    Ok(flags & ACC_PUBLIC != 0
        && flags & (ACC_PRIVATE | ACC_STATIC | ACC_ABSTRACT | ACC_NATIVE) == 0
        && code_count == 1)
}

/// Discover exact constructor references that share a decoded allocation owner. The `$` test is
/// only a cheap demand filter: the target-side InnerClasses row, not this name, proves membership.
/// A target this first slice can spell as `Outer.Inner` necessarily has that binary separator.
fn prove_class_source_member_capture(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    child_definition: &PhysicalDefinitionId,
    root_name: &[u8],
    child: &jarde_reader::classfile::ClassMemberFacts,
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<class_source::ClassSourceMemberCapture> {
    use class_source::ClassSourceMemberCapture as Capture;
    let mut analyses = Vec::new();
    for method in &child.methods {
        budget.poll()?;
        if !method
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Code")
        {
            continue;
        }
        let id = PhysicalMethodId {
            owner: child_definition.clone(),
            name: method.name.raw().clone(),
            descriptor: method.descriptor.raw().clone(),
        };
        let analyzed = jarde_jvm::analyze_method_ir(
            content,
            &crate::ir::MethodAnalysisRequest {
                environment: environment.clone(),
                method: id.clone(),
                stages: MethodOperation::Analysis.stages().to_vec(),
            },
            budget,
        )?;
        merge_execution(execution, analyzed.report().execution.clone());
        if analyzed.report().method != id
            || !matches!(
                analyzed.report().execution,
                ExecutionReport::Complete { .. }
            )
        {
            return Ok(Capture::Refused {
                reason: "member method SSA analysis did not complete".to_owned(),
            });
        }
        analyses.push((id, analyzed));
    }
    let irs: Vec<_> = analyses
        .iter()
        .map(|(id, analyzed)| (id.clone(), analyzed.ir()))
        .collect();
    match crate::member_inner::prove_family_capture(root_name, child, &irs, budget)? {
        Ok(proof) => Ok(Capture::Proved { proof }),
        Err(reason) => Ok(Capture::Refused { reason }),
    }
}

#[allow(clippy::too_many_arguments)]
fn project_class_source_member_family(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    root: &ClassSourceReport,
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<std::result::Result<(String, Vec<class_source::MemberFamilyDerivedProjection>), String>>
{
    use class_source::{ClassSourceMemberCalls, ClassSourceMemberCapture, ClassSourceMemberFamily};
    let ClassSourceMemberFamily::Prepared {
        relation,
        child,
        capture,
        calls,
        ..
    } = &root.member_family
    else {
        return Ok(Err("family relation is not prepared".to_owned()));
    };
    let ClassSourceMemberCapture::Proved { proof } = capture else {
        return Ok(Err("capture proof is incomplete".to_owned()));
    };
    let ClassSourceMemberCalls::Proved { sites } = calls else {
        return Ok(Err("one or more family call sites are unproved".to_owned()));
    };
    if !matches!(root.execution, ExecutionReport::Complete { .. })
        || !matches!(child.execution, ExecutionReport::Complete { .. })
        || root.declaration.is_none()
        || child.declaration.is_none()
    {
        return Ok(Err("physical family recovery is incomplete".to_owned()));
    }
    if let Err(reason) = prove_member_family_external_use_closure(
        content,
        environment,
        root,
        child,
        proof,
        sites,
        execution,
        budget,
    )? {
        return Ok(Err(reason));
    }
    let root_binary = &root
        .declaration
        .as_ref()
        .unwrap()
        .item
        .declaration
        .this_class
        .raw()
        .0;
    let child_binary = &child
        .declaration
        .as_ref()
        .unwrap()
        .item
        .declaration
        .this_class
        .raw()
        .0;
    let (Some(root_name), Some(child_name), Some(descriptor)) = (
        std::str::from_utf8(root_binary).ok(),
        std::str::from_utf8(child_binary).ok(),
        std::str::from_utf8(&proof.constructor.descriptor.0).ok(),
    ) else {
        return Ok(Err(
            "family identities have no exact Java source spelling".to_owned()
        ));
    };
    let root_source_name = root_name.replace('/', ".");
    let mut outer_super_bridges = Vec::new();
    if let Some(super_class) = root
        .declaration
        .as_ref()
        .unwrap()
        .item
        .declaration
        .super_class
        .as_ref()
    {
        for method in root
            .methods
            .iter()
            .filter(|method| method.item.access_flags & 0x1000 != 0)
        {
            let analyzed = jarde_jvm::analyze_method_ir(
                content,
                &crate::ir::MethodAnalysisRequest {
                    environment: environment.clone(),
                    method: method.item.identity.clone(),
                    stages: MethodOperation::Analysis.stages().to_vec(),
                },
                budget,
            )?;
            merge_execution(execution, analyzed.report().execution.clone());
            if analyzed.report().method != method.item.identity
                || !matches!(
                    analyzed.report().execution,
                    ExecutionReport::Complete { .. }
                )
            {
                return Ok(Err(
                    "synthetic root method analysis is incomplete for bridge screening".to_owned(),
                ));
            }
            let Some(code) = analyzed.ir().code() else {
                return Ok(Err(
                    "synthetic root method has no complete body for bridge screening".to_owned(),
                ));
            };
            for instruction in &code.instructions {
                budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
                if instruction.opcode != 0xb7 {
                    continue;
                }
                let Some(index) = instruction.constant_pool_index else {
                    return Ok(Err(
                        "synthetic invokespecial has no proved MethodRef".to_owned()
                    ));
                };
                let Ok(entry) =
                    jarde_reader::classfile::cp_entry(analyzed.ir().constant_pool(), index)
                else {
                    return Ok(Err(
                        "synthetic invokespecial MethodRef is unreadable".to_owned()
                    ));
                };
                let jarde_reader::classfile::CpEntryKind::MethodRef { owner, name, .. } =
                    &entry.kind
                else {
                    return Ok(Err(
                        "synthetic invokespecial target is not a MethodRef".to_owned()
                    ));
                };
                if owner.0 == super_class.raw().0 && name.0 != b"<init>" {
                    let Some((outer_definition, outer_read)) =
                        resolve_class_source_dependency_read_raw(
                            content,
                            environment,
                            None,
                            root_binary,
                            execution,
                            budget,
                        )?
                    else {
                        return Ok(Err(
                            "selected Outer physical definition cannot be reread for bridge proof"
                                .to_owned(),
                        ));
                    };
                    if outer_definition != root.class {
                        return Ok(Err(
                            "bridge proof resolved another Outer physical definition".to_owned(),
                        ));
                    }
                    let Some((superclass_definition, superclass_read)) =
                        resolve_class_source_dependency_read_raw(
                            content,
                            environment,
                            None,
                            &super_class.raw().0,
                            execution,
                            budget,
                        )?
                    else {
                        return Ok(Err(
                            "direct superclass physical definition is unavailable for bridge proof"
                                .to_owned(),
                        ));
                    };
                    let bridge = match crate::member_inner::prove_outer_super_bridge(
                        &outer_read.facts,
                        &superclass_read.facts,
                        &superclass_definition,
                        &method.item.identity,
                        analyzed.ir(),
                        budget,
                    )? {
                        Ok(proof) => proof,
                        Err(reason) => {
                            return Ok(Err(format!("Outer.super bridge refused: {reason}")));
                        }
                    };
                    let closure = match prove_outer_super_bridge_use_closure(
                        content,
                        environment,
                        root,
                        child,
                        proof,
                        &bridge,
                        execution,
                        budget,
                    )? {
                        Ok(closure) => closure,
                        Err(reason) => {
                            return Ok(Err(format!("Outer.super bridge refused: {reason}")));
                        }
                    };
                    if let Err(reason) = prove_outer_super_source_binding(
                        content,
                        environment,
                        &outer_read.facts,
                        &superclass_definition,
                        &superclass_read.facts,
                        &bridge,
                        execution,
                        budget,
                    )? {
                        return Ok(Err(format!("Outer.super source binding refused: {reason}")));
                    }
                    outer_super_bridges.push(closure);
                }
            }
        }
    }
    if [root, child.as_ref()].iter().any(|physical| {
        physical
            .declaration
            .as_ref()
            .is_some_and(|declaration| !declaration.annotation_refusals.is_empty())
            || physical.fields.iter().any(|field| {
                field.declaration.is_none()
                    || !field.markers.is_empty()
                    || !field.annotations.refusals.is_empty()
                    || !field.type_annotations.refusals.is_empty()
            })
            || physical.methods.iter().any(|method| {
                method.declaration.is_none()
                    || (!method.markers.is_empty()
                        && !(method.has_only_explanation_marker()
                            && (sites.iter().any(|site| site.caller == method.item.identity)
                                || proof
                                    .reads
                                    .iter()
                                    .any(|read| read.method == method.item.identity)
                                || outer_super_bridges
                                    .iter()
                                    .any(|closed| closed.bridge.bridge == method.item.identity))))
                    || !method.annotations.refusals.is_empty()
                    || !method.parameter_annotations.refusals.is_empty()
                    || !method.type_annotations.refusals.is_empty()
            })
    }) {
        return Ok(Err(
            "family has an unspelled member or unsupported declaration annotation".to_owned(),
        ));
    }
    if child
        .fields
        .iter()
        .any(|field| field.item.access_flags & 0x0008 != 0)
        || child
            .methods
            .iter()
            .any(|method| method.item.access_flags & 0x0008 != 0)
    {
        return Ok(Err(
            "Java 8 member declaration contains an unsupported static member".to_owned(),
        ));
    }
    let target = jarde_java::report::ProvedMemberInnerTarget {
        definition: child.class.clone(),
        owner: child_name.to_owned(),
        outer: root_name.to_owned(),
        simple_name: relation.simple_name.clone(),
        constructor_descriptor: descriptor.to_owned(),
        capture_field: proof.field_name.clone(),
        generic_diamond: false,
        source_type_path: vec![
            source_type_path_segment(
                &root.class,
                root_name,
                root_source_name.clone(),
                0,
                None,
                true,
            ),
            source_type_path_segment(
                &child.class,
                child_name,
                format!("{root_source_name}.{}", relation.simple_name),
                0,
                Some(root_name.to_owned()),
                false,
            ),
        ],
    };
    let mut root_methods = Vec::new();
    let mut child_methods = Vec::new();
    for physical in [root, child.as_ref()] {
        for method in &physical.methods {
            budget.poll()?;
            if physical.class == root.class
                && outer_super_bridges
                    .iter()
                    .any(|closed| closed.bridge.bridge == method.item.identity)
            {
                // Its complete bytecode and every use were proved before this loop. The physical
                // report remains in `root.methods`; only the family source writer omits its text.
                continue;
            }
            let class_source::ClassSourceOutcome::Recovered { report, analysis } = &method.outcome
            else {
                return Ok(Err(format!(
                    "family member {} has no complete physical body",
                    method.item.index
                )));
            };
            let call_sites: Vec<_> = sites
                .iter()
                .filter(|site| site.caller == method.item.identity)
                .collect();
            let capture_reads: Vec<_> = proof
                .reads
                .iter()
                .filter(|read| read.method == method.item.identity)
                .collect();
            let outer_super_sites: Vec<_> = outer_super_bridges
                .iter()
                .flat_map(|closed| &closed.calls)
                .filter(|site| site.caller == method.item.identity)
                .collect();
            if outer_super_sites.iter().any(|site| {
                !matches!(
                    capture_reads.iter().find(|read| read.bci == site.capture_read_bci),
                    Some(read) if read.consumer_bcis == [site.call_bci]
                )
            }) {
                return Ok(Err(
                    "Outer.super capture read has another consumer whose source span is unproved"
                        .to_owned(),
                ));
            }
            if !matches!(analysis.execution, ExecutionReport::Complete { .. })
                || !matches!(report.execution, ExecutionReport::Complete { .. })
                || !report.produced()
                || ((call_sites.is_empty()
                    && capture_reads.is_empty()
                    && outer_super_sites.is_empty())
                    && (report.content != RecoveryContent::ContainsStatements
                        || report.regions.iter().any(|region| !region.structured)
                        || !report.fallbacks.is_empty()))
            {
                return Ok(Err(format!(
                    "family member {} retains fallback or incomplete recovery",
                    method.item.index
                )));
            }
            if method.item.identity == proof.constructor {
                let Some(text) =
                    class_source::member_family_constructor_text(method, &relation.simple_name)
                else {
                    return Ok(Err(
                        "capture constructor has unsupported source signature or annotations"
                            .to_owned(),
                    ));
                };
                let Some(header_end) = text.rfind(" {\n") else {
                    return Ok(Err(
                        "capture constructor header cannot be located".to_owned()
                    ));
                };
                let header_start = text[..header_end].rfind('\n').map_or(0, |at| at + 1);
                if !text[header_start..header_end].contains(&relation.simple_name) {
                    return Ok(Err(
                        "capture constructor header does not match member".to_owned()
                    ));
                }
                let header_end = header_end + 2;
                child_methods.push(class_source::MemberFamilyMethodText {
                    index: method.item.index,
                    text,
                    derived: vec![
                        class_source::MemberFamilyDerivedProjection {
                            kind: class_source::MemberFamilyDerivedKind::HiddenConstructorParameter,
                            start: header_start,
                            end: header_end,
                            anchors: vec![
                                class_source::MemberFamilyPhysicalAnchor::ConstructorParameter {
                                    method: proof.constructor.clone(),
                                    index: 0,
                                },
                            ],
                        },
                        class_source::MemberFamilyDerivedProjection {
                            kind: class_source::MemberFamilyDerivedKind::HiddenCaptureWrite,
                            start: header_start,
                            end: header_end,
                            anchors: vec![class_source::MemberFamilyPhysicalAnchor::MethodPoint {
                                method: proof.constructor.clone(),
                                bci: proof.write_bci,
                            }],
                        },
                    ],
                });
                continue;
            }
            if call_sites.is_empty() && capture_reads.is_empty() && outer_super_sites.is_empty() {
                continue;
            }
            let analyzed = jarde_jvm::analyze_method_ir(
                content,
                &crate::ir::MethodAnalysisRequest {
                    environment: environment.clone(),
                    method: method.item.identity.clone(),
                    stages: MethodOperation::Analysis.stages().to_vec(),
                },
                budget,
            )?;
            merge_execution(execution, analyzed.report().execution.clone());
            if analyzed.report().method != method.item.identity
                || !matches!(
                    analyzed.report().execution,
                    ExecutionReport::Complete { .. }
                )
                || analyzed.ir().code().is_none()
            {
                return Ok(Err(
                    "projected family method analysis is incomplete".to_owned()
                ));
            }
            let facts = recovery_facts(
                analyzed.ir().declaration(),
                analyzed.ir().code(),
                &method.item.identity,
            );
            let captured: Vec<_> = capture_reads
                .iter()
                .map(|read| jarde_java::report::ProvedCapturedOuterRead {
                    method: method.item.identity.clone(),
                    read_bci: read.bci,
                    field_owner: child_name.to_owned(),
                    field_name: proof.field_name.clone(),
                    field_descriptor: format!("L{root_name};"),
                    outer_internal_name: root_name.to_owned(),
                    outer_source_name: root_source_name.clone(),
                    constructor: proof.constructor.clone(),
                    constructor_write_bci: proof.write_bci,
                })
                .collect();
            let mut proved_super_calls = Vec::new();
            for site in &outer_super_sites {
                let (Ok(owner), Ok(name), Ok(descriptor)) = (
                    std::str::from_utf8(&site.bridge.target_owner.0),
                    std::str::from_utf8(&site.bridge.target_name.0),
                    std::str::from_utf8(&site.bridge.target_descriptor.0),
                ) else {
                    return Ok(Err(
                        "Outer.super target has no exact Java spelling".to_owned()
                    ));
                };
                proved_super_calls.push(jarde_java::ProvedOuterSuperCall {
                    caller: site.caller.clone(),
                    call_bci: site.call_bci,
                    capture_read_bci: site.capture_read_bci,
                    argument_bcis: site.argument_bcis.clone(),
                    bridge: site.bridge.bridge.clone(),
                    bridge_invoke_bci: site.bridge.invoke_bci,
                    outer_source_name: root_source_name.clone(),
                    target_owner: owner.to_owned(),
                    target_name: name.to_owned(),
                    target_descriptor: descriptor.to_owned(),
                });
            }
            let recovery = jarde_java::recover(
                &jarde_java::RecoveryRequest::new(
                    analyzed.ir(),
                    &facts,
                    environment.runtime.profile.clone(),
                )
                .with_member_inner_targets(std::slice::from_ref(&target))
                .with_captured_outer_reads(&captured)
                .with_outer_super_calls(&proved_super_calls)
                .with_evidence(
                    RecoveryEvidenceRequest::essential()
                        .with_kind(RecoveryEvidenceKind::RuleDetails)
                        .with_kind(RecoveryEvidenceKind::SourceMap),
                ),
                budget,
            );
            merge_execution(execution, recovery.execution.clone());
            if !matches!(recovery.execution, ExecutionReport::Complete { .. })
                || !recovery.produced()
                || recovery.content != RecoveryContent::ContainsStatements
                || recovery.regions.iter().any(|region| !region.structured)
                || !recovery.fallbacks.is_empty()
                || call_sites.iter().any(|site| {
                    !recovery
                        .news
                        .iter()
                        .any(|record| record.head == site.allocation_bci && record.presented)
                })
            {
                return Ok(Err(format!(
                    "projected family method {} retains fallback or unproved new@1",
                    method.item.index
                )));
            }
            let Some(text) = class_source::member_family_recovered_method_text(method, &recovery)
            else {
                return Ok(Err(
                    "projected family method artifact cannot be placed".to_owned()
                ));
            };
            let mut derived = Vec::new();
            for site in call_sites {
                let needle = format!("new {}", relation.simple_name);
                let Some((start, end)) = family_recovery_token_span(
                    method,
                    &recovery,
                    site.allocation_bci,
                    site.constructor_bci,
                    &needle,
                    budget,
                )?
                else {
                    return Ok(Err(format!(
                        "family new@1 source span is absent for method {} BCI {}",
                        method.item.index, site.allocation_bci
                    )));
                };
                derived.push(class_source::MemberFamilyDerivedProjection {
                    kind: class_source::MemberFamilyDerivedKind::MemberConstruction,
                    start,
                    end,
                    anchors: vec![
                        class_source::MemberFamilyPhysicalAnchor::MethodPoint {
                            method: site.caller.clone(),
                            bci: site.allocation_bci,
                        },
                        class_source::MemberFamilyPhysicalAnchor::MethodPoint {
                            method: site.caller.clone(),
                            bci: site.constructor_bci,
                        },
                        class_source::MemberFamilyPhysicalAnchor::ConstructorParameter {
                            method: site.constructor.clone(),
                            index: 0,
                        },
                    ],
                });
            }
            for site in &outer_super_sites {
                let Ok(name) = std::str::from_utf8(&site.bridge.target_name.0) else {
                    return Ok(Err(
                        "Outer.super target name has no Java spelling".to_owned()
                    ));
                };
                let needle = format!("{root_source_name}.super.{name}");
                let Some((start, end)) = family_recovery_token_span(
                    method,
                    &recovery,
                    site.call_bci,
                    site.call_bci,
                    &needle,
                    budget,
                )?
                else {
                    return Ok(Err(format!(
                        "Outer.super source span is absent for method {} BCI {}",
                        method.item.index, site.call_bci
                    )));
                };
                let capture_field = child
                    .fields
                    .iter()
                    .find(|field| field.item.index == proof.field_index)
                    .expect("validated capture field");
                derived.push(class_source::MemberFamilyDerivedProjection {
                    kind: class_source::MemberFamilyDerivedKind::OuterSuperCall,
                    start,
                    end,
                    anchors: vec![
                        class_source::MemberFamilyPhysicalAnchor::MethodPoint {
                            method: site.caller.clone(),
                            bci: site.call_bci,
                        },
                        class_source::MemberFamilyPhysicalAnchor::MethodPoint {
                            method: site.caller.clone(),
                            bci: site.capture_read_bci,
                        },
                        class_source::MemberFamilyPhysicalAnchor::Field {
                            field: capture_field.item.identity.clone(),
                            index: capture_field.item.index,
                        },
                        class_source::MemberFamilyPhysicalAnchor::MethodPoint {
                            method: proof.constructor.clone(),
                            bci: proof.write_bci,
                        },
                        class_source::MemberFamilyPhysicalAnchor::MethodPoint {
                            method: site.bridge.bridge.clone(),
                            bci: site.bridge.invoke_bci,
                        },
                        class_source::MemberFamilyPhysicalAnchor::OuterSuperTarget {
                            method: site.bridge.target_method.clone(),
                            owner: site.bridge.target_owner.clone(),
                            name: site.bridge.target_name.clone(),
                            descriptor: site.bridge.target_descriptor.clone(),
                        },
                    ],
                });
            }
            for read in capture_reads {
                if outer_super_sites
                    .iter()
                    .any(|site| site.capture_read_bci == read.bci)
                {
                    continue;
                }
                let needle = format!("{root_source_name}.this");
                let Some((start, end)) = family_recovery_token_span(
                    method, &recovery, read.bci, read.bci, &needle, budget,
                )?
                else {
                    return Ok(Err(format!(
                        "captured outer source span is absent for method {} BCI {}",
                        method.item.index, read.bci
                    )));
                };
                let capture_field = child
                    .fields
                    .iter()
                    .find(|field| field.item.index == proof.field_index)
                    .expect("validated capture field");
                derived.push(class_source::MemberFamilyDerivedProjection {
                    kind: class_source::MemberFamilyDerivedKind::CapturedOuterRead,
                    start,
                    end,
                    anchors: vec![
                        class_source::MemberFamilyPhysicalAnchor::MethodPoint {
                            method: read.method.clone(),
                            bci: read.bci,
                        },
                        class_source::MemberFamilyPhysicalAnchor::Field {
                            field: capture_field.item.identity.clone(),
                            index: capture_field.item.index,
                        },
                        class_source::MemberFamilyPhysicalAnchor::MethodPoint {
                            method: proof.constructor.clone(),
                            bci: proof.write_bci,
                        },
                    ],
                });
            }
            let projected = class_source::MemberFamilyMethodText {
                index: method.item.index,
                text,
                derived,
            };
            if physical.class == root.class {
                root_methods.push(projected);
            } else {
                child_methods.push(projected);
            }
        }
    }
    let member = class_source::MemberFamilyTextProjection {
        relation,
        child,
        capture: proof,
        root_methods: &root_methods,
        child_methods: &child_methods,
        outer_super_bridges: &outer_super_bridges,
    };
    let Some((text, derived)) = class_source::member_family_source_text(root, &member) else {
        return Ok(Err(
            "physical class writer cannot re-emit the family without losing prior projections"
                .to_owned(),
        ));
    };
    budget.charge(CountedBudgetDimension::OutputBytes, text.len() as u64)?;
    Ok(Ok((text, derived)))
}

/// Hiding a capture field and constructor argument changes the source unit's public surface.
/// Only the exact bytecode points already certified by the capture and call proofs may consume
/// those declarations. The declaration-reference scanner supplies the bounded physical census;
/// its undecided and partial states are never evidence of absence.
#[allow(clippy::too_many_arguments)]
fn prove_member_family_external_use_closure(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    root: &ClassSourceReport,
    child: &ClassSourceReport,
    capture: &class_source::MemberCaptureProof,
    calls: &[class_source::MemberCallProof],
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<std::result::Result<(), String>> {
    use crate::resolver::DeclarationRefQuery;
    use jarde_query::query::XrefOperation;
    use jarde_reader::model::SymbolRef;

    // A declaration scan covers one selected scope. An explicit classpath or additional
    // snapshot can make another physical consumer visible without being in that scan.
    if content.len() != 1
        || environment.domains.len() != 1
        || !environment.providers.is_empty()
        || environment.runtime.load_domain.roots.len() != 1
        || environment.runtime.load_domain.roots[0]
            != (LoadRoot::Container {
                origin: ContainerOrigin {
                    snapshot: environment.runtime.physical.snapshot.clone(),
                    root_container: ContainerId(ROOT_CONTAINER.to_owned()),
                    steps: Vec::new(),
                },
                prefix: ArchiveNameBytes(Vec::new()),
            })
        || !matches!(
            environment.runtime.profile.multi_release,
            crate::MultiReleasePolicy::Disabled
        )
        || root.class.snapshot() != &environment.runtime.physical.snapshot
        || child.class.snapshot() != &environment.runtime.physical.snapshot
    {
        return Ok(Err(
            "external-use closure cannot prove every visible classpath, snapshot or multi-release scope"
                .to_owned(),
        ));
    }
    let Some(field) = child
        .fields
        .iter()
        .find(|field| field.item.index == capture.field_index)
    else {
        return Ok(Err("capture field has no physical declaration".to_owned()));
    };
    let Some(root_binary) = root
        .declaration
        .as_ref()
        .map(|declaration| &declaration.item.declaration.this_class.raw().0)
    else {
        return Ok(Err("root has no physical declaration".to_owned()));
    };
    let Some(child_binary) = child
        .declaration
        .as_ref()
        .map(|declaration| &declaration.item.declaration.this_class.raw().0)
    else {
        return Ok(Err("member has no physical declaration".to_owned()));
    };
    let MemberKey::Field { name, descriptor } = &field.item.identity.member else {
        return Ok(Err(
            "capture field has no physical field identity".to_owned()
        ));
    };
    if name.0 != capture.field_name.as_bytes()
        || descriptor.0 != [b"L".as_slice(), root_binary, b";"].concat()
        || field.item.identity.owner != child.class
    {
        return Ok(Err(
            "capture field identity does not match the proved family".to_owned(),
        ));
    }
    let field_query = DeclarationRefQuery {
        environment: environment.clone(),
        declaration: ResolvedMemberRef {
            loader: environment.runtime.load_domain.loader.clone(),
            definition: child.class.clone(),
            member: SymbolRef::Field {
                owner: JvmBytes(child_binary.clone()),
                name: name.clone(),
                descriptor: descriptor.clone(),
            },
        },
        scope: environment.runtime.physical.scope.clone(),
        consumers: ConsumerSchema::new(
            1,
            [
                ConsumerKind::Field,
                ConsumerKind::Constant,
                ConsumerKind::Bootstrap,
            ],
        ),
        max_items: 0,
    };
    let constructor_query = DeclarationRefQuery {
        environment: environment.clone(),
        declaration: ResolvedMemberRef {
            loader: environment.runtime.load_domain.loader.clone(),
            definition: child.class.clone(),
            member: SymbolRef::Method {
                owner: JvmBytes(child_binary.clone()),
                name: JvmBytes(b"<init>".to_vec()),
                descriptor: capture.constructor.descriptor.clone(),
            },
        },
        scope: environment.runtime.physical.scope.clone(),
        consumers: ConsumerSchema::new(
            1,
            [
                ConsumerKind::Invocation,
                ConsumerKind::Constant,
                ConsumerKind::Bootstrap,
            ],
        ),
        max_items: 0,
    };
    for (kind, query) in [
        ("capture field", field_query),
        ("member constructor", constructor_query),
    ] {
        budget.poll()?;
        let scanned = jarde_jvm::declaration_references(content, &query, budget)?;
        merge_execution(execution, scanned.execution.clone());
        if scanned.analysis != ResolutionAnalysis::Performed
            || !scanned.environment_problems.is_empty()
            || !scanned.unsupported_categories.is_empty()
            || scanned.unresolved_candidates != 0
            || scanned.has_more
            || !matches!(scanned.execution, ExecutionReport::Complete { .. })
            || scanned.coverage.artifact_structural.state != CoverageState::CompleteWithinSchema
            || scanned.coverage.runtime_resolution.state != CoverageState::CompleteWithinSchema
        {
            return Ok(Err(format!(
                "{kind} external-use closure is incomplete in selected physical scope"
            )));
        }
        for item in &scanned.items {
            let allowed = match (
                kind,
                item.consumer,
                item.operation,
                item.origin.members.as_slice(),
            ) {
                (
                    "capture field",
                    ConsumerKind::Field,
                    XrefOperation::PutField,
                    [OriginMember::MethodPoint { method, bci }],
                ) => method == &capture.constructor && *bci == capture.write_bci,
                (
                    "capture field",
                    ConsumerKind::Field,
                    XrefOperation::GetField,
                    [OriginMember::MethodPoint { method, bci }],
                ) => capture
                    .reads
                    .iter()
                    .any(|read| &read.method == method && read.bci == *bci),
                (
                    "member constructor",
                    ConsumerKind::Invocation,
                    XrefOperation::InvokeSpecial,
                    [OriginMember::MethodPoint { method, bci }],
                ) => calls.iter().any(|call| {
                    &call.caller == method
                        && call.constructor_bci == *bci
                        && call.constructor == capture.constructor
                }),
                _ => false,
            };
            if !allowed {
                let origin = item.origin.members.first().map_or_else(
                    || "unknown physical origin".to_owned(),
                    |member| {
                        let definition = match member {
                            OriginMember::MethodPoint { method, .. } => &method.owner,
                            OriginMember::ClassRange { definition, .. }
                            | OriginMember::ClassFile { definition } => definition,
                        };
                        let entry = definition.entry().map_or_else(
                            || "standalone class".to_owned(),
                            |entry| String::from_utf8_lossy(&entry.raw_name.0).into_owned(),
                        );
                        format!("{entry} {member:?}")
                    },
                );
                return Ok(Err(format!(
                    "{kind} has an unproved physical consumer at {origin}"
                )));
            }
        }
    }
    Ok(Ok(()))
}

/// Census the exact selected bridge definition. No single successful member call can authorize
/// deleting the helper: every physical invocation must yield its own capture/argument proof, and
/// a handle, bootstrap, unresolved use or partial scan rejects the whole certificate.
#[allow(clippy::too_many_arguments)]
fn prove_outer_super_bridge_use_closure(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    root: &ClassSourceReport,
    child: &ClassSourceReport,
    capture: &class_source::MemberCaptureProof,
    bridge: &class_source::OuterSuperBridgeProof,
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<std::result::Result<class_source::OuterSuperBridgeClosureProof, String>> {
    use crate::resolver::DeclarationRefQuery;
    use jarde_reader::model::SymbolRef;

    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    budget.poll()?;
    if content.len() != 1
        || environment.domains.len() != 1
        || !environment.providers.is_empty()
        || environment.runtime.load_domain.roots.len() != 1
        || environment.runtime.load_domain.roots[0]
            != (LoadRoot::Container {
                origin: ContainerOrigin {
                    snapshot: environment.runtime.physical.snapshot.clone(),
                    root_container: ContainerId(ROOT_CONTAINER.to_owned()),
                    steps: Vec::new(),
                },
                prefix: ArchiveNameBytes(Vec::new()),
            })
        || !matches!(
            environment.runtime.profile.multi_release,
            crate::MultiReleasePolicy::Disabled
        )
        || bridge.bridge.owner != root.class
        || child.class != capture.constructor.owner
        || root.class.snapshot() != &environment.runtime.physical.snapshot
        || child.class.snapshot() != &environment.runtime.physical.snapshot
    {
        return refuse(
            "bridge use closure cannot prove the selected input and visible dependency scope",
        );
    }
    let query = DeclarationRefQuery {
        environment: environment.clone(),
        declaration: ResolvedMemberRef {
            loader: environment.runtime.load_domain.loader.clone(),
            definition: root.class.clone(),
            member: SymbolRef::Method {
                owner: bridge.outer_name.clone(),
                name: bridge.bridge.name.clone(),
                descriptor: bridge.bridge.descriptor.clone(),
            },
        },
        scope: environment.runtime.physical.scope.clone(),
        consumers: ConsumerSchema::new(
            1,
            [
                ConsumerKind::Invocation,
                ConsumerKind::Constant,
                ConsumerKind::Bootstrap,
            ],
        ),
        max_items: 0,
    };
    let scanned = jarde_jvm::declaration_references(content, &query, budget)?;
    merge_execution(execution, scanned.execution.clone());
    if scanned.analysis != ResolutionAnalysis::Performed
        || !scanned.environment_problems.is_empty()
        || !scanned.unsupported_categories.is_empty()
        || scanned.unresolved_candidates != 0
        || scanned.has_more
        || !matches!(scanned.execution, ExecutionReport::Complete { .. })
        || scanned.coverage.artifact_structural.state != CoverageState::CompleteWithinSchema
        || scanned.coverage.runtime_resolution.state != CoverageState::CompleteWithinSchema
    {
        return refuse("bridge reference census is incomplete in selected physical scope");
    }
    let mut calls = Vec::new();
    for item in &scanned.items {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let Some((method, bci)) = direct_member_bridge_invocation(
            item.consumer,
            item.operation,
            &item.origin.members,
            child,
        ) else {
            return refuse("bridge has a non-member, handle, bootstrap or non-static consumer");
        };
        let analyzed = jarde_jvm::analyze_method_ir(
            content,
            &crate::ir::MethodAnalysisRequest {
                environment: environment.clone(),
                method: method.clone(),
                stages: MethodOperation::Analysis.stages().to_vec(),
            },
            budget,
        )?;
        merge_execution(execution, analyzed.report().execution.clone());
        if analyzed.report().method != method
            || !matches!(
                analyzed.report().execution,
                ExecutionReport::Complete { .. }
            )
        {
            return refuse("bridge caller analysis is incomplete");
        }
        let call = match crate::member_inner::prove_outer_super_call(
            &method,
            analyzed.ir(),
            capture,
            bridge,
            bci,
            budget,
        )? {
            Ok(call) => call,
            Err(reason) => return Ok(Err(reason)),
        };
        if calls
            .iter()
            .any(|prior: &class_source::OuterSuperCallProof| {
                prior.caller == call.caller && prior.call_bci == call.call_bci
            })
        {
            return refuse("bridge census repeated one physical call point");
        }
        calls.push(call);
    }
    if calls.is_empty() {
        return refuse("bridge has no proved member call site");
    }
    Ok(Ok(class_source::OuterSuperBridgeClosureProof {
        bridge: bridge.clone(),
        calls,
    }))
}

/// Prove that Java 8's `Outer.super.name(args)` has only the exact direct-parent
/// declaration named by the bridge's MethodRef in the selected source hierarchy.
/// The JVM target proof alone does not settle source overload or generic binding.
#[allow(clippy::too_many_arguments)]
fn prove_outer_super_source_binding(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    outer: &ClassMemberFacts,
    parent_definition: &PhysicalDefinitionId,
    parent: &ClassMemberFacts,
    bridge: &class_source::OuterSuperBridgeProof,
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<std::result::Result<(), String>> {
    let refuse = |reason: &str| Ok(Err(reason.to_owned()));
    if bridge.target_method.owner != *parent_definition
        || bridge.target_owner.0 != parent.this_class.raw().0
        || bridge.target_method.name != bridge.target_name
        || bridge.target_method.descriptor != bridge.target_descriptor
        || outer
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0 == b"Signature")
    {
        return refuse("bridge target or Outer generic declaration cannot bind as Java source");
    }
    let Some(source_name) = std::str::from_utf8(&bridge.target_name.0).ok() else {
        return refuse("bridge target method name has no Java source spelling");
    };
    let Some(source_owner) = std::str::from_utf8(&bridge.target_owner.0).ok() else {
        return refuse("bridge target owner has no Java source spelling");
    };
    if !jarde_java::is_java_identifier(source_name)
        || !source_owner.split('/').all(jarde_java::is_java_identifier)
        || std::str::from_utf8(&bridge.target_descriptor.0).is_err()
    {
        return refuse("bridge target has no exact Java source spelling");
    }
    let mut pending = vec![(parent.this_class.raw().0.clone(), false)];
    pending.extend(
        outer
            .interfaces
            .iter()
            .map(|interface| (interface.raw().0.clone(), true)),
    );
    let mut visited = std::collections::BTreeSet::new();
    let mut exact_target = 0_u32;
    while let Some((name, interface)) = pending.pop() {
        budget.poll()?;
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if name == b"java/lang/Object" {
            const OBJECT_METHODS: &[&[u8]] = &[
                b"getClass",
                b"hashCode",
                b"equals",
                b"clone",
                b"toString",
                b"notify",
                b"notifyAll",
                b"wait",
                b"finalize",
            ];
            if OBJECT_METHODS.contains(&bridge.target_name.0.as_slice()) {
                return refuse(
                    "Object method name prevents proving a source binding without the selected JDK declaration",
                );
            }
            continue;
        }
        if !visited.insert(name.clone()) {
            continue;
        }
        let (definition, facts) = if name == parent.this_class.raw().0 && !interface {
            (parent_definition.clone(), parent.clone())
        } else {
            let Some((definition, read)) = resolve_class_source_dependency_read_raw(
                content,
                environment,
                None,
                &name,
                execution,
                budget,
            )?
            else {
                return refuse(
                    "selected superclass or interface declaration is unavailable for Java source binding",
                );
            };
            (definition, read.facts)
        };
        if facts.stopped_at.is_some()
            || facts.method_count != facts.methods.len() as u64
            || facts.field_count != facts.fields.len() as u64
            || facts.this_class.raw().0 != name
            || (facts.access_flags & ACC_INTERFACE != 0) != interface
            || facts
                .attributes
                .iter()
                .any(|attribute| attribute.name.raw().0 == b"Signature")
        {
            return refuse("source hierarchy declaration is incomplete or generic");
        }
        for method in &facts.methods {
            budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
            if method.name.raw().0 != bridge.target_name.0 {
                continue;
            }
            let is_target = definition == bridge.target_method.owner
                && method.descriptor.raw().0 == bridge.target_descriptor.0;
            if !is_target {
                return refuse("source hierarchy contains a competing same-name method");
            }
            exact_target += 1;
            if exact_target != 1
                || method.attributes.iter().any(|attribute| {
                    matches!(
                        attribute.name.raw().0.as_slice(),
                        b"Signature" | b"Exceptions"
                    )
                })
            {
                return refuse("target method generic or checked-exception surface is unproved");
            }
        }
        pending.extend(
            facts
                .interfaces
                .iter()
                .map(|interface| (interface.raw().0.clone(), true)),
        );
        if !interface {
            let Some(super_class) = facts.super_class.as_ref() else {
                return refuse("source superclass chain ends before java/lang/Object");
            };
            pending.push((super_class.raw().0.clone(), false));
        }
    }
    if exact_target != 1 {
        return refuse("direct parent target has no unique source declaration");
    }
    Ok(Ok(()))
}

fn direct_member_bridge_invocation(
    consumer: ConsumerKind,
    operation: jarde_query::query::XrefOperation,
    origin: &[OriginMember],
    child: &ClassSourceReport,
) -> Option<(PhysicalMethodId, u32)> {
    let [OriginMember::MethodPoint { method, bci }] = origin else {
        return None;
    };
    (consumer == ConsumerKind::Invocation
        && operation == jarde_query::query::XrefOperation::InvokeStatic
        && method.owner == child.class
        && child
            .methods
            .iter()
            .any(|member| member.item.identity == *method))
    .then(|| (method.clone(), *bci))
}

#[cfg(test)]
mod outer_super_bridge_closure_tests {
    use super::*;
    use jarde_reader::budget::{CancellationToken, Limits};
    use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
    use std::io::{Cursor, Write};

    const FIXTURE: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-26/named-member-outer-receiver/variants/fixture.jar"
    );
    const EFFECTS_BASE: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/EffectsBase.class"
    );
    const EFFECTS_OUTER: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/OuterSuperEffects.class"
    );
    const EFFECTS_MEMBER: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-27/outer-super-bridge-effects/OuterSuperEffects$Member.class"
    );

    fn effects_jar() -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        {
            let mut archive = ZipArchiveWriter::new(&mut output);
            for (name, bytes) in [
                (b"EffectsBase.class".as_slice(), EFFECTS_BASE),
                (b"OuterSuperEffects.class".as_slice(), EFFECTS_OUTER),
                (b"OuterSuperEffects$Member.class".as_slice(), EFFECTS_MEMBER),
            ] {
                let (mut entry, config) = archive
                    .new_file(EntryPath::verbatim(name.to_vec()))
                    .compression_method(CompressionMethod::new(0))
                    .start()
                    .unwrap();
                let mut writer = config.wrap(&mut entry);
                writer.write_all(bytes).unwrap();
                let (_, descriptor) = writer.finish().unwrap();
                entry.finish(descriptor).unwrap();
            }
            archive.finish().unwrap();
        }
        output.into_inner()
    }

    fn budget() -> Budget {
        Budget::new(Limits {
            input_bytes: u64::MAX,
            archive_entries: u64::MAX,
            entry_bytes: u64::MAX,
            read_bytes: u64::MAX,
            class_bytes: u64::MAX,
            attribute_bytes: u64::MAX,
            code_bytes: u64::MAX,
            result_items: u64::MAX,
            output_bytes: u64::MAX,
            class_headers: u64::MAX,
            method_bodies: u64::MAX,
            ir_items: u64::MAX,
            ir_edges: u64::MAX,
            analysis_steps: u64::MAX,
            normalization_clones: u64::MAX,
            nested_depth: u64::MAX,
            dependency_depth: u64::MAX,
            elapsed_millis: u64::MAX,
        })
    }

    fn family_report_for(jar: Vec<u8>, class: &str, all: bool) -> ClassSourceReport {
        let engine = Engine::new();
        let mut budget = budget();
        let snapshot = engine.open(ArtifactInput::bytes(jar), &mut budget).unwrap();
        let request = ClassSourceRequest {
            class: ClassRef::Name {
                class: ClassNameQuery::internal(class),
            },
            environment: EnvironmentRequest {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
                policy: EnvironmentPolicy::PlainJar,
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: crate::MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                loader: LoaderId("app".to_owned()),
            },
        };
        let evidence = if all {
            RecoveryEvidenceRequest::all()
        } else {
            RecoveryEvidenceRequest::essential()
        };
        match engine
            .class_source_with_evidence(&[snapshot], &request, &evidence, &mut budget)
            .unwrap()
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("Outer must resolve: {other:?}"),
        }
    }

    fn family_report(all: bool) -> ClassSourceReport {
        family_report_for(FIXTURE.to_vec(), "OuterReceiverCases", all)
    }

    #[test]
    fn closed_bridge_projects_exact_outer_super_and_keeps_both_physical_methods() {
        let default = family_report(false);
        let all = family_report(true);
        assert_eq!(default.text, all.text);
        let class_source::ClassSourceMemberFamily::Prepared {
            projection: default_projection,
            ..
        } = &default.member_family
        else {
            panic!("default member relation must prepare");
        };
        let class_source::ClassSourceMemberFamily::Prepared {
            projection: all_projection,
            ..
        } = &all.member_family
        else {
            panic!("all-evidence member relation must prepare");
        };
        assert_eq!(default_projection, all_projection);
        let class_source::ClassSourceMemberFamily::Prepared {
            child, projection, ..
        } = &all.member_family
        else {
            panic!("frozen member relation must prepare");
        };
        let class_source::ClassSourceMemberProjection::Projected { derived } = projection else {
            panic!("frozen bridge family must project: {projection:?}");
        };
        assert!(
            all.text.contains("OuterReceiverCases.super.value()"),
            "{}",
            all.text
        );
        assert!(!all.text.contains("access$101("), "{}", all.text);
        let bridge = all
            .methods
            .iter()
            .find(|method| method.item.identity.name.0 == b"access$101")
            .expect("physical bridge report remains");
        let caller = child
            .methods
            .iter()
            .find(|method| method.item.identity.name.0 == b"compare")
            .expect("physical Member caller remains");
        let projected = derived
            .iter()
            .find(|item| item.kind == class_source::MemberFamilyDerivedKind::OuterSuperCall)
            .expect("derived call exists");
        assert_eq!(
            &all.text[projected.start..projected.end],
            "OuterReceiverCases.super.value"
        );
        assert!(projected.anchors.iter().any(|anchor| matches!(anchor,
            class_source::MemberFamilyPhysicalAnchor::MethodPoint { method, bci }
                if method == &caller.item.identity && *bci == 38)));
        assert!(projected.anchors.iter().any(|anchor| matches!(anchor,
            class_source::MemberFamilyPhysicalAnchor::MethodPoint { method, bci }
                if method == &bridge.item.identity && *bci == 1)));
        assert!(projected.anchors.iter().any(|anchor| matches!(anchor,
            class_source::MemberFamilyPhysicalAnchor::OuterSuperTarget { method, owner, name, descriptor }
                if method.owner != bridge.item.identity.owner
                    && method.name == *name
                    && method.descriptor == *descriptor
                    && owner.0 == b"ReceiverBase")));
    }

    #[test]
    fn effectful_outer_super_arguments_project_in_evaluation_order() {
        let default = family_report_for(effects_jar(), "OuterSuperEffects", false);
        let all = family_report_for(effects_jar(), "OuterSuperEffects", true);
        assert_eq!(default.text, all.text);
        let class_source::ClassSourceMemberFamily::Prepared {
            projection: default_projection,
            ..
        } = &default.member_family
        else {
            panic!("default member relation must prepare");
        };
        let class_source::ClassSourceMemberFamily::Prepared {
            projection: all_projection,
            ..
        } = &all.member_family
        else {
            panic!("all-evidence member relation must prepare");
        };
        assert_eq!(default_projection, all_projection);
        let class_source::ClassSourceMemberFamily::Prepared {
            child, projection, ..
        } = &all.member_family
        else {
            panic!("effectful member relation must prepare");
        };
        let class_source::ClassSourceMemberProjection::Projected { derived } = projection else {
            panic!("effectful bridge family must project: {projection:?}");
        };
        assert!(
            all.text.contains("OuterSuperEffects.super.combine("),
            "{}",
            all.text
        );
        assert!(all.text.contains("tick(1, failFirst)"), "{}", all.text);
        assert!(all.text.contains("tick(2, false)"), "{}", all.text);
        assert_eq!(all.text.matches("tick(1, failFirst)").count(), 1);
        assert_eq!(all.text.matches("tick(2, false)").count(), 1);
        assert!(
            all.text.find("tick(1, failFirst)").unwrap() < all.text.find("tick(2, false)").unwrap()
        );
        assert!(!all.text.contains("access$001("), "{}", all.text);
        let bridge = all
            .methods
            .iter()
            .find(|method| method.item.identity.name.0 == b"access$001")
            .expect("physical bridge report remains");
        let caller = child
            .methods
            .iter()
            .find(|method| method.item.identity.name.0 == b"run")
            .expect("physical Member caller remains");
        let call = derived
            .iter()
            .find(|item| item.kind == class_source::MemberFamilyDerivedKind::OuterSuperCall)
            .expect("derived call exists");
        assert_eq!(
            &all.text[call.start..call.end],
            "OuterSuperEffects.super.combine"
        );
        assert!(call.anchors.iter().any(|anchor| matches!(anchor,
            class_source::MemberFamilyPhysicalAnchor::MethodPoint { method, bci }
                if method == &caller.item.identity && *bci == 14)));
        assert!(call.anchors.iter().any(|anchor| matches!(anchor,
            class_source::MemberFamilyPhysicalAnchor::MethodPoint { method, bci }
                if method == &bridge.item.identity && *bci == 3)));
        assert!(call.anchors.iter().any(|anchor| matches!(anchor,
            class_source::MemberFamilyPhysicalAnchor::OuterSuperTarget { method, owner, name, descriptor }
                if method.name == *name && method.descriptor == *descriptor
                    && owner.0 == b"EffectsBase" && name.0 == b"combine"
                    && descriptor.0 == b"(II)I")));
        assert_eq!(
            derived
                .iter()
                .filter(|item| item.kind
                    == class_source::MemberFamilyDerivedKind::HiddenOuterSuperBridge)
                .count(),
            1
        );
    }

    #[test]
    fn exact_bridge_closure_requires_every_static_call_and_a_complete_scan() {
        let engine = Engine::new();
        let mut budget = budget();
        let snapshot = engine
            .open(ArtifactInput::bytes(FIXTURE.to_vec()), &mut budget)
            .unwrap();
        let request_environment = EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: crate::MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        };
        let root = match engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("OuterReceiverCases"),
                    },
                    environment: request_environment.clone(),
                },
                &mut budget,
            )
            .unwrap()
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("Outer must resolve: {other:?}"),
        };
        let class_source::ClassSourceMemberFamily::Prepared { child, capture, .. } =
            &root.member_family
        else {
            panic!("frozen Outer member relation must prepare")
        };
        let class_source::ClassSourceMemberCapture::Proved { proof: capture } = capture else {
            panic!("frozen Member capture must be proved")
        };
        let environment = request_environment
            .build(std::slice::from_ref(&snapshot))
            .unwrap();
        let (outer_id, outer_read) = resolve_class_source_dependency_read_raw(
            std::slice::from_ref(&snapshot),
            &environment,
            None,
            b"OuterReceiverCases",
            &mut ExecutionReport::Complete {
                usage: budget.usage(),
            },
            &mut budget,
        )
        .unwrap()
        .unwrap();
        assert_eq!(outer_id, root.class);
        let (parent_definition, parent_read) = resolve_class_source_dependency_read_raw(
            std::slice::from_ref(&snapshot),
            &environment,
            None,
            b"ReceiverBase",
            &mut ExecutionReport::Complete {
                usage: budget.usage(),
            },
            &mut budget,
        )
        .unwrap()
        .unwrap();
        let bridge_id = root
            .methods
            .iter()
            .find(|method| method.item.identity.name.0 == b"access$101")
            .unwrap()
            .item
            .identity
            .clone();
        let analyzed = jarde_jvm::analyze_method_ir(
            std::slice::from_ref(&snapshot),
            &crate::ir::MethodAnalysisRequest {
                environment: environment.clone(),
                method: bridge_id.clone(),
                stages: MethodOperation::Analysis.stages().to_vec(),
            },
            &mut budget,
        )
        .unwrap();
        let bridge = crate::member_inner::prove_outer_super_bridge(
            &outer_read.facts,
            &parent_read.facts,
            &parent_definition,
            &bridge_id,
            analyzed.ir(),
            &mut budget,
        )
        .unwrap()
        .unwrap();
        let mut execution = ExecutionReport::Complete {
            usage: budget.usage(),
        };
        let closure = prove_outer_super_bridge_use_closure(
            std::slice::from_ref(&snapshot),
            &environment,
            &root,
            child,
            capture,
            &bridge,
            &mut execution,
            &mut budget,
        )
        .unwrap()
        .unwrap();
        assert_eq!(closure.calls.len(), 1);
        assert_eq!(closure.calls[0].call_bci, 38);
        assert_eq!(closure.calls[0].capture_read_bci, 35);
        assert_eq!(closure.calls[0].bridge, bridge);
        let class_source::ClassSourceMemberFamily::Prepared {
            projection: class_source::ClassSourceMemberProjection::Projected { derived },
            ..
        } = &root.member_family
        else {
            panic!("closed frozen family must project");
        };
        let call = derived
            .iter()
            .find(|item| item.kind == class_source::MemberFamilyDerivedKind::OuterSuperCall)
            .unwrap();
        assert!(call.anchors.iter().any(|anchor| matches!(anchor,
            class_source::MemberFamilyPhysicalAnchor::OuterSuperTarget { method, owner, name, descriptor }
                if method == &bridge.target_method
                    && owner == &bridge.target_owner
                    && name == &bridge.target_name
                    && descriptor == &bridge.target_descriptor)));
        assert!(
            prove_outer_super_source_binding(
                std::slice::from_ref(&snapshot),
                &environment,
                &outer_read.facts,
                &parent_definition,
                &parent_read.facts,
                &bridge,
                &mut execution,
                &mut budget,
            )
            .unwrap()
            .is_ok()
        );
        let mut overloaded_parent = parent_read.facts.clone();
        let mut overload = overloaded_parent
            .methods
            .iter()
            .find(|method| method.name.raw().0 == bridge.target_name.0)
            .unwrap()
            .clone();
        overload.descriptor = overloaded_parent
            .methods
            .iter()
            .find(|method| method.name.raw().0 == b"<init>")
            .unwrap()
            .descriptor
            .clone();
        overloaded_parent.methods.push(overload);
        overloaded_parent.method_count += 1;
        assert!(
            prove_outer_super_source_binding(
                std::slice::from_ref(&snapshot),
                &environment,
                &outer_read.facts,
                &parent_definition,
                &overloaded_parent,
                &bridge,
                &mut execution,
                &mut budget,
            )
            .unwrap()
            .is_err()
        );
        let mut wrong_target = bridge.clone();
        wrong_target.target_method.owner = root.class.clone();
        assert!(
            prove_outer_super_source_binding(
                std::slice::from_ref(&snapshot),
                &environment,
                &outer_read.facts,
                &parent_definition,
                &parent_read.facts,
                &wrong_target,
                &mut execution,
                &mut budget,
            )
            .unwrap()
            .is_err()
        );
        let direct_origin = [OriginMember::MethodPoint {
            method: closure.calls[0].caller.clone(),
            bci: 38,
        }];
        assert!(
            direct_member_bridge_invocation(
                ConsumerKind::Invocation,
                jarde_query::query::XrefOperation::InvokeStatic,
                &direct_origin,
                child,
            )
            .is_some()
        );
        for consumer in [ConsumerKind::Constant, ConsumerKind::Bootstrap] {
            assert!(
                direct_member_bridge_invocation(
                    consumer,
                    jarde_query::query::XrefOperation::InvokeStatic,
                    &direct_origin,
                    child,
                )
                .is_none()
            );
        }
        assert!(
            direct_member_bridge_invocation(
                ConsumerKind::Invocation,
                jarde_query::query::XrefOperation::InvokeStatic,
                &[OriginMember::MethodPoint {
                    method: bridge_id.clone(),
                    bci: 38
                }],
                child,
            )
            .is_none()
        );

        // The same physical scope contains `access$000(other)` as well as capture uses. An
        // accurate census of that helper cannot certify all its invocations as lexical Outer.
        let mut unproved_helper = bridge.clone();
        unproved_helper.bridge.name = JvmBytes(b"access$000".to_vec());
        unproved_helper.bridge.descriptor = JvmBytes(b"(LOuterReceiverCases;)I".to_vec());
        let mut execution = ExecutionReport::Complete {
            usage: budget.usage(),
        };
        assert!(
            prove_outer_super_bridge_use_closure(
                std::slice::from_ref(&snapshot),
                &environment,
                &root,
                child,
                capture,
                &unproved_helper,
                &mut execution,
                &mut budget,
            )
            .unwrap()
            .is_err()
        );

        let mut limits = self::budget().limits().clone();
        limits.analysis_steps = 0;
        let mut limited = Budget::new(limits);
        let mut execution = ExecutionReport::Complete {
            usage: limited.usage(),
        };
        let stopped = prove_outer_super_bridge_use_closure(
            std::slice::from_ref(&snapshot),
            &environment,
            &root,
            child,
            capture,
            &bridge,
            &mut execution,
            &mut limited,
        );
        assert!(matches!(
            stopped,
            Err(Error::BudgetExceeded { .. }) | Ok(Err(_))
        ));
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let mut cancelled =
            Budget::with_cancellation_token(self::budget().limits().clone(), cancellation);
        let mut execution = ExecutionReport::Complete {
            usage: cancelled.usage(),
        };
        assert!(matches!(
            prove_outer_super_bridge_use_closure(
                std::slice::from_ref(&snapshot),
                &environment,
                &root,
                child,
                capture,
                &bridge,
                &mut execution,
                &mut cancelled,
            ),
            Err(Error::Cancelled { .. })
        ));
    }
}

/// A candidate is bounded by an emitter segment already tied to this exact physical method and
/// BCI. Only a unique token inside the smallest such segment is admitted; the class writer then
/// verifies the byte-for-byte translated span before publishing it.
fn family_recovery_token_span(
    method: &ClassSourceMethod,
    recovery: &RecoveryReport,
    bci: u32,
    primary_bci: u32,
    needle: &str,
    budget: &mut Budget,
) -> Result<Option<(usize, usize)>> {
    let mut candidates = Vec::new();
    for segment in recovery.source_map.of_bci(bci) {
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        if segment.origin().primary().method() != Some(&method.item.identity)
            || segment.origin().primary().bci() != primary_bci
            || segment.mentions(bci)
                != Some(if bci == primary_bci {
                    jarde_java::Provenance::Direct
                } else {
                    jarde_java::Provenance::Derived
                })
        {
            continue;
        }
        let source = segment.text(&recovery.text);
        let mut matches = source.match_indices(needle);
        let Some((at, _)) = matches.next() else {
            continue;
        };
        if matches.next().is_some() {
            continue;
        }
        candidates.push((
            segment.len(),
            segment.start() + at,
            segment.start() + at + needle.len(),
        ));
    }
    candidates.sort_unstable();
    let Some((shortest, start, end)) = candidates.first().copied() else {
        return Ok(None);
    };
    if candidates
        .iter()
        .take_while(|candidate| candidate.0 == shortest)
        .any(|candidate| (candidate.1, candidate.2) != (start, end))
    {
        return Ok(None);
    }
    Ok(class_source::member_family_recovered_span(
        method, recovery, start, end,
    ))
}

fn prove_class_source_member_calls(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    root: &ClassSourceReport,
    child: &ClassSourceReport,
    root_name: &[u8],
    candidate: &crate::member_inner::FamilyRootCandidate,
    capture: &class_source::MemberCaptureProof,
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Result<class_source::ClassSourceMemberCalls> {
    use class_source::{ClassSourceMemberCalls as Calls, MemberCallRefusal};
    let Some(root_binary) = std::str::from_utf8(root_name).ok() else {
        return Ok(Calls::Refused {
            reason: "root binary name cannot be used for a Java source path".to_owned(),
            sites: Vec::new(),
            refusals: Vec::new(),
        });
    };
    let Some(child_binary) = std::str::from_utf8(&candidate.child_name).ok() else {
        return Ok(Calls::Refused {
            reason: "member binary name cannot be used for a Java source path".to_owned(),
            sites: Vec::new(),
            refusals: Vec::new(),
        });
    };
    if !root_binary
        .split('/')
        .all(jarde_java::names::is_java_identifier)
    {
        return Ok(Calls::Refused {
            reason: "root binary name has no proved Java source spelling".to_owned(),
            sites: Vec::new(),
            refusals: Vec::new(),
        });
    }
    if capture.constructor.owner != child.class {
        return Ok(Calls::Refused {
            reason: "capture constructor belongs to another physical definition".to_owned(),
            sites: Vec::new(),
            refusals: Vec::new(),
        });
    }
    let Some(constructor_descriptor) = std::str::from_utf8(&capture.constructor.descriptor.0).ok()
    else {
        return Ok(Calls::Refused {
            reason: "capture constructor descriptor has no exact UTF-8 rule target".to_owned(),
            sites: Vec::new(),
            refusals: Vec::new(),
        });
    };
    if [root, child].iter().any(|physical| {
        physical.declaration.as_ref().is_none_or(|declaration| {
            declaration.generic_signature.is_some() || declaration.generic_refusal.is_some()
        })
    }) {
        return Ok(Calls::Refused {
            reason: "family call source path requires non-generic root and member headers"
                .to_owned(),
            sites: Vec::new(),
            refusals: Vec::new(),
        });
    }
    let root_source = root_binary.replace('/', ".");
    let target = jarde_java::report::ProvedMemberInnerTarget {
        definition: child.class.clone(),
        owner: child_binary.to_owned(),
        outer: root_binary.to_owned(),
        simple_name: candidate.simple_name.clone(),
        constructor_descriptor: constructor_descriptor.to_owned(),
        capture_field: capture.field_name.clone(),
        generic_diamond: false,
        source_type_path: vec![
            source_type_path_segment(&root.class, root_binary, root_source.clone(), 0, None, true),
            source_type_path_segment(
                &child.class,
                child_binary,
                format!("{root_source}.{}", candidate.simple_name),
                0,
                Some(root_binary.to_owned()),
                false,
            ),
        ],
    };
    let targets = [target];
    let mut sites = Vec::new();
    let mut refusals = Vec::new();
    for physical in [root, child] {
        for method in &physical.methods {
            match &method.outcome {
                class_source::ClassSourceOutcome::Recovered { .. } => {}
                class_source::ClassSourceOutcome::NoBody => continue,
                _ => {
                    return Ok(Calls::Refused {
                        reason: "a family method with unknown body prevents a complete call census"
                            .to_owned(),
                        sites,
                        refusals,
                    });
                }
            }
            if let Err(error) = budget.poll() {
                merge_execution(execution, stop_execution(&error, budget));
                return Ok(Calls::Refused {
                    reason: "member call scan stopped".to_owned(),
                    sites,
                    refusals,
                });
            }
            let caller = method.item.identity.clone();
            // Both callers are in the proved lexical source family. This is the accessibility
            // premise for a non-public member/constructor; raw ACC_PUBLIC is not consulted.
            if caller.owner != root.class && caller.owner != child.class {
                return Ok(Calls::Refused {
                    reason: "candidate caller is outside the proved source family".to_owned(),
                    sites,
                    refusals,
                });
            }
            let analyzed = match jarde_jvm::analyze_method_ir(
                content,
                &crate::ir::MethodAnalysisRequest {
                    environment: environment.clone(),
                    method: caller.clone(),
                    stages: MethodOperation::Analysis.stages().to_vec(),
                },
                budget,
            ) {
                Ok(analyzed) => analyzed,
                Err(error) => {
                    merge_execution(execution, stop_execution(&error, budget));
                    return Ok(Calls::Refused {
                        reason: "member call analysis stopped".to_owned(),
                        sites,
                        refusals,
                    });
                }
            };
            merge_execution(execution, analyzed.report().execution.clone());
            if analyzed.report().method != caller
                || !matches!(
                    analyzed.report().execution,
                    ExecutionReport::Complete { .. }
                )
            {
                return Ok(Calls::Refused {
                    reason: "member call analysis did not complete".to_owned(),
                    sites,
                    refusals,
                });
            }
            let Some(code) = analyzed.ir().code() else {
                return Ok(Calls::Refused {
                    reason: "recovered family method has no complete bytecode".to_owned(),
                    sites,
                    refusals,
                });
            };
            let mut allocations = Vec::new();
            for instruction in &code.instructions {
                if let Err(error) = budget.charge(CountedBudgetDimension::AnalysisSteps, 1) {
                    merge_execution(execution, stop_execution(&error, budget));
                    return Ok(Calls::Refused {
                        reason: "member allocation scan stopped".to_owned(),
                        sites,
                        refusals,
                    });
                }
                if instruction.opcode == 0xbb
                    && instruction.constant_pool_index.is_some_and(|index| {
                        jarde_reader::classfile::cp_class_name(analyzed.ir().constant_pool(), index)
                            .is_ok_and(|name| name.0 == candidate.child_name)
                    })
                {
                    allocations.push(instruction.bci);
                }
            }
            if allocations.is_empty() {
                continue;
            }
            let facts = recovery_facts(analyzed.ir().declaration(), Some(code), &caller);
            let proof_request = jarde_java::RecoveryRequest::new(
                analyzed.ir(),
                &facts,
                environment.runtime.profile.clone(),
            )
            .with_member_inner_targets(&targets)
            .with_evidence(
                RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::RuleDetails),
            );
            let recovery = jarde_java::recover(&proof_request, budget);
            merge_execution(execution, recovery.execution.clone());
            if !matches!(recovery.execution, ExecutionReport::Complete { .. })
                || !recovery.produced()
            {
                return Ok(Calls::Refused {
                    reason: "member call new@1 proof did not complete".to_owned(),
                    sites,
                    refusals,
                });
            }
            for head in allocations {
                let Some(record) = recovery
                    .news
                    .iter()
                    .find(|record| record.head == head && record.class == child_binary)
                else {
                    refusals.push(MemberCallRefusal {
                        caller: caller.clone(),
                        allocation_bci: head,
                        reason: "new@1 did not publish this child allocation".to_owned(),
                    });
                    continue;
                };
                match crate::member_inner::prove_family_call_site(
                    &caller,
                    analyzed.ir(),
                    record,
                    &capture.constructor,
                    &candidate.child_name,
                ) {
                    Ok(site) => sites.push(site),
                    Err(reason) => refusals.push(MemberCallRefusal {
                        caller: caller.clone(),
                        allocation_bci: head,
                        reason,
                    }),
                }
            }
        }
    }
    if refusals.is_empty() {
        Ok(Calls::Proved { sites })
    } else {
        Ok(Calls::Refused {
            reason: "one or more family member allocations lack a closed new@1 proof".to_owned(),
            sites,
            refusals,
        })
    }
}

fn class_source_member_inner_candidates(
    ir: &jarde_jvm::method_ir::MethodIr,
    budget: &mut Budget,
) -> Result<Vec<(String, String)>> {
    use jarde_reader::classfile::{CpEntryKind, cp_class_name, cp_entry};
    use std::collections::BTreeSet;

    let Some(code) = ir.code() else {
        return Ok(Vec::new());
    };
    let pool = ir.constant_pool();
    let mut allocated = BTreeSet::new();
    for instruction in &code.instructions {
        budget.poll()?;
        if instruction.opcode == 0xbb
            && let Some(index) = instruction.constant_pool_index
            && let Ok(name) = cp_class_name(pool, index)
        {
            allocated.insert(name.0);
        }
    }
    let mut candidates = BTreeSet::new();
    for instruction in &code.instructions {
        budget.poll()?;
        if instruction.opcode != 0xb7 {
            continue;
        }
        let Some(index) = instruction.constant_pool_index else {
            continue;
        };
        let Ok(entry) = cp_entry(pool, index) else {
            continue;
        };
        let CpEntryKind::MethodRef {
            owner,
            name,
            descriptor,
            ..
        } = &entry.kind
        else {
            continue;
        };
        if name.0 != b"<init>" || !allocated.contains(&owner.0) || !owner.0.contains(&b'$') {
            continue;
        }
        let (Ok(owner), Ok(descriptor)) = (
            std::str::from_utf8(&owner.0),
            std::str::from_utf8(&descriptor.0),
        ) else {
            continue;
        };
        candidates.insert((owner.to_owned(), descriptor.to_owned()));
    }
    Ok(candidates.into_iter().collect())
}

/// A caller row, when present for the exact target, must agree with the selected target's proof.
/// Absence is not a contradiction: Java verification does not require the caller to duplicate the
/// member declaration's InnerClasses attribute.
fn caller_member_relation_agrees(
    ir: &jarde_jvm::method_ir::MethodIr,
    caller: &class_source::ClassSourceAssemblyContext,
    proof: &crate::member_inner::MemberInnerTarget,
    budget: &mut Budget,
) -> Result<bool> {
    use jarde_reader::classfile::cp_class_name;

    let pool = ir.constant_pool();
    let mut match_entry = None;
    for entry in &caller.inner_classes {
        budget.poll()?;
        if !cp_class_name(pool, entry.class_index)
            .is_ok_and(|name| name.0 == proof.owner.as_bytes())
        {
            continue;
        }
        if match_entry.replace(entry).is_some() {
            return Ok(false);
        }
    }
    let Some(entry) = match_entry else {
        return Ok(true);
    };
    if entry.outer_class_index == 0 || entry.access_flags & (0x0001 | 0x0008) != 0x0001 {
        return Ok(false);
    }
    Ok(cp_class_name(pool, entry.outer_class_index)
        .is_ok_and(|outer| outer.0 == proof.outer.as_bytes())
        && entry
            .inner_name
            .as_ref()
            .is_some_and(|name| name.0 == proof.simple_name.as_bytes()))
}

fn source_type_path_segment(
    definition: &PhysicalDefinitionId,
    binary_name: &str,
    source_name: String,
    type_parameter_count: usize,
    enclosing_binary_name: Option<String>,
    is_static: bool,
) -> jarde_java::report::ProvedMemberInnerSourceSegment {
    jarde_java::report::ProvedMemberInnerSourceSegment {
        definition: definition.clone(),
        binary_name: binary_name.to_owned(),
        source_name,
        type_parameter_count,
        enclosing_binary_name,
        is_static,
    }
}

fn member_inner_source_type_path(
    target_definition: &PhysicalDefinitionId,
    target: &crate::member_inner::MemberInnerTarget,
    outer_definition: &PhysicalDefinitionId,
    generic_outer: &crate::member_inner::GenericOuterRelationProof,
    enclosing_definition: &PhysicalDefinitionId,
    enclosing_facts: &ClassMemberFacts,
) -> Option<Vec<jarde_java::report::ProvedMemberInnerSourceSegment>> {
    if generic_outer.top_level_non_generic_owner {
        if generic_outer.type_parameter_count != 0
            || !generic_outer.simple_name.is_empty()
            || target.outer != generic_outer.enclosing
            || target.owner != format!("{}${}", target.outer, target.simple_name)
            || target.source_type_parameter_count != 0
            || enclosing_facts.this_class.raw().0 != generic_outer.enclosing.as_bytes()
        {
            return None;
        }
        let enclosing_binary = std::str::from_utf8(&enclosing_facts.this_class.raw().0).ok()?;
        let package_and_simple: Vec<_> = enclosing_binary.split('/').collect();
        if package_and_simple
            .iter()
            .any(|part| !jarde_java::names::is_java_identifier(part) || part.contains('$'))
        {
            return None;
        }
        let root_source_name = enclosing_binary.replace('/', ".");
        let target_source_name = format!("{root_source_name}.{}", target.simple_name);
        return Some(vec![
            source_type_path_segment(
                enclosing_definition,
                enclosing_binary,
                root_source_name,
                0,
                None,
                true,
            ),
            source_type_path_segment(
                target_definition,
                &target.owner,
                target_source_name,
                0,
                Some(target.outer.clone()),
                false,
            ),
        ]);
    }
    if generic_outer.type_parameter_count != 1
        || enclosing_facts.this_class.raw().0 != generic_outer.enclosing.as_bytes()
        || target.outer != format!("{}${}", generic_outer.enclosing, generic_outer.simple_name)
        || target.owner != format!("{}${}", target.outer, target.simple_name)
    {
        return None;
    }
    let enclosing_binary = std::str::from_utf8(&enclosing_facts.this_class.raw().0).ok()?;
    let package_and_simple: Vec<_> = enclosing_binary.split('/').collect();
    if package_and_simple
        .iter()
        .any(|part| !jarde_java::names::is_java_identifier(part))
    {
        return None;
    }
    let root_source_name = enclosing_binary.replace('/', ".");
    let outer_source_name = format!("{root_source_name}.{}", generic_outer.simple_name);
    let target_source_name = format!("{outer_source_name}.{}", target.simple_name);
    Some(vec![
        source_type_path_segment(
            enclosing_definition,
            enclosing_binary,
            root_source_name,
            0,
            None,
            true,
        ),
        source_type_path_segment(
            outer_definition,
            &target.outer,
            outer_source_name,
            generic_outer.type_parameter_count,
            Some(generic_outer.enclosing.clone()),
            true,
        ),
        source_type_path_segment(
            target_definition,
            &target.owner,
            target_source_name,
            target.source_type_parameter_count,
            Some(target.outer.clone()),
            false,
        ),
    ])
}

fn member_inner_resolution_stop(execution: &ExecutionReport, budget: &mut Budget) -> Result<()> {
    match execution {
        ExecutionReport::Cancelled { .. } => Err(Error::Cancelled {
            reason: "member constructor target resolution was cancelled".to_owned(),
        }),
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded { dimension },
            ..
        }
        | ExecutionReport::Failed {
            reason: TerminationReason::BudgetExceeded { dimension },
            ..
        } => {
            if let Ok(counted) = CountedBudgetDimension::try_from(*dimension)
                && let Err(error) = budget.charge(counted, 1)
            {
                return Err(error);
            }
            use jarde_reader::budget::BudgetDimension as D;
            let limits = budget.limits();
            let usage = budget.usage();
            let (limit, consumed) = match dimension {
                D::InputBytes => (limits.input_bytes, usage.input_bytes),
                D::ArchiveEntries => (limits.archive_entries, usage.archive_entries),
                D::EntryBytes => (limits.entry_bytes, usage.entry_bytes),
                D::ReadBytes => (limits.read_bytes, usage.read_bytes),
                D::ClassBytes => (limits.class_bytes, usage.class_bytes),
                D::AttributeBytes => (limits.attribute_bytes, usage.attribute_bytes),
                D::CodeBytes => (limits.code_bytes, usage.code_bytes),
                D::ResultItems => (limits.result_items, usage.result_items),
                D::OutputBytes => (limits.output_bytes, usage.output_bytes),
                D::ClassHeaders => (limits.class_headers, usage.class_headers),
                D::MethodBodies => (limits.method_bodies, usage.method_bodies),
                D::IrItems => (limits.ir_items, usage.ir_items),
                D::IrEdges => (limits.ir_edges, usage.ir_edges),
                D::AnalysisSteps => (limits.analysis_steps, usage.analysis_steps),
                D::NormalizationClones => (limits.normalization_clones, usage.normalization_clones),
                D::NestedDepth => (limits.nested_depth, usage.nested_depth),
                D::DependencyDepth => (limits.dependency_depth, usage.dependency_depth),
                D::ElapsedMillis => (limits.elapsed_millis, usage.elapsed_millis),
            };
            Err(Error::BudgetExceeded {
                dimension: *dimension,
                limit,
                consumed,
                requested: 1,
            })
        }
        _ => Ok(()),
    }
}

fn read_class_source_member_inner_targets(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    ir: &jarde_jvm::method_ir::MethodIr,
    caller: &class_source::ClassSourceAssemblyContext,
    budget: &mut Budget,
) -> Result<Vec<jarde_java::report::ProvedMemberInnerTarget>> {
    let mut targets = Vec::new();
    let mut execution = ExecutionReport::Complete {
        usage: budget.usage(),
    };
    for (owner, descriptor) in class_source_member_inner_candidates(ir, budget)? {
        let resolved = resolve_class_source_dependency_read(
            content,
            &request.environment,
            Some(&request.method),
            &owner,
            &mut execution,
            budget,
        )?;
        member_inner_resolution_stop(&execution, budget)?;
        let Some((definition, read)) = resolved else {
            continue;
        };
        let Some(proof) = crate::member_inner::prove_target(
            &read.bytes,
            &read.facts,
            &owner,
            &descriptor,
            budget,
        )?
        else {
            continue;
        };
        if !caller_member_relation_agrees(ir, caller, &proof, budget)? {
            continue;
        }
        // A generic enclosing declaration brings type variables into the member's source scope
        // even when this target declares no Signature of its own. This first slice does not project
        // that scope, so confirm the selected outer definition before handing the target fact over.
        let outer_read = resolve_class_source_dependency_read(
            content,
            &request.environment,
            Some(&request.method),
            &proof.outer,
            &mut execution,
            budget,
        )?;
        member_inner_resolution_stop(&execution, budget)?;
        let Some((outer_definition, outer_read)) = outer_read else {
            continue;
        };
        let Some(generic_outer) = crate::member_inner::outer_relation_agrees(
            &outer_read.bytes,
            &outer_read.facts,
            &proof,
            budget,
        )?
        else {
            continue;
        };
        let enclosing_read = resolve_class_source_dependency_read(
            content,
            &request.environment,
            Some(&request.method),
            &generic_outer.enclosing,
            &mut execution,
            budget,
        )?;
        member_inner_resolution_stop(&execution, budget)?;
        let Some((enclosing_definition, enclosing_read)) = enclosing_read else {
            continue;
        };
        if !crate::member_inner::enclosing_relation_agrees(
            &enclosing_read.bytes,
            &enclosing_read.facts,
            &generic_outer,
            budget,
        )? {
            continue;
        }
        let Some(source_type_path) = member_inner_source_type_path(
            &definition,
            &proof,
            &outer_definition,
            &generic_outer,
            &enclosing_definition,
            &enclosing_read.facts,
        ) else {
            continue;
        };
        targets.push(jarde_java::report::ProvedMemberInnerTarget {
            definition,
            owner: proof.owner,
            outer: proof.outer,
            simple_name: proof.simple_name,
            constructor_descriptor: proof.constructor_descriptor,
            capture_field: proof.capture_field,
            generic_diamond: proof.generic_diamond,
            source_type_path,
        });
    }
    Ok(targets)
}

#[cfg(test)]
mod member_inner_target_tests {
    use super::*;
    use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
    use std::io::{Cursor, Write};

    const FULL_JAR: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/full-target.jar"
    );
    const MISSING_JAR: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/missing-target.jar"
    );
    const WRONG_RELATION_JAR: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/wrong-relation.jar"
    );
    const WRONG_OUTER_RELATION_JAR: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/invalid-controls/byte-variants/wrong-outer-relation.jar"
    );
    const OUTER: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/classes/nested/SimpleOuter.class"
    );
    const INNER: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/classes/nested/SimpleOuter$Inner.class"
    );
    const CALLER: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/classes/nested/UseInner.class"
    );
    const RUNNER: &[u8] = include_bytes!(
        "../openspec/evidence/java-syntax-2026-09-25/inner-generic-instance-constructor/simple-member/classes/nested/InnerRunner.class"
    );
    const MATRIX_OUTER: &[u8] = include_bytes!(
        "../tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/Outer.class"
    );
    const MATRIX_A: &[u8] = include_bytes!(
        "../tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/Outer$A.class"
    );
    const MATRIX_PLAIN: &[u8] = include_bytes!(
        "../tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/Outer$A$Plain.class"
    );
    const MATRIX_GENERIC: &[u8] = include_bytes!(
        "../tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/Outer$A$Generic.class"
    );
    const MATRIX_CALLERS: [(&str, &[u8]); 4] = [
        (
            "UsePlainRaw",
            include_bytes!(
                "../tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/UsePlainRaw.class"
            ),
        ),
        (
            "UsePlain",
            include_bytes!(
                "../tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/UsePlain.class"
            ),
        ),
        (
            "UseGenericObject",
            include_bytes!(
                "../tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/UseGenericObject.class"
            ),
        ),
        (
            "UseGenericTyped",
            include_bytes!(
                "../tests/fixtures/recover-generic-enclosing-member-call-sites/matrix/UseGenericTyped.class"
            ),
        ),
    ];

    fn test_budget() -> Budget {
        let mut limits = crate::task_budget(&[]).unwrap().limits().clone();
        limits.input_bytes = u64::MAX;
        limits.archive_entries = u64::MAX;
        limits.entry_bytes = u64::MAX;
        limits.read_bytes = u64::MAX;
        limits.class_bytes = u64::MAX;
        limits.attribute_bytes = u64::MAX;
        limits.code_bytes = u64::MAX;
        limits.class_headers = u64::MAX;
        limits.method_bodies = u64::MAX;
        limits.ir_items = u64::MAX;
        limits.ir_edges = u64::MAX;
        limits.analysis_steps = u64::MAX;
        limits.result_items = u64::MAX;
        limits.elapsed_millis = u64::MAX;
        Budget::new(limits)
    }

    fn fixture(
        jar: &[u8],
    ) -> (
        ArtifactSnapshot,
        crate::ir::MethodAnalysisRequest,
        jarde_jvm::method_ir::MethodIrAnalysis,
        class_source::ClassSourceAssemblyContext,
    ) {
        let engine = Engine::new();
        let snapshot = engine
            .open(ArtifactInput::bytes(jar.to_vec()), &mut test_budget())
            .unwrap();
        let environment = EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: jarde_reader::view::LoaderId("app".to_owned()),
        };
        let report = match engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("nested/UseInner"),
                    },
                    environment: environment.clone(),
                },
                &mut test_budget(),
            )
            .unwrap()
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("caller selection must be unique: {other:?}"),
        };
        let mut budget = test_budget();
        let (caller, _) = read_definition(&snapshot, &report.class, &mut budget).unwrap();
        let pool = class_constant_pool(&caller.bytes, &budget).unwrap();
        let shells: Vec<_> = caller
            .facts
            .attributes
            .iter()
            .filter(|attribute| {
                matches!(
                    attribute.name.raw().0.as_slice(),
                    b"InnerClasses" | b"EnclosingMethod"
                )
            })
            .cloned()
            .collect();
        let context = class_source::read_class_source_assembly_context(
            &caller.bytes,
            &shells,
            &pool,
            &mut budget,
        )
        .unwrap();
        let caller_definition = report.class.clone();
        let request = crate::ir::MethodAnalysisRequest {
            environment: environment.build(std::slice::from_ref(&snapshot)).unwrap(),
            method: PhysicalMethodId {
                owner: caller_definition,
                name: JvmBytes(b"make".to_vec()),
                descriptor: JvmBytes(b"(Lnested/SimpleOuter;I)Ljava/lang/Object;".to_vec()),
            },
            stages: MethodOperation::Recovery.stages().to_vec(),
        };
        let analyzed =
            jarde_jvm::analyze_method_ir(std::slice::from_ref(&snapshot), &request, &mut budget)
                .unwrap();
        (snapshot, request, analyzed, context)
    }

    fn jar_of(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        {
            let mut zip = ZipArchiveWriter::new(&mut output);
            for (name, bytes) in entries {
                let (mut entry, config) = zip
                    .new_file(EntryPath::verbatim(name.to_vec()))
                    .compression_method(CompressionMethod::new(0))
                    .start()
                    .unwrap();
                let mut writer = config.wrap(&mut entry);
                writer.write_all(bytes).unwrap();
                let (_, descriptor) = writer.finish().unwrap();
                entry.finish(descriptor).unwrap();
            }
            zip.finish().unwrap();
        }
        output.into_inner()
    }

    fn duplicate_target_jar() -> Vec<u8> {
        jar_of(&[
            (b"nested/SimpleOuter.class", OUTER),
            (b"nested/UseInner.class", CALLER),
            (b"nested/SimpleOuter$Inner.class", INNER),
            (b"nested/SimpleOuter$Inner.class", INNER),
        ])
    }

    fn matrix_jar(outer_a: &[u8], duplicate_a: bool) -> Vec<u8> {
        matrix_jar_variant(outer_a, duplicate_a, None)
    }

    fn matrix_jar_variant(
        outer_a: &[u8],
        duplicate_a: bool,
        omitted_class: Option<&str>,
    ) -> Vec<u8> {
        let mut entries = vec![
            (b"matrix/Outer.class".as_slice(), MATRIX_OUTER),
            (b"matrix/Outer$A.class".as_slice(), outer_a),
            (b"matrix/Outer$A$Plain.class".as_slice(), MATRIX_PLAIN),
            (b"matrix/Outer$A$Generic.class".as_slice(), MATRIX_GENERIC),
        ];
        if let Some(omitted_class) = omitted_class {
            entries.retain(|(name, _)| *name != omitted_class.as_bytes());
        }
        if duplicate_a {
            entries.push((b"matrix/Outer$A.class".as_slice(), outer_a));
        }
        let caller_paths = MATRIX_CALLERS
            .iter()
            .map(|(name, _)| format!("matrix/{name}.class"))
            .collect::<Vec<_>>();
        for (path, (_, bytes)) in caller_paths.iter().zip(&MATRIX_CALLERS) {
            entries.push((path.as_bytes(), bytes));
        }
        jar_of(&entries)
    }

    fn wrong_matrix_a_relation() -> Vec<u8> {
        let mut bytes = MATRIX_A.to_vec();
        let mut budget = test_budget();
        let facts = class_member_facts(MATRIX_A, &mut budget).unwrap();
        let pool = class_constant_pool(MATRIX_A, &budget).unwrap();
        let inner_classes = facts
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"InnerClasses")
            .expect("matrix A carries its nested member rows");
        let content_start = usize::try_from(inner_classes.content_span.start).unwrap();
        let content_end =
            content_start + usize::try_from(inner_classes.content_span.length).unwrap();
        let entries = &bytes[content_start..content_end];
        let count = usize::from(u16::from_be_bytes([entries[0], entries[1]]));
        let matching_rows = (0..count)
            .filter(|index| {
                let at = 2 + index * 8;
                let class_index = u16::from_be_bytes([entries[at], entries[at + 1]]);
                jarde_reader::classfile::cp_class_name(&pool, class_index)
                    .is_ok_and(|name| name.0 == b"matrix/Outer$A")
            })
            .collect::<Vec<_>>();
        let [row] = matching_rows.as_slice() else {
            panic!("matrix A has one self row in InnerClasses");
        };
        let outer_index = content_start + 2 + *row * 8 + 2;
        bytes[outer_index..outer_index + 2].copy_from_slice(&0_u16.to_be_bytes());
        bytes
    }

    fn matrix_fixture(
        jar: &[u8],
        caller_name: &str,
        method_descriptor: &[u8],
    ) -> (
        ArtifactSnapshot,
        crate::ir::MethodAnalysisRequest,
        jarde_jvm::method_ir::MethodIrAnalysis,
        class_source::ClassSourceAssemblyContext,
        class_source::ClassSourceReport,
    ) {
        let engine = Engine::new();
        let snapshot = engine
            .open(ArtifactInput::bytes(jar.to_vec()), &mut test_budget())
            .unwrap();
        let environment = EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: jarde_reader::view::LoaderId("app".to_owned()),
        };
        let report = match engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal(format!("matrix/{caller_name}")),
                    },
                    environment: environment.clone(),
                },
                &mut test_budget(),
            )
            .unwrap()
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("matrix caller selection must be unique: {other:?}"),
        };
        let mut budget = test_budget();
        let (caller, _) = read_definition(&snapshot, &report.class, &mut budget).unwrap();
        let pool = class_constant_pool(&caller.bytes, &budget).unwrap();
        let shells: Vec<_> = caller
            .facts
            .attributes
            .iter()
            .filter(|attribute| {
                matches!(
                    attribute.name.raw().0.as_slice(),
                    b"InnerClasses" | b"EnclosingMethod"
                )
            })
            .cloned()
            .collect();
        let context = class_source::read_class_source_assembly_context(
            &caller.bytes,
            &shells,
            &pool,
            &mut budget,
        )
        .unwrap();
        let caller_definition = report.class.clone();
        let request = crate::ir::MethodAnalysisRequest {
            environment: environment.build(std::slice::from_ref(&snapshot)).unwrap(),
            method: PhysicalMethodId {
                owner: caller_definition,
                name: JvmBytes(b"make".to_vec()),
                descriptor: JvmBytes(method_descriptor.to_vec()),
            },
            stages: MethodOperation::Recovery.stages().to_vec(),
        };
        let analyzed =
            jarde_jvm::analyze_method_ir(std::slice::from_ref(&snapshot), &request, &mut budget)
                .unwrap();
        (snapshot, request, analyzed, context, report)
    }

    fn wrong_matrix_a_signature() -> Vec<u8> {
        let mut bytes = MATRIX_A.to_vec();
        let original = b"<T:Ljava/lang/Object;>Ljava/lang/Object;";
        let replacement = b"<T:Ljava/lang/Number;>Ljava/lang/Object;";
        assert_eq!(original.len(), replacement.len());
        let matches: Vec<_> = bytes
            .windows(original.len())
            .enumerate()
            .filter_map(|(index, window)| (window == original).then_some(index))
            .collect();
        let [index] = matches.as_slice() else {
            panic!("matrix A has one class Signature byte sequence");
        };
        bytes[*index..*index + original.len()].copy_from_slice(replacement);
        bytes
    }

    fn caller_without_inner_classes_jar() -> Vec<u8> {
        let mut budget = test_budget();
        let facts = class_member_facts(CALLER, &mut budget).unwrap();
        let [attribute] = facts.attributes.as_slice() else {
            panic!("the frozen caller has one class attribute");
        };
        assert_eq!(attribute.name.raw().0, b"InnerClasses");
        let start = usize::try_from(attribute.span.start).unwrap();
        let end = start + usize::try_from(attribute.span.length).unwrap();
        let mut caller = CALLER.to_vec();
        caller[start - 2..start].copy_from_slice(&0_u16.to_be_bytes());
        caller.drain(start..end);
        assert!(
            class_member_facts(&caller, &mut budget)
                .unwrap()
                .attributes
                .is_empty()
        );
        jar_of(&[
            (b"nested/SimpleOuter.class", OUTER),
            (b"nested/UseInner.class", &caller),
            (b"nested/SimpleOuter$Inner.class", INNER),
            (b"nested/InnerRunner.class", RUNNER),
        ])
    }

    #[test]
    fn selected_target_is_required_and_ambiguous_definitions_do_not_prove() {
        for (jar, expected) in [
            (FULL_JAR.to_vec(), 1),
            (MISSING_JAR.to_vec(), 0),
            (WRONG_RELATION_JAR.to_vec(), 0),
            (WRONG_OUTER_RELATION_JAR.to_vec(), 0),
            (duplicate_target_jar(), 0),
        ] {
            let (snapshot, request, analyzed, context) = fixture(&jar);
            let targets = read_class_source_member_inner_targets(
                std::slice::from_ref(&snapshot),
                &request,
                analyzed.ir(),
                &context,
                &mut test_budget(),
            )
            .unwrap();
            assert_eq!(targets.len(), expected);
            if let [target] = targets.as_slice() {
                assert_eq!(target.owner, "nested/SimpleOuter$Inner");
                assert_eq!(target.constructor_descriptor, "(Lnested/SimpleOuter;I)V");
                assert_eq!(target.definition.snapshot(), snapshot.id());
            }
        }
    }

    #[test]
    fn caller_relation_is_optional_but_a_present_row_must_agree() {
        let absent_jar = caller_without_inner_classes_jar();
        let (absent_snapshot, absent_request, absent_analyzed, absent_context) =
            fixture(&absent_jar);
        assert!(absent_context.inner_classes.is_empty());
        let targets = read_class_source_member_inner_targets(
            std::slice::from_ref(&absent_snapshot),
            &absent_request,
            absent_analyzed.ir(),
            &absent_context,
            &mut test_budget(),
        )
        .unwrap();
        assert_eq!(
            targets.len(),
            1,
            "the selected target's own relation proves membership"
        );

        let (snapshot, request, analyzed, context) = fixture(FULL_JAR);
        let mut contradictory = context.clone();
        let row = contradictory
            .inner_classes
            .iter_mut()
            .find(|row| {
                row.inner_name
                    .as_ref()
                    .is_some_and(|name| name.0 == b"Inner")
            })
            .unwrap();
        row.outer_class_index = 0;
        assert!(
            read_class_source_member_inner_targets(
                std::slice::from_ref(&snapshot),
                &request,
                analyzed.ir(),
                &contradictory,
                &mut test_budget(),
            )
            .unwrap()
            .is_empty()
        );

        let mut duplicate = context.clone();
        duplicate
            .inner_classes
            .push(context.inner_classes[0].clone());
        assert!(
            read_class_source_member_inner_targets(
                std::slice::from_ref(&snapshot),
                &request,
                analyzed.ir(),
                &duplicate,
                &mut test_budget(),
            )
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn generic_outer_path_and_all_member_call_shapes_remain_selected_and_emittable() {
        let jar = matrix_jar(MATRIX_A, false);
        for (caller_name, descriptor, expected_header, expected_target, generic_diamond) in [
            (
                "UsePlainRaw",
                b"(Lmatrix/Outer$A;I)Ljava/lang/Object;".as_slice(),
                "java.lang.Object make(matrix.Outer.A arg0, int arg1)",
                "matrix/Outer$A$Plain",
                false,
            ),
            (
                "UsePlain",
                b"(Lmatrix/Outer$A;I)Ljava/lang/Object;".as_slice(),
                "java.lang.Object make(matrix.Outer.A<java.lang.String> arg0, int arg1)",
                "matrix/Outer$A$Plain",
                false,
            ),
            (
                "UseGenericObject",
                b"(Lmatrix/Outer$A;I)Ljava/lang/Object;".as_slice(),
                "java.lang.Object make(matrix.Outer.A<java.lang.String> arg0, int arg1)",
                "matrix/Outer$A$Generic",
                true,
            ),
            (
                "UseGenericTyped",
                b"(Lmatrix/Outer$A;I)Lmatrix/Outer$A$Generic;".as_slice(),
                "matrix.Outer.A<java.lang.String>.Generic<java.lang.Integer> make(",
                "matrix/Outer$A$Generic",
                true,
            ),
        ] {
            let (snapshot, request, analyzed, context, report) =
                matrix_fixture(&jar, caller_name, descriptor);
            let targets = read_class_source_member_inner_targets(
                std::slice::from_ref(&snapshot),
                &request,
                analyzed.ir(),
                &context,
                &mut test_budget(),
            )
            .unwrap();
            let [target] = targets.as_slice() else {
                panic!("{caller_name} must resolve one selected member target: {targets:?}");
            };
            assert_eq!(target.owner, expected_target);
            assert_eq!(
                target
                    .source_type_path
                    .iter()
                    .map(|segment| segment.source_name.as_str())
                    .collect::<Vec<_>>(),
                [
                    "matrix.Outer",
                    "matrix.Outer.A",
                    if expected_target.ends_with("Plain") {
                        "matrix.Outer.A.Plain"
                    } else {
                        "matrix.Outer.A.Generic"
                    }
                ]
            );
            assert_eq!(target.generic_diamond, generic_diamond);

            let method = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"make")
                .expect("matrix caller publishes make");
            let declaration = method.declaration.as_deref().unwrap_or_default();
            assert!(
                declaration.contains(expected_header),
                "{caller_name}: {declaration}"
            );
            assert!(
                method.text.contains("matrix.Outer.A.mark(arg1)"),
                "{caller_name}: {}",
                method.text
            );
            if expected_target.ends_with("Plain") {
                assert!(
                    method.text.contains(".new Plain("),
                    "{caller_name}: {}",
                    method.text
                );
            } else {
                assert!(
                    method.text.contains(".new Generic<>("),
                    "{caller_name}: {}",
                    method.text
                );
            }
            let class_source::ClassSourceOutcome::Recovered { .. } = &method.outcome else {
                panic!("{caller_name} body must be recovered");
            };
            let evidence = jarde_java::RecoveryEvidenceRequest::essential()
                .with_kind(jarde_java::RecoveryEvidenceKind::RuleDetails);
            let (detailed, _, _, _, _, _, _, generic_return, _, _) =
                recovery_from_with_class_candidates(
                    std::slice::from_ref(&snapshot),
                    &request,
                    analyzed,
                    CalleeClass::None,
                    Some(&context),
                    &evidence,
                    &mut test_budget(),
                    true,
                    true,
                    false,
                )
                .unwrap();
            assert!(matches!(
                generic_return.map(|candidate| candidate.value),
                Some(jarde_java::report::GenericReturnValue::MemberCreation { .. })
            ));
            let report = detailed.recovery();
            let [new_record] = report.news.as_slice() else {
                panic!(
                    "{caller_name} must retain one physical new@1 record: {:?}",
                    report.news
                );
            };
            assert!(new_record.presented);
            assert_eq!(new_record.class, expected_target);
            assert_eq!(new_record.arguments.len(), 2);
            assert_eq!(new_record.arguments[0], 5, "physical outer argument BCI");
            assert!(new_record.arguments[0] < new_record.arguments[1]);
            assert!(new_record.head < new_record.arguments[0]);
            assert!(new_record.constructor.unwrap() > new_record.arguments[1]);
        }
    }

    #[test]
    fn malformed_generic_outer_signature_and_ambiguous_outer_definitions_do_not_prove() {
        let wrong_signature = wrong_matrix_a_signature();
        let wrong_relation = wrong_matrix_a_relation();
        for jar in [
            matrix_jar(&wrong_signature, false),
            matrix_jar(&wrong_relation, false),
            matrix_jar(MATRIX_A, true),
            matrix_jar_variant(MATRIX_A, false, Some("matrix/Outer$A$Plain.class")),
            matrix_jar_variant(MATRIX_A, false, Some("matrix/Outer.class")),
        ] {
            let (snapshot, request, analyzed, context, _) =
                matrix_fixture(&jar, "UsePlain", b"(Lmatrix/Outer$A;I)Ljava/lang/Object;");
            let targets = read_class_source_member_inner_targets(
                std::slice::from_ref(&snapshot),
                &request,
                analyzed.ir(),
                &context,
                &mut test_budget(),
            )
            .unwrap();
            assert!(targets.is_empty());
        }
    }

    #[test]
    fn target_selection_observes_budget_and_cancellation() {
        let (snapshot, request, analyzed, context) = fixture(FULL_JAR);
        let mut limits = test_budget().limits().clone();
        limits.class_headers = 0;
        let error = read_class_source_member_inner_targets(
            std::slice::from_ref(&snapshot),
            &request,
            analyzed.ir(),
            &context,
            &mut Budget::new(limits),
        )
        .unwrap_err();
        assert!(matches!(error, Error::BudgetExceeded { .. }));

        let cancellation = jarde_reader::budget::CancellationToken::new();
        cancellation.cancel();
        let error = read_class_source_member_inner_targets(
            std::slice::from_ref(&snapshot),
            &request,
            analyzed.ir(),
            &context,
            &mut Budget::with_cancellation_token(test_budget().limits().clone(), cancellation),
        )
        .unwrap_err();
        assert!(matches!(error, Error::Cancelled { .. }));
    }
}

#[cfg(test)]
mod interface_super_proof_tests {
    use super::*;
    use jarde_reader::view::MultiReleasePolicy;
    use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
    use std::io::{Cursor, Write};

    const SPECIAL: &[u8] =
        include_bytes!("../tests/fixtures/p3-special-dispatch/v8/SpecialProbe.class");
    const BASE: &[u8] = include_bytes!("../tests/fixtures/p3-special-dispatch/v8/BaseProbe.class");
    const DEFAULT: &[u8] =
        include_bytes!("../tests/fixtures/p3-special-dispatch/v8/DefaultProbe.class");
    const LEFT_DEFAULT: &[u8] = include_bytes!(
        "../tests/fixtures/p3-special-dispatch/interface-super-proof/proof/LeftDefault.class"
    );
    const RIGHT_DEFAULT: &[u8] = include_bytes!(
        "../tests/fixtures/p3-special-dispatch/interface-super-proof/proof/RightDefault.class"
    );
    const BOTH_DEFAULT: &[u8] = include_bytes!(
        "../tests/fixtures/p3-special-dispatch/interface-super-proof/proof/BothDefault.class"
    );

    fn budget() -> Budget {
        let mut limits = crate::facade::task_limits(&[]).expect("default task limits");
        limits.input_bytes = u64::MAX;
        limits.archive_entries = u64::MAX;
        limits.entry_bytes = u64::MAX;
        limits.read_bytes = u64::MAX;
        limits.class_bytes = u64::MAX;
        limits.attribute_bytes = u64::MAX;
        limits.code_bytes = u64::MAX;
        limits.class_headers = u64::MAX;
        limits.method_bodies = u64::MAX;
        limits.ir_items = u64::MAX;
        limits.ir_edges = u64::MAX;
        limits.analysis_steps = u64::MAX;
        limits.normalization_clones = u64::MAX;
        limits.elapsed_millis = u64::MAX;
        Budget::new(limits)
    }

    fn jar(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        {
            let mut archive = ZipArchiveWriter::new(&mut output);
            for (name, bytes) in entries {
                let (mut entry, config) = archive
                    .new_file(EntryPath::verbatim(name.to_vec()))
                    .compression_method(CompressionMethod::new(0))
                    .start()
                    .unwrap();
                let mut writer = config.wrap(&mut entry);
                writer.write_all(bytes).unwrap();
                let (_, descriptor) = writer.finish().unwrap();
                entry.finish(descriptor).unwrap();
            }
            archive.finish().unwrap();
        }
        output.into_inner()
    }

    fn fixture() -> (
        ArtifactSnapshot,
        crate::ir::MethodAnalysisRequest,
        jarde_jvm::method_ir::MethodIrAnalysis,
    ) {
        let engine = Engine::new();
        let snapshot = engine
            .open(
                ArtifactInput::bytes(jar(&[
                    (b"SpecialProbe.class", SPECIAL),
                    (b"BaseProbe.class", BASE),
                    (b"DefaultProbe.class", DEFAULT),
                ])),
                &mut budget(),
            )
            .unwrap();
        let environment_request = EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        };
        let source = engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal("SpecialProbe"),
                    },
                    environment: environment_request.clone(),
                },
                &mut budget(),
            )
            .unwrap();
        let OperationOutcome::Performed(source) = source else {
            panic!("fixture selection must be unique");
        };
        let method = source
            .methods
            .iter()
            .find(|method| method.item.name.raw().0 == b"defaultCall")
            .unwrap()
            .item
            .identity
            .clone();
        let request = crate::ir::MethodAnalysisRequest {
            environment: environment_request
                .build(std::slice::from_ref(&snapshot))
                .unwrap(),
            method,
            stages: MethodOperation::Recovery.stages().to_vec(),
        };
        let analyzed =
            jarde_jvm::analyze_method_ir(std::slice::from_ref(&snapshot), &request, &mut budget())
                .unwrap();
        (snapshot, request, analyzed)
    }

    fn conflicting_default_closure() -> (
        Vec<SelectedInterfaceNode>,
        std::collections::BTreeMap<Vec<u8>, usize>,
        std::collections::BTreeMap<Vec<u8>, Vec<Vec<u8>>>,
    ) {
        let engine = Engine::new();
        let snapshot = engine
            .open(
                ArtifactInput::bytes(jar(&[
                    (b"proof/LeftDefault.class", LEFT_DEFAULT),
                    (b"proof/RightDefault.class", RIGHT_DEFAULT),
                    (b"proof/BothDefault.class", BOTH_DEFAULT),
                ])),
                &mut budget(),
            )
            .unwrap();
        let request = EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        };
        let environment = request.build(std::slice::from_ref(&snapshot)).unwrap();
        let mut read_budget = budget();
        let mut execution = ExecutionReport::Complete {
            usage: read_budget.usage(),
        };
        let mut nodes = Vec::new();
        let mut by_name = std::collections::BTreeMap::new();
        let mut graph = std::collections::BTreeMap::new();
        for owner in [
            "proof/LeftDefault",
            "proof/RightDefault",
            "proof/BothDefault",
        ] {
            let (definition, read) = resolve_class_source_dependency_read(
                std::slice::from_ref(&snapshot),
                &environment,
                None,
                owner,
                &mut execution,
                &mut read_budget,
            )
            .unwrap()
            .expect("the selected plain JAR contains every proof interface");
            let mut facts = read.facts;
            assert!(facts.stopped_at.is_none());
            assert_eq!(facts.method_count as usize, facts.methods.len());
            let raw_owner = facts.this_class.raw().0.clone();
            let parents = facts
                .interfaces
                .iter()
                .map(|parent| parent.raw().0.clone())
                .collect::<Vec<_>>();
            if raw_owner.as_slice() == b"proof/BothDefault" {
                assert!(facts.methods.iter().any(|method| {
                    method.name.raw().0.as_slice() == b"value"
                        && method.descriptor.raw().0.as_slice() == b"()I"
                }));
                // javac needs the conflict-resolving override. Remove that one declaration from
                // these already selected facts to present the inherited multi-default case
                // directly to unique_source_default; this test covers the proof function only.
                facts
                    .methods
                    .retain(|method| method.name.raw().0.as_slice() != b"value");
            }
            let index = nodes.len();
            by_name.insert(raw_owner.clone(), index);
            graph.insert(raw_owner, parents);
            nodes.push(SelectedInterfaceNode { definition, facts });
        }
        (nodes, by_name, graph)
    }

    #[test]
    fn interface_hierarchy_cycles_are_unknown() {
        let (snapshot, request, _) = fixture();
        let mut selection_budget = budget();
        let mut execution = ExecutionReport::Complete {
            usage: selection_budget.usage(),
        };
        let (definition, read) = resolve_class_source_dependency_read(
            std::slice::from_ref(&snapshot),
            &request.environment,
            Some(&request.method),
            "DefaultProbe",
            &mut execution,
            &mut selection_budget,
        )
        .unwrap()
        .expect("the selected environment supplies DefaultProbe");
        let mut facts = read.facts;
        let owner = facts.this_class.raw().0.clone();
        facts.interfaces.push(facts.this_class.clone());
        let mut read_budget = budget();
        let usage = read_budget.usage();
        let mut reads = InterfaceSuperReads {
            content: std::slice::from_ref(&snapshot),
            environment: &request.environment,
            enclosing: &request.method,
            budget: &mut read_budget,
            execution: ExecutionReport::Complete { usage },
            attempted: std::collections::BTreeSet::from([owner.clone()]),
            by_name: std::collections::BTreeMap::from([(owner.clone(), 0)]),
            nodes: vec![SelectedInterfaceNode { definition, facts }],
        };
        let complete = ensure_interface_closure(
            &owner,
            0,
            &mut reads,
            &mut std::collections::BTreeMap::new(),
            &mut std::collections::BTreeMap::new(),
            &mut Vec::new(),
        )
        .unwrap();
        assert!(!complete, "a repeated active interface edge is not a proof");
    }

    #[test]
    fn interface_hierarchy_observes_dependency_depth_and_cancellation() {
        let (snapshot, request, analyzed) = fixture();
        let mut limits = budget().limits().clone();
        limits.dependency_depth = 0;
        let error = prove_interface_super_calls(
            std::slice::from_ref(&snapshot),
            &request,
            analyzed.ir(),
            &mut Budget::new(limits),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            Error::BudgetExceeded {
                dimension: jarde_reader::budget::BudgetDimension::DependencyDepth,
                ..
            }
        ));

        let (snapshot, request, analyzed) = fixture();
        let cancellation = jarde_reader::budget::CancellationToken::new();
        cancellation.cancel();
        let error = prove_interface_super_calls(
            std::slice::from_ref(&snapshot),
            &request,
            analyzed.ir(),
            &mut Budget::with_cancellation_token(budget().limits().clone(), cancellation),
        )
        .unwrap_err();
        assert!(matches!(error, Error::Cancelled { .. }));
    }

    #[test]
    fn unique_source_default_rejects_multiple_inherited_defaults() {
        let (nodes, by_name, graph) = conflicting_default_closure();
        assert!(
            !unique_source_default(
                b"proof/BothDefault",
                b"value",
                b"()I",
                &nodes,
                &by_name,
                &graph,
                &mut budget(),
            )
            .unwrap()
        );
    }
}

#[cfg(test)]
mod enum_constant_body_relation_tests {
    use super::*;
    use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
    use std::{
        fs,
        io::{Cursor, Write},
        process::Command,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    const OP: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-25/enum-constant-specific-body/Op.java"
    );
    const MIXED: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-25/enum-constant-specific-body/Mixed.java"
    );
    const PLAIN: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-25/enum-constant-specific-body/Plain.java"
    );
    const STAGE: &str =
        include_str!("../openspec/evidence/java-syntax-2026-09-22/enum-declaration/Stage.java");
    const MEASURE: &str = include_str!(
        "../openspec/evidence/java-syntax-2026-09-22/enum-declaration/user-static-boundary/Measure.java"
    );
    const UNRELATED: &str = "package demo; public class Other { public static Object make() { return new Object() {}; } }";

    fn budget() -> Budget {
        let mut limits = crate::task_budget(&[]).unwrap().limits().clone();
        limits.input_bytes = u64::MAX;
        limits.archive_entries = u64::MAX;
        limits.entry_bytes = u64::MAX;
        limits.read_bytes = u64::MAX;
        limits.class_bytes = u64::MAX;
        limits.attribute_bytes = u64::MAX;
        limits.code_bytes = u64::MAX;
        limits.class_headers = u64::MAX;
        limits.method_bodies = u64::MAX;
        limits.ir_items = u64::MAX;
        limits.ir_edges = u64::MAX;
        limits.analysis_steps = u64::MAX;
        limits.output_bytes = u64::MAX;
        Budget::new(limits)
    }

    fn compiled_entries(debug: bool) -> Vec<(Vec<u8>, Vec<u8>)> {
        compiled_entries_with_op(debug, OP)
    }

    fn compiled_entries_with_op(debug: bool, op_source: &str) -> Vec<(Vec<u8>, Vec<u8>)> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "jarde-enum-body-{}-{nonce}-{}",
            std::process::id(),
            NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(dir.join("source")).unwrap();
        fs::create_dir_all(dir.join("classes")).unwrap();
        for (name, source) in [
            ("Op.java", op_source),
            ("Mixed.java", MIXED),
            ("Plain.java", PLAIN),
            ("Other.java", UNRELATED),
            ("Stage.java", STAGE),
            ("Measure.java", MEASURE),
        ] {
            fs::write(dir.join("source").join(name), source).unwrap();
        }
        let compile = Command::new("javac")
            .args(["--release", "8"])
            .arg(if debug { "-g" } else { "-g:none" })
            .arg("-d")
            .arg(dir.join("classes"))
            .args(["-sourcepath"])
            .arg(dir.join("source"))
            .args(["-d"])
            .arg(dir.join("classes"))
            .arg(dir.join("source/Op.java"))
            .arg(dir.join("source/Mixed.java"))
            .arg(dir.join("source/Plain.java"))
            .arg(dir.join("source/Other.java"))
            .arg(dir.join("source/Stage.java"))
            .arg(dir.join("source/Measure.java"))
            .output()
            .expect("javac is available for the frozen enum evidence");
        assert!(
            compile.status.success(),
            "javac failed: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let mut entries = Vec::new();
        for entry in fs::read_dir(dir.join("classes/demo")).unwrap() {
            let path = entry.unwrap().path();
            entries.push((
                format!("demo/{}", path.file_name().unwrap().to_string_lossy()).into_bytes(),
                fs::read(path).unwrap(),
            ));
        }
        for name in ["Stage.class", "Measure.class"] {
            entries.push((
                name.as_bytes().to_vec(),
                fs::read(dir.join("classes").join(name)).unwrap(),
            ));
        }
        fs::remove_dir_all(dir).unwrap();
        entries
    }

    fn jar(entries: &[(Vec<u8>, Vec<u8>)]) -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        let mut zip = ZipArchiveWriter::new(&mut output);
        for (name, bytes) in entries {
            let (mut entry, config) = zip
                .new_file(EntryPath::verbatim(name.clone()))
                .compression_method(CompressionMethod::new(0))
                .start()
                .unwrap();
            let mut writer = config.wrap(&mut entry);
            writer.write_all(bytes).unwrap();
            let (_, descriptor) = writer.finish().unwrap();
            entry.finish(descriptor).unwrap();
        }
        zip.finish().unwrap();
        output.into_inner()
    }

    fn report_with_budget(
        entries: &[(Vec<u8>, Vec<u8>)],
        class: &str,
        mut budget: Budget,
    ) -> ClassSourceReport {
        let engine = Engine::new();
        let snapshot = engine
            .open(ArtifactInput::bytes(jar(entries)), &mut budget)
            .unwrap();
        let environment = EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::PlainJar,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: jarde_reader::view::LoaderId("app".to_owned()),
        };
        match engine
            .class_source(
                std::slice::from_ref(&snapshot),
                &ClassSourceRequest {
                    class: ClassRef::Name {
                        class: ClassNameQuery::internal(class),
                    },
                    environment,
                },
                &mut budget,
            )
            .unwrap()
        {
            OperationOutcome::Performed(report) => report,
            other => panic!("class source did not select one enum definition: {other:?}"),
        }
    }

    fn report(entries: &[(Vec<u8>, Vec<u8>)], class: &str) -> ClassSourceReport {
        report_with_budget(entries, class, budget())
    }

    fn run_enum_source(
        directory: &std::path::Path,
        label: &str,
        enum_name: &str,
        enum_source: Option<&str>,
        probe_source: &str,
        entries: &[(Vec<u8>, Vec<u8>)],
        debug: bool,
    ) -> String {
        let root = directory.join(label);
        let source_dir = root.join("source/demo");
        let classes = root.join("classes");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(classes.join("demo")).unwrap();
        let probe_file = source_dir.join("Probe.java");
        fs::write(&probe_file, probe_source).unwrap();
        if enum_source.is_none() {
            for (name, bytes) in entries {
                if name.starts_with(format!("demo/{enum_name}").as_bytes()) {
                    fs::write(classes.join(String::from_utf8_lossy(name).as_ref()), bytes).unwrap();
                }
            }
        }
        let mut javac = Command::new("javac");
        javac
            .arg("--release")
            .arg("8")
            .arg(if debug { "-g" } else { "-g:none" })
            .arg("-d")
            .arg(&classes);
        if let Some(source) = enum_source {
            let enum_file = source_dir.join(format!("{enum_name}.java"));
            fs::write(&enum_file, source).unwrap();
            javac.arg(enum_file);
        } else {
            javac.arg("-cp").arg(&classes);
        }
        let compile = javac.arg(&probe_file).output().unwrap();
        assert!(
            compile.status.success(),
            "{label} {enum_name} javac: {}",
            String::from_utf8_lossy(&compile.stderr)
        );
        let run = Command::new("java")
            .arg("-Xverify:all")
            .arg("-cp")
            .arg(&classes)
            .arg("demo.Probe")
            .output()
            .unwrap();
        assert!(
            run.status.success(),
            "{label} {enum_name} java: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        String::from_utf8(run.stdout).unwrap()
    }

    #[test]
    fn proved_enum_bodies_recompile_and_match_original_and_jadx_behavior() {
        const OP_PROBE: &str = r#"package demo;
public class Probe {
    public static void main(String[] args) {
        Op[] first = Op.values(); Op[] second = Op.values();
        System.out.println("arraysDistinct=" + (first != second));
        System.out.println("instancesStable=" + (first[0] == second[0]));
        for (Op value : first) System.out.println(value.name() + ":" + value.ordinal()
            + ":" + value.tag() + ":" + value.apply(7, 3)
            + ":" + (value.getClass() == Op.class)
            + ":" + value.getDeclaringClass().getSimpleName());
        try { Op.valueOf("MISSING"); System.out.println("missing=accepted"); }
        catch (IllegalArgumentException expected) { System.out.println("missing=IllegalArgumentException"); }
    }
}"#;
        const MIXED_PROBE: &str = r#"package demo;
public class Probe {
    public static void main(String[] args) {
        Mixed[] first = Mixed.values(); Mixed[] second = Mixed.values();
        System.out.println("arraysDistinct=" + (first != second));
        System.out.println("instancesStable=" + (first[0] == second[0]));
        for (Mixed value : first) System.out.println(value.name() + ":" + value.ordinal()
            + ":" + value.value() + ":" + (value.getClass() == Mixed.class)
            + ":" + value.getDeclaringClass().getSimpleName());
        try { Mixed.valueOf("MISSING"); System.out.println("missing=accepted"); }
        catch (IllegalArgumentException expected) { System.out.println("missing=IllegalArgumentException"); }
    }
}"#;
        for debug in [true, false] {
            let entries = compiled_entries(debug);
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let directory = std::env::temp_dir().join(format!(
                "jarde-enum-body-projection-{}-{nonce}-{}",
                std::process::id(),
                NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
            ));
            let mode = if debug { "g" } else { "g-none" };
            for (name, probe, expected) in [
                (
                    "Op",
                    OP_PROBE,
                    "arraysDistinct=true\ninstancesStable=true\nADD:0:ADD:0:10:false:Op\nMULTIPLY:1:MULTIPLY:1:21:false:Op\nmissing=IllegalArgumentException\n",
                ),
                (
                    "Mixed",
                    MIXED_PROBE,
                    "arraysDistinct=true\ninstancesStable=true\nSPECIAL:0:7:false:Mixed\nPLAIN:1:0:true:Mixed\nmissing=IllegalArgumentException\n",
                ),
            ] {
                let original = run_enum_source(
                    &directory,
                    &format!("{name}-original"),
                    name,
                    None,
                    probe,
                    &entries,
                    debug,
                );
                assert_eq!(original, expected);
                let jadx_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
                    "openspec/evidence/java-syntax-2026-09-25/enum-constant-specific-body/outputs/{mode}/jadx-source/demo/{name}.java"
                ));
                let jadx_source = fs::read_to_string(jadx_path).unwrap();
                let jadx = run_enum_source(
                    &directory,
                    &format!("{name}-jadx"),
                    name,
                    Some(&jadx_source),
                    probe,
                    &entries,
                    debug,
                );
                let jarde_source = report(&entries, &format!("demo/{name}")).text;
                let jarde = run_enum_source(
                    &directory,
                    &format!("{name}-jarde"),
                    name,
                    Some(&jarde_source),
                    probe,
                    &entries,
                    debug,
                );
                assert_eq!(jadx, original, "{name}, debug={debug}");
                assert_eq!(jarde, original, "{name}, debug={debug}");
            }
            fs::remove_dir_all(directory).unwrap();
        }
    }

    fn mutate_anonymous_outer_index(bytes: &[u8], child: &[u8], outer: &[u8]) -> Vec<u8> {
        let mut read_budget = budget();
        let facts = class_member_facts(bytes, &mut read_budget).unwrap();
        let pool = class_constant_pool(bytes, &read_budget).unwrap();
        let shells: Vec<_> = facts
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0 == b"InnerClasses")
            .cloned()
            .collect();
        let context = class_source::read_class_source_assembly_context(
            bytes,
            &shells,
            &pool,
            &mut read_budget,
        )
        .unwrap();
        let row = context
            .inner_classes
            .iter()
            .position(|inner| {
                jarde_reader::classfile::cp_class_name(&pool, inner.class_index)
                    .is_ok_and(|name| name.0 == child)
            })
            .unwrap();
        assert_eq!(context.inner_classes[row].outer_class_index, 0);
        let outer_index = (1..pool.len())
            .find_map(|index| {
                let index = u16::try_from(index).ok()?;
                jarde_reader::classfile::cp_class_name(&pool, index)
                    .ok()
                    .filter(|name| name.0 == outer)
                    .map(|_| index)
            })
            .unwrap();
        let attribute = shells
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"InnerClasses")
            .unwrap();
        let outer_offset = usize::try_from(attribute.content_span.start).unwrap() + 2 + row * 8 + 2;
        let mut changed = bytes.to_vec();
        changed[outer_offset..outer_offset + 2].copy_from_slice(&outer_index.to_be_bytes());
        changed
    }

    fn mutate_method_flags(
        bytes: &[u8],
        target_name: &[u8],
        target_descriptor: &[u8],
        clear: u16,
    ) -> Vec<u8> {
        let mut changed = bytes.to_vec();
        let cp_count = u16::from_be_bytes([changed[8], changed[9]]) as usize;
        let mut offset = 10usize;
        let mut utf8 = vec![None; cp_count];
        let mut index = 1usize;
        while index < cp_count {
            let tag = changed[offset];
            offset += 1;
            match tag {
                1 => {
                    let length =
                        u16::from_be_bytes([changed[offset], changed[offset + 1]]) as usize;
                    offset += 2;
                    utf8[index] = Some(changed[offset..offset + length].to_vec());
                    offset += length;
                }
                3 | 4 => offset += 4,
                5 | 6 => {
                    offset += 8;
                    index += 1;
                }
                7 | 8 | 16 | 19 | 20 => offset += 2,
                9 | 10 | 11 | 12 | 17 | 18 => offset += 4,
                15 => offset += 3,
                _ => panic!("unknown constant-pool tag {tag}"),
            }
            index += 1;
        }
        offset += 6;
        let interfaces = u16::from_be_bytes([changed[offset], changed[offset + 1]]) as usize;
        offset += 2 + interfaces * 2;
        let fields = u16::from_be_bytes([changed[offset], changed[offset + 1]]) as usize;
        offset += 2;
        for _ in 0..fields {
            offset = skip_member(&changed, offset);
        }
        let methods = u16::from_be_bytes([changed[offset], changed[offset + 1]]) as usize;
        offset += 2;
        for _ in 0..methods {
            let flags_offset = offset;
            let flags = u16::from_be_bytes([changed[offset], changed[offset + 1]]);
            let name_index =
                u16::from_be_bytes([changed[offset + 2], changed[offset + 3]]) as usize;
            let descriptor_index =
                u16::from_be_bytes([changed[offset + 4], changed[offset + 5]]) as usize;
            let member_name = utf8[name_index].as_deref().unwrap();
            let member_descriptor = utf8[descriptor_index].as_deref().unwrap();
            if member_name == target_name && member_descriptor == target_descriptor {
                changed[flags_offset..flags_offset + 2]
                    .copy_from_slice(&(flags & !clear).to_be_bytes());
                return changed;
            }
            offset = skip_member(&changed, offset);
        }
        panic!("target method was absent from the class file");
    }

    fn skip_member(bytes: &[u8], mut offset: usize) -> usize {
        offset += 6;
        let attributes = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
        offset += 2;
        for _ in 0..attributes {
            let length =
                u32::from_be_bytes(bytes[offset + 2..offset + 6].try_into().unwrap()) as usize;
            offset += 6 + length;
        }
        offset
    }

    fn mutate_initializer_constructor_owner(
        bytes: &[u8],
        constructor_bci: u32,
        new_owner: &[u8],
    ) -> Vec<u8> {
        let mut read_budget = budget();
        let facts = class_member_facts(bytes, &mut read_budget).unwrap();
        let pool = class_constant_pool(bytes, &read_budget).unwrap();
        let method_ref = pool
            .iter()
            .find(|entry| {
                matches!(&entry.kind,
                    jarde_reader::classfile::CpEntryKind::MethodRef { owner, name, descriptor, .. }
                        if owner.0 == new_owner && name.0 == b"<init>" && descriptor.0 == b"(Ljava/lang/String;I)V")
            })
            .expect("the enum private constructor reference exists")
            .index;
        let clinit = facts
            .methods
            .iter()
            .find(|method| method.name.raw().0 == b"<clinit>")
            .unwrap();
        let code = clinit
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"Code")
            .unwrap();
        let code_offset = usize::try_from(code.content_span.start).unwrap() + 8;
        let operand_offset = code_offset + constructor_bci as usize + 1;
        let mut changed = bytes.to_vec();
        changed[operand_offset..operand_offset + 2].copy_from_slice(&method_ref.to_be_bytes());
        changed
    }

    fn mutate_initializer_new_owner(bytes: &[u8], new_bci: u32, new_owner: &[u8]) -> Vec<u8> {
        let mut read_budget = budget();
        let facts = class_member_facts(bytes, &mut read_budget).unwrap();
        let pool = class_constant_pool(bytes, &read_budget).unwrap();
        let class_index = pool
            .iter()
            .find(|entry| {
                matches!(&entry.kind,
                    jarde_reader::classfile::CpEntryKind::Class { name, .. }
                        if name.0 == new_owner)
            })
            .expect("the other anonymous child class reference exists")
            .index;
        let clinit = facts
            .methods
            .iter()
            .find(|method| method.name.raw().0 == b"<clinit>")
            .unwrap();
        let code = clinit
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"Code")
            .unwrap();
        let operand_offset =
            usize::try_from(code.content_span.start).unwrap() + 8 + new_bci as usize + 1;
        let mut changed = bytes.to_vec();
        changed[operand_offset..operand_offset + 2].copy_from_slice(&class_index.to_be_bytes());
        changed
    }

    fn mutate_initializer_byte(bytes: &[u8], bci: u32, byte: u8) -> Vec<u8> {
        let mut read_budget = budget();
        let facts = class_member_facts(bytes, &mut read_budget).unwrap();
        let code = facts
            .methods
            .iter()
            .find(|method| method.name.raw().0 == b"<clinit>")
            .unwrap()
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"Code")
            .unwrap();
        let mut changed = bytes.to_vec();
        changed[usize::try_from(code.content_span.start).unwrap() + 8 + bci as usize] = byte;
        changed
    }

    fn mutate_constructor_byte(bytes: &[u8], descriptor: &[u8], bci: u32, byte: u8) -> Vec<u8> {
        let mut read_budget = budget();
        let facts = class_member_facts(bytes, &mut read_budget).unwrap();
        let code = facts
            .methods
            .iter()
            .find(|method| {
                method.name.raw().0 == b"<init>" && method.descriptor.raw().0 == descriptor
            })
            .unwrap()
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"Code")
            .unwrap();
        let mut changed = bytes.to_vec();
        changed[usize::try_from(code.content_span.start).unwrap() + 8 + bci as usize] = byte;
        changed
    }

    fn rename_single_utf8(bytes: &[u8], old: &[u8], new: &[u8]) -> Vec<u8> {
        assert_eq!(old.len(), new.len());
        let mut encoded = u16::try_from(old.len()).unwrap().to_be_bytes().to_vec();
        encoded.extend_from_slice(old);
        let mut changed = bytes.to_vec();
        let positions: Vec<_> = changed
            .windows(encoded.len())
            .enumerate()
            .filter_map(|(index, bytes)| (bytes == encoded).then_some(index))
            .collect();
        assert_eq!(positions.len(), 1);
        changed[positions[0] + 2..positions[0] + 2 + old.len()].copy_from_slice(new);
        changed
    }

    fn mutate_constructor_call_target(
        bytes: &[u8],
        descriptor: &[u8],
        bci: u32,
        target_owner: &[u8],
    ) -> Vec<u8> {
        let mut read_budget = budget();
        let facts = class_member_facts(bytes, &mut read_budget).unwrap();
        let pool = class_constant_pool(bytes, &read_budget).unwrap();
        let target = pool
            .iter()
            .find(|entry| {
                matches!(&entry.kind,
                jarde_reader::classfile::CpEntryKind::MethodRef { owner, name, descriptor, .. }
                    if owner.0 == target_owner && name.0 == b"<init>"
                        && descriptor.0 == b"(Ljava/lang/String;I)V")
            })
            .unwrap();
        let code = facts
            .methods
            .iter()
            .find(|method| {
                method.name.raw().0 == b"<init>" && method.descriptor.raw().0 == descriptor
            })
            .unwrap()
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"Code")
            .unwrap();
        let offset = usize::try_from(code.content_span.start).unwrap() + 8 + bci as usize + 1;
        let mut changed = bytes.to_vec();
        changed[offset..offset + 2].copy_from_slice(&target.index.to_be_bytes());
        changed
    }

    fn mutate_initializer_name_argument(bytes: &[u8], argument_bci: u32, name: &[u8]) -> Vec<u8> {
        let read_budget = budget();
        let pool = class_constant_pool(bytes, &read_budget).unwrap();
        let string_index = pool
            .iter()
            .find(|entry| {
                matches!(&entry.kind,
                    jarde_reader::classfile::CpEntryKind::String { value, .. }
                        if value.0 == name)
            })
            .unwrap()
            .index;
        assert!(string_index <= u16::from(u8::MAX));
        mutate_initializer_byte(bytes, argument_bci + 1, string_index as u8)
    }

    fn mutate_values_factory_field(bytes: &[u8], field_bci: u32, field_name: &[u8]) -> Vec<u8> {
        let mut read_budget = budget();
        let facts = class_member_facts(bytes, &mut read_budget).unwrap();
        let pool = class_constant_pool(bytes, &read_budget).unwrap();
        let field_index = pool
            .iter()
            .find(|entry| {
                matches!(&entry.kind,
                    jarde_reader::classfile::CpEntryKind::FieldRef { owner, name, .. }
                        if owner.0 == b"demo/Op" && name.0 == field_name)
            })
            .unwrap()
            .index;
        let code = facts
            .methods
            .iter()
            .find(|method| method.name.raw().0 == b"$values")
            .unwrap()
            .attributes
            .iter()
            .find(|attribute| attribute.name.raw().0 == b"Code")
            .unwrap();
        let offset = usize::try_from(code.content_span.start).unwrap() + 8 + field_bci as usize + 1;
        let mut changed = bytes.to_vec();
        changed[offset..offset + 2].copy_from_slice(&field_index.to_be_bytes());
        changed
    }

    fn mutate_child_super_call_owner(bytes: &[u8], new_owner: &[u8]) -> Vec<u8> {
        let read_budget = budget();
        let pool = class_constant_pool(bytes, &read_budget).unwrap();
        let class_index = pool
            .iter()
            .find(|entry| {
                matches!(&entry.kind,
                    jarde_reader::classfile::CpEntryKind::Class { name, .. }
                        if name.0 == new_owner)
            })
            .expect("the marker child class reference exists")
            .index;
        let call = pool
            .iter()
            .find(|entry| {
                matches!(&entry.kind,
                    jarde_reader::classfile::CpEntryKind::MethodRef { owner, name, .. }
                        if owner.0 == b"demo/Op" && name.0 == b"<init>")
            })
            .expect("the child constructor's super call exists");
        let offset = usize::try_from(call.span.start).unwrap() + 1;
        let mut changed = bytes.to_vec();
        changed[offset..offset + 2].copy_from_slice(&class_index.to_be_bytes());
        changed
    }

    fn assert_positive(entries: &[(Vec<u8>, Vec<u8>)], debug: bool) {
        for (class, expected) in [("demo/Op", 2), ("demo/Mixed", 1), ("demo/Plain", 0)] {
            let source_report = report(entries, class);
            assert_eq!(
                source_report.enum_constant_body_relations.len(),
                expected,
                "{class}, debug={debug}"
            );
            for relation in &source_report.enum_constant_body_relations {
                let edge = relation.constructor_bridge.as_ref().unwrap();
                assert_eq!(edge.caller.owner, relation.subclass);
                assert_eq!(edge.call_bci, 4);
                assert_eq!(edge.target_owner, class.as_bytes());
                assert_eq!(
                    relation
                        .group_shape
                        .constructor_chain
                        .as_ref()
                        .unwrap()
                        .len(),
                    2
                );
                assert_eq!(
                    relation
                        .group_shape
                        .direct_constant_bcis
                        .as_ref()
                        .unwrap()
                        .len(),
                    usize::from(class == "demo/Mixed")
                );
                assert_eq!(relation.use_census.scans.len(), 1, "{class}, debug={debug}");
                let scan = &relation.use_census.scans[0];
                assert!(!scan.has_more);
                assert_eq!(
                    scan.coverage.dimensions.artifact_structural.state,
                    CoverageState::CompleteWithinSchema
                );
                assert_eq!(scan.coverage.unknown_candidates, 0);
                assert!(scan.coverage.unsupported_categories.is_empty());
                assert!(matches!(scan.execution, ExecutionReport::Complete { .. }));
                assert!(scan.items.iter().any(|item| {
                    matches!(&item.source.location, Location::Code { bci, .. }
                        if *bci == relation.allocation_bci)
                        && item.operation == jarde_query::query::XrefOperation::New
                }));
                assert!(scan.items.iter().any(|item| {
                    matches!(&item.source.location, Location::Code { bci, .. }
                        if *bci == relation.constructor_bci)
                        && item.operation == jarde_query::query::XrefOperation::InvokeSpecial
                }));
                // The unique access constructor marker and the selected sibling's unique
                // anonymous InnerClasses row are compiler structure, not a second allocation.
                assert!(relation.use_census.exclusive, "{class}, debug={debug}");
            }
            if expected == 0 {
                assert!(matches!(
                    source_report.enum_constant_proof,
                    crate::enum_constants::ClassSourceEnumConstantProof::Refused { .. }
                ));
            } else {
                let crate::enum_constants::ClassSourceEnumConstantProof::Proved(
                    crate::enum_constants::ProvedEnumConstantGroup::Body(group),
                ) = &source_report.enum_constant_proof
                else {
                    panic!(
                        "the complete body group must prove: {class}: {:?}",
                        source_report.enum_constant_proof
                    );
                };
                assert_eq!(group.constants.len(), 2);
                assert_eq!(
                    group
                        .constants
                        .iter()
                        .filter(|constant| constant.subclass.is_some())
                        .count(),
                    expected
                );
                assert_eq!(
                    group
                        .constants
                        .iter()
                        .map(|constant| constant.field_index)
                        .collect::<Vec<_>>(),
                    vec![0, 1]
                );
                assert_eq!(
                    group
                        .constants
                        .iter()
                        .map(|constant| constant.constructor_bci)
                        .collect::<Vec<_>>(),
                    vec![7, 20]
                );
            }
            match class {
                "demo/Op" => {
                    assert!(
                        source_report.text.contains("ADD {")
                            && source_report.text.contains("MULTIPLY {")
                    );
                }
                "demo/Mixed" => {
                    assert!(
                        source_report.text.contains("SPECIAL {")
                            && source_report.text.contains("    PLAIN;")
                    );
                }
                _ => assert!(!source_report.text.contains("READY {")),
            }
            assert_eq!(
                source_report
                    .text
                    .matches("selected enum child definition")
                    .count(),
                expected,
                "{class}, debug={debug}"
            );
            if expected == 0 {
                let constructors: Vec<_> = source_report
                    .methods
                    .iter()
                    .filter(|method| method.item.name.raw().0 == b"<init>")
                    .collect();
                assert_eq!(constructors.len(), 1, "{class}, debug={debug}");
                assert_eq!(
                    constructors[0].item.descriptor.raw().0,
                    b"(Ljava/lang/String;I)V"
                );
                assert!(constructors[0].enum_constructor_no_arg_source_signature);
                assert_eq!(constructors[0].item.access_flags & 0x1000, 0);
            } else {
                let first_shape = &source_report.enum_constant_body_relations[0].group_shape;
                assert_eq!(
                    first_shape.initializer_prefix,
                    Ok(PendingEnumConstantBodyInitializerPrefix {
                        constant_field_write_bcis: vec![10, 23],
                        values_factory_element_bcis: vec![6, 12],
                        values_factory_call_bci: 26,
                        values_field_write_bci: 29,
                        prefix_end_bci: 32,
                    }),
                    "exact <clinit> prefix for {class}, debug={debug}"
                );
                assert_eq!(
                    first_shape.enum_access_flags & 0x0400 != 0,
                    class == "demo/Op"
                );
                assert_eq!(first_shape.constants.len(), 2);
                assert_eq!(
                    first_shape
                        .constants
                        .iter()
                        .map(|constant| constant.field_name.as_slice())
                        .collect::<Vec<_>>(),
                    if class == "demo/Op" {
                        vec![b"ADD".as_slice(), b"MULTIPLY".as_slice()]
                    } else {
                        vec![b"SPECIAL".as_slice(), b"PLAIN".as_slice()]
                    }
                );
                assert_eq!(
                    first_shape
                        .constants
                        .iter()
                        .map(|constant| (constant.field_index, constant.expected_ordinal))
                        .collect::<Vec<_>>(),
                    vec![(0, 0), (1, 1)]
                );
                assert!(first_shape.constants.iter().all(|constant| {
                    constant.descriptor_source_argument_count == 0
                        && constant.constructor_descriptor == b"(Ljava/lang/String;I)V"
                        && constant.constructor_owner == constant.allocation_owner
                }));
                assert_eq!(first_shape.constructors.len(), 2);
                let access_constructors: Vec<_> = first_shape
                    .constructors
                    .iter()
                    .filter(|constructor| constructor.access_flags == 0x1000)
                    .collect();
                assert_eq!(access_constructors.len(), 1);
                assert!(access_constructors[0].has_code);
                let marker_owner =
                    enum_access_constructor_marker_owner(&access_constructors[0].descriptor)
                        .expect("the access constructor descriptor has one object marker");
                assert_eq!(
                    access_constructors[0].access_marker_owner.as_deref(),
                    Some(marker_owner.as_slice())
                );
                assert!(
                    source_report
                        .enum_constant_body_relations
                        .iter()
                        .any(|relation| relation.subclass_owner == marker_owner)
                );
                assert_eq!(first_shape.abstract_methods.is_empty(), class != "demo/Op");
                if class == "demo/Op" {
                    assert_eq!(first_shape.abstract_methods.len(), 1);
                    assert_eq!(first_shape.abstract_methods[0].name, b"apply");
                    assert!(!first_shape.abstract_methods[0].has_code);
                }
                assert!(first_shape.implicit_members.iter().any(|member| {
                    member.name == b"$VALUES" && member.access_flags == 0x101a && !member.has_code
                }));
                for member_name in [b"values".as_slice(), b"valueOf", b"$values", b"<clinit>"] {
                    assert!(
                        first_shape
                            .implicit_members
                            .iter()
                            .any(|member| { member.name == member_name && member.has_code })
                    );
                }
                assert!(
                    source_report
                        .enum_constant_body_relations
                        .iter()
                        .all(|relation| relation.group_shape == *first_shape)
                );
            }
            assert!(
                serde_json::to_value(&source_report)
                    .unwrap()
                    .get("enum_constant_body_relations")
                    .is_none()
            );
            let mut relation_tuples: Vec<_> = source_report
                .enum_constant_body_relations
                .iter()
                .map(|relation| {
                    (
                        relation.field_index,
                        relation.allocation_bci,
                        relation.constructor_bci,
                        relation.subclass_owner.as_slice(),
                    )
                })
                .collect();
            relation_tuples.sort_unstable();
            let expected_tuples: Vec<(u64, u32, u32, &[u8])> = match class {
                "demo/Op" => vec![(0, 0, 7, b"demo/Op$1"), (1, 13, 20, b"demo/Op$2")],
                "demo/Mixed" => vec![(0, 0, 7, b"demo/Mixed$1")],
                _ => Vec::new(),
            };
            assert_eq!(
                relation_tuples, expected_tuples,
                "relation mapping for {class}"
            );
            if expected > 0 {
                for relation in &source_report.enum_constant_body_relations {
                    let bodies = relation.body_proof.as_ref().expect("child body proof ran");
                    let methods = bodies
                        .as_ref()
                        .unwrap_or_else(|reason| panic!("{class}: {reason}"));
                    assert_eq!(methods.len(), 1);
                    assert_eq!(methods[0].item.identity.owner, relation.subclass);
                    assert_eq!(
                        methods[0].item.name.raw().0,
                        if class == "demo/Op" {
                            b"apply".as_slice()
                        } else {
                            b"value".as_slice()
                        }
                    );
                    assert!(complete_enum_child_override(&methods[0]));
                    let mut fallback = methods[0].clone();
                    if let class_source::ClassSourceOutcome::Recovered { report, .. } =
                        &mut fallback.outcome
                    {
                        report.quality = Quality::Fallback;
                        report.fallbacks.push("jre_test_fallback");
                    }
                    assert!(!complete_enum_child_override(&fallback));
                }
            }
            assert!(
                source_report
                    .enum_constant_body_relations
                    .iter()
                    .all(|relation| {
                        relation.constructor_descriptor == b"(Ljava/lang/String;I)V"
                    })
            );
            let definitions: std::collections::HashSet<_> = source_report
                .enum_constant_body_relations
                .iter()
                .map(|relation| &relation.subclass)
                .collect();
            assert_eq!(
                definitions.len(),
                expected,
                "selected definitions for {class}"
            );
        }
        for class in ["Stage", "Measure"] {
            let complete = report(entries, class);
            assert!(matches!(
                complete.enum_constant_proof,
                crate::enum_constants::ClassSourceEnumConstantProof::Proved(_)
            ));
            assert!(complete.enum_constant_body_relations.is_empty());
            let mut limits = budget().limits().clone();
            limits.analysis_steps = complete.usage.analysis_steps;
            let bounded = report_with_budget(entries, class, Budget::new(limits));
            assert!(matches!(
                bounded.execution,
                ExecutionReport::Complete { .. }
            ));
            let mut bounded_usage = bounded.usage;
            bounded_usage.elapsed_millis = complete.usage.elapsed_millis;
            assert_eq!(bounded_usage, complete.usage, "budget usage for {class}");
            assert!(bounded.enum_constant_body_relations.is_empty());
        }
    }

    #[test]
    fn proved_body_group_shares_selected_methods_and_their_physical_origins() {
        for debug in [true, false] {
            let entries = compiled_entries(debug);
            for (class, children) in [("demo/Op", 2), ("demo/Mixed", 1)] {
                let parent = report(&entries, class);
                let crate::enum_constants::ClassSourceEnumConstantProof::Proved(
                    crate::enum_constants::ProvedEnumConstantGroup::Body(group),
                ) = &parent.enum_constant_proof
                else {
                    panic!("the complete {class} body group must prove");
                };
                assert_eq!(group.constants.len(), 2);
                let main_code_count = parent
                    .methods
                    .iter()
                    .filter(|method| {
                        matches!(
                            method.item.body,
                            crate::MemberBodyEvidence::CodeAttribute { .. }
                        )
                    })
                    .count() as u64;
                // Each selected child pays once for the typed constructor edge, once for its
                // constructor recovery, and once per retained override. Group assembly pays zero.
                let child_body_count: u64 = group
                    .constants
                    .iter()
                    .filter_map(|item| {
                        item.methods
                            .as_ref()
                            .map(|methods| 2 + methods.len() as u64)
                    })
                    .sum();
                assert_eq!(
                    parent.usage.method_bodies,
                    main_code_count + child_body_count
                );
                assert_eq!(
                    group
                        .constants
                        .iter()
                        .filter(|item| item.subclass.is_some())
                        .count(),
                    children
                );
                let shape = &parent.enum_constant_body_relations[0].group_shape;
                let prefix = shape.initializer_prefix.as_ref().unwrap();
                for (position, item) in group.constants.iter().enumerate() {
                    let candidate = &shape.constants[position];
                    assert_eq!(item.field_index, candidate.field_index);
                    assert_eq!(item.allocation_bci, candidate.allocation_bci);
                    assert_eq!(item.constructor_bci, candidate.constructor_bci);
                    assert_eq!(
                        item.field_write_bci,
                        prefix.constant_field_write_bcis[position]
                    );
                    let relation = parent
                        .enum_constant_body_relations
                        .iter()
                        .find(|relation| relation.field_index == item.field_index);
                    match (&item.subclass, &item.methods, relation) {
                        (None, None, None) => {}
                        (Some(subclass), Some(methods), Some(relation)) => {
                            assert_eq!(subclass, &relation.subclass);
                            let retained = relation.body_proof.as_ref().unwrap().as_ref().unwrap();
                            assert!(std::sync::Arc::ptr_eq(methods, retained));
                            assert_eq!(methods.len(), 1);
                            for method in methods.iter() {
                                assert_eq!(method.item.identity.owner, *subclass);
                                let class_source::ClassSourceOutcome::Recovered { report, .. } =
                                    &method.outcome
                                else {
                                    panic!("the selected override must retain its recovered Code");
                                };
                                assert!(report.source_map.segments().iter().any(|segment| {
                                    segment.origin().primary().method()
                                        == Some(&method.item.identity)
                                }));
                            }
                        }
                        other => panic!("proved constant and relation disagree: {other:?}"),
                    }
                }
            }
        }
    }

    #[test]
    fn enum_body_projection_is_atomic_when_the_second_body_cannot_be_written() {
        let entries = compiled_entries(false);
        let parent = report(&entries, "demo/Op");
        let crate::enum_constants::ClassSourceEnumConstantProof::Proved(
            crate::enum_constants::ProvedEnumConstantGroup::Body(mut group),
        ) = parent.enum_constant_proof.clone()
        else {
            panic!("the source must have a proved body group");
        };
        let shape = &parent.enum_constant_body_relations[0].group_shape;
        let second = group.constants[1].methods.as_ref().unwrap();
        let mut broken = second.to_vec();
        broken[0].text = "incomplete method".to_owned();
        group.constants[1].methods = Some(broken.into());
        assert!(
            class_source::prepare_enum_constant_body_source_projection(
                parent.declaration.as_ref().unwrap(),
                &parent.fields,
                &parent.methods,
                &group,
                shape,
                &mut budget(),
            )
            .unwrap()
            .is_none()
        );
        let physical = class_source::source_text(
            parent.declaration.as_ref().unwrap(),
            &parent.fields,
            &parent.methods,
            &class_source::ClassSourceTextContext {
                initializer_field_order: None,
                declared_methods: parent.methods.len() as u64,
                member_table: None,
                execution: &parent.execution,
                enum_projection: None,
                array_helper_indices: None,
                array_method_texts: None,
                array_helper_markers: None,
            },
        );
        assert!(!physical.contains("ADD {"));
        assert!(!physical.contains("MULTIPLY {"));
        assert!(!physical.contains("selected enum child definition"));
        assert!(physical.contains(parent.fields[0].declaration.as_ref().unwrap()));
        assert!(physical.contains(parent.fields[1].declaration.as_ref().unwrap()));
        assert!(
            physical.contains(
                &parent
                    .methods
                    .iter()
                    .find(|method| method.item.name.raw().0 == b"<init>")
                    .unwrap()
                    .text
            )
        );
        // The first body was already staged locally when the second failed. No caller-visible
        // projection exists, and neither the parent members nor the child Code was rewritten.
        assert!(parent.text.contains("ADD {"));
        assert!(parent.text.contains("MULTIPLY {"));
        assert!(
            parent
                .fields
                .iter()
                .any(|field| field.item.name.raw().0 == b"ADD")
        );
        assert!(
            parent
                .methods
                .iter()
                .any(|method| method.item.name.raw().0 == b"<init>")
        );
        for relation in &parent.enum_constant_body_relations {
            let child = relation.body_proof.as_ref().unwrap().as_ref().unwrap();
            assert!(matches!(
                child[0].outcome,
                class_source::ClassSourceOutcome::Recovered { .. }
            ));
        }
    }

    #[test]
    fn enum_body_projection_keeps_physical_json_and_independent_child_requests() {
        for debug in [true, false] {
            let entries = compiled_entries(debug);
            let parent = report(&entries, "demo/Op");
            let crate::enum_constants::ClassSourceEnumConstantProof::Proved(
                crate::enum_constants::ProvedEnumConstantGroup::Body(group),
            ) = &parent.enum_constant_proof
            else {
                panic!("the source must have a proved body group");
            };
            let json = serde_json::to_value(&parent).unwrap();
            assert_eq!(
                json["fields"].as_array().unwrap().len(),
                parent.fields.len()
            );
            assert_eq!(
                json["methods"].as_array().unwrap().len(),
                parent.methods.len()
            );
            for constant in &group.constants {
                let field = &parent.fields[constant.field_index as usize];
                assert_eq!(
                    json["fields"][constant.field_index as usize]["item"]["identity"],
                    serde_json::to_value(&field.item.identity).unwrap()
                );
                let child_id = constant.subclass.as_ref().unwrap();
                assert!(parent.text.contains(&format!(
                    "// jarde: selected enum child definition: {child_id:?}"
                )));
                let method = &constant.methods.as_ref().unwrap()[0];
                let child = report(
                    &entries,
                    if constant.field_index == group.constants[0].field_index {
                        "demo/Op$1"
                    } else {
                        "demo/Op$2"
                    },
                );
                assert_eq!(&child.class, child_id);
                let independent = child
                    .methods
                    .iter()
                    .find(|item| item.item.identity == method.item.identity)
                    .unwrap();
                let class_source::ClassSourceOutcome::Recovered {
                    report: child_code, ..
                } = &independent.outcome
                else {
                    panic!("the independent child class source retains Code");
                };
                let class_source::ClassSourceOutcome::Recovered { report, .. } = &method.outcome
                else {
                    panic!("the selected child method has Code");
                };
                assert_eq!(child_code.text, report.text);
                let method_json = serde_json::to_value(method).unwrap();
                assert_eq!(
                    method_json["item"]["identity"],
                    serde_json::to_value(&method.item.identity).unwrap()
                );
                assert_eq!(
                    method_json["outcome"]["report"]["source_map"],
                    serde_json::to_value(&report.source_map).unwrap()
                );
                assert!(!report.source_map.segments().is_empty());

                let engine = Engine::new();
                let mut method_budget = budget();
                let snapshot = engine
                    .open(ArtifactInput::bytes(jar(&entries)), &mut method_budget)
                    .unwrap();
                let request = MethodOperationRequest {
                    method: MethodRef::Method {
                        method: method.item.identity.clone(),
                    },
                    environment: EnvironmentRequest {
                        snapshot: snapshot.id().clone(),
                        scope: PhysicalScope::SnapshotAll,
                        policy: EnvironmentPolicy::PlainJar,
                        profile: RuntimeProfile {
                            java_release: 8,
                            multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
                            layout: LayoutMode::Generic,
                        },
                        loader: jarde_reader::view::LoaderId("app".to_owned()),
                    },
                };
                let OperationOutcome::Performed(only) = engine
                    .recover_target_with_evidence(
                        std::slice::from_ref(&snapshot),
                        &request,
                        &RecoveryEvidenceRequest::all(),
                        &mut method_budget,
                    )
                    .unwrap()
                else {
                    panic!("the selected child method must remain independently recoverable");
                };
                assert_eq!(only.method, method.item.identity);
                assert_eq!(only.recovered.recovery().text, report.text);
                assert_eq!(only.recovered.recovery().source_map, report.source_map);
            }
        }
    }

    #[test]
    fn enum_body_projection_budget_and_cancellation_leave_no_half_body() {
        let entries = compiled_entries(false);
        let complete = report(&entries, "demo/Op");
        for dimension in ["output", "read", "ir"] {
            let mut limits = budget().limits().clone();
            match dimension {
                "output" => limits.output_bytes = complete.usage.output_bytes - 1,
                "read" => limits.read_bytes = complete.usage.read_bytes - 1,
                "ir" => limits.ir_items = complete.usage.ir_items - 1,
                _ => unreachable!(),
            }
            let stopped = report_with_budget(&entries, "demo/Op", Budget::new(limits));
            assert!(
                matches!(
                    stopped.enum_constant_proof,
                    crate::enum_constants::ClassSourceEnumConstantProof::Stopped { .. }
                ),
                "{dimension}"
            );
            assert!(!stopped.text.contains("ADD {"), "{dimension}");
            assert!(!stopped.text.contains("MULTIPLY {"), "{dimension}");
            assert!(
                !stopped.text.contains("selected enum child definition"),
                "{dimension}"
            );
            if dimension == "output" {
                assert_eq!(stopped.fields, complete.fields);
                assert_eq!(stopped.methods.len(), complete.methods.len());
                for (left, right) in stopped.methods.iter().zip(&complete.methods) {
                    assert_eq!(left.item, right.item);
                    assert_eq!(left.text, right.text);
                }
            }
        }

        let crate::enum_constants::ClassSourceEnumConstantProof::Proved(
            crate::enum_constants::ProvedEnumConstantGroup::Body(group),
        ) = &complete.enum_constant_proof
        else {
            panic!("the complete input has a proved group");
        };
        let cancellation = jarde_reader::budget::CancellationToken::new();
        let mut cancelled =
            Budget::with_cancellation_token(budget().limits().clone(), cancellation.clone());
        cancellation.cancel();
        assert!(matches!(
            class_source::prepare_enum_constant_body_source_projection(
                complete.declaration.as_ref().unwrap(),
                &complete.fields,
                &complete.methods,
                group,
                &complete.enum_constant_body_relations[0].group_shape,
                &mut cancelled,
            ),
            Err(Error::Cancelled { .. })
        ));
    }

    #[test]
    fn enum_constant_body_relations_follow_each_verified_construction_and_reject_missing_or_changed_rows()
     {
        let entries = compiled_entries(true);
        assert_positive(&entries, true);
        let op_first = b"demo/Op$1.class".to_vec();
        let mut missing = entries.clone();
        missing.retain(|(name, _)| name != &op_first);
        let missing_report = report(&missing, "demo/Op");
        assert_eq!(missing_report.enum_constant_body_relations.len(), 1);
        assert!(!missing_report.text.contains("ADD {"));

        let mut ambiguous = entries.clone();
        let duplicate = ambiguous
            .iter()
            .find(|(name, _)| name == &op_first)
            .unwrap()
            .clone();
        ambiguous.push(duplicate);
        let ambiguous_report = report(&ambiguous, "demo/Op");
        assert_eq!(ambiguous_report.enum_constant_body_relations.len(), 1);
        assert!(!ambiguous_report.text.contains("ADD {"));

        let mut changed = entries.clone();
        let child = changed
            .iter_mut()
            .find(|(name, _)| name == &op_first)
            .unwrap();
        child.1 = mutate_anonymous_outer_index(&child.1, b"demo/Op$1", b"demo/Op");
        let changed_report = report(&changed, "demo/Op");
        assert_eq!(changed_report.enum_constant_body_relations.len(), 1);
        assert!(!changed_report.text.contains("ADD {"));

        let mut incomplete_member = entries.clone();
        let op = incomplete_member
            .iter_mut()
            .find(|(name, _)| name == b"demo/Op.class")
            .unwrap();
        op.1 = mutate_method_flags(&op.1, b"$values", b"()[Ldemo/Op;", 0x1000);
        let incomplete_report = report(&incomplete_member, "demo/Op");
        assert!(incomplete_report.enum_constant_body_relations.is_empty());
        assert!(matches!(
            incomplete_report.enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Refused { .. }
        ));

        let mut wrong_owner = entries.clone();
        let op = wrong_owner
            .iter_mut()
            .find(|(name, _)| name == b"demo/Op.class")
            .unwrap();
        op.1 = mutate_initializer_constructor_owner(&op.1, 7, b"demo/Op");
        let wrong_owner_report = report(&wrong_owner, "demo/Op");
        assert!(wrong_owner_report.enum_constant_body_relations.is_empty());
        assert!(matches!(
            wrong_owner_report.enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Stopped { .. }
        ));

        let complete = report(&entries, "demo/Op");
        let mut limits = budget().limits().clone();
        limits.analysis_steps = complete.usage.analysis_steps.saturating_sub(1);
        let bounded = report_with_budget(&entries, "demo/Op", Budget::new(limits));
        assert!(!matches!(
            bounded.enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Proved(_)
        ));
        assert!(matches!(
            bounded.enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Stopped { .. }
        ));

        assert_positive(&compiled_entries(false), false);
    }

    #[test]
    fn enum_body_group_requires_each_abstract_implementation_and_keeps_physical_child() {
        for debug in [true, false] {
            let mut entries = compiled_entries(debug);
            let child = entries
                .iter_mut()
                .find(|(name, _)| name == b"demo/Op$2.class")
                .unwrap();
            child.1 = rename_single_utf8(&child.1, b"apply", b"aplly");
            let parent = report(&entries, "demo/Op");
            assert!(matches!(
                parent.enum_constant_proof,
                crate::enum_constants::ClassSourceEnumConstantProof::Refused { .. }
            ));
            assert_eq!(parent.enum_constant_body_relations.len(), 2);
            assert!(matches!(&parent.enum_constant_body_relations[1].body_proof,
                Some(Err(reason)) if reason.contains("base declaration")));
            assert!(
                parent
                    .methods
                    .iter()
                    .any(|method| method.item.name.raw().0 == b"apply"
                        && matches!(method.outcome, class_source::ClassSourceOutcome::NoBody))
            );
            let child = report(&entries, "demo/Op$2");
            assert!(
                child
                    .methods
                    .iter()
                    .any(|method| method.item.name.raw().0 == b"aplly"
                        && matches!(
                            method.outcome,
                            class_source::ClassSourceOutcome::Recovered { .. }
                        ))
            );
        }
    }

    #[test]
    fn enum_body_group_rejects_a_changed_generated_values_helper() {
        for debug in [true, false] {
            let mut entries = compiled_entries(debug);
            let main = entries
                .iter_mut()
                .find(|(name, _)| name == b"demo/Op.class")
                .unwrap();
            main.1 = rename_single_utf8(&main.1, b"clone", b"cloen");
            let parent = report(&entries, "demo/Op");
            assert!(matches!(&parent.enum_constant_proof,
                crate::enum_constants::ClassSourceEnumConstantProof::Refused { reason }
                    if reason.contains("generated enum API")));
            assert_eq!(parent.enum_constant_body_relations.len(), 2);
            assert!(parent.enum_constant_body_relations.iter().all(|relation| {
                relation.use_census.exclusive
                    && matches!(&relation.body_proof, Some(Ok(methods)) if methods.len() == 1)
            }));
            assert!(
                parent
                    .methods
                    .iter()
                    .any(|method| method.item.name.raw().0 == b"values")
            );
        }
    }

    #[test]
    fn enum_child_body_proof_rejects_fields_effects_and_unspellable_overrides() {
        const FIELD: &str = "package demo; public enum Op { ADD { int capture; public int apply(int a, int b) { return a + b; } }, MULTIPLY { public int apply(int a, int b) { return a * b; } }; public abstract int apply(int a, int b); }";
        let field = compiled_entries_with_op(false, FIELD);
        let field_report = report(&field, "demo/Op");
        let first = field_report
            .enum_constant_body_relations
            .iter()
            .find(|relation| relation.subclass_owner == b"demo/Op$1")
            .unwrap();
        assert!(first.use_census.exclusive);
        assert!(matches!(&first.body_proof, Some(Err(reason)) if reason.contains("field")));
        assert!(matches!(
            field_report.enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Refused { .. }
        ));

        const EFFECT: &str = "package demo; public enum Op { ADD { { System.nanoTime(); } public int apply(int a, int b) { return a + b; } }, MULTIPLY { public int apply(int a, int b) { return a * b; } }; public abstract int apply(int a, int b); }";
        let effect = compiled_entries_with_op(false, EFFECT);
        let effect_report = report(&effect, "demo/Op");
        let first = effect_report
            .enum_constant_body_relations
            .iter()
            .find(|relation| relation.subclass_owner == b"demo/Op$1")
            .unwrap();
        assert!(first.constructor_bridge.is_err());
        assert!(first.body_proof.is_none());
        assert!(matches!(
            effect_report.enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Refused { .. }
        ));

        let mut illegal = compiled_entries(false);
        let child = illegal
            .iter_mut()
            .find(|(name, _)| name == b"demo/Op$1.class")
            .unwrap();
        child.1 = mutate_method_flags(&child.1, b"apply", b"(II)I", 0x0001);
        let illegal_report = report(&illegal, "demo/Op");
        let first = illegal_report
            .enum_constant_body_relations
            .iter()
            .find(|relation| relation.subclass_owner == b"demo/Op$1")
            .unwrap();
        assert!(matches!(&first.body_proof, Some(Err(reason)) if reason.contains("override")));
        assert!(matches!(
            illegal_report.enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Refused { .. }
        ));
    }

    #[test]
    fn enum_child_exceptional_body_cannot_pass_full_recovery() {
        const EXCEPTIONAL: &str = "package demo; public enum Op { ADD { public int apply(int a, int b) { try { return a + b; } catch (RuntimeException e) { return -1; } } }, MULTIPLY { public int apply(int a, int b) { return a * b; } }; public abstract int apply(int a, int b); }";
        let entries = compiled_entries_with_op(false, EXCEPTIONAL);
        let report = report(&entries, "demo/Op");
        let first = report
            .enum_constant_body_relations
            .iter()
            .find(|relation| relation.subclass_owner == b"demo/Op$1")
            .unwrap();
        assert!(
            matches!(&first.body_proof, Some(Err(_))),
            "{:#?}",
            first.body_proof
        );
        assert!(matches!(
            report.enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Refused { .. }
        ));
    }

    #[test]
    fn enum_child_body_recovery_is_evidence_independent_and_budgeted() {
        let entries = compiled_entries(false);
        let essential = report(&entries, "demo/Op");
        assert!(
            essential
                .enum_constant_body_relations
                .iter()
                .all(|relation| {
                    matches!(&relation.body_proof, Some(Ok(methods)) if methods.len() == 1)
                })
        );
        let engine = Engine::new();
        let mut all_budget = budget();
        let snapshot = engine
            .open(ArtifactInput::bytes(jar(&entries)), &mut all_budget)
            .unwrap();
        let request = ClassSourceRequest {
            class: ClassRef::Name {
                class: ClassNameQuery::internal("demo/Op"),
            },
            environment: EnvironmentRequest {
                snapshot: snapshot.id().clone(),
                scope: PhysicalScope::SnapshotAll,
                policy: EnvironmentPolicy::PlainJar,
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: jarde_reader::view::MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                loader: jarde_reader::view::LoaderId("app".to_owned()),
            },
        };
        let OperationOutcome::Performed(all) = engine
            .class_source_with_evidence(
                std::slice::from_ref(&snapshot),
                &request,
                &RecoveryEvidenceRequest::all(),
                &mut all_budget,
            )
            .unwrap()
        else {
            panic!("the selected enum is unique")
        };
        assert_eq!(all.text, essential.text);
        assert!(all.text.contains("selected enum child definition"));
        for (left, right) in essential
            .enum_constant_body_relations
            .iter()
            .zip(&all.enum_constant_body_relations)
        {
            assert_eq!(left.subclass, right.subclass);
            let left = left.body_proof.as_ref().unwrap().as_ref().unwrap();
            let right = right.body_proof.as_ref().unwrap().as_ref().unwrap();
            assert_eq!(left[0].item.identity, right[0].item.identity);
            assert_eq!(left[0].text, right[0].text);
        }
        let mut limits = budget().limits().clone();
        limits.method_bodies = essential.usage.method_bodies.saturating_sub(1);
        let bounded = report_with_budget(&entries, "demo/Op", Budget::new(limits));
        assert!(matches!(
            bounded.enum_constant_proof,
            crate::enum_constants::ClassSourceEnumConstantProof::Stopped { .. }
        ));
        assert!(!bounded.text.contains("ADD {"));

        let mut read_budget = budget();
        let op_bytes = &entries
            .iter()
            .find(|(name, _)| name == b"demo/Op.class")
            .unwrap()
            .1;
        let op_facts = class_member_facts(op_bytes, &mut read_budget).unwrap();
        let cancellation = jarde_reader::budget::CancellationToken::new();
        let mut cancelled =
            Budget::with_cancellation_token(budget().limits().clone(), cancellation.clone());
        cancellation.cancel();
        let result = prove_enum_constant_child_body(
            &engine,
            std::slice::from_ref(&snapshot),
            &request,
            &essential.enum_constant_body_relations[0],
            &op_facts.methods,
            &mut cancelled,
        );
        assert!(matches!(
            result,
            Err(Error::Cancelled { .. }) | Ok((Err(_), ExecutionReport::Cancelled { .. }))
        ));
    }

    #[test]
    fn enum_constant_body_census_rejects_another_constant_and_other_class_code_use() {
        let entries = compiled_entries(true);
        let mut reused = entries.clone();
        let op = &mut reused
            .iter_mut()
            .find(|(name, _)| name == b"demo/Op.class")
            .unwrap()
            .1;
        *op = mutate_initializer_new_owner(op, 13, b"demo/Op$1");
        *op = mutate_initializer_constructor_owner(op, 20, b"demo/Op$1");
        let reused_report = report(&reused, "demo/Op");
        assert_eq!(reused_report.enum_constant_body_relations.len(), 2);
        assert!(
            reused_report
                .enum_constant_body_relations
                .iter()
                .all(|relation| !relation.use_census.exclusive)
        );

        let mut other_code = entries;
        let child = &mut other_code
            .iter_mut()
            .find(|(name, _)| name == b"demo/Op$2.class")
            .unwrap()
            .1;
        *child = mutate_child_super_call_owner(child, b"demo/Op$1");
        let other_report = report(&other_code, "demo/Op");
        let first = other_report
            .enum_constant_body_relations
            .iter()
            .find(|relation| relation.subclass_owner == b"demo/Op$1")
            .expect("the selected child relation survives a change to its Code");
        let changed_child = other_report
            .enum_constant_body_relations
            .iter()
            .find(|relation| relation.subclass_owner == b"demo/Op$2")
            .unwrap();
        assert!(changed_child.constructor_bridge.is_err());
        assert!(!first.use_census.exclusive);
        assert!(first.use_census.scans[0].items.iter().any(|item| {
            matches!(&item.source.location, Location::Code { method, .. }
                if method.owner != first.subclass && method.owner != other_report.class)
                && item.operation == jarde_query::query::XrefOperation::InvokeSpecial
        }));

        let mut changed_sibling = compiled_entries(false);
        let sibling = &mut changed_sibling
            .iter_mut()
            .find(|(name, _)| name == b"demo/Op$2.class")
            .unwrap()
            .1;
        *sibling = mutate_anonymous_outer_index(sibling, b"demo/Op$1", b"demo/Op");
        let changed_report = report(&changed_sibling, "demo/Op");
        let first = changed_report
            .enum_constant_body_relations
            .iter()
            .find(|relation| relation.subclass_owner == b"demo/Op$1")
            .unwrap();
        assert!(!first.use_census.exclusive);
        assert!(first.use_census.scans[0].items.iter().any(|item| {
            item.operation == jarde_query::query::XrefOperation::InnerClass
                && matches!(&item.source.location, Location::ClassOffset { definition, .. }
                    if definition != &first.subclass && definition != &changed_report.class)
        }));
    }

    #[test]
    fn enum_constructor_edges_require_exact_forwarding_and_targets() {
        const BASE: &[u8] = b"(Ljava/lang/String;I)V";
        const ACCESS: &[u8] = b"(Ljava/lang/String;ILdemo/Op$1;)V";
        for debug in [true, false] {
            let entries = compiled_entries(debug);
            let good = report(&entries, "demo/Op");
            assert_eq!(good.enum_constant_body_relations.len(), 2);
            assert!(good.enum_constant_body_relations.iter().all(|relation| {
                relation.group_shape.constructor_chain.is_ok()
                    && relation.constructor_bridge.is_ok()
            }));

            let plain = entries
                .iter()
                .find(|(name, _)| name == b"demo/Plain.class")
                .unwrap();
            let plain_report = report(&entries, "demo/Plain");
            assert!(plain_report.enum_constant_body_relations.is_empty());
            let prove_plain = |bytes: &[u8]| {
                let mut read_budget = budget();
                let facts = class_member_facts(bytes, &mut read_budget).unwrap();
                let pool = class_constant_pool(bytes, &read_budget).unwrap();
                let header = facts
                    .methods
                    .iter()
                    .find(|method| {
                        method.name.raw().0 == b"<init>" && method.descriptor.raw().0 == BASE
                    })
                    .unwrap();
                prove_enum_physical_constructor(
                    bytes,
                    header,
                    member_identity(&plain_report.class, header),
                    &pool,
                    b"java/lang/Enum",
                    BASE,
                    false,
                    &mut read_budget,
                )
                .unwrap()
            };
            let direct = prove_plain(&plain.1).unwrap();
            assert_eq!(direct.call_bci, 3);
            assert_eq!(direct.target_owner, b"java/lang/Enum");
            assert!(prove_plain(&mutate_constructor_byte(&plain.1, BASE, 2, 0x04)).is_err());
            assert!(
                prove_plain(&mutate_constructor_call_target(
                    &plain.1,
                    BASE,
                    3,
                    b"demo/Plain",
                ))
                .is_err()
            );

            let mixed = report(&entries, "demo/Mixed");
            assert_eq!(mixed.enum_constant_body_relations.len(), 1);
            assert_eq!(
                mixed.enum_constant_body_relations[0]
                    .group_shape
                    .direct_constant_bcis,
                Ok(vec![20]),
            );
            let mut changed_mixed_constructor = entries.clone();
            let mixed_bytes = &mut changed_mixed_constructor
                .iter_mut()
                .find(|(name, _)| name == b"demo/Mixed.class")
                .unwrap()
                .1;
            *mixed_bytes = mutate_constructor_byte(mixed_bytes, BASE, 2, 0x04);
            let changed_mixed_report = report(&changed_mixed_constructor, "demo/Mixed");
            assert_eq!(changed_mixed_report.enum_constant_body_relations.len(), 1);
            assert!(
                changed_mixed_report.enum_constant_body_relations[0]
                    .group_shape
                    .constructor_chain
                    .is_err()
            );
            assert!(
                changed_mixed_report.enum_constant_body_relations[0]
                    .group_shape
                    .direct_constant_bcis
                    .is_err()
            );

            // BCI 20 is the ordinary constant's invokespecial in Mixed's physical <clinit>.
            // It must call Mixed.<init>, even though the first constant uses Mixed$1.
            let mut changed_mixed_target = entries.clone();
            let mixed_bytes = &mut changed_mixed_target
                .iter_mut()
                .find(|(name, _)| name == b"demo/Mixed.class")
                .unwrap()
                .1;
            *mixed_bytes = mutate_initializer_constructor_owner(mixed_bytes, 20, b"demo/Mixed$1");
            let changed_mixed_target_report = report(&changed_mixed_target, "demo/Mixed");
            assert!(
                changed_mixed_target_report
                    .enum_constant_body_relations
                    .is_empty()
            );

            for (bci, opcode) in [(2, 0x04), (0, 0x2d), (6, 0xbf)] {
                let mut changed = entries.clone();
                let op = &mut changed
                    .iter_mut()
                    .find(|(name, _)| name == b"demo/Op.class")
                    .unwrap()
                    .1;
                *op = mutate_constructor_byte(op, ACCESS, bci, opcode);
                let changed_report = report(&changed, "demo/Op");
                assert!(
                    changed_report
                        .enum_constant_body_relations
                        .iter()
                        .all(|relation| { relation.group_shape.constructor_chain.is_err() }),
                    "bridge edit at {bci}, debug={debug}"
                );
            }

            let mut changed_target = entries.clone();
            let op = &mut changed_target
                .iter_mut()
                .find(|(name, _)| name == b"demo/Op.class")
                .unwrap()
                .1;
            *op = mutate_constructor_call_target(op, ACCESS, 3, b"java/lang/Enum");
            let target_report = report(&changed_target, "demo/Op");
            assert!(
                target_report
                    .enum_constant_body_relations
                    .iter()
                    .all(|relation| { relation.group_shape.constructor_chain.is_err() }),
                "wrong bridge target, debug={debug}"
            );

            let mut changed_child = entries.clone();
            let child = &mut changed_child
                .iter_mut()
                .find(|(name, _)| name == b"demo/Op$1.class")
                .unwrap()
                .1;
            *child = mutate_constructor_byte(child, BASE, 3, 0x2c);
            let child_report = report(&changed_child, "demo/Op");
            let first = child_report
                .enum_constant_body_relations
                .iter()
                .find(|relation| relation.subclass_owner == b"demo/Op$1")
                .unwrap();
            assert!(
                first.constructor_bridge.is_err(),
                "non-null marker, debug={debug}"
            );

            let op = entries
                .iter()
                .find(|(name, _)| name == b"demo/Op.class")
                .unwrap();
            let mutated_private = mutate_constructor_byte(&op.1, BASE, 2, 0x04);
            let mut changed_private = entries.clone();
            changed_private
                .iter_mut()
                .find(|(name, _)| name == b"demo/Op.class")
                .unwrap()
                .1 = mutated_private;
            let private_report = report(&changed_private, "demo/Op");
            assert!(
                private_report
                    .enum_constant_body_relations
                    .iter()
                    .all(|relation| { relation.group_shape.constructor_chain.is_err() }),
                "private constructor changed ordinal, debug={debug}"
            );
        }
    }

    #[test]
    fn enum_child_constructor_read_preserves_budget_and_cancellation_stops() {
        let entries = compiled_entries(false);
        let child = entries
            .iter()
            .find(|(name, _)| name == b"demo/Op$1.class")
            .unwrap();
        let selected = report(&entries, "demo/Op").enum_constant_body_relations[0]
            .subclass
            .clone();
        let mut read_budget = budget();
        let facts = class_member_facts(&child.1, &mut read_budget).unwrap();
        let pool = class_constant_pool(&child.1, &read_budget).unwrap();
        let header = facts
            .methods
            .iter()
            .find(|method| method.name.raw().0 == b"<init>")
            .unwrap();
        let identity = member_identity(&selected, header);
        let mut limits = budget().limits().clone();
        limits.code_bytes = 3;
        let mut bounded = Budget::new(limits);
        assert!(matches!(
            prove_enum_physical_constructor(
                &child.1,
                header,
                identity.clone(),
                &pool,
                b"demo/Op",
                b"(Ljava/lang/String;ILdemo/Op$1;)V",
                true,
                &mut bounded,
            ),
            Err(Error::BudgetExceeded { .. })
        ));
        let cancellation = jarde_reader::budget::CancellationToken::new();
        let mut cancelled =
            Budget::with_cancellation_token(budget().limits().clone(), cancellation.clone());
        cancellation.cancel();
        assert!(matches!(
            prove_enum_physical_constructor(
                &child.1,
                header,
                identity,
                &pool,
                b"demo/Op",
                b"(Ljava/lang/String;ILdemo/Op$1;)V",
                true,
                &mut cancelled,
            ),
            Err(Error::Cancelled { .. })
        ));
    }

    #[test]
    fn enum_body_prefix_reads_actual_name_and_ordinal_and_allows_closed_user_suffix() {
        for debug in [true, false] {
            let entries = compiled_entries(debug);
            for (bci, byte, expected) in [(4, None, "name"), (6, Some(0x04), "ordinal")] {
                let mut changed = entries.clone();
                let op = &mut changed
                    .iter_mut()
                    .find(|(name, _)| name == b"demo/Op.class")
                    .unwrap()
                    .1;
                *op = if let Some(opcode) = byte {
                    mutate_initializer_byte(op, bci, opcode)
                } else {
                    mutate_initializer_name_argument(op, bci, b"MULTIPLY")
                };
                let changed_report = report(&changed, "demo/Op");
                assert_eq!(changed_report.enum_constant_body_relations.len(), 2);
                assert!(
                    changed_report
                        .enum_constant_body_relations
                        .iter()
                        .all(|relation| relation.group_shape.initializer_prefix.is_err()),
                    "equal-width {expected} edit, debug={debug}"
                );
                assert!(!matches!(
                    changed_report.enum_constant_proof,
                    crate::enum_constants::ClassSourceEnumConstantProof::Proved(_)
                ));
            }

            let mut changed_factory = entries.clone();
            let op = &mut changed_factory
                .iter_mut()
                .find(|(name, _)| name == b"demo/Op.class")
                .unwrap()
                .1;
            *op = mutate_values_factory_field(op, 6, b"MULTIPLY");
            let changed_report = report(&changed_factory, "demo/Op");
            assert_eq!(changed_report.enum_constant_body_relations.len(), 2);
            assert!(
                changed_report
                    .enum_constant_body_relations
                    .iter()
                    .all(|relation| relation.group_shape.initializer_prefix.is_err()),
                "equal-width $values() element edit, debug={debug}"
            );

            let source = OP.replace(
                "    public abstract int apply",
                "    static int initialized; static { initialized = 5; }\n    public abstract int apply",
            );
            let with_suffix = compiled_entries_with_op(debug, &source);
            let suffix_report = report(&with_suffix, "demo/Op");
            assert_eq!(suffix_report.enum_constant_body_relations.len(), 2);
            assert!(
                suffix_report
                    .enum_constant_body_relations
                    .iter()
                    .all(|relation| relation.group_shape.initializer_prefix.is_ok()),
                "closed user static suffix, debug={debug}"
            );
        }
    }

    #[test]
    fn enum_body_prefix_rejects_extra_alias_store_and_unclosed_suffix() {
        use crate::enum_constants::{
            EnumCodeInstruction, EnumCodeReference, EnumMethodCodeCandidate,
        };

        let entries = compiled_entries(true);
        let shape = report(&entries, "demo/Op").enum_constant_body_relations[0]
            .group_shape
            .clone();
        let owner = b"demo/Op";
        let descriptor = b"Ldemo/Op;".to_vec();
        let instruction = |bci, width, opcode, reference| EnumCodeInstruction {
            bci,
            width,
            opcode,
            immediate: None,
            local: None,
            reference,
        };
        let mut instructions = Vec::new();
        for constant in &shape.constants {
            let bci = constant.allocation_bci;
            instructions.extend([
                instruction(
                    bci,
                    3,
                    0xbb,
                    Some(EnumCodeReference::Class(constant.allocation_owner.clone())),
                ),
                instruction(bci + 3, 1, 0x59, None),
                instruction(
                    bci + 4,
                    2,
                    0x12,
                    Some(EnumCodeReference::String(constant.field_name.clone())),
                ),
                instruction(bci + 6, 1, 0x03 + constant.expected_ordinal as u8, None),
                instruction(
                    bci + 7,
                    3,
                    0xb7,
                    Some(EnumCodeReference::Method {
                        owner: constant.constructor_owner.clone(),
                        name: b"<init>".to_vec(),
                        descriptor: constant.constructor_descriptor.clone(),
                        interface: false,
                    }),
                ),
                instruction(
                    bci + 10,
                    3,
                    0xb3,
                    Some(EnumCodeReference::Field {
                        owner: owner.to_vec(),
                        name: constant.field_name.clone(),
                        descriptor: descriptor.clone(),
                    }),
                ),
            ]);
        }
        instructions.extend([
            instruction(
                26,
                3,
                0xb8,
                Some(EnumCodeReference::Method {
                    owner: owner.to_vec(),
                    name: b"$values".to_vec(),
                    descriptor: b"()[Ldemo/Op;".to_vec(),
                    interface: false,
                }),
            ),
            instruction(
                29,
                3,
                0xb3,
                Some(EnumCodeReference::Field {
                    owner: owner.to_vec(),
                    name: b"$VALUES".to_vec(),
                    descriptor: b"[Ldemo/Op;".to_vec(),
                }),
            ),
            instruction(32, 1, 0xb1, None),
        ]);
        let code = EnumMethodCodeCandidate {
            table_index: 0,
            member: None,
            complete: true,
            exception_handler_count: 0,
            instructions,
            member_uses: Vec::new(),
        };
        let mut factory_instructions = vec![
            instruction(0, 1, 0x05, None),
            instruction(1, 3, 0xbd, Some(EnumCodeReference::Class(owner.to_vec()))),
        ];
        for (ordinal, constant) in shape.constants.iter().enumerate() {
            let start = 4 + ordinal as u32 * 6;
            factory_instructions.extend([
                instruction(start, 1, 0x59, None),
                instruction(start + 1, 1, 0x03 + ordinal as u8, None),
                instruction(
                    start + 2,
                    3,
                    0xb2,
                    Some(EnumCodeReference::Field {
                        owner: owner.to_vec(),
                        name: constant.field_name.clone(),
                        descriptor: descriptor.clone(),
                    }),
                ),
                instruction(start + 5, 1, 0x53, None),
            ]);
        }
        factory_instructions.push(instruction(16, 1, 0xb0, None));
        let factory = EnumMethodCodeCandidate {
            table_index: shape.implicit_members[4].table_index,
            member: None,
            complete: true,
            exception_handler_count: 0,
            instructions: factory_instructions,
            member_uses: Vec::new(),
        };
        let prove = |code: &EnumMethodCodeCandidate| {
            prove_enum_body_initializer_prefix(
                code,
                Some(&factory),
                owner,
                &shape.constants,
                &shape.implicit_members[0],
            )
        };
        assert!(prove(&code).is_ok());
        assert!(
            prove_enum_body_initializer_prefix(
                &code,
                None,
                owner,
                &shape.constants,
                &shape.implicit_members[0],
            )
            .is_err()
        );
        let mut incomplete = code.clone();
        incomplete.complete = false;
        assert!(prove(&incomplete).is_err());

        let mut aliased = code.clone();
        aliased
            .instructions
            .insert(5, instruction(10, 1, 0x59, None));
        aliased
            .instructions
            .insert(6, instruction(11, 1, 0x4b, None));
        for later in &mut aliased.instructions[7..] {
            later.bci += 2;
        }
        assert!(
            prove(&aliased).is_err(),
            "dup/astore must not retain the object"
        );

        let mut extra_store = code.clone();
        extra_store
            .instructions
            .insert(5, instruction(10, 1, 0x59, None));
        extra_store.instructions.insert(
            6,
            instruction(
                11,
                3,
                0xb3,
                Some(EnumCodeReference::Field {
                    owner: owner.to_vec(),
                    name: b"extra".to_vec(),
                    descriptor: descriptor.clone(),
                }),
            ),
        );
        for later in &mut extra_store.instructions[7..] {
            later.bci += 4;
        }
        assert!(
            prove(&extra_store).is_err(),
            "dup/putstatic must not add a store"
        );

        let mut unfinished = code.clone();
        unfinished.instructions.pop();
        assert!(prove(&unfinished).is_err());

        let mut suffix_alias = code.clone();
        suffix_alias.instructions.pop();
        suffix_alias.instructions.extend([
            instruction(
                32,
                3,
                0xb2,
                Some(EnumCodeReference::Field {
                    owner: owner.to_vec(),
                    name: b"ADD".to_vec(),
                    descriptor: descriptor.clone(),
                }),
            ),
            instruction(
                35,
                3,
                0xb3,
                Some(EnumCodeReference::Field {
                    owner: owner.to_vec(),
                    name: b"alias".to_vec(),
                    descriptor,
                }),
            ),
            instruction(38, 1, 0xb1, None),
        ]);
        assert!(
            prove(&suffix_alias).is_err(),
            "the suffix must not re-store a constant"
        );

        let mut user_suffix = code;
        user_suffix.instructions.pop();
        user_suffix.instructions.extend([
            instruction(32, 1, 0x08, None),
            instruction(33, 1, 0x57, None),
            instruction(34, 1, 0xb1, None),
        ]);
        assert!(
            prove(&user_suffix).is_ok(),
            "a closed suffix may contain user effects"
        );
    }
}

fn prove_class_source_bridges(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    policy: &EnvironmentPolicy,
    definition: &PhysicalDefinitionId,
    facts: &ClassMemberFacts,
    methods: &[ClassSourceMethod],
    candidates: &[jarde_java::bridge::ClassSourceBridgeCandidate],
    execution: &mut ExecutionReport,
    budget: &mut Budget,
) -> Vec<class_source::ClassSourceBridgeProof> {
    const ACC_PUBLIC: u16 = 0x0001;
    const ACC_PRIVATE: u16 = 0x0002;
    const ACC_STATIC: u16 = 0x0008;
    const ACC_BRIDGE: u16 = 0x0040;
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_NATIVE: u16 = 0x0100;
    const ACC_SYNTHETIC: u16 = 0x1000;
    const RECONSTRUCTIBLE_BRIDGE_FLAGS: u16 = ACC_PUBLIC | ACC_BRIDGE | ACC_SYNTHETIC;

    let class_name = facts.this_class.raw().0.as_slice();
    let mut proofs = Vec::new();
    for candidate in candidates {
        let Some(member) = candidate.member.as_ref() else {
            continue;
        };
        let refuse = |reason: &str| class_source::ClassSourceBridgeProof {
            member: member.clone(),
            target: None,
            call_bci: candidate.call_bci,
            admitted: false,
            projected: false,
            refusal: Some(reason.to_owned()),
        };
        if &member.owner != definition {
            proofs.push(refuse(
                "the sidecar identity belongs to another physical class",
            ));
            continue;
        }
        if facts.stopped_at.is_some() {
            proofs.push(refuse("the physical class method table is incomplete"));
            continue;
        }
        let Some(flags) = candidate.access_flags else {
            proofs.push(refuse("the bridge member flags were not stated"));
            continue;
        };
        if flags != RECONSTRUCTIBLE_BRIDGE_FLAGS {
            proofs.push(refuse(
                "the physical bridge has modifiers beyond public bridge synthetic that source reconstruction does not prove",
            ));
            continue;
        }
        if candidate.has_exception_handlers {
            proofs.push(refuse(
                "the bridge Code declares an exception handler or was unavailable",
            ));
            continue;
        }
        if !candidate.presented || !candidate.pure_forward {
            proofs.push(refuse("bridge@1 did not prove a pure single forward"));
            continue;
        }
        let Some(target) = candidate.target.as_ref() else {
            proofs.push(refuse("bridge@1 did not retain a structured call target"));
            continue;
        };
        let Some(call_bci) = candidate.call_bci else {
            proofs.push(refuse("bridge@1 did not retain the forward call BCI"));
            continue;
        };
        let bridge_headers: Vec<_> = facts
            .methods
            .iter()
            .filter(|header| {
                header.name.raw().0 == member.name.0
                    && header.descriptor.raw().0 == member.descriptor.0
            })
            .collect();
        let [bridge_header] = bridge_headers.as_slice() else {
            proofs.push(refuse("the physical bridge header is missing or ambiguous"));
            continue;
        };
        if bridge_header.access_flags != flags {
            proofs.push(refuse(
                "bridge sidecar flags disagree with the unique physical method header",
            ));
            continue;
        }
        if bridge_header
            .attributes
            .iter()
            .filter(|attribute| attribute.name.raw().0.as_slice() == b"Code")
            .count()
            != 1
        {
            proofs.push(refuse(
                "the physical bridge header does not declare exactly one Code attribute",
            ));
            continue;
        }
        if bridge_header
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0.as_slice() != b"Code")
        {
            proofs.push(refuse(
                "the bridge declares method metadata whose source copying is unproved",
            ));
            continue;
        }
        let bridge_methods: Vec<_> = methods
            .iter()
            .filter(|method| method.item.identity == *member)
            .collect();
        let [bridge_method] = bridge_methods.as_slice() else {
            proofs.push(refuse(
                "the physical bridge has no unique class-source method record",
            ));
            continue;
        };
        let complete_bridge = matches!(
            &bridge_method.outcome,
            class_source::ClassSourceOutcome::Recovered { report, analysis }
                if report.produced()
                    && report.quality == Quality::Structured
                    && report.fallbacks.is_empty()
                    && matches!(&report.execution, ExecutionReport::Complete { .. })
                    && matches!(&analysis.execution, ExecutionReport::Complete { .. })
        );
        if !complete_bridge {
            proofs.push(refuse(
                "the bridge body recovery did not complete as a structured artifact",
            ));
            continue;
        }
        if target.kind() != jarde_java::facts::InvokeKind::Virtual {
            proofs.push(refuse("the verified invocation is not the instance virtual call required for source override"));
            continue;
        }
        if target.owner().as_bytes() != class_name
            || target.name().as_bytes() != member.name.0.as_slice()
            || target.is_interface_reference()
        {
            proofs.push(refuse(
                "the verified invocation does not name this class's source method",
            ));
            continue;
        }
        let Some((bridge_parameters, bridge_return)) =
            method_descriptor_parts(&member.descriptor.0)
        else {
            proofs.push(refuse(
                "the bridge descriptor is not a complete method descriptor",
            ));
            continue;
        };
        let Some((target_parameters, target_return)) =
            method_descriptor_parts(target.descriptor().as_bytes())
        else {
            proofs.push(refuse(
                "the invocation descriptor is not a complete method descriptor",
            ));
            continue;
        };
        if bridge_parameters != target_parameters
            || target.name().as_bytes() != member.name.0.as_slice()
        {
            proofs.push(refuse(
                "the bridge and invoked method do not share a name and parameter descriptor",
            ));
            continue;
        }
        if bridge_return == target_return
            || bridge_return != b"Ljava/lang/Object;"
            || !(target_return.starts_with(b"L") || target_return.starts_with(b"["))
        {
            proofs.push(refuse("the source return type is not a proved covariant subtype of the erased Object return"));
            continue;
        }
        let matching: Vec<_> = methods
            .iter()
            .filter(|method| {
                method.item.identity.name.0 == member.name.0
                    && method.item.identity.descriptor.0.as_slice()
                        == target.descriptor().as_bytes()
            })
            .collect();
        let [source] = matching.as_slice() else {
            proofs.push(refuse(
                "the invoked source method is missing or not unique in the current class",
            ));
            continue;
        };
        let source_headers: Vec<_> = facts
            .methods
            .iter()
            .filter(|header| {
                header.name.raw().0 == source.item.identity.name.0
                    && header.descriptor.raw().0 == source.item.identity.descriptor.0
            })
            .collect();
        let [source_header] = source_headers.as_slice() else {
            proofs.push(refuse("the source method header is missing or ambiguous"));
            continue;
        };
        let source_flags = source.item.access_flags;
        if source_flags != source_header.access_flags {
            proofs.push(refuse(
                "the source record flags disagree with the unique physical method header",
            ));
            continue;
        }
        if source_flags & ACC_BRIDGE != 0
            || source_flags & ACC_PUBLIC == 0
            || source_flags & (ACC_PRIVATE | ACC_STATIC | ACC_ABSTRACT | ACC_NATIVE) != 0
            || source_flags & ACC_SYNTHETIC != 0
            || source.declaration.is_none()
        {
            proofs.push(refuse(
                "the unique target is not a spellable, concrete public source method",
            ));
            continue;
        }
        if source_header
            .attributes
            .iter()
            .any(|attribute| attribute.name.raw().0.as_slice() != b"Code")
            || source_header
                .attributes
                .iter()
                .filter(|attribute| attribute.name.raw().0.as_slice() == b"Code")
                .count()
                != 1
        {
            proofs.push(refuse("the source method metadata could be copied to an elided bridge and is not proved reconstructible"));
            continue;
        }
        let complete_source = matches!(
            &source.outcome,
            class_source::ClassSourceOutcome::Recovered { report, analysis }
                if report.produced()
                    && report.quality == Quality::Structured
                    && report.fallbacks.is_empty()
                    && matches!(&report.execution, ExecutionReport::Complete { .. })
                    && matches!(&analysis.execution, ExecutionReport::Complete { .. })
        );
        if !complete_source {
            proofs.push(refuse(
                "the unique source method did not recover completely",
            ));
            continue;
        }
        let mut inherited = false;
        let mut saw_unresolved = false;
        let mut reloaded_current = false;
        let mut stopped = false;
        let mut direct_supers: Vec<(jarde_reader::model::JvmBytes, ReferenceUse)> = facts
            .interfaces
            .iter()
            .map(|name| (name.raw().clone(), ReferenceUse::InvokeInterface))
            .collect();
        if let Some(super_class) = &facts.super_class {
            direct_supers.push((super_class.raw().clone(), ReferenceUse::InvokeVirtual));
        }
        for (owner, use_kind) in direct_supers {
            if owner.0.as_slice() == class_name {
                saw_unresolved = true;
                continue;
            }
            // Object is the implicit root for every class. It is not evidence that a direct
            // bridge contract exists, and this explicit environment need not carry a JRE image.
            if use_kind == ReferenceUse::InvokeVirtual && owner.0.as_slice() == b"java/lang/Object"
            {
                continue;
            }
            if matches!(policy, EnvironmentPolicy::SingleClass) {
                saw_unresolved = true;
                continue;
            }
            let resolution = match jarde_jvm::resolve_symbol(
                content,
                &ResolutionRequest {
                    environment: environment.clone(),
                    target: jarde_reader::model::SymbolRef::Method {
                        owner: owner.clone(),
                        name: member.name.clone(),
                        descriptor: member.descriptor.clone(),
                    },
                    use_kind,
                    caller: jarde_jvm::environment::CallerContext {
                        loader: environment.runtime.load_domain.loader.clone(),
                        enclosing: None,
                    },
                    dispatch: None,
                },
                budget,
            ) {
                Ok(resolution) => resolution,
                Err(error) => {
                    merge_execution(execution, stop_execution(&error, budget));
                    stopped = true;
                    break;
                }
            };
            merge_execution(execution, resolution.execution.clone());
            if !matches!(&resolution.execution, ExecutionReport::Complete { .. }) {
                stopped = true;
                break;
            }
            reloaded_current |= resolution
                .reads
                .iter()
                .any(|read| &read.definition == definition);
            if resolution.state == Some(ResolutionState::Resolved)
                && matches!(&resolution.execution, ExecutionReport::Complete { .. })
                && resolution.unresolved_dependencies.is_empty()
                && !resolution
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == "resolution_access_not_checked")
                && !reloaded_current
                && resolution.resolved.as_ref().is_some_and(|resolved| {
                    matches!(
                        &resolved.member,
                        jarde_reader::model::SymbolRef::Method { name, descriptor, .. }
                            if name.0.as_slice() == member.name.0.as_slice()
                                && descriptor.0.as_slice() == member.descriptor.0.as_slice()
                    ) && resolution
                        .reads
                        .iter()
                        .any(|read| read.definition == resolved.definition)
                })
            {
                inherited = true;
                break;
            }
            if resolution.state == Some(ResolutionState::UnresolvedDependency)
                || resolution.state == Some(ResolutionState::Missing)
                || resolution.state == Some(ResolutionState::Ambiguous)
                || !resolution.unresolved_dependencies.is_empty()
            {
                saw_unresolved = true;
            }
        }
        if !inherited {
            let reason = if stopped {
                "resolution of a direct parent or interface stopped before it proved the erased method"
            } else if reloaded_current {
                "the existing resolver traversed back into this prepared class; admission is refused"
            } else if saw_unresolved {
                "a direct parent or interface needed for the erased method is unresolved"
            } else {
                "no resolved direct parent or interface requires the erased method descriptor"
            };
            if stopped {
                proofs.clear();
                proofs.push(refuse(reason));
                return proofs;
            }
            proofs.push(refuse(reason));
            continue;
        }
        proofs.push(class_source::ClassSourceBridgeProof {
            member: member.clone(),
            target: Some(source.item.identity.clone()),
            call_bci: Some(call_bci),
            admitted: true,
            projected: false,
            refusal: None,
        });
    }
    proofs
}

fn method_descriptor_parts(descriptor: &[u8]) -> Option<(&[u8], &[u8])> {
    descriptor_facts(descriptor, DescriptorKind::Method).ok()?;
    let close = descriptor.iter().position(|byte| *byte == b')')?;
    Some((&descriptor[..=close], &descriptor[close + 1..]))
}

/// Plans all bridge source replacements without mutating the physical member records. A missing or
/// ambiguous join is a fail-closed class projection: no staged note is committed.
fn stage_class_source_bridge_projections(
    proofs: &[class_source::ClassSourceBridgeProof],
    methods: &[ClassSourceMethod],
) -> Option<Vec<(usize, String)>> {
    let mut staged = Vec::new();
    for proof in proofs.iter().filter(|proof| proof.admitted) {
        let target = proof.target.as_ref()?;
        let call_bci = proof.call_bci?;
        let bridge_matches: Vec<_> = methods
            .iter()
            .enumerate()
            .filter(|(_, method)| method.item.identity == proof.member)
            .collect();
        let [(bridge_index, bridge)] = bridge_matches.as_slice() else {
            return None;
        };
        let target_matches: Vec<_> = methods
            .iter()
            .filter(|method| method.item.identity == *target)
            .collect();
        let [target_method] = target_matches.as_slice() else {
            return None;
        };
        staged.push((
            *bridge_index,
            bridge.bridge_projection_marker(target_method, call_bci),
        ));
    }
    Some(staged)
}

// ---------------------------------------------------------------------------------------------
// Class and member navigation: what an artifact holds, and the identity of what it holds
// ---------------------------------------------------------------------------------------------
//
// Two evidence levels, two report shapes, one identity vocabulary (P0/P1's physical identities).
// An entry whose *raw name* matches the class-name rule is a candidate and nothing more; a class
// whose *header* was really read is a declaration, and only that read may carry `this_class`, the
// class flags and the members. The two are never one record with a flag, because "the path looks
// like a class" and "the bytes declare a class" are different evidence — and because the second is
// the only one a caller may bind an identity to.
//
// One artifact can hold the same class name at several physical origins, and every one of them is
// published: nothing here merges, deduplicates or elects a first. That is why the items carry
// `PhysicalDefinitionId`/`PhysicalMethodId` rather than a display name, and why a caller that
// chooses one can hand that identity back and read exactly that definition.

/// The raw-name rule a class candidate is selected by: a case-sensitive `.class` suffix.
///
/// The rule decides *candidacy* on the entry's own bytes and derives nothing: it is not a name, and
/// an entry that matches it has not been shown to hold a class. It is byte-for-byte the rule
/// `jarde-jvm`'s load-root lookup applies to a composed name, restated here because this layer owns
/// the listing rule and must not import the runtime provider's internals for it.
const CLASS_FILE_SUFFIX: &[u8] = b".class";

/// One item of the class-candidate listing: a stored entry the class-name rule selects, or the
/// ordinary resource that rule leaves beside it.
///
/// The partition is physical and total: every entry the scope enumerated appears exactly once, in
/// the scope's own order, and nothing is renamed, merged or dropped.
///
/// [`ClassListingItem::ClassCandidate`] states where a candidate lives and **nothing else** — that
/// its raw name matched a rule, never that a class was found, that the entry's bytes decode, that
/// the class is loadable or that its path states its name.
///
/// [`ClassListingItem::Resource`] is the remainder of the same partition: an entry no class-name
/// rule presents as a class candidate — a resource, a directory, a nested library, a manifest. It is
/// a *physical* classification and no interpretation: this change does not parse a resource, does
/// not read it and does not claim it was understood. A resource the query side reports as a consumed
/// reference (a Manifest attribute, a `META-INF/services` entry) is that side's finding, not this
/// one's.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClassListingItem {
    /// An entry whose raw name ends in a case-sensitive `.class`, or the root of a standalone
    /// `CLASS` snapshot (which has no entry name to match).
    ClassCandidate { location: PhysicalClassLocation },
    /// A stored entry the class-name rule does not select.
    Resource { entry: PhysicalEntryId },
}

/// The class candidates of one scope and the ordinary resources beside them, read from names alone.
///
/// Every item was charged as one result item. The scope scan's own ranges and stops are the
/// `coverage`, `execution` and `diagnostics`; the candidates were never read, so this report neither
/// charges nor claims a `class_headers` attempt.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassCandidateListing {
    /// The physical view this listing covered.
    pub view: PhysicalView,
    /// The partition, in the scope's own order (container order for a tree, central-directory order
    /// inside one container).
    pub items: Vec<ClassListingItem>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

impl ClassCandidateListing {
    /// The candidate locations, in the scope's own order.
    pub fn candidates(&self) -> impl Iterator<Item = &PhysicalClassLocation> {
        self.items.iter().filter_map(|item| match item {
            ClassListingItem::ClassCandidate { location } => Some(location),
            ClassListingItem::Resource { .. } => None,
        })
    }

    /// The ordinary resources, in the scope's own order.
    pub fn resources(&self) -> impl Iterator<Item = &PhysicalEntryId> {
        self.items.iter().filter_map(|item| match item {
            ClassListingItem::ClassCandidate { .. } => None,
            ClassListingItem::Resource { entry } => Some(entry),
        })
    }
}

/// The class-level facts one header read established.
///
/// These are *parse* facts of the one read that produced them: the name the class declares, its own
/// access flags, its superclass and its declared interfaces, each as the class file states it. They
/// are not a dialect validation, a runtime resolution, a JVM verification or a statement that the
/// class is loadable, and they are not a recovery payload: no method body was read.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassDeclarationFacts {
    /// The internal name the class's own `this_class` states.
    pub this_class: JvmString,
    /// The class's own access flags, as the class file's `access_flags` states them.
    pub access_flags: u16,
    /// The superclass the class declares, or `None` for a class whose `super_class` entry is zero
    /// (the `module-info` shape).
    pub super_class: Option<JvmString>,
    /// The interfaces the class declares, in declaration order.
    pub interfaces: Vec<JvmString>,
}

/// Whether an entry's own raw path and the class's own declaration state the same internal name.
///
/// A path is not a declaration, so the two are published side by side and neither rewrites the
/// other: the item's own `this_class` keeps the declared name, and this value adds what the *path*
/// states. `PathNameDiffers` is not a verdict that either name is wrong and not a claim that the
/// class is unusable — it states that this entry must not be read as "the class the path named".
/// `PathNameAgrees` is not a claim of loadability either: it says this entry is a binding for the
/// name a caller asked under, and nothing more.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClassNameBinding {
    /// The class was read from a standalone root, which has no path to compare with.
    StandaloneRoot,
    /// The entry's raw path states the declared name at a `/` boundary: the path without its
    /// `.class` suffix either *is* that name or ends with `/<name>`.
    ///
    /// The prefix a longer path carries is deliberately **not** claimed: an entry
    /// `WEB-INF/classes/p/S.class` states `p/S` inside whatever layout its container has, and this
    /// change infers no layout, asserts no root and activates no nested library. What this value
    /// says is only that the entry's path states the name the class itself declares — and that a
    /// caller holding that name may read this entry under the prefix it declares for itself.
    PathNameAgrees,
    /// The entry's raw path does not state the declared name at any boundary.
    ///
    /// `path_name` is what the path does state: the raw name without a `.class` suffix, or the whole
    /// raw name when it has no such suffix (`None` in that case would hide the only name the entry
    /// has, so the name is always published). Neither side rewrites the other, and an entry in this
    /// state MUST NOT be read as "the class the path named".
    PathNameDiffers { path_name: JvmBytes },
}

/// What one member's own declaration says about its body, as the same read saw it.
///
/// The `Code` shell is a *declaration*: the attribute is present in the member's attribute table, at
/// the position stated here. Nothing about its content was read, decoded or verified, so this value
/// must not be read as "this member has a body this engine can read" — the body was not read at all,
/// charged no `code_bytes` and was never bounded by `method_bodies`.
///
/// An `abstract` or `native` member is required by the class-file format not to carry a `Code`
/// attribute, so `NoCodeAttribute` is how those members are stated: they are returned as declarations
/// with their name, descriptor and flags, and no empty body is invented for them. A member that
/// carries neither flag and still declares no `Code` is stated the same way — this value reports the
/// bytes, and whether that shape is legal is not this layer's verdict.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MemberBodyEvidence {
    /// The member declares a `Code` attribute whose content occupies `content_span` in the class
    /// file. The content itself was not read.
    CodeAttribute { content_span: ByteSpan },
    /// The member's own attribute table declares no `Code` attribute.
    NoCodeAttribute,
}

/// One class a navigation read read: the physical definition the bytes are, the facts its own header
/// stated, and whether its raw path and its declared name agree.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassDeclarationItem {
    /// The physical definition the bytes are — the identity a later read is addressed by.
    pub definition: PhysicalDefinitionId,
    pub declaration: ClassDeclarationFacts,
    pub binding: ClassNameBinding,
    /// Where the same read's member walk stopped, when a member record did not decode.
    ///
    /// `None` is a member table read to its declared end. A stop does **not** qualify the class
    /// declaration above it: the declaration was read and the class is confirmed, and the members
    /// before the stop are exactly what [`Engine::list_members`] reads and publishes. It is recorded
    /// here because a caller that has this identity should know that the class's member table is a
    /// prefix before it asks for the members.
    pub member_table: Option<MemberTableStop>,
}

/// One field record of a class's own field table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldItem {
    /// The field's position in its own table, counted from zero.
    pub index: u64,
    /// The field's physical identity: the owner definition read and the key its own declaration
    /// states. The raw bytes here are the same read's bytes as `name`/`descriptor` below.
    pub identity: PhysicalMemberId,
    /// The field's own raw name, as the read decoded it.
    pub name: JvmString,
    /// The field's own raw descriptor, as the read decoded it.
    pub descriptor: JvmString,
    pub access_flags: u16,
}

/// One method record of a class's own method table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodItem {
    /// The method's position in its own table, counted from zero.
    pub index: u64,
    /// The method's physical identity — owner definition, raw name and raw descriptor — which a
    /// method request consumes directly. The raw bytes here are the same read's bytes as
    /// `name`/`descriptor` below.
    pub identity: PhysicalMethodId,
    /// The method's own raw name, as the read decoded it.
    pub name: JvmString,
    /// The method's own raw descriptor, as the read decoded it.
    pub descriptor: JvmString,
    pub access_flags: u16,
    pub body: MemberBodyEvidence,
}

/// One thing a class-content navigation found: the class-level facts of the read, a field record or a
/// method record. Each keeps its own kind and its own physical position, and none is folded into
/// another. A resource kind never appears here: a class's own member tables cannot state one, and
/// this vocabulary must not invent what it cannot observe.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClassContentItem {
    ClassDeclaration(ClassDeclarationItem),
    Field(FieldItem),
    Method(MethodItem),
}

/// The class declarations one scope's header-confirmed scan really read.
///
/// `candidates` is how many the raw-name rule selected in the scope, and `unconfirmed` names every
/// one of them that is not in `items`, in scope order: a candidate the listing never read, or the one
/// whose read failed and is named by its own diagnostic. An unread candidate never enters `items` —
/// and no item here is a claim that the class is complete, legal, loadable or verified.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassDeclarationListing {
    pub view: PhysicalView,
    /// How many candidates the class-name rule selected in the scope.
    pub candidates: u64,
    /// The declarations this scan confirmed, in the order it confirmed them.
    pub items: Vec<ClassDeclarationItem>,
    /// Every candidate that is not in `items`, in scope order.
    pub unconfirmed: Vec<PhysicalClassLocation>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

/// The class declaration and member tables one bounded read of one class published.
///
/// The items are the members of that read in the read's own order — the class-level item first, then
/// its fields in declaration order, then its methods — each charged as one result item. The listing
/// reads no body and starts no analysis, so `code_bytes` and `method_bodies` are untouched and
/// nothing here is a recovery, a resolution or a verification statement.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberListing {
    /// The physical definition this read read, exactly as the caller named it.
    pub definition: PhysicalDefinitionId,
    pub items: Vec<ClassContentItem>,
    /// Where the member walk stopped, when a member record did not decode; `None` exactly when both
    /// member tables were read to their declared end.
    pub stopped_at: Option<MemberTableStop>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

impl MemberListing {
    /// The class-level facts of this read, when the class-level item itself was published.
    pub fn declaration(&self) -> Option<&ClassDeclarationItem> {
        self.items.iter().find_map(|item| match item {
            ClassContentItem::ClassDeclaration(class) => Some(class),
            ClassContentItem::Field(_) | ClassContentItem::Method(_) => None,
        })
    }

    /// The field records this read published, in declaration order.
    pub fn fields(&self) -> impl Iterator<Item = &FieldItem> {
        self.items.iter().filter_map(|item| match item {
            ClassContentItem::Field(field) => Some(field),
            ClassContentItem::ClassDeclaration(_) | ClassContentItem::Method(_) => None,
        })
    }

    /// The method records this read published, in declaration order.
    pub fn methods(&self) -> impl Iterator<Item = &MethodItem> {
        self.items.iter().filter_map(|item| match item {
            ClassContentItem::Method(method) => Some(method),
            ClassContentItem::ClassDeclaration(_) | ClassContentItem::Field(_) => None,
        })
    }
}

/// One class name as the caller spells it, and the internal name it denotes.
///
/// Two spellings denote one internal name: the source-style dotted name
/// ([`ClassNameQuery::dotted`], `com.demo.A`) and the class file's own internal name
/// ([`ClassNameQuery::internal`], `com/demo/A`). The derived internal name is what a listing compares
/// `this_class` against, and the spelling is kept only so the report can echo the request: **it is
/// never an identity**, and no candidate is selected because a spelling "looks right".
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassNameQuery {
    spelling: String,
    internal_name: JvmBytes,
}

impl ClassNameQuery {
    /// A source-style dotted class name; every `.` becomes the internal name's `/`.
    pub fn dotted(name: impl Into<String>) -> Self {
        let spelling = name.into();
        Self {
            internal_name: JvmBytes(spelling.replace('.', "/").into_bytes()),
            spelling,
        }
    }

    /// A class file's own internal name, taken byte for byte as UTF-8.
    pub fn internal(name: impl Into<String>) -> Self {
        let spelling = name.into();
        Self {
            internal_name: JvmBytes(spelling.as_bytes().to_vec()),
            spelling,
        }
    }

    /// The name as the caller spelled it.
    pub fn spelling(&self) -> &str {
        &self.spelling
    }

    /// The internal name the spelling denotes.
    pub fn internal_name(&self) -> &JvmBytes {
        &self.internal_name
    }
}

/// Which member records one navigation request asks for.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberQueryKind {
    Methods,
    Fields,
    Both,
}

/// The member filter one navigation request may name.
///
/// A member candidate must carry `name` byte for byte, `descriptor` byte for byte when one is given,
/// and a kind the request admits. A filter that names a method name without a descriptor is the
/// overload case: every declared descriptor is a candidate, and choosing between them is the
/// caller's, on the identity each candidate carries.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberQuery {
    /// The raw member name every candidate must carry.
    pub name: JvmBytes,
    /// The exact raw descriptor when the caller has one; `None` asks for every overload.
    pub descriptor: Option<JvmBytes>,
    pub kind: MemberQueryKind,
}

impl MemberQuery {
    /// Whether one navigation item states this filter: the same raw name, the same raw descriptor
    /// when one was given, and an admitted kind.
    fn matches(&self, item: &ClassContentItem) -> bool {
        match item {
            ClassContentItem::Field(field) => {
                self.admits(false)
                    && field.name.raw().0 == self.name.0
                    && self
                        .descriptor
                        .as_ref()
                        .is_none_or(|descriptor| field.descriptor.raw().0 == descriptor.0)
            }
            ClassContentItem::Method(method) => {
                self.admits(true)
                    && method.name.raw().0 == self.name.0
                    && self
                        .descriptor
                        .as_ref()
                        .is_none_or(|descriptor| method.identity.descriptor.0 == descriptor.0)
            }
            ClassContentItem::ClassDeclaration(_) => false,
        }
    }

    fn admits(&self, method: bool) -> bool {
        match self.kind {
            MemberQueryKind::Methods => method,
            MemberQueryKind::Fields => !method,
            MemberQueryKind::Both => true,
        }
    }
}

/// One navigation lookup: a class in either accepted spelling, optionally narrowed to one member.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationQuery {
    pub class: ClassNameQuery,
    /// The member filter, when the request asks for members rather than for the class itself.
    pub member: Option<MemberQuery>,
}

/// What one navigation name search found in the scope it searched, and what it did not.
///
/// The candidates are the items the search bound to a physical identity, each carrying its own
/// definition (location, class bytes, variant) or its owner definition plus declared name and
/// descriptor. An empty `candidates` list is an answer: the coverage states the range that was
/// searched and a `Complete` execution states that the whole scope was searched, without inventing a
/// definition, a member or a diagnostic. A non-`Complete` execution is the opposite statement — the
/// search ended before the scope did — and the prefix published before that point stays usable.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavigationReport {
    /// The physical view this search covered.
    pub view: PhysicalView,
    /// The request as it was answered, spelling included.
    pub query: NavigationQuery,
    pub candidates: Vec<ClassContentItem>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

impl NavigationReport {
    /// The classes the search bound, in scope order.
    pub fn classes(&self) -> impl Iterator<Item = &ClassDeclarationItem> {
        self.candidates.iter().filter_map(|item| match item {
            ClassContentItem::ClassDeclaration(class) => Some(class),
            ClassContentItem::Field(_) | ClassContentItem::Method(_) => None,
        })
    }

    /// The fields the search bound, in the order their classes were searched.
    pub fn fields(&self) -> impl Iterator<Item = &FieldItem> {
        self.candidates.iter().filter_map(|item| match item {
            ClassContentItem::Field(field) => Some(field),
            ClassContentItem::ClassDeclaration(_) | ClassContentItem::Method(_) => None,
        })
    }

    /// The methods the search bound, in the order their classes were searched.
    pub fn methods(&self) -> impl Iterator<Item = &MethodItem> {
        self.candidates.iter().filter_map(|item| match item {
            ClassContentItem::Method(method) => Some(method),
            ClassContentItem::ClassDeclaration(_) | ClassContentItem::Field(_) => None,
        })
    }
}

/// The physical entries one declared scope holds, with the scope scan's own planes.
///
/// The entries are the reader's own records, in the reader's own order, and the planes are the ones
/// that scan reported: this layer copies nothing and re-derives no coverage.
struct ScopeScan {
    entries: Vec<PhysicalEntry>,
    physical_coverage: Coverage,
    execution: ExecutionReport,
    diagnostics: Vec<Diagnostic>,
}

/// One class candidate of a scope: where it lives, and the entry record a read is addressed with.
///
/// A standalone root has no entry, which is why the record is optional — and why the location, not
/// the record, is what a report publishes.
struct ClassCandidate<'a> {
    location: PhysicalClassLocation,
    record: Option<&'a PhysicalEntry>,
}

/// One confirmed class read: the class-level item, the member facts the *same* read established, the
/// diagnostics that read itself published, and the very bytes it materialized.
///
/// The facts travel with the item because a caller that asks for members derives them from *this*
/// read — one read of the class per request, never a second opinion about the same bytes. The bytes
/// travel with both because a class view decodes the bodies it was asked for out of them instead of
/// reading the same class a second time, and the source travels with them because they are what a
/// preparation is built from (`add-demand-driven-core-results` task 3.1): the consumer that needs a
/// prepared class hands *this* read over instead of reading the same definition again.
struct ConfirmedRead {
    class: ClassContentItem,
    facts: ClassMemberFacts,
    diagnostics: Vec<Diagnostic>,
    bytes: Vec<u8>,
    source: ClassSource,
}

impl ConfirmedRead {
    /// This read in the shape a class task consumes: the very bytes this binding read, stated once
    /// more as the read a preparation is built from.
    ///
    /// Nothing is read or charged here — the materialization already happened, and this is the
    /// handover that keeps it from happening twice (task 3.1). `container` says whether the
    /// preparation's consumers will read the class's container again (a method analysis's loader
    /// binding query) or only decode bodies out of it (a class view).
    fn prepared_read(
        &self,
        snapshot: &ArtifactSnapshot,
        container: jarde_reader::prepared::ContainerHandover,
        budget: &mut Budget,
    ) -> Result<jarde_reader::prepared::PreparedClassRead> {
        snapshot.prepared_read_of(
            self.source.location.clone(),
            self.source.class_bytes.clone(),
            self.bytes.clone(),
            container,
            budget,
        )
    }
}

/// Enumerates the entries one declared scope holds, under the reader's own accounting.
///
/// `SnapshotAll` on a `ZIP` snapshot is the root container's own enumeration; `ArtifactTree` is the
/// explicit bounded walk of the whole tree, whose containers are the ones the walk reached — an upper
/// directory entry and an explicitly expanded nested library are two positions, and both are listed.
///
/// A standalone `CLASS` snapshot holds no entries: its scope is the class itself, and the one
/// candidate the listings publish for it is its root. An artifact-tree scope on such a snapshot is an
/// input error, because a standalone class has no containers to walk.
fn scan_scope(
    snapshot: &ArtifactSnapshot,
    scope: &PhysicalScope,
    budget: &mut Budget,
) -> Result<ScopeScan> {
    match (snapshot.kind(), scope) {
        (ArtifactKind::StandaloneClass, PhysicalScope::SnapshotAll) => Ok(ScopeScan {
            entries: Vec::new(),
            physical_coverage: Coverage {
                artifact_structural: CoverageDimension {
                    state: CoverageState::CompleteWithinSchema,
                    scanned: Vec::new(),
                    skipped: Vec::new(),
                    uninterpreted_extensions: Vec::new(),
                },
                runtime_resolution: CoverageDimension::not_requested(),
                dynamic_analysis: CoverageDimension::not_requested(),
            },
            execution: ExecutionReport::Complete {
                usage: budget.usage(),
            },
            diagnostics: Vec::new(),
        }),
        (ArtifactKind::StandaloneClass, PhysicalScope::ArtifactTree { .. }) => {
            Err(Error::invalid_input(
                "navigation_not_zip",
                "an artifact-tree scope requires a ZIP snapshot; a standalone CLASS snapshot has no containers",
            ))
        }
        (ArtifactKind::Zip, PhysicalScope::SnapshotAll) => match snapshot.enumerate(budget) {
            Ok(report) => Ok(ScopeScan {
                entries: report.entries,
                physical_coverage: report.coverage,
                execution: report.execution,
                diagnostics: report.diagnostics,
            }),
            Err(error) => Ok(ScopeScan::stopped(&error, budget)),
        },
        (ArtifactKind::Zip, PhysicalScope::ArtifactTree { root_container }) => {
            if root_container.0 != ROOT_CONTAINER {
                return Err(root_container_mismatch());
            }
            match snapshot.enumerate_artifact_tree(budget) {
                Ok(report) => {
                    debug_assert_eq!(report.view.scope, *scope);
                    let mut entries = Vec::new();
                    for container in &report.containers {
                        entries.extend(container.entries.iter().cloned());
                    }
                    Ok(ScopeScan {
                        entries,
                        physical_coverage: report.coverage,
                        execution: report.execution,
                        diagnostics: report.diagnostics,
                    })
                }
                Err(error) => Ok(ScopeScan::stopped(&error, budget)),
            }
        }
    }
}

impl ScopeScan {
    /// A scope whose own scan did not finish: no entry, no range, the stop and its diagnostic.
    ///
    /// The scope's *shape* is checked before this point, so an entry here is work that stopped — a
    /// refused charge, a cancellation, a container that cannot be read — and the listing that asked
    /// for it answers with the honest empty prefix, a diagnostic naming the failure and a non-
    /// `Complete` execution, never with a silent empty scope.
    fn stopped(error: &Error, budget: &Budget) -> Self {
        Self {
            entries: Vec::new(),
            physical_coverage: Coverage {
                artifact_structural: CoverageDimension {
                    state: CoverageState::Partial,
                    scanned: Vec::new(),
                    skipped: Vec::new(),
                    uninterpreted_extensions: Vec::new(),
                },
                runtime_resolution: CoverageDimension::not_requested(),
                dynamic_analysis: CoverageDimension::not_requested(),
            },
            execution: stop_execution(error, budget),
            diagnostics: vec![stop_diagnostic(error, None)],
        }
    }
}

/// The name of the one root container a fresh ZIP establishes.
const ROOT_CONTAINER: &str = "root";

fn root_container_mismatch() -> Error {
    Error::invalid_input(
        "navigation_root_container_mismatch",
        "the tree scope's root container is not this snapshot's root container",
    )
}

/// The scope's items, in the scope's own order: every entry exactly once, as a class candidate or as
/// the ordinary resource the rule leaves beside it.
fn scope_partition(
    kind: ArtifactKind,
    snapshot: &SnapshotId,
    entries: &[PhysicalEntry],
) -> Result<Vec<ClassListingItem>> {
    if kind == ArtifactKind::StandaloneClass {
        return Ok(vec![ClassListingItem::ClassCandidate {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.clone(),
            },
        }]);
    }
    Ok(entries
        .iter()
        .map(|entry| {
            if entry.id.raw_name.0.ends_with(CLASS_FILE_SUFFIX) {
                ClassListingItem::ClassCandidate {
                    location: PhysicalClassLocation::ArchiveEntry {
                        entry: entry.id.clone(),
                    },
                }
            } else {
                ClassListingItem::Resource {
                    entry: entry.id.clone(),
                }
            }
        })
        .collect())
}

/// Whether one candidate's own path could state the requested internal name.
///
/// A standalone root states no name until it is read — there is no path to compare with — so it is
/// always read and the confirmation decides it; a container candidate must have a `.class` raw name
/// whose path states the requested name at a `/` boundary. The rule is the lookup's own, and it is
/// deliberately the same one [`ClassNameBinding`] applies to the name a class declares.
fn candidate_states_name(candidate: &ClassCandidate<'_>, requested: &[u8]) -> bool {
    match candidate.location.entry() {
        None => true,
        Some(entry) => {
            entry.raw_name.0.ends_with(CLASS_FILE_SUFFIX)
                && path_states_name(&class_path_name(&entry.raw_name.0), requested)
        }
    }
}

/// The class candidates of a scope, in the scope's own order.
///
/// The rule is the raw-name one and nothing else: a candidate is a claim about a name, not about
/// bytes, and the read that follows is what turns it into a declaration.
fn scope_class_candidates<'a>(
    kind: ArtifactKind,
    snapshot: &SnapshotId,
    entries: &'a [PhysicalEntry],
) -> Vec<ClassCandidate<'a>> {
    if kind == ArtifactKind::StandaloneClass {
        return vec![ClassCandidate {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.clone(),
            },
            record: None,
        }];
    }
    entries
        .iter()
        .filter(|entry| entry.id.raw_name.0.ends_with(CLASS_FILE_SUFFIX))
        .map(|entry| ClassCandidate {
            location: PhysicalClassLocation::ArchiveEntry {
                entry: entry.id.clone(),
            },
            record: Some(entry),
        })
        .collect()
}

/// Reads one candidate's class bytes and its own declaration and member tables.
///
/// The read is `jarde_reader::classfile::class_member_facts`: it establishes the class's own
/// declaration and walks the member tables, and it stops at a member record that does not decode
/// instead of failing the whole class. That is what makes a damaged member a *per-item* fact here
/// rather than the end of the class: the class is confirmed, the members before the stop are read,
/// and the stop travels in [`ClassDeclarationItem::member_table`] with a diagnostic. A class whose
/// own declaration cannot be established is the case that fails, and the caller of this function
/// turns that into the listing's stop.
fn read_class_declaration(
    snapshot: &ArtifactSnapshot,
    candidate: &ClassCandidate<'_>,
    budget: &mut Budget,
) -> Result<ConfirmedRead> {
    let (bytes, source) = match candidate.record {
        Some(record) => {
            let materialized = snapshot.read_entry_for_analysis(record, budget)?;
            let class_bytes = ClassBytesId {
                digest: materialized.content_digest,
                length: to_u64(materialized.bytes.len())?,
            };
            (
                materialized.bytes,
                ClassSource {
                    location: PhysicalClassLocation::ArchiveEntry {
                        entry: materialized.entry,
                    },
                    class_bytes,
                },
            )
        }
        None => materialize_root(snapshot, budget)?,
    };
    let facts = class_member_facts(&bytes, budget)?;
    let definition = definition_of(&source);
    let binding = class_name_binding(&source, &facts.this_class);
    // The read's own stop is re-published with the class's physical origin, which is the one thing
    // the read does not know: it read bytes, and this layer knows where they came from. The
    // path/declaration finding is *not* added here: it is a statement about the item this read
    // produced, so the caller that publishes the item states it (and a lookup states its own,
    // narrower one about the name it asked under).
    let diagnostics = facts
        .stopped_at
        .as_ref()
        .map(|stop| vec![member_stop_diagnostic(&definition, stop)])
        .unwrap_or_default();
    let class = ClassContentItem::ClassDeclaration(ClassDeclarationItem {
        definition,
        declaration: declaration_facts(
            &facts.this_class,
            facts.access_flags,
            &facts.super_class,
            &facts.interfaces,
        ),
        binding,
        member_table: facts.stopped_at.clone(),
    });
    Ok(ConfirmedRead {
        class,
        facts,
        diagnostics,
        bytes,
        source,
    })
}

/// The physical definition one materialized class is: its source location and bytes, and the
/// syntactic variant its own raw path derives.
fn definition_of(source: &ClassSource) -> PhysicalDefinitionId {
    let variant = match source.entry() {
        Some(entry) => physical_variant_for_path(&entry.raw_name.0),
        None => PhysicalVariant::Base,
    };
    PhysicalDefinitionId {
        location: source.location.clone(),
        class_bytes: source.class_bytes.clone(),
        variant,
    }
}

/// The class-level facts of one read, from the parts every class-level read states.
fn declaration_facts(
    this_class: &JvmString,
    access_flags: u16,
    super_class: &Option<JvmString>,
    interfaces: &[JvmString],
) -> ClassDeclarationFacts {
    ClassDeclarationFacts {
        this_class: this_class.clone(),
        access_flags,
        super_class: super_class.clone(),
        interfaces: interfaces.to_vec(),
    }
}

/// What the entry's own raw path states, against the name the class declares.
fn class_name_binding(source: &ClassSource, declared: &JvmString) -> ClassNameBinding {
    match source.entry() {
        None => ClassNameBinding::StandaloneRoot,
        Some(entry) => {
            let path_name = class_path_name(&entry.raw_name.0);
            if path_states_name(&path_name, &declared.raw().0) {
                ClassNameBinding::PathNameAgrees
            } else {
                ClassNameBinding::PathNameDiffers {
                    path_name: JvmBytes(path_name),
                }
            }
        }
    }
}

/// The name one entry's raw path states: the raw name without a case-sensitive `.class` suffix.
///
/// No case folding, no `.`/`/` translation and no directory convention applies: an entry whose raw
/// name does not end in `.class` states the only name it has, its whole raw name.
fn class_path_name(raw_name: &[u8]) -> Vec<u8> {
    raw_name
        .strip_suffix(CLASS_FILE_SUFFIX)
        .unwrap_or(raw_name)
        .to_vec()
}

/// Whether a raw path states one internal name at a `/` boundary.
///
/// The rule is the path's own tail and nothing else: the path states the name when it *is* that name
/// or ends with `/<name>`. It is deliberately not a prefix rule — a longer path's prefix is left to
/// the caller's own declaration, because inferring a container layout or a class-path root is not
/// this change's to do.
fn path_states_name(path_name: &[u8], internal_name: &[u8]) -> bool {
    if path_name == internal_name {
        return true;
    }
    let mut boundary = Vec::with_capacity(internal_name.len() + 1);
    boundary.push(b'/');
    boundary.extend_from_slice(internal_name);
    path_name.ends_with(&boundary)
}

fn field_item(
    definition: &PhysicalDefinitionId,
    index: usize,
    field: &MemberHeader,
) -> Result<ClassContentItem> {
    Ok(ClassContentItem::Field(FieldItem {
        index: to_u64(index)?,
        identity: PhysicalMemberId {
            owner: definition.clone(),
            member: MemberKey::Field {
                name: JvmBytes(field.name.raw().0.clone()),
                descriptor: JvmBytes(field.descriptor.raw().0.clone()),
            },
        },
        name: field.name.clone(),
        descriptor: field.descriptor.clone(),
        access_flags: field.access_flags,
    }))
}

fn method_item(
    definition: &PhysicalDefinitionId,
    index: usize,
    method: &MemberHeader,
) -> Result<ClassContentItem> {
    let body = match method
        .attributes
        .iter()
        .find(|shell| shell.name.raw().0 == b"Code")
    {
        Some(shell) => MemberBodyEvidence::CodeAttribute {
            content_span: shell.content_span.clone(),
        },
        None => MemberBodyEvidence::NoCodeAttribute,
    };
    Ok(ClassContentItem::Method(MethodItem {
        index: to_u64(index)?,
        identity: PhysicalMethodId {
            owner: definition.clone(),
            name: JvmBytes(method.name.raw().0.clone()),
            descriptor: JvmBytes(method.descriptor.raw().0.clone()),
        },
        name: method.name.clone(),
        descriptor: method.descriptor.clone(),
        access_flags: method.access_flags,
        body,
    }))
}

/// Publishes one read's diagnostics, charging each as the item it is.
///
/// A diagnostic this layer re-publishes is an item of this layer's report, so it is billed here like
/// any other; a refused charge is the caller's stop to report, and the diagnostics published before
/// it stay.
fn publish_diagnostics(
    incoming: Vec<Diagnostic>,
    out: &mut Vec<Diagnostic>,
    budget: &mut Budget,
) -> Result<()> {
    for diagnostic in incoming {
        charge_item(budget)?;
        out.push(diagnostic);
    }
    Ok(())
}

/// Charges one item the caller will see, before it is published.
///
/// `ResultItems` counts items a public result returns, so the items this layer adds to a report are
/// charged here: the listing's partition items, the confirmed items, the navigation candidates and
/// this layer's own diagnostics. A read's own charges (the attribute shells and version diagnostics a
/// header read bills, the member records a member read bills) stay that read's — an engine budget is
/// an upper bound on a request's work, not a per-pass construction counter.
fn charge_item(budget: &mut Budget) -> Result<()> {
    budget.charge(CountedBudgetDimension::ResultItems, 1)
}

pub(crate) fn to_u64(value: usize) -> Result<u64> {
    u64::try_from(value).map_err(|_| {
        Error::invalid_input("navigation_size_overflow", "an item count does not fit u64")
    })
}

/// The structural coverage of one listing: the scope scan's own ranges plus the listing's progress.
///
/// The physical ranges keep the reader's labels and the reader's coordinates, and the listing's own
/// progress is one labeled range over the items it selected — scanned up to what it examined, and
/// skipped from there to the end of what it selected. The dimension is `CompleteWithinSchema` only
/// when the physical scan and the listing both ran to their end.
pub(crate) fn listing_coverage(
    physical: &Coverage,
    label: &str,
    examined: u64,
    total: u64,
    complete: bool,
) -> Coverage {
    let mut scanned = physical.artifact_structural.scanned.clone();
    let mut skipped = physical.artifact_structural.skipped.clone();
    scanned.push(CoverageRange {
        label: label.to_owned(),
        start: 0,
        end: examined,
    });
    if examined < total {
        skipped.push(CoverageRange {
            label: label.to_owned(),
            start: examined,
            end: total,
        });
    }
    let structural_complete =
        complete && physical.artifact_structural.state == CoverageState::CompleteWithinSchema;
    Coverage {
        artifact_structural: CoverageDimension {
            state: if structural_complete {
                CoverageState::CompleteWithinSchema
            } else {
                CoverageState::Partial
            },
            scanned,
            skipped,
            uninterpreted_extensions: physical
                .artifact_structural
                .uninterpreted_extensions
                .clone(),
        },
        runtime_resolution: CoverageDimension::not_requested(),
        dynamic_analysis: CoverageDimension::not_requested(),
    }
}

/// The coverage of one member listing, in the member tables' own coordinates.
///
/// One range per table states how far that table was read and where it stopped: the members this
/// listing published are the scanned prefix and the records after a stop are the skipped remainder —
/// `class_fields` and `class_methods`, the coordinates the members are read in. A table the walk never
/// reached has nothing scanned and everything skipped, which is exactly what "the walk stopped in the
/// field table" means.
fn member_coverage(facts: &ClassMemberFacts, complete: bool) -> Coverage {
    let stopped_in_fields = facts
        .stopped_at
        .as_ref()
        .is_some_and(|stop| stop.phase == MemberTablePhase::Fields);
    let fields_read = to_u64(facts.fields.len()).unwrap_or(u64::MAX);
    let methods_read = if stopped_in_fields {
        0
    } else {
        to_u64(facts.methods.len()).unwrap_or(u64::MAX)
    };
    let mut scanned = Vec::new();
    let mut skipped = Vec::new();
    push_table_ranges(
        "class_fields",
        fields_read,
        facts.field_count,
        &mut scanned,
        &mut skipped,
    );
    push_table_ranges(
        "class_methods",
        methods_read,
        facts.method_count,
        &mut scanned,
        &mut skipped,
    );
    Coverage {
        artifact_structural: CoverageDimension {
            state: if complete {
                CoverageState::CompleteWithinSchema
            } else {
                CoverageState::Partial
            },
            scanned,
            skipped,
            uninterpreted_extensions: Vec::new(),
        },
        runtime_resolution: CoverageDimension::not_requested(),
        dynamic_analysis: CoverageDimension::not_requested(),
    }
}

fn push_table_ranges(
    label: &str,
    read: u64,
    declared: u64,
    scanned: &mut Vec<CoverageRange>,
    skipped: &mut Vec<CoverageRange>,
) {
    if read > 0 {
        scanned.push(CoverageRange {
            label: label.to_owned(),
            start: 0,
            end: read,
        });
    }
    if read < declared {
        skipped.push(CoverageRange {
            label: label.to_owned(),
            start: read,
            end: declared,
        });
    }
}

/// The terminal state one failed step leaves, in the vocabulary the whole engine reports stops in.
///
/// The mapping is the one the artifact tree, the query scan and the resolver already apply: a
/// cancellation is `Cancelled`, an exhausted dimension `Partial` with that dimension named, an
/// unsupported refusal `Partial` with its code, and a structure this read could not read `Failed` with
/// the reader's own code.
pub(crate) fn stop_execution(error: &Error, budget: &Budget) -> ExecutionReport {
    let usage = budget.usage();
    match error {
        Error::Cancelled { .. } => ExecutionReport::Cancelled { usage },
        Error::BudgetExceeded { dimension, .. } => ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: *dimension,
            },
            usage,
        },
        Error::Unsupported { code, .. } => ExecutionReport::Partial {
            reason: TerminationReason::Unsupported { code: code.clone() },
            usage,
        },
        Error::InvalidInput { code, .. }
        | Error::Io {
            operation: code, ..
        } => ExecutionReport::Failed {
            reason: TerminationReason::Error { code: code.clone() },
            usage,
        },
    }
}

/// The state one member-table stop leaves: the class is read and the table stopped, so the read is
/// `Partial` with the reader's own code for the failure — the same shape the classfile layer uses for
/// a body decode that stopped.
fn member_stop_execution(stop: &MemberTableStop, budget: &Budget) -> ExecutionReport {
    ExecutionReport::Partial {
        reason: TerminationReason::Error {
            code: stop.code.clone(),
        },
        usage: budget.usage(),
    }
}

/// Merges one stop into a report's state, keeping the strongest one.
///
/// The order is the engine's: a cancellation outranks an exhausted dimension, which outranks a
/// failure, which outranks any other non-`Complete` state, which outranks `Complete`. A `Complete`
/// state never overwrites a stop, so the earliest evidence of work that did not finish survives the
/// steps that follow it.
pub(crate) fn merge_execution(execution: &mut ExecutionReport, incoming: ExecutionReport) {
    if stop_priority(&incoming) > stop_priority(execution) {
        *execution = incoming;
    }
}

fn merge_execution_option(execution: &mut Option<ExecutionReport>, incoming: ExecutionReport) {
    match execution {
        Some(current) => merge_execution(current, incoming),
        None => *execution = Some(incoming),
    }
}

fn stop_priority(execution: &ExecutionReport) -> u8 {
    match execution {
        ExecutionReport::Complete { .. } => 0,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded { .. },
            ..
        }
        | ExecutionReport::Failed {
            reason: TerminationReason::BudgetExceeded { .. },
            ..
        } => 3,
        ExecutionReport::Cancelled { .. } => 4,
        ExecutionReport::Failed { .. } => 2,
        ExecutionReport::Partial { .. } => 1,
    }
}

/// Whether one stop ends the shared request rather than the value it was reading.
///
/// A cancellation and an exhausted shared dimension are the *request* ending: everything after it
/// is work the same budget cannot fund, so a composed operation that has hit one starts nothing
/// further. A value-level stop — a decode that stopped at the bytes it was reading, an unsupported
/// construct — is isolated to the result it happened in, and the work the caller asked for beside
/// it still runs.
fn ends_the_request(execution: &ExecutionReport) -> bool {
    match execution {
        ExecutionReport::Complete { .. } => false,
        ExecutionReport::Cancelled { .. } => true,
        ExecutionReport::Partial { reason, .. } | ExecutionReport::Failed { reason, .. } => {
            matches!(reason, TerminationReason::BudgetExceeded { .. })
        }
    }
}

/// One diagnostic for a step that stopped the work it was part of.
///
/// The code is the failure's own, so one failure has one code wherever this engine reports it, and the
/// provenance names the physical position the step was working on when it stopped — the entry it was
/// reading, or the definition it was reading from. A stop diagnostic is control metadata: it explains
/// why the report is not complete, so it is published whether or not the item budget that stopped it
/// could pay for it.
pub(crate) fn stop_diagnostic(error: &Error, provenance: Option<Provenance>) -> Diagnostic {
    Diagnostic {
        code: error_code(error),
        severity: if matches!(
            error,
            Error::InvalidInput { .. } | Error::Unsupported { .. }
        ) {
            DiagnosticSeverity::Error
        } else {
            DiagnosticSeverity::Warning
        },
        message: error.to_string(),
        provenance,
    }
}

/// The diagnostic of one item whose own raw path and declared name disagree.
///
/// A listing has no request to compare against, so the finding it can state is the entry's own: the
/// path states one name and the class's header states another. A standalone root has no path and
/// therefore no finding.
fn item_path_mismatch_diagnostic(class: &ClassDeclarationItem) -> Option<Diagnostic> {
    if !matches!(class.binding, ClassNameBinding::PathNameDiffers { .. }) {
        return None;
    }
    let entry = class.definition.entry()?;
    Some(path_name_mismatch_diagnostic(
        entry,
        class.definition.class_bytes.length,
        &class.declaration.this_class.raw().0,
        &class.declaration.this_class,
    ))
}

/// The diagnostic of one entry whose raw path does not state the name it was read under.
///
/// `read_under` is the name this entry was asked to be — the requested internal name in a lookup, or
/// the class's own declared name in a listing that has no request — and `declared` is the name the
/// class's header states. `class_length` is the entry's own byte length, so the diagnostic can carry
/// the whole entry as its physical origin.
///
/// It is a `Warning` about the artifact, not a failure of a read: the entry keeps both names apart
/// (the item's `binding` and its own raw name), and this only states why the entry is not a binding
/// for the name it was read under. A path is not a declaration, and which of the two names is "right"
/// is not this layer's to decide.
fn path_name_mismatch_diagnostic(
    entry: &PhysicalEntryId,
    class_length: u64,
    read_under: &[u8],
    declared: &JvmString,
) -> Diagnostic {
    let path_name = class_path_name(&entry.raw_name.0);
    Diagnostic {
        code: "navigation_path_name_mismatch".into(),
        severity: DiagnosticSeverity::Warning,
        message: format!(
            "the entry's raw path states `{}` while it was read under `{}` and the class it holds declares `{}`; both are published and this entry is not a binding for `{}`",
            String::from_utf8_lossy(&path_name),
            String::from_utf8_lossy(read_under),
            declared.escaped(),
            String::from_utf8_lossy(read_under),
        ),
        provenance: Some(entry_provenance(entry, class_length)),
    }
}

/// The diagnostic of a member-table read that stopped inside a member record.
///
/// The class's declaration was read and the members before the stop stay usable, so this states the
/// position of the stop in the table and the reader's own code for it; the class-file offset the read
/// was working at travels in the message the reader reported.
fn member_stop_diagnostic(definition: &PhysicalDefinitionId, stop: &MemberTableStop) -> Diagnostic {
    let table = match stop.phase {
        MemberTablePhase::Fields => "fields",
        MemberTablePhase::Methods => "methods",
    };
    Diagnostic {
        code: stop.code.clone(),
        severity: DiagnosticSeverity::Error,
        message: format!(
            "the member table stopped at {table}[{}] (class-file offset {}): {}; the members before it are the reliable prefix",
            stop.index, stop.class_offset, stop.message
        ),
        provenance: Some(definition_provenance(definition)),
    }
}

/// The stable code of one failure, using the code the error already carries.
///
/// The budget case keeps the engine's own convention (`budget_exceeded_<dimension>`), so a report's
/// diagnostic and a request-level error name the same dimension the same way.
pub(crate) fn error_code(error: &Error) -> String {
    match error {
        Error::InvalidInput { code, .. } | Error::Unsupported { code, .. } => code.clone(),
        Error::BudgetExceeded { dimension, .. } => {
            format!("budget_exceeded_{}", budget_dimension_code(*dimension))
        }
        Error::Cancelled { .. } => "cancelled".to_owned(),
        Error::Io { operation, .. } => operation.clone(),
    }
}

/// The physical origin of one entry, as the whole entry's bytes.
pub(crate) fn entry_provenance(entry: &PhysicalEntryId, length: u64) -> Provenance {
    Provenance {
        location: Location::Entry {
            id: entry.clone(),
            span: ByteSpan::new(0, length),
        },
    }
}

/// The physical origin of one definition, as that class file from its first byte.
pub(crate) fn definition_provenance(definition: &PhysicalDefinitionId) -> Provenance {
    Provenance {
        location: Location::ClassOffset {
            definition: definition.clone(),
            offset: 0,
        },
    }
}

/// The physical origin of one candidate read, when the candidate has one.
///
/// A standalone root has no entry to name: the snapshot's own id is that class's origin, and a
/// listing of it says so in its message rather than inventing an entry that does not exist.
fn candidate_provenance(candidate: &ClassCandidate<'_>) -> Option<Provenance> {
    match (candidate.location.entry(), candidate.record) {
        (Some(entry), Some(record)) => Some(entry_provenance(entry, record.uncompressed_size)),
        _ => None,
    }
}

/// The physical origin of one listing item, as the entry's whole bytes.
///
/// The entry's length comes from the scope scan's own record of it when the scan holds one — the
/// partition is built from exactly those records — and the location alone is stated otherwise. A
/// standalone root has no entry to name: its origin is the snapshot itself, and a listing of it says
/// so in its message rather than inventing an entry that does not exist.
fn item_provenance(item: &ClassListingItem, entries: &[PhysicalEntry]) -> Option<Provenance> {
    let entry = match item {
        ClassListingItem::ClassCandidate { location } => location.entry()?.clone(),
        ClassListingItem::Resource { entry } => entry.clone(),
    };
    let length = entries
        .iter()
        .find(|record| record.id == entry)
        .map_or(0, |record| record.uncompressed_size);
    Some(entry_provenance(&entry, length))
}

/// The call sites of one presented body the `accessor@1` rule reads a verdict from, and the
/// definition the class they may come from is (P3 3.2).
///
/// The candidates are the presented body's **own decode**: the call sites the `accessor@1` rule
/// would decide from, enumerated by that rule ([`jarde_java::accessor::candidates`]) so that what a
/// run reads the class's members for is what the rule reads a verdict from, and never a second
/// opinion about which calls matter. `None` is the answer for a body that names no such call site —
/// and for one whose run read no member header at all, which states no definition its members could
/// come from (P3 3.1/3.2): neither reads a header or a member.
///
/// The definition is the one the run read the presented body from, as the payload's own declaration
/// states it: not a name a call site spells, and never a second class. A call site naming another
/// class is refused by the read with that stated, so "a member of a class that happens to share this
/// name" cannot be read as this call's callee.
fn named_callee_candidates(
    ir: &jarde_jvm::method_ir::MethodIr,
) -> Option<(
    PhysicalDefinitionId,
    Vec<jarde_jvm::callee::CalleeCandidate>,
)> {
    let mut candidates: Vec<jarde_jvm::callee::CalleeCandidate> =
        jarde_java::accessor::candidates(ir)
            .into_iter()
            .map(|candidate| {
                jarde_jvm::callee::CalleeCandidate::new(
                    candidate.call_site,
                    candidate.owner.as_bytes(),
                    candidate.name.as_bytes(),
                    candidate.descriptor.as_bytes(),
                )
            })
            .collect();
    candidates.extend(
        jarde_java::lambda::array_helper_candidates(ir)
            .into_iter()
            .map(|candidate| {
                jarde_jvm::callee::CalleeCandidate::new(
                    candidate.call_site,
                    candidate.owner.0,
                    candidate.name.0,
                    candidate.descriptor.0,
                )
            }),
    );
    if candidates.is_empty() {
        return None;
    }
    let declaration = ir.declaration()?;
    let mut unique = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if !unique.contains(&candidate) {
            unique.push(candidate);
        }
    }
    Some((declaration.identity().owner.clone(), unique))
}

/// The class's own members the presented body's call sites named, read on demand (P3 3.2).
///
/// This is the read for a caller that holds **no** prepared class: one header read by identity, then
/// one `MethodBodies` attempt per distinct named member. A caller that holds one — a bulk worker's
/// class task, or this operation's own read of the same definition (D2 3.3) — calls
/// [`read_prepared_named_callees`] instead and pays no class read at all.
fn read_named_callees(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    candidates: &(
        PhysicalDefinitionId,
        Vec<jarde_jvm::callee::CalleeCandidate>,
    ),
    budget: &mut Budget,
) -> Result<jarde_jvm::callee::CalleeReadReport> {
    let (definition, candidates) = candidates;
    jarde_jvm::callee::read_callees(
        content,
        &jarde_jvm::callee::CalleeReadRequest::new(&request.environment, definition, candidates),
        budget,
    )
}

/// The same read, from a class the caller already prepared (bulk tasks 2.3 and 3.2, D2 3.3).
///
/// The candidates, their order and the read's refusals are [`named_callee_candidates`]'s, exactly as
/// the direct read's are; what changes is where the member records and the bodies come from — the
/// caller's prepared class, which charged one class read for everything that consumes it.
fn read_prepared_named_callees(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    candidates: &(
        PhysicalDefinitionId,
        Vec<jarde_jvm::callee::CalleeCandidate>,
    ),
    prepared: &jarde_reader::prepared::PreparedClass<'_>,
    budget: &mut Budget,
) -> Result<jarde_jvm::callee::CalleeReadReport> {
    let (definition, candidates) = candidates;
    jarde_jvm::callee::read_prepared_callees(
        content,
        prepared,
        &jarde_jvm::callee::CalleeReadRequest::new(&request.environment, definition, candidates),
        budget,
    )
}

/// One callee read as the member table the accessor rule reads (P3 3.2).
///
/// The read hands over the class's declarations and bodies in the reader's vocabulary; this is the
/// one adapter between it and [`jarde_java::ClassMembers`], which is `jarde-java`'s own type: the
/// fact layer cannot produce it (it does not depend on the layer above), and it is the layer that
/// owns what a member means. Every member keeps the identity it was read under, so the anchors the
/// presentation derives from one state which class file their BCI and constant pool belong to.
fn member_table(read: &jarde_jvm::callee::CalleeReadReport) -> jarde_java::ClassMembers {
    let members = read
        .members()
        .iter()
        .map(|member| {
            let owner = read.class().to_string();
            let identity = member.identity().clone();
            let flags = member.access_flags();
            match member.body() {
                Some(body) => {
                    jarde_java::MemberBody::new(owner, identity, flags, body.facts().clone())
                }
                None => jarde_java::MemberBody::without_body(owner, identity, flags),
            }
        })
        .collect();
    jarde_java::ClassMembers::new(read.class(), members)
}

/// One method-analysis run and the presentation of its own payload (P3 1.3).
///
/// Both halves are of the *same* run: [`RecoveredMethod::analysis`] is the report
/// [`jarde_jvm::analyze_method_ir`] assembled for the run whose tables the recovery read, so the
/// stage results, the coverage, the reads and the planes of the analysis and the planes of the
/// recovery describe one request. A caller that wants only the presentation reads
/// [`RecoveredMethod::recovery`] and ignores the other half; a caller that wants the run's own
/// evidence (which reads it charged, which stages completed) reads both.
///
/// [`RecoveredMethod::callees`] is the third part and the only thing this entry reads beyond that
/// run (P3 3.2): the class's own members the presented body's call sites named, read on demand,
/// charged, and bound to the definition the run read the body from. It is `None` when the body named
/// no such call site — which is the ordinary case, and the reason a recovery request that presents
/// an ordinary body still reads exactly one header and one body.
#[derive(Clone, Debug, Serialize)]
pub struct RecoveredMethod {
    analysis: crate::ir::MethodAnalysisReport,
    recovery: jarde_java::RecoveryReport,
    callees: Option<jarde_jvm::callee::CalleeReadReport>,
    /// The facts this run read (P3 3.3's [`RecoveredMethod::facts`]). Not part of the serialized
    /// document: the report's own `declaration` record is that half of it, and a second copy of the
    /// same facts in the payload would be a second schema to keep in step.
    #[serde(skip)]
    facts: jarde_java::RecoveryFacts,
}

impl RecoveredMethod {
    /// The report of the run that produced the payload the presentation read.
    pub fn analysis(&self) -> &crate::ir::MethodAnalysisReport {
        &self.analysis
    }

    /// The presentation of that run's payload.
    pub fn recovery(&self) -> &jarde_java::RecoveryReport {
        &self.recovery
    }

    /// The facts the run's own payload stated, and therefore the declaration the artifact was
    /// written from: the member's flags, raw name, descriptor and parameter-slot count as the
    /// header read that decoded the body located them, plus the raw debug name of every slot that
    /// read carried (P3 3.1, P3 2.4).
    ///
    /// A consumer that has to **state the same declaration** — P3 3.3's comparison wraps the
    /// recovered text in a method declaration, and a signature written by hand would be a second
    /// opinion about the member, wrong the moment a parameter is a `boolean` — reads it here: it is
    /// the very value the presentation of this run was handed, so the two cannot disagree.
    pub fn facts(&self) -> &jarde_java::RecoveryFacts {
        &self.facts
    }

    /// The class's own members this request read for the presented body's named call sites, when it
    /// named any: which class was read, which members it declares among them, which candidates it
    /// refused and what the read charged (P3 3.2).
    pub fn callees(&self) -> Option<&jarde_jvm::callee::CalleeReadReport> {
        self.callees.as_ref()
    }

    /// Every part, by value: the adapter that serializes them does not clone what it owns.
    pub fn into_parts(
        self,
    ) -> (
        crate::ir::MethodAnalysisReport,
        jarde_java::RecoveryReport,
        Option<jarde_jvm::callee::CalleeReadReport>,
    ) {
        (self.analysis, self.recovery, self.callees)
    }
}

/// One analysis run presented, from whichever read produced it (bulk task 3.2).
///
/// This is the whole of `Engine`'s recovery presentation: the facts the recovery layer needs, the
/// on-demand callee read of the presented body's own call sites, and one [`jarde_java::recover`]
/// call over the run's payload. It exists as one function because the bulk operation presents **the
/// same run** for every method of a prepared class ([`jarde_jvm::analyze_prepared_method_ir`]) and a
/// second presentation path would be a second spelling of the same contract — the callers differ in
/// one thing only: which read of the presented body's own class a same-class callee read consumes
/// ([`CalleeClass`]).
///
/// Everything else — which candidates are read, in which order, with which refusals, and how the
/// payload is presented — is this function's, so a method presented through the bulk path and the
/// same method presented through [`Engine::recover_method`] cannot drift.
pub(crate) fn recovery_presented(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    analyzed: jarde_jvm::method_ir::MethodIrAnalysis,
    prepared: Option<&jarde_reader::prepared::PreparedClass<'_>>,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> Result<RecoveredMethod> {
    let callee_class = match prepared {
        Some(prepared) => CalleeClass::Prepared(prepared),
        None => CalleeClass::None,
    };
    recovery_from(content, request, analyzed, callee_class, evidence, budget)
}

/// The class-source member path, which keeps same-run `<clinit>` AST candidates private until the
/// class-level field projection consumes them.
fn recovery_presented_for_class_source(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    analyzed: jarde_jvm::method_ir::MethodIrAnalysis,
    prepared: &jarde_reader::prepared::PreparedClass<'_>,
    assembly_context: &class_source::ClassSourceAssemblyContext,
    evidence: &RecoveryEvidenceRequest,
    prove_generic_return: bool,
    capture_enum_constructor_ast: bool,
    budget: &mut Budget,
) -> Result<(
    RecoveredMethod,
    Option<jarde_java::report::ClassInitializerCandidates>,
    Option<jarde_java::report::ClassEnumConstructorCandidates>,
    Option<jarde_java::bridge::ClassSourceBridgeCandidate>,
    Option<Vec<jarde_java::enumswitch::ClassSourceEnumSwitchCandidate>>,
    Option<Vec<jarde_java::report::ClassSourceArrayConstructorCandidate>>,
    Option<Vec<jarde_java::report::ClassSourceEnumSwitchFieldUse>>,
    Option<jarde_java::report::GenericReturnCandidate>,
    Option<jarde_java::report::GenericConstructorCandidate>,
    Option<jarde_java::report::AnonymousAllocationScan>,
)> {
    recovery_from_with_class_candidates(
        content,
        request,
        analyzed,
        CalleeClass::Prepared(prepared),
        Some(assembly_context),
        evidence,
        budget,
        true,
        prove_generic_return,
        capture_enum_constructor_ast,
    )
}

/// The same presentation for a caller that holds the read the run performed, not a preparation
/// (D2 3.1/3.3).
///
/// `read` is [`jarde_jvm::AnalyzedMethod::read`]: the class the run itself read, when it read one.
/// The same-class callee read consumes a preparation of *that* read — made here, exactly when the
/// presented body really names members of the same class — so the class is read once for the whole
/// request, and a body that names no such member prepares nothing at all.
fn recovery_read(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    analyzed: jarde_jvm::method_ir::MethodIrAnalysis,
    read: Option<jarde_reader::prepared::PreparedClassRead>,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> Result<RecoveredMethod> {
    let callee_class = match &read {
        Some(read) => CalleeClass::Read(read),
        None => CalleeClass::None,
    };
    recovery_from(content, request, analyzed, callee_class, evidence, budget)
}

/// The class one recovery presentation reads its same-class callee evidence from.
///
/// The three cases are the three callers of the presentation, and the difference between them is
/// exactly "who already read the presented body's class":
enum CalleeClass<'a> {
    /// A class the caller already prepared — a bulk worker's class task, or this operation's own
    /// preparation of a read its binding performed. Every callee read is answered from it, and no
    /// class is read for one.
    Prepared(&'a jarde_reader::prepared::PreparedClass<'a>),
    /// The read the run performed itself: prepared once, and only when the presented body's call
    /// sites name members of the same class — the read a callee read would otherwise have to
    /// perform again is this one.
    Read(&'a jarde_reader::prepared::PreparedClassRead),
    /// No class: the presentation reads the class the call sites named, as it always did.
    None,
}

/// The member record one prepared class located for a member, as the ordinal the reader states.
///
/// The locator is multi-valued on purpose (`PreparedClass::locate_method` returns *every* ordinal
/// declaring a name and descriptor): a class that declares one name and descriptor twice has two
/// records, and neither is *the* record. So exactly one ordinal is an answer and anything else —
/// none, or several — is `None`: a position this read did not establish must not be invented, and a
/// duplicate declaration has no artifact to bind in the first place (the body read refuses it).
fn member_ordinal(
    prepared: &jarde_reader::prepared::PreparedClass<'_>,
    method: &PhysicalMethodId,
) -> Option<jarde_reader::prepared::MethodOrdinal> {
    match prepared.locate_method(&method.name.0, &method.descriptor.0) {
        [ordinal] => Some(*ordinal),
        _ => None,
    }
}

/// One run presented, with the class a same-class callee read may come from.
fn recovery_from(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    analyzed: jarde_jvm::method_ir::MethodIrAnalysis,
    callee_class: CalleeClass<'_>,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> Result<RecoveredMethod> {
    recovery_from_with_class_candidates(
        content,
        request,
        analyzed,
        callee_class,
        None,
        evidence,
        budget,
        false,
        false,
        false,
    )
    .map(|(recovered, _, _, _, _, _, _, _, _, _)| recovered)
}

fn recovery_from_with_class_candidates(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    analyzed: jarde_jvm::method_ir::MethodIrAnalysis,
    callee_class: CalleeClass<'_>,
    assembly_context: Option<&class_source::ClassSourceAssemblyContext>,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
    include_class_source_candidates: bool,
    prove_generic_return: bool,
    capture_enum_constructor_ast: bool,
) -> Result<(
    RecoveredMethod,
    Option<jarde_java::report::ClassInitializerCandidates>,
    Option<jarde_java::report::ClassEnumConstructorCandidates>,
    Option<jarde_java::bridge::ClassSourceBridgeCandidate>,
    Option<Vec<jarde_java::enumswitch::ClassSourceEnumSwitchCandidate>>,
    Option<Vec<jarde_java::report::ClassSourceArrayConstructorCandidate>>,
    Option<Vec<jarde_java::report::ClassSourceEnumSwitchFieldUse>>,
    Option<jarde_java::report::GenericReturnCandidate>,
    Option<jarde_java::report::GenericConstructorCandidate>,
    Option<jarde_java::report::AnonymousAllocationScan>,
)> {
    let facts = crate::facade::recovery_facts(
        analyzed.ir().declaration(),
        analyzed.ir().code(),
        &request.method,
    );
    let profile = request.environment.runtime.profile.clone();
    // The callee evidence one recovery run's own call sites justify, read on demand from the very
    // definition the run read the presented body from (P3 3.2). It happens **between** the run and
    // the presentation — not inside the recovery layer, which holds no artifact, no loader and no
    // budget — and it is the only read this entry performs beyond the one run: a recovery request
    // whose body names no such call site reads no member at all, and (D2 3.3) no preparation either.
    let candidates = named_callee_candidates(analyzed.ir());
    // The member record this presentation's own selection established (change
    // `add-demand-driven-core-results`, D3'). The ordered member table of a prepared class is the
    // one thing that says *which* of two records declaring one name and descriptor an artifact was
    // written for, so it is read from the preparation this entry already holds — the class task's
    // own, or the one the callee read below makes — and from nothing else. An entry that holds no
    // preparation walks no member table: it states `None` rather than a position it never located.
    let mut ordinal = match &callee_class {
        CalleeClass::Prepared(prepared) => member_ordinal(prepared, &request.method),
        _ => None,
    };
    let callees = match (&callee_class, &candidates) {
        (_, None) => None,
        (CalleeClass::Prepared(prepared), Some(candidates)) => Some(read_prepared_named_callees(
            content, request, candidates, prepared, budget,
        )?),
        (CalleeClass::Read(read), Some(candidates)) => {
            // One prepared class over one materialization (`crate::d0_counts`), made exactly when a
            // same-class callee read consumes it: the class read once serves the run above and this
            // callee read, and a body that named no such member never gets here.
            crate::d0_counts::class_prepared();
            let prepared = jarde_reader::prepared::PreparedClass::prepare(read, budget)?;
            ordinal = member_ordinal(&prepared, &request.method);
            Some(read_prepared_named_callees(
                content, request, candidates, &prepared, budget,
            )?)
        }
        (CalleeClass::None, Some(candidates)) => {
            Some(read_named_callees(content, request, candidates, budget)?)
        }
    };
    let members = callees.as_ref().map(member_table);
    let member_inner_targets = match assembly_context {
        Some(caller) => {
            read_class_source_member_inner_targets(content, request, analyzed.ir(), caller, budget)?
        }
        None => Vec::new(),
    };
    let interface_super_calls =
        interface_super_calls_presented(content, request, analyzed.ir(), budget)?;
    // What the artifact this run is about to commit is *of*, as this entry's own trusted read states
    // it (D3'): the physical identity the run was bound to, the member record the selection above
    // established and the environment the run was validated under. This is the entry's statement and
    // never the caller's: a caller's `expected_artifact` is only ever a candidate for verification.
    let analysis = analyzed.report();
    let subject = jarde_java::ArtifactSubject::new(
        analysis.method.clone(),
        ordinal,
        analysis.environment_identity.clone(),
    );
    let request = jarde_java::RecoveryRequest::new(analyzed.ir(), &facts, profile)
        .with_evidence(evidence.clone())
        .with_subject(subject)
        .with_member_inner_targets(&member_inner_targets)
        .with_interface_super_calls(&interface_super_calls);
    let (
        mut recovery,
        initializer_candidates,
        enum_constructor_candidates,
        bridge_candidate,
        enum_switch_candidates,
        array_constructor_candidates,
        enum_switch_field_uses,
        generic_return,
        generic_constructor,
        anonymous_allocations,
    ) = match &members {
        Some(members) if include_class_source_candidates => {
            let result = jarde_java::report::recover_for_class_source(
                &request.with_members(members),
                budget,
                prove_generic_return || !member_inner_targets.is_empty(),
                capture_enum_constructor_ast,
            );
            (
                result.report,
                result.initializer,
                result.enum_constructor,
                result.bridge,
                Some(result.enum_switches),
                Some(result.array_constructors),
                Some(result.enum_switch_field_uses),
                result.generic_return,
                result.generic_constructor,
                result.anonymous_allocations,
            )
        }
        None if include_class_source_candidates => {
            let result = jarde_java::report::recover_for_class_source(
                &request,
                budget,
                prove_generic_return || !member_inner_targets.is_empty(),
                capture_enum_constructor_ast,
            );
            (
                result.report,
                result.initializer,
                result.enum_constructor,
                result.bridge,
                Some(result.enum_switches),
                Some(result.array_constructors),
                Some(result.enum_switch_field_uses),
                result.generic_return,
                result.generic_constructor,
                result.anonymous_allocations,
            )
        }
        Some(members) => (
            jarde_java::recover(&request.with_members(members), budget),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
        None => (
            jarde_java::recover(&request, budget),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ),
    };
    // What the read evidence publishes (change `add-demand-driven-core-results`, D3). The read above
    // is the accessor rule's own input whatever the caller selected — the member table it decides
    // from — so what the selection decides is the **record**: the callee facts and the header-read
    // proofs are the `ReadDetails` expansion, published only when the request selected it and the
    // presentation really produced an artifact. Every other case publishes the binding results alone:
    // which member each candidate resolved to, which candidates this class does not answer, and what
    // the read charged. Nothing is read a second time on either branch.
    //
    // The run's own verdict about the artifact the request named gates this too (D3', tasks 5.2/5.3):
    // this is the one category the entry materializes, and a mismatch attaches *no* evidence —
    // including this one — to a text the run did not write. The binding results beside it are not
    // evidence: they are the answer the read already had to give the accessor rule, and they are
    // carried as they always were.
    let publish_read_details = evidence.requests(RecoveryEvidenceKind::ReadDetails)
        && recovery.produced()
        && recovery.artifact.attaches();
    let callees = match callees {
        None => None,
        Some(read) if publish_read_details => {
            crate::d0_counts::read_detail_records(read.detail_records());
            Some(read)
        }
        Some(read) => Some(read.without_read_details()),
    };
    if publish_read_details {
        // The category is stated even when the body named no callee candidate at all: nothing was
        // read, which is a legal *empty* delivery of the category, and not the `NotPerformed` a run
        // that stopped before the read states.
        recovery.read_details_materialized();
    }
    // One recovery presentation over one analysis run (`crate::d0_counts`), and the owning records
    // the publication below builds: the cloned analysis report itself, its stage records, its read
    // records and its diagnostics. The recovery report's own optional tables are built in
    // `jarde-java` and are counted there when that layer's hook is placed (D3).
    let analysis = analyzed.report();
    crate::d0_counts::recovery_presented_run();
    crate::d0_counts::owned_records(
        1 + u64::try_from(analysis.stages.len()).unwrap_or(u64::MAX)
            + u64::try_from(analysis.reads.len()).unwrap_or(u64::MAX)
            + u64::try_from(analysis.diagnostics.len()).unwrap_or(u64::MAX),
    );
    Ok((
        RecoveredMethod {
            analysis: analysis.clone(),
            recovery,
            callees,
            facts,
        },
        initializer_candidates,
        enum_constructor_candidates,
        bridge_candidate,
        enum_switch_candidates,
        array_constructor_candidates,
        enum_switch_field_uses,
        generic_return,
        generic_constructor,
        anonymous_allocations,
    ))
}

/// The facts of one member: what the run's own header read declared about it, or nothing but the
/// request's identity when the run read no member header.
///
/// The declaration is the payload's (P3 3.1): the flags, the raw name and descriptor and the
/// parameter-slot count all come from the member the `raw_facts` pass located in the same header read
/// that decoded the body, so this entry reads **once** and cannot state a second opinion about the
/// member. The flags travel with it, which is what makes a member the class declares a bridge a
/// bridge here: the `bridge@1` rule reads them and no shape is guessed from the body.
///
/// The class that declares the member travels with the same declaration (the declaring-class
/// handoff): its internal name and its access flags are the two facts the *same* header read already
/// held, so the `DeclaringClass` the recovery layer reads is filled from the run's own evidence
/// instead of being left to the caller — which is what lets `declaration@1` tell an interface's
/// `default` method from an ordinary one, and `init@1`/`field@1` read a constructor's prologue and
/// the writes it makes on its own uninitialized `this`. The name is spelled here, at this boundary
/// only, with the lossy UTF-8 read this layer's other names already use ([`lossy_jvm_name`]);
/// the payload's own bytes stay the identity, and no second name system is introduced for it.
///
/// With no declaration — a run that stopped before `raw_facts`, or a member that declares no body —
/// the name and descriptor are the request's own bytes and the parameter-slot count is **zero**: no
/// read stated how many slots the parameters occupy, so every slot is named by its ordinal as a local
/// (`local0`, `local1`, …), which is A10's deterministic naming and no claim about the signature. No
/// class is stated either: a run that published no declaration published no class facts, and the
/// request's owner spelling is not a substitute for them.
fn recovery_facts(
    declaration: Option<&jarde_jvm::method_ir::MethodDeclaration>,
    code: Option<&jarde_reader::classfile::MethodCodeFacts>,
    method: &PhysicalMethodId,
) -> jarde_java::RecoveryFacts {
    let debug = debug_locals(code);
    let Some(declaration) = declaration else {
        let name = String::from_utf8_lossy(&method.name.0).into_owned();
        let descriptor = String::from_utf8_lossy(&method.descriptor.0).into_owned();
        return jarde_java::RecoveryFacts::new(jarde_java::MethodFacts::new(name, descriptor, 0))
            .with_debug_locals(debug);
    };
    let name = String::from_utf8_lossy(&declaration.name().0).into_owned();
    let descriptor = String::from_utf8_lossy(&declaration.descriptor().0).into_owned();
    jarde_java::RecoveryFacts::new(
        jarde_java::MethodFacts::new(name, descriptor, declaration.parameter_slots())
            .with_access_flags(declaration.access_flags())
            .with_declaring_class(jarde_java::DeclaringClass::new(
                lossy_jvm_name(declaration.class_name()),
                declaration.class_access_flags(),
            )),
    )
    .with_debug_locals(debug)
}

/// One JVM name of the run's own read, as the text the recovery layer states it with.
///
/// This is the display boundary, and the one thing it may not do is change what a name *is*: the
/// payload keeps the class's raw bytes, and this function only spells them for a `String` field of
/// the recovery input — the same lossy Modified-UTF-8 read this entry already applies to the member's
/// name and descriptor above, and the one `jarde-java`'s own decode applies to a pool's names
/// ([`jarde_java::RecoveryFacts`] compares this spelling with the owner of a constructor call, which
/// is spelled the same way). A JVM name is not text for every payload: a surrogate pair or a byte
/// sequence Modified-UTF-8 does not allow becomes the replacement character here, and that is a
/// display limitation of this boundary rather than a claim that the class's name is legal Java. The
/// reader's escaped rendering of a `JvmString` is not reachable from here (a `JvmString` can only be
/// built by a read of its own), and inventing a second spelling here — an ASCII escape, say — would
/// give this layer two names for one class and could no longer be compared with the names the rules
/// read out of the pool.
fn lossy_jvm_name(bytes: &jarde_reader::model::JvmBytes) -> String {
    String::from_utf8_lossy(&bytes.0).into_owned()
}

/// The debug records the body's own `Code` attribute states, as the recovery layer's own evidence
/// type (P3 3.1, 3.4).
///
/// The table is the same read's (see [`jarde_reader::classfile::MethodCodeFacts::debug`]), so this is
/// not a second reading of the class: the names, and the bytecode range each one covers, are the ones
/// the run that decoded the body already had.
///
/// Nothing is decided here. A slot the table names **twice** over disjoint ranges is exactly what a
/// compiler that reuses a storage location states, and the recovered text is *two* variables in that
/// case (P3 3.4): the decision belongs to the recovery layer, which reads the body's own uses, and a
/// reader that dropped the second record would take the evidence away before it could be used. A
/// body the table states nothing for passes no record, which the recovery layer names by ordinal
/// (A10).
fn debug_locals(
    code: Option<&jarde_reader::classfile::MethodCodeFacts>,
) -> Vec<jarde_java::DebugLocal> {
    let Some(code) = code else {
        return Vec::new();
    };
    let jarde_reader::classfile::LocalDebugTable::Read(records) = code.debug() else {
        return Vec::new();
    };
    records
        .iter()
        .map(|record| {
            jarde_java::DebugLocal::over(
                record.slot,
                record.name_lossy(),
                record.start_bci,
                record.end_bci,
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Task-oriented operations: one target, one bounded budget, one explicit environment
// ---------------------------------------------------------------------------------------------
//
// The layer above the entry points. A caller states *what it wants* — this class, this method,
// this body, this declaration's references — and the library answers with the physical identity it
// bound, the stages it really scheduled, the complete configuration it ran under and the report of
// the run itself. Nothing here re-implements navigation, resolution, analysis or recovery: the
// selection consumes [`Engine::find_targets`]'s own rules, the budget is the existing [`Budget`]
// under bounded defaults, and every report publishes the layer's own planes rather than a second
// copy of them.

/// A class a task-oriented request names: the friendly way, or an identity the caller already holds.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClassRef {
    /// One class name in either accepted spelling, matched by the navigation rules over the
    /// request's scope. Several definitions of the name are all returned, never elected.
    Name { class: ClassNameQuery },
    /// The physical definition a listing handed back, used exactly as given: its location, class
    /// bytes and variant are verified before it is read, and a definition of another snapshot is
    /// an input error rather than a same-named substitute.
    Definition { definition: PhysicalDefinitionId },
}

/// A method a task-oriented request names: the friendly way, or an identity the caller already
/// holds.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MethodRef {
    /// A class name plus the member's raw name and, when the caller has one, its raw descriptor.
    /// Several declared overloads are all returned as candidates; the display name never becomes
    /// the identity.
    Name {
        class: ClassNameQuery,
        name: JvmBytes,
        descriptor: Option<JvmBytes>,
    },
    /// The physical method identity a listing handed back, used exactly as given.
    Method { method: PhysicalMethodId },
}

/// One member body a class view asks for on demand.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BodyRef {
    /// A raw method name and, when the caller has one, its raw descriptor. The name is resolved
    /// against the class view's own member listing; every declared descriptor is a candidate until
    /// one is given.
    Name {
        name: JvmBytes,
        descriptor: Option<JvmBytes>,
    },
    /// The physical method identity a member listing handed back.
    Method { method: PhysicalMethodId },
}

/// The candidates one friendly name answered with, and the physical basis for choosing between
/// them.
///
/// A selection that bound several identities and a selection that never finished are both answers,
/// not failures: the operation that returned this executed nothing — no body was read, no stage ran
/// — and the caller continues by handing one candidate's own identity back
/// ([`ClassRef::Definition`], [`MethodRef::Method`], [`BodyRef::Method`]).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetCandidates {
    /// The query as the search answered it, spelling included.
    pub query: NavigationQuery,
    /// The physical definitions or members the search confirmed and published, each with its own
    /// identity. An [`OperationOutcome::Ambiguous`] states every binding the name has; an
    /// [`OperationOutcome::Incomplete`] states the reliable prefix the search obtained before it
    /// stopped, which may be empty.
    pub candidates: Vec<ClassContentItem>,
    /// The complete effective limits the selection ran under.
    pub limits: Limits,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

/// What one task-oriented operation did with its target.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum OperationOutcome<T> {
    /// Exactly one physical identity was bound and the operation ran over it.
    Performed(T),
    /// The friendly name matched more than one physical definition: the operation returns them and
    /// **executed nothing**. Boxed because a candidate list is many items and an outcome of a
    /// performed operation is not.
    Ambiguous(Box<TargetCandidates>),
    /// The friendly name search did not finish — a damaged candidate, an exhausted dimension or a
    /// cancellation stopped it — so nothing may be claimed about the target: the operation returns
    /// the search's own execution, coverage, diagnostics, real usage and the candidates it did
    /// confirm (zero or more) and **executed nothing**.
    ///
    /// This is deliberately not [`OperationOutcome::Ambiguous`]: a search that stopped has not
    /// shown that the name is ambiguous, and it cannot answer "not found" either, because the
    /// target may be in the part it never read. To continue, the caller selects a confirmed
    /// candidate's own physical identity explicitly, or retries under a budget that can finish the
    /// search.
    Incomplete(Box<TargetCandidates>),
}

// ---------------------------------------------------------------------------------------------
// The bounded default budget and its overrides
// ---------------------------------------------------------------------------------------------

/// The budget dimensions a task-oriented request may override, by their snake_case names.
///
/// The set is every **counted** dimension of [`CountedBudgetDimension::ALL`], in that order, and
/// then the wall clock ([`BudgetDimension::ElapsedMillis`]) last. One name here is one name
/// [`BudgetOverride::new`] accepts, [`BudgetOverride::dimension_code`] states back and
/// [`task_limits`] replaces one [`Limits`] field with. A name outside this list is an input error
/// (`budget_override_dimension_unknown`), never a silently ignored field.
///
/// The list is the whole counted set on purpose: a counted dimension is a quantity of work, so a
/// caller that knows how much of it a request needs states the number instead of being refused it —
/// a bulk request over a whole package needs orders of magnitude more derived items and worklist
/// steps than a single view, and a default set that could not be raised to that scale would make
/// every real package an unbudgetable request.
///
/// The two dimensions it deliberately leaves out are the high-water ones,
/// [`BudgetDimension::NestedDepth`] and [`BudgetDimension::DependencyDepth`]. Neither is a count: a
/// run stores the deepest value it accepted, it never accumulates one, and the limit is compared
/// **before** the depth is walked. Raising a depth does not fund more of the work a request named —
/// it decides **which containers and dependencies the request is allowed to walk at all**, which is
/// what the physical scope and the environment declare. A default set that raised them to cover a
/// whole package would silently read containers the request did not name; a caller that really means
/// to walk deeper states that in its scope and its roots. They therefore keep the bounded defaults
/// [`task_limits`] states, and no task override reaches them.
pub const OVERRIDABLE_BUDGET_DIMENSIONS: [&str; 16] = [
    "input_bytes",
    "archive_entries",
    "entry_bytes",
    "read_bytes",
    "class_bytes",
    "attribute_bytes",
    "code_bytes",
    "result_items",
    "output_bytes",
    "class_headers",
    "method_bodies",
    "ir_items",
    "ir_edges",
    "analysis_steps",
    "normalization_clones",
    "elapsed_millis",
];

/// One explicit override of the bounded default budget ([`task_limits`]).
///
/// An override replaces exactly one dimension and leaves the others at their defaults. Every variant
/// is one name of [`OVERRIDABLE_BUDGET_DIMENSIONS`] — the fifteen counted dimensions and the wall
/// clock — and nothing else: a limit of zero is rejected (`budget_override_invalid`), because a
/// dimension that cannot fund one unit of work would make every operation stop before its first
/// charge, which is a degenerate request rather than a tighter budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "dimension", rename_all = "snake_case", deny_unknown_fields)]
pub enum BudgetOverride {
    InputBytes { limit: u64 },
    ArchiveEntries { limit: u64 },
    EntryBytes { limit: u64 },
    ReadBytes { limit: u64 },
    ClassBytes { limit: u64 },
    AttributeBytes { limit: u64 },
    CodeBytes { limit: u64 },
    ResultItems { limit: u64 },
    OutputBytes { limit: u64 },
    ClassHeaders { limit: u64 },
    MethodBodies { limit: u64 },
    IrItems { limit: u64 },
    IrEdges { limit: u64 },
    AnalysisSteps { limit: u64 },
    NormalizationClones { limit: u64 },
    ElapsedMillis { limit: u64 },
}

impl BudgetOverride {
    /// One override named by its snake_case dimension, checked against the closed set.
    pub fn new(dimension: &str, limit: u64) -> Result<Self> {
        let value = match dimension {
            "input_bytes" => Self::InputBytes { limit },
            "archive_entries" => Self::ArchiveEntries { limit },
            "entry_bytes" => Self::EntryBytes { limit },
            "read_bytes" => Self::ReadBytes { limit },
            "class_bytes" => Self::ClassBytes { limit },
            "attribute_bytes" => Self::AttributeBytes { limit },
            "code_bytes" => Self::CodeBytes { limit },
            "result_items" => Self::ResultItems { limit },
            "output_bytes" => Self::OutputBytes { limit },
            "class_headers" => Self::ClassHeaders { limit },
            "method_bodies" => Self::MethodBodies { limit },
            "ir_items" => Self::IrItems { limit },
            "ir_edges" => Self::IrEdges { limit },
            "analysis_steps" => Self::AnalysisSteps { limit },
            "normalization_clones" => Self::NormalizationClones { limit },
            "elapsed_millis" => Self::ElapsedMillis { limit },
            other => {
                return Err(Error::invalid_input(
                    "budget_override_dimension_unknown",
                    format!(
                        "`{other}` is not one of the budget dimensions a task-oriented request may \
                         override ({}); the two high-water depths keep their bounded defaults and \
                         are declared by the scope and the environment instead",
                        OVERRIDABLE_BUDGET_DIMENSIONS.join(", ")
                    ),
                ));
            }
        };
        value.check_limit()?;
        Ok(value)
    }

    /// The snake_case name of the dimension this override replaces.
    pub const fn dimension_code(self) -> &'static str {
        match self {
            Self::InputBytes { .. } => "input_bytes",
            Self::ArchiveEntries { .. } => "archive_entries",
            Self::EntryBytes { .. } => "entry_bytes",
            Self::ReadBytes { .. } => "read_bytes",
            Self::ClassBytes { .. } => "class_bytes",
            Self::AttributeBytes { .. } => "attribute_bytes",
            Self::CodeBytes { .. } => "code_bytes",
            Self::ResultItems { .. } => "result_items",
            Self::OutputBytes { .. } => "output_bytes",
            Self::ClassHeaders { .. } => "class_headers",
            Self::MethodBodies { .. } => "method_bodies",
            Self::IrItems { .. } => "ir_items",
            Self::IrEdges { .. } => "ir_edges",
            Self::AnalysisSteps { .. } => "analysis_steps",
            Self::NormalizationClones { .. } => "normalization_clones",
            Self::ElapsedMillis { .. } => "elapsed_millis",
        }
    }

    /// The limit this override states.
    pub const fn limit(self) -> u64 {
        match self {
            Self::InputBytes { limit }
            | Self::ArchiveEntries { limit }
            | Self::EntryBytes { limit }
            | Self::ReadBytes { limit }
            | Self::ClassBytes { limit }
            | Self::AttributeBytes { limit }
            | Self::CodeBytes { limit }
            | Self::ResultItems { limit }
            | Self::OutputBytes { limit }
            | Self::ClassHeaders { limit }
            | Self::MethodBodies { limit }
            | Self::IrItems { limit }
            | Self::IrEdges { limit }
            | Self::AnalysisSteps { limit }
            | Self::NormalizationClones { limit }
            | Self::ElapsedMillis { limit } => limit,
        }
    }

    fn check_limit(self) -> Result<()> {
        if self.limit() == 0 {
            return Err(Error::invalid_input(
                "budget_override_invalid",
                format!(
                    "the override of `{}` is 0; a dimension that cannot fund one unit of work is \
                     not a budget — raise the limit or leave the bounded default in place",
                    self.dimension_code()
                ),
            ));
        }
        Ok(())
    }
}

/// The complete effective limits one task-oriented request runs under: bounded defaults, then the
/// caller's few explicit overrides.
///
/// This is the one place the default set exists, and it is the [`Limits`] a task entry point runs
/// under (see [`task_budget`]). Every dimension is bounded; no dimension is unbounded and no
/// override may be unknown or zero — both are input errors, and neither falls back to a default
/// silently.
///
/// The defaults are a *single request's* ceilings, and one entry point that is not a single request
/// states its own instead of reading a bigger number out of this function: the bulk export of a
/// whole package replaces every counted dimension with its own finite ceiling — the dimension
/// numbers a real package measured are three orders of magnitude past the ones below — and the
/// header of that stream publishes the result (`add-parallel-bulk-recovery` decision 1, task 1.2).
/// The two high-water depths and the clock are the dimensions no default set raises on a caller's
/// behalf; see [`OVERRIDABLE_BUDGET_DIMENSIONS`] for why.
pub fn task_limits(overrides: &[BudgetOverride]) -> Result<Limits> {
    let mut limits = Limits {
        input_bytes: 1 << 26,
        archive_entries: 1 << 16,
        entry_bytes: 1 << 26,
        read_bytes: 1 << 26,
        class_bytes: 1 << 26,
        attribute_bytes: 1 << 26,
        code_bytes: 1 << 24,
        result_items: 1 << 16,
        output_bytes: 1 << 24,
        class_headers: 4096,
        method_bodies: 4096,
        ir_items: 1 << 22,
        ir_edges: 1 << 22,
        analysis_steps: 1 << 22,
        normalization_clones: 1 << 20,
        nested_depth: 4,
        dependency_depth: 8,
        elapsed_millis: 30_000,
    };
    for over in overrides {
        over.check_limit()?;
        match over {
            BudgetOverride::InputBytes { limit } => limits.input_bytes = *limit,
            BudgetOverride::ArchiveEntries { limit } => limits.archive_entries = *limit,
            BudgetOverride::EntryBytes { limit } => limits.entry_bytes = *limit,
            BudgetOverride::ReadBytes { limit } => limits.read_bytes = *limit,
            BudgetOverride::ClassBytes { limit } => limits.class_bytes = *limit,
            BudgetOverride::AttributeBytes { limit } => limits.attribute_bytes = *limit,
            BudgetOverride::CodeBytes { limit } => limits.code_bytes = *limit,
            BudgetOverride::ResultItems { limit } => limits.result_items = *limit,
            BudgetOverride::OutputBytes { limit } => limits.output_bytes = *limit,
            BudgetOverride::ClassHeaders { limit } => limits.class_headers = *limit,
            BudgetOverride::MethodBodies { limit } => limits.method_bodies = *limit,
            BudgetOverride::IrItems { limit } => limits.ir_items = *limit,
            BudgetOverride::IrEdges { limit } => limits.ir_edges = *limit,
            BudgetOverride::AnalysisSteps { limit } => limits.analysis_steps = *limit,
            BudgetOverride::NormalizationClones { limit } => limits.normalization_clones = *limit,
            BudgetOverride::ElapsedMillis { limit } => limits.elapsed_millis = *limit,
        }
    }
    Ok(limits)
}

/// The same effective configuration as the [`Budget`] a task entry point takes.
///
/// A caller that wants a cancellation token, a facts cache or a second observation of the same
/// limits builds its budget from [`task_limits`] with the existing [`Budget`] constructors; this
/// is the shortcut for the ordinary case.
pub fn task_budget(overrides: &[BudgetOverride]) -> Result<Budget> {
    Ok(Budget::new(task_limits(overrides)?))
}

// ---------------------------------------------------------------------------------------------
// Environment policies: three explicit shapes, and no inferred classpath
// ---------------------------------------------------------------------------------------------

/// One explicit environment shape a task-oriented request declares.
///
/// Each policy declares its roots, delegation and module mode from the caller's own statement and
/// nothing else: no Manifest `Class-Path` entry, no nested library and no detected layout ever
/// generates a root. The built environment is the very declaration [`EnvironmentRequest::build`]
/// returns, so a caller can validate it with the engine's own
/// [`crate::environment::validate_environment`] and compare it with a hand-written one.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EnvironmentPolicy {
    /// The request's own snapshot is one whole CLASS file: the root is that snapshot, at its own
    /// declared name, and no container or entry identity is fabricated for it.
    SingleClass,
    /// The request's own snapshot is a plain JAR: the root is that snapshot's own root container at
    /// the container's own root (empty prefix). A nested library the archive happens to hold is
    /// **not** activated: a class inside it still needs the explicit artifact-tree root that names
    /// its container.
    PlainJar,
    /// The caller's own roots, in the caller's order: the search takes the first position that
    /// provides a name, so the declaration order decides between same-named definitions and is the
    /// reason such a lookup is not `Ambiguous`.
    ExplicitClasspath { roots: Vec<LoadRoot> },
    /// A container layout this stage is asked to organize roots for. It provides none: a layout
    /// detection is evidence about paths, not a load policy, and a root set that claimed to equal
    /// the container's own loading would be a claim this engine cannot make.
    Layout { mode: LayoutMode },
}

/// One environment declaration: the physical view it belongs to, the policy that states its roots,
/// and the runtime profile it runs under.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentRequest {
    /// The snapshot (and scope) the runtime view names — the artifact this request is about.
    pub snapshot: SnapshotId,
    pub scope: PhysicalScope,
    pub policy: EnvironmentPolicy,
    pub profile: RuntimeProfile,
    /// The one loader that owns the one domain this request declares.
    pub loader: LoaderId,
}

impl EnvironmentRequest {
    /// Builds the [`ResolutionEnvironment`] this request declares.
    ///
    /// The three provided policies build the same declaration a caller would write by hand for the
    /// same shapes: one loader, one domain, no parent, `parent_first` delegation, `class_path`
    /// module mode, no provider, and roots that name exactly the caller's content. `single_class`
    /// and `plain_jar` check the snapshot's own kind and refuse a mismatch instead of inventing a
    /// root for bytes of another shape. A root that names content the request did not provide is
    /// left in the declaration for the existing validator to report
    /// (`content_not_provided`/`unreadable_root`): a rejected environment keeps its honest
    /// unavailable state and its original symbols rather than being silently rewritten.
    ///
    /// Kind checks are skipped for a snapshot the request did not provide, because the validator
    /// owns that finding; and a `layout` policy is `environment_policy_layout_not_provided` — an
    /// explicit unsupported answer, never a set of guessed roots.
    pub fn build(&self, content: &[ArtifactSnapshot]) -> Result<ResolutionEnvironment> {
        let provided = content
            .iter()
            .find(|candidate| candidate.id() == &self.snapshot);
        let roots = match &self.policy {
            EnvironmentPolicy::SingleClass => {
                require_policy_kind(
                    provided,
                    self,
                    ArtifactKind::StandaloneClass,
                    "single_class",
                )?;
                vec![LoadRoot::StandaloneClass {
                    snapshot: self.snapshot.clone(),
                }]
            }
            EnvironmentPolicy::PlainJar => {
                require_policy_kind(provided, self, ArtifactKind::Zip, "plain_jar")?;
                vec![LoadRoot::Container {
                    origin: ContainerOrigin {
                        snapshot: self.snapshot.clone(),
                        root_container: ContainerId(ROOT_CONTAINER.to_owned()),
                        steps: Vec::new(),
                    },
                    prefix: ArchiveNameBytes(Vec::new()),
                }]
            }
            EnvironmentPolicy::ExplicitClasspath { roots } => roots.clone(),
            EnvironmentPolicy::Layout { mode } => {
                return Err(Error::unsupported(
                    "environment_policy_layout_not_provided",
                    format!(
                        "this stage provides no `{}` layout policy: roots are never organized from \
                         a detected `WEB-INF/classes/` or `BOOT-INF` layout, and a set of roots \
                         that claimed to equal the container's own loading is not an answer this \
                         engine may give",
                        layout_code(mode)
                    ),
                ));
            }
        };
        let domain = LoadDomain {
            loader: self.loader.clone(),
            parent_loader: None,
            delegation: DelegationPolicy::ParentFirst,
            roots,
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        };
        Ok(ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: self.snapshot.clone(),
                    scope: self.scope.clone(),
                },
                profile: self.profile.clone(),
                load_domain: domain.clone(),
            },
            domains: vec![domain],
            providers: Vec::new(),
        })
    }
}

/// One policy's own root shape against the snapshot's actual kind.
///
/// `Ok(())` when the snapshot was provided and has the kind the policy names, and also when the
/// snapshot was not provided at all — that finding belongs to the environment validator, which
/// reports it as a problem on the same declaration.
fn require_policy_kind(
    provided: Option<&ArtifactSnapshot>,
    request: &EnvironmentRequest,
    expected: ArtifactKind,
    policy: &str,
) -> Result<()> {
    let Some(snapshot) = provided else {
        return Ok(());
    };
    if snapshot.kind() == expected {
        return Ok(());
    }
    Err(Error::invalid_input(
        "environment_policy_snapshot_kind_mismatch",
        format!(
            "the `{policy}` policy is declared over snapshot `{}`, whose kind is not the one the \
             policy names; a root is never invented for another kind of bytes",
            request.snapshot.0
        ),
    ))
}

/// The snake_case name of one declared layout mode.
fn layout_code(mode: &LayoutMode) -> String {
    match mode {
        LayoutMode::Generic => "generic".to_owned(),
        LayoutMode::War => "war".to_owned(),
        LayoutMode::SpringBoot => "spring_boot".to_owned(),
        LayoutMode::Custom { id } => format!("custom ({id})"),
        LayoutMode::Unknown => "unknown".to_owned(),
    }
}

// ---------------------------------------------------------------------------------------------
// The class view: one class read, one member listing, on-demand bodies
// ---------------------------------------------------------------------------------------------

/// One class view request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassViewRequest {
    /// The class to view: a friendly name or an identity the caller holds.
    pub class: ClassRef,
    /// The method bodies to read on demand, in request order. Empty reads no body at all, and the
    /// whole request still charges exactly one class header and one member walk.
    pub bodies: Vec<BodyRef>,
}

/// The class one view read, its members and the bodies it decoded on demand.
///
/// `items` is the same vocabulary a member listing publishes — the class-level item first, then its
/// fields, then its methods, each charged as one result item — and `bodies` holds one entry per
/// requested body, in request order. `class` is the identity the view bound: the definition the
/// name search confirmed, or the identity the caller gave, verified against this snapshot's bytes.
///
/// The planes are this view's own: `limits` is the complete effective configuration the request ran
/// under and `usage` is what that configuration was charged, `coverage` states the candidate search
/// (when the class was named) beside the member tables under their own labels, and `execution`
/// merges the search, the class read and every body-level stop — a body that stopped makes this
/// report non-`Complete` even though the class and member planes of the same request did finish.
/// Each body keeps its own result, coverage and diagnostics, and a [`ClassViewBody::NotDeclared`]
/// member is a declaration rather than a stop. The class and member structural coverage keeps its
/// own evidence rather than being rewritten by a body: a member table that really was read to its
/// declared end stays `complete_within_schema` beside a stopped body, and one that stopped reports
/// its scanned prefix and skipped remainder under `class_fields`/`class_methods`. Nothing here is a
/// resolution, a recovery or a verification statement, and no analysis plane (`ir_items`,
/// `ir_edges`, `analysis_steps`, `normalization_clones`) moves.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassViewReport {
    pub view: PhysicalView,
    /// The physical definition this view is of.
    pub class: PhysicalDefinitionId,
    pub items: Vec<ClassContentItem>,
    pub bodies: Vec<ClassViewBody>,
    pub limits: Limits,
    pub usage: UsageSnapshot,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

impl ClassViewReport {
    /// The class-level facts of this view, when the item itself was published.
    pub fn declaration(&self) -> Option<&ClassDeclarationItem> {
        self.items.iter().find_map(|item| match item {
            ClassContentItem::ClassDeclaration(class) => Some(class),
            ClassContentItem::Field(_) | ClassContentItem::Method(_) => None,
        })
    }

    /// The field records this view published, in declaration order.
    pub fn fields(&self) -> impl Iterator<Item = &FieldItem> {
        self.items.iter().filter_map(|item| match item {
            ClassContentItem::Field(field) => Some(field),
            ClassContentItem::ClassDeclaration(_) | ClassContentItem::Method(_) => None,
        })
    }

    /// The method records this view published, in declaration order.
    pub fn methods(&self) -> impl Iterator<Item = &MethodItem> {
        self.items.iter().filter_map(|item| match item {
            ClassContentItem::Method(method) => Some(method),
            ClassContentItem::ClassDeclaration(_) | ClassContentItem::Field(_) => None,
        })
    }

    /// The body read of one requested method, when the request asked for it.
    pub fn body(&self, method: &PhysicalMethodId) -> Option<&ClassViewBody> {
        self.bodies
            .iter()
            .find(|body| body.method() == Some(method))
    }

    /// The body result of one requested reference, when the request asked for it.
    pub fn reference_body(&self, reference: &BodyRef) -> Option<&ClassViewBody> {
        self.bodies
            .iter()
            .find(|body| &body.reference() == reference)
    }
}

/// One on-demand method body of a class view.
///
/// Every variant keeps the member's identity, so a body result is bound to exactly one member of
/// the class the view read. A member-level failure is isolated to its own variant and does not
/// erase the class, the members or the other bodies (A13).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[allow(
    clippy::large_enum_variant,
    reason = "a report value built once per request, where boxing a variant would only move the \
              body's own fields behind a pointer for pattern matches to undo"
)]
pub enum ClassViewBody {
    /// The member's own `Code` attribute was decoded, out of the very bytes the view's one class
    /// read materialized. `stages` are that decode's own two phases, `coverage` is that decode's
    /// plane and `execution` its stop or completion — none of them is an analysis stage result.
    Read {
        method: PhysicalMethodId,
        stages: Vec<BodyStageResult>,
        max_stack: u16,
        max_locals: u16,
        code_span: ByteSpan,
        instructions: Vec<InstructionFact>,
        exception_handlers: Vec<ExceptionHandlerFact>,
        exception_handler_count: u32,
        stopped_at: Option<BytecodeStop>,
        coverage: Coverage,
        execution: ExecutionReport,
        diagnostics: Vec<Diagnostic>,
    },
    /// The member declares no `Code` attribute. `no_body_kind` is `Some` exactly when the member's
    /// own flags declare it `abstract` or `native`; it is `None` for a member that carries neither
    /// flag and still declares no body, which this value states as the bytes state it. No empty
    /// body is invented and no `method_bodies` attempt is charged.
    NotDeclared {
        method: PhysicalMethodId,
        no_body_kind: Option<NoBodyKind>,
    },
    /// One member-level refusal, isolated to this member: a read that failed after the reference
    /// resolved, or a member the class's member walk did not reach because a damaged record stopped
    /// it. The other bodies and the class declaration keep their own results (A13).
    ///
    /// `reference` is the request's own reference as it was made, and `method` is the identity it
    /// resolved to when it resolved at all: a member the walk never reached has no identity this
    /// layer may state, so none is invented for it.
    Refused {
        reference: BodyRef,
        method: Option<PhysicalMethodId>,
        execution: ExecutionReport,
        diagnostics: Vec<Diagnostic>,
    },
}

impl ClassViewBody {
    /// The member identity this body result is bound to, when it has one: every read, every
    /// no-body declaration, and a refusal whose reference resolved before the read failed.
    pub fn method(&self) -> Option<&PhysicalMethodId> {
        match self {
            Self::Read { method, .. } | Self::NotDeclared { method, .. } => Some(method),
            Self::Refused { method, .. } => method.as_ref(),
        }
    }

    /// The request's own body reference, exactly as it was made.
    pub fn reference(&self) -> BodyRef {
        match self {
            Self::Read { method, .. } | Self::NotDeclared { method, .. } => BodyRef::Method {
                method: method.clone(),
            },
            Self::Refused { reference, .. } => reference.clone(),
        }
    }
}

/// One phase of a body read, in the reader's own two-phase order: the exception handlers are read
/// before the instruction stream.
///
/// This is the reader's phase vocabulary of a single body decode, not the analysis pipeline's
/// [`AnalysisStage`]: a class view decodes bodies and never runs the pipeline.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyStageResult {
    pub phase: BytecodeStopPhase,
    pub state: BodyStageState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum BodyStageState {
    /// The earlier phase stopped, so this one was never entered.
    NotReached,
    /// The phase ran to the end of its own range.
    Completed,
    /// The phase stopped under the reader's own code.
    Stopped { code: String },
}

// ---------------------------------------------------------------------------------------------
// Task-oriented method operations: the operation picks its stages and publishes them
// ---------------------------------------------------------------------------------------------

/// The task-oriented method operations, each with its own fixed stage table.
///
/// The caller states the operation, never a stage list: the table below is the library's, it is
/// published in the report the request produced, and the same list passed explicitly to
/// [`Engine::analyze_method`] reproduces the same schedule. The explicit stage list stays the
/// low-level control, with its own validation and stop semantics, unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MethodOperation {
    /// The method analysis alone: the run's own report, payload not presented.
    Analysis,
    /// The method analysis and the presentation of the same run's payload.
    Recovery,
}

impl MethodOperation {
    pub const ALL: [Self; 2] = [Self::Analysis, Self::Recovery];

    /// The stages this operation schedules, in the fixed phase order.
    ///
    /// Both operations of today's table schedule the whole fixed pipeline: the recovery layer reads
    /// the canonical, SSA and code tables of the payload and the analysis operation hands the same
    /// payload out, so neither can stop short of `ssa`. An operation whose answer needed a shorter
    /// prefix declares its own set here; the entry points never take a stage list from the caller.
    pub fn stages(self) -> &'static [AnalysisStage] {
        &AnalysisStage::ALL
    }

    /// The snake_case name of this operation.
    pub const fn code(self) -> &'static str {
        match self {
            Self::Analysis => "analysis",
            Self::Recovery => "recovery",
        }
    }
}

/// One task-oriented method request: the target and the environment it runs under.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodOperationRequest {
    pub method: MethodRef,
    pub environment: EnvironmentRequest,
}

/// One analysis run performed by a task-oriented operation, with the configuration it ran under.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodOperationReport {
    pub operation: MethodOperation,
    /// The physical identity the operation bound and ran over.
    pub method: PhysicalMethodId,
    /// The stages this operation's table scheduled, in phase order — the very list the run was
    /// asked for explicitly.
    pub stages: Vec<AnalysisStage>,
    /// The complete effective limits of this request.
    pub limits: Limits,
    pub usage: UsageSnapshot,
    pub analysis: crate::ir::MethodAnalysisReport,
}

/// One recovery run performed by a task-oriented operation: the same run's analysis, presentation
/// and on-demand callee evidence, beside the configuration and stages they ran under.
#[derive(Clone, Debug, Serialize)]
pub struct MethodRecoveryReport {
    pub operation: MethodOperation,
    pub method: PhysicalMethodId,
    pub stages: Vec<AnalysisStage>,
    pub limits: Limits,
    pub usage: UsageSnapshot,
    pub recovered: RecoveredMethod,
    /// The presentation of [`Self::recovered`]'s own recovery report, content first.
    pub presentation: RecoveryPresentation,
}

// ---------------------------------------------------------------------------------------------
// Reference results: organised by owning method, derivation classes kept apart
// ---------------------------------------------------------------------------------------------

/// The derivation class one reference finding was produced under.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceFindingClass {
    /// A constant-pool occurrence no consumer used: a candidate, never a call.
    ConstantPoolCandidate,
    /// A use-site a consumer really used, or a bootstrap edge of one; the raw symbol stays.
    StructuralReference,
    /// A use-site resolved to the declaration the caller named, under the stated environment.
    ResolvedDeclaration,
}

/// One reference finding, keeping the shape the scan that produced it published.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReferenceFinding {
    /// From a [`QueryReport`]: a constant-pool occurrence with no consumer.
    ConstantPoolCandidate { item: XrefItem },
    /// From a [`QueryReport`]: a use-site a consumer really used, or a bootstrap edge. The item's
    /// own `derivation` still tells the two apart.
    StructuralReference { item: XrefItem },
    /// From a [`DeclarationRefReport`]: a use-site that resolved to the requested declaration under
    /// the scan's explicit environment, with its own `state` and `resolved` evidence untouched.
    ResolvedDeclaration { item: DeclarationRefItem },
}

impl ReferenceFinding {
    /// The derivation class of this finding.
    pub fn class(&self) -> ReferenceFindingClass {
        match self {
            Self::ConstantPoolCandidate { .. } => ReferenceFindingClass::ConstantPoolCandidate,
            Self::StructuralReference { .. } => ReferenceFindingClass::StructuralReference,
            Self::ResolvedDeclaration { .. } => ReferenceFindingClass::ResolvedDeclaration,
        }
    }

    /// The physical origin this finding keeps: a use-site provenance for a query item, the scan's
    /// own origin set for a declaration item.
    pub fn bci(&self) -> Option<u32> {
        match self {
            Self::ConstantPoolCandidate { item } | Self::StructuralReference { item } => {
                item.evidence.bci
            }
            Self::ResolvedDeclaration { item } => {
                item.origin.members.iter().find_map(|member| match member {
                    OriginMember::MethodPoint { bci, .. } => Some(*bci),
                    OriginMember::ClassFile { .. } | OriginMember::ClassRange { .. } => None,
                })
            }
        }
    }
}

/// The findings one owning method holds, in first-appearance order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodReferences {
    /// The physical method that owns every finding in this group.
    pub method: PhysicalMethodId,
    pub findings: Vec<ReferenceFinding>,
}

/// The planes of the scan a [`ReferenceGrouping`] organises, exactly as the scan published them.
///
/// One report is one source: a grouping never merges two scans and never re-derives a plane. The
/// item list is the only thing the grouping restructures, and every item it moved is here complete
/// — including an unresolved candidate's count and diagnostics, which stay with the declaration
/// source and are never completed by a group.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[allow(
    clippy::large_enum_variant,
    reason = "one source report per grouping; a wire-shaped document whose variants carry their \
              scan's own planes, with nothing to gain from indirection"
)]
pub enum ReferenceSource {
    Query {
        physical: PhysicalView,
        relation: QueryRelation,
        consumers: ConsumerSchema,
        analysis: QueryAnalysis,
        page: QueryPage,
        coverage: QueryCoverage,
        execution: ExecutionReport,
        diagnostics: Vec<Diagnostic>,
    },
    Declaration {
        environment_identity: EnvironmentIdentity,
        environment_problems: Vec<EnvironmentProblem>,
        declaration: ResolvedMemberRef,
        scope: PhysicalScope,
        consumers: ConsumerSchema,
        unsupported_categories: Vec<ConsumerKind>,
        analysis: ResolutionAnalysis,
        unresolved_candidates: u64,
        has_more: bool,
        returned_items: u64,
        reads: Vec<HeaderRead>,
        coverage: Coverage,
        execution: ExecutionReport,
        diagnostics: Vec<Diagnostic>,
    },
}

/// One reference result set organised by the method that owns each hit.
///
/// A hit whose position is inside a method body is grouped under that method's physical identity,
/// with its BCI and its own evidence; a class-level or entry/resource position keeps its own place
/// and is assigned to no method. No item is renamed, no derivation class is flattened and no
/// finding is dropped: [`Self::source`] holds the scan's own planes and every item the scan
/// published appears exactly once across the groups.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceGrouping {
    pub source: ReferenceSource,
    /// Body hits, grouped by owning method, in first-appearance order.
    pub methods: Vec<MethodReferences>,
    /// Class-level hits, in source order; no method owns them.
    pub class_level: Vec<ReferenceFinding>,
    /// Entry and resource hits, in source order; no method owns them.
    pub resources: Vec<ReferenceFinding>,
}

impl ReferenceGrouping {
    /// Organises one query report by owning method.
    pub fn from_query(report: QueryReport) -> Self {
        let QueryReport {
            physical,
            relation,
            consumers,
            analysis,
            items,
            page,
            coverage,
            execution,
            diagnostics,
        } = report;
        let mut methods = Vec::new();
        let mut class_level = Vec::new();
        let mut resources = Vec::new();
        for item in items {
            let owner = owner_of_location(&item.source.location);
            let finding = match item.derivation {
                XrefDerivation::ConstantPoolCandidate => {
                    ReferenceFinding::ConstantPoolCandidate { item }
                }
                XrefDerivation::StructuralConsumer | XrefDerivation::BootstrapEdge => {
                    ReferenceFinding::StructuralReference { item }
                }
            };
            place_finding(
                &mut methods,
                &mut class_level,
                &mut resources,
                owner,
                finding,
            );
        }
        Self {
            source: ReferenceSource::Query {
                physical,
                relation,
                consumers,
                analysis,
                page,
                coverage,
                execution,
                diagnostics,
            },
            methods,
            class_level,
            resources,
        }
    }

    /// Organises one declaration-reference report by owning method.
    pub fn from_declaration(report: DeclarationRefReport) -> Self {
        let DeclarationRefReport {
            environment_identity,
            environment_problems,
            declaration,
            scope,
            consumers,
            unsupported_categories,
            analysis,
            items,
            unresolved_candidates,
            has_more,
            returned_items,
            reads,
            coverage,
            execution,
            diagnostics,
        } = report;
        let mut methods = Vec::new();
        let mut class_level = Vec::new();
        let mut resources = Vec::new();
        for item in items {
            let owners = origin_owners(&item.origin);
            let finding = ReferenceFinding::ResolvedDeclaration { item };
            for owner in owners {
                place_finding(
                    &mut methods,
                    &mut class_level,
                    &mut resources,
                    owner,
                    finding.clone(),
                );
            }
        }
        Self {
            source: ReferenceSource::Declaration {
                environment_identity,
                environment_problems,
                declaration,
                scope,
                consumers,
                unsupported_categories,
                analysis,
                unresolved_candidates,
                has_more,
                returned_items,
                reads,
                coverage,
                execution,
                diagnostics,
            },
            methods,
            class_level,
            resources,
        }
    }

    /// How many findings the grouping holds, across every group and both unowned lists.
    pub fn finding_count(&self) -> usize {
        self.methods
            .iter()
            .map(|group| group.findings.len())
            .sum::<usize>()
            + self.class_level.len()
            + self.resources.len()
    }
}

/// Which method owns one position, or that none does.
enum FindingOwner {
    /// Boxed because one owner alternative is one physical identity and the other two carry none.
    Method(Box<PhysicalMethodId>),
    ClassLevel,
    Resource,
}

fn owner_of_location(location: &Location) -> FindingOwner {
    match location {
        Location::Code { method, .. } => FindingOwner::Method(Box::new(method.clone())),
        Location::ClassOffset { .. } | Location::Attribute { .. } => FindingOwner::ClassLevel,
        Location::Entry { .. } | Location::Resource { .. } | Location::Container { .. } => {
            FindingOwner::Resource
        }
    }
}

/// The owners one origin set names: every distinct method point in order, or the class level when
/// only class-file coordinates are there, or the resource side when there is nothing narrower.
fn origin_owners(origin: &OriginSet) -> Vec<FindingOwner> {
    let mut methods: Vec<PhysicalMethodId> = Vec::new();
    let mut class_level = false;
    for member in &origin.members {
        match member {
            OriginMember::MethodPoint { method, .. } => {
                if !methods.contains(method) {
                    methods.push(method.clone());
                }
            }
            OriginMember::ClassFile { .. } | OriginMember::ClassRange { .. } => class_level = true,
        }
    }
    if !methods.is_empty() {
        methods
            .into_iter()
            .map(|method| FindingOwner::Method(Box::new(method)))
            .collect()
    } else if class_level {
        vec![FindingOwner::ClassLevel]
    } else {
        vec![FindingOwner::Resource]
    }
}

fn place_finding(
    methods: &mut Vec<MethodReferences>,
    class_level: &mut Vec<ReferenceFinding>,
    resources: &mut Vec<ReferenceFinding>,
    owner: FindingOwner,
    finding: ReferenceFinding,
) {
    match owner {
        FindingOwner::Method(method) => {
            if let Some(group) = methods.iter_mut().find(|group| group.method == *method) {
                group.findings.push(finding);
            } else {
                methods.push(MethodReferences {
                    method: *method,
                    findings: vec![finding],
                });
            }
        }
        FindingOwner::ClassLevel => class_level.push(finding),
        FindingOwner::Resource => resources.push(finding),
    }
}

// ---------------------------------------------------------------------------------------------
// Recovery presentation: content first, then quality, then any stop
// ---------------------------------------------------------------------------------------------

/// The presentation order of one recovery report.
///
/// Every value is read from the report's own committed fields — [`RecoveryReport::content`],
/// [`RecoveryReport::quality`], [`RecoveryReport::outcome`] and [`RecoveryReport::execution`] — and
/// nothing here reads [`RecoveryReport::text`]: no comment is stripped, no token is counted and no
/// text is parsed, so a message that spells `return` changes nothing. The presentation keeps the
/// existing contracts as they are: `content` is not a claim of completeness, compilability or
/// semantic equivalence, and the representation, syntax, compile, semantic and verification planes
/// stay the recovery report's own.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryPresentation {
    /// What the delivered artifact holds, from [`RecoveryReport::content`].
    pub content: RecoveryContent,
    /// How strong the produced structure is, from [`RecoveryReport::quality`].
    pub quality: Quality,
    /// Why the run stopped before delivering an artifact, from [`RecoveryReport::outcome`]; `None`
    /// exactly when an artifact was delivered.
    pub stop: Option<StopReason>,
    /// The execution plane of the same report, echoed.
    pub execution: ExecutionReport,
}

impl RecoveryPresentation {
    /// Reads one recovery report's own fields; nothing is re-analysed.
    pub fn of(report: &RecoveryReport) -> Self {
        Self {
            content: report.content.clone(),
            quality: report.quality,
            stop: report.outcome.stop().cloned(),
            execution: report.execution.clone(),
        }
    }

    /// The parts in the order a host presents them: content, then quality, then the stop when there
    /// is one.
    pub fn parts(&self) -> Vec<RecoveryPresentationPart> {
        let mut parts = vec![
            RecoveryPresentationPart::Content {
                content: self.content.clone(),
            },
            RecoveryPresentationPart::Quality {
                quality: self.quality,
            },
        ];
        if let Some(stop) = &self.stop {
            parts.push(RecoveryPresentationPart::Stop { stop: stop.clone() });
        }
        parts
    }
}

/// One ordered part of a [`RecoveryPresentation`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "part", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecoveryPresentationPart {
    Content { content: RecoveryContent },
    Quality { quality: Quality },
    Stop { stop: StopReason },
}

// ---------------------------------------------------------------------------------------------
// The one name search both the navigation entry and the class view bind through
// ---------------------------------------------------------------------------------------------

/// One named-class search over a scope: the reads that confirmed the requested name, the items the
/// search published for them, and every plane it stated.
///
/// This is the one implementation of the friendly-name rules — the raw-path boundary test, the
/// declaration confirmation, the path/declaration mismatch, the member-table stop, the item charges
/// and the stop — so the navigation entry and a task-oriented operation cannot answer "which
/// definition" differently. What the search does *not* do is decide between its own answers: a
/// caller reads [`NamedClassSearch::execution`] to know whether the search finished, and only then
/// reads the candidate counts as one, zero or many.
struct NamedClassSearch {
    matches: Vec<ConfirmedRead>,
    items: Vec<ClassContentItem>,
    total: u64,
    searched: u64,
    coverage: Coverage,
    execution: ExecutionReport,
    diagnostics: Vec<Diagnostic>,
}

/// The members one confirmed read contributes to a navigation query's item list.
///
/// The class-level item for a query that asks for the class itself, and the field/method records
/// the filter admits for a query that names a member; each is built from the same read that
/// confirmed the class.
fn member_candidates(read: &ConfirmedRead, filter: &MemberQuery) -> Result<Vec<ClassContentItem>> {
    let ClassContentItem::ClassDeclaration(class) = &read.class else {
        unreachable!("a class declaration read publishes a class declaration item")
    };
    let mut found = Vec::new();
    for (index, field) in read.facts.fields.iter().enumerate() {
        let item = field_item(&class.definition, index, field)?;
        if filter.matches(&item) {
            found.push(item);
        }
    }
    for (index, method) in read.facts.methods.iter().enumerate() {
        let item = method_item(&class.definition, index, method)?;
        if filter.matches(&item) {
            found.push(item);
        }
    }
    Ok(found)
}

/// What one name search is answering with.
///
/// The two questions rest on different evidence, and a member table that stopped separates them. A
/// class declaration is established by the header the read confirmed, so the members beyond a stop
/// do not change *which* definition the name bound. A member answer *is* a walk over that table: the
/// records past the stop were never read, so the search cannot claim either a unique match or the
/// absence of one, and it states the stop instead.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SearchSubject {
    /// The class declaration each candidate's own header states.
    Declaration,
    /// The member records of the requested name, selected from the candidate's own member table.
    Member,
}

/// The one search behind the navigation entry and the method binding.
///
/// "Which definitions of this name exist, which of them are confirmed and what does each declare"
/// is answered once, here, so [`Engine::find_targets`] and [`bind_method`] cannot drift in what
/// they examine, in the items they select or in the stop they publish: the report entry publishes
/// this search's own planes, and the method binding elects the read this search confirmed.
fn search_targets(
    snapshot: &ArtifactSnapshot,
    scope: &PhysicalScope,
    query: &NavigationQuery,
    budget: &mut Budget,
) -> Result<NamedClassSearch> {
    search_named_classes(
        snapshot,
        scope,
        &query.class,
        match &query.member {
            None => SearchSubject::Declaration,
            Some(_) => SearchSubject::Member,
        },
        |read| match &query.member {
            None => Ok(vec![read.class.clone()]),
            Some(filter) => member_candidates(read, filter),
        },
        budget,
    )
}

fn search_named_classes<F>(
    snapshot: &ArtifactSnapshot,
    scope: &PhysicalScope,
    class: &ClassNameQuery,
    subject: SearchSubject,
    mut select: F,
    budget: &mut Budget,
) -> Result<NamedClassSearch>
where
    F: FnMut(&ConfirmedRead) -> Result<Vec<ClassContentItem>>,
{
    let scan = scan_scope(snapshot, scope, budget)?;
    let requested = class.internal_name();
    let candidates: Vec<ClassCandidate<'_>> =
        scope_class_candidates(snapshot.kind(), snapshot.id(), &scan.entries)
            .into_iter()
            .filter(|candidate| candidate_states_name(candidate, &requested.0))
            .collect();
    let total = to_u64(candidates.len())?;
    let mut matches = Vec::new();
    let mut items = Vec::new();
    let mut diagnostics = scan.diagnostics;
    let mut execution = scan.execution;
    let mut searched = 0_u64;
    for candidate in &candidates {
        let provenance = candidate_provenance(candidate);
        if let Err(error) = budget.charge(CountedBudgetDimension::ClassHeaders, 1) {
            merge_execution(&mut execution, stop_execution(&error, budget));
            diagnostics.push(stop_diagnostic(&error, provenance));
            break;
        }
        searched = searched.saturating_add(1);
        match read_class_declaration(snapshot, candidate, budget) {
            Ok(mut read) => {
                if let Err(error) = publish_diagnostics(
                    std::mem::take(&mut read.diagnostics),
                    &mut diagnostics,
                    budget,
                ) {
                    merge_execution(&mut execution, stop_execution(&error, budget));
                    diagnostics.push(stop_diagnostic(&error, provenance));
                    break;
                }
                let ClassContentItem::ClassDeclaration(item) = &read.class else {
                    unreachable!("a class declaration read publishes a class declaration item")
                };
                if item.declaration.this_class.raw().0 != requested.0 {
                    if let Err(error) = charge_item(budget) {
                        merge_execution(&mut execution, stop_execution(&error, budget));
                        diagnostics.push(stop_diagnostic(&error, provenance));
                        break;
                    }
                    // The finding is about the entry's own path, so it is stated for a candidate
                    // that has one: a standalone root states no path to contradict.
                    if let Some(entry) = candidate.location.entry() {
                        diagnostics.push(path_name_mismatch_diagnostic(
                            entry,
                            item.definition.class_bytes.length,
                            &requested.0,
                            &item.declaration.this_class,
                        ));
                    }
                    continue;
                }
                let found = select(&read)?;
                let mut refused = false;
                for found_item in found {
                    if let Err(error) = charge_item(budget) {
                        merge_execution(&mut execution, stop_execution(&error, budget));
                        diagnostics.push(stop_diagnostic(&error, provenance.clone()));
                        refused = true;
                        break;
                    }
                    items.push(found_item);
                }
                // A member answer is a walk over exactly this table: when the table stopped, the
                // records past the stop were never read, so the search ends here with the stop
                // stated and the confirmed prefix published rather than answering a member
                // selection the truncated table cannot support.
                let member_stop = match subject {
                    SearchSubject::Member => read.facts.stopped_at.clone(),
                    SearchSubject::Declaration => None,
                };
                let definition = item.definition.clone();
                matches.push(read);
                if refused {
                    break;
                }
                if let Some(stop) = member_stop {
                    merge_execution(&mut execution, member_stop_execution(&stop, budget));
                    diagnostics.push(member_stop_diagnostic(&definition, &stop));
                    break;
                }
            }
            Err(error) => {
                merge_execution(&mut execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, provenance));
                break;
            }
        }
    }
    let complete = matches!(execution, ExecutionReport::Complete { .. });
    Ok(NamedClassSearch {
        matches,
        items,
        total,
        searched,
        coverage: listing_coverage(
            &scan.physical_coverage,
            "navigation_candidates",
            searched,
            total,
            complete,
        ),
        execution: with_usage(execution, budget.usage()),
        diagnostics,
    })
}

// ---------------------------------------------------------------------------------------------
// Binding the class view and the method operations to one physical identity
// ---------------------------------------------------------------------------------------------

/// What one class-level selection bound, or the candidates that refuse to be one.
enum ClassBinding {
    Bound(Box<BoundClass>),
    Ambiguous(Box<TargetCandidates>),
    /// The name search did not finish, so no definition may be elected — and none may be reported
    /// missing either, because the target may be in the part the search never read.
    Incomplete(Box<TargetCandidates>),
}

/// One bound class: the read that confirmed it, the search plane that found it (when it was named)
/// and the class item that read published as this view's own.
struct BoundClass {
    read: ConfirmedRead,
    search_coverage: Option<Coverage>,
    class_item: Option<ClassContentItem>,
}

/// One bound method: the identity the operation runs over, the environment it runs in, and the
/// trusted read the binding itself performed, when it performed one.
struct BoundMethod {
    method: PhysicalMethodId,
    environment: ResolutionEnvironment,
    /// The read of the definition this binding elected, when a name search confirmed it (task 3.1).
    ///
    /// A search reads the definitions it examines, so the definition it elects has already been
    /// read: this is that read, handed to the operation so the selected definition is never read a
    /// second time. The identity path binds from the caller's own identity and reads nothing, so it
    /// has none.
    read: Option<ConfirmedRead>,
}

/// What one method-level selection bound, or the candidates that refuse to be one.
enum MethodBinding {
    Bound(Box<BoundMethod>),
    Ambiguous(Box<TargetCandidates>),
    /// The name search did not finish, so no method identity may be elected — and none may be
    /// reported missing either.
    Incomplete(Box<TargetCandidates>),
}

/// Binds one class reference to exactly one confirmed read.
///
/// The identity path verifies that the definition belongs to this snapshot and then reads exactly
/// those bytes — a definition of another artifact is `operation_target_snapshot_mismatch` and is
/// never replaced by a same-named class this snapshot happens to hold. The name path reuses the
/// navigation search, so "which definitions of this name exist", "which of them declares it" and
/// "which paths disagree" are answered by the one implementation.
fn bind_class(
    snapshot: &ArtifactSnapshot,
    scope: &PhysicalScope,
    class: &ClassRef,
    execution: &mut ExecutionReport,
    diagnostics: &mut Vec<Diagnostic>,
    budget: &mut Budget,
) -> Result<ClassBinding> {
    match class {
        ClassRef::Definition { definition } => {
            require_definition_snapshot(snapshot, definition)?;
            budget.charge(CountedBudgetDimension::ClassHeaders, 1)?;
            let (read, retained) = read_definition(snapshot, definition, budget)?;
            // One class materialization for this operation's own selected definition
            // (`crate::d0_counts`), counted at the read that really happened: a request whose
            // charge, cancellation or read was refused above materialized nothing, a name-based
            // request's *search* reads are the search's own cost and are not counted here, and a
            // read the request's store answered from retention is not a materialization this request
            // performed.
            if !retained {
                crate::d0_counts::class_materialized();
            }
            let provenance = Some(definition_provenance(definition));
            let class_item = match charge_item(budget) {
                Ok(()) => {
                    if let ClassContentItem::ClassDeclaration(item) = &read.class
                        && let Some(diagnostic) = item_path_mismatch_diagnostic(item)
                    {
                        diagnostics.push(diagnostic);
                    }
                    Some(read.class.clone())
                }
                Err(error) => {
                    merge_execution(execution, stop_execution(&error, budget));
                    diagnostics.push(stop_diagnostic(&error, provenance.clone()));
                    None
                }
            };
            if let Err(error) = publish_diagnostics(read.diagnostics.clone(), diagnostics, budget) {
                merge_execution(execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, provenance));
            }
            Ok(ClassBinding::Bound(Box::new(BoundClass {
                read,
                search_coverage: None,
                class_item,
            })))
        }
        ClassRef::Name { class } => {
            let search = search_named_classes(
                snapshot,
                scope,
                class,
                SearchSubject::Declaration,
                |read| Ok(vec![read.class.clone()]),
                budget,
            )?;
            // The search's own completeness is read before its candidate count: a confirmed prefix
            // — one candidate, or none — is neither a unique binding nor a missing definition
            // while the search has not finished.
            if !matches!(search.execution, ExecutionReport::Complete { .. }) {
                return Ok(ClassBinding::Incomplete(Box::new(TargetCandidates {
                    query: NavigationQuery {
                        class: class.clone(),
                        member: None,
                    },
                    candidates: search.items,
                    limits: budget.limits().clone(),
                    coverage: search.coverage,
                    execution: search.execution,
                    diagnostics: search.diagnostics,
                })));
            }
            match search.matches.len() {
                0 => Err(Error::invalid_input(
                    "operation_target_not_found",
                    format!(
                        "no class in this scope declares `{}`; the search examined {} of {} \
                         candidate(s) before it answered",
                        class.spelling(),
                        search.searched,
                        search.total
                    ),
                )),
                1 => {
                    let NamedClassSearch {
                        mut matches,
                        mut items,
                        coverage,
                        execution: search_execution,
                        diagnostics: search_diagnostics,
                        ..
                    } = search;
                    let read = matches.remove(0);
                    let class_item = if items.is_empty() {
                        None
                    } else {
                        Some(items.remove(0))
                    };
                    merge_execution(execution, search_execution);
                    diagnostics.extend(search_diagnostics);
                    Ok(ClassBinding::Bound(Box::new(BoundClass {
                        read,
                        search_coverage: Some(coverage),
                        class_item,
                    })))
                }
                _ => Ok(ClassBinding::Ambiguous(Box::new(TargetCandidates {
                    query: NavigationQuery {
                        class: class.clone(),
                        member: None,
                    },
                    candidates: search.items,
                    limits: budget.limits().clone(),
                    coverage: search.coverage,
                    execution: search.execution,
                    diagnostics: search.diagnostics,
                }))),
            }
        }
    }
}

/// Binds one method reference to exactly one physical identity and the environment it runs under.
fn bind_method(
    content: &[ArtifactSnapshot],
    request: &MethodOperationRequest,
    budget: &mut Budget,
) -> Result<MethodBinding> {
    // The identity the caller gave is checked against the request's own physical view before the
    // environment is built: a foreign identity is an input error of the request, and it must not be
    // hidden behind a policy problem of a declaration the caller would then fix for nothing.
    let snapshot_id = request.environment.snapshot.clone();
    if let MethodRef::Method { method } = &request.method
        && method.owner.snapshot() != &snapshot_id
    {
        return Err(Error::invalid_input(
            "operation_target_snapshot_mismatch",
            format!(
                "the method identity names snapshot `{}` while this request runs over `{}`; an \
                 identity of another artifact is never replaced by a same-named method of this one",
                method.owner.snapshot().0,
                snapshot_id.0
            ),
        ));
    }
    let environment = request.environment.build(content)?;
    let provided = content
        .iter()
        .find(|candidate| candidate.id() == &snapshot_id);
    match &request.method {
        MethodRef::Method { method } => {
            if provided.is_none() {
                return Err(snapshot_not_provided(&snapshot_id));
            }
            Ok(MethodBinding::Bound(Box::new(BoundMethod {
                method: method.clone(),
                environment,
                // The identity path binds from the caller's own identity: it read no class, so
                // there is no read to hand over — the request's own read is the one
                // `read_method_class` performs for it.
                read: None,
            })))
        }
        MethodRef::Name {
            class,
            name,
            descriptor,
        } => {
            let Some(snapshot) = provided else {
                return Err(snapshot_not_provided(&snapshot_id));
            };
            let query = NavigationQuery {
                class: class.clone(),
                member: Some(MemberQuery {
                    name: name.clone(),
                    descriptor: descriptor.clone(),
                    kind: MemberQueryKind::Methods,
                }),
            };
            let search = search_targets(
                snapshot,
                &environment.runtime.physical.scope,
                &query,
                budget,
            )?;
            // The search's own completeness is read before its candidate count: a confirmed prefix
            // — one candidate, or none — is neither a unique binding nor a missing method while the
            // search has not finished.
            if !matches!(search.execution, ExecutionReport::Complete { .. }) {
                return Ok(MethodBinding::Incomplete(Box::new(TargetCandidates {
                    query,
                    candidates: search.items,
                    limits: budget.limits().clone(),
                    coverage: search.coverage,
                    execution: search.execution,
                    diagnostics: search.diagnostics,
                })));
            }
            match search.items.len() {
                0 => Err(Error::invalid_input(
                    "operation_target_not_found",
                    format!(
                        "no method `{}` of class `{}` matches this request in the requested scope; \
                         a descriptor narrows the overloads, and a name with several declared \
                         descriptors is answered with every candidate",
                        String::from_utf8_lossy(&name.0),
                        class.spelling()
                    ),
                )),
                1 => {
                    // The elected read travels with the binding (task 3.1): the operation this
                    // binding is for consumes *this* read — the one the search performed to
                    // confirm the definition — instead of reading the same definition again, and
                    // the search's own candidate reads stay the search's own cost.
                    let read = search.matches.into_iter().next();
                    Ok(MethodBinding::Bound(Box::new(BoundMethod {
                        method: method_identity_of(&search.items[0]),
                        environment,
                        read,
                    })))
                }
                _ => Ok(MethodBinding::Ambiguous(Box::new(TargetCandidates {
                    query,
                    candidates: search.items,
                    limits: budget.limits().clone(),
                    coverage: search.coverage,
                    execution: search.execution,
                    diagnostics: search.diagnostics,
                }))),
            }
        }
    }
}

/// The provided snapshot one physical definition lives in, when the request provides it.
///
/// The definition carries its own snapshot, so this is the one lookup every entry that has to reach
/// the definition's bytes performs: the read entries state their own refusal when it is absent
/// (`content_not_provided`), and this answers `None` so a caller that only *adopts* a read can let
/// them.
fn definition_snapshot<'a>(
    content: &'a [ArtifactSnapshot],
    definition: &PhysicalDefinitionId,
) -> Option<&'a ArtifactSnapshot> {
    content
        .iter()
        .find(|candidate| candidate.id() == definition.snapshot())
}

pub(crate) fn snapshot_not_provided(snapshot: &SnapshotId) -> Error {
    Error::invalid_input(
        "resolution_snapshot_mismatch",
        format!(
            "request snapshot `{}` is not provided by the request content",
            snapshot.0
        ),
    )
}

/// The physical identity of one published navigation item.
///
/// A method search filters for method records, so this is the one shape that can appear here: a
/// class item or a field record would mean the filter and the publication disagreed about what
/// was asked for.
fn method_identity_of(item: &ClassContentItem) -> PhysicalMethodId {
    match item {
        ClassContentItem::Method(method) => method.identity.clone(),
        ClassContentItem::ClassDeclaration(_) | ClassContentItem::Field(_) => {
            unreachable!("a method search publishes method records")
        }
    }
}

/// One class reference's own snapshot must be this snapshot: anything else is a caller error.
fn require_definition_snapshot(
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
) -> Result<()> {
    if definition.snapshot() == snapshot.id() {
        return Ok(());
    }
    Err(Error::invalid_input(
        "operation_target_snapshot_mismatch",
        format!(
            "the definition names snapshot `{}` while this request reads `{}`; an identity of \
             another artifact is never replaced by a same-named definition of this one",
            definition.snapshot().0,
            snapshot.id().0
        ),
    ))
}

/// Reads one class by the physical identity a caller holds, under the same read a listing performs.
///
/// The bytes are verified against the definition's own digest, length and variant before anything
/// is parsed — a definition that does not match the bytes at its location is the reader's own
/// input error — so the identity this returns is the identity the caller gave.
///
/// The second element says whether **this** request performed that read or the request's store
/// answered it from a read an earlier request of the same snapshot already performed (change
/// `reuse-selected-class-read`): the class content, the facts and the diagnostics are the same either
/// way, and only a caller that counts this request's own reads has anything to do with it.
fn read_definition(
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    budget: &mut Budget,
) -> Result<(ConfirmedRead, bool)> {
    let (bytes, source, retained) = materialize_definition(snapshot, definition, budget)?;
    let facts = class_member_facts(&bytes, budget)?;
    let resolved = definition_of(&source);
    let binding = class_name_binding(&source, &facts.this_class);
    let diagnostics = facts
        .stopped_at
        .as_ref()
        .map(|stop| vec![member_stop_diagnostic(&resolved, stop)])
        .unwrap_or_default();
    let class = ClassContentItem::ClassDeclaration(ClassDeclarationItem {
        definition: resolved,
        declaration: declaration_facts(
            &facts.this_class,
            facts.access_flags,
            &facts.super_class,
            &facts.interfaces,
        ),
        binding,
        member_table: facts.stopped_at.clone(),
    });
    Ok((
        ConfirmedRead {
            class,
            facts,
            diagnostics,
            bytes,
            source,
        },
        retained,
    ))
}

// ---------------------------------------------------------------------------------------------
// The class view's bodies
// ---------------------------------------------------------------------------------------------

/// How one body reference resolved against the class view's own member listing.
#[allow(
    clippy::large_enum_variant,
    reason = "a resolution held for the length of one request, where boxing the bound member would \
              only move the header a read is about to borrow behind a pointer"
)]
enum BodyResolution {
    /// Exactly one declared method matched: its identity, its own ordinal in the class's member
    /// table (the coordinate the prepared class decodes it by) and its member header for the read.
    Method(PhysicalMethodId, usize, MemberHeader),
    /// The member table stopped before the reference could be decided: the member may or may not be
    /// declared beyond the stop, and nothing is claimed about it (A13).
    NotReached(MemberTableStop),
    /// Several declared descriptors matched: every candidate, and nothing read.
    Ambiguous {
        query: NavigationQuery,
        candidates: Vec<ClassContentItem>,
    },
}

fn resolve_body_ref(read: &ConfirmedRead, body: &BodyRef) -> Result<BodyResolution> {
    let ClassContentItem::ClassDeclaration(class) = &read.class else {
        unreachable!("a class read publishes a class declaration item")
    };
    let definition = &class.definition;
    match body {
        BodyRef::Method { method } => {
            if &method.owner != definition {
                return Err(Error::invalid_input(
                    "class_view_body_foreign_owner",
                    format!(
                        "the body identity names a member of another definition while this view \
                         reads `{}`; an identity of another physical definition is never replaced \
                         by a same-named member of this one",
                        String::from_utf8_lossy(&class.declaration.this_class.raw().0)
                    ),
                ));
            }
            match declared_method(read, &method.name, &method.descriptor) {
                Some((ordinal, member)) => Ok(BodyResolution::Method(
                    method.clone(),
                    ordinal,
                    member.clone(),
                )),
                None => not_reached_or_missing(
                    read,
                    &format!(
                        "`{}` `{}`",
                        String::from_utf8_lossy(&method.name.0),
                        String::from_utf8_lossy(&method.descriptor.0)
                    ),
                ),
            }
        }
        BodyRef::Name { name, descriptor } => {
            let matched: Vec<(usize, &MemberHeader)> = read
                .facts
                .methods
                .iter()
                .enumerate()
                .filter(|(_, member)| {
                    member.name.raw().0 == name.0
                        && descriptor
                            .as_ref()
                            .is_none_or(|descriptor| member.descriptor.raw().0 == descriptor.0)
                })
                .collect();
            match matched.as_slice() {
                [] => not_reached_or_missing(
                    read,
                    &match descriptor {
                        Some(descriptor) => format!(
                            "`{}` `{}`",
                            String::from_utf8_lossy(&name.0),
                            String::from_utf8_lossy(&descriptor.0)
                        ),
                        None => format!(
                            "`{}` (any declared descriptor)",
                            String::from_utf8_lossy(&name.0)
                        ),
                    },
                ),
                [(index, member)] => Ok(BodyResolution::Method(
                    member_identity(definition, member),
                    *index,
                    (*member).clone(),
                )),
                many => {
                    let mut candidates = Vec::new();
                    for (index, member) in many {
                        candidates.push(method_item(definition, *index, member)?);
                    }
                    Ok(BodyResolution::Ambiguous {
                        query: NavigationQuery {
                            class: ClassNameQuery::internal(
                                String::from_utf8_lossy(&class.declaration.this_class.raw().0)
                                    .into_owned(),
                            ),
                            member: Some(MemberQuery {
                                name: name.clone(),
                                descriptor: descriptor.clone(),
                                kind: MemberQueryKind::Methods,
                            }),
                        },
                        candidates,
                    })
                }
            }
        }
    }
}

/// The member table stopped before this reference could be decided, or the whole table was read and
/// declares no such method — the first is a member-level refusal, the second a caller error.
fn not_reached_or_missing(read: &ConfirmedRead, requested: &str) -> Result<BodyResolution> {
    match &read.facts.stopped_at {
        Some(stop) => Ok(BodyResolution::NotReached(stop.clone())),
        None => Err(Error::invalid_input(
            "class_view_body_not_found",
            format!(
                "the class declares no method {requested}; the whole member table was read, and no \
                 empty body is invented for a member that is not there"
            ),
        )),
    }
}

/// The one member record declaring this raw name and descriptor, with the ordinal the class's own
/// member table states it at — the coordinate the prepared class decodes a body by.
fn declared_method<'a>(
    read: &'a ConfirmedRead,
    name: &JvmBytes,
    descriptor: &JvmBytes,
) -> Option<(usize, &'a MemberHeader)> {
    read.facts.methods.iter().enumerate().find(|(_, member)| {
        member.name.raw().0 == name.0 && member.descriptor.raw().0 == descriptor.0
    })
}

fn member_identity(definition: &PhysicalDefinitionId, member: &MemberHeader) -> PhysicalMethodId {
    PhysicalMethodId {
        owner: definition.clone(),
        name: member.name.raw().clone(),
        descriptor: member.descriptor.raw().clone(),
    }
}

/// The `Code` attribute shell one member's own declaration carries, when it has one.
fn code_shell(member: &MemberHeader) -> Option<&AttributeShell> {
    member
        .attributes
        .iter()
        .find(|shell| shell.name.raw().0 == b"Code")
}

/// How one class view produces the bodies the request asked for (D2 3.2).
///
/// Exactly one of these is decided per request, before the body loop: the class is prepared once
/// over the read the binding performed and every requested body is decoded against that
/// preparation, or the one attempt to obtain that preparation failed and each body that would have
/// been decoded states that failure, or the request asks for no body this view decodes at all (a
/// member that declares no `Code`, a reference the member table never reached) and nothing was
/// prepared.
enum ViewBodies<'a> {
    /// The class was prepared: every body that declares a `Code` entry is decoded by it.
    Prepared(&'a jarde_reader::prepared::PreparedClass<'a>),
    /// The class could not be prepared, and this is why: the reader's own failure, which becomes
    /// the refusal of every body that would have been decoded, while the class, its members and the
    /// declarations stay published (A13).
    Refused(Error),
    /// No requested body reaches a decode, so nothing was prepared.
    NotNeeded,
}

/// Reads one member's body out of the class this view prepared.
///
/// One `method_bodies` attempt is charged before a body that is there; a member that declares no
/// `Code` is answered as such and charges none. The decode is
/// [`jarde_reader::prepared::PreparedClass::method_code`] — the one implementation
/// [`jarde_reader::classfile::method_code_facts`] delegates to, so a body decoded here and the same
/// body decoded by the single-method entry cannot drift in facts, in stop position or in what they
/// charge — which is what keeps a view from parsing the class once per requested body. A decode that
/// fails, and a preparation that failed before it, are answered with the member's own refusal value
/// rather than ending the view: the class, the members and the other bodies stay published (A13).
fn body_result(
    definition: &PhysicalDefinitionId,
    bodies: &ViewBodies<'_>,
    ordinal: usize,
    member: &MemberHeader,
    method: &PhysicalMethodId,
    reference: &BodyRef,
    budget: &mut Budget,
) -> Result<ClassViewBody> {
    if code_shell(member).is_none() {
        return Ok(ClassViewBody::NotDeclared {
            method: method.clone(),
            no_body_kind: no_body_kind(member.access_flags),
        });
    }
    budget.charge(CountedBudgetDimension::MethodBodies, 1)?;
    let decoded = match bodies {
        ViewBodies::Prepared(prepared) => match u32::try_from(ordinal) {
            Ok(ordinal) => {
                prepared.method_code(jarde_reader::prepared::MethodOrdinal(ordinal), budget)
            }
            Err(_) => Err(Error::invalid_input(
                "classfile_result_overflow",
                "method ordinal exceeds u32",
            )),
        },
        ViewBodies::Refused(error) => Err(error.clone()),
        // Unreachable by construction: the request's own members are the ones that decided the
        // preparation above, so a member with a `Code` entry here belongs to a class this view
        // prepared or states why it could not.
        ViewBodies::NotNeeded => {
            unreachable!("a member that declares a body belongs to a class the view prepared")
        }
    };
    match decoded {
        Ok(facts) => {
            // One demand-path decode (`crate::d0_counts`), counted at the decode that really
            // happened — a member that declares no `Code` returned above and counts nothing.
            crate::d0_counts::body_decoded();
            // The owning records this publication builds: the body record itself, its two vectors,
            // and one record per instruction and per handler cloned into them. A view that decoded a
            // body it then dropped would count the decode and not the records.
            let records = 3
                + u64::try_from(facts.instructions.len()).unwrap_or(u64::MAX)
                + u64::try_from(facts.exception_handlers.len()).unwrap_or(u64::MAX);
            crate::d0_counts::owned_records(records);
            let coverage = method_code_coverage(
                facts.code_span.length,
                &facts.instructions,
                facts.exception_handlers.len(),
                facts.exception_handler_count,
                &facts.execution,
                facts.stopped_at.as_ref(),
            )?;
            Ok(ClassViewBody::Read {
                method: method.clone(),
                stages: body_stages(facts.stopped_at.as_ref()),
                max_stack: facts.max_stack,
                max_locals: facts.max_locals,
                code_span: facts.code_span.clone(),
                instructions: facts.instructions.clone(),
                exception_handlers: facts.exception_handlers.clone(),
                exception_handler_count: facts.exception_handler_count,
                stopped_at: facts.stopped_at.clone(),
                coverage,
                execution: facts.execution.clone(),
                diagnostics: Vec::new(),
            })
        }
        Err(error) => Ok(ClassViewBody::Refused {
            reference: reference.clone(),
            method: Some(method.clone()),
            execution: stop_execution(&error, budget),
            diagnostics: vec![stop_diagnostic(
                &error,
                Some(definition_provenance(definition)),
            )],
        }),
    }
}

/// The two phases of one body decode, in the reader's own order, from that decode's own stop.
///
/// A handler-phase stop leaves the instruction phase unentered — the reader decodes handlers first —
/// and an instruction-phase stop leaves the handler phase completed. Nothing is re-analysed: the
/// states are a reading of [`MethodCodeFacts::stopped_at`].
fn body_stages(stopped_at: Option<&BytecodeStop>) -> Vec<BodyStageResult> {
    let completed = |phase| BodyStageResult {
        phase,
        state: BodyStageState::Completed,
    };
    let not_reached = |phase| BodyStageResult {
        phase,
        state: BodyStageState::NotReached,
    };
    let stopped = |phase, code: &str| BodyStageResult {
        phase,
        state: BodyStageState::Stopped {
            code: code.to_owned(),
        },
    };
    match stopped_at {
        None => vec![
            completed(BytecodeStopPhase::ExceptionHandlers),
            completed(BytecodeStopPhase::Instructions),
        ],
        Some(BytecodeStop::ExceptionHandlers { code, .. }) => vec![
            stopped(BytecodeStopPhase::ExceptionHandlers, code),
            not_reached(BytecodeStopPhase::Instructions),
        ],
        Some(BytecodeStop::Instructions { code, .. }) => vec![
            completed(BytecodeStopPhase::ExceptionHandlers),
            stopped(BytecodeStopPhase::Instructions, code),
        ],
    }
}

/// The `abstract`/`native` kind a member's own flags declare, when they declare one.
fn no_body_kind(access_flags: u16) -> Option<NoBodyKind> {
    const ACC_ABSTRACT: u16 = 0x0400;
    const ACC_NATIVE: u16 = 0x0100;
    if access_flags & ACC_ABSTRACT != 0 {
        Some(NoBodyKind::Abstract)
    } else if access_flags & ACC_NATIVE != 0 {
        Some(NoBodyKind::Native)
    } else {
        None
    }
}

/// The coverage of one class view: the candidate search's own ranges (when the class was named)
/// beside the member tables of the class the view read, each under its own label.
fn class_view_coverage(
    search: Option<&Coverage>,
    facts: &ClassMemberFacts,
    complete: bool,
) -> Coverage {
    let mut coverage = member_coverage(facts, complete);
    if let Some(search) = search {
        let mut scanned = search.artifact_structural.scanned.clone();
        scanned.append(&mut coverage.artifact_structural.scanned);
        coverage.artifact_structural.scanned = scanned;
        let mut skipped = search.artifact_structural.skipped.clone();
        skipped.append(&mut coverage.artifact_structural.skipped);
        coverage.artifact_structural.skipped = skipped;
        coverage.artifact_structural.uninterpreted_extensions =
            search.artifact_structural.uninterpreted_extensions.clone();
        if search.artifact_structural.state != CoverageState::CompleteWithinSchema {
            coverage.artifact_structural.state = CoverageState::Partial;
        }
    }
    coverage
}
