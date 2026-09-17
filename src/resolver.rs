//! P2 resolution requests and reports (demand-resolver entry).
//!
//! The schema is fixed by the design's public skeleton section: request shape, resolution
//! state, dispatch report and the declaration-reference query/report. Symbols stay raw
//! bytes; nothing here normalizes an owner, name or descriptor.
//!
//! This slice delivers the schema and the honest unavailable state: [`validate_request`]
//! checks the request shape, [`resolution_report`] and [`declaration_reference_report`]
//! report that nothing was performed and that no byte was read. The resolver itself is
//! implemented by the following slices.
//!
//! The module borrows vocabulary from `query` ([`ConsumerKind`], [`ConsumerSchema`],
//! [`XrefOperation`]) and never the other way round: `query` and `xref` must not know that
//! this module exists (A17).

use crate::artifact::ArtifactSnapshot;
use crate::budget::Budget;
use crate::environment::{
    CallerContext, EnvironmentIdentity, EnvironmentProblem, ResolutionEnvironment,
    environment_diagnostics, require_content_snapshot, unavailable_diagnostic,
    validate_environment,
};
use crate::error::{Error, Result};
use crate::model::{
    Coverage, Diagnostic, ExecutionReport, OriginSet, PhysicalDefinitionId, SymbolRef,
    TerminationReason,
};
use crate::query::{ConsumerKind, ConsumerSchema, XrefOperation};
use crate::view::{LoaderId, PhysicalScope};
use serde::{Deserialize, Serialize};

/// Capability name of the resolution entry points while they are not implemented.
pub(crate) const RESOLUTION_NOT_IMPLEMENTED: &str = "resolution_not_implemented";

/// Access or invocation kind of a reference: the input of the resolution rules.
///
/// This is not the `query` operation vocabulary: `XrefOperation` reports what a consumer
/// found in an artifact, while this enum states what the caller is asking about. The two
/// sets differ (for example a field read and a field write share `GetField`).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceUse {
    ClassReference,
    FieldRead,
    FieldWrite,
    InvokeStatic,
    InvokeSpecial,
    InvokeVirtual,
    InvokeInterface,
    InvokeDynamic,
}

impl ReferenceUse {
    /// Whether a symbol of this kind can be the target of this use.
    ///
    /// A use of one kind that names a symbol of another kind is a request mismatch and is
    /// rejected before anything is read.
    pub fn matches_symbol(&self, symbol: &SymbolRef) -> bool {
        match self {
            Self::ClassReference => matches!(symbol, SymbolRef::Class { .. }),
            Self::FieldRead | Self::FieldWrite => matches!(symbol, SymbolRef::Field { .. }),
            Self::InvokeStatic
            | Self::InvokeSpecial
            | Self::InvokeVirtual
            | Self::InvokeInterface
            | Self::InvokeDynamic => matches!(symbol, SymbolRef::Method { .. }),
        }
    }
}

/// Explicit physical range of a known-candidate dispatch request.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchScope {
    pub scope: PhysicalScope,
    pub consumers: ConsumerSchema,
}

/// One demand-bound symbol resolution request.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionRequest {
    pub environment: ResolutionEnvironment,
    /// Raw symbol bytes as they appear in the artifact; never normalized.
    pub target: SymbolRef,
    pub use_kind: ReferenceUse,
    pub caller: CallerContext,
    /// When present, the request also asks for `KnownCandidates` inside this range.
    pub dispatch: Option<DispatchScope>,
}

/// Whether the capability ran at all. Semantic outcome lives in [`ResolutionState`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionAnalysis {
    Performed,
    NotPerformed,
}

/// Semantic decision of a performed resolution.
///
/// The set deliberately has no `NotPerformed` and no `Cancelled`: a capability that did
/// not run is `analysis = NotPerformed` with `state = None`, and cancellation is
/// `execution = Cancelled` with `state = None`. A budget stop is `BudgetExceeded` here and
/// in `execution` at the same time.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionState {
    Resolved,
    Missing,
    Ambiguous,
    Inaccessible,
    IncompatibleClassChange,
    UnsupportedPolicy,
    BudgetExceeded,
}

/// One resolved member: loader, physical definition and the raw member symbol.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedMemberRef {
    pub loader: LoaderId,
    pub definition: PhysicalDefinitionId,
    /// Owner is the internal name of the declaring class (which may differ from the
    /// owner of the reference); name and descriptor are raw bytes.
    pub member: SymbolRef,
}

/// Why a candidate cannot be proven to be the only runtime target.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OpenWorldEvidence {
    ExternalSubclass,
    UnknownLoader,
    RuntimeTransformation,
    MissingDependency,
    OrderedRoot { index: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchCandidate {
    pub member: ResolvedMemberRef,
    pub evidence: OpenWorldEvidence,
}

/// Known candidates of an explicitly scoped dispatch request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchReport {
    pub scope: PhysicalScope,
    pub candidates: Vec<DispatchCandidate>,
    pub open_world: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionReport {
    pub environment_identity: EnvironmentIdentity,
    pub environment_problems: Vec<EnvironmentProblem>,
    pub target: SymbolRef,
    pub use_kind: ReferenceUse,
    pub caller: CallerContext,
    pub analysis: ResolutionAnalysis,
    /// `None` exactly when `analysis == NotPerformed`.
    pub state: Option<ResolutionState>,
    pub resolved: Option<ResolvedMemberRef>,
    /// Only definitions that are indistinguishable at one selection position.
    pub candidates: Vec<ResolvedMemberRef>,
    pub dispatch: Option<DispatchReport>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    /// Includes one diagnostic per environment problem, under the same code.
    pub diagnostics: Vec<Diagnostic>,
}

/// Declaration-reference scan request; implemented by the declaration-query slice.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclarationRefQuery {
    pub environment: ResolutionEnvironment,
    pub declaration: ResolvedMemberRef,
    pub scope: PhysicalScope,
    pub consumers: ConsumerSchema,
    /// `0` means "no item limit"; a P1 cursor is never used as a runtime-view cursor.
    pub max_items: u64,
}

