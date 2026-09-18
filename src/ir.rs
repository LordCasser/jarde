//! P2 method-analysis requests and reports, with the phase and product state of one method.
//!
//! The schema is fixed by the design's public skeleton section. Two things are kept apart
//! on purpose: the requested phase set (what the caller asked for, normalized to the fixed
//! phase order and deduplicated) and the scheduled phase list (which also carries the
//! prerequisites of the requested phases), plus the product planes `representation`,
//! `quality`, `syntax_status`, `compile_status`, `semantic_validation` and `verification`.
//! No plane is derived from another: a stage this build does not implement does not make the
//! bytecode representation incomplete, and a complete bytecode range does not make an IR
//! stage performed.
//!
//! This module owns the request shape ([`validate_request`]) and the report assembly
//! ([`analysis_report`], from an [`AnalysisRun`]); the code that drives the stages themselves
//! is the engine's, because it needs the reader, the providers and the raw CFG builder, which
//! this module does not depend on. 3.3 is the first slice whose runs are real: the engine
//! locates and decodes the driver method's body and builds its raw CFG, and the report states
//! which stages completed, which stopped and what was read on the way.

use crate::artifact::ArtifactSnapshot;
use crate::budget::{Budget, UsageSnapshot};
use crate::classfile::VerificationStatus;
use crate::environment::{
    EnvironmentIdentity, EnvironmentProblem, ResolutionEnvironment, environment_diagnostics,
    require_content_snapshot, unavailable_diagnostic,
};
use crate::error::{Error, Result};
use crate::model::{
    Coverage, Diagnostic, DiagnosticSeverity, ExecutionReport, OriginSet, PhysicalMethodId,
    TerminationReason,
};
use crate::resolver::HeaderRead;
use crate::view::LoaderId;
use serde::{Deserialize, Serialize};

/// Capability code of a method-analysis request that may not run at all.
///
/// Since 3.3 the entry point really runs the phases this build implements, so this code no
/// longer describes the entry point itself: it is what a request gets when the environment
/// validator rejected its environment, where no phase and no read may start (invariants 1
/// and 2). A request whose environment is legal but whose pipeline stops reports the stop
/// instead, and a phase this build does not implement (`ir_pass_not_implemented`) is that
/// phase's own failure.
pub(crate) const METHOD_ANALYSIS_NOT_IMPLEMENTED: &str = "method_analysis_not_implemented";

/// Fixed phase order of the P2 IR pipeline; declaration order is the phase order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisStage {
    RawFacts,
    RawCfg,
    LegacyNormalization,
    CanonicalCfg,
    Frame,
    Ssa,
}

impl AnalysisStage {
    /// Every phase in the fixed order.
    ///
    /// Ordering helpers iterate this list, so a new phase is added in one place, in phase
    /// order. The schedule of a request — the requested phases plus their prerequisites — is
    /// the prefix rule of the crate-private pass table (`crate::passes::PASSES`), which is
    /// declared in this same order and is what the engine runs.
    pub const ALL: [Self; 6] = [
        Self::RawFacts,
        Self::RawCfg,
        Self::LegacyNormalization,
        Self::CanonicalCfg,
        Self::Frame,
        Self::Ssa,
    ];
}

/// State of one phase of one request.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StageState {
    NotRequested,
    NotPerformed,
    Completed,
    Partial,
    Failed { code: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageResult {
    pub stage: AnalysisStage,
    pub state: StageState,
}

/// Representation a report describes. P2 produces bytecode only; Java/Mixed is P3.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Representation {
    Bytecode,
}

