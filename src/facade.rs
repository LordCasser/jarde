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
/// With no declaration — a run that stopped before `raw_facts`, or a member that declares no body —
/// the name and descriptor are the request's own bytes and the parameter-slot count is **zero**: no
/// read stated how many slots the parameters occupy, so every slot is named by its ordinal as a local
/// (`local0`, `local1`, …), which is A10's deterministic naming and no claim about the signature.
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
            .with_access_flags(declaration.access_flags()),
    )
    .with_debug_locals(debug)
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
