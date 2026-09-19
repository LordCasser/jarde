//! P2 resolution requests and reports (demand-resolver entry).
//!
//! The schema is fixed by the design's public skeleton section: request shape, resolution
//! state, dispatch report and the declaration-reference query/report. Symbols stay raw
//! bytes; nothing here normalizes an owner, name or descriptor.
//!
//! This module owns the request/report shape and maps a performed lookup into it.
//! [`validate_request`] checks the request shape, [`resolution_report`] answers one request: a
//! class symbol in an environment the validator accepted is looked up for real by the 2.2
//! closure over [`crate::providers`], a member symbol is resolved by the 2.3 rules over
//! [`crate::members`], a request that asks for dispatch candidates runs the 2.5 plane over
//! [`crate::dispatch`] once its declaration resolved, every header the request really read is
//! published as a [`HeaderRead`], and a request no slice can perform keeps the honest
//! unavailable state (`NotPerformed` / `state = None` / `Failed { Unsupported }`) instead of
//! claiming a result the closure did not produce.
//!
//! One capability code still names what no slice implements: a request whose environment the
//! validator rejected reports `resolution_not_implemented`, because a rejected environment never
//! yields a definition and no search may start from it. It disappears only when a slice decides
//! what a rejected environment can honestly answer.
//!
//! The module borrows vocabulary from `query` ([`ConsumerKind`], [`ConsumerSchema`],
//! [`XrefOperation`]) and never the other way round: `query` and `xref` must not know that
//! this module exists (A17).

use crate::dispatch::{self, DeclarationShape, DispatchEvidence, DispatchOutcome, DispatchStop};
use crate::environment::{
    CallerContext, EnvironmentIdentity, EnvironmentProblem, ResolutionEnvironment,
    environment_diagnostics, require_content_snapshot, unavailable_diagnostic,
    validate_environment, validate_environment_with_caller,
};
use crate::providers::{
    HeaderClosure, HeaderDemand, HeaderLookupState, HierarchyGapKind, WalkGaps, escaped,
};
use jarde_query::query::{ConsumerKind, ConsumerSchema, XrefItem, XrefOperation, XrefTarget};
use jarde_reader::accounting::with_usage;
use jarde_reader::artifact::{ArtifactSnapshot, budget_dimension_code};
use jarde_reader::budget::{Budget, CountedBudgetDimension, UsageSnapshot};
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    Coverage, CoverageDimension, CoverageRange, CoverageState, Diagnostic, DiagnosticSeverity,
    ExecutionReport, JvmBytes, Location, OriginMember, OriginSet, PhysicalDefinitionId, SymbolRef,
    TerminationReason,
};
use jarde_reader::view::{LoaderId, PhysicalScope};
use serde::{Deserialize, Serialize};

/// Capability name of the resolution entry points while they are not implemented.
///
/// It survives the member slice (2.3), the declaration-query slice (2.4) and the dispatch slice
/// (2.5) as the honest state of a request whose environment the validator rejected, and as the
/// state of a declaration a query states no candidate rule for: a rejected environment never
/// yields a definition, so no lookup starts, and a class symbol is not a member declaration.
pub(crate) const RESOLUTION_NOT_IMPLEMENTED: &str = "resolution_not_implemented";

/// Diagnostic code of a dispatch request whose declaration did not resolve.
///
/// A warning next to the resolution that really ran: the request asked for known candidates
/// inside a range, and a range without a resolved member declaration has nothing to enumerate
/// overrides of, so `dispatch` stays absent instead of looking like an empty candidate list.
const DISPATCH_NO_DECLARATION: &str = "resolution_dispatch_no_declaration";

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
///
/// The range is the P1 scope vocabulary (`SnapshotAll` or a tree root), and `consumers` is the
/// structural-consumer schema a later refinement would read use sites with. The CHA-lite plane
/// of 2.5 enumerates class headers only, so it reads no consumer fact and consumes no consumer
/// byte: the field stays part of the request schema the design fixed for the whole dispatch
/// capability.
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
///
/// The states answer different questions about the requested symbol and are never each other's
/// approximation (A11). `Resolved` names the declaration the order selected, `Missing` is the
/// negation — no readable position of the order provides it — and it is claimed **only** when the
/// search read every branch it entered. `UnresolvedDependency` is what a member search reports
/// instead of that negation when a class its hierarchy needed could not be read: the missing
/// dependency is not evidence that the name does not exist, so the report states what it could
/// not read (see [`ResolutionReport::unresolved_dependencies`]) rather than answering the
/// question it never got to ask. `Inaccessible` and `IncompatibleClassChange` are decided facts
/// about a selected declaration (the access rules denied it, the invocation kind or the inherited
/// interface defaults contradict it), `Ambiguous` is several declarations of the member that
/// cannot be told apart at one selection position, and `UnsupportedPolicy` is an owner kind this
/// slice does not resolve.
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
    /// A class the search needed is not in this snapshot's order, so no declaration is claimed:
    /// neither the negation `Missing` nor the selection `Resolved`. The classes that could not be
    /// read are published in [`ResolutionReport::unresolved_dependencies`].
    UnresolvedDependency,
}

/// One class a request's closure needed and the order did not resolve.
///
/// The record is evidence, never a verdict about the class it names: it says which name the
/// closure was asked for, which loader's own order searched it, which hierarchy edge demanded it
/// and which class declares that edge. It is what A11's "a missing dependency is not a negative"
/// means concretely — reading nothing for a name is not the statement that the name does not
/// exist, and the report publishes the name instead of turning it into [`ResolutionState::Missing`].
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedDependency {
    /// The class name the closure demanded and could not resolve.
    pub name: JvmBytes,
    /// The loader whose own order searched for the name (JVMS 5.4.3.1).
    pub loader: LoaderId,
    /// The edge that demanded the name, in the read vocabulary: a `super_class` step is
    /// [`ReadReason::ParentChain`], an `interfaces` step is [`ReadReason::HierarchyClosure`], and a
    /// dispatch range's own enumeration is [`ReadReason::DispatchScope`].
    pub reason: ReadReason,
    /// The class that declares that edge: the last ancestor of the path that reached the name, or
    /// the class of a range whose own name the order could not resolve. `None` only for a walk's
    /// root layer, which no edge reached.
    pub declared_by: Option<JvmBytes>,
    /// What the order stated about the name.
    pub gap: DependencyGap,
}

/// Why one class of a hierarchy could not be read.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyGap {
    /// No position of the order provides the name: the class the closure needed is not in this
    /// snapshot, which is the missing dependency of A11 and not the statement that the name does
    /// not exist.
    Missing,
    /// One position holds the name with definitions that cannot be told apart, so this request
    /// cannot say which of them the search had to continue through.
    Ambiguous,
    /// The name's own supertype edge repeats: an illegal hierarchy the walk refuses to follow.
    Cyclic,
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
/// recorded as the edge that reached it — a `super_class` step is `ParentChain`, an `interfaces`
/// step is `HierarchyClosure` — even when the name it was demanded for turned out to resolve to
/// that definition, and a read that happened before the search stopped keeps the demand that
/// caused it. A class the request names by identity (the member's owner, the use site's
/// enclosing class) is a `MemberOwner` read even when a hierarchy edge reaches the same class
/// later, because the closure memo keeps the reason of the read that really happened.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadReason {
    /// The request target itself.
    RequestedDefinition,
    /// A class reached along a `super_class` edge (the type's parent chain).
    ParentChain,
    /// A class reached along an `interfaces`/superinterface edge (the interface graph).
    HierarchyClosure,
    /// One class of an explicitly scoped candidate enumeration (2.5).
    DispatchScope,
    /// A class the member rules read by identity: the member reference's owner, and the class
    /// that declares the use site's enclosing method (2.3).
    MemberOwner,
    /// The target method's body — the only reason that upgrades to a body read (3.x).
    DriverMethodBody,
    /// The class definition a presented body's named callees were read from (P3 3.2): the members
    /// one recovery run's own call sites named, read on demand and one body attempt each.
    CalleeMemberBody,
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

