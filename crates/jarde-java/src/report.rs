//! The recovery entry point and the report it produces (P3 1.3's public seam).
//!
//! # One run, one request, one artifact
//!
//! [`recover`] takes the P2 payload by reference ([`MethodIr`], the 1.1 handoff) plus the facts the
//! layer below read ([`RecoveryFacts`]), and produces at most one artifact: the Java text, its
//! segment table, a diagnostic list, and the six planes that describe what was produced.
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
use crate::emit::{Emitted, emit};
use crate::enumswitch::{self, EnumSwitchRecord};
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
}

impl<'a> RecoveryRequest<'a> {
    /// One request over one payload, one fact set and one profile, with no member table.
    pub fn new(ir: &'a MethodIr, facts: &'a RecoveryFacts, profile: RecoveryProfile) -> Self {
        Self {
            ir,
            facts,
            profile,
            members: None,
        }
    }

    /// The same request, with the class's other members: the evidence a synthetic accessor call site
    /// is decided from (P3 2.2, A12).
    pub fn with_members(mut self, members: &'a ClassMembers) -> Self {
        self.members = Some(members);
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
    let Some(canonical) = request.ir.canonical() else {
        return stopped(
            method,
            profile.clone(),
            StopReason::IrTableMissing { table: "canonical" },
            budget,
        );
    };
    let Some(frames) = request.ir.frames() else {
        return stopped(
            method,
            profile.clone(),
            StopReason::IrTableMissing { table: "frames" },
            budget,
        );
    };
    let Some(ssa) = request.ir.ssa() else {
        return stopped(
            method,
            profile.clone(),
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
            StopReason::IrTableMissing { table: "code" },
            budget,
        );
    };
    let operations = Operations::of(code, request.ir.constant_pool());
    if canonical.blocks().is_empty() {
        return stopped(
            method,
            profile.clone(),
            StopReason::IrTableMissing {
                table: "canonical blocks",
            },
            budget,
        );
    }
    let view = match NormalFlowView::build(canonical, budget) {
        Ok(view) => view,
        Err(stop) => return stopped(method, profile.clone(), stop, budget),
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
        Err(stop) => return stopped(method, profile.clone(), stop, budget),
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
    let names = NameTable::build(request.facts.method().parameters(), slots, reuse.evidence());
    // The two shapes this run decides *before* a single statement is written, each from this run's
    // own tables: the concatenation chains the body builds (P3 2.2) and the bridge verdict for the
    // member itself, when its declaration or its body makes it one. Both are decisions about the
    // bytes, not about the text, which is why they are taken here and read by the builder.
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
        Err(stop) => return stopped(method, profile.clone(), stop, budget),
    };
    let emitted: Emitted = match emit(
        &program.stmts,
        request.facts,
        declaration.declaration(),
        // The identity of the body being presented, as the payload's own declaration states it: the
        // member every anchor of this artifact belongs to (P3 3.2). A run that read no member header
        // states none.
        request
            .ir
            .declaration()
            .map(|declaration| declaration.identity()),
        budget,
    ) {
        Ok(emitted) => emitted,
        Err(stop) => return stopped(method, profile.clone(), stop, budget),
    };

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
    let aliased_names: Vec<String> = names
        .names()
        .filter(|name| name.aliased().is_some())
        .map(|name| {
            format!(
                "slot {} written as `{}` (source spelling `{}`)",
                name.slot(),
                name.text(),
                name.raw().unwrap_or("")
            )
        })
        .collect();
    if !aliased_names.is_empty() {
        diagnostics.push(diagnostic(
            "jre_name_aliased",
            DiagnosticSeverity::Warning,
            &format!(
                "{} local name(s) could not be written as the source spelled them: {}",
                aliased_names.len(),
                aliased_names.join("; ")
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
            emitted.source_map.len(),
            emitted.written,
            recovered.blocks
        ),
    ));
    let regions = region_records(&recovered.regions);
    let mut rules = recovered.rules();
    // A dynamic site is a rule's answer too: the record names `lambda@1` whether it presented the
    // site or refused it, so the report's rule list states both. A body with no site names no
    // lambda rule, which is why the list is built from the records rather than from the table.
    for lambda in &program.lambdas {
        let rule = lambda.rule();
        if !rules.contains(&rule) {
            rules.push(rule);
        }
    }
    // The three shapes of P3 2.2 are rules' answers in the same way, and each record says which
    // rule: a body with none of them names none of the rules.
    let concats = chains.records().to_vec();
    let bridges: Vec<BridgeRecord> = bridge.iter().map(|plan| plan.record().clone()).collect();
    for rule in concats
        .iter()
        .map(ConcatRecord::rule)
        .chain(program.accessors.iter().map(AccessorRecord::rule))
        .chain(bridges.iter().map(BridgeRecord::rule))
        .chain(sites.records().iter().map(NewRecord::rule))
        .chain(fields.records().iter().map(FieldRecord::rule))
        .chain(enums.records().iter().map(EnumSwitchRecord::rule))
        .chain(prologues.record().map(InitRecord::rule))
        // The declaration rule is listed when it *wrote* something: its output is the envelope line,
        // and a run that was not told the declaration facts wrote none. The refusal is not invisible
        // for that — the record and its diagnostic name the rule — but a rule that concluded nothing
        // about these bytes is not a rule that produced this artifact.
        .chain(
            declaration
                .record()
                .presented()
                .then(|| declaration.record().rule()),
        )
    {
        if !rules.contains(&rule) {
            rules.push(rule);
        }
    }
    // The refusals are diagnostics of their own: a site that was not presented says which link of
    // the chain failed, whether it was the class's table, the factory, the SAM's shape or a value
    // this layer may not replay (A04).
    for lambda in &program.lambdas {
        if let Some(refusal) = &lambda.refusal {
            diagnostics.push(diagnostic(
                refusal.code,
                DiagnosticSeverity::Warning,
                &refusal.message,
            ));
        }
    }
    if !program.lambdas.is_empty() {
        diagnostics.push(diagnostic(
            "jre_lambda_sites",
            DiagnosticSeverity::Info,
            &format!(
                "{} dynamic site(s) read under {}: {} presented, {} refused",
                program.lambdas.len(),
                LAMBDA.rule(),
                program
                    .lambdas
                    .iter()
                    .filter(|site| site.presented())
                    .count(),
                program
                    .lambdas
                    .iter()
                    .filter(|site| !site.presented())
                    .count(),
            ),
        ));
    }
    // The three shapes of 2.2 report the same way: every refusal is a diagnostic of its own, and a
    // summary states how many candidates were read and how many were presented. A run that read no
    // candidate of a shape says nothing about that shape's rule at all.
    for record in &concats {
        if let Some(refusal) = &record.refusal {
            diagnostics.push(diagnostic(
                refusal.code,
                DiagnosticSeverity::Warning,
                &refusal.message,
            ));
        }
    }
    if !concats.is_empty() {
        diagnostics.push(diagnostic(
            "jre_concat_chains",
            DiagnosticSeverity::Info,
            &format!(
                "{} concatenation candidate(s) read under {}: {} presented, {} refused",
                concats.len(),
                CONCAT.rule(),
                concats.iter().filter(|record| record.presented()).count(),
                concats.iter().filter(|record| !record.presented()).count(),
            ),
        ));
    }
    for record in &program.accessors {
        if let Some(refusal) = &record.refusal {
            diagnostics.push(diagnostic(
                refusal.code,
                DiagnosticSeverity::Warning,
                &refusal.message,
            ));
        }
    }
    if !program.accessors.is_empty() {
        diagnostics.push(diagnostic(
            "jre_accessor_sites",
            DiagnosticSeverity::Info,
            &format!(
                "{} accessor call site(s) read under {}: {} presented as a field access, {} refused",
                program.accessors.len(),
                ACCESSOR.rule(),
                program
                    .accessors
                    .iter()
                    .filter(|record| record.presented())
                    .count(),
                program
                    .accessors
                    .iter()
                    .filter(|record| !record.presented())
                    .count(),
            ),
        ));
    }
    for record in &bridges {
        if let Some(refusal) = &record.refusal {
            diagnostics.push(diagnostic(
                refusal.code,
                DiagnosticSeverity::Warning,
                &refusal.message,
            ));
        }
    }
    if !bridges.is_empty() {
        diagnostics.push(diagnostic(
            "jre_bridge",
            DiagnosticSeverity::Info,
            &format!(
                "the body was read under {}: {}",
                BRIDGE.rule(),
                if bridges.iter().any(|record| record.presented()) {
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
    for record in sites.records() {
        if let Some(refusal) = &record.refusal {
            diagnostics.push(diagnostic(
                refusal.code,
                DiagnosticSeverity::Warning,
                &refusal.message,
            ));
        }
    }
    if !sites.records().is_empty() {
        diagnostics.push(diagnostic(
            "jre_new_sites",
            DiagnosticSeverity::Info,
            &format!(
                "{} construction candidate(s) read under {}: {} presented as `new`, {} refused",
                sites.records().len(),
                NEW.rule(),
                sites
                    .records()
                    .iter()
                    .filter(|record| record.presented())
                    .count(),
                sites
                    .records()
                    .iter()
                    .filter(|record| !record.presented())
                    .count(),
            ),
        ));
    }
    for record in fields.records() {
        if let Some(refusal) = &record.refusal {
            diagnostics.push(diagnostic(
                refusal.code,
                DiagnosticSeverity::Warning,
                &refusal.message,
            ));
        }
    }
    if !fields.records().is_empty() {
        diagnostics.push(diagnostic(
            "jre_field_accesses",
            DiagnosticSeverity::Info,
            &format!(
                "{} field instruction(s) read under {}: {} presented, {} refused",
                fields.records().len(),
                FIELD.rule(),
                fields
                    .records()
                    .iter()
                    .filter(|record| record.presented())
                    .count(),
                fields
                    .records()
                    .iter()
                    .filter(|record| !record.presented())
                    .count(),
            ),
        ));
    }
    for record in enums.records() {
        if let Some(refusal) = &record.refusal {
            diagnostics.push(diagnostic(
                refusal.code,
                DiagnosticSeverity::Warning,
                &refusal.message,
            ));
        }
    }
    if !enums.records().is_empty() {
        // The boundary is stated where the shape is: the read is the bytecode's own dispatch, and the
        // mapping from its entries to enum constants is a class-level fact of *another* class this
        // run never read (P3 2.3).
        diagnostics.push(diagnostic(
            "jre_enumswitch",
            DiagnosticSeverity::Info,
            &format!(
                "{} dispatch-table read(s) read under {}: {} presented as the table read the bytecode performs, {} refused; the constants those entries stand for are the enum class's own declaration, which this run does not hold, so no `case T.CONST:` label is written",
                enums.records().len(),
                ENUMSWITCH.rule(),
                enums
                    .records()
                    .iter()
                    .filter(|record| record.presented())
                    .count(),
                enums
                    .records()
                    .iter()
                    .filter(|record| !record.presented())
                    .count(),
            ),
        ));
    }
    if let Some(record) = prologues.record() {
        if let Some(refusal) = &record.refusal {
            diagnostics.push(diagnostic(
                refusal.code,
                DiagnosticSeverity::Warning,
                &refusal.message,
            ));
        } else {
            diagnostics.push(diagnostic(
                "jre_constructor_prologue",
                DiagnosticSeverity::Info,
                &format!(
                    "the body is an instance initializer and its prologue is written under {} as `{}`: the call at BCI {} names `{}`, and the class that declares this constructor is `{}`",
                    INIT.rule(),
                    record.target.map_or("?", |target| target.spell()),
                    record.bci.unwrap_or(0),
                    record.class.as_deref().unwrap_or("?"),
                    record.declared.as_deref().unwrap_or("?"),
                ),
            ));
        }
    }
    let declaration_record = declaration.record().clone();
    if let Some(refusal) = &declaration_record.refusal {
        diagnostics.push(diagnostic(
            refusal.code,
            DiagnosticSeverity::Warning,
            &refusal.message,
        ));
    } else {
        diagnostics.push(diagnostic(
            "jre_declaration",
            DiagnosticSeverity::Info,
            &format!(
                "the artifact's envelope states the member's declaration under {}: {}",
                DECLARATION.rule(),
                declaration_record.form.map_or("?", |form| form.spell())
            ),
        ));
    }
    RecoveryReport {
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
        execution: ExecutionReport::Complete {
            usage: budget.usage(),
        },
        outcome: RecoveryOutcome::Produced,
        text: emitted.text,
        source_map: emitted.source_map,
        regions,
        lambdas: program.lambdas,
        concats,
        accessors: program.accessors,
        bridges,
        news: sites.records().to_vec(),
        fields: fields.records().to_vec(),
        enum_switches: enums.records().to_vec(),
        init: prologues.record().cloned(),
        declaration: Some(declaration_record),
        fallbacks,
        aliased_names,
        diagnostics,
        method,
        rules,
    }
}

/// One region record per recovered region, with its fallbacks stated.
fn region_records(regions: &[Region]) -> Vec<RegionRecord> {
    regions
        .iter()
        .map(|region| {
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
        })
        .collect()
}

/// The report of a run that stopped: no text, no segments, and an execution plane that says so.
fn stopped(
    method: String,
    profile: RecoveryProfile,
    reason: StopReason,
    budget: &Budget,
) -> RecoveryReport {
    let usage: UsageSnapshot = budget.usage();
    let (execution, code, severity) = match &reason {
        StopReason::IrTableMissing { table: _ } => (
            ExecutionReport::Partial {
                reason: TerminationReason::Unsupported {
                    code: "jre_ir_table_missing".to_string(),
                },
                usage,
            },
            "jre_ir_table_missing",
            DiagnosticSeverity::Error,
        ),
        StopReason::Budget { dimension, .. } => (
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: BudgetDimension::from(*dimension),
                },
                usage,
            },
            "jre_output_budget",
            DiagnosticSeverity::Error,
        ),
        StopReason::Cancelled { .. } => (
            ExecutionReport::Cancelled { usage },
            "jre_cancelled",
            DiagnosticSeverity::Warning,
        ),
        StopReason::Interrupted { code, .. } => (
            ExecutionReport::Partial {
                reason: TerminationReason::Error {
                    code: (*code).to_string(),
                },
                usage,
            },
            *code,
            DiagnosticSeverity::Error,
        ),
    };
    let message = match &reason {
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
        StopReason::Interrupted { code, at } => format!(
            "the budget interrupted the run ({code}) at {}",
            at.map_or("no node".to_string(), |bci| format!("BCI {bci}"))
        ),
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
        execution,
        outcome: RecoveryOutcome::Stopped(reason),
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
        aliased_names: Vec::new(),
        diagnostics: vec![diagnostic(code, severity, &message)],
    }
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
