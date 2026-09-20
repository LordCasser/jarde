//! The facade's [`Engine`]: one line per entry point, delegating to the layer that owns the
//! work.
//!
//! Artifact I/O, enumeration and inspection are the reader's (P2 1.2); the query entry is
//! `jarde-query`'s (P2 2.1); resolution, declaration references and method analysis are
//! `jarde-jvm`'s (P2 2.2). Nothing here implements analysis — the module exists so that a
//! caller has one type to hold and one set of names to import, and so this crate can publish
//! that type without publishing the layers behind it or the mutable internals they keep to
//! themselves.

use jarde_query::query::{QueryReport, QueryRequest};
use jarde_reader::accounting::with_usage;
use jarde_reader::artifact::{
    ArtifactInput, ArtifactKind, ArtifactSnapshot, ArtifactTreeReport, EnumerationReport,
    PhysicalEntry, budget_dimension_code,
};
use jarde_reader::budget::{Budget, CountedBudgetDimension};
use jarde_reader::classfile::{
    ClassMemberFacts, InspectionMode, MemberHeader, MemberTablePhase, MemberTableStop,
    MethodSelector, class_member_facts,
};
use jarde_reader::error::{Error, Result};
use jarde_reader::inspect::{
    ClassSource, ClassTarget, EngineBytecodeReport, EngineHeaderReport, materialize_definition,
    materialize_root,
};
use jarde_reader::model::{
    ByteSpan, ClassBytesId, Coverage, CoverageDimension, CoverageRange, CoverageState, Diagnostic,
    DiagnosticSeverity, ExecutionReport, JvmBytes, JvmString, Location, MemberKey,
    PhysicalClassLocation, PhysicalDefinitionId, PhysicalEntryId, PhysicalMemberId,
    PhysicalMethodId, PhysicalVariant, Provenance, SnapshotId, TerminationReason,
    physical_variant_for_path,
};
use jarde_reader::view::{PhysicalScope, PhysicalView};
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
    pub fn recover_method(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::ir::MethodAnalysisRequest,
        budget: &mut Budget,
    ) -> Result<RecoveredMethod> {
        let analyzed = jarde_jvm::analyze_method_ir(content, request, budget)?;
        let facts = crate::facade::recovery_facts(
            analyzed.ir().declaration(),
            analyzed.ir().code(),
            &request.method,
        );
        let profile = request.environment.runtime.profile.clone();
        // The callee evidence one recovery run's own call sites justify, read on demand from the very
        // definition the run read the presented body from (P3 3.2). It happens **between** the run
        // and the presentation — not inside the recovery layer, which holds no artifact, no loader
        // and no budget — and it is the only read this entry performs beyond the one run: a recovery
        // request whose body names no such call site reads no member at all.
        let callees = read_named_callees(content, request, analyzed.ir(), budget)?;
        let members = callees.as_ref().map(member_table);
        let request = jarde_java::RecoveryRequest::new(analyzed.ir(), &facts, profile);
        let recovery = jarde_java::recover(
            &match &members {
                Some(members) => request.with_members(members),
                None => request,
            },
            budget,
        );
        Ok(RecoveredMethod {
            analysis: analyzed.report().clone(),
            recovery,
            callees,
            facts,
        })
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
    /// definition, member or diagnostic is invented. A candidate whose read *fails* is the one case
    /// that ends the search, with the items it published before it, a diagnostic carrying that entry's
    /// origin and a non-`Complete` execution.
    pub fn find_targets(
        &self,
        snapshot: &ArtifactSnapshot,
        scope: &PhysicalScope,
        query: &NavigationQuery,
        budget: &mut Budget,
    ) -> Result<NavigationReport> {
        let scan = scan_scope(snapshot, scope, budget)?;
        let requested = query.class.internal_name();
        let candidates: Vec<ClassCandidate<'_>> =
            scope_class_candidates(snapshot.kind(), snapshot.id(), &scan.entries)
                .into_iter()
                .filter(|candidate| candidate_states_name(candidate, &requested.0))
                .collect();
        let total = to_u64(candidates.len())?;
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
                Ok(read) => {
                    if let Err(error) =
                        publish_diagnostics(read.diagnostics, &mut diagnostics, budget)
                    {
                        merge_execution(&mut execution, stop_execution(&error, budget));
                        diagnostics.push(stop_diagnostic(&error, provenance));
                        break;
                    }
                    let ClassContentItem::ClassDeclaration(class) = &read.class else {
                        unreachable!("a class declaration read publishes a class declaration item")
                    };
                    if class.declaration.this_class.raw().0 != query.class.internal_name().0 {
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
                                class.definition.class_bytes.length,
                                &query.class.internal_name().0,
                                &class.declaration.this_class,
                            ));
                        }
                        continue;
                    }
                    let found = match &query.member {
                        None => vec![read.class],
                        Some(filter) => {
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
                            found
                        }
                    };
                    let mut refused = false;
                    for item in found {
                        if let Err(error) = charge_item(budget) {
                            merge_execution(&mut execution, stop_execution(&error, budget));
                            diagnostics.push(stop_diagnostic(&error, provenance.clone()));
                            refused = true;
                            break;
                        }
                        items.push(item);
                    }
                    if refused {
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
        Ok(NavigationReport {
            view: PhysicalView {
                snapshot: snapshot.id().clone(),
                scope: scope.clone(),
            },
            query: query.clone(),
            candidates: items,
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

/// One confirmed class read: the class-level item, the member facts the *same* read established, and
/// the diagnostics that read itself published.
///
/// The facts travel with the item because a caller that asks for members derives them from *this*
/// read — one read of the class per request, never a second opinion about the same bytes.
struct ConfirmedRead {
    class: ClassContentItem,
    facts: ClassMemberFacts,
    diagnostics: Vec<Diagnostic>,
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

fn to_u64(value: usize) -> Result<u64> {
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
fn listing_coverage(
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
fn stop_execution(error: &Error, budget: &Budget) -> ExecutionReport {
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
fn merge_execution(execution: &mut ExecutionReport, incoming: ExecutionReport) {
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

/// One diagnostic for a step that stopped the work it was part of.
///
/// The code is the failure's own, so one failure has one code wherever this engine reports it, and the
/// provenance names the physical position the step was working on when it stopped — the entry it was
/// reading, or the definition it was reading from. A stop diagnostic is control metadata: it explains
/// why the report is not complete, so it is published whether or not the item budget that stopped it
/// could pay for it.
fn stop_diagnostic(error: &Error, provenance: Option<Provenance>) -> Diagnostic {
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
fn error_code(error: &Error) -> String {
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
fn entry_provenance(entry: &PhysicalEntryId, length: u64) -> Provenance {
    Provenance {
        location: Location::Entry {
            id: entry.clone(),
            span: ByteSpan::new(0, length),
        },
    }
}

/// The physical origin of one definition, as that class file from its first byte.
fn definition_provenance(definition: &PhysicalDefinitionId) -> Provenance {
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

/// The class's own members the presented body's call sites named, read on demand (P3 3.2).
///
/// The candidates are the presented body's **own decode**: the call sites the `accessor@1` rule
/// would decide from, enumerated by that rule ([`jarde_java::accessor::candidates`]) so that what a
/// run reads the class's members for is what the rule reads a verdict from, and never a second
/// opinion about which calls matter. A body that names no such call site reads nothing here — no
/// header, no member — and `None` is what this entry then hands on.
///
/// The class they may come from is the definition the run read the presented body from, as the
/// payload's own declaration states it: not a name a call site spells, and never a second class. A
/// call site naming another class is refused by the read with that stated, so "a member of a class
/// that happens to share this name" cannot be read as this call's callee.
fn read_named_callees(
    content: &[ArtifactSnapshot],
    request: &crate::ir::MethodAnalysisRequest,
    ir: &jarde_jvm::method_ir::MethodIr,
    budget: &mut Budget,
) -> Result<Option<jarde_jvm::callee::CalleeReadReport>> {
    let candidates = jarde_java::accessor::candidates(ir);
    if candidates.is_empty() {
        return Ok(None);
    }
    // A run that read no member header states no definition its members could come from: there is
    // nothing to bind the evidence to, so no member is read and the rule states the table it is
    // missing (P3 3.1/3.2).
    let Some(declaration) = ir.declaration() else {
        return Ok(None);
    };
    let candidates: Vec<jarde_jvm::callee::CalleeCandidate> = candidates
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
    let read = jarde_jvm::callee::read_callees(
        content,
        &jarde_jvm::callee::CalleeReadRequest::new(
            &request.environment,
            &declaration.identity().owner,
            &candidates,
        ),
        budget,
    )?;
    Ok(Some(read))
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
