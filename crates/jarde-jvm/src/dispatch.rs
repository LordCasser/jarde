//! Known-candidate dispatch inside one explicit physical scope (2.5).
//!
//! This module is the 2.5 slice of the demand resolver: given a declaration that 2.3 already
//! resolved and an explicit physical range, it enumerates the classes the range covers and
//! reports the ones that *override or implement* that declaration, each with the open-world
//! evidence the range states. It is CHA-lite: the subtype test is the class's own supertype
//! closure, the member test is the class's own declaration list, and no method body is read.
//!
//! The plane answers "which overrides of this declaration exist inside this range" — never
//! "this is the runtime target". That is why several candidates are an ordinary answer, why a
//! single candidate is not marked as unique, and why every candidate that stands under an
//! open-world fact says which fact it is:
//!
//! * the range's classes are enumerated by name — a ZIP entry `a/B.class` is the name `a/B`, a
//!   standalone CLASS root is the name its own `this_class` declares — and each name is demanded
//!   through the 2.2 closure, so every read is one `ClassHeaders` attempt recorded with the
//!   [`HeaderDemand::DispatchScope`] reason and the request memo keeps one read per
//!   `(definition, loader)`;
//! * a class the requested range does not hold — because the environment's order selects a
//!   definition outside it, or because no position provides the name at all — is an *undecided*
//!   position, not a candidate and not an exclusion: that part of the range was never searched;
//! * a class is a candidate when its supertype closure reaches the declaration's owner above
//!   itself (so the declaring class, and a same-named duplicate of it, are not their own
//!   override) and the class itself declares a member of the declaration's kind, name and
//!   descriptor;
//! * a class whose closure could not be read completely (a missing supertype, an ambiguous
//!   position, a cyclic hierarchy) leaves an unread branch, the candidate itself carries the
//!   `missing` gap as [`DispatchEvidence::MissingDependency`], and the name that could not be read
//!   is published with the demand that reached it (2.2) — a class nothing resolved is unsearched
//!   range, never an exclusion;
//! * `open_world` is true as soon as one candidate stands under any open-world fact, as soon as
//!   the range holds a position this request could not decide, and as soon as the plane stopped:
//!   a budget stop or a cancellation ends the range where its content becomes unknown.
//!
//! A stop is reported, never absorbed: the candidates already found are published as the
//! trustworthy prefix, the stop is handed back for the report layer to map onto `execution`, and
//! `open_world` stays true. The report layer also asks the stop whether it ended a *search*
//! ([`DispatchStop::ended_a_search`]) before it reads the closure's declared positions as
//! unfinished range: a refused charge for publishing a candidate is not one. Nothing here formats
//! a public report — the report layer owns the public vocabulary and maps these crate-private
//! mirrors exhaustively.

use crate::environment::ResolutionEnvironment;
use crate::members::MemberKind;
use crate::providers::{
    HeaderClosure, HeaderDemand, HeaderLookupState, HierarchyGap, HierarchyGapKind, NodeIdentity,
    WalkGaps,
};
use jarde_reader::artifact::{ArtifactKind, ArtifactSnapshot, NestedArchiveState, PhysicalEntry};
use jarde_reader::budget::{Budget, BudgetDimension, CountedBudgetDimension};
use jarde_reader::classfile::ClassFacts;
use jarde_reader::error::{Error, Result};
use jarde_reader::model::{
    Diagnostic, ExecutionReport, JvmBytes, PhysicalClassLocation, PhysicalDefinitionId, SnapshotId,
    SymbolRef,
};
use jarde_reader::view::{LoadRoot, LoaderId, PhysicalScope, RuntimeUncertainty};

/// One class of the range that overrides or implements the resolved declaration.
///
/// The member is the class's *own* declaration, so the report publishes a candidate definition;
/// the loader and the physical definition are those of the header the declaration was read from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DispatchCandidate {
    pub(crate) loader: LoaderId,
    pub(crate) definition: PhysicalDefinitionId,
    pub(crate) member: SymbolRef,
    /// The open-world fact this candidate stands under, or `None` when the plane has none.
    pub(crate) evidence: Option<DispatchEvidence>,
}

/// Crate-private mirror of the report's `OpenWorldEvidence`.
///
/// The dependency direction is `resolver -> dispatch`, so the public enum cannot live here; the
/// report layer maps this vocabulary exhaustively, which makes a variant added later fail to
/// compile until it is mapped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DispatchEvidence {
    /// The environment declares content the request cannot read (an external root, or a root
    /// whose snapshot the request does not provide): subclasses may live there unseen.
    ExternalSubclass,
    /// The environment declares a loader outside the caller's delegation chain: another loader
    /// may load another definition of the same class.
    UnknownLoader,
    /// A participating domain declares external override or runtime transformation: the runtime
    /// may hold a class this snapshot does not.
    RuntimeTransformation,
    /// The candidate's own supertype closure holds a name no position of the order provides.
    MissingDependency,
    /// The class's own lookup was decided at a declared root position other than the first, so
    /// the loader's ordered roots hold another position for the name's layer.
    OrderedRoot { index: u32 },
}

