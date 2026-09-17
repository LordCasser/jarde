//! P2 resolution requests and reports (demand-resolver entry).
//!
//! The schema is fixed by the design's public skeleton section: request shape, resolution
//! state, dispatch report and the declaration-reference query/report. Symbols stay raw
//! bytes; nothing here normalizes an owner, name or descriptor.
//!
//! This module owns the request/report shape and maps a performed lookup into it.
//! [`validate_request`] checks the request shape, [`resolution_report`] answers one request:
//! a class symbol in an environment the validator accepted is looked up for real by the
//! 2.2 closure over [`crate::providers`], every header that lookup really read is published as
//! a [`HeaderRead`], every other request keeps the honest unavailable state of the schema
//! slice (`NotPerformed` / `state = None` / `Failed { Unsupported }`), and no report ever
//! claims a result the closure did not produce.
//!
//! Two capability codes still name what this engine slice does not do: a member symbol reports
//! `resolution_not_implemented` (2.3 implements it) and a request that also asks for dispatch
//! candidates reports `dispatch_not_implemented` as a warning (2.5 implements it). Both disappear
//! with the capability they name.
//!
//! The module borrows vocabulary from `query` ([`ConsumerKind`], [`ConsumerSchema`],
//! [`XrefOperation`]) and never the other way round: `query` and `xref` must not know that
//! this module exists (A17).

use crate::artifact::{ArtifactSnapshot, budget_dimension_code};
use crate::budget::{Budget, UsageSnapshot};
use crate::environment::{
    CallerContext, EnvironmentIdentity, EnvironmentProblem, ResolutionEnvironment,
    environment_diagnostics, require_content_snapshot, unavailable_diagnostic,
    validate_environment, validate_environment_with_caller,
};
use crate::error::{Error, Result};
use crate::model::{
    Coverage, CoverageDimension, CoverageRange, CoverageState, Diagnostic, DiagnosticSeverity,
    ExecutionReport, JvmBytes, OriginSet, PhysicalDefinitionId, SymbolRef, TerminationReason,
};
use crate::providers::{HeaderClosure, HeaderDemand, HeaderLookupState};
use crate::query::{ConsumerKind, ConsumerSchema, XrefOperation};
use crate::view::{LoaderId, PhysicalScope};
use serde::{Deserialize, Serialize};

/// Capability name of the resolution entry points while they are not implemented.
pub(crate) const RESOLUTION_NOT_IMPLEMENTED: &str = "resolution_not_implemented";

/// Capability name of the dispatch-candidate enumeration this slice does not perform.
///
/// Reported as a warning next to a performed declaration lookup, and replaced by the real
/// `DispatchReport` in 2.5.
const DISPATCH_NOT_IMPLEMENTED: &str = "dispatch_not_implemented";

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

/// Why one class header was read by the closure that served a request.
///
/// The reason names the demand, not the outcome: a header read while expanding a hierarchy is
/// recorded as `HierarchyClosure` even when the name it was demanded for turned out to
/// resolve to that definition, and a read that happened before the search stopped keeps the
/// demand that caused it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadReason {
    /// The request target itself.
    RequestedDefinition,
    /// One step up a `super_class` chain.
    ParentChain,
    /// One class of a superclass/interface closure expansion.
    HierarchyClosure,
    /// One class of an explicitly scoped candidate enumeration (2.5).
    DispatchScope,
    /// The owner a member resolution landed on (2.3).
    MemberOwner,
    /// The target method's body — the only reason that upgrades to a body read (3.x).
    DriverMethodBody,
}

/// One class header a request really read.
///
/// A record exists only for a read that produced a definition identity: a refused charge, a
/// damaged candidate and a read-layer failure produce no record, and their evidence is the
/// diagnostic that names them. The same `(definition, loader)` binding is recorded once per
/// request — reusing an already read header for another reason is not a second read — so
/// `reads.len() <= usage.class_headers` holds for every report.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeaderRead {
    pub loader: LoaderId,
    pub definition: PhysicalDefinitionId,
    pub reason: ReadReason,
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
    /// Every class header this request read, in read order, at most once per
    /// `(definition, loader)`; empty when the request performed nothing.
    pub reads: Vec<HeaderRead>,
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
    /// Every class header this scan read, in read order, at most once per
    /// `(definition, loader)`; empty when the scan performed nothing.
    pub reads: Vec<HeaderRead>,
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

