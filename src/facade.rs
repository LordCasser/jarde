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
use jarde_reader::artifact::{
    ArtifactInput, ArtifactSnapshot, ArtifactTreeReport, EnumerationReport,
};
use jarde_reader::budget::Budget;
use jarde_reader::classfile::{InspectionMode, MethodSelector};
use jarde_reader::error::Result;
use jarde_reader::inspect::{ClassTarget, EngineBytecodeReport, EngineHeaderReport};
use jarde_reader::model::PhysicalMethodId;
use serde::Serialize;

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
    /// The facts the recovery layer needs and the payload does not carry — the member's identity, its
    /// parameter slots and its debug names — are derived here from the request itself, and where the
    /// run's own evidence does not reach, this entry states *nothing* rather than a guess:
    ///
    /// * the identity is the member the request named;
    /// * **no parameter slots** are stated (see `recovery_facts`): the payload publishes no access
    ///   flags, so slot 0 cannot be told from a receiver, and a body is presented with ordinal names
    ///   throughout instead of being split on an assumption;
    /// * **no debug names** are stated: the payload does not publish the `LocalVariableTable`, and
    ///   reading it would be a second read of the class (billed again, and a second truth about one
    ///   method).
    ///
    /// A body with neither is presented with deterministic ordinal names (A10) rather than refused;
    /// naming a slot from its attribute or its parameter position is 3.1's, over the same-run header
    /// read that slice owns.
    pub fn recover_method(
        &self,
        content: &[ArtifactSnapshot],
        request: &crate::ir::MethodAnalysisRequest,
        budget: &mut Budget,
    ) -> Result<RecoveredMethod> {
        let analyzed = jarde_jvm::analyze_method_ir(content, request, budget)?;
        let facts = crate::facade::recovery_facts(&request.method);
        let profile = request.environment.runtime.profile.clone();
        let recovery = jarde_java::recover(
            &jarde_java::RecoveryRequest::new(analyzed.ir(), &facts, profile),
            budget,
        );
        Ok(RecoveredMethod {
            analysis: analyzed.report().clone(),
            recovery,
        })
    }
}

/// One method-analysis run and the presentation of its own payload (P3 1.3).
///
/// Both halves are of the *same* run: [`RecoveredMethod::analysis`] is the report
/// [`jarde_jvm::analyze_method_ir`] assembled for the run whose tables the recovery read, so the
/// stage results, the coverage, the reads and the planes of the analysis and the planes of the
/// recovery describe one request. A caller that wants only the presentation reads
/// [`RecoveredMethod::recovery`] and ignores the other half; a caller that wants the run's own
/// evidence (which reads it charged, which stages completed) reads both.
#[derive(Clone, Debug, Serialize)]
pub struct RecoveredMethod {
    analysis: crate::ir::MethodAnalysisReport,
    recovery: jarde_java::RecoveryReport,
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

    /// Both halves, by value: the adapter that serializes them does not clone what it owns.
    pub fn into_parts(self) -> (crate::ir::MethodAnalysisReport, jarde_java::RecoveryReport) {
        (self.analysis, self.recovery)
    }
}

/// The facts of one member, as much as this entry can state from the request alone.
///
/// The identity is the member's own name and descriptor, exactly as the request spells them (JVM
/// bytes, so a name that is not UTF-8 is presented as the decode wrote it rather than dropped).
///
/// The parameter-slot count is deliberately **zero**: the payload of the run does not publish the
/// member's access flags, and a descriptor alone cannot say whether slot 0 holds a receiver, so any
/// count this entry stated would be an assumption that the member is static. Stating none is the
/// honest reading of the evidence the run has — every slot is then named by its ordinal as a local
/// (`local0`, `local1`, …), which is A10's deterministic naming and no claim about which slots are
/// parameters. A caller that does know (its own declaration facts, or the same-run header read that
/// 3.1's naming work owns) states the count itself in [`jarde_java::RecoveryFacts`]; the recovery
/// layer's own tests do exactly that.
fn recovery_facts(method: &PhysicalMethodId) -> jarde_java::RecoveryFacts {
    let name = String::from_utf8_lossy(&method.name.0).into_owned();
    let descriptor = String::from_utf8_lossy(&method.descriptor.0).into_owned();
    jarde_java::RecoveryFacts::new(jarde_java::MethodFacts::new(name, descriptor, 0))
}
