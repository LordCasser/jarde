//! Header providers: class-name lookup, the effective search order and content identity.
//!
//! This module is the 2.1 slice of the demand resolver and it answers exactly one question:
//! under one explicit [`ResolutionEnvironment`], which physical definition does a class name
//! select, and what does that definition's header say. It implements no closure (2.2), no
//! member resolution (2.3) and no dispatch (2.5).
//!
//! The search order is declared, never guessed. [`LoadDomain::roots`] lists one loader's
//! readable positions in order, the parent chain of `runtime.load_domain` lists the loaders
//! that participate, and each domain's own delegation policy says whether that loader's roots
//! are searched before its parent's sequence (`ChildFirst`) or after it (`ParentFirst`). For a
//! chain that declares one policy this is exactly the contract's two sequences: `ParentFirst`
//! is the top ancestor down to the caller, `ChildFirst` the caller down to the top ancestor.
//!
//! Content is only what the entry provided. A root whose snapshot is absent, an `External`
//! root and an unsupported module mode or delegation policy are environment problems that
//! [`crate::environment::validate_environment`] reports, and the report layer starts no lookup
//! for such an environment; the functions here refuse them instead of building a shorter
//! sequence, so a search that reaches one is a refusal and never `Missing`.
//!
//! `Ok` is a decision and `Err` is a stop, and the report layer maps them to the planes the
//! design fixes: a decision becomes `state = Some(..)` (a found, missing or ambiguous name, or
//! the budget stop in [`Error::BudgetExceeded`]), while a cancellation and a damaged candidate
//! or stopped listing are the `Err` of a run that ended before deciding, so they report
//! `state = None` with `execution` carrying the stop.
//!
//! Three rules keep the lookup honest under bounds:
//!
//! * a listing that stopped early (budget, cancellation, damage) cannot decide a position —
//!   the candidate set is unknown — so the lookup propagates the listing's own refusal instead
//!   of reading the prefix as "this root has no such name",
//! * the first position that holds a candidate decides the lookup: a damaged candidate is
//!   reported at its own origin and the search does not continue to a later root,
//! * several candidates at one position are `Ambiguous` and keep their own origins; equal
//!   bytes never merge two origins, and a byte-equal duplicate is not ordered by ordinal
//!   either.
//!
//! The closure slice (2.2) organizes those single lookups into one request-scoped machine.
//! [`HeaderClosure`] demands a class header per `(loader, internal name)`, answers a repeated
//! demand from its own request memo, records every header the request really read together with
//! the demand that read it, and [`HierarchyWalk`] expands the superclass/interface graph one
//! layer at a time under the dependency-depth and worklist budgets. A closure reads headers
//! only: a method body stays untouched until a later slice asks for one with an explicit
//! reason, and this machine has no body path at all.

use crate::artifact::{ArtifactKind, ArtifactSnapshot, PhysicalEntry};
use crate::budget::{Budget, BudgetDimension, CountedBudgetDimension, Limits, UsageSnapshot};
use crate::classfile::{ClassFacts, class_facts};
use crate::environment::{EnvironmentProblemCode, ResolutionEnvironment};
use crate::error::{Error, Result};
use crate::model::{
    ClassBytesId, ContainerOrigin, Diagnostic, DiagnosticSeverity, Digest, ExecutionReport,
    JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, PhysicalEntryId, PhysicalVariant,
    SnapshotId, TerminationReason, physical_variant_for_path,
};
use crate::view::{DelegationPolicy, LoadDomain, LoadRoot, LoaderId, ModuleMode};
use std::collections::VecDeque;

/// Suffix every archive entry of a class carries; the comparison is byte-exact.
const CLASS_SUFFIX: &[u8] = b".class";

/// Diagnostic code of a cyclic hierarchy, shared by the closure walk and the member search.
pub(crate) const HIERARCHY_CYCLE: &str = "resolution_hierarchy_cycle";

/// One physical position with the definition it selected or read.
///
/// The position is a declaration coordinate: the loader that owns the root, the root's index
/// in that loader's `roots`, and the definition the position produced. A lookup publishes the
/// one position that decided it ([`HeaderLookup`]); a read publishes the position of every
/// header it really read, in read order ([`HeaderSearch::reads`]), which is what the closure
/// records per request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HeaderLocation {
    pub(crate) loader: LoaderId,
    /// Declaration index of the root inside its loader's `roots`.
    pub(crate) root_index: u32,
    pub(crate) definition: PhysicalDefinitionId,
    /// Entry the definition was read from; `None` for a standalone CLASS root.
    ///
    /// The closure slice (2.2) reads it to record read reasons and origins; the lookup itself
    /// only publishes it.
    #[allow(dead_code)]
    pub(crate) entry: Option<PhysicalEntryId>,
}

/// What one class-name lookup decided.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HeaderLookupState {
    Found,
    Missing,
    Ambiguous,
}

/// Result of one class-name lookup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HeaderLookup {
    pub(crate) state: HeaderLookupState,
    /// The selected position: `Some` exactly when the state is `Found`.
    pub(crate) location: Option<HeaderLocation>,
    /// Positions that cannot be told apart: non-empty exactly when the state is `Ambiguous`.
    pub(crate) candidates: Vec<HeaderLocation>,
    /// Header of the selected definition; `Some` exactly when the state is `Found`.
    ///
    /// The closure slice (2.2) expands supertypes and interfaces from it, which is why the
    /// lookup publishes it together with the position it selected; the name lookup itself is
    /// decided by the position and needs no fact from the header.
    #[allow(dead_code)]
    pub(crate) header: Option<ClassHeaderFacts>,
}

impl HeaderLookup {
    fn found(location: HeaderLocation, header: ClassHeaderFacts) -> Self {
        Self {
            state: HeaderLookupState::Found,
            location: Some(location),
            candidates: Vec::new(),
            header: Some(header),
        }
    }

    /// No position held the name. An empty search order is the same fact: the environment
    /// declares no readable position for this name.
    fn missing() -> Self {
        Self {
            state: HeaderLookupState::Missing,
            location: None,
            candidates: Vec::new(),
            header: None,
        }
    }

    fn ambiguous(candidates: Vec<HeaderLocation>) -> Self {
        Self {
            state: HeaderLookupState::Ambiguous,
            location: None,
            candidates,
            header: None,
        }
    }
}

/// Minimal wrapper of the existing reader facts for one definition's header.
///
/// It adds no field and no public type: it is the reader's [`ClassFacts`] bundle
/// (`major/minor/access_flags/this_class/super_class/interfaces/member headers/constant pool`)
/// under the name the 2.1 contract fixes for header lookup results. The lookup fills it for
/// every `Found` result; the closure slice (2.2) is its reader, and the standalone name
/// comparison already reads `this_class` through the bundle it wraps.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClassHeaderFacts {
    pub(crate) facts: ClassFacts,
}

/// One lookup attempt plus the extent of the effective order that produced it.
///
/// The 2.1 contract fixes the lookup itself ([`HeaderLookup`]); the extent is what only the
/// search knows and the report layer publishes as the resolution coverage plane. It is filled
/// in every path, including a refusal, because a stopped search still has to declare the
/// positions it examined and the positions it never reached.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HeaderSearch {
    /// `Ok` when one position decided the lookup, `Err` when the search stopped.
    ///
    /// The refusals are the existing ones: a budget stop stays [`Error::BudgetExceeded`] with
    /// its dimension and usage, a cancellation stays [`Error::Cancelled`], and a structural
    /// failure keeps the reader's or listing's own code together with the origin of the
    /// position it was found at.
    pub(crate) lookup: Result<HeaderLookup>,
    /// Positions examined to a decision; a position that refused the search is not one of
    /// them, and neither is a position the stop never reached.
    pub(crate) examined: u32,
    /// Positions the effective order declares in total, `0` when the order itself is refused.
    pub(crate) positions: u32,
    /// Every header this search really read, in read order: the position of each successful
    /// read, whether or not that position decided the lookup (a standalone root that declares
    /// another name is read and recorded), and whether or not the search concluded (a read
    /// that happened before a later position refused the search still happened).
    ///
    /// A failed attempt appends nothing: bytes that are not a class, a read-layer failure and
    /// a refused charge produce no definition identity, and their evidence is the diagnostic
    /// that names them. Each recorded read is one charged `ClassHeaders` attempt.
    pub(crate) reads: Vec<HeaderLocation>,
}

impl HeaderSearch {
    /// A refusal that no position was even derived for: the order itself is unusable.
    fn refused(error: Error) -> Self {
        Self {
            lookup: Err(error),
            examined: 0,
            positions: 0,
            reads: Vec::new(),
        }
    }
}

/// Looks one raw class internal name up in the effective search order.
///
/// The name is compared as raw bytes: no normalization, no case folding, no descriptor
/// interpretation. Every header read attempt costs one `ClassHeaders`; the listing and entry
/// reads keep their own P1 accounting.
///
/// The order starts at `environment.runtime.load_domain`, which 1.1 fixes as the caller's own
/// domain: that declaration is what the validator binds to a unique, equal `domains` entry. A
/// `CallerContext` that names a different loader does not move the starting point, and 1.1's
/// closed problem set has no code for the two disagreeing.
pub(crate) fn lookup_class_header(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    internal_name: &[u8],
    budget: &mut Budget,
) -> HeaderSearch {
    let domains = match ordered_domains(environment) {
        Ok(domains) => domains,
        Err(error) => return HeaderSearch::refused(error),
    };
    let positions = search_positions(&domains);
    let total = u32::try_from(positions.len()).unwrap_or(u32::MAX);
    let expected = entry_name(internal_name);

    let mut reads = Vec::new();
    let mut examined = 0_u32;
    for position in &positions {
        let decided = match probe_root(
            content,
            position,
            internal_name,
            &expected,
            budget,
            &mut reads,
        ) {
            Ok(decided) => decided,
            Err(error) => {
                return HeaderSearch {
                    lookup: Err(error),
                    examined,
                    positions: total,
                    reads,
                };
            }
        };
        examined = examined.saturating_add(1);
        if let Some(lookup) = decided {
            return HeaderSearch {
                lookup: Ok(lookup),
                examined,
                positions: total,
                reads,
            };
        }
    }
    HeaderSearch {
        lookup: Ok(HeaderLookup::missing()),
        examined,
        positions: total,
        reads,
    }
}

/// Why one class header was demanded inside one request.
///
/// This is the crate-private side of the report's public read reason: the closure owns the
/// demands it serves, and the report layer maps each one onto the public vocabulary it
/// publishes. The dependency direction is `resolver -> providers`, so the public enum cannot
/// live here.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum HeaderDemand {
    /// The request target itself.
    RequestedDefinition,
    /// A class reached along a `super_class` edge of the hierarchy being searched, and the top
    /// of such a chain when the walk itself reads it.
    ParentChain,
    /// A class reached along an `interfaces`/superinterface edge of the hierarchy being
    /// searched: the interface graph, never the class chain.
    HierarchyClosure,
    /// A class the request names by identity and reads directly, instead of reaching it along a
    /// hierarchy edge: the class a member reference names as its owner, and the class that
    /// declares the use site's enclosing method.
    ///
    /// The second one is the only class a request reads by physical definition rather than by
    /// name — the request carries it as a definition, and its name is what reading the header
    /// finds out — so [`HeaderClosure::read_definition`] serves it.
    MemberOwner,
    /// One class of an explicitly scoped candidate enumeration (2.5).
    ///
    /// The enumeration names the classes the requested range covers — for the class itself and
    /// as the starting point of its supertype walk — and every header it reads is one class of
    /// that range.
    DispatchScope,
}

/// One header this request read, with the demand that read it.
///
/// A binding is recorded once per request: the memo makes a second *read* of one
/// `(loader, internal name)` impossible, and a class that a later demand reaches for another
/// reason is the same read, so the record keeps the reason of the demand that performed it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HeaderReadRecord {
    pub(crate) loader: LoaderId,
    pub(crate) definition: PhysicalDefinitionId,
    pub(crate) demand: HeaderDemand,
}

/// Handle of one class inside one request's closure; stable for the request's lifetime.
///
/// A handle is only produced by the closure that resolved it, and [`HeaderClosure::resolution`]
/// panics for one from another request: a handle is a request-local coordinate, not an
/// identity. The identity of a class is its `(loader, definition)` binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct ClassHandle(usize);

/// What one demand resolved for one class name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClassResolution {
    /// The loader whose declared order searched for this name.
    pub(crate) loader: LoaderId,
    /// The raw internal name that was demanded.
    pub(crate) name: JvmBytes,
    /// The 2.1 lookup decision: state, selected position, candidates and header facts.
    pub(crate) lookup: HeaderLookup,
}

