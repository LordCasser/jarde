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
    self, ClassSourceDeclaration, ClassSourceField, ClassSourceMethod, ClassSourceReport,
    ClassSourceRequest, ClassSourceRunFacts,
};
use crate::environment::{EnvironmentIdentity, EnvironmentProblem, ResolutionEnvironment};
use crate::ir::{AnalysisStage, NoBodyKind, Quality};
use crate::resolver::{
    DeclarationRefItem, DeclarationRefReport, HeaderRead, ResolutionAnalysis, ResolvedMemberRef,
};
use jarde_java::{
    RecoveryContent, RecoveryEvidenceKind, RecoveryEvidenceRequest, RecoveryReport, StopReason,
};
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
    AttributeShell, BytecodeStop, BytecodeStopPhase, ClassMemberFacts, ExceptionHandlerFact,
    InspectionMode, InstructionFact, MemberHeader, MemberTablePhase, MemberTableStop,
    MethodSelector, class_member_facts, method_code_coverage,
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
use jarde_reader::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, PhysicalScope,
    PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};
use serde::{Deserialize, Serialize};

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
        if read.is_some() {
            // One class materialization for this operation's own selected definition
            // (`crate::d0_counts`): the run above performed it, and the presentation below consumes
            // it rather than reading the definition again.
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
        let (bytes, source) = materialize_definition(snapshot, definition, budget)?;
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
        let (read, search_coverage, class_item) = match bind_class(
            snapshot,
            &view.scope,
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
        let ClassContentItem::ClassDeclaration(item) = read.class.clone() else {
            unreachable!("a class read publishes a class declaration item")
        };
        let definition = item.definition.clone();
        let declaration = ClassSourceDeclaration::of(item);
        // The class and member planes are complete on their own evidence, taken before any member is
        // run, so that a member which stopped below cannot rewrite what the member table really was
        // (A13/A14) — the same rule the class view applies to its bodies.
        let structure_complete = matches!(execution, ExecutionReport::Complete { .. })
            && read.facts.stopped_at.is_none();
        if class_item.is_none() {
            // The class confirmed its own item and could not publish it (a refused `result_items`
            // charge): the report keeps the identity and the stop and presents nothing at all, which
            // is what a report that could not pay for its own declaration may say.
            return Ok(OperationOutcome::Performed(ClassSourceReport {
                view,
                class: definition,
                declaration: None,
                stages,
                fields: Vec::new(),
                methods: Vec::new(),
                text: String::new(),
                limits: budget.limits().clone(),
                usage: budget.usage(),
                coverage: class_view_coverage(search_coverage.as_ref(), &read.facts, false),
                execution: with_usage(execution, budget.usage()),
                diagnostics,
            }));
        }
        let class_provenance = Some(definition_provenance(&definition));
        // The fields of the same read, in declaration order, each charged as the item it is.
        let mut fields = Vec::new();
        let mut ended = false;
        for (index, field) in read.facts.fields.iter().enumerate() {
            if let Err(error) = charge_item(budget) {
                merge_execution(&mut execution, stop_execution(&error, budget));
                diagnostics.push(stop_diagnostic(&error, class_provenance.clone()));
                ended = true;
                break;
            }
            let ClassContentItem::Field(item) = field_item(&definition, index, field)? else {
                unreachable!("a field record publishes a field item")
            };
            fields.push(ClassSourceField::of(item));
        }
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
        let mut attempted = 0_u64;
        let mut methods = Vec::new();
        for (index, member) in read.facts.methods.iter().enumerate() {
            if ended {
                break;
            }
            let ClassContentItem::Method(item) = method_item(&definition, index, member)? else {
                unreachable!("a method record publishes a method item")
            };
            let spelled = class_source::spell_method(&item, None, &declaration.name);
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
                    (
                        ClassSourceMethod::no_body(
                            item,
                            no_body_kind(member.access_flags),
                            spelled,
                        ),
                        Vec::new(),
                        false,
                    )
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
                                content, &request, prepared, evidence, budget,
                            ) {
                                Ok(recovered) => {
                                    let analysis = ClassSourceRunFacts {
                                        execution: recovered.analysis().execution.clone(),
                                        diagnostics: to_u64(
                                            recovered.analysis().diagnostics.len(),
                                        )?,
                                    };
                                    let spelled = class_source::spell_method(
                                        &item,
                                        Some(recovered.facts()),
                                        &declaration.name,
                                    );
                                    let (_, report, _) = recovered.into_parts();
                                    let stops =
                                        vec![analysis.execution.clone(), report.execution.clone()];
                                    let ends = stops.iter().any(ends_the_request);
                                    let record = ClassSourceMethod::recovered(
                                        item,
                                        spelled,
                                        Box::new(report),
                                        analysis,
                                    );
                                    (record, stops, ends)
                                }
                                Err(error) => {
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
        if let Some(stop) = &read.facts.stopped_at {
            merge_execution(&mut execution, member_stop_execution(stop, budget));
        }
        let text = class_source::source_text(
            &declaration,
            &fields,
            &methods,
            read.facts.method_count,
            read.facts.stopped_at.as_ref(),
            &execution,
        );
        let coverage = class_source_coverage(
            class_view_coverage(search_coverage.as_ref(), &read.facts, structure_complete),
            attempted,
            declared_bodies,
            attempted == declared_bodies,
        );
        Ok(OperationOutcome::Performed(ClassSourceReport {
            view,
            class: definition,
            declaration: Some(declaration),
            stages,
            fields,
            methods,
            text,
            limits: budget.limits().clone(),
            usage: budget.usage(),
            coverage,
            execution: with_usage(execution, budget.usage()),
            diagnostics,
        }))
    }
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
/// A member recovered here and the same member recovered through [`Engine::recover_method`] cannot
/// drift, because the two share everything but that read.
fn recover_prepared_member(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    prepared: &jarde_reader::prepared::PreparedClass<'_>,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> Result<RecoveredMethod> {
    let analyzed = jarde_jvm::analyze_prepared_method_ir(content, prepared, request, budget)?;
    if analyzed.ir().code().is_some() {
        // The prepared half of the same demand-path decode (`crate::d0_counts`): one count per
        // member body this presentation really decoded.
        crate::d0_counts::body_decoded();
    }
    recovery_presented(content, request, analyzed, Some(prepared), evidence, budget)
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
    let candidates = jarde_java::accessor::candidates(ir);
    if candidates.is_empty() {
        return None;
    }
    let declaration = ir.declaration()?;
    let candidates = candidates
        .iter()
        .map(|candidate| {
            jarde_jvm::callee::CalleeCandidate::new(
                candidate.call_site,
                candidate.owner.as_bytes(),
                candidate.name.as_bytes(),
                candidate.descriptor.as_bytes(),
            )
        })
        .collect();
    Some((declaration.identity().owner.clone(), candidates))
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

/// One run presented, with the class a same-class callee read may come from.
fn recovery_from(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    analyzed: jarde_jvm::method_ir::MethodIrAnalysis,
    callee_class: CalleeClass<'_>,
    evidence: &RecoveryEvidenceRequest,
    budget: &mut Budget,
) -> Result<RecoveredMethod> {
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
            Some(read_prepared_named_callees(
                content, request, candidates, &prepared, budget,
            )?)
        }
        (CalleeClass::None, Some(candidates)) => {
            Some(read_named_callees(content, request, candidates, budget)?)
        }
    };
    let members = callees.as_ref().map(member_table);
    let request = jarde_java::RecoveryRequest::new(analyzed.ir(), &facts, profile)
        .with_evidence(evidence.clone());
    let mut recovery = jarde_java::recover(
        &match &members {
            Some(members) => request.with_members(members),
            None => request,
        },
        budget,
    );
    // What the read evidence publishes (change `add-demand-driven-core-results`, D3). The read above
    // is the accessor rule's own input whatever the caller selected — the member table it decides
    // from — so what the selection decides is the **record**: the callee facts and the header-read
    // proofs are the `ReadDetails` expansion, published only when the request selected it and the
    // presentation really produced an artifact. Every other case publishes the binding results alone:
    // which member each candidate resolved to, which candidates this class does not answer, and what
    // the read charged. Nothing is read a second time on either branch.
    let publish_read_details =
        evidence.requests(RecoveryEvidenceKind::ReadDetails) && recovery.produced();
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
    Ok(RecoveredMethod {
        analysis: analysis.clone(),
        recovery,
        callees,
        facts,
    })
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
            let read = read_definition(snapshot, definition, budget)?;
            // One class materialization for this operation's own selected definition
            // (`crate::d0_counts`), counted at the read that really happened: a request whose
            // charge, cancellation or read was refused above materialized nothing, and a name-based
            // request's *search* reads are the search's own cost and are not counted here.
            crate::d0_counts::class_materialized();
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
fn read_definition(
    snapshot: &ArtifactSnapshot,
    definition: &PhysicalDefinitionId,
    budget: &mut Budget,
) -> Result<ConfirmedRead> {
    let (bytes, source) = materialize_definition(snapshot, definition, budget)?;
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
    Ok(ConfirmedRead {
        class,
        facts,
        diagnostics,
        bytes,
        source,
    })
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