/// The resolved declaration a dispatch request enumerates overrides of.
///
/// The shape is the member vocabulary, not the report: `declaring` is the declaring class as a
/// **node** — its defining loader plus its physical definition — and `kind`, `name` and
/// `descriptor` are what a candidate has to declare to be an override of this declaration.
///
/// The subtype test compares nodes rather than the declaring class's *name*, because an owner
/// string is not evidence of inheritance: a class another loader defines under the same name is a
/// different class and overrides nothing here (0.1, JVMS 5.3/5.4.3.1).
pub(crate) struct DeclarationShape<'a> {
    pub(crate) kind: MemberKind,
    pub(crate) declaring: NodeIdentity,
    pub(crate) name: &'a [u8],
    pub(crate) descriptor: &'a [u8],
}

/// Why the plane stopped before it reached the end of the range.
#[derive(Clone, Debug)]
pub(crate) enum DispatchStop {
    /// The range could not be listed completely: the entries it did list are the prefix, and the
    /// listing's own execution says why it stopped.
    Truncated(ExecutionReport),
    /// A lookup, a walk or a refused charge ended the plane.
    Refused(Error),
}

impl DispatchStop {
    /// Whether this stop ended a **search** of the plane rather than the publication of a result
    /// it had already found.
    ///
    /// The difference decides whether the positions the order declared and never examined stay
    /// unfinished range: only a search the stop really cut short leaves such positions behind. A
    /// refused charge for publishing one candidate is this plane's *publication* phase — every
    /// search that ran reached its own conclusion, and a search stops by rule at the first
    /// position that holds the class it looked for — so publishing its remaining positions as
    /// skipped would state unsearched range that no search left behind.
    pub(crate) fn ended_a_search(&self) -> bool {
        match self {
            // The range's own listing stopped: the classes behind the listed prefix were never
            // searched at all, so the range this plane could reach is a prefix of the requested
            // one.
            Self::Truncated(_) => true,
            // The one refusal that is a publication: one candidate's `ResultItems` charge.
            Self::Refused(Error::BudgetExceeded {
                dimension: BudgetDimension::ResultItems,
                ..
            }) => false,
            // Every other refusal — a `ClassHeaders` or `DependencyDepth` charge, a cancellation,
            // a damaged class or a listing that raised — stopped the plane inside its search or
            // listing phase.
            Self::Refused(_) => true,
        }
    }
}