/// How far class-name searches reached, in declared positions and examined ones.
///
/// It is the extent of one demand's own search where a caller reads it out of a
/// [`HeaderSearch`], and the sum of every search a request really ran where
/// [`HeaderClosure::searched_extent`] publishes it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SearchExtent {
    /// Positions examined to a decision.
    pub(crate) examined: u32,
    /// Positions the effective order declares in total.
    pub(crate) positions: u32,
}

/// Answer of one demand.
#[derive(Clone, Debug)]
pub(crate) struct DemandAnswer {
    /// The class this demand decided, or the stop that ended it before deciding anything.
    pub(crate) decision: Result<ClassHandle>,
}

/// One request-scoped closure over class headers.
///
/// Every class a request needs — the target itself, one step up a `super_class` chain, or a
/// whole superclass/interface closure — is demanded here, and a repeated demand is answered
/// from the request memo. The memo key is `(loader, internal name)`: the second demand of one
/// key is not read again, not charged again and appends no second read record, so one
/// `(definition, loader)` binding is read at most once per request while the same bytes under
/// two loaders or two origins stay two bindings.
///
/// The memo holds the *decisions* a demand reached: a `Missing` is a fact about the declared
/// order, not a retryable miss, and an `Ambiguous` position stays ambiguous. A demand that
/// stopped before deciding (budget, cancellation, damaged bytes) decided nothing and is not
/// remembered, so asking again searches and stops again instead of inventing an answer.
///
/// A closure reads headers only. It has no method-body path at all: upgrading a body needs the
/// explicit `DriverMethodBody` demand of the analysis slices, and no closure demand may read
/// one.
pub(crate) struct HeaderClosure<'a> {
    content: &'a [ArtifactSnapshot],
    environment: &'a ResolutionEnvironment,
    /// One entry per decided `(loader, internal name)` key, in first-demand order.
    resolutions: Vec<ClassResolution>,
    reads: Vec<HeaderReadRecord>,
    diagnostics: Vec<Diagnostic>,
    /// Positions examined and declared by every class-name search this request really ran.
    searched: (u64, u64),
}

impl<'a> HeaderClosure<'a> {
    /// A closure over one request's own environment and provided content.
    pub(crate) fn new(
        content: &'a [ArtifactSnapshot],
        environment: &'a ResolutionEnvironment,
    ) -> Self {
        Self {
            content,
            environment,
            resolutions: Vec::new(),
            reads: Vec::new(),
            diagnostics: Vec::new(),
            searched: (0, 0),
        }
    }

    /// Demands one class header for this request.
    ///
    /// Every demand of one request searches the same way the 2.1 lookup does: it starts at the
    /// caller's own domain (`environment.runtime.load_domain`), the loader 1.1 binds to the
    /// unique equal `domains` entry and rejects a `CallerContext` that names another one, so a
    /// request has exactly one search start and the memo's loader component names it.
    ///
    /// A key the request has not decided yet is searched by [`lookup_class_header`], the 2.1
    /// lookup; every header that search read is recorded under this demand's reason, including
    /// the headers read before a later position refused the search. A key the request already
    /// decided is answered from the memo with the same handle: no second search runs, so the
    /// closure's own totals stay where the first search left them.
    pub(crate) fn demand(
        &mut self,
        name: &[u8],
        demand: HeaderDemand,
        budget: &mut Budget,
    ) -> DemandAnswer {
        let loader = self.environment.runtime.load_domain.loader.clone();
        if let Some(index) = self.remembered(&loader, name) {
            return DemandAnswer {
                decision: Ok(ClassHandle(index)),
            };
        }
        let search = lookup_class_header(self.content, self.environment, name, budget);
        for location in &search.reads {
            self.record_read(&location.loader, &location.definition, demand);
        }
        self.searched.0 = self.searched.0.saturating_add(u64::from(search.examined));
        self.searched.1 = self.searched.1.saturating_add(u64::from(search.positions));
        match search.lookup {
            Ok(lookup) => {
                self.resolutions.push(ClassResolution {
                    loader,
                    name: JvmBytes(name.to_vec()),
                    lookup,
                });
                DemandAnswer {
                    decision: Ok(ClassHandle(self.resolutions.len() - 1)),
                }
            }
            Err(error) => DemandAnswer {
                decision: Err(error),
            },
        }
    }

    /// The resolution one handle names.
    pub(crate) fn resolution(&self, handle: ClassHandle) -> &ClassResolution {
        &self.resolutions[handle.0]
    }

    /// The index of an already decided key, if the request remembers one.
    fn remembered(&self, loader: &LoaderId, name: &[u8]) -> Option<usize> {
        self.resolutions
            .iter()
            .position(|resolution| &resolution.loader == loader && resolution.name.0 == name)
    }

    /// Records one header this request read, under the demand that caused the read.
    fn record_read(
        &mut self,
        loader: &LoaderId,
        definition: &PhysicalDefinitionId,
        demand: HeaderDemand,
    ) {
        let recorded = self
            .reads
            .iter()
            .any(|read| &read.loader == loader && &read.definition == definition);
        if !recorded {
            self.reads.push(HeaderReadRecord {
                loader: loader.clone(),
                definition: definition.clone(),
                demand,
            });
        }
    }

    /// Every header this request read, in the order the reads happened.
    pub(crate) fn reads(&self) -> &[HeaderReadRecord] {
        &self.reads
    }

    /// Reads one class header the request names by identity instead of by name.
    ///
    /// The member slice (2.3) needs the class that declares the use site's enclosing method.
    /// A request carries that class as a physical definition, never as a name — the name is
    /// what reading the header finds out — so no class-name search can be its entry point.
    /// The read is one header attempt like any other (charged before the bytes are read and
    /// recorded under the demand that needed it), and a definition this request already read
    /// is answered from the memo: the same `(definition, loader)` binding is charged once per
    /// request, exactly like a repeated name.
    pub(crate) fn read_definition(
        &mut self,
        loader: &LoaderId,
        definition: &PhysicalDefinitionId,
        demand: HeaderDemand,
        budget: &mut Budget,
    ) -> Result<ClassHeaderFacts> {
        if let Some(index) = self.read_by_definition(loader, definition) {
            return Ok(self
                .resolutions
                .get(index)
                .and_then(|resolution| resolution.lookup.header.clone())
                .expect("a remembered definition published the header it was read for"));
        }
        let facts = read_definition_header(self.content, definition, budget)?;
        self.record_read(loader, definition, demand);
        Ok(facts)
    }

    /// The index of a decided lookup that selected one `(loader, definition)` binding.
    fn read_by_definition(
        &self,
        loader: &LoaderId,
        definition: &PhysicalDefinitionId,
    ) -> Option<usize> {
        self.resolutions.iter().position(|resolution| {
            &resolution.loader == loader
                && resolution
                    .lookup
                    .location
                    .as_ref()
                    .is_some_and(|location| &location.definition == definition)
        })
    }

    /// How far the class-name searches of this request reached, in total.
    ///
    /// The 2.1 lookup publishes the extent of its one search; a request that reads a whole
    /// hierarchy runs one search per class it reads, so the resolution plane publishes their
    /// sum. Every demand searches the same declared order from position 0, so the sum counts
    /// examined positions across searches, not distinct positions.
    pub(crate) fn searched_extent(&self) -> SearchExtent {
        SearchExtent {
            examined: u32::try_from(self.searched.0).unwrap_or(u32::MAX),
            positions: u32::try_from(self.searched.1).unwrap_or(u32::MAX),
        }
    }

    /// Diagnostics this request itself produced, in generation order.
    ///
    /// The report layer owns the published diagnostic list, so a walk that refuses an edge
    /// hands its diagnostic here instead of dropping it. The class-symbol path produces none.
    pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Records one diagnostic this request produced.
    ///
    /// Consumed by the walks below, which are reached by 2.3/2.5 rather than by the class
    /// symbol path of this slice.
    #[allow(dead_code)]
    pub(crate) fn record_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Walks the `super_class` chain of `root`, one class per layer.
    ///
    /// Every layer is a `super_class` edge, so every layer is demanded with
    /// [`HeaderDemand::ParentChain`]. `root_reason` is the demand of the root layer itself: the
    /// caller states why it needed that class (`MemberOwner` for the class a member rule names),
    /// because the root has no incoming edge to derive it from.
    #[allow(dead_code)]
    pub(crate) fn parent_chain(&self, root: &[u8], root_reason: HeaderDemand) -> HierarchyWalk {
        HierarchyWalk::new(root, WalkEdges::ParentChain, root_reason)
    }

    /// Walks every superclass and interface of `root`, breadth-first.
    ///
    /// The reason of each layer above the root follows the edge that reached it: the
    /// `super_class` edge is [`HeaderDemand::ParentChain`] and each `interfaces` edge is
    /// [`HeaderDemand::HierarchyClosure`], so one walk that follows both edges publishes both
    /// reasons instead of labelling its whole expansion with one of them. `root_reason` is the
    /// demand of the root layer itself, which no edge reached.
    #[allow(dead_code)]
    pub(crate) fn hierarchy_closure(
        &self,
        root: &[u8],
        root_reason: HeaderDemand,
    ) -> HierarchyWalk {
        HierarchyWalk::new(root, WalkEdges::SupertypeClosure, root_reason)
    }
}

/// Which supertype edges one walk follows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WalkEdges {
    /// `super_class` only: the linear chain of ancestors.
    ParentChain,
    /// `super_class` and `interfaces`: the whole supertype closure.
    SupertypeClosure,
}

/// One layer waiting to be expanded: the name to demand, the reason that reached it and the
/// path that reached it.
#[allow(dead_code)]
struct PendingLayer {
    name: JvmBytes,
    /// Why this layer is read: the demand of the edge that queued it, or the walk's own root
    /// reason for the layer the walk starts from.
    demand: HeaderDemand,
    /// The classes already on the supertype path that reached this layer. The length is the
    /// layer's dependency depth — the root has an empty path, because the root is the starting
    /// point of the walk and not a step up.
    ancestors: Vec<JvmBytes>,
}

/// What one walk could not expand, and why.
///
/// A caller reports these as the unfinished part of the closure's coverage: a missing or
/// ambiguous supertype is an open-world fact that ends its own branch, and a name that repeats
/// on one supertype path is an illegal hierarchy the walk refuses to follow.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct WalkGaps {
    /// Names no position of the order holds.
    pub(crate) missing: Vec<JvmBytes>,
    /// Names one position holds but cannot tell apart.
    pub(crate) ambiguous: Vec<JvmBytes>,
    /// Names that reached themselves: cyclic supertypes, refused instead of expanded.
    pub(crate) cycles: Vec<JvmBytes>,
}

/// One class-graph walk over one request's closure, resumed one layer at a time.
///
/// The walk keeps its own visited set, so a diamond-shaped or cyclic hierarchy terminates:
/// every class is expanded at most once per walk, while the request memo keeps the underlying
/// read at most once per `(loader, internal name)` for the whole request. The first layer is
/// the root the walk was created for, so a caller that already demanded the root gets it back
/// from the memo without a second read.
///
/// Every layer above the root is demanded with the reason of the edge that reached it: a
/// `super_class` edge is [`HeaderDemand::ParentChain`] and an `interfaces` edge is
/// [`HeaderDemand::HierarchyClosure`], so a walk that follows both edges publishes both reasons
/// instead of labelling its whole expansion with one of them. The root layer has no incoming
/// edge, so the caller states its reason when it creates the walk.
///
/// Each layer above the root calls [`Budget::observe_dependency_depth`] **before** the layer is
/// demanded — a depth stop therefore leaves the layers already returned as the trustworthy
/// prefix and reads none of the rest — and each layer charges one `AnalysisSteps` before it is
/// processed. A missing or ambiguous supertype ends its own branch and is recorded in
/// [`HierarchyWalk::gaps`]; a budget stop or a cancellation ends the walk with the stop's own
/// refusal, never as `Missing` and never as an empty closure.
#[allow(dead_code)]
pub(crate) struct HierarchyWalk {
    edges: WalkEdges,
    /// Reason of the layer the walk starts from, which no edge reached.
    root_reason: HeaderDemand,
    pending: VecDeque<PendingLayer>,
    /// Names this walk already expanded, so a shared supertype is not expanded twice.
    visited: Vec<JvmBytes>,
    gaps: WalkGaps,
    /// The stop that ended this walk: a stopped walk reports the same stop again instead of
    /// claiming that it is complete.
    stop: Option<Error>,
}

///
/// Consumed by 2.3/2.5, which ask the request for a hierarchy instead of one declaration; the
/// class-symbol path of this slice demands its target only, and this module's tests pin the
/// walk's semantics until those slices reach it.
#[allow(dead_code)]
impl HierarchyWalk {
    /// A walk with its root layer pending.
    fn new(root: &[u8], edges: WalkEdges, root_reason: HeaderDemand) -> Self {
        Self {
            edges,
            root_reason,
            pending: VecDeque::from([PendingLayer {
                name: JvmBytes(root.to_vec()),
                demand: root_reason,
                ancestors: Vec::new(),
            }]),
            visited: Vec::new(),
            gaps: WalkGaps::default(),
            stop: None,
        }
    }