/// Why one known candidate stands under an open world.
///
/// The variants are the facts the range itself states, never a verdict about the candidate:
/// [`OpenWorldEvidence::ExternalSubclass`] and [`OpenWorldEvidence::UnknownLoader`] name
/// content and loaders the request cannot see, [`OpenWorldEvidence::RuntimeTransformation`]
/// names a runtime the snapshot does not prove, [`OpenWorldEvidence::MissingDependency`] names
/// an unread supertype of that candidate's own closure, and
/// [`OpenWorldEvidence::OrderedRoot`] names the declared root position the candidate's own
/// lookup was decided at — past the first position, which means this loader's ordered roots
/// really hold more than one position for the name's layer, so the same layer may hold another
/// definition. A candidate decided at the very first position states nothing of that kind, and
/// a candidate the plane can state completely carries no evidence at all.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OpenWorldEvidence {
    ExternalSubclass,
    UnknownLoader,
    RuntimeTransformation,
    MissingDependency,
    OrderedRoot { index: u32 },
}

/// One known candidate of a dispatch request.
///
/// The member is the candidate class's own declaration of the resolved member (its owner is that
/// class's internal name), and `evidence` is the open-world fact the candidate stands under:
/// `None` when the plane can state the candidate completely, which is what lets a complete range
/// answer `open_world = false`. There is deliberately no field that names one runtime target —
/// a single candidate is one known candidate, never a proven target.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispatchCandidate {
    pub member: ResolvedMemberRef,
    pub evidence: Option<OpenWorldEvidence>,
}

/// Known candidates of an explicitly scoped dispatch request.
///
/// Present exactly when the request asked for a range and its member declaration resolved. The
/// plane answers "which overrides or implementations of this declaration exist inside this
/// range", so `candidates` may hold one entry or several and `open_world` says whether the plane
/// can state the range completely: it is true as soon as one candidate stands under an
/// open-world fact, as soon as the range holds a position this request could not decide, and as
/// soon as the plane stopped. There is no "unique target" field to read instead — a caller
/// decides for itself from `candidates` and the evidence.
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
    /// The semantic decision, or `None` when the run reached none: a capability that never ran
    /// (`analysis = NotPerformed`) and a run that stopped before deciding (`analysis =
    /// Performed` with `execution` `Cancelled`/`Failed`) both leave this `None` (invariant 3).
    pub state: Option<ResolutionState>,
    pub resolved: Option<ResolvedMemberRef>,
    /// Only definitions that are indistinguishable at one selection position.
    pub candidates: Vec<ResolvedMemberRef>,
    pub dispatch: Option<DispatchReport>,
    /// Every class a closure this request ran needed and the order did not resolve, in the order
    /// the searches found them, each stated once: the declaration search's own hierarchy and,
    /// when the request asked for a range, the range's classes and their hierarchies.
    ///
    /// The plane is what makes the two facts a caller must not confuse separable. A name read
    /// through this list may well exist outside the snapshot the request was given, so the report
    /// never publishes [`ResolutionState::Missing`] for a member whose hierarchy this list leaves
    /// incomplete — and a `Resolved` declaration found in a branch the order did read stays a
    /// resolved declaration, with the unread branches that sit above it published beside it. Like
    /// `reads`, this is evidence of the search and not a result entry: it is published uncharged.
    pub unresolved_dependencies: Vec<UnresolvedDependency>,
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
    /// The candidates that really resolve to `declaration`; each one answers the query with the
    /// raw symbol, consumer, operation and physical use site the structural scan found.
    ///
    /// Publishing one item costs one `ResultItems`, charged before it is published, exactly like
    /// a P1 item (and like every diagnostic below).
    pub items: Vec<DeclarationRefItem>,
    /// Candidates that a missing dependency, an ambiguous position, a damaged read or a budget
    /// stop left undecided; they are never presented as excluded, and each one keeps its
    /// physical use site in a diagnostic of the same report. A candidate the search resolved to
    /// a *different* declaration is not counted here: it is decided, and it is not a reference
    /// to this declaration.
    ///
    /// The count and the diagnostics are published together — one `resolution_candidate_unresolved`
    /// per counted candidate, each charged one `ResultItems` — so a candidate that could not be
    /// reported is not counted either.
    pub unresolved_candidates: u64,
    /// Whether the answer this report publishes is a prefix rather than the whole result: the
    /// scan stopped before the end of its range (its item limit, a budget or cancellation stop),
    /// or the report itself could not publish everything the scan found. A candidate behind
    /// either stop was neither published nor counted as undecided.
    ///
    /// This is a truncation flag and not a cursor: a declaration-reference query replays nothing.
    pub has_more: bool,
    pub returned_items: u64,
    /// Every class header this scan read, in read order, at most once per
    /// `(definition, loader)`; empty when the scan performed nothing.
    pub reads: Vec<HeaderRead>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    /// Includes one diagnostic per environment problem, under the same code, the scan's own
    /// diagnostics, every rule diagnostic the member searches produced, one diagnostic per
    /// candidate the query could not decide, and the diagnostics that explain a stop.
    ///
    /// Each diagnostic the *resolution* produced costs one `ResultItems`, charged before it
    /// enters the report; the environment plane and the stop explanations are control metadata
    /// and are not charged.
    pub diagnostics: Vec<Diagnostic>,
}

/// Request-level checks of a resolution request: shape only, no artifact access.
///
/// A dispatch range is checked the way the P1 query and the declaration query check their own
/// scope: the only root a fresh snapshot establishes is its own root container, so a range that
/// names another one cannot describe this snapshot and is refused instead of being silently
/// ignored (2.3's scope rule, applied to the dispatch plane).
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
    if let Some(dispatch) = &request.dispatch
        && let PhysicalScope::ArtifactTree { root_container } = &dispatch.scope
        && root_container.0 != "root"
    {
        return Err(Error::invalid_input(
            "query_artifact_tree_root_mismatch",
            "dispatch scope tree root does not match the snapshot root container",
        ));
    }
    Ok(())
}

/// Request-level checks of a declaration-reference query: shape only, no artifact access.
///
/// The scope is checked the way the P1 query checks its own physical view: the only root a
/// fresh snapshot establishes is its own root container, so a scope that names another one
/// cannot describe this snapshot and is refused instead of being silently ignored.
pub(crate) fn validate_declaration_reference_query(
    content: &[ArtifactSnapshot],
    query: &DeclarationRefQuery,
) -> Result<()> {
    require_content_snapshot(content, &query.environment.runtime.physical.snapshot)?;
    if let PhysicalScope::ArtifactTree { root_container } = &query.scope
        && root_container.0 != "root"
    {
        return Err(Error::invalid_input(
            "query_artifact_tree_root_mismatch",
            "declaration-reference scope tree root does not match the snapshot root container",
        ));
    }
    Ok(())
}

