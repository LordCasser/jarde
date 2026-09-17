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

use crate::artifact::{ArtifactKind, ArtifactSnapshot, PhysicalEntry};
use crate::budget::{Budget, BudgetDimension, CountedBudgetDimension, Limits, UsageSnapshot};
use crate::classfile::{ClassFacts, class_facts};
use crate::environment::{EnvironmentProblemCode, ResolutionEnvironment};
use crate::error::{Error, Result};
use crate::model::{
    ClassBytesId, ContainerOrigin, Diagnostic, Digest, ExecutionReport, PhysicalClassLocation,
    PhysicalDefinitionId, PhysicalEntryId, PhysicalVariant, SnapshotId, TerminationReason,
    physical_variant_for_path,
};
use crate::view::{DelegationPolicy, LoadDomain, LoadRoot, LoaderId, ModuleMode};

/// Suffix every archive entry of a class carries; the comparison is byte-exact.
const CLASS_SUFFIX: &[u8] = b".class";

/// One physical position a class name was found at.
///
/// The position is a declaration coordinate: the loader that owns the root, the root's index
/// in that loader's `roots`, and the definition the position selected.
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
}

impl HeaderSearch {
    /// A refusal that no position was even derived for: the order itself is unusable.
    fn refused(error: Error) -> Self {
        Self {
            lookup: Err(error),
            examined: 0,
            positions: 0,
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

    let mut examined = 0_u32;
    for position in &positions {
        let decided = match probe_root(content, position, internal_name, &expected, budget) {
            Ok(decided) => decided,
            Err(error) => {
                return HeaderSearch {
                    lookup: Err(error),
                    examined,
                    positions: total,
                };
            }
        };
        examined = examined.saturating_add(1);
        if let Some(lookup) = decided {
            return HeaderSearch {
                lookup: Ok(lookup),
                examined,
                positions: total,
            };
        }
    }
    HeaderSearch {
        lookup: Ok(HeaderLookup::missing()),
        examined,
        positions: total,
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
/// decided at all.
fn probe_root(
    content: &[ArtifactSnapshot],
    position: &SearchPosition<'_>,
    internal_name: &[u8],
    entry_name: &[u8],
    budget: &mut Budget,
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
                decide(snapshot, position, candidates, budget)
            }
            ArtifactKind::StandaloneClass => {
                standalone_probe(snapshot, position, internal_name, budget)
            }
        },
        LoadRoot::ArtifactTree { root } => {
            let candidates = tree_candidates(snapshot, root, entry_name, &label, budget)?;
            decide(snapshot, position, candidates, budget)
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
/// that position's failure and the search does not continue past it.
fn decide(
    snapshot: &ArtifactSnapshot,
    position: &SearchPosition<'_>,
    candidates: Vec<PhysicalEntry>,
    budget: &mut Budget,
) -> Result<Option<HeaderLookup>> {
    match candidates.as_slice() {
        [] => Ok(None),
        [entry] => {
            let content = read_candidate(snapshot, position, entry, budget)?;
            let facts = class_facts(&content.bytes, budget)
                .map_err(|error| at_origin(error, &content.origin))?;
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
/// position; the read attempt is charged either way.
fn standalone_probe(
    snapshot: &ArtifactSnapshot,
    position: &SearchPosition<'_>,
    internal_name: &[u8],
    budget: &mut Budget,
) -> Result<Option<HeaderLookup>> {
    let origin = format!("{} (standalone CLASS root)", position_label(position));
    charge_header_attempt(budget)?;
    let bytes = snapshot
        .root_bytes(budget)
        .map_err(|error| at_origin(error, &origin))?;
    let facts = class_facts(&bytes, budget).map_err(|error| at_origin(error, &origin))?;
    if facts.this_class.raw().0 != internal_name {
        return Ok(None);
    }
    let length = u64::try_from(bytes.len()).map_err(|_| {
        Error::invalid_input("class_size_overflow", "class length does not fit u64")
    })?;
    Ok(Some(HeaderLookup::found(
        HeaderLocation {
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
        },
        ClassHeaderFacts { facts },
    )))
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
fn escaped(raw: &[u8]) -> String {
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
    use crate::budget::Limits;
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
        let mut pool = Vec::new();
        pool.push(1_u8); // CONSTANT_Utf8 this_class
        pool.extend_from_slice(
            &u16::try_from(this_class.len())
                .expect("name fits u16")
                .to_be_bytes(),
        );
        pool.extend_from_slice(this_class);
        pool.extend_from_slice(&[7, 0, 1]); // CONSTANT_Class #1
        pool.push(1_u8); // CONSTANT_Utf8 "java/lang/Object"
        pool.extend_from_slice(&16_u16.to_be_bytes());
        pool.extend_from_slice(b"java/lang/Object");
        pool.extend_from_slice(&[7, 0, 3]); // CONSTANT_Class #3

        let mut bytes = 0xcafebabe_u32.to_be_bytes().to_vec();
        bytes.extend_from_slice(&0_u16.to_be_bytes());
        bytes.extend_from_slice(&major.to_be_bytes());
        bytes.extend_from_slice(&5_u16.to_be_bytes()); // constant_pool_count
        bytes.extend_from_slice(&pool);
        bytes.extend_from_slice(&0x0021_u16.to_be_bytes()); // ACC_PUBLIC | ACC_SUPER
        bytes.extend_from_slice(&2_u16.to_be_bytes()); // this_class
        bytes.extend_from_slice(&4_u16.to_be_bytes()); // super_class
        for _ in 0..4 {
            bytes.extend_from_slice(&0_u16.to_be_bytes()); // interfaces, fields, methods, attributes
        }
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
}