    /// Expands the next layer.
    ///
    /// `Ok(Some(handle))` is the layer this call demanded and decided — its state may be
    /// `Found`, `Missing` or `Ambiguous`, and the caller reads it from
    /// [`HeaderClosure::resolution`]. `Ok(None)` means nothing is left to expand. `Err` is a
    /// stop before the next layer was expanded: a budget stop (`Partial`), a cancellation or a
    /// refused position, with every layer already returned kept as the trustworthy prefix. A
    /// stopped walk stays stopped.
    pub(crate) fn next(
        &mut self,
        closure: &mut HeaderClosure<'_>,
        budget: &mut Budget,
    ) -> Result<Option<ClassHandle>> {
        if let Some(stop) = &self.stop {
            return Err(stop.clone());
        }
        let expanded = self.expand_next(closure, budget);
        if let Err(error) = &expanded {
            // The budget or cancellation that ended this walk cannot be spent again, and
            // resuming would read a layer the stop already refused.
            self.stop = Some(error.clone());
            self.pending.clear();
        }
        expanded
    }

    /// What this walk could not expand.
    pub(crate) fn gaps(&self) -> &WalkGaps {
        &self.gaps
    }

    fn expand_next(
        &mut self,
        closure: &mut HeaderClosure<'_>,
        budget: &mut Budget,
    ) -> Result<Option<ClassHandle>> {
        let Some(layer) = self.pending.pop_front() else {
            return Ok(None);
        };
        if !layer.ancestors.is_empty() {
            let depth = u64::try_from(layer.ancestors.len()).unwrap_or(u64::MAX);
            budget.observe_dependency_depth(depth)?;
        }
        budget.charge(CountedBudgetDimension::AnalysisSteps, 1)?;
        let handle = closure
            .demand(&layer.name.0, layer.demand, budget)
            .decision?;
        self.visited.push(layer.name.clone());
        match closure.resolution(handle).lookup.state {
            HeaderLookupState::Found => self.queue_supertypes(closure, handle, &layer),
            HeaderLookupState::Missing => self.gaps.missing.push(layer.name.clone()),
            HeaderLookupState::Ambiguous => self.gaps.ambiguous.push(layer.name.clone()),
        }
        Ok(Some(handle))
    }

    /// Queues the supertypes one found class declares, one dependency step deeper.
    fn queue_supertypes(
        &mut self,
        closure: &mut HeaderClosure<'_>,
        handle: ClassHandle,
        layer: &PendingLayer,
    ) {
        let loader = closure.resolution(handle).loader.clone();
        for edge in supertype_edges(closure.resolution(handle), self.edges) {
            let child = edge.name;
            if layer.name == child || layer.ancestors.contains(&child) {
                // `A -> B -> A`: the name is already on the path that reached it, so this
                // hierarchy is cyclic (illegal) and expanding it would never terminate.
                self.gaps.cycles.push(child.clone());
                closure.record_diagnostic(cycle_diagnostic(&loader, layer, &child));
                continue;
            }
            if self.visited.contains(&child)
                || self.pending.iter().any(|pending| pending.name == child)
            {
                // A shared supertype (a diamond): already expanded or already queued by this
                // walk, and expanding it twice would read nothing new.
                continue;
            }
            let mut ancestors = layer.ancestors.clone();
            ancestors.push(layer.name.clone());
            self.pending.push_back(PendingLayer {
                name: child,
                demand: edge.demand,
                ancestors,
            });
        }
    }
}

/// One supertype edge of a class, with the demand that reading its target stands for.
///
/// The reason follows the edge: a `super_class` edge is a parent-chain step and an `interfaces`
/// edge is an interface-closure step, which is what keeps the two read reasons of the report
/// apart for a walk that follows both.
#[allow(dead_code)]
struct SupertypeEdge {
    name: JvmBytes,
    demand: HeaderDemand,
}

/// The supertype edges one decision declares, in expansion order.
///
/// The superclass is queued before the interfaces, so both walks visit the class chain of a
/// layer before its interface fan-out. A decision that is not `Found` declares nothing.
///
/// Consumed by [`HierarchyWalk`] (2.3/2.5).
#[allow(dead_code)]
fn supertype_edges(resolution: &ClassResolution, edges: WalkEdges) -> Vec<SupertypeEdge> {
    let Some(header) = resolution.lookup.header.as_ref() else {
        return Vec::new();
    };
    let mut edges_out = Vec::new();
    if let Some(super_class) = &header.facts.super_class {
        edges_out.push(SupertypeEdge {
            name: JvmBytes(super_class.raw().0.clone()),
            demand: HeaderDemand::ParentChain,
        });
    }
    if edges == WalkEdges::SupertypeClosure {
        for interface in &header.facts.interfaces {
            edges_out.push(SupertypeEdge {
                name: JvmBytes(interface.raw().0.clone()),
                demand: HeaderDemand::HierarchyClosure,
            });
        }
    }
    edges_out
}

/// The diagnostic of a cyclic supertype edge.
///
/// A warning, not an error: the walk refuses one edge and keeps every layer it already
/// expanded, so the report names the illegal hierarchy without turning the request into a
/// failure. The message carries the loader and the whole path, so the cycle can be located.
///
/// Consumed by [`HierarchyWalk`] (2.3/2.5).
#[allow(dead_code)]
fn cycle_diagnostic(loader: &LoaderId, layer: &PendingLayer, repeated: &JvmBytes) -> Diagnostic {
    let mut path = layer
        .ancestors
        .iter()
        .map(|name| escaped(&name.0))
        .collect::<Vec<_>>();
    path.push(escaped(&layer.name.0));
    path.push(escaped(&repeated.0));
    Diagnostic {
        code: HIERARCHY_CYCLE.to_string(),
        severity: DiagnosticSeverity::Warning,
        message: format!(
            "loader `{}`: `{}` is its own supertype ({}); a cyclic hierarchy is illegal, so this \
             edge is refused instead of expanded",
            loader.0,
            escaped(&repeated.0),
            path.join(" -> ")
        ),
        provenance: None,
    }
}

/// One readable position of the effective order.
struct SearchPosition<'a> {
    loader: &'a LoaderId,
    root_index: u32,
    root: &'a LoadRoot,
}

/// The participating domains, caller first.
///
/// The walk refuses what it cannot order instead of shortening the sequence: a loader without
/// a domain is a missing parent, a re-entered loader is a parent cycle, and a module mode or
/// delegation policy this slice cannot execute keeps its `UnsupportedPolicy` meaning. All of
/// them are environment problems, so the report layer never starts a lookup for such an
/// environment and a refusal here can only mean that a caller ignored the validator.
fn ordered_domains(environment: &ResolutionEnvironment) -> Result<Vec<&LoadDomain>> {
    let mut chain: Vec<&LoadDomain> = Vec::new();
    let mut loader = &environment.runtime.load_domain.loader;
    loop {
        if let Some(repeated) = chain.iter().find(|domain| &domain.loader == loader) {
            return Err(problem_error(
                EnvironmentProblemCode::ParentCycle,
                format!(
                    "parent chain of loader `{}` re-enters `{}`; a cyclic order has no first \
                     position",
                    environment.runtime.load_domain.loader.0, repeated.loader.0
                ),
            ));
        }
        let Some(domain) = domain_for(environment, loader) else {
            return Err(problem_error(
                EnvironmentProblemCode::MissingParent,
                format!(
                    "loader `{}` has no domain in `domains`; the parent chain cannot be ordered",
                    loader.0
                ),
            ));
        };
        match &domain.module_mode {
            ModuleMode::ClassPath => {}
            ModuleMode::ModulePath => {
                return Err(unsupported_policy(domain, "module mode `module_path`"));
            }
            ModuleMode::Hybrid => return Err(unsupported_policy(domain, "module mode `hybrid`")),
            ModuleMode::Custom { id } => {
                return Err(unsupported_policy(
                    domain,
                    &format!("module mode `custom` ({id})"),
                ));
            }
            ModuleMode::Unknown => return Err(unsupported_policy(domain, "unknown module mode")),
        }
        chain.push(domain);
        match &domain.parent_loader {
            Some(parent) => loader = parent,
            None => break,
        }
    }
    // Each domain decides where its own roots sit: a `ChildFirst` loader is searched before its
    // parent's sequence, a `ParentFirst` loader after it.
    let mut ordered: Vec<&LoadDomain> = Vec::with_capacity(chain.len());
    for domain in chain.iter().rev() {
        match &domain.delegation {
            DelegationPolicy::ParentFirst => ordered.push(domain),
            DelegationPolicy::ChildFirst => ordered.insert(0, domain),
            DelegationPolicy::Custom { id } => {
                return Err(unsupported_policy(
                    domain,
                    &format!("custom delegation policy `{id}`"),
                ));
            }
            DelegationPolicy::Unknown => {
                return Err(unsupported_policy(domain, "unknown delegation policy"));
            }
        }
    }
    Ok(ordered)
}

/// Every root of every participating loader, in effective order.
///
/// A loader contributes its `roots` in declaration order, and nothing else: a provider is a
/// name for content, not a second order. A domain without roots contributes no position.
fn search_positions<'a>(domains: &[&'a LoadDomain]) -> Vec<SearchPosition<'a>> {
    let mut positions = Vec::new();
    for domain in domains {
        for (index, root) in domain.roots.iter().enumerate() {
            positions.push(SearchPosition {
                loader: &domain.loader,
                root_index: u32::try_from(index).unwrap_or(u32::MAX),
                root,
            });
        }
    }
    positions
}

/// Decides one position: `Ok(None)` when the root holds no candidate for this name,
/// `Ok(Some(lookup))` when this position decided the lookup, `Err` when the position cannot be
/// decided at all. Every read the position really performed is appended to `reads`.
fn probe_root(
    content: &[ArtifactSnapshot],
    position: &SearchPosition<'_>,
    internal_name: &[u8],
    entry_name: &[u8],
    budget: &mut Budget,
    reads: &mut Vec<HeaderLocation>,
) -> Result<Option<HeaderLookup>> {
    let label = position_label(position);
    let Some(snapshot) = content
        .iter()
        .find(|candidate| Some(candidate.id()) == root_snapshot(position.root))
    else {
        return Err(problem_error(
            EnvironmentProblemCode::ContentNotProvided,
            format!(
                "{label} names content the request does not provide; the position cannot be \
                 searched"
            ),
        ));
    };
    match position.root {
        LoadRoot::External { id } => Err(problem_error(
            EnvironmentProblemCode::UnreadableRoot,
            format!(
                "{label} is the external declaration `{id}`; an external root is declared but \
                 not readable and resolves nothing"
            ),
        )),
        LoadRoot::Snapshot { .. } => match snapshot.kind() {
            ArtifactKind::Zip => {
                let candidates = zip_candidates(snapshot, entry_name, &label, budget)?;
                decide(snapshot, position, candidates, budget, reads)
            }
            ArtifactKind::StandaloneClass => {
                standalone_probe(snapshot, position, internal_name, budget, reads)
            }
        },
        LoadRoot::ArtifactTree { root } => {
            let candidates = tree_candidates(snapshot, root, entry_name, &label, budget)?;
            decide(snapshot, position, candidates, budget, reads)
        }
    }
}

/// The raw-name candidates of a ZIP root.
///
/// The listing is the existing `snapshot.enumerate` path, so its `archive_entries` and
/// `result_items` accounting is unchanged and 2.1 builds no index of its own.
fn zip_candidates(
    snapshot: &ArtifactSnapshot,
    entry_name: &[u8],
    label: &str,
    budget: &mut Budget,
) -> Result<Vec<PhysicalEntry>> {
    let report = snapshot.enumerate(budget)?;
    require_complete_listing(budget, &report.execution, &report.diagnostics, label)?;
    Ok(matching_entries(report.entries, entry_name))
}

/// The raw-name candidates of one artifact-tree root container.
///
/// The position is the declared container itself: entries of nested containers are separate
/// positions and are searched only when a loader declares them as its own root.
fn tree_candidates(
    snapshot: &ArtifactSnapshot,
    root: &ContainerOrigin,
    entry_name: &[u8],
    label: &str,
    budget: &mut Budget,
) -> Result<Vec<PhysicalEntry>> {
    let report = snapshot.enumerate_artifact_tree(budget)?;
    let Some(container) = report
        .containers
        .iter()
        .find(|container| container.origin == *root)
    else {
        // A complete enumeration proves this snapshot has no such container, so the declared
        // position holds no candidate; an incomplete one cannot prove it, and reading it as
        // "the name is not here" would be a guess.
        require_complete_listing(budget, &report.execution, &report.diagnostics, label)?;
        return Ok(Vec::new());
    };
    require_complete_listing(budget, &container.execution, &report.diagnostics, label)?;
    Ok(container
        .entries
        .iter()
        .filter(|entry| entry.id.raw_name.0 == entry_name)
        .cloned()
        .collect())
}