/// What one dispatch request produced.
#[derive(Clone, Debug)]
pub(crate) struct DispatchOutcome {
    /// The candidates found before any stop, in range order.
    pub(crate) candidates: Vec<DispatchCandidate>,
    pub(crate) open_world: bool,
    /// The range holds positions this request could not decide: a name no position provides, an
    /// ambiguous position, a definition the environment resolves outside the range, or a
    /// supertype branch that could not be read. Such a position is unsearched range, which is
    /// why it also makes the plane's coverage partial.
    pub(crate) undecided_positions: bool,
    /// The names this range needed and the order did not resolve, each with the demand that
    /// reached them: a class of the range the order provides no definition for (or whose
    /// definitions cannot be told apart), and a supertype branch of a class of the range that
    /// stayed unread. Such a name is unsearched range and never an exclusion, and publishing it by
    /// name is what keeps "this class could not be read" apart from "this class is not there".
    pub(crate) unread: WalkGaps,
    /// The stop that ended the plane, if any; publishing it as `execution` is the caller's job.
    pub(crate) stop: Option<DispatchStop>,
    /// Diagnostics the range enumeration itself produced (its own listing facts).
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl DispatchOutcome {
    fn refused(error: Error) -> Self {
        Self {
            candidates: Vec::new(),
            open_world: true,
            undecided_positions: true,
            unread: WalkGaps::default(),
            stop: Some(DispatchStop::Refused(error)),
            diagnostics: Vec::new(),
        }
    }
}

/// Enumerates the overrides of `declaration` inside `scope`.
///
/// `closure` is the request's own 2.2 closure, so the declaration resolution and this plane share
/// one memo and one read record. Every class the range covers is one `DispatchScope` demand, and
/// the walk above it uses the edge semantics of 2.3 — a `super_class` step is a parent-chain
/// read, an `interfaces` step is a hierarchy-closure read — with `DispatchScope` as the reason of
/// the walk's own root layer.
pub(crate) fn dispatch_candidates(
    content: &[ArtifactSnapshot],
    environment: &ResolutionEnvironment,
    closure: &mut HeaderClosure<'_>,
    scope: &PhysicalScope,
    declaration: &DeclarationShape<'_>,
    budget: &mut Budget,
) -> DispatchOutcome {
    let declared = declared_evidence(environment, content);
    let Some(snapshot) = content
        .iter()
        .find(|candidate| candidate.id() == &environment.runtime.physical.snapshot)
    else {
        // Unreachable from `Engine::resolve_symbol`, which refuses a snapshot the content does
        // not provide; a plane that cannot see its own range claims no candidate instead of
        // enumerating an unknown snapshot.
        return DispatchOutcome::refused(Error::invalid_input(
            "resolution_snapshot_mismatch",
            "the dispatch range names a snapshot the request content does not provide",
        ));
    };
    let range = match enumerate_range(snapshot, scope, budget) {
        Ok(range) => range,
        Err(error) => return DispatchOutcome::refused(error),
    };
    let mut outcome = DispatchOutcome {
        candidates: Vec::new(),
        open_world: false,
        undecided_positions: false,
        unread: WalkGaps::default(),
        // The listing runs first, so its own truncation is the first stop this plane knows: a
        // later refusal is a consequence of the same exhausted budget and must not replace it.
        stop: range.truncation.clone().map(DispatchStop::Truncated),
        diagnostics: range.diagnostics,
    };
    for name in &range.names {
        let step = match class_step(
            snapshot,
            scope,
            closure,
            declaration,
            declared,
            name,
            budget,
        ) {
            Ok(step) => step,
            Err(error) => {
                // A refusal ends the plane: the budget or the cancellation that produced it is
                // sticky, and the first stop is the one the report explains.
                if outcome.stop.is_none() {
                    outcome.stop = Some(DispatchStop::Refused(error));
                }
                break;
            }
        };
        outcome.open_world |= step.undecided;
        outcome.undecided_positions |= step.undecided;
        // A branch the plane could not read is one more open-world fact of this range, and its
        // record is published by name: the class it names is range this request never searched.
        outcome.open_world |= !step.gaps.is_empty();
        outcome.unread.absorb(&step.gaps);
        let Some(candidate) = step.candidate else {
            continue;
        };
        outcome.open_world |= candidate.evidence.is_some();
        // Publishing one candidate costs one `ResultItems`, charged before it is published, so a
        // refused charge keeps the candidates already published and stops the plane.
        if let Err(error) = budget.charge(CountedBudgetDimension::ResultItems, 1) {
            if outcome.stop.is_none() {
                outcome.stop = Some(DispatchStop::Refused(error));
            }
            break;
        }
        outcome.candidates.push(candidate);
    }
    if outcome.stop.is_some() {
        // A stopped plane covers an unknown part of the range, which is itself the open-world
        // fact the caller must not read as a complete answer.
        outcome.open_world = true;
    }
    outcome
}

/// The open-world facts the request's own declarations state.
///
/// The classification is a pure function of the environment and of the provided content: it
/// reads no artifact byte and it never decides a candidate. `external_content` names the
/// declarations 1.1 rejects wholesale, so a request that performs at all never carries it today;
/// it stays classified here because the evidence vocabulary belongs to the range, not to this
/// slice, and a later per-position gate can make it reachable without redesigning the plane.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DeclaredEvidence {
    /// Some declared root is an external one or names content the request does not provide.
    pub(crate) external_content: bool,
    /// Some declared loader is not on the caller's delegation chain.
    pub(crate) other_loaders: bool,
    /// A participating domain declares external override or runtime transformation.
    pub(crate) runtime_uncertainty: bool,
}

pub(crate) fn declared_evidence(
    environment: &ResolutionEnvironment,
    content: &[ArtifactSnapshot],
) -> DeclaredEvidence {
    let participating = participating_loaders(environment);
    let mut declared = DeclaredEvidence {
        external_content: false,
        other_loaders: false,
        runtime_uncertainty: false,
    };
    let roots = environment
        .domains
        .iter()
        .flat_map(|domain| domain.roots.iter())
        .chain(
            environment
                .providers
                .iter()
                .flat_map(|provider| provider.roots.iter()),
        );
    for root in roots {
        declared.external_content |= match root {
            LoadRoot::External { .. } => true,
            LoadRoot::Snapshot { snapshot } => !provides(content, snapshot),
            LoadRoot::ArtifactTree { root } => !provides(content, &root.snapshot),
        };
    }
    for domain in &environment.domains {
        let on_chain = participating.contains(&&domain.loader);
        declared.other_loaders |= !on_chain;
        declared.runtime_uncertainty |= on_chain
            && (domain.external_override != RuntimeUncertainty::None
                || domain.runtime_transformation != RuntimeUncertainty::None);
    }
    declared
}

