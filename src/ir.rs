//! P2 method-analysis requests and reports, with the phase and product state of one method.
//!
//! The schema is fixed by the design's public skeleton section. Two things are kept apart
//! on purpose: the requested phase set (what the caller asked for, normalized to the fixed
//! phase order and deduplicated) and the scheduled phase list (which also carries the
//! prerequisites of the requested phases), plus the product planes `representation`,
//! `quality`, `syntax_status`, `compile_status`, `semantic_validation` and `verification`.
//! No plane is derived from another: an unimplemented stage does not make the bytecode
//! representation incomplete, and a complete bytecode range does not make an IR stage
//! performed.
//!
//! This slice delivers the schema and the honest unavailable state: [`validate_request`]
//! checks the request shape, [`analysis_report`] reports that no phase ran, that the body
//! was never located ([`MethodBodyState::NotInspected`]) and that no byte was read. The IR
//! itself is implemented by the following slices.

use crate::artifact::ArtifactSnapshot;
use crate::budget::Budget;
use crate::classfile::VerificationStatus;
use crate::environment::{
    EnvironmentIdentity, EnvironmentProblem, ResolutionEnvironment, environment_diagnostics,
    require_content_snapshot, unavailable_diagnostic, validate_environment,
};
use crate::error::{Error, Result};
use crate::model::{
    Coverage, Diagnostic, ExecutionReport, OriginSet, PhysicalMethodId, TerminationReason,
};
use crate::resolver::HeaderRead;
use crate::view::LoaderId;
use serde::{Deserialize, Serialize};

/// Capability name of the method-analysis entry point while it is not implemented.
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
    /// Ordering helpers (`normalize` on a request, prerequisite expansion) iterate this
    /// list, so a new phase is added in one place, in phase order.
    pub const ALL: [Self; 6] = [
        Self::RawFacts,
        Self::RawCfg,
        Self::LegacyNormalization,
        Self::CanonicalCfg,
        Self::Frame,
        Self::Ssa,
    ];

    /// Position in the fixed phase order; also the number of prerequisites.
    fn position(self) -> usize {
        Self::ALL
            .iter()
            .position(|stage| *stage == self)
            .unwrap_or(Self::ALL.len())
    }
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
    /// This slice has no body path at all, so `reads` is empty here. A body is upgraded only
    /// by an explicit request for the target method's body, which the analysis slices record
    /// in this same list under
    /// [`crate::resolver::ReadReason::DriverMethodBody`] — the one reason that may name a
    /// body read, and the one that lets `reads` differ from the header-only closure.
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

    /// Requested stages plus the prerequisites of each requested phase.
    ///
    /// The phases form one fixed order, so requesting a phase also requires every earlier
    /// phase; the scheduled list is the prefix of the phase order up to the last requested
    /// phase.
    fn scheduled_stages(&self) -> Vec<AnalysisStage> {
        let Some(last) = self
            .normalized_stages()
            .into_iter()
            .map(AnalysisStage::position)
            .max()
        else {
            return Vec::new();
        };
        AnalysisStage::ALL.into_iter().take(last + 1).collect()
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

/// Honest result of a legally shaped method-analysis request in this slice.
///
/// The scheduled phases are listed as `NotPerformed` because that is what happened: the
/// request was validated and no phase ran. The product planes keep the P2 baseline
/// (`Bytecode`, `NotJava`, `NotAttempted`, `NotPerformed`); `quality = Fallback` only means
/// "not `Conservative`" here, because no artifact was produced to classify, and
/// `semantic_validation` stays `Unproven` for the same reason. Nothing was located or read,
/// so the body is `NotInspected` — not a `Present`/`DeclaredWithoutBody` claim. Coverage
/// stays `NotRequested` and the counted usage stays zero, so an unimplemented request
/// cannot be read as a partial analysis.
pub(crate) fn analysis_report(
    content: &[ArtifactSnapshot],
    request: &MethodAnalysisRequest,
    budget: &Budget,
) -> MethodAnalysisReport {
    let (problems, environment_identity) = validate_environment(content, &request.environment);
    let mut diagnostics = environment_diagnostics(&problems);
    diagnostics.push(unavailable_diagnostic(
        METHOD_ANALYSIS_NOT_IMPLEMENTED,
        "method IR analysis",
    ));
    let requested_stages = request.normalized_stages();
    let stages = request
        .scheduled_stages()
        .into_iter()
        .map(|stage| StageResult {
            stage,
            state: StageState::NotPerformed,
        })
        .collect();
    MethodAnalysisReport {
        environment_identity,
        environment_problems: problems,
        method: request.method.clone(),
        // The request binds one caller domain; the defining loader of the method is a
        // resolution result and is therefore not claimed here.
        loader: request.environment.runtime.load_domain.loader.clone(),
        // No class byte was read, so nothing was located: the body stays `NotInspected`.
        // Claiming `Present` or `DeclaredWithoutBody` here would state a body fact this
        // slice never observed, and the same state is what a budget stop or a canceled
        // request reports before the `Code` attribute is read.
        body: MethodBodyState::NotInspected,
        representation: Representation::Bytecode,
        quality: Quality::Fallback,
        syntax_status: SyntaxStatus::NotJava,
        compile_status: CompileStatus::NotAttempted,
        semantic_validation: SemanticValidation::Unproven,
        verification: VerificationStatus::NotPerformed,
        requested_stages,
        stages,
        // No IR artifact was generated.
        origin: OriginSet::default(),
        // No header was demanded: this slice performs no closure work at all, and the only
        // reason that may upgrade to a body read (`DriverMethodBody`) belongs to 3.x.
        reads: Vec::new(),
        coverage: Coverage::not_requested(),
        execution: ExecutionReport::Failed {
            reason: TerminationReason::Unsupported {
                code: METHOD_ANALYSIS_NOT_IMPLEMENTED.to_string(),
            },
            usage: budget.usage(),
        },
        diagnostics,
    }
}