/// The listing's raw-name candidates, in the listing's own order.
///
/// The rule is P1's candidate discipline: byte-exact, case-sensitive equality with
/// `internal_name + ".class"`, container order and entry ordinal preserved. Nothing is
/// normalized, so a `Foo.class` entry is never a candidate for `foo`; a directory name ends
/// with `/` and can therefore never equal a class entry name.
fn matching_entries(entries: Vec<PhysicalEntry>, entry_name: &[u8]) -> Vec<PhysicalEntry> {
    entries
        .into_iter()
        .filter(|entry| entry.id.raw_name.0 == entry_name)
        .collect()
}

/// Decides one position from the candidates it really holds.
///
/// One candidate is the definition the position selects. Several candidates cannot be told
/// apart, so each one is read for its origin and byte identity and the position is `Ambiguous`:
/// byte-equal duplicates are not ordered by ordinal either. A candidate that cannot be read is
/// that position's failure and the search does not continue past it. Every candidate that was
/// read successfully is appended to `reads`, the ambiguous ones included: those reads happened
/// and produced identities, they just did not elect a definition.
fn decide(
    snapshot: &ArtifactSnapshot,
    position: &SearchPosition<'_>,
    candidates: Vec<PhysicalEntry>,
    budget: &mut Budget,
    reads: &mut Vec<HeaderLocation>,
) -> Result<Option<HeaderLookup>> {
    match candidates.as_slice() {
        [] => Ok(None),
        [entry] => {
            let content = read_candidate(snapshot, position, entry, budget)?;
            let facts = class_facts(&content.bytes, budget)
                .map_err(|error| at_origin(error, &content.origin))?;
            reads.push(content.location.clone());
            Ok(Some(HeaderLookup::found(
                content.location,
                ClassHeaderFacts { facts },
            )))
        }
        several => {
            let mut locations = Vec::with_capacity(several.len());
            for entry in several {
                locations.push(read_candidate(snapshot, position, entry, budget)?.location);
            }
            reads.extend(locations.iter().cloned());
            Ok(Some(HeaderLookup::ambiguous(locations)))
        }
    }
}

/// One archive candidate's bytes, identity and origin.
struct CandidateContent {
    location: HeaderLocation,
    /// Origin of this candidate, so a failure names where it was read.
    origin: String,
    bytes: Vec<u8>,
}

fn read_candidate(
    snapshot: &ArtifactSnapshot,
    position: &SearchPosition<'_>,
    entry: &PhysicalEntry,
    budget: &mut Budget,
) -> Result<CandidateContent> {
    let origin = format!("{} {}", position_label(position), entry_label(&entry.id));
    charge_header_attempt(budget)?;
    let materialized = snapshot
        .read_entry_internal(entry, budget)
        .map_err(|error| at_origin(error, &origin))?;
    let length = u64::try_from(materialized.bytes.len()).map_err(|_| {
        Error::invalid_input("class_size_overflow", "class length does not fit u64")
    })?;
    Ok(CandidateContent {
        location: HeaderLocation {
            loader: position.loader.clone(),
            root_index: position.root_index,
            definition: PhysicalDefinitionId {
                location: PhysicalClassLocation::ArchiveEntry {
                    entry: entry.id.clone(),
                },
                class_bytes: ClassBytesId {
                    digest: materialized.content_digest,
                    length,
                },
                variant: physical_variant_for_path(&entry.id.raw_name.0),
            },
            entry: Some(entry.id.clone()),
        },
        origin,
        bytes: materialized.bytes,
    })
}

/// Decides a standalone CLASS root, which declares its own name.
///
/// The root is its own single candidate: its bytes are read, and `this_class` decides whether
/// this root provides the requested name. The comparison is on raw bytes. A root that declares
/// another name simply holds no candidate for this name and the search continues with the next
/// position; the read attempt is charged either way, and the read is recorded either way,
/// because the bytes of this definition really were read.
fn standalone_probe(
    snapshot: &ArtifactSnapshot,
    position: &SearchPosition<'_>,
    internal_name: &[u8],
    budget: &mut Budget,
    reads: &mut Vec<HeaderLocation>,
) -> Result<Option<HeaderLookup>> {
    let origin = format!("{} (standalone CLASS root)", position_label(position));
    charge_header_attempt(budget)?;
    let bytes = snapshot
        .root_bytes(budget)
        .map_err(|error| at_origin(error, &origin))?;
    let facts = class_facts(&bytes, budget).map_err(|error| at_origin(error, &origin))?;
    let length = u64::try_from(bytes.len()).map_err(|_| {
        Error::invalid_input("class_size_overflow", "class length does not fit u64")
    })?;
    let location = HeaderLocation {
        loader: position.loader.clone(),
        root_index: position.root_index,
        definition: PhysicalDefinitionId {
            location: PhysicalClassLocation::StandaloneRoot {
                snapshot: snapshot.id().clone(),
            },
            class_bytes: ClassBytesId {
                digest: Digest(blake3::hash(&bytes).to_hex().to_string()),
                length,
            },
            variant: PhysicalVariant::Base,
        },
        entry: None,
    };
    reads.push(location.clone());
    if facts.this_class.raw().0 != internal_name {
        return Ok(None);
    }
    Ok(Some(HeaderLookup::found(
        location,
        ClassHeaderFacts { facts },
    )))
}

/// Reads one definition's header by identity: the bytes the definition names, parsed the way
/// every other header read is parsed.
///
/// The definition carries its own snapshot and container coordinates, so no search order is
/// involved and no position is examined. What the read does keep is the discipline of every
/// other read: the attempt is charged before the bytes are read, an incomplete listing cannot
/// prove an entry absent, and the bytes at the definition's own location must be the bytes the
/// definition names (a definition whose content changed under the same coordinate would
/// otherwise be measured against a class the request never named).
pub(crate) fn read_definition_content(
    content: &[ArtifactSnapshot],
    definition: &PhysicalDefinitionId,
    budget: &mut Budget,
) -> Result<DefinitionContent> {
    let label = definition_label(definition);
    let Some(snapshot) = content
        .iter()
        .find(|candidate| candidate.id() == definition.snapshot())
    else {
        return Err(problem_error(
            EnvironmentProblemCode::ContentNotProvided,
            format!("{label} names content the request does not provide; the class cannot be read"),
        ));
    };
    charge_header_attempt(budget)?;
    let (bytes, digest) = match &definition.location {
        PhysicalClassLocation::ArchiveEntry { entry } => {
            let listed = listed_entry(snapshot, entry, &label, budget)?;
            let materialized = snapshot
                .read_entry_internal(&listed, budget)
                .map_err(|error| at_origin(error, &label))?;
            (materialized.bytes, materialized.content_digest)
        }
        PhysicalClassLocation::StandaloneRoot { .. } => {
            let bytes = snapshot
                .root_bytes(budget)
                .map_err(|error| at_origin(error, &label))?;
            let digest = Digest(blake3::hash(&bytes).to_hex().to_string());
            (bytes, digest)
        }
    };
    let length = u64::try_from(bytes.len()).map_err(|_| {
        Error::invalid_input("class_size_overflow", "class length does not fit u64")
    })?;
    require_definition_bytes(definition, &digest, length, &label)?;
    let facts = class_facts(&bytes, budget).map_err(|error| at_origin(error, &label))?;
    Ok(DefinitionContent {
        bytes,
        header: ClassHeaderFacts { facts },
    })
}

/// The bytes and the header facts of one class definition, read by identity.
///
/// A body demand needs both halves of the same read: it locates the member in the header and
/// decodes the `Code` attribute from the bytes the definition names. Returning them together
/// keeps that one read — one charge, one identity check — instead of making the caller read
/// the same definition twice and compare two readings itself.
pub(crate) struct DefinitionContent {
    pub(crate) bytes: Vec<u8>,
    pub(crate) header: ClassHeaderFacts,
}

/// The header facts of one class definition, read by identity.
///
/// The read (and its `ClassHeaders` charge, identity check and diagnostics) belongs to
/// [`read_definition_content`]; this is the header-only view of it.
pub(crate) fn read_definition_header(
    content: &[ArtifactSnapshot],
    definition: &PhysicalDefinitionId,
    budget: &mut Budget,
) -> Result<ClassHeaderFacts> {
    Ok(read_definition_content(content, definition, budget)?.header)
}

/// The listed entry one definition names, so its bytes can be read.
///
/// A flat entry comes from the snapshot's own listing and a nested one from the container that
/// holds it. Both listings have to be complete: an incomplete listing cannot prove that the
/// entry is absent, and reading its prefix as "the definition is gone" would be a guess.
fn listed_entry(
    snapshot: &ArtifactSnapshot,
    entry: &PhysicalEntryId,
    label: &str,
    budget: &mut Budget,
) -> Result<PhysicalEntry> {
    if entry.origin.steps.is_empty() {
        let report = snapshot.enumerate(budget)?;
        require_complete_listing(budget, &report.execution, &report.diagnostics, label)?;
        return report
            .entries
            .into_iter()
            .find(|candidate| candidate.id == *entry)
            .ok_or_else(|| missing_entry(label));
    }
    let report = snapshot.enumerate_artifact_tree(budget)?;
    let Some(container) = report
        .containers
        .iter()
        .find(|container| container.origin == entry.origin)
    else {
        require_complete_listing(budget, &report.execution, &report.diagnostics, label)?;
        return Err(missing_entry(label));
    };
    require_complete_listing(budget, &container.execution, &report.diagnostics, label)?;
    container
        .entries
        .iter()
        .find(|candidate| candidate.id == *entry)
        .cloned()
        .ok_or_else(|| missing_entry(label))
}

fn missing_entry(label: &str) -> Error {
    problem_error(
        EnvironmentProblemCode::ContentNotProvided,
        format!("{label} is not an entry of the provided snapshot"),
    )
}

/// The bytes at a definition's own location must be the bytes that definition names.
fn require_definition_bytes(
    definition: &PhysicalDefinitionId,
    digest: &Digest,
    length: u64,
    label: &str,
) -> Result<()> {
    if definition.class_bytes.digest == *digest && definition.class_bytes.length == length {
        return Ok(());
    }
    Err(Error::invalid_input(
        "class_definition_mismatch",
        format!(
            "{label} announces {} bytes with digest `{}`, but the provided content holds \
             {length} bytes with digest `{}`; the definition does not describe the bytes at its \
             own location",
            definition.class_bytes.length, definition.class_bytes.digest.0, digest.0
        ),
    ))
}

/// Human-readable coordinate of one class definition, for the messages of the reads that name
/// it by identity rather than by a search position.
fn definition_label(definition: &PhysicalDefinitionId) -> String {
    match &definition.location {
        PhysicalClassLocation::ArchiveEntry { entry } => format!(
            "the class definition at {} of snapshot `{}`",
            entry_label(entry),
            definition.snapshot().0
        ),
        PhysicalClassLocation::StandaloneRoot { snapshot } => format!(
            "the standalone CLASS definition of snapshot `{}`",
            snapshot.0
        ),
    }
}

/// A listing that stopped early cannot decide a position.
///
/// The listing keeps its own evidence (execution reason and diagnostics); the lookup propagates
/// the same refusal as an `Err`, so the report layer maps one plane per cause: a budget stop
/// stays a budget stop, a cancellation stays a cancellation, and a structural failure keeps the
/// code of the diagnostic that names the damage. The search never reads a stopped listing as
/// "this root has no such name".
fn require_complete_listing(
    budget: &Budget,
    execution: &ExecutionReport,
    diagnostics: &[Diagnostic],
    label: &str,
) -> Result<()> {
    let reason = match execution {
        ExecutionReport::Complete { .. } => return Ok(()),
        ExecutionReport::Partial { reason, .. } | ExecutionReport::Failed { reason, .. } => reason,
        ExecutionReport::Cancelled { .. } => {
            return Err(Error::Cancelled {
                reason: listing_message(diagnostics, label),
            });
        }
    };
    Err(match reason {
        TerminationReason::BudgetExceeded { dimension } => budget_refusal(budget, *dimension),
        TerminationReason::Error { code } => {
            Error::invalid_input(code.clone(), listing_message(diagnostics, label))
        }
        TerminationReason::Unsupported { code } => {
            Error::unsupported(code.clone(), listing_message(diagnostics, label))
        }
    })
}

/// The refusal a stopped listing stands for, with the numbers the listing hit.
///
/// The stopped listing already charged its dimension to the limit, so the limit and usage the
/// budget holds are the evidence the listing stopped with.
fn budget_refusal(budget: &Budget, dimension: BudgetDimension) -> Error {
    let limits = budget.limits();
    let usage = budget.usage();
    let (limit, consumed) = dimension_evidence(limits, &usage, dimension);
    Error::BudgetExceeded {
        dimension,
        limit,
        consumed,
        requested: 1,
    }
}