/// The loaders the caller's own delegation chain reaches, caller first.
///
/// A chain that cannot be walked (a missing parent, a cycle) ends where 1.1 would refuse it; the
/// classification only needs the set, so an unusable chain contributes the loaders it declared.
fn participating_loaders(environment: &ResolutionEnvironment) -> Vec<&LoaderId> {
    let mut chain: Vec<&LoaderId> = Vec::new();
    let mut loader = &environment.runtime.load_domain.loader;
    loop {
        if chain.contains(&loader) {
            break;
        }
        chain.push(loader);
        let Some(domain) = environment
            .domains
            .iter()
            .find(|domain| &domain.loader == loader)
        else {
            break;
        };
        match &domain.parent_loader {
            Some(parent) => loader = parent,
            None => break,
        }
    }
    chain
}

fn provides(content: &[ArtifactSnapshot], snapshot: &SnapshotId) -> bool {
    content.iter().any(|candidate| candidate.id() == snapshot)
}

/// The classes one explicit range covers, in range order.
struct RangeListing {
    /// Raw internal names, first occurrence first: a name two containers both hold is one class
    /// of the range, and the environment's order decides which definition it resolves to.
    names: Vec<JvmBytes>,
    /// The listing's own execution when it is not complete.
    truncation: Option<ExecutionReport>,
    diagnostics: Vec<Diagnostic>,
}

/// Lists the class names `scope` covers.
///
/// The scope vocabulary is the P1 one, so the covered names are the ones `ProviderScan` would
/// visit for the same view: `SnapshotAll` is the snapshot's root container — for a standalone
/// CLASS root, the class itself — and `ArtifactTree` is the container tree. A nested container is
/// a class of the range only when the tree lists it, exactly like 2.4's declaration scan.
fn enumerate_range(
    snapshot: &ArtifactSnapshot,
    scope: &PhysicalScope,
    budget: &mut Budget,
) -> Result<RangeListing> {
    match (snapshot.kind(), scope) {
        (ArtifactKind::StandaloneClass, PhysicalScope::SnapshotAll) => {
            // A standalone CLASS root has no path-derived name, so its header is read once to
            // learn the name it declares for itself; `class_step` then demands that name, which
            // is what reads the root through the closure and records the definition and reason.
            let bytes = snapshot.root_bytes(budget)?;
            let facts = jarde_reader::classfile::class_facts(&bytes, budget)?;
            Ok(RangeListing {
                names: vec![JvmBytes(facts.this_class.raw().0.clone())],
                truncation: None,
                diagnostics: Vec::new(),
            })
        }
        (ArtifactKind::Zip, PhysicalScope::SnapshotAll) => {
            let report = snapshot.enumerate(budget)?;
            Ok(RangeListing {
                names: class_names(report.entries.iter()),
                truncation: truncation_of(&report.execution),
                diagnostics: report.diagnostics,
            })
        }
        (ArtifactKind::Zip, PhysicalScope::ArtifactTree { .. }) => {
            let report = snapshot.enumerate_artifact_tree(budget)?;
            let names = class_names(
                report
                    .containers
                    .iter()
                    .flat_map(|container| container.entries.iter()),
            );
            Ok(RangeListing {
                names,
                truncation: truncation_of(&report.execution),
                diagnostics: report.diagnostics,
            })
        }
        // An artifact tree needs a ZIP snapshot, and the range is refused with the artifact
        // layer's own code instead of being reported as an empty tree.
        (ArtifactKind::StandaloneClass, PhysicalScope::ArtifactTree { .. }) => {
            Err(Error::invalid_input(
                "not_zip",
                "an artifact-tree dispatch range requires a ZIP snapshot",
            ))
        }
    }
}

fn truncation_of(execution: &ExecutionReport) -> Option<ExecutionReport> {
    match execution {
        ExecutionReport::Complete { .. } => None,
        other => Some(other.clone()),
    }
}

/// The class names a listing's entries hold, in listing order.
///
/// The discipline is the 2.1 candidate discipline: a class entry is a non-directory raw name
/// ending in `.class`, compared byte-exactly, and a nested-archive candidate is not a class even
/// when its name ends in `.class`. No entry is read here.
fn class_names<'a>(entries: impl Iterator<Item = &'a PhysicalEntry>) -> Vec<JvmBytes> {
    let mut names: Vec<JvmBytes> = Vec::new();
    let mut seen: Vec<Vec<u8>> = Vec::new();
    for entry in entries {
        if entry.nested_archive != NestedArchiveState::NotCandidate {
            continue;
        }
        let raw = entry.id.raw_name.0.as_slice();
        let Some(name) = raw.strip_suffix(b".class") else {
            continue;
        };
        if name.is_empty() || seen.iter().any(|known| known == name) {
            continue;
        }
        seen.push(name.to_vec());
        names.push(JvmBytes(name.to_vec()));
    }
    names
}