/// One use site of a declaration-reference scan. Every item carries a resolution attempt.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclarationRefItem {
    /// Raw symbol bytes found at the use site, before any inheritance resolution.
    pub referenced: SymbolRef,
    pub consumer: ConsumerKind,
    pub operation: XrefOperation,
    /// Physical use site.
    pub origin: OriginSet,
    pub resolved: Option<ResolvedMemberRef>,
    pub state: ResolutionState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclarationRefReport {
    pub environment_identity: EnvironmentIdentity,
    pub environment_problems: Vec<EnvironmentProblem>,
    pub declaration: ResolvedMemberRef,
    pub scope: PhysicalScope,
    pub consumers: ConsumerSchema,
    pub unsupported_categories: Vec<ConsumerKind>,
    pub analysis: ResolutionAnalysis,
    pub items: Vec<DeclarationRefItem>,
    /// Candidates that a missing dependency or a budget stop left undecided; they are
    /// never presented as excluded.
    pub unresolved_candidates: u64,
    pub has_more: bool,
    pub returned_items: u64,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<Diagnostic>,
}

/// Request-level checks of a resolution request: shape only, no artifact access.
pub(crate) fn validate_request(
    content: &[ArtifactSnapshot],
    request: &ResolutionRequest,
) -> Result<()> {
    require_content_snapshot(content, &request.environment.runtime.physical.snapshot)?;
    if !request.use_kind.matches_symbol(&request.target) {
        return Err(Error::invalid_input(
            "resolution_target_use_mismatch",
            format!(
                "`{:?}` cannot reference a `{}` symbol; the target and the reference use \
                 kind must describe the same member kind",
                request.use_kind,
                symbol_kind(&request.target)
            ),
        ));
    }
    Ok(())
}

/// Request-level checks of a declaration-reference query: shape only, no artifact access.
pub(crate) fn validate_declaration_reference_query(
    content: &[ArtifactSnapshot],
    query: &DeclarationRefQuery,
) -> Result<()> {
    require_content_snapshot(content, &query.environment.runtime.physical.snapshot)
}

/// Honest result of a legally shaped resolution request in this slice.
///
/// Environment problems are part of the report; they are never an `Err` and never a
/// fallback search order. Nothing was performed, so `state`, `resolved`, `candidates` and
/// `dispatch` stay empty, coverage stays `NotRequested` and the counted usage stays zero.
pub(crate) fn resolution_report(
    content: &[ArtifactSnapshot],
    request: &ResolutionRequest,
    budget: &Budget,
) -> ResolutionReport {
    let (problems, environment_identity) = validate_environment(content, &request.environment);
    let mut diagnostics = environment_diagnostics(&problems);
    diagnostics.push(unavailable_diagnostic(
        RESOLUTION_NOT_IMPLEMENTED,
        "demand-bound symbol resolution",
    ));
    ResolutionReport {
        environment_identity,
        environment_problems: problems,
        target: request.target.clone(),
        use_kind: request.use_kind,
        caller: request.caller.clone(),
        analysis: ResolutionAnalysis::NotPerformed,
        state: None,
        resolved: None,
        candidates: Vec::new(),
        // A dispatch report would claim a performed candidate enumeration, so an
        // unimplemented request reports no dispatch at all.
        dispatch: None,
        coverage: Coverage::not_requested(),
        execution: ExecutionReport::Failed {
            reason: TerminationReason::Unsupported {
                code: RESOLUTION_NOT_IMPLEMENTED.to_string(),
            },
            usage: budget.usage(),
        },
        diagnostics,
    }
}

/// Honest result of a legally shaped declaration-reference query in this slice.
pub(crate) fn declaration_reference_report(
    content: &[ArtifactSnapshot],
    query: &DeclarationRefQuery,
    budget: &Budget,
) -> DeclarationRefReport {
    let (problems, environment_identity) = validate_environment(content, &query.environment);
    let mut diagnostics = environment_diagnostics(&problems);
    diagnostics.push(unavailable_diagnostic(
        RESOLUTION_NOT_IMPLEMENTED,
        "declaration-reference resolution",
    ));
    DeclarationRefReport {
        environment_identity,
        environment_problems: problems,
        declaration: query.declaration.clone(),
        scope: query.scope.clone(),
        consumers: query.consumers.clone(),
        // Nothing was scanned, so no category can be called unsupported.
        unsupported_categories: Vec::new(),
        analysis: ResolutionAnalysis::NotPerformed,
        items: Vec::new(),
        unresolved_candidates: 0,
        has_more: false,
        returned_items: 0,
        coverage: Coverage::not_requested(),
        execution: ExecutionReport::Failed {
            reason: TerminationReason::Unsupported {
                code: RESOLUTION_NOT_IMPLEMENTED.to_string(),
            },
            usage: budget.usage(),
        },
        diagnostics,
    }
}

/// Symbol kind of one reference, for messages that must not invent an owner.
fn symbol_kind(symbol: &SymbolRef) -> &'static str {
    match symbol {
        SymbolRef::Class { .. } => "class",
        SymbolRef::Field { .. } => "field",
        SymbolRef::Method { .. } => "method",
    }
}