/// Limit and usage of one dimension, read from its own slots.
///
/// Every dimension has its own arm, so a dimension added later fails to compile here instead of
/// silently reporting another dimension's numbers. The counted ones read the counted table, the
/// three high-water ones their own high-water slots.
fn dimension_evidence(
    limits: &Limits,
    usage: &UsageSnapshot,
    dimension: BudgetDimension,
) -> (u64, u64) {
    let counted = |dimension| {
        (
            limits.counted_limit(dimension),
            usage.counted_usage(dimension),
        )
    };
    match dimension {
        BudgetDimension::InputBytes => counted(CountedBudgetDimension::InputBytes),
        BudgetDimension::ArchiveEntries => counted(CountedBudgetDimension::ArchiveEntries),
        BudgetDimension::EntryBytes => counted(CountedBudgetDimension::EntryBytes),
        BudgetDimension::ReadBytes => counted(CountedBudgetDimension::ReadBytes),
        BudgetDimension::ClassBytes => counted(CountedBudgetDimension::ClassBytes),
        BudgetDimension::AttributeBytes => counted(CountedBudgetDimension::AttributeBytes),
        BudgetDimension::CodeBytes => counted(CountedBudgetDimension::CodeBytes),
        BudgetDimension::ResultItems => counted(CountedBudgetDimension::ResultItems),
        BudgetDimension::OutputBytes => counted(CountedBudgetDimension::OutputBytes),
        BudgetDimension::ClassHeaders => counted(CountedBudgetDimension::ClassHeaders),
        BudgetDimension::MethodBodies => counted(CountedBudgetDimension::MethodBodies),
        BudgetDimension::IrItems => counted(CountedBudgetDimension::IrItems),
        BudgetDimension::IrEdges => counted(CountedBudgetDimension::IrEdges),
        BudgetDimension::AnalysisSteps => counted(CountedBudgetDimension::AnalysisSteps),
        BudgetDimension::NormalizationClones => {
            counted(CountedBudgetDimension::NormalizationClones)
        }
        BudgetDimension::NestedDepth => (limits.nested_depth, usage.nested_depth),
        BudgetDimension::DependencyDepth => (limits.dependency_depth, usage.dependency_depth),
        BudgetDimension::ElapsedMillis => (limits.elapsed_millis, usage.elapsed_millis),
    }
}

/// Message of a stopped listing: the position stays undecided, and the listing's own last
/// diagnostic (the one that names the stop) is kept in the text.
fn listing_message(diagnostics: &[Diagnostic], label: &str) -> String {
    match diagnostics.last() {
        Some(diagnostic) => format!(
            "{label} could not be listed completely ({}: {}); the search position stays \
             undecided and the lookup does not continue",
            diagnostic.code, diagnostic.message
        ),
        None => format!(
            "{label} could not be listed completely; the search position stays undecided and \
             the lookup does not continue"
        ),
    }
}

/// One header read attempt costs one `ClassHeaders`, charged **before** the read.
///
/// The attempt is the unit: a candidate whose bytes cannot be parsed, a standalone root that
/// declares another name, and each of several indistinguishable candidates are all read
/// attempts and all cost one.
fn charge_header_attempt(budget: &mut Budget) -> Result<()> {
    budget.charge(CountedBudgetDimension::ClassHeaders, 1)
}

/// The raw entry name a ZIP or tree position has to spell for this class.
fn entry_name(internal_name: &[u8]) -> Vec<u8> {
    let mut expected = Vec::with_capacity(internal_name.len() + CLASS_SUFFIX.len());
    expected.extend_from_slice(internal_name);
    expected.extend_from_slice(CLASS_SUFFIX);
    expected
}

/// The snapshot one root names, if the declaration names one at all.
fn root_snapshot(root: &LoadRoot) -> Option<&SnapshotId> {
    match root {
        LoadRoot::Snapshot { snapshot } => Some(snapshot),
        LoadRoot::ArtifactTree { root } => Some(&root.snapshot),
        LoadRoot::External { .. } => None,
    }
}

fn domain_for<'a>(
    environment: &'a ResolutionEnvironment,
    loader: &LoaderId,
) -> Option<&'a LoadDomain> {
    environment
        .domains
        .iter()
        .find(|domain| &domain.loader == loader)
}

/// Keeps the reader's own code and names the position the failure was found at.
///
/// A budget stop or a cancellation is not a reader failure and keeps its own evidence, so it is
/// propagated untouched.
fn at_origin(error: Error, origin: &str) -> Error {
    match error {
        Error::InvalidInput { code, message } => {
            Error::invalid_input(code, format!("{origin}: {message}"))
        }
        Error::Unsupported { code, message } => {
            Error::unsupported(code, format!("{origin}: {message}"))
        }
        other => other,
    }
}

/// A refusal that carries one of the closed environment problem codes.
///
/// The lookup refuses what the validator already reported instead of guessing an order; the
/// codes stay the validator's, so a diagnostic can never name a second vocabulary.
fn problem_error(code: EnvironmentProblemCode, message: String) -> Error {
    Error::invalid_input(code.as_str(), message)
}

fn unsupported_policy(domain: &LoadDomain, declaration: &str) -> Error {
    problem_error(
        EnvironmentProblemCode::UnsupportedPolicy,
        format!(
            "loader `{}` declares {declaration}; this slice searches only `class_path` domains \
             with `parent_first` or `child_first` delegation",
            domain.loader.0
        ),
    )
}

/// Human-readable origin of one search position.
fn position_label(position: &SearchPosition<'_>) -> String {
    let loader = &position.loader.0;
    let index = position.root_index;
    match position.root {
        LoadRoot::Snapshot { snapshot } => {
            format!(
                "root {index} of loader `{loader}` (snapshot `{}`)",
                snapshot.0
            )
        }
        LoadRoot::ArtifactTree { root } => format!(
            "root {index} of loader `{loader}` (artifact tree container `{}` of snapshot `{}`)",
            root.current_container().0,
            root.snapshot.0
        ),
        LoadRoot::External { id } => {
            format!("root {index} of loader `{loader}` (external declaration `{id}`)")
        }
    }
}

/// Human-readable origin of one archive entry.
fn entry_label(entry: &PhysicalEntryId) -> String {
    format!(
        "entry \"{}\" (ordinal {})",
        escaped(&entry.raw_name.0),
        entry.ordinal
    )
}