/// Result of one legally shaped resolution request.
///
/// A class symbol in an environment whose declarations are usable is looked up by
/// [`crate::providers`] and the lookup is mapped into the schema's own planes; a member symbol
/// (2.3) is resolved by [`crate::members`], which searches the class and interface paths of
/// JVMS 5.4.3 and then holds the selected declaration to the invocation-kind and access rules.
/// The mapping rule is invariant 3: `state = Some(v)` exactly when the run reached a semantic
/// decision, and `state = None` covers both a capability that never ran
/// (`analysis = NotPerformed`) and a run that stopped before deciding (`analysis = Performed`
/// with `execution` `Cancelled`/`Failed`/`Partial`).
///
/// Concretely: for a class symbol, `Resolved`, `Missing` and `Ambiguous` are the lookup's three
/// decisions; for a member symbol the 2.3 rule table adds `Inaccessible` (the access rules
/// denied the reference) and `IncompatibleClassChange` (the invocation kind contradicts the
/// declaration or the hierarchy, or two defaults conflict) and `UnsupportedPolicy` (an owner
/// kind this slice does not resolve), and a member search that could not read a class of its
/// hierarchy reports `UnresolvedDependency` with the classes it needed in
/// `unresolved_dependencies` (2.2) instead of the negation `Missing`. In both paths a budget stop
/// is the fourth decision (`BudgetExceeded`, with the same stop in `execution`). A cancellation is
/// `execution = Cancelled` with `state = None`; a damaged candidate or a stopped listing is
/// `execution = Failed { Error { code } }` with `state = None` and a diagnostic that names the
/// origin — a read failure never occupies a semantic state.
///
/// A rejected environment never yields a unique definition (invariant 2), so it keeps the
/// honest unavailable state; environment problems are part of the report and are never an `Err`
/// and never a fallback search order.
pub(crate) fn resolution_report(
    content: &[ArtifactSnapshot],
    request: &ResolutionRequest,
    budget: &mut Budget,
) -> ResolutionReport {
    let (problems, environment_identity) =
        validate_environment_with_caller(content, &request.environment, &request.caller);
    let mut diagnostics = environment_diagnostics(&problems);
    let Some(performable) = performable_target(&request.target, request.use_kind, &problems) else {
        diagnostics.push(unavailable_diagnostic(
            RESOLUTION_NOT_IMPLEMENTED,
            "demand-bound symbol resolution",
        ));
        // A dispatch report would claim a performed candidate enumeration, and this request
        // performed nothing at all, so the requested range is named and no dispatch is claimed.
        if request.dispatch.is_some() {
            diagnostics.push(dispatch_no_declaration_diagnostic());
        }
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
            dispatch: None,
            // Nothing was demanded, so nothing was read and no dependency was searched for.
            unresolved_dependencies: Vec::new(),
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

    let mut closure = HeaderClosure::new(content, &request.environment);
    // A refused charge during the publication of a resolution result: the entries already
    // published stay, and the stop is the execution the report has to name. It is recorded at
    // the phase that refused (2.3's rule diagnostics or the closure's own diagnostics), so the
    // earliest stop keeps governing `execution`.
    let mut rule_stop: Option<ExecutionReport> = None;
    // The classes this request's closures needed and could not read, as the report publishes
    // them: the declaration search's own hierarchy first, then the dispatch range's classes.
    let mut unresolved: Vec<UnresolvedDependency> = Vec::new();
    let (analysis, state, resolved, candidates, concluded, covered, execution) = match performable {
        Performable::Class(name) => {
            // The request's own target is the one symbol demand the *caller* initiates; every
            // successor the resolved header declares is searched from that header's own loader.
            let answer =
                closure.demand_from_caller(&name.0, HeaderDemand::RequestedDefinition, budget);
            let concluded = answer.decision.is_ok();
            let usage = budget.usage();
            match answer.decision {
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
                        concluded,
                        true,
                        ExecutionReport::Complete { usage },
                    )
                }
                Err(error) => {
                    let (execution, diagnostic) = terminal(&error, usage);
                    // The lookup ran and stopped before a semantic decision. Only a budget stop
                    // *is* a decision (`BudgetExceeded`, because the declared bounds are what
                    // decided the answer); a cancellation and a damaged candidate or stopped
                    // listing stay `state = None` and are recorded as the interruption they are,
                    // so `Inaccessible`/`IncompatibleClassChange` keep their access and link
                    // meaning.
                    let state = matches!(error, Error::BudgetExceeded { .. })
                        .then_some(ResolutionState::BudgetExceeded);
                    diagnostics.push(diagnostic);
                    (
                        ResolutionAnalysis::Performed,
                        state,
                        None,
                        Vec::new(),
                        concluded,
                        true,
                        execution,
                    )
                }
            }
        }
        Performable::Member(target, use_kind) => {
            let outcome = crate::members::resolve_member(
                &mut closure,
                target,
                use_kind,
                &request.caller,
                budget,
            );
            let usage = budget.usage();
            match outcome {
                Ok(outcome) => {
                    let (state, resolved, candidates) =
                        member_planes(&outcome.decision, &request.target, &outcome.unread);
                    let complete = outcome.hierarchy_complete();
                    // The branches the search could not read are published before the rule
                    // diagnostics: they are the evidence the state above is read with, and the
                    // request cannot afford them any less than the records of the reads it made.
                    publish_dependencies(&outcome.unread, &mut unresolved);
                    // Every rule diagnostic 2.3 produced is a resolution result like any other
                    // entry of the report, so it costs one `ResultItems` before it is published
                    // (the same discipline the declaration-reference query applies to its own
                    // rule diagnostics). A refused charge keeps the diagnostics already
                    // published and ends the request's publication, which is why the stop it
                    // leaves behind is handed back uncharged.
                    for rule in outcome.diagnostics {
                        match charge_and_publish(rule, &mut diagnostics, budget) {
                            None => {}
                            Some(stop) => {
                                push_stop_diagnostic(&mut diagnostics, stop.diagnostic);
                                rule_stop = Some(stop.execution);
                                break;
                            }
                        }
                    }
                    // An owner kind this slice does not resolve covers no range of the requested
                    // resolution, so the plane is partial even though the decision itself is
                    // complete: the request asked for a range that was never searched.
                    let covered = complete
                        && !matches!(outcome.decision, crate::members::MemberDecision::ArrayOwner);
                    (
                        ResolutionAnalysis::Performed,
                        Some(state),
                        resolved,
                        candidates,
                        true,
                        covered,
                        ExecutionReport::Complete { usage },
                    )
                }
                Err(error) => {
                    let (execution, diagnostic) = terminal(&error, usage);
                    let state = matches!(error, Error::BudgetExceeded { .. })
                        .then_some(ResolutionState::BudgetExceeded);
                    diagnostics.push(diagnostic);
                    (
                        ResolutionAnalysis::Performed,
                        state,
                        None,
                        Vec::new(),
                        false,
                        false,
                        execution,
                    )
                }
            }
        }
    };
    // The 2.5 dispatch plane runs only over a declaration that really resolved: a candidate is an
    // override of a resolved member declaration, so a class symbol, a rejected environment and
    // every non-`Resolved` state publish no dispatch at all instead of an empty candidate list.
    // A request whose publication already stopped starts no plane either: the stop ended the
    // report, and a range this request could not pay for is not a range it searched — `dispatch`
    // stays absent, exactly like a request that stopped before the plane, and the stop explains
    // why.
    let mut dispatch = None;
    let mut dispatch_incomplete = false;
    let mut dispatch_execution = None;
    let mut dispatch_search_stopped = false;
    match (
        request.dispatch.as_ref().filter(|_| rule_stop.is_none()),
        state,
        resolved.as_ref(),
    ) {
        (Some(scope), Some(ResolutionState::Resolved), Some(declaration)) => {
            match declaration_shape(declaration) {
                Some(shape) => {
                    let outcome = dispatch::dispatch_candidates(
                        content,
                        &request.environment,
                        &mut closure,
                        &scope.scope,
                        &shape,
                        budget,
                    );
                    // The open-world answer is the plane's own field: the plane sets it when it
                    // stopped, so a test that observes `open_world` after a stop observes the
                    // plane's rule and not a second copy of it in this layer.
                    dispatch_execution =
                        publish_dispatch(&outcome, &mut diagnostics, budget.usage());
                    // The range's own unread names are evidence of the same kind as the
                    // declaration search's: a class the order could not resolve is unsearched
                    // range, and it is published by name instead of being left as a flag.
                    publish_dependencies(&outcome.unread, &mut unresolved);
                    dispatch_search_stopped = outcome
                        .stop
                        .as_ref()
                        .is_some_and(DispatchStop::ended_a_search);
                    dispatch_incomplete = outcome.stop.is_some()
                        || outcome.undecided_positions
                        || !outcome.unread.is_empty();
                    dispatch = Some(DispatchReport {
                        scope: scope.scope.clone(),
                        candidates: outcome.candidates.iter().map(dispatch_candidate).collect(),
                        open_world: outcome.open_world,
                    });
                }
                // A class symbol is a type, not a member declaration: the candidate rules 2.5
                // states are member rules, so the range is named and no candidate is claimed.
                None => diagnostics.push(dispatch_no_declaration_diagnostic()),
            }
        }
        (Some(_), _, _) => diagnostics.push(dispatch_no_declaration_diagnostic()),
        (None, _, _) => {}
    }
    // The closure's own diagnostics belong to this report: a refused cyclic hierarchy names
    // the classes and loader it refused, and dropping it would hide the reason a closure is
    // short.
    let mut closure_stop: Option<ExecutionReport> = None;
    for diagnostic in closure.diagnostics().to_vec() {
        match charge_and_publish(diagnostic, &mut diagnostics, budget) {
            None => {}
            Some(stop) => {
                push_stop_diagnostic(&mut diagnostics, stop.diagnostic);
                closure_stop = Some(stop.execution);
                break;
            }
        }
    }
    // Coverage is published after every search this request ran, so a dispatch range's lookups
    // are part of the same plane's sum. An undecided or stopped range is a prefix of the range
    // the request asked for, which is partial however the declaration itself was decided — and
    // so is a range whose publication stopped: the answer is a prefix of what the plane found.
    //
    // `concluded` is the search's own statement, and only a stop that ended a *search* revokes
    // it (see [`DispatchStop::ended_a_search`]): the searches that ran before a refused
    // `ResultItems` charge reached their own conclusions, so their remaining declared positions
    // are not unsearched range, and a skipped range would claim a search this request never
    // left unfinished.
    let extent = closure.searched_extent();
    let coverage = search_coverage(
        extent.examined,
        extent.positions,
        concluded && !dispatch_search_stopped,
        covered && !dispatch_incomplete && rule_stop.is_none() && closure_stop.is_none(),
    );
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
        dispatch,
        unresolved_dependencies: unresolved,
        reads: published_reads(&closure),
        coverage,
        // The usage snapshot of the whole request: the dispatch range's own reads and published
        // candidates happened after the resolution built its execution, so the report publishes
        // the final usage under whatever execution the request ended with. The earliest stop
        // governs: a refused rule diagnostic, then a plane that stopped searching, then the
        // closure diagnostics a refused charge kept out of the report.
        execution: with_usage(
            rule_stop
                .or(dispatch_execution)
                .or(closure_stop)
                .unwrap_or(execution),
            budget.usage(),
        ),
        diagnostics,
    }
}