/// What one class of the range contributed.
struct ClassStep {
    candidate: Option<DispatchCandidate>,
    /// The range's position for this name could not be decided.
    undecided: bool,
    /// The branches this step could not read, with the demand that reached them: the class's own
    /// unread supertype branch, or the class's own name when the order resolved no definition for
    /// it. A position the environment resolved *outside* the range is undecided too and records no
    /// name here — the name did resolve, so no dependency of this request is unread for it.
    gaps: WalkGaps,
}

impl ClassStep {
    /// A range position the order could not resolve, named by the kind of gap it stated.
    fn unread(kind: HierarchyGapKind, name: &JvmBytes, loader: &LoaderId) -> Self {
        let mut gaps = WalkGaps::default();
        gaps.record(
            kind,
            HierarchyGap {
                name: name.clone(),
                loader: loader.clone(),
                // The demand of this read is the range's own enumeration: the class is asked for
                // because it is one class of the requested range, and no hierarchy edge reached it.
                demand: HeaderDemand::DispatchScope,
                declared_by: None,
            },
        );
        Self {
            candidate: None,
            undecided: true,
            gaps,
        }
    }

    /// A range position the environment resolved to a definition outside the requested range.
    fn outside_range() -> Self {
        Self {
            candidate: None,
            undecided: true,
            gaps: WalkGaps::default(),
        }
    }
}

/// The gap one lookup state states for the name that was demanded, or `None` when it found one.
fn unread_kind(state: HeaderLookupState) -> Option<HierarchyGapKind> {
    match state {
        HeaderLookupState::Found => None,
        HeaderLookupState::Missing => Some(HierarchyGapKind::Missing),
        HeaderLookupState::Ambiguous => Some(HierarchyGapKind::Ambiguous),
    }
}

/// Decides one class of the range.
fn class_step(
    snapshot: &ArtifactSnapshot,
    scope: &PhysicalScope,
    closure: &mut HeaderClosure<'_>,
    declaration: &DeclarationShape<'_>,
    declared: DeclaredEvidence,
    name: &JvmBytes,
    budget: &mut Budget,
) -> Result<ClassStep> {
    // A class of the requested range is a symbol request the **caller** issues, so its order is
    // the caller's own; the walk above it switches to each declaring class's defining loader.
    let handle = closure
        .demand_from_caller(&name.0, HeaderDemand::DispatchScope, budget)
        .decision?;
    let resolution = closure.resolution(handle);
    if let Some(kind) = unread_kind(resolution.lookup.state) {
        // No position of the order provides this name, or several positions cannot be told
        // apart: the range holds a class this request could not read, so nothing is claimed —
        // and the name that could not be read is published, never turned into an exclusion.
        let loader = closure.caller_loader().clone();
        return Ok(ClassStep::unread(kind, name, &loader));
    }
    let location = resolution
        .lookup
        .location
        .as_ref()
        .expect("a found lookup publishes its position");
    if !covered_by_scope(snapshot.id(), scope, &location.definition) {
        // The environment's order selects a definition outside the requested range; the class of
        // the range is shadowed for this request, and an unread position is not an answer.
        return Ok(ClassStep::outside_range());
    }
    let header = resolution
        .lookup
        .header
        .as_ref()
        .expect("a found lookup publishes its header");
    let declares = declares_member(&header.facts, declaration);
    let this_class = header.facts.this_class.raw().0.clone();
    let loader = resolution
        .defining_loader()
        .expect("a found lookup selects a definition")
        .clone();
    let definition = location.definition.clone();
    let root_index = location.root_index;
    let self_node = NodeIdentity::new(&loader, &definition);
    let caller = closure.caller_loader().clone();
    let mut walk = closure.hierarchy_closure(&caller, &name.0, HeaderDemand::DispatchScope);
    let mut owner_above = false;
    while let Some(layer) = walk.next(closure, budget)? {
        // The declaring class itself — this exact loader and definition — must be on this class's
        // supertype path *above* the class for the class to override or implement it. The class's
        // own node is excluded, which is why a class is never its own override, and a same-named
        // class that another loader defines never qualifies either: an owner string is not
        // inheritance evidence (0.1).
        owner_above |= closure
            .identity(layer)
            .is_some_and(|node| node != self_node && node == declaration.declaring);
    }
    let mut gaps = WalkGaps::default();
    gaps.absorb(walk.gaps());
    let candidate = (declares && owner_above).then(|| DispatchCandidate {
        loader,
        definition,
        member: member_of(&this_class, declaration),
        evidence: candidate_evidence(
            declared,
            !gaps.any_of(HierarchyGapKind::Missing),
            root_index,
        ),
    });
    Ok(ClassStep {
        candidate,
        undecided: false,
        gaps,
    })
}