/// Byte-safe display of a raw name, so a message cannot carry a broken line or hide which bytes
/// it means.
///
/// Shared with the member slice, which names raw owner, name and descriptor bytes in the
/// diagnostics of its own rules.
pub(crate) fn escaped(raw: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut escaped = String::new();
    for &byte in raw {
        match byte {
            b'\\' => escaped.push_str("\\\\"),
            b'"' => escaped.push_str("\\\""),
            0x20..=0x7e => escaped.push(char::from(byte)),
            _ => write!(escaped, "\\x{byte:02X}").expect("writing to String cannot fail"),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::ArtifactInput;
    use crate::budget::{CancellationToken, Limits};
    use crate::view::{
        LayoutMode, ModuleMode, MultiReleasePolicy, PhysicalScope, PhysicalView, RuntimeProfile,
        RuntimeUncertainty, RuntimeView,
    };
    use rawzip::{CompressionMethod, ZipArchiveWriter, path::EntryPath};
    use std::io::{Cursor, Write};

    const STORE: u16 = 0;

    fn limits() -> Limits {
        Limits {
            input_bytes: 1 << 20,
            archive_entries: 1_000,
            entry_bytes: 1 << 20,
            read_bytes: 1 << 20,
            class_bytes: 1 << 20,
            attribute_bytes: 1 << 20,
            output_bytes: 1 << 20,
            result_items: 10_000,
            class_headers: 100,
            nested_depth: 4,
            elapsed_millis: u64::MAX,
            ..Limits::default()
        }
    }

    /// Smallest class file the reader accepts, with `this_class` set to `this_class`.
    fn class_bytes(this_class: &[u8], major: u16) -> Vec<u8> {
        class_with(this_class, Some(b"java/lang/Object"), &[], major)
    }

    /// Smallest class file the reader accepts, with a chosen name, superclass and interfaces.
    ///
    /// The constant pool is built in declaration order: the class's own name first, then the
    /// superclass (when there is one), then the interfaces, each name as a `CONSTANT_Utf8`
    /// followed by its `CONSTANT_Class`. Nothing else is in the pool, so a fixture that claims
    /// a supertype really declares it.
    fn class_with(
        this_class: &[u8],
        super_class: Option<&[u8]>,
        interfaces: &[&[u8]],
        major: u16,
    ) -> Vec<u8> {
        let mut names: Vec<&[u8]> = vec![this_class];
        names.extend(super_class);
        names.extend(interfaces.iter().copied());
        let mut pool = Vec::new();
        for (index, name) in names.iter().enumerate() {
            pool.push(1_u8); // CONSTANT_Utf8 #(1 + 2 * index)
            pool.extend_from_slice(
                &u16::try_from(name.len())
                    .expect("fixture name fits u16")
                    .to_be_bytes(),
            );
            pool.extend_from_slice(name);
            pool.push(7_u8); // CONSTANT_Class #(2 + 2 * index) -> the name above
            pool.extend_from_slice(
                &u16::try_from(1 + index * 2)
                    .expect("fixture pool index fits u16")
                    .to_be_bytes(),
            );
        }
        let count = u16::try_from(names.len() * 2 + 1).expect("fixture pool count fits u16");

        let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // minor
        bytes.extend_from_slice(&major.to_be_bytes());
        bytes.extend_from_slice(&count.to_be_bytes()); // constant_pool_count
        bytes.extend_from_slice(&pool);
        bytes.extend_from_slice(&0x0021_u16.to_be_bytes()); // ACC_PUBLIC | ACC_SUPER
        bytes.extend_from_slice(&2_u16.to_be_bytes()); // this_class -> Class #2
        bytes.extend_from_slice(&if super_class.is_some() { 4_u16 } else { 0_u16 }.to_be_bytes());
        bytes.extend_from_slice(
            &u16::try_from(interfaces.len())
                .expect("interface count fits u16")
                .to_be_bytes(),
        );
        for index in 0..interfaces.len() {
            bytes.extend_from_slice(
                &u16::try_from(6 + index * 2)
                    .expect("fixture interface index fits u16")
                    .to_be_bytes(),
            );
        }
        for _ in 0..3 {
            bytes.extend_from_slice(&0_u16.to_be_bytes()); // fields, methods, attributes
        }
        bytes
    }

    /// Limits with the dimensions a closure charges funded, on top of a 2.1 fixture.
    fn closure_limits() -> Limits {
        Limits {
            dependency_depth: 4,
            analysis_steps: 1_000,
            ..limits()
        }
    }

    /// A single `app` loader whose only root is the given snapshot.
    fn app_environment(snapshot: &ArtifactSnapshot) -> ResolutionEnvironment {
        let caller = domain(
            "app",
            None,
            DelegationPolicy::ParentFirst,
            vec![root_of(snapshot)],
        );
        environment(caller.clone(), vec![caller])
    }

    /// A class file whose single method carries a `Code` attribute.
    ///
    /// The body is one `return`, so the class really holds bytes a body read would find: a
    /// closure that reports `method_bodies == 0` on a class without a body proves nothing.
    /// Pool: `#1` name, `#2` its class, `#3` `java/lang/Object`, `#4` its class, `#5` `m`,
    /// `#6` `()V`, `#7` `Code`.
    fn class_with_body(this_class: &[u8]) -> Vec<u8> {
        let mut pool = Vec::new();
        for name in [this_class, b"java/lang/Object", b"m", b"()V", b"Code"] {
            pool.push(1_u8); // CONSTANT_Utf8
            pool.extend_from_slice(
                &u16::try_from(name.len())
                    .expect("fixture name fits u16")
                    .to_be_bytes(),
            );
            pool.extend_from_slice(name);
        }
        // Pool: #1 the class name, #2 `java/lang/Object`, #3 `m`, #4 `()V`, #5 `Code`,
        // #6 the class entry of #1, #7 the class entry of #2.
        pool.extend_from_slice(&[7, 0, 1]);
        pool.extend_from_slice(&[7, 0, 3]);

        // max_stack, max_locals, code_length, one `return`, no handlers, no code attributes.
        let code: &[u8] = &[0, 0, 0, 1, 0, 0, 0, 1, 0xb1, 0, 0, 0, 0];
        let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // minor
        bytes.extend_from_slice(&52_u16.to_be_bytes()); // major
        bytes.extend_from_slice(&8_u16.to_be_bytes()); // constant_pool_count: #1..#7
        bytes.extend_from_slice(&pool);
        bytes.extend_from_slice(&0x0021_u16.to_be_bytes()); // ACC_PUBLIC | ACC_SUPER
        bytes.extend_from_slice(&6_u16.to_be_bytes()); // this_class -> #6
        bytes.extend_from_slice(&7_u16.to_be_bytes()); // super_class -> #7
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // fields
        bytes.extend_from_slice(&1_u16.to_be_bytes()); // methods
        bytes.extend_from_slice(&0x0001_u16.to_be_bytes()); // ACC_PUBLIC
        bytes.extend_from_slice(&3_u16.to_be_bytes()); // name_index -> "m"
        bytes.extend_from_slice(&4_u16.to_be_bytes()); // descriptor_index -> "()V"
        bytes.extend_from_slice(&1_u16.to_be_bytes()); // method attributes
        bytes.extend_from_slice(&5_u16.to_be_bytes()); // attribute_name_index -> "Code"
        bytes.extend_from_slice(
            &u32::try_from(code.len())
                .expect("fixture code fits u32")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(code);
        bytes.extend_from_slice(&0_u16.to_be_bytes()); // class attributes
        bytes
    }

    fn zip(entries: &[(&[u8], &[u8])]) -> Vec<u8> {
        let mut output = Cursor::new(Vec::new());
        {
            let mut archive = ZipArchiveWriter::new(&mut output);
            for (name, data) in entries {
                let (mut entry, config) = archive
                    .new_file(EntryPath::verbatim(name.to_vec()))
                    .compression_method(CompressionMethod::new(STORE))
                    .start()
                    .unwrap();
                let mut writer = config.wrap(&mut entry);
                writer.write_all(data).unwrap();
                let (_, descriptor) = writer.finish().unwrap();
                entry.finish(descriptor).unwrap();
            }
            archive.finish().unwrap();
        }
        output.into_inner()
    }

    fn open(bytes: Vec<u8>) -> ArtifactSnapshot {
        ArtifactSnapshot::open(ArtifactInput::bytes(bytes), &mut Budget::new(limits()))
            .expect("the fixture archive opens")
    }

    fn domain(
        loader: &str,
        parent: Option<&str>,
        delegation: DelegationPolicy,
        roots: Vec<LoadRoot>,
    ) -> LoadDomain {
        LoadDomain {
            loader: LoaderId(loader.to_string()),
            parent_loader: parent.map(|parent| LoaderId(parent.to_string())),
            delegation,
            roots,
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        }
    }

    fn environment(caller: LoadDomain, domains: Vec<LoadDomain>) -> ResolutionEnvironment {
        ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: SnapshotId("runtime".into()),
                    scope: PhysicalScope::SnapshotAll,
                },
                profile: RuntimeProfile {
                    java_release: 8,
                    multi_release: MultiReleasePolicy::Disabled,
                    layout: LayoutMode::Generic,
                },
                load_domain: caller,
            },
            domains,
            providers: Vec::new(),
        }
    }

    fn root_of(snapshot: &ArtifactSnapshot) -> LoadRoot {
        LoadRoot::Snapshot {
            snapshot: snapshot.id().clone(),
        }
    }

    /// Loader names of the effective sequence, caller first.
    fn sequence(environment: &ResolutionEnvironment) -> Result<Vec<String>> {
        Ok(ordered_domains(environment)?
            .iter()
            .map(|domain| domain.loader.0.clone())
            .collect())
    }

    #[test]
    fn parent_first_walks_up_and_child_first_starts_at_the_caller() {
        let parent_first = environment(
            domain(
                "app",
                Some("platform"),
                DelegationPolicy::ParentFirst,
                Vec::new(),
            ),
            vec![
                domain(
                    "app",
                    Some("platform"),
                    DelegationPolicy::ParentFirst,
                    Vec::new(),
                ),
                domain(
                    "platform",
                    Some("boot"),
                    DelegationPolicy::ParentFirst,
                    Vec::new(),
                ),
                domain("boot", None, DelegationPolicy::ParentFirst, Vec::new()),
            ],
        );
        assert_eq!(
            sequence(&parent_first).unwrap(),
            vec!["boot", "platform", "app"],
            "ParentFirst walks from the top ancestor down to the caller"
        );

        let child_first = environment(
            domain(
                "app",
                Some("platform"),
                DelegationPolicy::ChildFirst,
                Vec::new(),
            ),
            vec![
                domain(
                    "app",
                    Some("platform"),
                    DelegationPolicy::ChildFirst,
                    Vec::new(),
                ),
                domain(
                    "platform",
                    Some("boot"),
                    DelegationPolicy::ChildFirst,
                    Vec::new(),
                ),
                domain("boot", None, DelegationPolicy::ChildFirst, Vec::new()),
            ],
        );
        assert_eq!(
            sequence(&child_first).unwrap(),
            vec!["app", "platform", "boot"],
            "ChildFirst starts at the caller"
        );
    }

    #[test]
    fn a_mixed_chain_orders_each_loader_by_its_own_delegation() {
        let mixed = environment(
            domain(
                "app",
                Some("platform"),
                DelegationPolicy::ParentFirst,
                Vec::new(),
            ),
            vec![
                domain(
                    "app",
                    Some("platform"),
                    DelegationPolicy::ParentFirst,
                    Vec::new(),
                ),
                domain(
                    "platform",
                    Some("boot"),
                    DelegationPolicy::ChildFirst,
                    Vec::new(),
                ),
                domain("boot", None, DelegationPolicy::ParentFirst, Vec::new()),
            ],
        );
        assert_eq!(
            sequence(&mixed).unwrap(),
            vec!["platform", "boot", "app"],
            "the child-first loader is searched before its own parent's sequence"
        );
    }

    #[test]
    fn an_unusable_chain_is_refused_instead_of_shortened() {
        let refused = |caller: LoadDomain, domains: Vec<LoadDomain>| {
            sequence(&environment(caller, domains)).expect_err("an unordered chain is refused")
        };
        let code = |error: Error| match error {
            Error::InvalidInput { code, .. } => code,
            other => panic!("a refused chain keeps a problem code, got {other}"),
        };

        // The caller names a parent no domain binds.
        let caller = domain(
            "app",
            Some("platform"),
            DelegationPolicy::ParentFirst,
            Vec::new(),
        );
        assert_eq!(
            code(refused(caller.clone(), vec![caller])),
            EnvironmentProblemCode::MissingParent.as_str(),
            "a chain that cannot be walked is refused, not shortened"
        );

        // The parent chain re-enters the caller.
        let caller = domain(
            "app",
            Some("platform"),
            DelegationPolicy::ParentFirst,
            Vec::new(),
        );
        let parent = domain(
            "platform",
            Some("app"),
            DelegationPolicy::ParentFirst,
            Vec::new(),
        );
        assert_eq!(
            code(refused(caller.clone(), vec![caller, parent])),
            EnvironmentProblemCode::ParentCycle.as_str()
        );

        // A module mode this slice cannot execute.
        let mut caller = domain("app", None, DelegationPolicy::ParentFirst, Vec::new());
        caller.module_mode = ModuleMode::ModulePath;
        assert_eq!(
            code(refused(caller.clone(), vec![caller])),
            EnvironmentProblemCode::UnsupportedPolicy.as_str()
        );

        // A delegation policy this slice cannot execute.
        let caller = domain(
            "app",
            None,
            DelegationPolicy::Custom {
                id: "osgi".to_string(),
            },
            Vec::new(),
        );
        assert_eq!(
            code(refused(caller.clone(), vec![caller])),
            EnvironmentProblemCode::UnsupportedPolicy.as_str()
        );
    }

    #[test]
    fn positions_follow_declaration_order_with_their_root_indices() {
        let first = open(zip(&[(b"A.class", b"a")]));
        let second = open(zip(&[(b"B.class", b"b")]));
        let environment = environment(
            domain(
                "app",
                None,
                DelegationPolicy::ParentFirst,
                vec![root_of(&first), root_of(&second)],
            ),
            vec![domain(
                "app",
                None,
                DelegationPolicy::ParentFirst,
                vec![root_of(&first), root_of(&second)],
            )],
        );
        let domains = ordered_domains(&environment).unwrap();
        let positions = search_positions(&domains);
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].loader.0, "app");
        assert_eq!(positions[0].root_index, 0);
        assert_eq!(positions[1].root_index, 1);
        assert_eq!(
            root_snapshot(positions[1].root),
            Some(second.id()),
            "a position keeps the root it was declared at"
        );
    }

    #[test]
    fn a_lookup_reports_the_position_it_selected() {
        let empty = open(zip(&[(b"other/C.class", &class_bytes(b"other/C", 52))]));
        let winning = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 52))]));
        let winning_id = winning.id().clone();
        let trailing = open(zip(&[(b"p/S.class", &class_bytes(b"p/S", 51))]));
        let roots = vec![root_of(&empty), root_of(&winning), root_of(&trailing)];
        let environment = environment(
            domain("app", None, DelegationPolicy::ParentFirst, roots.clone()),
            vec![domain("app", None, DelegationPolicy::ParentFirst, roots)],
        );
        let mut budget = Budget::new(limits());
        let search = lookup_class_header(
            &[empty, winning, trailing],
            &environment,
            b"p/S",
            &mut budget,
        );

        let lookup = search.lookup.expect("the second root holds the name");
        assert_eq!(lookup.state, HeaderLookupState::Found);
        assert_eq!(
            search.examined, 2,
            "the position that decided counts as examined"
        );
        assert_eq!(search.positions, 3);
        assert_eq!(budget.usage().class_headers, 1);
        let location = lookup.location.expect("a found lookup has a location");
        assert_eq!(location.loader.0, "app");
        assert_eq!(
            location.root_index, 1,
            "the root that holds the name is selected"
        );
        assert_eq!(location.definition.snapshot(), &winning_id);
        assert_eq!(
            location.definition.class_bytes.length,
            u64::try_from(class_bytes(b"p/S", 52).len()).unwrap()
        );
        assert!(matches!(
            location.definition.location,
            PhysicalClassLocation::ArchiveEntry { .. }
        ));
        assert!(location.entry.is_some());
        let header = lookup.header.expect("a found lookup carries the header");
        assert_eq!(header.facts.this_class.raw().0, b"p/S");
        assert_eq!(
            header.facts.super_class.as_ref().unwrap().raw().0,
            b"java/lang/Object"
        );
    }

    #[test]
    fn a_multi_release_path_keeps_the_physical_variant_p1_derives() {
        let snapshot = open(zip(&[(
            b"META-INF/versions/11/X.class",
            &class_bytes(b"META-INF/versions/11/X", 52),
        )]));
        let roots = vec![root_of(&snapshot)];
        let environment = environment(
            domain("app", None, DelegationPolicy::ParentFirst, roots.clone()),
            vec![domain("app", None, DelegationPolicy::ParentFirst, roots)],
        );
        let mut budget = Budget::new(limits());
        let search = lookup_class_header(
            &[snapshot],
            &environment,
            b"META-INF/versions/11/X",
            &mut budget,
        );
        let lookup = search
            .lookup
            .expect("the entry matches the name byte for byte");
        assert_eq!(
            lookup.location.unwrap().definition.variant,
            PhysicalVariant::MultiRelease { version: 11 },
            "the lookup and the physical scan derive one variant for one path"
        );
    }

    #[test]
    fn a_name_no_position_holds_is_missing_and_reads_nothing() {
        let snapshot = open(zip(&[(b"other/C.class", &class_bytes(b"other/C", 52))]));
        let roots = vec![root_of(&snapshot)];
        let environment = environment(
            domain("app", None, DelegationPolicy::ParentFirst, roots.clone()),
            vec![domain("app", None, DelegationPolicy::ParentFirst, roots)],
        );
        let mut budget = Budget::new(limits());
        let search = lookup_class_header(&[snapshot], &environment, b"p/S", &mut budget);
        let lookup = search.lookup.expect("a miss is a fact, not a refusal");
        assert_eq!(lookup.state, HeaderLookupState::Missing);
        assert!(lookup.location.is_none());
        assert!(lookup.header.is_none());
        assert_eq!(search.examined, search.positions);
        assert_eq!(budget.usage().class_headers, 0, "no candidate was read");
    }

    // -----------------------------------------------------------------------
    // 2.2: the request-scoped closure
    // -----------------------------------------------------------------------

    /// Walks to the end, appending the handle of every layer the walk decided to `handles`.
    ///
    /// `Err` is the walk's own stop; the handles already collected stay the trustworthy prefix,
    /// and the request's reads stay available on the closure.
    fn walk_handles(
        closure: &mut HeaderClosure<'_>,
        walk: &mut HierarchyWalk,
        budget: &mut Budget,
        handles: &mut Vec<ClassHandle>,
    ) -> Result<()> {
        loop {
            let Some(handle) = walk.next(closure, budget)? else {
                return Ok(());
            };
            handles.push(handle);
        }
    }

    fn names(names: &[&[u8]]) -> Vec<JvmBytes> {
        names.iter().map(|name| JvmBytes(name.to_vec())).collect()
    }

    fn layers(closure: &HeaderClosure<'_>, handles: &[ClassHandle]) -> Vec<JvmBytes> {
        handles
            .iter()
            .map(|handle| closure.resolution(*handle).name.clone())
            .collect()
    }

    #[test]
    fn a_repeated_demand_reuses_the_first_read_without_a_second_record() {
        let snapshot = open(zip(&[(b"p/C.class", &class_bytes(b"p/C", 52))]));
        let environment = app_environment(&snapshot);
        let mut budget = Budget::new(closure_limits());
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);

        let first = closure
            .demand(b"p/C", HeaderDemand::RequestedDefinition, &mut budget)
            .decision
            .expect("the fixture holds the name");
        let searched = closure.searched_extent();
        let repeated = closure.demand(b"p/C", HeaderDemand::HierarchyClosure, &mut budget);

        assert_eq!(
            repeated.decision.expect("the request remembers the key"),
            first,
            "one handle per (loader, internal name) key"
        );
        assert_eq!(
            closure.searched_extent(),
            searched,
            "the memo answers without searching again: no position is examined twice"
        );
        assert_eq!(budget.usage().class_headers, 1, "one read attempt in total");
        assert_eq!(
            closure.reads().len(),
            1,
            "one read record in total, not one per demand"
        );
        assert_eq!(
            closure.reads()[0].demand,
            HeaderDemand::RequestedDefinition,
            "the record keeps the demand that really read the header"
        );
        assert_eq!(closure.reads()[0].loader.0, "app");
        assert_eq!(closure.resolution(first).name, JvmBytes(b"p/C".to_vec()));
        assert_eq!(
            closure.resolution(first).lookup.state,
            HeaderLookupState::Found
        );
    }

    #[test]
    fn one_binding_is_one_record_even_when_two_demands_read_it() {
        // A standalone root that declares the requested name is the only position, so a second
        // demand for another name probes the same root and reads the same bytes again: two read
        // attempts, one `(loader, definition)` binding, and therefore one record — which is why
        // the invariant is `reads.len() <= class_headers` and not equality.
        let bytes = class_bytes(b"p/S", 52);
        let standalone = open(bytes.clone());
        let environment = app_environment(&standalone);
        let mut budget = Budget::new(closure_limits());
        let mut closure = HeaderClosure::new(std::slice::from_ref(&standalone), &environment);

        let first = closure
            .demand(b"p/S", HeaderDemand::RequestedDefinition, &mut budget)
            .decision
            .expect("the root declares this name");
        assert_eq!(
            closure.resolution(first).lookup.state,
            HeaderLookupState::Found
        );
        let second = closure
            .demand(b"p/Other", HeaderDemand::HierarchyClosure, &mut budget)
            .decision
            .expect("a name no position holds is a fact");
        assert_eq!(
            closure.resolution(second).lookup.state,
            HeaderLookupState::Missing
        );

        assert_eq!(
            budget.usage().class_headers,
            2,
            "the standalone root was probed twice"
        );
        assert_eq!(
            closure.reads().len(),
            1,
            "one (definition, loader) binding is one read record"
        );
        assert_eq!(
            closure.reads()[0].definition,
            PhysicalDefinitionId {
                location: PhysicalClassLocation::StandaloneRoot {
                    snapshot: standalone.id().clone(),
                },
                class_bytes: ClassBytesId {
                    digest: Digest(blake3::hash(&bytes).to_hex().to_string()),
                    length: u64::try_from(bytes.len()).expect("fixture length fits u64"),
                },
                variant: PhysicalVariant::Base,
            }
        );
        assert_eq!(
            closure.reads()[0].demand,
            HeaderDemand::RequestedDefinition,
            "the record keeps the demand that performed the read"
        );
    }

    #[test]
    fn a_parent_chain_stops_before_the_step_over_the_dependency_depth() {
        // p/A -> p/B -> p/C -> p/D, every class of the chain in one root.
        let snapshot = open(zip(&[
            (b"p/A.class", &class_with(b"p/A", Some(b"p/B"), &[], 52)),
            (b"p/B.class", &class_with(b"p/B", Some(b"p/C"), &[], 52)),
            (b"p/C.class", &class_with(b"p/C", Some(b"p/D"), &[], 52)),
            (
                b"p/D.class",
                &class_with(b"p/D", Some(b"java/lang/Object"), &[], 52),
            ),
        ]));
        let environment = app_environment(&snapshot);
        let mut budget = Budget::new(Limits {
            dependency_depth: 2,
            ..closure_limits()
        });
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);
        // The caller names p/A for a member rule, so the root layer is that rule's own read and
        // every step above it is a parent-chain read.
        let mut walk = closure.parent_chain(b"p/A", HeaderDemand::MemberOwner);

        let mut expanded = Vec::new();
        let stop = walk_handles(&mut closure, &mut walk, &mut budget, &mut expanded)
            .expect_err("the chain is one step deeper than the limit");

        assert!(
            matches!(
                stop,
                Error::BudgetExceeded {
                    dimension: BudgetDimension::DependencyDepth,
                    limit: 2,
                    consumed: 2,
                    ..
                }
            ),
            "the stop names the depth dimension: {stop:?}"
        );
        assert_eq!(
            layers(&closure, &expanded),
            names(&[b"p/A", b"p/B", b"p/C"]),
            "the layers already expanded are the trustworthy prefix"
        );
        assert_eq!(
            budget.usage().dependency_depth,
            2,
            "the deep-water mark is the deepest layer that was read"
        );
        assert_eq!(
            closure.reads().len(),
            3,
            "reads only hold the layers that were really read"
        );
        assert_eq!(
            closure
                .reads()
                .iter()
                .map(|read| read.demand)
                .collect::<Vec<_>>(),
            vec![
                HeaderDemand::MemberOwner,
                HeaderDemand::ParentChain,
                HeaderDemand::ParentChain,
            ],
            "the root layer keeps the caller's own reason; every step above it is a \
             parent-chain read"
        );
        assert_eq!(budget.usage().class_headers, 3);
        assert_eq!(
            budget.usage().analysis_steps,
            3,
            "one worklist iteration per layer processed"
        );
        assert_eq!(walk.gaps(), &WalkGaps::default());
        assert!(closure.diagnostics().is_empty());
    }

    #[test]
    fn a_supertype_closure_reads_every_header_once() {
        // p/C extends p/Base and implements p/I1, p/I2; the two interfaces extend p/I3 and
        // p/I4, and every class's own superclass is in the same root.
        let snapshot = open(zip(&[
            (
                b"p/C.class",
                &class_with(b"p/C", Some(b"p/Base"), &[b"p/I1", b"p/I2"], 52),
            ),
            (
                b"p/Base.class",
                &class_with(b"p/Base", Some(b"java/lang/Object"), &[], 52),
            ),
            (
                b"p/I1.class",
                &class_with(b"p/I1", Some(b"java/lang/Object"), &[b"p/I3"], 52),
            ),
            (
                b"p/I2.class",
                &class_with(b"p/I2", Some(b"java/lang/Object"), &[b"p/I4"], 52),
            ),
            (
                b"p/I3.class",
                &class_with(b"p/I3", Some(b"java/lang/Object"), &[], 52),
            ),
            (
                b"p/I4.class",
                &class_with(b"p/I4", Some(b"java/lang/Object"), &[], 52),
            ),
            (
                // The root class of the hierarchy declares no superclass: declaring itself
                // would be the cyclic fixture the cycle test builds on purpose.
                b"java/lang/Object.class",
                &class_with(b"java/lang/Object", None, &[], 52),
            ),
        ]));
        let environment = app_environment(&snapshot);
        let mut budget = Budget::new(closure_limits());
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);
        let mut walk = closure.hierarchy_closure(b"p/C", HeaderDemand::MemberOwner);

        let mut expanded = Vec::new();
        walk_handles(&mut closure, &mut walk, &mut budget, &mut expanded)
            .expect("every supertype of the fixture is in the root");

        assert_eq!(
            layers(&closure, &expanded),
            names(&[
                b"p/C",
                b"p/Base",
                b"p/I1",
                b"p/I2",
                b"java/lang/Object",
                b"p/I3",
                b"p/I4",
            ]),
            "breadth-first: the superclass before the interfaces, and every layer before the \
             supertypes it declares"
        );
        assert_eq!(
            budget.usage().class_headers,
            7,
            "one read per class, none repeated"
        );
        assert_eq!(closure.reads().len(), 7);
        assert_eq!(
            budget.usage().dependency_depth,
            2,
            "the interfaces of the interfaces are the deepest layer"
        );
        assert!(closure.diagnostics().is_empty());
        assert_eq!(
            closure
                .reads()
                .iter()
                .map(|read| read.demand)
                .collect::<Vec<_>>(),
            vec![
                // The root layer: the class the caller named.
                HeaderDemand::MemberOwner,
                // p/Base, then java/lang/Object: both reached along a `super_class` edge.
                HeaderDemand::ParentChain,
                // p/I1 and p/I2: reached along an `interfaces` edge.
                HeaderDemand::HierarchyClosure,
                HeaderDemand::HierarchyClosure,
                HeaderDemand::ParentChain,
                // p/I3 and p/I4: the interfaces of the interfaces.
                HeaderDemand::HierarchyClosure,
                HeaderDemand::HierarchyClosure,
            ],
            "one walk that follows both edges publishes both reasons, in read order: the reason \
             names the edge that reached the layer, never the walk as a whole"
        );
        for (index, read) in closure.reads().iter().enumerate() {
            for other in &closure.reads()[index + 1..] {
                assert_ne!(
                    (&read.loader, &read.definition),
                    (&other.loader, &other.definition),
                    "one (definition, loader) binding is one read record"
                );
            }
        }
    }

    #[test]
    fn a_missing_supertype_ends_its_branch_without_a_fabricated_definition() {
        // p/Base and p/Gone are declared as supertypes but no position holds them; p/I1 is
        // there and declares p/Base, which is missing too.
        let snapshot = open(zip(&[
            (
                b"p/C.class",
                &class_with(b"p/C", Some(b"p/Base"), &[b"p/I1", b"p/Gone"], 52),
            ),
            (
                b"p/I1.class",
                &class_with(b"p/I1", Some(b"java/lang/Object"), &[], 52),
            ),
        ]));
        let environment = app_environment(&snapshot);
        let mut budget = Budget::new(closure_limits());
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);
        let mut walk = closure.hierarchy_closure(b"p/C", HeaderDemand::MemberOwner);

        let mut expanded = Vec::new();
        walk_handles(&mut closure, &mut walk, &mut budget, &mut expanded)
            .expect("a missing name is a fact, not a stop");

        assert_eq!(
            layers(&closure, &expanded),
            names(&[b"p/C", b"p/Base", b"p/I1", b"p/Gone", b"java/lang/Object",]),
            "the walk still visits every branch it was declared"
        );
        assert_eq!(
            walk.gaps().missing,
            names(&[b"p/Base", b"p/Gone", b"java/lang/Object"]),
            "every name no position holds is published as a gap — the unfinished part of the \
             closure a report has to mark as skipped"
        );
        assert_eq!(
            budget.usage().class_headers,
            2,
            "only the found names were read"
        );
        assert_eq!(closure.reads().len(), 2);
        for handle in &expanded {
            let resolution = closure.resolution(*handle);
            if resolution.lookup.state == HeaderLookupState::Missing {
                assert!(
                    resolution.lookup.location.is_none() && resolution.lookup.header.is_none(),
                    "a missing name has no definition and no header to publish"
                );
            }
        }
    }

    #[test]
    fn a_cyclic_hierarchy_terminates_with_a_locatable_diagnostic() {
        let snapshot = open(zip(&[
            (b"p/A.class", &class_with(b"p/A", Some(b"p/B"), &[], 52)),
            (b"p/B.class", &class_with(b"p/B", Some(b"p/A"), &[], 52)),
        ]));
        let environment = app_environment(&snapshot);
        let mut budget = Budget::new(closure_limits());
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);
        let mut walk = closure.hierarchy_closure(b"p/A", HeaderDemand::MemberOwner);

        let mut expanded = Vec::new();
        walk_handles(&mut closure, &mut walk, &mut budget, &mut expanded)
            .expect("a cycle ends a branch, it does not fail the request");

        assert_eq!(
            layers(&closure, &expanded),
            names(&[b"p/A", b"p/B"]),
            "the walk terminates: each class is expanded once"
        );
        assert_eq!(walk.gaps().cycles, names(&[b"p/A"]));
        assert_eq!(
            budget.usage().class_headers,
            2,
            "the cycle is refused before a third read"
        );
        assert_eq!(closure.reads().len(), 2);
        assert_eq!(closure.diagnostics().len(), 1);
        let diagnostic = &closure.diagnostics()[0];
        assert_eq!(diagnostic.code, "resolution_hierarchy_cycle");
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        for expected in [
            "loader `app`",
            "p/A -> p/B -> p/A",
            "`p/A` is its own supertype",
        ] {
            assert!(
                diagnostic.message.contains(expected),
                "the diagnostic locates the cycle ({expected}): {}",
                diagnostic.message
            );
        }
    }

    #[test]
    fn a_walk_stops_before_the_next_expansion_on_cancellation() {
        let snapshot = open(zip(&[
            (b"p/A.class", &class_with(b"p/A", Some(b"p/B"), &[], 52)),
            (
                b"p/B.class",
                &class_with(b"p/B", Some(b"java/lang/Object"), &[], 52),
            ),
        ]));
        let environment = app_environment(&snapshot);

        // Cancelled before the walk starts: no layer is read at all.
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        let mut budget = Budget::with_cancellation_token(closure_limits(), cancelled);
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);
        let mut walk = closure.parent_chain(b"p/A", HeaderDemand::RequestedDefinition);
        let stop = walk
            .next(&mut closure, &mut budget)
            .expect_err("a cancelled request reads nothing");
        assert!(matches!(stop, Error::Cancelled { .. }), "{stop:?}");
        assert!(closure.reads().is_empty());
        assert_eq!(budget.usage().class_headers, 0);

        // Cancelled between two layers: the layer that was read stays, the next one is not
        // reached, and the stop is a cancellation rather than a missing definition.
        let token = CancellationToken::new();
        let mut budget = Budget::with_cancellation_token(closure_limits(), token.clone());
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);
        let mut walk = closure.parent_chain(b"p/A", HeaderDemand::RequestedDefinition);
        let root = walk
            .next(&mut closure, &mut budget)
            .expect("the root layer is read")
            .expect("a walk always has its root layer");
        assert_eq!(closure.resolution(root).name, JvmBytes(b"p/A".to_vec()));
        token.cancel();
        let stop = walk
            .next(&mut closure, &mut budget)
            .expect_err("the next expansion is refused");
        assert!(matches!(stop, Error::Cancelled { .. }), "{stop:?}");
        assert_eq!(
            closure.reads().len(),
            1,
            "only the layer read before the cancellation is recorded"
        );
        assert_eq!(budget.usage().class_headers, 1);
        assert!(
            walk.next(&mut closure, &mut budget).is_err(),
            "a stopped walk stays stopped"
        );
        assert!(
            u64::try_from(closure.reads().len()).expect("a read count fits u64")
                <= budget.usage().class_headers,
            "reads.len() <= class_headers holds under a stop"
        );
    }

    #[test]
    fn a_closure_reads_headers_only_and_never_a_body() {
        let bytes = class_with_body(b"p/C");
        // Fixture sanity: the class really carries a method with a body, so "no body was read"
        // is a statement about the closure and not about an empty fixture.
        let mut sanity = Budget::new(closure_limits());
        let facts = class_facts(&bytes, &mut sanity).expect("the fixture is a readable class");
        assert_eq!(facts.methods.len(), 1);
        assert_eq!(facts.methods[0].attributes.len(), 1);
        assert_eq!(facts.methods[0].attributes[0].name.raw().0, b"Code");

        let snapshot = open(zip(&[(b"p/C.class", &bytes)]));
        let environment = app_environment(&snapshot);
        let mut budget = Budget::new(closure_limits());
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);
        let handle = closure
            .demand(b"p/C", HeaderDemand::RequestedDefinition, &mut budget)
            .decision
            .expect("the fixture holds the name");
        let header = closure
            .resolution(handle)
            .lookup
            .header
            .as_ref()
            .expect("a found class carries its header");

        assert_eq!(
            header.facts.methods.len(),
            1,
            "the header facts are the declarations, and the closure uses them as such"
        );
        assert_eq!(
            budget.usage().method_bodies,
            0,
            "a closure reads headers only, even when the class has a body"
        );
        assert_eq!(budget.usage().code_bytes, 0, "no code byte was read");
        assert!(
            closure.reads().iter().all(|read| matches!(
                read.demand,
                HeaderDemand::RequestedDefinition
                    | HeaderDemand::ParentChain
                    | HeaderDemand::HierarchyClosure
            )),
            "the reason set is exactly the demands a header closure may publish"
        );
    }

    #[test]
    fn a_parent_chain_walk_follows_the_superclass_and_ignores_interfaces() {
        // p/C extends p/Base and implements p/I; p/Base extends java/lang/Object.
        //
        // `ReadReason::ParentChain` names one step up the *type's* `super_class` chain, so this
        // walk reads the class chain and never an interface: it answers "what does this class
        // extend", which is a different question from the supertype closure the fan-out test
        // covers. The root layer is the class the caller named, so it keeps that caller's own
        // reason.
        let snapshot = open(zip(&[
            (
                b"p/C.class",
                &class_with(b"p/C", Some(b"p/Base"), &[b"p/I"], 52),
            ),
            (
                b"p/Base.class",
                &class_with(b"p/Base", Some(b"java/lang/Object"), &[], 52),
            ),
            (
                b"p/I.class",
                &class_with(b"p/I", Some(b"java/lang/Object"), &[], 52),
            ),
            (
                b"java/lang/Object.class",
                &class_with(b"java/lang/Object", None, &[], 52),
            ),
        ]));
        let environment = app_environment(&snapshot);
        let mut budget = Budget::new(closure_limits());
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);
        let mut walk = closure.parent_chain(b"p/C", HeaderDemand::RequestedDefinition);

        let mut expanded = Vec::new();
        walk_handles(&mut closure, &mut walk, &mut budget, &mut expanded)
            .expect("every class of the chain is in the root");

        assert_eq!(
            layers(&closure, &expanded),
            names(&[b"p/C", b"p/Base", b"java/lang/Object"]),
            "the parent chain is the superclass chain: the interface p/I is not on it"
        );
        assert_eq!(
            budget.usage().class_headers,
            3,
            "p/I is never read by a parent-chain walk, even though p/C implements it"
        );
        assert_eq!(closure.reads().len(), 3);
        assert_eq!(
            closure
                .reads()
                .iter()
                .map(|read| read.demand)
                .collect::<Vec<_>>(),
            vec![
                HeaderDemand::RequestedDefinition,
                HeaderDemand::ParentChain,
                HeaderDemand::ParentChain,
            ],
            "the root keeps the caller's reason and every step above it is a parent-chain read"
        );
        assert_eq!(
            walk.gaps(),
            &WalkGaps::default(),
            "nothing was missing, ambiguous or cyclic"
        );
        assert!(
            expanded
                .iter()
                .all(|handle| closure.resolution(*handle).name != JvmBytes(b"p/I".to_vec())),
            "no layer of this walk is the interface"
        );
    }

    #[test]
    fn a_stopped_demand_is_not_remembered_as_a_decision() {
        // Position 0 is a standalone CLASS root that declares another name: it is read — and
        // recorded — before the search continues. Position 1 holds the requested name, and a
        // one-attempt header budget refuses that read, so the demand stops without deciding.
        let other_bytes = class_bytes(b"other/O", 52);
        let other = open(other_bytes.clone());
        let bytes = class_bytes(b"p/S", 52);
        let snapshot = open(zip(&[(b"p/S.class", &bytes)]));
        let roots = vec![root_of(&other), root_of(&snapshot)];
        let environment = environment(
            domain("app", None, DelegationPolicy::ParentFirst, roots.clone()),
            vec![domain("app", None, DelegationPolicy::ParentFirst, roots)],
        );
        let content = [other, snapshot];
        let mut closure = HeaderClosure::new(&content, &environment);

        let mut tight = Budget::new(Limits {
            class_headers: 1,
            ..closure_limits()
        });
        let stop = closure
            .demand(b"p/S", HeaderDemand::RequestedDefinition, &mut tight)
            .decision
            .expect_err("the second read attempt is refused");
        assert!(
            matches!(
                stop,
                Error::BudgetExceeded {
                    dimension: BudgetDimension::ClassHeaders,
                    ..
                }
            ),
            "the stop keeps its own dimension: {stop:?}"
        );
        assert_eq!(
            closure.reads().len(),
            1,
            "the standalone root's read happened before the stop"
        );

        // The same key asked again searches again on the fresh budget: a stop decided nothing,
        // so it must not be remembered as `Missing`, as an empty closure, or as a memo hit.
        let mut fresh = Budget::new(closure_limits());
        let before = closure.searched_extent();
        let answer = closure.demand(b"p/S", HeaderDemand::RequestedDefinition, &mut fresh);
        assert!(
            closure.searched_extent().positions > before.positions,
            "the second demand performed a search of its own instead of reading the stop out \
             of the memo"
        );
        assert_eq!(
            fresh.usage().class_headers,
            2,
            "the retry really searched the order again"
        );
        let handle = answer
            .decision
            .expect("the retry reaches the decision the stop prevented");
        assert_eq!(
            closure.resolution(handle).lookup.state,
            HeaderLookupState::Found,
            "the retry decides the name; the stop did not invent a `Missing`"
        );
        assert_eq!(
            closure.reads().len(),
            2,
            "the retry records the candidate it read; the standalone root is already known, so \
             it is not recorded twice"
        );
    }

    #[test]
    fn an_ambiguous_supertype_is_a_gap_of_its_own_kind() {
        // p/C extends p/Base, and the root holds two indistinguishable p/Base entries: the
        // branch ends as `ambiguous` — its own gap field, not a missing or a cyclic one — and
        // both candidates are read and recorded.
        let snapshot = open(zip(&[
            (b"p/C.class", &class_with(b"p/C", Some(b"p/Base"), &[], 52)),
            (b"p/Base.class", &class_with(b"p/Base", None, &[], 52)),
            (b"p/Base.class", &class_with(b"p/Base", None, &[], 51)),
        ]));
        let environment = app_environment(&snapshot);
        let mut budget = Budget::new(closure_limits());
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);
        let mut walk = closure.hierarchy_closure(b"p/C", HeaderDemand::MemberOwner);

        let mut expanded = Vec::new();
        walk_handles(&mut closure, &mut walk, &mut budget, &mut expanded)
            .expect("an ambiguous name ends its branch without failing the request");

        assert_eq!(walk.gaps().ambiguous, names(&[b"p/Base"]));
        assert!(
            walk.gaps().missing.is_empty() && walk.gaps().cycles.is_empty(),
            "an ambiguous name is not a missing or a cyclic one: {:?}",
            walk.gaps()
        );
        assert_eq!(
            budget.usage().class_headers,
            3,
            "both candidates of the ambiguous position are read attempts"
        );
        assert_eq!(
            closure.reads().len(),
            3,
            "each candidate that was really read is recorded, the ambiguous ones included"
        );
        assert_eq!(
            layers(&closure, &expanded),
            names(&[b"p/C", b"p/Base"]),
            "the ambiguous layer is expanded once and its branch stops there"
        );
        assert_eq!(
            closure
                .resolution(*expanded.last().expect("the walk expanded a layer"))
                .lookup
                .state,
            HeaderLookupState::Ambiguous
        );
        assert!(closure.diagnostics().is_empty());
    }

    #[test]
    fn a_class_that_extends_itself_is_refused_with_a_diagnostic() {
        let snapshot = open(zip(&[(
            b"p/A.class",
            &class_with(b"p/A", Some(b"p/A"), &[], 52),
        )]));
        let environment = app_environment(&snapshot);
        let mut budget = Budget::new(closure_limits());
        let mut closure = HeaderClosure::new(std::slice::from_ref(&snapshot), &environment);
        let mut walk = closure.hierarchy_closure(b"p/A", HeaderDemand::MemberOwner);

        let mut expanded = Vec::new();
        walk_handles(&mut closure, &mut walk, &mut budget, &mut expanded)
            .expect("a self-extending class ends its branch, it does not fail the request");

        assert_eq!(
            layers(&closure, &expanded),
            names(&[b"p/A"]),
            "the walk terminates at the class that extends itself"
        );
        assert_eq!(walk.gaps().cycles, names(&[b"p/A"]));
        assert_eq!(
            budget.usage().class_headers,
            1,
            "the cycle is refused before a second read"
        );
        assert_eq!(closure.diagnostics().len(), 1);
        let diagnostic = &closure.diagnostics()[0];
        assert_eq!(diagnostic.code, "resolution_hierarchy_cycle");
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
        for expected in ["loader `app`", "p/A -> p/A", "`p/A` is its own supertype"] {
            assert!(
                diagnostic.message.contains(expected),
                "the diagnostic locates the cycle ({expected}): {}",
                diagnostic.message
            );
        }
    }
}