/// Publishes one dispatch outcome into the report's own planes.
///
/// The range's listing diagnostics and the stop explanation enter `diagnostics`; the returned
/// execution is the one the report has to publish instead of the resolution's own, and it is
/// present exactly when the plane stopped. The open-world answer is not answered here: the plane
/// states it (including the fact a stop adds, because a stopped plane covers an unknown part of
/// the range) and the report publishes the plane's own field, so the two can never disagree.
fn publish_dispatch(
    outcome: &DispatchOutcome,
    diagnostics: &mut Vec<Diagnostic>,
    usage: UsageSnapshot,
) -> Option<ExecutionReport> {
    diagnostics.extend(outcome.diagnostics.iter().cloned());
    let stop = outcome.stop.as_ref()?;
    Some(match stop {
        // The listing's own execution already names its stop; only its usage snapshot is
        // replaced, so the report's usage is the usage of the whole request.
        DispatchStop::Truncated(execution) => with_usage(execution.clone(), usage),
        DispatchStop::Refused(error) => {
            let (execution, diagnostic) = terminal(error, usage);
            diagnostics.push(diagnostic);
            execution
        }
    })
}

/// The member shape a dispatch request enumerates overrides of.
///
/// A class symbol names a type, not a member declaration, so it has no shape: the candidate
/// rules of 2.5 are member rules and there is nothing to compare a class against.
fn declaration_shape(declaration: &ResolvedMemberRef) -> Option<DeclarationShape<'_>> {
    match &declaration.member {
        SymbolRef::Field {
            name, descriptor, ..
        } => Some(DeclarationShape {
            kind: crate::members::MemberKind::Field,
            declaring: declaring_node(declaration),
            name: &name.0,
            descriptor: &descriptor.0,
        }),
        SymbolRef::Method {
            name, descriptor, ..
        } => Some(DeclarationShape {
            kind: crate::members::MemberKind::Method,
            declaring: declaring_node(declaration),
            name: &name.0,
            descriptor: &descriptor.0,
        }),
        SymbolRef::Class { .. } => None,
    }
}

/// The declaring class of one resolved member as a **node**.
///
/// The dispatch subtype test compares this pair, never the owner string: a class another loader
/// defines under the declaring name is a different class and overrides nothing (0.1).
fn declaring_node(declaration: &ResolvedMemberRef) -> crate::providers::NodeIdentity {
    crate::providers::NodeIdentity::new(&declaration.loader, &declaration.definition)
}

/// One crate-private candidate as the report publishes it.
fn dispatch_candidate(candidate: &dispatch::DispatchCandidate) -> DispatchCandidate {
    DispatchCandidate {
        member: ResolvedMemberRef {
            loader: candidate.loader.clone(),
            definition: candidate.definition.clone(),
            member: candidate.member.clone(),
        },
        evidence: candidate.evidence.map(open_world_evidence),
    }
}

/// The crate-private evidence vocabulary as the public one.
///
/// The mapping is exhaustive at the boundary that owns the public vocabulary, so an evidence
/// kind added later fails to compile until it is mapped.
fn open_world_evidence(evidence: DispatchEvidence) -> OpenWorldEvidence {
    match evidence {
        DispatchEvidence::ExternalSubclass => OpenWorldEvidence::ExternalSubclass,
        DispatchEvidence::UnknownLoader => OpenWorldEvidence::UnknownLoader,
        DispatchEvidence::RuntimeTransformation => OpenWorldEvidence::RuntimeTransformation,
        DispatchEvidence::MissingDependency => OpenWorldEvidence::MissingDependency,
        DispatchEvidence::OrderedRoot { index } => OpenWorldEvidence::OrderedRoot { index },
    }
}

/// The read records of one performed closure, in read order.
///
/// The crate-private demands are mapped onto the public vocabulary here, at the boundary that
/// owns it: a demand added by a later slice fails to compile until it is mapped, so the report
/// cannot publish a reason the closure never had.
///
/// The method-analysis entry point publishes its own closure's records through this same
/// mapping, so the one place where a demand becomes a public reason covers every report that
/// publishes reads — the resolution slice's own `published_reads` match cannot drift from the
/// body demand's.
pub(crate) fn published_reads(closure: &HeaderClosure<'_>) -> Vec<HeaderRead> {
    closure
        .reads()
        .iter()
        .map(|read| HeaderRead {
            loader: read.loader.clone(),
            definition: read.definition.clone(),
            reason: read_reason(read.demand),
        })
        .collect()
}

/// The public reason of one crate-private demand.
///
/// The mapping lives at the boundary that owns the public vocabulary and is exhaustive: a demand
/// added later fails to compile until it is named here. The read records and the unread-branch
/// records therefore state one and the same demand the same way — an edge is a `ParentChain` edge
/// in both, whatever it turned out to reach.
fn read_reason(demand: HeaderDemand) -> ReadReason {
    match demand {
        HeaderDemand::RequestedDefinition => ReadReason::RequestedDefinition,
        HeaderDemand::ParentChain => ReadReason::ParentChain,
        HeaderDemand::HierarchyClosure => ReadReason::HierarchyClosure,
        HeaderDemand::MemberOwner => ReadReason::MemberOwner,
        HeaderDemand::DispatchScope => ReadReason::DispatchScope,
        HeaderDemand::DriverMethodBody => ReadReason::DriverMethodBody,
        HeaderDemand::CalleeMemberBody => ReadReason::CalleeMemberBody,
    }
}

/// Publishes the branches a closure could not read, stating each fact once.
///
/// The crate-private gap vocabulary is mapped here, at the boundary that owns the public one, and
/// the mapping is exhaustive: a gap kind added later fails to compile until the report can state
/// it. Two records are the same fact when every field agrees — the same name demanded by the same
/// loader through the same edge of the same class — and one fact is published once however many
/// paths of the search reached it.
fn publish_dependencies(gaps: &WalkGaps, dependencies: &mut Vec<UnresolvedDependency>) {
    for (kind, gap) in gaps.iter() {
        let dependency = UnresolvedDependency {
            name: gap.name.clone(),
            loader: gap.loader.clone(),
            reason: read_reason(gap.demand),
            declared_by: gap.declared_by.clone(),
            gap: match kind {
                HierarchyGapKind::Missing => DependencyGap::Missing,
                HierarchyGapKind::Ambiguous => DependencyGap::Ambiguous,
                HierarchyGapKind::Cyclic => DependencyGap::Cyclic,
            },
        };
        if !dependencies.contains(&dependency) {
            dependencies.push(dependency);
        }
    }
}

/// The class symbol this slice performs a lookup for, if any.
///
/// A rejected environment never yields a unique definition (invariant 2), so it starts no
/// search at all. A member symbol is the 2.3 path and needs the reference use mapped into the
/// member vocabulary.
fn performable_target<'a>(
    target: &'a SymbolRef,
    use_kind: ReferenceUse,
    problems: &[EnvironmentProblem],
) -> Option<Performable<'a>> {
    if !problems.is_empty() {
        return None;
    }
    match target {
        SymbolRef::Class { owner } => Some(Performable::Class(owner)),
        // A class reference names no member: the entry point rejects that pairing
        // (`resolution_target_use_mismatch`) before a report is built, so `None` here is
        // unreachable from `Engine::resolve_symbol`, and a report built by hand keeps the honest
        // unavailable state instead of inventing a member rule for a class reference.
        SymbolRef::Field { .. } | SymbolRef::Method { .. } => {
            member_use(use_kind).map(|use_kind| Performable::Member(target, use_kind))
        }
    }
}