/// The open-world fact one candidate stands under, in the fixed priority the report publishes.
///
/// The candidate's own missing supertype is the most specific fact about *that* candidate, the
/// declared content and loader facts come next, and the ordered-root hint is the weakest: the
/// class's own lookup was decided at a declared root position other than the first, so this
/// loader's ordered roots really hold more than one position for the name's layer — a candidate
/// found at the very first position states nothing of that kind, because no root precedes it.
/// A candidate the plane can speak about completely carries no evidence at all, which is what
/// lets a complete range answer `open_world = false` instead of always claiming an open world.
fn candidate_evidence(
    declared: DeclaredEvidence,
    no_missing_supertype: bool,
    root_index: u32,
) -> Option<DispatchEvidence> {
    if !no_missing_supertype {
        return Some(DispatchEvidence::MissingDependency);
    }
    if declared.external_content {
        return Some(DispatchEvidence::ExternalSubclass);
    }
    if declared.other_loaders {
        return Some(DispatchEvidence::UnknownLoader);
    }
    if declared.runtime_uncertainty {
        return Some(DispatchEvidence::RuntimeTransformation);
    }
    if root_index > 0 {
        return Some(DispatchEvidence::OrderedRoot { index: root_index });
    }
    None
}

/// Whether one physical definition lives inside the requested range.
///
/// `SnapshotAll` covers the snapshot's root container — a standalone CLASS root *is* that
/// container — and `ArtifactTree` covers the containers of that snapshot's tree. A position of
/// another snapshot is never inside the range, even when its container name matches.
fn covered_by_scope(
    snapshot: &SnapshotId,
    scope: &PhysicalScope,
    definition: &PhysicalDefinitionId,
) -> bool {
    if definition.location.snapshot() != snapshot {
        return false;
    }
    match (scope, &definition.location) {
        (PhysicalScope::SnapshotAll, PhysicalClassLocation::StandaloneRoot { .. }) => true,
        (PhysicalScope::SnapshotAll, PhysicalClassLocation::ArchiveEntry { entry }) => {
            entry.origin.steps.is_empty()
        }
        (
            PhysicalScope::ArtifactTree { root_container },
            PhysicalClassLocation::ArchiveEntry { entry },
        ) => &entry.origin.root_container == root_container,
        (PhysicalScope::ArtifactTree { .. }, PhysicalClassLocation::StandaloneRoot { .. }) => false,
    }
}

/// Whether one class declares the member the declaration names.
///
/// The comparison is byte-exact on the name and the descriptor and exact on the member kind: a
/// field never overrides a method, and the bytes the report publishes are the bytes it read.
fn declares_member(facts: &ClassFacts, declaration: &DeclarationShape<'_>) -> bool {
    let declared = match declaration.kind {
        MemberKind::Field => &facts.fields,
        MemberKind::Method => &facts.methods,
    };
    declared.iter().any(|member| {
        member.name.raw().0.as_slice() == declaration.name
            && member.descriptor.raw().0.as_slice() == declaration.descriptor
    })
}

