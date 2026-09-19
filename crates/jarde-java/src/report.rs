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

use crate::build;
use crate::emit::{Emitted, emit};
use crate::facts::RecoveryFacts;
use crate::names::NameTable;
use crate::normal_flow::NormalFlowView;
use crate::region::{FallbackReason, Recovered, Region};
use crate::source_map::SourceMap;
use crate::stop::StopReason;

/// One recovery request: the payload of a P2 run and the facts that run did not publish.
#[derive(Clone, Copy, Debug)]
pub struct RecoveryRequest<'a> {
    /// The IR payload of the method-analysis run whose body is being presented.
    pub ir: &'a MethodIr,
    /// The facts the layer below read: the method's identity, its debug names, and the decoded
    /// operations of its body.
    pub facts: &'a RecoveryFacts,
}

impl<'a> RecoveryRequest<'a> {
    /// One request over one payload and one fact set.
    pub fn new(ir: &'a MethodIr, facts: &'a RecoveryFacts) -> Self {
        Self { ir, facts }
    }
}

/// Whether the run produced an artifact or stopped before it had one.
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
}

/// The result of one recovery run: one artifact, or the reason there is none.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    /// The method this report is about, as its facts state it.
    pub method: String,
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
    let Some(canonical) = request.ir.canonical() else {
        return stopped(
            method,
            StopReason::IrTableMissing { table: "canonical" },
            budget,
        );
    };
    let Some(frames) = request.ir.frames() else {
        return stopped(
            method,
            StopReason::IrTableMissing { table: "frames" },
            budget,
        );
    };
    let Some(ssa) = request.ir.ssa() else {
        return stopped(method, StopReason::IrTableMissing { table: "ssa" }, budget);
    };
    if canonical.blocks().is_empty() {
        return stopped(
            method,
            StopReason::IrTableMissing {
                table: "canonical blocks",
            },
            budget,
        );
    }
    let view = match NormalFlowView::build(canonical, budget) {
        Ok(view) => view,
        Err(stop) => return stopped(method, stop, budget),
    };
    let recovered: Recovered =
        match crate::region::recover(canonical, &view, ssa, request.facts, budget) {
            Ok(recovered) => recovered,
            Err(stop) => return stopped(method, stop, budget),
        };
    // The slots the names are decided for are the body's own local slots: the frames table states
    // how many there are, and a local the debug metadata never named still needs a name.
    let slots = u16::try_from(frames.locals_slots()).unwrap_or(u16::MAX);
    let names = NameTable::build(
        request.facts.method().parameters(),
        slots,
        request.facts.debug_locals(),
    );
    let program = match build::build(
        canonical,
        ssa,
        request.facts,
        &names,
        &recovered.regions,
        budget,
    ) {
        Ok(program) => program,
        Err(stop) => return stopped(method, stop, budget),
    };
    let emitted: Emitted = match emit(&program.stmts, request.facts, budget) {
        Ok(emitted) => emitted,
        Err(stop) => return stopped(method, stop, budget),
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
    RecoveryReport {
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
        fallbacks,
        aliased_names,
        diagnostics,
        method,
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
            }
        })
        .collect()
}

/// The report of a run that stopped: no text, no segments, and an execution plane that says so.
fn stopped(method: String, reason: StopReason, budget: &Budget) -> RecoveryReport {
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