/// Result of one legally shaped resolution request.
///
/// A class symbol in an environment whose declarations are usable is looked up by
/// [`crate::providers`] and the lookup is mapped into the schema's own planes. The mapping rule
/// is invariant 3: `state = Some(v)` exactly when the run reached a semantic decision, and
/// `state = None` covers both a capability that never ran (`analysis = NotPerformed`) and a run
/// that stopped before deciding (`analysis = Performed` with `execution` `Cancelled`/`Failed`/
/// `Partial`).
///
/// Concretely: `Resolved`, `Missing` and `Ambiguous` are the lookup's three decisions, and a
/// budget stop is the fourth (`BudgetExceeded`, with the same stop in `execution`). A
/// cancellation is `execution = Cancelled` with `state = None`; a damaged candidate or a
/// stopped listing is `execution = Failed { Error { code } }` with `state = None` and a
/// diagnostic that names the origin. `Inaccessible` and `IncompatibleClassChange` are reserved
/// for the access and link rules of 2.3/2.5 — a read failure never occupies a semantic state.
///
/// Everything else keeps the honest unavailable state: a member symbol is resolved by 2.3, and
/// an environment the validator rejected never yields a unique definition (invariant 2), so
/// neither one starts a search. Environment problems are part of the report; they are never an
/// `Err` and never a fallback search order.
pub(crate) fn resolution_report(
    content: &[ArtifactSnapshot],
    request: &ResolutionRequest,
    budget: &mut Budget,
) -> ResolutionReport {
    let (problems, environment_identity) =
        validate_environment_with_caller(content, &request.environment, &request.caller);
    let mut diagnostics = environment_diagnostics(&problems);
    let Some(target) = performable_class(&request.target, &problems) else {
        diagnostics.push(unavailable_diagnostic(
            RESOLUTION_NOT_IMPLEMENTED,
            "demand-bound symbol resolution",
        ));
        return ResolutionReport {
            environment_identity,
            environment_problems: problems,
            target: request.target.clone(),
            use_kind: request.use_kind,
            caller: request.caller.clone(),
            analysis: ResolutionAnalysis::NotPerformed,
            state: None,
            resolved: None,
            candidates: Vec::new(),
            // A dispatch report would claim a performed candidate enumeration, so a request
            // whose declaration was not resolved reports no dispatch at all.
            dispatch: None,
            // Nothing was demanded, so nothing was read.
            reads: Vec::new(),
            coverage: Coverage::not_requested(),
            execution: ExecutionReport::Failed {
                reason: TerminationReason::Unsupported {
                    code: RESOLUTION_NOT_IMPLEMENTED.to_string(),
                },
                usage: budget.usage(),
            },
            diagnostics,
        };
    };

    // `dispatch = Some(..)` also asks for `KnownCandidates` inside one explicit scope. This
    // slice resolves the declaration only, so the request is not answered in silence: the
    // report names the capability that did not run and claims no candidate.
    if request.dispatch.is_some() {
        diagnostics.push(dispatch_not_implemented_diagnostic());
    }
    let mut closure = HeaderClosure::new(content, &request.environment);
    let answer = closure.demand(&target.0, HeaderDemand::RequestedDefinition, budget);
    let extent = answer
        .searched
        .expect("the first demand of a fresh closure performs the search it answers");
    let reads = published_reads(&closure);
    let concluded = answer.decision.is_ok();
    let usage = budget.usage();
    let (analysis, state, resolved, candidates, execution) = match answer.decision {
        Ok(handle) => {
            let lookup = &closure.resolution(handle).lookup;
            let state = match lookup.state {
                HeaderLookupState::Found => ResolutionState::Resolved,
                HeaderLookupState::Missing => ResolutionState::Missing,
                HeaderLookupState::Ambiguous => ResolutionState::Ambiguous,
            };
            let resolved = lookup.location.as_ref().map(|location| {
                resolved_class(&location.loader, &location.definition, &request.target)
            });
            let candidates = lookup
                .candidates
                .iter()
                .map(|location| {
                    resolved_class(&location.loader, &location.definition, &request.target)
                })
                .collect();
            (
                ResolutionAnalysis::Performed,
                Some(state),
                resolved,
                candidates,
                ExecutionReport::Complete { usage },
            )
        }
        Err(error) => {
            let (execution, diagnostic) = terminal(&error, usage);
            // The lookup ran and stopped before a semantic decision. Only a budget stop *is* a
            // decision (`BudgetExceeded`, because the declared bounds are what decided the
            // answer); a cancellation and a damaged candidate or stopped listing stay
            // `state = None` and are recorded as the interruption they are, so
            // `Inaccessible`/`IncompatibleClassChange` keep their access and link meaning.
            let state = matches!(error, Error::BudgetExceeded { .. })
                .then_some(ResolutionState::BudgetExceeded);
            diagnostics.push(diagnostic);
            (
                ResolutionAnalysis::Performed,
                state,
                None,
                Vec::new(),
                execution,
            )
        }
    };
    // The closure's own diagnostics belong to this report: a refused cyclic hierarchy names
    // the classes and loader it refused, and dropping it would hide the reason a closure is
    // short.
    diagnostics.extend(closure.diagnostics().iter().cloned());
    ResolutionReport {
        environment_identity,
        environment_problems: problems,
        target: request.target.clone(),
        use_kind: request.use_kind,
        caller: request.caller.clone(),
        analysis,
        state,
        resolved,
        candidates,
        dispatch: None,
        reads,
        coverage: search_coverage(extent.examined, extent.positions, concluded),
        execution,
        diagnostics,
    }
}

