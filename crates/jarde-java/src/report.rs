//! The recovery entry point and the report it produces (P3 1.3's public seam).
//!
//! # One run, one request, one artifact
//!
//! [`recover`] takes the P2 payload by reference ([`MethodIr`], the 1.1 handoff) plus the facts the
//! layer below read ([`RecoveryFacts`]), and produces at most one artifact: the Java text, its
//! segment table, a diagnostic list, the six planes that describe what was produced, and
//! [`RecoveryContent`] — what the artifact holds.
//!
//! # The six planes are written, not inferred from each other
//!
//! Each plane has exactly one input, and no plane is derived from another's value (P3 1.2's
//! independent-input table):
//!
//! | plane | this slice writes it from |
//! | --- | --- |
//! | [`Representation`] | whether any region kept bytecode: all Java → `Java`, any fallback → `Mixed` |
//! | [`Quality`] | how strong the recovered structure is: every region structured → `Structured`, any fallback → `Fallback` |
//! | [`SyntaxStatus`] | whether an alias replaced a spelling the source had: any alias → `NotJava`, else `Unchecked` (nothing in this slice runs a syntax check, so `Checked` is never claimed) |
//! | [`CompileStatus`] | `NotAttempted` — only 3.3's controlled recompilation writes anything else |
//! | [`SemanticValidation`] | `Unproven` — this run checks no invariant of its own, and the P2 run's `LocalInvariants` is that run's evidence, not this one's |
//! | [`VerificationStatus`] | `NotPerformed` — nothing verified the artifact |
//!
//! The combinations the P3 spec states are therefore all reachable without any plane being bent to
//! fit another: `Mixed`/`Fallback` for a body with an unprovable region, and `Java`/`NotJava` for a
//! body whose local is named `int` in the source and `int_` in the text.
//!
//! # The content classification is read from the committed artifact
//!
//! [`RecoveryContent`] is not a second quality: it is read from the emission that committed the
//! artifact — how many statements the emitter wrote that were not fallbacks — and no plane is read
//! from it or writes it. It exists because `Produced` says only that an artifact was delivered: a
//! report whose whole artifact is reasons and quoted bytecode is `Produced` too, and before this
//! field a caller had to strip comments from [`RecoveryReport::text`] to guess which of the two it
//! held. A stopped run holds no artifact, so the report of a stop states `NotProduced` without asking
//! whether anything was built or written before the refusal.
//!
//! # A stop is not a produced artifact
//!
//! When the budget refuses, the run is cancelled, or a table the payload needs is missing, the
//! report carries no text, no segments, an [`ExecutionReport`] that says so, and a
//! [`RecoveryOutcome::Stopped`] reason naming what refused and where. Nothing in a stopped report can
//! be read as "an empty body was recovered successfully" — that is the property the P3 tasks call
//! out, and the one a caller has to be able to rely on without reading diagnostics.

use jarde_jvm::ir::{CompileStatus, Quality, Representation, SemanticValidation, SyntaxStatus};
use jarde_jvm::method_ir::MethodIr;
use jarde_reader::budget::{Budget, BudgetDimension, UsageSnapshot};
use jarde_reader::classfile::VerificationStatus;
use jarde_reader::model::{Diagnostic, DiagnosticSeverity, ExecutionReport, TerminationReason};
use serde::Serialize;

use crate::accessor::AccessorRecord;
use crate::bridge::{self, BridgeRecord};
use crate::build;
use crate::concat::{self, ConcatRecord};
use crate::declaration::{self, DeclarationRecord};
use crate::decode::Operations;
use crate::emit::{Emitted, emit, emit_source_map};
use crate::enumswitch::{self, EnumSwitchRecord};
use crate::evidence::{
    EvidencePayload, EvidencePhase, EvidenceRefusal, Materialized, Publication, RecoveryEvidence,
    RecoveryEvidenceKind, RecoveryEvidenceRequest, SegmentPublication,
};
use crate::facts::{ClassMembers, RecoveryFacts};
use crate::field::{self, FieldRecord};
use crate::init::{self, InitRecord, NewRecord};
use crate::lambda::LambdaRecord;
use crate::names::NameTable;
use crate::normal_flow::NormalFlowView;
use crate::pass::{
    ACCESSOR, BRIDGE, CONCAT, DECLARATION, ENUMSWITCH, FIELD, INIT, LAMBDA, NEW, RecoveryProfile,
    RuleVersion,
};
use crate::region::{FallbackReason, Recovered, Region};
use crate::reuse;
use crate::source_map::SourceMap;
use crate::stop::StopReason;

/// One recovery request: the payload of a P2 run, the facts that run did not publish, and the
/// profile the presentation is written under.
#[derive(Clone, Debug)]
pub struct RecoveryRequest<'a> {
    /// The IR payload of the method-analysis run whose body is being presented.
    pub ir: &'a MethodIr,
    /// The two facts the payload does not carry: the method's identity and its debug names. The
    /// decoded operations are not here — they travel inside the payload, so that the branch a
    /// comparison performs, the slot a load names and the value a constant pushes have exactly one
    /// source, the run that decoded them.
    pub facts: &'a RecoveryFacts,
    /// The recovery profile the run is presented under: the rule set whose passes the gate admits
    /// ([`crate::pass`]). It is the request's own runtime profile, stated by the caller — the entry
    /// point hands the environment's profile over — and it is a *policy* input, not evidence: unlike
    /// the three tables above, it says nothing about what the bytes are.
    pub profile: RecoveryProfile,
    /// The class's **other members**, as the caller read them beside this body: the declaration and
    /// the decoded body of a member the payload cannot hold, because the payload is one method's
    /// (P3 2.2). A caller that did not read the class's members states `None`, and the accessor rule
    /// then records the table it is missing rather than guessing from a call's name.
    pub members: Option<&'a ClassMembers>,
    /// Which **optional evidence** this request wants delivered (change
    /// `add-demand-driven-core-results`, D1): the categories of detail records, and the driver BCI
    /// range they are restricted to. [`RecoveryEvidenceRequest::essential`] — the default
    /// [`RecoveryRequest::new`] states — selects none of them, and the run then delivers the
    /// necessary results: the artifact, the planes, the core gaps and the stops. The selection never
    /// decides whether a rule runs, whether a value is proven or whether a region is refused.
    pub evidence: RecoveryEvidenceRequest,
}