/// What one request can perform.
enum Performable<'a> {
    /// A class symbol: one class-name lookup (2.1/2.2).
    Class(&'a JvmBytes),
    /// A member symbol: the member rules of 2.3, held to this reference use.
    Member(&'a SymbolRef, crate::members::MemberUse),
}

/// The crate-private member vocabulary of one public reference use.
///
/// The mapping lives at the boundary that owns the public vocabulary, and it is exhaustive: a
/// reference kind added later fails to compile here until its member rule is stated. A class
/// reference names no member, so it maps to nothing.
fn member_use(use_kind: ReferenceUse) -> Option<crate::members::MemberUse> {
    Some(match use_kind {
        ReferenceUse::FieldRead => crate::members::MemberUse::FieldRead,
        ReferenceUse::FieldWrite => crate::members::MemberUse::FieldWrite,
        ReferenceUse::InvokeStatic => crate::members::MemberUse::InvokeStatic,
        ReferenceUse::InvokeSpecial => crate::members::MemberUse::InvokeSpecial,
        ReferenceUse::InvokeVirtual => crate::members::MemberUse::InvokeVirtual,
        ReferenceUse::InvokeInterface => crate::members::MemberUse::InvokeInterface,
        ReferenceUse::InvokeDynamic => crate::members::MemberUse::InvokeDynamic,
        ReferenceUse::ClassReference => return None,
    })
}

/// The report planes of one member decision.
///
/// `resolved` and `Ambiguous` are mutually exclusive by construction: `resolved` is published
/// for `Resolved` only — the access and link rejections say why the reference may not use the
/// member, and publishing a selected declaration next to a non-`Resolved` state would read as a
/// resolution — while an ambiguous position publishes its candidates and no `resolved` at all.
/// The declaration a rejection refused is still named in the diagnostic that refused it.
///
/// The one state the decision alone does not decide is the negation. `Missing` says no readable
/// position declares the member, and that is a statement the search may make only when it read
/// every branch it entered: with a branch left unread (`unread`) the member was never searched
/// there, so the report publishes `UnresolvedDependency` and the names of the classes it could
/// not read instead of answering a question it could not ask (A11). A selection the search did
/// reach stays `Resolved`, because the declaration it names was really read — the unread branches
/// above it are published beside the answer.
fn member_planes(
    decision: &crate::members::MemberDecision,
    target: &SymbolRef,
    unread: &WalkGaps,
) -> (
    ResolutionState,
    Option<ResolvedMemberRef>,
    Vec<ResolvedMemberRef>,
) {
    use crate::members::MemberDecision;
    match decision {
        MemberDecision::Resolved(location) => (
            ResolutionState::Resolved,
            Some(resolved_member(location)),
            Vec::new(),
        ),
        MemberDecision::Missing if unread.is_empty() => {
            (ResolutionState::Missing, None, Vec::new())
        }
        MemberDecision::Missing => (ResolutionState::UnresolvedDependency, None, Vec::new()),
        MemberDecision::OwnerAmbiguous(owners) => (
            ResolutionState::Ambiguous,
            None,
            // The candidates are class definitions, so each one carries the raw reference as it
            // was asked, exactly like the class-symbol path publishes an ambiguous position.
            owners
                .iter()
                .map(|owner| ResolvedMemberRef {
                    loader: owner.loader.clone(),
                    definition: owner.definition.clone(),
                    member: target.clone(),
                })
                .collect(),
        ),
        MemberDecision::DeclarationAmbiguous(locations) => (
            ResolutionState::Ambiguous,
            None,
            locations.iter().map(resolved_member).collect(),
        ),
        MemberDecision::KindMismatch | MemberDecision::DefaultConflict => {
            (ResolutionState::IncompatibleClassChange, None, Vec::new())
        }
        MemberDecision::AccessDenied => (ResolutionState::Inaccessible, None, Vec::new()),
        MemberDecision::ArrayOwner => (ResolutionState::UnsupportedPolicy, None, Vec::new()),
    }
}

/// One resolved member: the **declaration** the search selected.
///
/// The owner is the declaring class's own internal name (which may differ from the reference's
/// owner), the name and descriptor are the declaration's raw bytes, and the loader and physical
/// definition are those of the header the declaration was read from. `report.target` keeps the
/// raw reference, so a signature-polymorphic call site shows both descriptors side by side.
fn resolved_member(location: &crate::members::MemberLocation) -> ResolvedMemberRef {
    use crate::members::MemberKind;
    let owner = location.declaring_class.clone();
    let name = location.name.clone();
    let descriptor = location.descriptor.clone();
    let member = match location.kind {
        MemberKind::Field => SymbolRef::Field {
            owner,
            name,
            descriptor,
        },
        MemberKind::Method => SymbolRef::Method {
            owner,
            name,
            descriptor,
        },
    };
    ResolvedMemberRef {
        loader: location.loader.clone(),
        definition: location.definition.clone(),
        member,
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

/// Resolution coverage of one performed search.
///
/// The resolution plane reports the prefix of the effective order the searches examined and,
/// when a search stopped, the positions it never reached. A concluded lookup declares the
/// positions its decision covers and nothing as skipped: the order stops there by rule, because
/// the first position that holds a candidate is the definition. A member request runs one search
/// per class it reads, so the numbers are the sums the closure accumulated — every search starts
/// at position 0 of the same declared order, so a skipped range counts unexamined search
/// positions across those searches.
///
/// `hierarchy_complete` is the member search's own extent: a branch of the hierarchy that could
/// not be read leaves the coverage partial even though a decision was reached, and those
/// branches are named by the diagnostics that found them (their size is unknown, so no skipped
/// range can state them). The request declares no artifact range of its own, so the artifact and
/// dynamic planes stay `NotRequested`.
fn search_coverage(
    examined: u32,
    positions: u32,
    concluded: bool,
    hierarchy_complete: bool,
) -> Coverage {
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
            state: if concluded && hierarchy_complete {
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

/// The diagnostic of a dispatch request whose declaration did not resolve.
///
/// Warning, not `Error`: the declaration resolution in the same request really ran, so this
/// diagnostic names a range that was not enumerated rather than a rejected request. It covers
/// every reason a dispatch plane cannot run — a rejected environment, a class symbol, a
/// declaration the member rules decided otherwise, or a resolution that stopped — and no
/// candidate is claimed with it.
fn dispatch_no_declaration_diagnostic() -> Diagnostic {
    Diagnostic {
        code: DISPATCH_NO_DECLARATION.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: "the request also asks for known dispatch candidates inside an explicit scope, \
                  but no member declaration resolved: the dispatch plane enumerates the known \
                  overrides and implementations of a resolved member declaration, so `dispatch` \
                  stays absent and no runtime target is claimed"
            .to_string(),
        provenance: None,
    }
}

/// Result of one legally shaped declaration-reference query.
///
/// The query scans the requested scope with the structure consumers themselves — the same
/// scan machine `Engine::query` runs, opened with the candidate filter the queried declaration
/// is found under instead of one exact target — and then resolves every candidate it found
/// through 2.1 and 2.3 and compares the selected declaration with the requested one. The three
/// outcomes are kept apart:
///
/// * a candidate that really resolves to `query.declaration` becomes an item, with the raw
///   symbol, physical use site, operation and consumer the structural scan found;
/// * a candidate that resolves to **another** declaration is decided and left out: it is a
///   reference to a different declaration, not a reference to this one;
/// * a candidate no search could decide — a missing owner, an ambiguous position, damaged
///   bytes, a budget stop, a cancellation, or a use site whose instruction-level member kind
///   its evidence does not carry — is counted as unresolved and keeps its physical use site in
///   a diagnostic, because the report publishes no item for it and reporting it as excluded
///   would be a claim the engine cannot make.
///
/// Publishing one entry (an item or a diagnostic) costs one `ResultItems`, charged before the
/// entry is published — the rule diagnostics 2.3 returned, the undecided-candidate diagnostics
/// and the closure's own branch warnings alike, exactly as a P1 item or a domain diagnostic
/// does. A refused charge keeps the entries already published, ends the assembly and reports
/// `Partial { BudgetExceeded { ResultItems } }` with `has_more`, so the report is a prefix that
/// says so instead of an answer it could not afford. The diagnostics that explain the request
/// (`environment_problems` and the state of a capability that never ran) and the ones that
/// explain a stop are control metadata and are published uncharged, so a report never hides why
/// it ended. When a resolution stop and an assembly stop coincide, `execution` reports the
/// resolution stop that came first and the truncation is stated by `has_more` and the coverage
/// planes.
///
/// A request whose environment the validator rejected, and a declaration that is not a
/// member of a class, keep the honest unavailable state: nothing is scanned, no byte is read
/// and no candidate is claimed.
pub(crate) fn declaration_reference_report(
    content: &[ArtifactSnapshot],
    query: &DeclarationRefQuery,
    budget: &mut Budget,
) -> Result<DeclarationRefReport> {
    let (problems, environment_identity) = validate_environment(content, &query.environment);
    if !problems.is_empty() {
        return Ok(unavailable_declaration_reference_report(
            query,
            budget,
            problems,
            environment_identity,
            "declaration-reference resolution",
        ));
    }
    let Some(declaration_shape) = member_shape(&query.declaration.member) else {
        // The query answers member declarations: its candidate rule is a member shape, and a
        // class symbol names a type, not a member. No candidate rule is stated for one, so
        // the report says so instead of scanning with a shape that cannot be expressed.
        return Ok(unavailable_declaration_reference_report(
            query,
            budget,
            problems,
            environment_identity,
            "declaration-reference resolution of a class symbol",
        ));
    };
    // The entry point refuses a query whose environment names content the request does not
    // provide, so this is unreachable from `Engine::declaration_references`; a report built
    // by hand keeps the honest unavailable state instead of scanning an unknown snapshot.
    let Some(snapshot) = content
        .iter()
        .find(|candidate| candidate.id() == &query.environment.runtime.physical.snapshot)
    else {
        return Ok(unavailable_declaration_reference_report(
            query,
            budget,
            problems,
            environment_identity,
            "declaration-reference resolution",
        ));
    };

    // Which candidate shape this declaration can be found under. A signature-polymorphic
    // method is the one case where the site's own descriptor is not the declaration's — JVMS
    // 2.9 matches `MethodHandle.invoke`/`invokeExact` by name, because the call site chooses
    // the descriptor — so that shape compares owner and name, and every other member compares
    // name and descriptor. Selecting the wrong one is not a detail: the descriptor comparison
    // would answer "no candidate at all" for a call site that really resolves to this
    // declaration.
    let filter = match signature_polymorphic_shape(&query.declaration.member) {
        Some((owner, name)) => {
            jarde_query::xref::CandidateFilter::SignaturePolymorphic { owner, name }
        }
        None => jarde_query::xref::CandidateFilter::MemberShape {
            name: declaration_shape.0,
            descriptor: declaration_shape.1,
        },
    };
    let scan = jarde_query::xref::scan_candidates(
        snapshot,
        &query.scope,
        &query.consumers,
        filter,
        query.max_items,
        budget,
    )?;
    let mut diagnostics = scan.diagnostics;
    // The declaration query carries no caller: a use site's kind is the reference's, but the
    // class that declares the use site is the query's own attribute, not the reference's, so
    // the member rules run with the caller's class unknown — which is exactly the state 2.3
    // reports as `resolution_access_not_checked` for a member whose access depends on it.
    let caller = CallerContext {
        loader: query.environment.runtime.load_domain.loader.clone(),
        enclosing: None,
    };
    let mut closure = HeaderClosure::new(content, &query.environment);
    let mut items: Vec<DeclarationRefItem> = Vec::new();
    let mut unresolved_candidates = 0_u64;
    let mut stopped: Option<ExecutionReport> = None;
    let mut assembly: Option<ExecutionReport> = None;
    let mut hierarchies_complete = true;

    for item in &scan.items {
        // A shape filter admits member symbols only, and the scan of a symbolic relation
        // publishes consumer facts only (the raw pool candidate has no consumer and belongs to
        // the pool-probe relation), so neither guard can drop a candidate this engine's own
        // query path produced.
        let Some(referenced) = member_symbol(&item.target) else {
            continue;
        };
        let Some(consumer) = item.consumer else {
            continue;
        };
        // The decision phase: which entry this candidate contributes, the rule diagnostics the
        // search produced for it, and — when the search could not finish — the stop that ended
        // it. Nothing is published here; the entries are published below, each charged first.
        let mut rules: Vec<Diagnostic> = Vec::new();
        let mut resolution_stop: Option<Diagnostic> = None;
        let contribution = if stopped.is_some() {
            // The budget or cancellation already refused a resolution. It is a property of the
            // request, not of one candidate, so the candidates after it were not decided either
            // — the reliable prefix stays and nothing here counts as excluded.
            Some(Contribution::Undecided(
                STOPPED_BEFORE_THIS_CANDIDATE.to_string(),
            ))
        } else if let Some(use_kind) = member_use_of(item.operation, referenced) {
            let origin = origin_of(item);
            match crate::members::resolve_member(
                &mut closure,
                referenced,
                use_kind,
                &caller,
                budget,
            ) {
                Ok(outcome) => {
                    hierarchies_complete &= outcome.hierarchy_complete();
                    rules = outcome.diagnostics;
                    match &outcome.decision {
                        crate::members::MemberDecision::Resolved(location) => {
                            let resolved = resolved_member(location);
                            if same_declaration(&resolved, &query.declaration) {
                                Some(Contribution::Reference(Box::new(DeclarationRefItem {
                                    referenced: referenced.clone(),
                                    consumer,
                                    operation: item.operation,
                                    origin,
                                    resolved: Some(resolved),
                                    state: ResolutionState::Resolved,
                                })))
                            } else {
                                // Decided to be a reference to another declaration: neither a
                                // reference to this one nor undecided.
                                None
                            }
                        }
                        // The reason names the unread branches too: a candidate left undecided
                        // because a class of its hierarchy could not be read is not the same fact
                        // as one whose readable hierarchy declares no such member, and calling it
                        // the latter would be the negation A11 forbids (2.4's own plane).
                        decision => Some(Contribution::Undecided(undecided_reason(
                            decision,
                            &outcome.unread,
                        ))),
                    }
                }
                Err(error) => {
                    let (execution, diagnostic) = terminal(&error, budget.usage());
                    resolution_stop = Some(at_candidate(diagnostic, item));
                    stopped = Some(execution);
                    Some(Contribution::Undecided(
                        STOPPED_AT_THIS_CANDIDATE.to_string(),
                    ))
                }
            }
        } else {
            Some(Contribution::Undecided(
                NO_INSTRUCTION_KIND_IN_EVIDENCE.to_string(),
            ))
        };
        let Some(contribution) = contribution else {
            continue;
        };

        // The publish phase. Every entry this query puts into the report — an item, an undecided
        // diagnostic, or one of the rule diagnostics the search produced — costs one
        // `ResultItems`, charged before it enters the report. A refused charge keeps the entries
        // already published, ends the assembly and reports the stop, so the report is a prefix
        // that says so instead of an answer it could not afford. The stop itself is control
        // metadata and is published uncharged, even when the refused charge was the last unit of
        // the very dimension it names.
        let mut refused: Option<Diagnostic> = None;
        for rule in rules {
            match charge_and_publish(rule, &mut diagnostics, budget) {
                None => {}
                Some(stop) => {
                    refused = Some(at_candidate(stop.diagnostic, item));
                    assembly = Some(stop.execution);
                    break;
                }
            }
        }
        if refused.is_none()
            && let Some(stop) = resolution_stop
        {
            push_stop_diagnostic(&mut diagnostics, stop);
        }
        if refused.is_none() {
            match contribution {
                Contribution::Reference(entry) => match charge_entry(budget) {
                    None => items.push(*entry),
                    Some(stop) => {
                        refused = Some(at_candidate(stop.diagnostic, item));
                        assembly = Some(stop.execution);
                    }
                },
                Contribution::Undecided(reason) => {
                    let diagnostic = unresolved_candidate_diagnostic(item, referenced, &reason);
                    match charge_and_publish(diagnostic, &mut diagnostics, budget) {
                        None => unresolved_candidates += 1,
                        Some(stop) => {
                            refused = Some(at_candidate(stop.diagnostic, item));
                            assembly = Some(stop.execution);
                        }
                    }
                }
            }
        }
        if let Some(stop) = refused {
            push_stop_diagnostic(&mut diagnostics, stop);
            break;
        }
    }

    // The closure's own diagnostics (an unreadable or cyclic hierarchy branch) are resolution
    // results like any other, so each one is charged before it enters the report. They come
    // last, after every candidate, so a refused charge here only shortens the tail.
    for diagnostic in closure.diagnostics().to_vec() {
        match charge_and_publish(diagnostic, &mut diagnostics, budget) {
            None => {}
            Some(stop) => {
                // There is no candidate left to locate this stop at: a closure diagnostic is
                // about a hierarchy branch, and its own entry belongs to a read this report
                // already describes in `reads`.
                diagnostics.push(stop.diagnostic);
                assembly = Some(stop.execution);
                break;
            }
        }
    }

    // The resolution plane is complete only when every candidate reached a decision about a
    // declaration and every hierarchy it read was readable — and only when the scan that found
    // the candidates reached the end of its own range and the report really published everything
    // it found, because a scan or an assembly that stopped may hold candidates no resolution has
    // seen yet. Either stop makes the answer a prefix, which is what `has_more` says.
    let assembly_stopped = assembly.is_some();
    let extent = closure.searched_extent();
    let search = search_coverage(
        extent.examined,
        extent.positions,
        stopped.is_none() && !assembly_stopped,
        hierarchies_complete && unresolved_candidates == 0 && !scan.has_more,
    );
    let coverage = Coverage {
        artifact_structural: scan.coverage.dimensions.artifact_structural.clone(),
        runtime_resolution: search.runtime_resolution,
        dynamic_analysis: CoverageDimension::not_requested(),
    };
    // The scan ran first, so a stop of its own is the condition that limited this query; a
    // resolution stop governs the report when the scan completed, and an assembly stop — the
    // report being unable to publish what it found — only when neither of those happened.
    let execution = match (&scan.execution, stopped.or(assembly.clone())) {
        (ExecutionReport::Complete { .. }, Some(stop)) => stop,
        (scan_execution, _) => scan_execution.clone(),
    };
    let returned_items = u64::try_from(items.len()).map_err(|_| {
        Error::invalid_input(
            "query_size_overflow",
            "declaration-reference item count does not fit u64",
        )
    })?;
    Ok(DeclarationRefReport {
        environment_identity,
        environment_problems: problems,
        declaration: query.declaration.clone(),
        scope: query.scope.clone(),
        consumers: query.consumers.clone(),
        unsupported_categories: scan.coverage.unsupported_categories.clone(),
        analysis: ResolutionAnalysis::Performed,
        items,
        unresolved_candidates,
        // The scan's own truncation and an assembly that could not publish what it found both
        // leave the report a reliable prefix: each candidate behind either stop was neither
        // published nor counted as undecided.
        has_more: scan.has_more || assembly_stopped,
        returned_items,
        reads: published_reads(&closure),
        coverage,
        execution: with_usage(execution, budget.usage()),
        diagnostics,
    })
}

/// What one scanned candidate contributes to the report.
///
/// A candidate the search resolved to *another* declaration contributes nothing and is not one
/// of these: it is a decided candidate that is simply not a reference to the queried
/// declaration.
enum Contribution {
    /// The candidate really is a reference to the queried declaration.
    ///
    /// Boxed because an item carries a whole raw symbol, origin and resolution and this enum is
    /// handed around as one value, exactly like [`crate::members::MemberDecision`] boxes its
    /// location.
    Reference(Box<DeclarationRefItem>),
    /// The query could not decide whether the candidate is a reference, and says why. The reason
    /// is owned rather than static because it names the classes a member search could not read.
    Undecided(String),
}

/// Why a candidate found after a resolution stop stays undecided.
const STOPPED_BEFORE_THIS_CANDIDATE: &str =
    "the resolution stopped before this candidate was decided";
/// Why the candidate that hit the resolution stop stays undecided.
const STOPPED_AT_THIS_CANDIDATE: &str = "the resolution stopped at this candidate";

/// Why a candidate whose evidence carries no instruction-level member kind stays undecided.
const NO_INSTRUCTION_KIND_IN_EVIDENCE: &str =
    "this use site's instruction-level member kind is not part of its evidence";

/// One stop diagnostic, located at the candidate it stopped on.
fn at_candidate(mut diagnostic: Diagnostic, item: &XrefItem) -> Diagnostic {
    diagnostic.provenance = Some(item.source.clone());
    diagnostic
}

/// The stop a refused publication leaves behind.
struct PublicationStop {
    /// The terminal execution of the refused charge, reported as `Partial`/`Cancelled`/`Failed`.
    execution: ExecutionReport,
    /// The diagnostic that explains it; control metadata, published uncharged.
    diagnostic: Diagnostic,
}

/// Charges one entry of this report.
///
/// One item, one resolution diagnostic and one closure diagnostic each cost one `ResultItems`,
/// and the charge happens before the entry is published — the same discipline the P1 scan
/// applies to the items and domain diagnostics it returns. `None` means the charge was
/// accepted; `Some(stop)` means the budget refused it, which ends the assembly.
fn charge_entry(budget: &mut Budget) -> Option<PublicationStop> {
    match budget.charge(CountedBudgetDimension::ResultItems, 1) {
        Ok(()) => None,
        Err(error) => {
            let (execution, diagnostic) = terminal(&error, budget.usage());
            Some(PublicationStop {
                execution,
                diagnostic,
            })
        }
    }
}

/// Charges one resolution diagnostic and publishes it.
///
/// Every diagnostic this query produces — the rule diagnostics 2.3 returned for a declaration,
/// the undecided-candidate diagnostics, and the closure's own branch warnings — enters the
/// report the way a P1 item or domain diagnostic does: it costs one `ResultItems`, charged
/// before it is published. The environment plane and the diagnostics that explain a stop are
/// not charged here: they describe the request or the interruption rather than a result, and a
/// stop that could not pay for its own explanation would hide why the run ended.
///
/// `None` means the diagnostic was published. `Some(stop)` means the charge was refused: the
/// refused diagnostic is not published, and the caller publishes the stop uncharged where it
/// decides (at the candidate, or after the last one).
fn charge_and_publish(
    diagnostic: Diagnostic,
    diagnostics: &mut Vec<Diagnostic>,
    budget: &mut Budget,
) -> Option<PublicationStop> {
    match charge_entry(budget) {
        None => {
            diagnostics.push(diagnostic);
            None
        }
        Some(stop) => Some(stop),
    }
}

/// Queues one stop diagnostic, keeping the report free of an exact repeat.
///
/// A resolution stop and the publication of the same candidate can hit the same dimension with
/// the same counts at the same use site; the second copy adds no fact, and the report already
/// says that the run stopped, why, and where. Two *different* stops — another dimension, or
/// another candidate — are both kept.
fn push_stop_diagnostic(diagnostics: &mut Vec<Diagnostic>, diagnostic: Diagnostic) {
    if diagnostics.last() != Some(&diagnostic) {
        diagnostics.push(diagnostic);
    }
}

/// Diagnostic code of one candidate a declaration-reference query could not decide.
///
/// The candidate is not a reference to the declaration and it is not an excluded candidate
/// either. The report counts unresolved candidates but publishes no item for them, so this
/// diagnostic is where their physical use site stays locatable: its provenance is the
/// candidate's own `source`.
const CANDIDATE_UNRESOLVED: &str = "resolution_candidate_unresolved";

/// Diagnostic of one candidate the query could not decide, at the candidate's own use site.
fn unresolved_candidate_diagnostic(
    item: &XrefItem,
    referenced: &SymbolRef,
    reason: &str,
) -> Diagnostic {
    Diagnostic {
        code: CANDIDATE_UNRESOLVED.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: format!(
            "candidate use site {}::{} is unresolved ({reason}), so it is neither a reference to \
             the queried declaration nor an excluded candidate",
            member_owner(referenced),
            member_name(referenced)
        ),
        provenance: Some(item.source.clone()),
    }
}

/// Why one member search left a candidate undecided.
///
/// Every non-`Resolved` decision says the search reached no declaration it could compare with
/// the query's own; a candidate that resolved to a *different* declaration is not undecided
/// and never reaches this mapping.
///
/// The unread branches are part of the answer, and they are the reason this mapping owns its
/// text: a search that could not read a class of its hierarchy did not establish that nothing
/// declares the member, and the diagnostic that counts the candidate as undecided names the
/// classes it could not read instead of stating that negation (A11).
fn undecided_reason(decision: &crate::members::MemberDecision, unread: &WalkGaps) -> String {
    use crate::members::MemberDecision;
    let reason = match decision {
        MemberDecision::Resolved(_) => "the declaration this candidate resolves to",
        MemberDecision::Missing => "no class of the searched hierarchy declares this member",
        MemberDecision::OwnerAmbiguous(_) => {
            "the owner class cannot be told apart at its selection position"
        }
        MemberDecision::DeclarationAmbiguous(_) => {
            "the declaring class declares this member more than once"
        }
        MemberDecision::KindMismatch => "the reference kind contradicts the declaration",
        MemberDecision::DefaultConflict => "two or more interface defaults are maximally specific",
        MemberDecision::AccessDenied => "the member is not accessible to a known caller",
        MemberDecision::ArrayOwner => "the owner is an array type",
    };
    if unread.is_empty() {
        return reason.to_string();
    }
    let names = unread
        .iter()
        .map(|(_, gap)| escaped(&gap.name.0))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{reason}, and a class of its hierarchy could not be read ({names}), so this candidate is \
         undecided rather than excluded"
    )
}

/// The raw name and descriptor of a member declaration, or `None` for a class symbol.
fn member_shape(member: &SymbolRef) -> Option<(JvmBytes, JvmBytes)> {
    match member {
        SymbolRef::Field {
            name, descriptor, ..
        }
        | SymbolRef::Method {
            name, descriptor, ..
        } => Some((name.clone(), descriptor.clone())),
        SymbolRef::Class { .. } => None,
    }
}

/// The class that declares the two signature-polymorphic methods (JVMS 2.9).
const METHOD_HANDLE: &[u8] = b"java/lang/invoke/MethodHandle";
/// The name of the first signature-polymorphic method.
const INVOKE: &[u8] = b"invoke";
/// The name of the second signature-polymorphic method.
const INVOKE_EXACT: &[u8] = b"invokeExact";

/// The owner and name of a declaration whose call sites choose their own descriptor.
///
/// JVMS 2.9 defines exactly two methods this way, and only on `java/lang/invoke/MethodHandle`:
/// a call site of theirs names the method by name and picks the descriptor, so the declaration
/// is found without comparing descriptors at all. Answering `Some` here is what makes the scan
/// use [`jarde_query::xref::CandidateFilter::SignaturePolymorphic`] instead of a member shape. The
/// rule mirrors 2.3's own name-only branch, which the same owner and names select.
fn signature_polymorphic_shape(member: &SymbolRef) -> Option<(JvmBytes, JvmBytes)> {
    let SymbolRef::Method { owner, name, .. } = member else {
        return None;
    };
    let signature_polymorphic =
        owner.0 == METHOD_HANDLE && (name.0 == INVOKE || name.0 == INVOKE_EXACT);
    signature_polymorphic.then(|| (owner.clone(), name.clone()))
}

/// The member symbol one candidate item names, or `None` for a class or literal target.
fn member_symbol(target: &XrefTarget) -> Option<&SymbolRef> {
    match target {
        XrefTarget::Symbol { value } => match value {
            SymbolRef::Field { .. } | SymbolRef::Method { .. } => Some(value),
            SymbolRef::Class { .. } => None,
        },
        XrefTarget::Literal { .. } => None,
    }
}

/// The member use one candidate's own instruction states.
///
/// The use site is the only place the instruction-level kind lives, so the query maps the
/// operation of an item back onto the member rules instead of inventing one: the `invoke*`
/// family keeps its kind and a field access keeps its direction. `ldc` is the boundary case —
/// a method handle records its own reference kind in the class bytes, which this item's
/// evidence does not carry, so a method handle stays undecided while a *field* handle is
/// resolved as the read it is (the field rules do not read the use kind at all, so nothing is
/// invented by it). `invokedynamic` names a dynamic call site whose symbol carries no owner,
/// which the member search answers with `Missing`.
fn member_use_of(
    operation: XrefOperation,
    referenced: &SymbolRef,
) -> Option<crate::members::MemberUse> {
    use crate::members::MemberUse;
    match (referenced, operation) {
        (SymbolRef::Field { .. }, XrefOperation::GetField | XrefOperation::GetStatic) => {
            Some(MemberUse::FieldRead)
        }
        (SymbolRef::Field { .. }, XrefOperation::PutField | XrefOperation::PutStatic) => {
            Some(MemberUse::FieldWrite)
        }
        (SymbolRef::Field { .. }, XrefOperation::Ldc) => Some(MemberUse::FieldRead),
        (SymbolRef::Method { .. }, XrefOperation::InvokeVirtual) => Some(MemberUse::InvokeVirtual),
        (SymbolRef::Method { .. }, XrefOperation::InvokeSpecial) => Some(MemberUse::InvokeSpecial),
        (SymbolRef::Method { .. }, XrefOperation::InvokeStatic) => Some(MemberUse::InvokeStatic),
        (SymbolRef::Method { .. }, XrefOperation::InvokeInterface) => {
            Some(MemberUse::InvokeInterface)
        }
        (SymbolRef::Method { .. }, XrefOperation::InvokeDynamic) => Some(MemberUse::InvokeDynamic),
        _ => None,
    }
}

/// Whether one resolved declaration is the declaration the query names.
///
/// The three dimensions are the query's own: the loader the declaration was read from, the
/// physical definition it was read from, and the member symbol itself. The symbol's owner is
/// the declaring class's own internal name, so a `Sub.foo` reference that resolves to
/// `Base.foo` compares equal to a `Base.foo` declaration while the same name under another
/// loader or definition does not.
fn same_declaration(found: &ResolvedMemberRef, asked: &ResolvedMemberRef) -> bool {
    found.loader == asked.loader
        && found.definition == asked.definition
        && found.member == asked.member
}

/// Physical use site of one candidate item, in the shared origin vocabulary.
///
/// A class-file coordinate maps onto the origin member that says the same thing: the method
/// and BCI of an instruction are a `MethodPoint`, the range of an attribute fact is a
/// `ClassRange` of the class it was read from, and a fact the reader located by offset alone
/// keeps the range its evidence recorded (or the whole class, when it recorded none). The
/// container-relative locations (`Container`, `Entry`, `Resource`) carry no class-file
/// coordinate, and no member candidate is one, so they contribute no origin member.
fn origin_of(item: &XrefItem) -> OriginSet {
    let mut origin = OriginSet::default();
    match &item.source.location {
        Location::Code { method, bci } => origin.insert(OriginMember::MethodPoint {
            method: method.clone(),
            bci: *bci,
        }),
        Location::Attribute { owner, span, .. } => origin.insert(OriginMember::ClassRange {
            definition: owner.clone(),
            span: span.clone(),
        }),
        Location::ClassOffset { definition, .. } => match &item.evidence.span {
            Some(span) => origin.insert(OriginMember::ClassRange {
                definition: definition.clone(),
                span: span.clone(),
            }),
            None => origin.insert(OriginMember::ClassFile {
                definition: definition.clone(),
            }),
        },
        Location::Container { .. } | Location::Entry { .. } | Location::Resource { .. } => {}
    }
    origin
}

/// Owner bytes of one member symbol, for a message that must not invent one.
fn member_owner(symbol: &SymbolRef) -> String {
    match symbol {
        SymbolRef::Field { owner, .. } | SymbolRef::Method { owner, .. } => escaped(&owner.0),
        SymbolRef::Class { owner } => escaped(&owner.0),
    }
}

/// Name and descriptor of one member symbol, for a message that must not invent them.
fn member_name(symbol: &SymbolRef) -> String {
    match symbol {
        SymbolRef::Field {
            name, descriptor, ..
        }
        | SymbolRef::Method {
            name, descriptor, ..
        } => format!("{}:{}", escaped(&name.0), escaped(&descriptor.0)),
        SymbolRef::Class { .. } => "class".to_string(),
    }
}

/// Report of a declaration-reference query that performed nothing.
///
/// The state is the honest unavailable one: the request shape and its environment were
/// validated, no artifact byte was read, no candidate was found or excluded, and `coverage`
/// stays `not_requested`. It is the answer for a rejected environment and for a declaration
/// this query states no candidate rule for.
fn unavailable_declaration_reference_report(
    query: &DeclarationRefQuery,
    budget: &Budget,
    problems: Vec<EnvironmentProblem>,
    environment_identity: EnvironmentIdentity,
    capability: &str,
) -> DeclarationRefReport {
    let mut diagnostics = environment_diagnostics(&problems);
    diagnostics.push(unavailable_diagnostic(
        RESOLUTION_NOT_IMPLEMENTED,
        capability,
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