/// The read records of one performed closure, in read order.
///
/// The crate-private demands are mapped onto the public vocabulary here, at the boundary that
/// owns it: a demand added by a later slice fails to compile until it is mapped, so the report
/// cannot publish a reason the closure never had.
fn published_reads(closure: &HeaderClosure<'_>) -> Vec<HeaderRead> {
    closure
        .reads()
        .iter()
        .map(|read| HeaderRead {
            loader: read.loader.clone(),
            definition: read.definition.clone(),
            reason: match read.demand {
                HeaderDemand::RequestedDefinition => ReadReason::RequestedDefinition,
                HeaderDemand::ParentChain => ReadReason::ParentChain,
                HeaderDemand::HierarchyClosure => ReadReason::HierarchyClosure,
            },
        })
        .collect()
}

/// The class symbol this slice performs a lookup for, if any.
///
/// A member symbol belongs to 2.3, and a rejected environment never yields a unique definition
/// (invariant 2), so neither one starts a search.
fn performable_class<'a>(
    target: &'a SymbolRef,
    problems: &[EnvironmentProblem],
) -> Option<&'a JvmBytes> {
    match target {
        SymbolRef::Class { owner } if problems.is_empty() => Some(owner),
        _ => None,
    }
}

/// One resolved class: the position that selected it and the raw symbol that was asked for.
///
/// A class symbol has no separate declaring member, so the raw reference is published as it
/// was asked (`report.target` carries the same bytes by contract) and the new information is
/// the position: the loader and the physical definition.
fn resolved_class(
    loader: &LoaderId,
    definition: &PhysicalDefinitionId,
    target: &SymbolRef,
) -> ResolvedMemberRef {
    ResolvedMemberRef {
        loader: loader.clone(),
        definition: definition.clone(),
        member: target.clone(),
    }
}

/// Resolution coverage of one performed lookup.
///
/// The resolution plane reports the prefix of the effective order the search examined and,
/// when the search stopped, the positions it never reached. A concluded lookup declares the
/// positions its decision covers and nothing as skipped: the order stops there by rule, because
/// the first position that holds a candidate is the definition. The request declares no artifact
/// range of its own, so the artifact and dynamic planes stay `NotRequested`.
fn search_coverage(examined: u32, positions: u32, concluded: bool) -> Coverage {
    const LABEL: &str = "provider_search_position";
    let mut scanned = Vec::new();
    let mut skipped = Vec::new();
    if examined > 0 {
        scanned.push(CoverageRange {
            label: LABEL.to_string(),
            start: 0,
            end: u64::from(examined),
        });
    }
    if !concluded && examined < positions {
        skipped.push(CoverageRange {
            label: LABEL.to_string(),
            start: u64::from(examined),
            end: u64::from(positions),
        });
    }
    Coverage {
        artifact_structural: CoverageDimension::not_requested(),
        runtime_resolution: CoverageDimension {
            state: if concluded {
                CoverageState::CompleteWithinSchema
            } else {
                CoverageState::Partial
            },
            scanned,
            skipped,
            uninterpreted_extensions: Vec::new(),
        },
        dynamic_analysis: CoverageDimension::not_requested(),
    }
}

/// Terminal mapping of a refusal, under the same contract the physical reports use: a budget
/// stop is a partial execution that names its dimension, a cancellation is a cancellation, and
/// a structural failure keeps the reader's or listing's own code.
fn terminal(error: &Error, usage: UsageSnapshot) -> (ExecutionReport, Diagnostic) {
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
                format!("budget_exceeded_{}", budget_dimension_code(*dimension)),
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

/// The diagnostic of a requested dispatch enumeration that this slice does not perform.
///
/// Warning, not `Error`: the declaration lookup in the same request did run, so this diagnostic
/// reports a requested range that was not covered rather than a rejected request. 2.5 replaces
/// it with a real `DispatchReport` and the code disappears with the capability.
fn dispatch_not_implemented_diagnostic() -> Diagnostic {
    Diagnostic {
        code: DISPATCH_NOT_IMPLEMENTED.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: "the request also asks for known dispatch candidates inside an explicit scope; \
                  this engine slice resolves the declaration only, so `dispatch` stays empty and \
                  no runtime target is claimed"
            .to_string(),
        provenance: None,
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
        // Nothing was demanded, so no class header was read.
        reads: Vec::new(),
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