impl<'a> RecoveryRequest<'a> {
    /// One request over one payload, one fact set and one profile, with no member table.
    ///
    /// The evidence selection is [`RecoveryEvidenceRequest::essential`]: the ordinary recovery, which
    /// delivers the necessary results and materializes no optional detail record. A caller that
    /// wants detail states it with [`RecoveryRequest::with_evidence`].
    pub fn new(ir: &'a MethodIr, facts: &'a RecoveryFacts, profile: RecoveryProfile) -> Self {
        Self {
            ir,
            facts,
            profile,
            members: None,
            evidence: RecoveryEvidenceRequest::essential(),
        }
    }

    /// The same request, with the class's other members: the evidence a synthetic accessor call site
    /// is decided from (P3 2.2, A12).
    pub fn with_members(mut self, members: &'a ClassMembers) -> Self {
        self.members = Some(members);
        self
    }

    /// The same request, selecting the optional evidence this run materializes.
    ///
    /// The selection is the caller's own statement and is echoed in the report beside what each
    /// category really delivered ([`RecoveryReport::evidence`]); it is never a second way to ask for
    /// a different *decision*.
    pub fn with_evidence(mut self, evidence: RecoveryEvidenceRequest) -> Self {
        self.evidence = evidence;
        self
    }
}

/// Whether the run produced an artifact or stopped before it had one.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryOutcome {
    /// The artifact is in [`RecoveryReport::text`], and its positions are in
    /// [`RecoveryReport::source_map`].
    Produced,
    /// Nothing was produced; the reason says what refused and where.
    Stopped(StopReason),
}

impl RecoveryOutcome {
    /// Whether the run produced an artifact.
    pub fn produced(&self) -> bool {
        matches!(self, Self::Produced)
    }

    /// The stop, when the run stopped.
    pub fn stop(&self) -> Option<&StopReason> {
        match self {
            Self::Produced => None,
            Self::Stopped(reason) => Some(reason),
        }
    }
}

/// What a report's artifact holds: whether the run stopped without one, delivered explanation alone,
/// or delivered Java statements.
///
/// This is the answer to "does the artifact hold anything a caller can call code", and it is
/// deliberately the *only* answer to it — not a second quality plane and not a recovery measure. It
/// is read from the committed structure (the statements the emitter wrote, in `crate::emit`) and
/// never from [`RecoveryReport::text`]: no comment is stripped, no token is counted, and no caller
/// has to parse the artifact to learn this. A statement is a declaration, an assignment, a call, a
/// constructor call, a `return` or a control-flow statement; a wrapper line, a brace, a fallback's
/// reason and its bytecode indexes are not.
///
/// Two shapes keep the classification honest in both directions: `return;` alone is
/// `ContainsStatements`, and `if (arg0) {}` is `ContainsStatements` too (its condition is
/// evaluated), while neither of them raises `quality` or becomes a claim about the rest of the body.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryContent {
    /// The run stopped before an artifact was committed, so there is no content to describe: the
    /// text and the segment table are empty, as the stop contract states.
    NotProduced,
    /// An artifact was delivered and it holds no statement: what it says is the envelope, the
    /// reasons and the quoted bytecode of the regions the run refused.
    ExplanationOnly,
    /// The artifact holds at least one statement the emitter wrote as Java.
    ContainsStatements,
}

/// What one recovered region is, stated so that a reader can check the run's own claim about it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RegionRecord {
    /// The BCI the region starts at.
    pub bci: u32,
    /// Whether every block of the region was presented as Java structure.
    pub structured: bool,
    /// The BCIs of the blocks the region claims, in method order.
    pub blocks: Vec<u32>,
    /// The fallback's diagnostic code, when it has one.
    pub code: Option<&'static str>,
    /// The fallback's message, when it has one.
    pub message: Option<String>,
    /// The rule that produced this record: the pass that claimed the region, or the pass whose
    /// declared precondition refused it. `None` when no registered rule is answerable for it (a
    /// whole-body refusal the walk itself states) — never a borrowed rule name.
    pub rule: Option<RuleVersion>,
}

/// The result of one recovery run: one artifact, or the reason there is none.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecoveryReport {
    /// The method this report is about, as its facts state it.
    pub method: String,
    /// The recovery profile this run was presented under, echoed from the request.
    pub profile: RecoveryProfile,
    /// Every rule that produced a region of this method, each once, in the order it first did —
    /// how "which rule produced this output" is answered by the run's own record.
    pub rules: Vec<RuleVersion>,
    /// What the artifact is made of.
    pub representation: Representation,
    /// How strong the recovered structure is.
    pub quality: Quality,
    /// Whether the text is claimed to be Java syntax.
    pub syntax_status: SyntaxStatus,
    /// Whether the text was ever compiled; never, in this slice.
    pub compile_status: CompileStatus,
    /// What semantic evidence applies to the artifact.
    pub semantic_validation: SemanticValidation,
    /// Whether the artifact was verified; never, in this slice.
    pub verification: VerificationStatus,
    /// The execution of *this* run, in the vocabulary the fact layer already states.
    pub execution: ExecutionReport,
    /// Whether the run produced an artifact or stopped.
    pub outcome: RecoveryOutcome,
    /// What the delivered artifact holds, read from the committed structure. `Produced` says only
    /// that an artifact was delivered; this says whether any statement is in it — and says nothing
    /// about completeness, compilability or semantic equivalence, which the planes above state for
    /// themselves.
    pub content: RecoveryContent,
    /// The Java text, empty when the run stopped.
    pub text: String,
    /// The segment table of [`Self::text`], empty when the run stopped.
    pub source_map: SourceMap,
    /// Every region the run recovered, in method order.
    pub regions: Vec<RegionRecord>,
    /// Every `invokedynamic` site of the body, in BCI order, with the bootstrap, SAM, implementation
    /// and capture evidence behind it and what the run did with it (P3 2.1, A04). A site appears
    /// whether it was presented as a lambda or refused and left as bytecode: the refusals are part
    /// of the answer, not an absence from it.
    pub lambdas: Vec<LambdaRecord>,
    /// Every concatenation chain candidate of the body, in BCI order, with the class it built, every
    /// `append` it called and whether it was presented (P3 2.2). A refused candidate is part of the
    /// answer too: it names the link of the verification that failed.
    pub concats: Vec<ConcatRecord>,
    /// Every synthetic accessor call site of the body, in BCI order, with the member the call named,
    /// the flags the class declared, the field its body accesses and whether the call site was
    /// presented as that field (P3 2.2, A12).
    pub accessors: Vec<AccessorRecord>,
    /// The `bridge@1` rule's verdict for this very method, when the member is declared a bridge or
    /// its body is the forward a bridge is written as (P3 2.2). Empty for every other body: an
    /// ordinary member is not a bridge question.
    pub bridges: Vec<BridgeRecord>,
    /// Every construction site of the body, in BCI order, with the class it allocates, the
    /// constructor it calls and the BCIs of its arguments (P3 2.3, `new@1`). A refused candidate is
    /// part of the answer: it names the link of the verification that failed. This is also the rule
    /// that presents a local, anonymous or inner class's **use** — the class's own name is the one
    /// the pool spells — while the nesting relation itself is a class-level fact this run does not
    /// hold and never claims.
    pub news: Vec<NewRecord>,
    /// Every field instruction of the body, in BCI order, with the member each names and whether it
    /// was presented as a field access (P3 2.3, `field@1`).
    pub fields: Vec<FieldRecord>,
    /// Every dispatch-table read of the body, in BCI order (P3 2.3, `enumswitch@1`). What the record
    /// deliberately does not state is a mapping to enum constants: that is the enum class's own
    /// declaration, which this run never read.
    pub enum_switches: Vec<EnumSwitchRecord>,
    /// The body's constructor prologue, when the body is an instance initializer (P3 2.3, `init@1`).
    /// `None` for every body that is not one.
    pub init: Option<InitRecord>,
    /// What the run read of the member's declaration, and therefore what the artifact's envelope
    /// states (P3 2.3, `declaration@1`). Present for every produced artifact: every body has a
    /// declaration, and a run that cannot read one records the fact it was missing. `None` only when
    /// the run stopped before it read anything.
    pub declaration: Option<DeclarationRecord>,
    /// Every fallback the run had to keep, with its code.
    pub fallbacks: Vec<&'static str>,
    /// The evidence selection this run was presented under, and what each category delivered
    /// (change `add-demand-driven-core-results`, D1).
    ///
    /// The status list is fixed-size — one entry per category, whatever the selection was — and it is
    /// the answer to "was this asked for, and did it arrive", which no empty `Vec` and no `None` can
    /// give on its own. It is checked against this report's own payload before the run returns.
    pub evidence: RecoveryEvidence,
    /// The names the presentation decided, when the run reached the naming step.
    pub aliased_names: Vec<String>,
    /// What the run states about itself, in the fact layer's diagnostic vocabulary.
    pub diagnostics: Vec<Diagnostic>,
}