/// The member symbol of one candidate: the class's own internal name with the declared member.
fn member_of(this_class: &[u8], declaration: &DeclarationShape<'_>) -> SymbolRef {
    let owner = JvmBytes(this_class.to_vec());
    let name = JvmBytes(declaration.name.to_vec());
    let descriptor = JvmBytes(declaration.descriptor.to_vec());
    match declaration.kind {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::{HeaderProvider, ProviderId};
    use jarde_reader::model::{
        ClassBytesId, ContainerId, ContainerOrigin, ContainerOriginStep, Digest, PhysicalEntryId,
    };
    use jarde_reader::view::{
        DelegationPolicy, LayoutMode, LoadDomain, ModuleMode, MultiReleasePolicy, PhysicalView,
        RuntimeProfile, RuntimeView,
    };

    fn loader(name: &str) -> LoaderId {
        LoaderId(name.to_string())
    }

    fn domain(loader: &LoaderId, parent: Option<LoaderId>, roots: Vec<LoadRoot>) -> LoadDomain {
        LoadDomain {
            loader: loader.clone(),
            parent_loader: parent,
            delegation: DelegationPolicy::ParentFirst,
            roots,
            module_mode: ModuleMode::ClassPath,
            external_override: RuntimeUncertainty::None,
            runtime_transformation: RuntimeUncertainty::None,
        }
    }

    fn environment(
        caller: LoadDomain,
        domains: Vec<LoadDomain>,
        providers: Vec<HeaderProvider>,
    ) -> ResolutionEnvironment {
        ResolutionEnvironment {
            runtime: RuntimeView {
                physical: PhysicalView {
                    snapshot: SnapshotId("snap".to_string()),
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
            providers,
        }
    }

    fn root_container(snapshot: &str, container: &str) -> ContainerOrigin {
        ContainerOrigin {
            snapshot: SnapshotId(snapshot.to_string()),
            root_container: ContainerId(container.to_string()),
            steps: Vec::new(),
        }
    }

    fn entry(snapshot: &str, steps: Vec<ContainerOriginStep>) -> PhysicalEntryId {
        PhysicalEntryId {
            origin: ContainerOrigin {
                snapshot: SnapshotId(snapshot.to_string()),
                root_container: ContainerId("root".to_string()),
                steps,
            },
            ordinal: 0,
            raw_name: jarde_reader::model::ArchiveNameBytes(b"p/A.class".to_vec()),
        }
    }

    fn definition(location: PhysicalClassLocation) -> PhysicalDefinitionId {
        PhysicalDefinitionId {
            location,
            class_bytes: ClassBytesId {
                digest: Digest("0".repeat(64)),
                length: 1,
            },
            variant: jarde_reader::model::PhysicalVariant::Base,
        }
    }

    /// The external and unprovided root declarations classify as external content.
    #[test]
    fn external_and_unprovided_roots_are_external_content() {
        let app = loader("app");
        let cases: Vec<(LoadDomain, bool)> = vec![
            (
                domain(
                    &app,
                    None,
                    vec![LoadRoot::External {
                        id: "boot".to_string(),
                    }],
                ),
                true,
            ),
            (
                domain(
                    &app,
                    None,
                    vec![LoadRoot::Snapshot {
                        snapshot: SnapshotId("missing".to_string()),
                    }],
                ),
                true,
            ),
            (
                domain(
                    &app,
                    None,
                    vec![LoadRoot::ArtifactTree {
                        root: root_container("missing", "root"),
                    }],
                ),
                true,
            ),
            // No root at all: a loader without positions states nothing about unseen content.
            (domain(&app, None, Vec::new()), false),
        ];
        for (caller, expected) in cases {
            let declared =
                declared_evidence(&environment(caller.clone(), vec![caller], Vec::new()), &[]);
            assert_eq!(
                declared.external_content, expected,
                "the classification follows the declared roots"
            );
            assert!(!declared.other_loaders);
            assert!(!declared.runtime_uncertainty);
        }

        // A provider declares content too: its roots are classified even without a domain that
        // lists them (binding them is the validator's job, not this classification's).
        let caller = domain(&app, None, Vec::new());
        let declared = declared_evidence(
            &environment(
                caller.clone(),
                vec![caller],
                vec![HeaderProvider {
                    id: ProviderId("platform".to_string()),
                    roots: vec![LoadRoot::External {
                        id: "boot".to_string(),
                    }],
                }],
            ),
            &[],
        );
        assert!(declared.external_content);
    }

    /// A declared loader outside the caller's chain is the one observable other-loader fact.
    #[test]
    fn a_domain_outside_the_chain_is_the_other_loader_fact() {
        let app = loader("app");
        let platform = loader("platform");
        let caller = domain(&app, None, Vec::new());
        let declared = declared_evidence(
            &environment(
                caller.clone(),
                vec![caller, domain(&platform, None, Vec::new())],
                Vec::new(),
            ),
            &[],
        );
        assert!(declared.other_loaders, "the dangling loader is declared");
        assert!(!declared.external_content);
        assert!(!declared.runtime_uncertainty);

        // The chain only: a parent of the caller participates, a second parent of that parent
        // is still on the chain and therefore not an unknown loader.
        let parent = loader("platform");
        let caller = domain(&app, Some(parent.clone()), Vec::new());
        let declared = declared_evidence(
            &environment(
                caller.clone(),
                vec![caller, domain(&parent, None, Vec::new())],
                Vec::new(),
            ),
            &[],
        );
        assert!(!declared.other_loaders);
    }

    /// Runtime uncertainty counts for the domains this request searches, and only for them.
    #[test]
    fn runtime_uncertainty_is_read_from_the_participating_domains() {
        let app = loader("app");
        let platform = loader("platform");

        let mut uncertain = domain(&app, None, Vec::new());
        uncertain.external_override = RuntimeUncertainty::Possible;
        let declared = declared_evidence(
            &environment(uncertain.clone(), vec![uncertain], Vec::new()),
            &[],
        );
        assert!(declared.runtime_uncertainty);
        assert!(!declared.external_content);
        assert!(!declared.other_loaders);

        let mut transformed = domain(&app, None, Vec::new());
        transformed.runtime_transformation = RuntimeUncertainty::Unknown;
        let declared = declared_evidence(
            &environment(transformed.clone(), vec![transformed], Vec::new()),
            &[],
        );
        assert!(declared.runtime_uncertainty);

        // The uncertainty of a loader this request never searches is not this answer's fact: it
        // is the other-loader fact instead.
        let caller = domain(&app, None, Vec::new());
        let mut dangling = domain(&platform, None, Vec::new());
        dangling.runtime_transformation = RuntimeUncertainty::Unknown;
        let declared = declared_evidence(
            &environment(caller.clone(), vec![caller, dangling], Vec::new()),
            &[],
        );
        assert!(!declared.runtime_uncertainty);
        assert!(declared.other_loaders);
    }

    /// The evidence priority: the candidate's own gap first, then the range's facts, and the
    /// ordered-root hint only for a position other than the first of its layer.
    #[test]
    fn candidate_evidence_names_the_strongest_applicable_fact() {
        let clean = DeclaredEvidence::default();
        let all_declared = DeclaredEvidence {
            external_content: true,
            other_loaders: true,
            runtime_uncertainty: true,
        };

        assert_eq!(candidate_evidence(clean, true, 0), None);
        assert_eq!(
            candidate_evidence(clean, true, 2),
            Some(DispatchEvidence::OrderedRoot { index: 2 }),
            "the class was decided at a declared root position other than the first"
        );
        assert_eq!(
            candidate_evidence(all_declared, true, 0),
            Some(DispatchEvidence::ExternalSubclass)
        );
        assert_eq!(
            candidate_evidence(
                DeclaredEvidence {
                    external_content: false,
                    other_loaders: true,
                    runtime_uncertainty: true,
                },
                true,
                0
            ),
            Some(DispatchEvidence::UnknownLoader)
        );
        assert_eq!(
            candidate_evidence(
                DeclaredEvidence {
                    external_content: false,
                    other_loaders: false,
                    runtime_uncertainty: true,
                },
                true,
                0
            ),
            Some(DispatchEvidence::RuntimeTransformation)
        );
        assert_eq!(
            candidate_evidence(
                DeclaredEvidence {
                    external_content: false,
                    other_loaders: false,
                    runtime_uncertainty: false,
                },
                false,
                0
            ),
            Some(DispatchEvidence::MissingDependency),
            "the candidate's own unread supertype outranks every range fact"
        );
    }

    /// The range vocabulary of 2.5 is the P1 one: a flat root container, or the tree.
    #[test]
    fn covered_by_scope_follows_the_p1_range_vocabulary() {
        let snapshot = SnapshotId("snap".to_string());
        let other = SnapshotId("other".to_string());
        let nested = vec![ContainerOriginStep {
            via_ordinal: 0,
            via_raw_name: jarde_reader::model::ArchiveNameBytes(b"lib/inner.jar".to_vec()),
            child_container: ContainerId("inner".to_string()),
        }];
        let cases = [
            (
                definition(PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.clone(),
                }),
                PhysicalScope::SnapshotAll,
                true,
            ),
            (
                definition(PhysicalClassLocation::ArchiveEntry {
                    entry: entry("snap", Vec::new()),
                }),
                PhysicalScope::SnapshotAll,
                true,
            ),
            (
                definition(PhysicalClassLocation::ArchiveEntry {
                    entry: entry("snap", nested.clone()),
                }),
                PhysicalScope::SnapshotAll,
                false,
            ),
            (
                definition(PhysicalClassLocation::ArchiveEntry {
                    entry: entry("snap", nested.clone()),
                }),
                PhysicalScope::ArtifactTree {
                    root_container: ContainerId("root".to_string()),
                },
                true,
            ),
            (
                definition(PhysicalClassLocation::ArchiveEntry {
                    entry: entry("snap", Vec::new()),
                }),
                PhysicalScope::ArtifactTree {
                    root_container: ContainerId("other".to_string()),
                },
                false,
            ),
            (
                definition(PhysicalClassLocation::ArchiveEntry {
                    entry: entry("other", Vec::new()),
                }),
                PhysicalScope::SnapshotAll,
                false,
            ),
            (
                definition(PhysicalClassLocation::StandaloneRoot {
                    snapshot: snapshot.clone(),
                }),
                PhysicalScope::ArtifactTree {
                    root_container: ContainerId("root".to_string()),
                },
                false,
            ),
        ];
        for (definition, scope, expected) in cases {
            assert_eq!(
                covered_by_scope(&snapshot, &scope, &definition),
                expected,
                "range {scope:?} for {definition:?}"
            );
        }
        assert!(!covered_by_scope(
            &other,
            &PhysicalScope::SnapshotAll,
            &definition(PhysicalClassLocation::StandaloneRoot { snapshot })
        ));
    }

    /// The declared-evidence struct is total: every combination is representable.
    #[test]
    fn declared_evidence_defaults_to_no_fact() {
        let declared = DeclaredEvidence::default();
        assert!(!declared.external_content);
        assert!(!declared.other_loaders);
        assert!(!declared.runtime_uncertainty);
    }
}