/// Strength of what was produced.
///
/// `Conservative` keeps low-level structures that are still faithful; `Fallback`
/// publishes the bytecode baseline with its failure reasons. The classification rule is
/// only fixed once the normalization and product slices (3.5/5.1) land, so this is a
/// property of a *produced* artifact and nothing else:
///
/// - when `analysis = NotPerformed` there is no artifact at all, so `Fallback` merely
///   means "not `Conservative`" and MUST NOT be read as evidence that a real fallback
///   recovery happened, or that a body was read, decoded or degraded;
/// - neither variant says anything about coverage, termination or verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quality {
    Conservative,
    Fallback,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyntaxStatus {
    NotJava,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompileStatus {
    NotAttempted,
}

/// Strongest semantic evidence that applies to this report.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticValidation {
    LocalInvariants,
    FixtureDifferential,
    Unproven,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoBodyKind {
    Abstract,
    Native,
}

/// Whether the method body was located and read, and why not when it has none.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MethodBodyState {
    /// No body was located or read, so the report states no body fact at all.
    ///
    /// This is also the state when a budget stop, cancellation or an unrequested body
    /// phase ends the request before the `Code` attribute was read: it does NOT mean that
    /// the method has no body, and it is not a `DeclaredWithoutBody` finding.
    NotInspected,
    Present,
    /// The payload field name is the line-format name: the internal tag and the payload
    /// field must not both be `kind` (invariant 7).
    DeclaredWithoutBody {
        no_body_kind: NoBodyKind,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodAnalysisRequest {
    pub environment: ResolutionEnvironment,
    pub method: PhysicalMethodId,
    /// Requested set; the engine normalizes it to the fixed phase order, deduplicates it
    /// and expands the prerequisites of the requested phases.
    pub stages: Vec<AnalysisStage>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodAnalysisReport {
    pub environment_identity: EnvironmentIdentity,
    pub environment_problems: Vec<EnvironmentProblem>,
    pub method: PhysicalMethodId,
    pub loader: LoaderId,
    /// Body fact of this report. Nothing read is `NotInspected`, never `Present`; see
    /// [`MethodBodyState`].
    pub body: MethodBodyState,
    pub representation: Representation,
    /// Quality of the produced artifact. When `analysis = NotPerformed` no artifact was
    /// produced, so `Fallback` here only means "not `Conservative`" and never that a
    /// fallback recovery really happened; see [`Quality`].
    pub quality: Quality,
    pub syntax_status: SyntaxStatus,
    pub compile_status: CompileStatus,
    pub semantic_validation: SemanticValidation,
    pub verification: VerificationStatus,
    /// Requested phases after normalization, in phase order.
    pub requested_stages: Vec<AnalysisStage>,
    /// Phases this request scheduled, including the prerequisites of the requested ones,
    /// in phase order.
    pub stages: Vec<StageResult>,
    pub origin: OriginSet,
    /// Every class header this request read, in read order, at most once per
    /// `(definition, loader)`; empty when the request read nothing.
    ///
    /// A method-analysis request reads the driver method's own class definition by identity,
    /// recorded under [`crate::resolver::ReadReason::DriverMethodBody`] — the one reason that
    /// may name a body read, and the one that lets `reads` differ from the header-only
    /// closure. A refused charge and a rejected environment record nothing, so an empty list
    /// is also what a run that never reached its first read reports.
    pub reads: Vec<HeaderRead>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

impl MethodAnalysisRequest {
    /// Requested stages in fixed phase order, without duplicates.
    pub(crate) fn normalized_stages(&self) -> Vec<AnalysisStage> {
        AnalysisStage::ALL
            .into_iter()
            .filter(|stage| self.stages.contains(stage))
            .collect()
    }
}

/// Request-level checks of a method-analysis request: shape only, no artifact access.
pub(crate) fn validate_request(
    content: &[ArtifactSnapshot],
    request: &MethodAnalysisRequest,
) -> Result<()> {
    require_content_snapshot(content, &request.environment.runtime.physical.snapshot)?;
    if request.stages.is_empty() {
        return Err(Error::invalid_input(
            "analysis_no_stages",
            "a method-analysis request must ask for at least one stage",
        ));
    }
    Ok(())
}

/// One method-analysis run: what the scheduled passes reached, and what they read and
/// produced on the way.
///
/// The passes themselves are driven by the engine (they need the reader, the providers and the
/// raw CFG builder); this is the outcome a report is assembled from, so the report layer keeps
/// owning the schema and the engine keeps owning the pipeline. Nothing here is derived from
/// another field: a run that read no body states `NotInspected`, a run that covered a prefix
/// states `Partial`, and a run that stopped states why.
pub(crate) struct AnalysisRun {
    pub(crate) body: MethodBodyState,
    pub(crate) stages: Vec<StageResult>,
    pub(crate) reads: Vec<HeaderRead>,
    pub(crate) coverage: Coverage,
    pub(crate) execution: ExecutionReport,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl AnalysisRun {
    /// A run that performs nothing: every scheduled stage stays `NotPerformed`, the body was
    /// never located, and the capability is reported as unavailable under `code`.
    ///
    /// This is the honest state of a request whose environment the validator rejected
    /// (invariant 2: a rejected environment never yields a result and never starts a read).
    pub(crate) fn not_performed(scheduled: &[AnalysisStage], code: &str, budget: &Budget) -> Self {
        Self {
            body: MethodBodyState::NotInspected,
            stages: scheduled
                .iter()
                .map(|stage| StageResult {
                    stage: *stage,
                    state: StageState::NotPerformed,
                })
                .collect(),
            reads: Vec::new(),
            coverage: Coverage::not_requested(),
            execution: ExecutionReport::Failed {
                reason: TerminationReason::Unsupported {
                    code: code.to_string(),
                },
                usage: budget.usage(),
            },
            diagnostics: vec![unavailable_diagnostic(code, "method IR analysis")],
        }
    }
}

/// Terminal mapping of a refusal, under the same contract the physical reports use: a budget
/// stop is a partial execution that names its dimension, a cancellation is a cancellation, and
/// a structural failure keeps the reader's own code.
pub(crate) fn terminal(error: &Error, usage: UsageSnapshot) -> (ExecutionReport, Diagnostic) {
    let diagnostic = |code: String, severity: DiagnosticSeverity| Diagnostic {
        code,
        severity,
        message: error.to_string(),
        provenance: None,
    };
    match error {
        Error::Cancelled { .. } => (
            ExecutionReport::Cancelled { usage },
            diagnostic("cancelled".to_string(), DiagnosticSeverity::Warning),
        ),
        Error::BudgetExceeded { dimension, .. } => (
            ExecutionReport::Partial {
                reason: TerminationReason::BudgetExceeded {
                    dimension: *dimension,
                },
                usage,
            },
            diagnostic(
                format!(
                    "budget_exceeded_{}",
                    crate::artifact::budget_dimension_code(*dimension)
                ),
                DiagnosticSeverity::Warning,
            ),
        ),
        Error::InvalidInput { code, .. }
        | Error::Io {
            operation: code, ..
        } => (
            ExecutionReport::Failed {
                reason: TerminationReason::Error { code: code.clone() },
                usage,
            },
            diagnostic(code.clone(), DiagnosticSeverity::Error),
        ),
        Error::Unsupported { code, .. } => (
            ExecutionReport::Failed {
                reason: TerminationReason::Unsupported { code: code.clone() },
                usage,
            },
            diagnostic(code.clone(), DiagnosticSeverity::Error),
        ),
    }
}

/// Report of one method-analysis request: the environment facts, the run's own planes and the
/// product planes of this slice.
///
/// The environment problems and their diagnostics come first, then the run's diagnostics, which
/// is the order 1.1 fixed for a rejected environment and keeps every problem visible next to
/// the capability it prevented. The product planes are the P2 baseline (`Bytecode`, `NotJava`,
/// `NotAttempted`, `Unproven`, `NotPerformed`): this slice builds no Java, compiles nothing and
/// proves no semantic invariant, and `quality = Fallback` means "not `Conservative`" until 3.5
/// fixes the classification of a produced artifact. `origin` stays empty because the IR
/// payloads are crate-private in P2 (invariant 11): the report anchors the request by
/// `method`, and 5.1 is where a published IR count would have to add its own field first.
pub(crate) fn analysis_report(
    request: &MethodAnalysisRequest,
    problems: Vec<EnvironmentProblem>,
    environment_identity: EnvironmentIdentity,
    run: AnalysisRun,
) -> MethodAnalysisReport {
    let mut diagnostics = environment_diagnostics(&problems);
    diagnostics.extend(run.diagnostics);
    MethodAnalysisReport {
        environment_identity,
        environment_problems: problems,
        method: request.method.clone(),
        // The request binds one caller domain; the defining loader of the method is a
        // resolution result and is therefore not claimed here.
        loader: request.environment.runtime.load_domain.loader.clone(),
        body: run.body,
        representation: Representation::Bytecode,
        quality: Quality::Fallback,
        syntax_status: SyntaxStatus::NotJava,
        compile_status: CompileStatus::NotAttempted,
        semantic_validation: SemanticValidation::Unproven,
        verification: VerificationStatus::NotPerformed,
        requested_stages: request.normalized_stages(),
        stages: run.stages,
        origin: OriginSet::default(),
        reads: run.reads,
        coverage: run.coverage,
        execution: run.execution,
        diagnostics,
    }
}