impl RecoveryReport {
    /// Whether the artifact is Java text this run wrote.
    pub fn produced(&self) -> bool {
        self.outcome.produced()
    }

    /// The stop reason, when the run stopped.
    pub fn stop(&self) -> Option<&StopReason> {
        self.outcome.stop()
    }

    /// The text one bytecode index reached, in writing order — the question the segment table
    /// answers and the one 3.2 grows on.
    pub fn text_of_bci(&self, bci: u32) -> Vec<&str> {
        self.source_map.text_of_bci(&self.text, bci)
    }

    /// States that the entry which performed this run's read materialized `ReadDetails` beside this
    /// report (change `add-demand-driven-core-results`, D3).
    ///
    /// `ReadDetails` is the one category this layer does not build: the read evidence belongs to the
    /// entry that performed the read and is published beside the report — `RecoveredMethod::callees`
    /// for the facade — so the entry states what it published here, and the list stays the report's
    /// own fixed-size statement about every category.
    ///
    /// The read is one step: an entry that published it published all of it, so the category is
    /// [`EvidenceState::Complete`], and a read that named no member at all is a legal *empty*
    /// complete result for the same reason an empty region table is. An entry that did not publish
    /// it — the run stopped before the read was materialized, or the selection was refused — states
    /// nothing here, and the category stays [`EvidenceState::NotPerformed`].
    pub fn read_details_materialized(&mut self) {
        self.evidence.delivered(RecoveryEvidenceKind::ReadDetails);
    }
}

/// Recovers one method's body.
///
/// The run is charged to the caller's budget: blocks and statements to `IrItems`, the region walk to
/// `AnalysisSteps`, the text to `OutputBytes`. Every charge happens before the work it pays for, so a
/// refusal leaves no work half done — see [`crate::stop`].
pub fn recover(request: &RecoveryRequest<'_>, budget: &mut Budget) -> RecoveryReport {
    let method = format!(
        "{}{}",
        request.facts.method().name(),
        request.facts.method().descriptor()
    );
    // The profile the presentation is written under, taken once: every path of this function —
    // including the ones that stop before any pass runs — echoes the request's own profile, so a
    // stopped report cannot claim a rule set the request did not declare.
    let profile = request.profile.clone();
    // The selection this run is presented under, taken once: every path below — the refusals, the
    // stopped reports and the report itself — echoes the request's own statement, so no report can
    // claim an evidence selection the request did not make.
    let selection = request.evidence.clone();
    let Some(canonical) = request.ir.canonical() else {
        return stopped(
            method,
            profile.clone(),
            &selection,
            StopReason::IrTableMissing { table: "canonical" },
            budget,
        );
    };
    let Some(frames) = request.ir.frames() else {
        return stopped(
            method,
            profile.clone(),
            &selection,
            StopReason::IrTableMissing { table: "frames" },
            budget,
        );
    };
    let Some(ssa) = request.ir.ssa() else {
        return stopped(
            method,
            profile.clone(),
            &selection,
            StopReason::IrTableMissing { table: "ssa" },
            budget,
        );
    };
    // The decode facts of the same run: the operations the presentation is written in, and the
    // exception table it states. A payload that holds a graph but no decode is not one run's
    // artifact (the graph is built from the decode), so this is a malformed payload rather than a
    // body without facts.
    let Some(code) = request.ir.code() else {
        return stopped(
            method,
            profile.clone(),
            &selection,
            StopReason::IrTableMissing { table: "code" },
            budget,
        );
    };
    // The request's own applicability check: a category this entry does not materialize, or a
    // driver range this body cannot support, is refused *before* anything is presented, with the
    // read this run already performed still charged. An unsupported or illegal selection is never
    // widened into a full-evidence delivery.
    if let Err(refusal) = selection.check(code) {
        return refused(method, profile.clone(), &selection, refusal, budget);
    }
    let operations = Operations::of(code, request.ir.constant_pool());
    if canonical.blocks().is_empty() {
        return stopped(
            method,
            profile.clone(),
            &selection,
            StopReason::IrTableMissing {
                table: "canonical blocks",
            },
            budget,
        );
    }
    let view = match NormalFlowView::build(canonical, budget) {
        Ok(view) => view,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    let recovered: Recovered = match crate::region::recover(
        canonical,
        &view,
        ssa,
        &operations,
        code,
        &request.profile,
        budget,
    ) {
        Ok(recovered) => recovered,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    // The slots the names are decided for are the body's own local slots: the frames table states
    // how many there are, and a local the debug metadata never named still needs a name.
    let slots = u16::try_from(frames.locals_slots()).unwrap_or(u16::MAX);
    // Which *variable* each slot holds, before any name is decided (P3 3.4): a slot the debug table
    // names over two disjoint ranges is two variables, and the naming below states a name for each.
    // The slot a guarded statement declares in its own header is never split, so the guard's own
    // naming rule is untouched.
    let reuse = reuse::plan(
        ssa,
        slots,
        request.facts.debug_locals(),
        &build::resource_slots(&recovered.regions),
    );
    let names = if request.facts.method().has_receiver() {
        // Slot 0 holds the receiver (JVMS 4.10.1.9), so it is spelled as one: the answer comes from
        // the member's own flags and from nothing else, which is why the naming is told it instead of
        // finding it out from a debug name or a slot ordinal.
        NameTable::build_with_receiver(request.facts.method().parameters(), slots, reuse.evidence())
    } else {
        NameTable::build(request.facts.method().parameters(), slots, reuse.evidence())
    };
    // The two shapes this run decides *before* a single statement is written, each from this run's
    // own tables: the concatenation chains the body builds (P3 2.2) and the bridge verdict for the
    // member itself, when its declaration or its body makes it one. Both are decisions about the
    // bytes, not about the text, which is why they are taken here and read by the builder.
    // Every plan below decides for every evidence selection: the premises are checked, the candidates
    // are verified or refused and the gaps are stated here, and none of these calls builds an owning
    // record. The records the request selected are materialized from these plans *after* the artifact
    // is committed (the evidence phase below), so a run that stops inside the evidence keeps its text.
    let chains = concat::plan(ssa, &operations);
    let bridge = bridge::plan(request.facts.method(), ssa, &operations);
    // The four shapes P3 2.3 reads — each decided before a statement is written, each from this run's
    // own tables. The construction sites reserve the concatenation chains' instructions, because one
    // instruction is never two shapes: the allocation a verified chain builds is written inside the
    // `+` expression and not a second time as a `new`.
    let sites = init::sites(ssa, &operations, chains.owned());
    let prologues = init::prologue(ssa, &operations, request.facts.method());
    let fields = field::plan(ssa, &operations, request.facts.method().declaring_class());
    let enums = enumswitch::plan(ssa, &operations);
    // The declaration is read from the two facts the caller stated and decides the artifact's
    // envelope; it never decides a statement, and it is the only shape of this slice that is read
    // without an instruction to read it from.
    let declaration = declaration::plan(request.facts.method());
    // The type each parameter slot holds, as the member's own **descriptor** states it (P3-R5): the
    // frames cannot tell a `boolean` parameter from an `int` one, and the descriptor can.
    let parameter_types = request.facts.method().parameter_types();
    // The same reading for the **return** position: the type this member's own signature returns,
    // which decides both the boolean shape (`(I)Z`, `()Z` and the rest of the descriptor spellings
    // are the same fact) and the conversion every other `return` type requires of its value. The
    // builder reads it as this run's fact — like the parameter types, it is derived here rather than
    // re-read out of a descriptor inside the layer that writes the statements.
    let return_type = build::return_type(request.facts.method().descriptor());
    let program = match build::build(
        canonical,
        ssa,
        &operations,
        build::Inputs {
            pool: request.ir.constant_pool(),
            bootstrap: request.ir.bootstrap_methods(),
            profile: request.profile.clone(),
            parameters: request.facts.method().parameters(),
            parameter_types: &parameter_types,
            return_type,
            names: &names,
            reuse: &reuse,
            chains: &chains,
            members: request.members,
            bridge: bridge.as_ref(),
            sites: &sites,
            prologues: &prologues,
            fields: &fields,
            enums: &enums,
        },
        &recovered.regions,
        budget,
    ) {
        Ok(program) => program,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    // The identity of the body being presented, as the payload's own declaration states it: the
    // member every anchor of this artifact belongs to (P3 3.2). A run that read no member header
    // states none. Both emitter passes state it for the anchors that name no member of their own.
    let member = request
        .ir
        .declaration()
        .map(|declaration| declaration.identity());
    let emitted: Emitted = match emit(
        &program.stmts,
        request.facts,
        declaration.declaration(),
        member,
        budget,
    ) {
        Ok(emitted) => emitted,
        Err(stop) => return stopped(method, profile.clone(), &selection, stop, budget),
    };
    // What the artifact that was just committed holds. The classification is taken here, from the
    // emission itself, and not from the AST the build had produced: a statement that was built and
    // then never committed (the emitter stopped inside it) is not in any artifact, and the run that
    // stopped is classified by [`stopped`], which never asks this question.
    let content = content_of(&emitted);

    // The planes, each from its own input.
    let structured = recovered.is_structured() && !program.ragged;
    let fallbacks: Vec<&'static str> = recovered
        .fallbacks()
        .iter()
        .map(FallbackReason::code)
        .collect();
    let mut diagnostics = Vec::new();
    for region in &recovered.regions {
        for reason in region.fallbacks() {
            diagnostics.push(diagnostic(
                reason.code(),
                DiagnosticSeverity::Warning,
                &reason.message(),
            ));
        }
    }
    // The names the presentation could not write as the source spelled them: a core gap, stated in
    // every selection from the naming table itself. What it states is the count and the affected
    // slots — the alias the run wrote instead, and the raw spelling it could not write, are
    // NameDetails and are materialized only when the request selects them.
    let aliased_slots: Vec<String> = names
        .names()
        .filter(|name| name.aliased().is_some())
        .map(|name| format!("slot {}", name.slot()))
        .collect();
    if !aliased_slots.is_empty() {
        diagnostics.push(diagnostic(
            "jre_name_aliased",
            DiagnosticSeverity::Warning,
            &format!(
                "{} local name(s) could not be written as the source spelled them: {}",
                aliased_slots.len(),
                aliased_slots.join(", ")
            ),
        ));
    }
    diagnostics.push(diagnostic(
        "jre_recovery_produced",
        DiagnosticSeverity::Info,
        &format!(
            "{} region(s), {} statement(s), {} segment(s), {} byte(s) written from {} canonical block(s)",
            recovered.regions.len(),
            program.statements,
            emitted.segments,
            emitted.written,
            recovered.blocks
        ),
    ));
    // The rule index: every rule that decided something about this body, each once, in the order
    // this layer reads them. It is rule evidence — a category the request selects or does not — and
    // it is read from the *plans*, so the driver range never changes it: a range selects records,
    // not the decisions they are written from, and the decisions are the same for every selection.
    let mut rules = if selection.requests(RecoveryEvidenceKind::RuleDetails) {
        recovered.rules()
    } else {
        Vec::new()
    };
    if selection.requests(RecoveryEvidenceKind::RuleDetails) {
        // A dynamic site is a rule's answer too: the decision names `lambda@1` whether it presented
        // the site or refused it, so the report's rule list states both. A body with no site names no
        // lambda rule, which is why the list is built from the plan rather than from a record table.
        let mut answered: Vec<RuleVersion> = Vec::new();
        if !program.lambdas.is_empty() {
            answered.push(crate::lambda::RULE);
        }
        if chains.answered() {
            answered.push(concat::RULE);
        }
        if !program.accessors.is_empty() {
            answered.push(crate::accessor::RULE);
        }
        if bridge.is_some() {
            answered.push(bridge::RULE);
        }
        if sites.answered() {
            answered.push(init::NEW_RULE);
        }
        if fields.answered() {
            answered.push(field::RULE);
        }
        if enums.answered() {
            answered.push(enumswitch::RULE);
        }
        if prologues.answered() {
            answered.push(init::INIT_RULE);
        }
        // The declaration rule is listed when it *read* a declaration: its output is the envelope
        // line, and a run that was not told the declaration facts wrote none. The refusal is not
        // invisible for that — the gap and its diagnostic name the rule — but a rule that concluded
        // nothing about these bytes is not a rule that produced this artifact.
        if declaration.declaration().is_some() {
            answered.push(declaration::RULE);
        }
        for rule in answered {
            if !rules.contains(&rule) {
                rules.push(rule);
            }
        }
    }
    // The refusals are diagnostics of their own: a site that was not presented says which link of
    // the chain failed, whether it was the class's table, the factory, the SAM's shape or a value
    // this layer may not replay (A04). Every one of them is stated from the *gap* the rule recorded
    // where it decided, so closing the rule records does not close the gaps.
    for gap in &program.lambda_refusals {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let lambda_sites = program.lambdas_presented + count_of(program.lambda_refusals.len());
    if lambda_sites > 0 {
        diagnostics.push(diagnostic(
            "jre_lambda_sites",
            DiagnosticSeverity::Info,
            &format!(
                "{lambda_sites} dynamic site(s) read under {}: {} presented, {} refused",
                LAMBDA.rule(),
                program.lambdas_presented,
                program.lambda_refusals.len(),
            ),
        ));
    }
    // The three shapes of 2.2 report the same way: every refusal is a diagnostic of its own, and a
    // summary states how many candidates were read and how many were presented. A run that read no
    // candidate of a shape says nothing about that shape's rule at all.
    for gap in chains.refusals() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let (concat_read, concat_presented) = chains.counts();
    if concat_read > 0 {
        diagnostics.push(diagnostic(
            "jre_concat_chains",
            DiagnosticSeverity::Info,
            &format!(
                "{concat_read} concatenation candidate(s) read under {}: {concat_presented} presented, {} refused",
                CONCAT.rule(),
                concat_read - concat_presented,
            ),
        ));
    }
    for gap in &program.accessor_refusals {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let accessor_sites = program.accessors_presented + count_of(program.accessor_refusals.len());
    if accessor_sites > 0 {
        diagnostics.push(diagnostic(
            "jre_accessor_sites",
            DiagnosticSeverity::Info,
            &format!(
                "{accessor_sites} accessor call site(s) read under {}: {} presented as a field access, {} refused",
                ACCESSOR.rule(),
                program.accessors_presented,
                program.accessor_refusals.len(),
            ),
        ));
    }
    if let Some(gap) = bridge.as_ref().and_then(|plan| plan.refusal()) {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    if let Some(plan) = bridge.as_ref() {
        diagnostics.push(diagnostic(
            "jre_bridge",
            DiagnosticSeverity::Info,
            &format!(
                "the body was read under {}: {}",
                BRIDGE.rule(),
                if plan.presented() {
                    "presented as the forward it is"
                } else {
                    "not presented as a forward"
                }
            ),
        ));
    }
    // P3 2.3's four shapes report the same way the three of 2.2 do: every refusal is a diagnostic of
    // its own, and a summary states how many candidates were read and how many were presented. A body
    // that read none of a shape says nothing about that shape's rule at all — except for the
    // declaration, which every body has and which therefore always states what it read.
    for gap in sites.refusals() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let (new_read, new_presented) = sites.counts();
    if new_read > 0 {
        diagnostics.push(diagnostic(
            "jre_new_sites",
            DiagnosticSeverity::Info,
            &format!(
                "{new_read} construction candidate(s) read under {}: {new_presented} presented as `new`, {} refused",
                NEW.rule(),
                new_read - new_presented,
            ),
        ));
    }
    for gap in fields.refusals() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let (field_read, field_presented) = fields.counts();
    if field_read > 0 {
        diagnostics.push(diagnostic(
            "jre_field_accesses",
            DiagnosticSeverity::Info,
            &format!(
                "{field_read} field instruction(s) read under {}: {field_presented} presented, {} refused",
                FIELD.rule(),
                field_read - field_presented,
            ),
        ));
    }
    for gap in enums.refusals() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    }
    let (enum_read, enum_presented) = enums.counts();
    if enum_read > 0 {
        // The boundary is stated where the shape is: the read is the bytecode's own dispatch, and the
        // mapping from its entries to enum constants is a class-level fact of *another* class this
        // run never read (P3 2.3).
        diagnostics.push(diagnostic(
            "jre_enumswitch",
            DiagnosticSeverity::Info,
            &format!(
                "{enum_read} dispatch-table read(s) read under {}: {enum_presented} presented as the table read the bytecode performs, {} refused; the constants those entries stand for are the enum class's own declaration, which this run does not hold, so no `case T.CONST:` label is written",
                ENUMSWITCH.rule(),
                enum_read - enum_presented,
            ),
        ));
    }
    if let Some(gap) = prologues.refusal() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    } else if let Some(prologue) = prologues.prologue() {
        diagnostics.push(diagnostic(
            "jre_constructor_prologue",
            DiagnosticSeverity::Info,
            &format!(
                "the body is an instance initializer and its prologue is written under {} as `{}`: the call at BCI {} names `{}`, and the class that declares this constructor is `{}`",
                INIT.rule(),
                prologue.target.spell(),
                prologue.bci,
                prologue.class,
                prologue.declared,
            ),
        ));
    }
    if let Some(gap) = declaration.refusal() {
        diagnostics.push(diagnostic(
            gap.code(),
            DiagnosticSeverity::Warning,
            gap.message(),
        ));
    } else if let Some(declared) = declaration.declaration() {
        diagnostics.push(diagnostic(
            "jre_declaration",
            DiagnosticSeverity::Info,
            &format!(
                "the artifact's envelope states the member's declaration under {}: {}",
                DECLARATION.rule(),
                declared.form.spell()
            ),
        ));
    }
    // ---------------------------------------------------------------------------------------
    // The evidence phase: the artifact is committed and every diagnostic of this run is stated, and
    // what the request selected is materialized now — one owning record at a time, within the same
    // budget, in the fixed order this layer states its categories in.
    //
    // Every category is materialized *after* this line, including the rule records and the segment
    // table: the rules decided before the artifact was written, but their owning records are written
    // from the plans here. A refusal in the phase therefore stops the *materialization* and nothing
    // else — the text, its planes and the gaps above stay exactly what this run produced, the
    // category in flight reports the prefix it delivered, the categories the phase never reached
    // stay `NotPerformed`, and the run's execution states the real stop.
    // ---------------------------------------------------------------------------------------
    let publication = Publication::of(&selection);
    let mut evidence = RecoveryEvidence::pending(&selection);
    let mut phase = EvidencePhase::new();
    let range = selection.driver_bci_range();

    // The region records: the run holds every region either way, and the records are built here —
    // one by one, each after the charge that pays for it — only for the regions the selected driver
    // range intersects. A region is kept or dropped as a unit, with the origins it states for
    // itself.
    let (regions, reached) = if selection.requests(RecoveryEvidenceKind::RegionDetails) {
        phase.materialize(
            budget,
            recovered
                .regions
                .iter()
                .filter(|region| in_driver_range(region, range)),
            region_record,
        )
    } else {
        (Vec::new(), Materialized::None)
    };
    evidence.materialized(RecoveryEvidenceKind::RegionDetails, reached);

    // Every rule record of the run, in the order this layer reads the rules. One category, nine
    // plans: the category is complete only when every one of its records was materialized, and the
    // phase's own stop ends it wherever it lands.
    let mut lambdas: Vec<LambdaRecord> = Vec::new();
    let mut concats: Vec<ConcatRecord> = Vec::new();
    let mut accessors: Vec<AccessorRecord> = Vec::new();
    let mut bridges: Vec<BridgeRecord> = Vec::new();
    let mut news: Vec<NewRecord> = Vec::new();
    let mut field_records: Vec<FieldRecord> = Vec::new();
    let mut enum_switches: Vec<EnumSwitchRecord> = Vec::new();
    let mut init: Option<InitRecord> = None;
    let mut declaration_record: Option<DeclarationRecord> = None;
    if selection.requests(RecoveryEvidenceKind::RuleDetails) {
        let mut delivery = Delivery::new(&phase);
        lambdas = delivery.take(program.materialize_lambdas(publication, &mut phase, budget));
        concats = delivery.take(chains.materialize(publication, &mut phase, budget));
        accessors = delivery.take(program.materialize_accessors(publication, &mut phase, budget));
        bridges = match bridge.as_ref() {
            Some(plan) => delivery
                .take_one(plan.materialize(publication, &mut phase, budget))
                .into_iter()
                .collect(),
            None => Vec::new(),
        };
        news = delivery.take(sites.materialize(publication, &mut phase, budget));
        field_records = delivery.take(fields.materialize(publication, &mut phase, budget));
        enum_switches = delivery.take(enums.materialize(publication, &mut phase, budget));
        init = prologues
            .answered()
            .then(|| delivery.take_one(prologues.materialize(publication, &mut phase, budget)))
            .flatten();
        declaration_record =
            delivery.take_one(declaration.materialize(publication, &mut phase, budget));
        evidence.materialized(RecoveryEvidenceKind::RuleDetails, delivery.reached());
    }

    // The names the presentation had to replace: per local slot rather than per bytecode index, so a
    // driver range neither selects nor drops one of them — a request that selects this category over
    // a range gets it whole.
    let (aliased_names, reached) = if selection.requests(RecoveryEvidenceKind::NameDetails) {
        phase.materialize(
            budget,
            names.names().filter(|name| name.aliased().is_some()),
            aliased_name,
        )
    } else {
        (Vec::new(), Materialized::None)
    };
    evidence.materialized(RecoveryEvidenceKind::NameDetails, reached);

    // The source map: the last category, because it is the only one that replays the whole decided
    // AST. The committing pass wrote the text and owns no table; this pass writes no text and
    // verifies every byte it re-writes against the artifact the committing pass produced.
    let mut gate_stop: Option<StopReason> = None;
    let (source_map, reached) = if selection.requests(RecoveryEvidenceKind::SourceMap) {
        match emit_source_map(
            &program.stmts,
            request.facts,
            declaration.declaration(),
            member,
            SegmentPublication::of(&selection),
            &emitted,
            &mut phase,
            budget,
        ) {
            Ok((map, reached)) => (map, reached),
            Err(stop) => {
                gate_stop = Some(stop);
                (SourceMap::default(), Materialized::None)
            }
        }
    } else {
        (SourceMap::default(), Materialized::None)
    };
    if gate_stop.is_none() {
        evidence.materialized(RecoveryEvidenceKind::SourceMap, reached);
    }

    // The stop of the evidence phase, when it stopped — and the one the source-map gate states when
    // the replayed writes disagreed with the committed artifact: both are stated in the same
    // vocabulary every other stop of this layer uses, and neither is a claim about the artifact.
    let stop = gate_stop.or_else(|| phase.stopped().then(|| phase.reason(budget)));
    let execution = match &stop {
        Some(reason) => {
            diagnostics.push(stop_diagnostic(reason));
            stop_execution(reason, budget.usage())
        }
        None => ExecutionReport::Complete {
            usage: budget.usage(),
        },
    };
    let report = RecoveryReport {
        profile: request.profile.clone(),
        representation: if structured {
            Representation::Java
        } else {
            Representation::Mixed
        },
        quality: if structured {
            Quality::Structured
        } else {
            Quality::Fallback
        },
        syntax_status: if !structured || names.any_aliased() {
            SyntaxStatus::NotJava
        } else {
            SyntaxStatus::Unchecked
        },
        compile_status: CompileStatus::NotAttempted,
        semantic_validation: SemanticValidation::Unproven,
        verification: VerificationStatus::NotPerformed,
        execution,
        outcome: RecoveryOutcome::Produced,
        content,
        text: emitted.text,
        source_map,
        regions,
        lambdas,
        concats,
        accessors,
        bridges,
        news,
        fields: field_records,
        enum_switches,
        init,
        declaration: declaration_record,
        fallbacks,
        aliased_names,
        diagnostics,
        method,
        rules,
        evidence,
    };
    debug_assert!(
        report.evidence.agrees_with(&report),
        "the evidence status list disagrees with the payload it describes"
    );
    report
}

/// One category's materialization across the several plans that publish into it.
///
/// `RuleDetails` is one category and nine rules: the phase delivers their records in one order, and
/// the category is `Complete` only when it reached the end of every one of them. The accumulator is
/// where "reached the end" of several materializations becomes the one state the list states, and it
/// never deletes a record an earlier plan delivered.
struct Delivery {
    /// Whether the phase reached the end of every plan seen so far.
    complete: bool,
    /// How many owning records the category holds so far.
    records: u64,
}

impl Delivery {
    /// A category nothing has been materialized for yet, in a phase that has not stopped.
    fn new(phase: &EvidencePhase) -> Self {
        Self {
            complete: !phase.stopped(),
            records: 0,
        }
    }

    /// Takes one plan's materialization into the category's own account.
    fn take<T>(&mut self, (records, reached): (Vec<T>, Materialized)) -> Vec<T> {
        self.records = self
            .records
            .saturating_add(u64::try_from(records.len()).unwrap_or(u64::MAX));
        if !matches!(reached, Materialized::Complete) {
            self.complete = false;
        }
        records
    }

    /// The same, for a plan that publishes at most one record.
    fn take_one<T>(&mut self, (record, reached): (Option<T>, Materialized)) -> Option<T> {
        self.take((record.into_iter().collect::<Vec<T>>(), reached))
            .into_iter()
            .next()
    }

    /// What the category reached, as the list states it.
    fn reached(&self) -> Materialized {
        if self.complete {
            Materialized::Complete
        } else if self.records == 0 {
            Materialized::None
        } else {
            Materialized::Partial {
                delivered: self.records,
            }
        }
    }
}

/// One replaced name, as a selected `NameDetails` category states it.
fn aliased_name(name: &crate::names::RenderedName) -> String {
    crate::demand_counts::record_built(RecoveryEvidenceKind::NameDetails);
    format!(
        "slot {} written as `{}` (source spelling `{}`)",
        name.slot(),
        name.text(),
        name.raw().unwrap_or("")
    )
}

/// One count as the report states it.
fn count_of(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// Whether one region is inside the selected driver range.
///
/// The range selects the records that *intersect* it and a record is kept or dropped as a unit: the
/// region's own blocks are the positions it states for itself.
fn in_driver_range(region: &Region, range: Option<crate::evidence::BytecodeRange>) -> bool {
    range.is_none_or(|range| range.intersects_any(region.blocks().iter().map(|block| block.bci())))
}

/// One region record, with its fallbacks stated: what a selected `RegionDetails` category holds for
/// one region, built when the evidence phase materializes it.
///
/// The construction is counted **here**, in the function that builds the record, rather than at the
/// call site: a record built anywhere and then dropped is a record that was built, and the port
/// (`crate::demand_counts`) exists to say so.
fn region_record(region: &Region) -> RegionRecord {
    crate::demand_counts::record_built(RecoveryEvidenceKind::RegionDetails);
    let blocks: Vec<u32> = region.blocks().iter().map(|block| block.bci()).collect();
    let reasons = region.fallbacks();
    RegionRecord {
        bci: blocks.first().copied().unwrap_or(0),
        structured: region.is_structured(),
        blocks,
        code: reasons.first().map(FallbackReason::code),
        message: reasons.first().map(FallbackReason::message),
        rule: region.rule(),
    }
}

impl EvidencePayload for RecoveryReport {
    /// How many owning records of one category this report holds right now.
    ///
    /// The count is read off the payload itself — the same fields the status list is checked against
    /// — so the check cannot drift from what a caller receives.
    fn owning_records(&self, kind: RecoveryEvidenceKind) -> u64 {
        match kind {
            RecoveryEvidenceKind::SourceMap => count_of(self.source_map.len()),
            RecoveryEvidenceKind::RegionDetails => count_of(self.regions.len()),
            RecoveryEvidenceKind::RuleDetails => count_of(
                self.lambdas.len()
                    + self.concats.len()
                    + self.accessors.len()
                    + self.bridges.len()
                    + self.news.len()
                    + self.fields.len()
                    + self.enum_switches.len()
                    + usize::from(self.init.is_some())
                    + usize::from(self.declaration.is_some()),
            ),
            RecoveryEvidenceKind::NameDetails => count_of(self.aliased_names.len()),
            // The read evidence a presentation consumed is published beside this report by the entry
            // that performed the read: this layer materializes none of it yet.
            RecoveryEvidenceKind::ReadDetails => 0,
        }
    }
}

/// The content of a committed artifact, from the statements its emission wrote: an artifact that
/// holds only the envelope, the reasons and the quoted bytecode is [`RecoveryContent::ExplanationOnly`],
/// and one statement the emitter spelled as Java makes it [`RecoveryContent::ContainsStatements`].
fn content_of(emitted: &Emitted) -> RecoveryContent {
    if emitted.statements == 0 {
        RecoveryContent::ExplanationOnly
    } else {
        RecoveryContent::ContainsStatements
    }
}

/// The report of a run that stopped: no text, no segments, and an execution plane that says so.
fn stopped(
    method: String,
    profile: RecoveryProfile,
    selection: &RecoveryEvidenceRequest,
    reason: StopReason,
    budget: &Budget,
) -> RecoveryReport {
    let execution = stop_execution(&reason, budget.usage());
    RecoveryReport {
        method,
        profile,
        rules: Vec::new(),
        representation: Representation::Bytecode,
        quality: Quality::Fallback,
        syntax_status: SyntaxStatus::NotJava,
        compile_status: CompileStatus::NotAttempted,
        semantic_validation: SemanticValidation::Unproven,
        verification: VerificationStatus::NotPerformed,
        execution,
        outcome: RecoveryOutcome::Stopped(reason.clone()),
        // Nothing was committed, so there is no content to describe. A stop is never classified from
        // whatever the failed run had built or written before it refused.
        content: RecoveryContent::NotProduced,
        text: String::new(),
        source_map: SourceMap::default(),
        regions: Vec::new(),
        lambdas: Vec::new(),
        concats: Vec::new(),
        accessors: Vec::new(),
        bridges: Vec::new(),
        news: Vec::new(),
        fields: Vec::new(),
        enum_switches: Vec::new(),
        init: None,
        declaration: None,
        fallbacks: Vec::new(),
        // Nothing of a selected category was materialized, and the status list says exactly that
        // rather than leaving an empty `Vec` to be read as "this body has no such evidence".
        evidence: RecoveryEvidence::pending(selection),
        aliased_names: Vec::new(),
        diagnostics: vec![stop_diagnostic(&reason)],
    }
}

/// The report of a run whose own evidence selection cannot be answered for this body.
///
/// The refusal is the *request's* fact, not the body's: the decode succeeded, the run could have
/// presented the method, and what it cannot do is apply the selection the caller stated. So nothing
/// is presented — a refused selection is never widened into a full-evidence delivery, and a category
/// this entry does not materialize is never answered with nothing — every category the request
/// selected states `NotPerformed`, and the report carries the refusal's own code and position.
fn refused(
    method: String,
    profile: RecoveryProfile,
    selection: &RecoveryEvidenceRequest,
    refusal: EvidenceRefusal,
    budget: &Budget,
) -> RecoveryReport {
    let reason = StopReason::EvidenceRefused {
        code: refusal.code,
        at: refusal.at,
        message: refusal.message,
    };
    RecoveryReport {
        method,
        profile,
        rules: Vec::new(),
        representation: Representation::Bytecode,
        quality: Quality::Fallback,
        syntax_status: SyntaxStatus::NotJava,
        compile_status: CompileStatus::NotAttempted,
        semantic_validation: SemanticValidation::Unproven,
        verification: VerificationStatus::NotPerformed,
        execution: stop_execution(&reason, budget.usage()),
        outcome: RecoveryOutcome::Stopped(reason.clone()),
        content: RecoveryContent::NotProduced,
        text: String::new(),
        source_map: SourceMap::default(),
        regions: Vec::new(),
        lambdas: Vec::new(),
        concats: Vec::new(),
        accessors: Vec::new(),
        bridges: Vec::new(),
        news: Vec::new(),
        fields: Vec::new(),
        enum_switches: Vec::new(),
        init: None,
        declaration: None,
        fallbacks: Vec::new(),
        evidence: RecoveryEvidence::pending(selection),
        aliased_names: Vec::new(),
        diagnostics: vec![stop_diagnostic(&reason)],
    }
}

/// The execution plane one stop states, in the fact layer's own vocabulary.
fn stop_execution(reason: &StopReason, usage: UsageSnapshot) -> ExecutionReport {
    match reason {
        StopReason::IrTableMissing { .. } => ExecutionReport::Partial {
            reason: TerminationReason::Unsupported {
                code: "jre_ir_table_missing".to_string(),
            },
            usage,
        },
        StopReason::EvidenceRefused { code, .. } => ExecutionReport::Partial {
            reason: TerminationReason::Unsupported {
                code: (*code).to_string(),
            },
            usage,
        },
        StopReason::Budget { dimension, .. } => ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::from(*dimension),
            },
            usage,
        },
        StopReason::Cancelled { .. } => ExecutionReport::Cancelled { usage },
        StopReason::Interrupted { code, .. } => ExecutionReport::Partial {
            reason: TerminationReason::Error {
                code: (*code).to_string(),
            },
            usage,
        },
    }
}

/// The one diagnostic a stop states: its code, its severity and its sentence, in the vocabulary the
/// fact layer already uses.
fn stop_diagnostic(reason: &StopReason) -> Diagnostic {
    let (code, severity) = match reason {
        StopReason::IrTableMissing { .. } => ("jre_ir_table_missing", DiagnosticSeverity::Error),
        StopReason::EvidenceRefused { code, .. } => (*code, DiagnosticSeverity::Error),
        StopReason::Budget { .. } => ("jre_output_budget", DiagnosticSeverity::Error),
        StopReason::Cancelled { .. } => ("jre_cancelled", DiagnosticSeverity::Warning),
        StopReason::Interrupted { code, .. } => (*code, DiagnosticSeverity::Error),
    };
    let message = match reason {
        StopReason::IrTableMissing { table } => format!(
            "the payload of this run has no {table} table, so no body can be presented from it"
        ),
        StopReason::Budget {
            dimension,
            written,
            limit,
            at,
        } => format!(
            "the run stopped on {dimension:?} after {written} byte(s) of {limit}, at {}",
            at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
        ),
        StopReason::Cancelled { at } => format!(
            "the run was cancelled{}",
            at.map_or(String::new(), |bci| format!(" at BCI {bci}"))
        ),
        StopReason::Interrupted { code, at } => match *code {
            // The three ways this run can be interrupted at a node are different facts about it —
            // "the input went deeper than the bound", "the walk was back inside a structure it is
            // already building" and "a budget poll refused" — and the message names which one
            // stopped it, at which node. The budget's own wording is the one it always had.
            crate::stop::RECURSION_REENTRY_CODE => format!(
                "the recovery recursion re-entered a block this run had already entered and cannot \
                 complete it: the run stopped at the recursion bound ({code}) at {}",
                at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
            ),
            crate::stop::RECURSION_BOUND_CODE => format!(
                "the recovery recursion reached its explicit depth bound ({code}) at {}",
                at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
            ),
            crate::stop::BUDGET_INTERRUPTED_CODE => format!(
                "the budget interrupted the run ({code}) at {}",
                at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
            ),
            crate::stop::SOURCE_MAP_MISMATCH_CODE => format!(
                "the source-map replay of this run did not agree with the artifact the committing                  pass wrote: the map cannot be attached to that text, so it is not delivered and the                  artifact is left exactly as it was ({code})"
            ),
            _ => format!(
                "the run was interrupted ({code}) at {}",
                at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
            ),
        },
        StopReason::EvidenceRefused { message, .. } => message.clone(),
    };
    diagnostic(code, severity, &message)
}

/// One diagnostic in the fact layer's vocabulary.
fn diagnostic(code: &str, severity: DiagnosticSeverity, message: &str) -> Diagnostic {
    Diagnostic {
        code: code.to_string(),
        severity,
        message: message.to_string(),
        provenance: None,
    }
}
