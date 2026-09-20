//! One physical scan, many runtime profiles, no collapsed answer (`P4` 2.1).
//!
//! A [`RuntimeMatrix`] answers "what would this snapshot select under each of these profiles"
//! without ever letting one profile stand for the rest. Three properties carry that contract:
//!
//! - **The physical scan is shared.** [`physical_evidence`](crate::multi_release::physical_evidence)
//!   runs **once** for the whole batch and every profile's selection is computed over that same
//!   evidence by [`select_over_physical`](crate::multi_release::select_over_physical) — the existing
//!   selection, not a second implementation of it (P4 design decision 2).
//! - **Every view is reported, separately.** A profile that cannot decide does not inherit another
//!   profile's answer: `MultiReleasePolicy::Custom`/`Unknown` keep their existing
//!   `unsupported_policy` verdict and appear as [`RuntimeSelectionRule::Unknown`], never as a
//!   concrete selection.
//! - **Profiles merge only where the merge is a proof.** [`SelectionGroup`] compression groups
//!   profiles whose selection is *identical* **and** whose own selection is a function of the
//!   target release that is constant on one interval (see [`StableInterval`]). Profiles that agree
//!   without such an interval are reported as separate groups: agreement is not a licence to claim
//!   that the choice holds for every version (P4 risk 2).
//!
//! Physical fullness is preserved too: the matrix keeps the whole enumeration once
//! ([`RuntimeMatrix::physical`], every entry of every container) and each view keeps the per-entry
//! decisions of the reused selection, so "which entries were not chosen, and by which rule" is
//! answerable without re-reading the archive.

use crate::artifact::{ArtifactSnapshot, LayoutNodeKind, LayoutNodeSource, PhysicalEntry};
use crate::budget::{Budget, UsageSnapshot};
use crate::error::{Error, Result};
use crate::model::{
    ArchiveNameBytes, ContainerOrigin, Coverage, CoverageDimension, CoverageRange, CoverageState,
    DiagnosticSeverity, ExecutionReport, PhysicalEntryId, Provenance,
};
use crate::multi_release::{
    self, Issues, ManifestState, MultiReleaseCompliance, MultiReleaseContainerReport,
    MultiReleaseEntryVariant, MultiReleaseNoSelectionReason, MultiReleasePhysicalEvidence,
    MultiReleaseReportDiagnostic, MultiReleaseSelection, MultiReleaseSelectionDecision,
    MultiReleaseUnknownReason, MultiReleaseViewReport,
};
use crate::view::{
    DelegationPolicy, LayoutMode, LoadDomain, LoadRoot, LoaderId, ModuleMode, MultiReleasePolicy,
    PhysicalView, RuntimeProfile, RuntimeUncertainty, RuntimeView,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// How many profiles one matrix may compare.
///
/// P4 risk 2 asks for an explicit profile ceiling rather than an unbounded cross product: the
/// shared scan is one enumeration, but every profile adds its own selection, projections and
/// diagnostics, and a request that wants more views than this is asking the matrix to be a
/// per-release sweep, which is a different (and much larger) report.
pub const MAX_PROFILES: usize = 8;

/// One matrix request: a physical view, the profiles to compare, and the loader graph to order
/// physical origins through.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMatrixRequest {
    pub physical: PhysicalView,
    /// At least one profile, at most [`MAX_PROFILES`], and no two identical: a repeated profile
    /// would report one view twice while reading like several.
    pub profiles: Vec<RuntimeProfile>,
    /// Every load domain whose roots the query may order, including [`Self::requester`].
    pub domains: Vec<LoadDomain>,
    /// The loader the query is asked from: the domain whose search path decides order.
    pub requester: LoaderId,
}

/// The result of one shared physical scan compared across profiles.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMatrix {
    /// **One** enumeration, shared by every view below.
    pub physical: MultiReleasePhysicalEvidence,
    /// The declared loader graph, in search order, with each domain's policy verdicts.
    pub domain_graph: DomainGraph,
    /// One entry per requested profile, in request order. Every requested profile is present.
    pub profiles: Vec<RuntimeMatrixProfile>,
    /// The compression: every profile index appears in exactly one group.
    pub groups: Vec<SelectionGroup>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<RuntimeMatrixDiagnostic>,
    /// What the shared scan cost, and what each profile's selection cost on top of it.
    pub scan: ScanAccounting,
    pub usage: UsageSnapshot,
    pub verification: crate::classfile::VerificationStatus,
}

impl RuntimeMatrix {
    /// Every physical entry the one shared scan enumerated, in provider order.
    ///
    /// This is the "physical 全量" a profile selects *from*: it is the same list for every view,
    /// and it stays complete whatever a profile decided.
    pub fn physical_entries(&self) -> Vec<&PhysicalEntryId> {
        evidence_containers(&self.physical)
            .into_iter()
            .flat_map(|(_, entries)| entries.iter().map(|entry| &entry.id))
            .collect()
    }

    /// The one group `index` belongs to.
    pub fn group_of(&self, index: usize) -> &SelectionGroup {
        self.groups
            .iter()
            .find(|group| group.members.contains(&index))
            .expect("every profile belongs to exactly one group")
    }
}

/// The declared loaders around the requester, in search order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainGraph {
    pub requester: LoaderId,
    /// The chain from the requester outwards; `chain[0]` is the requester.
    pub chain: Vec<DomainFacts>,
    /// The verdict on ordering through this chain, and the reason when there is none.
    pub ordering: OrderVerdict,
    /// The requester's roots in **search order**, with the position each root has there.
    pub search: Vec<SearchPosition>,
}

/// One declared root at the position the loader graph gives it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchPosition {
    pub loader: LoaderId,
    pub root: LoadRoot,
    /// Position in the search order the delegation policies produce (0 = searched first).
    pub position: u64,
}

/// One domain of the chain, as declared.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainFacts {
    pub loader: LoaderId,
    pub parent_loader: Option<LoaderId>,
    pub delegation: PolicyVerdict,
    pub module_mode: ModuleMode,
    pub module: PolicyVerdict,
    pub roots: Vec<LoadRoot>,
    pub external_override: RuntimeUncertainty,
    pub runtime_transformation: RuntimeUncertainty,
}

/// Whether a declared policy value is one this engine can act on.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PolicyVerdict {
    Supported,
    /// The value is declared but not acted on: the matrix reports it instead of guessing what it
    /// would mean, mirroring `jarde_jvm::providers`' refusal to build a provider for it.
    Unsupported {
        detail: String,
    },
}

impl PolicyVerdict {
    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Supported)
    }
}

/// Whether the declared loader graph determines a search order at all.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OrderVerdict {
    Determined,
    /// No order is derivable, so the report claims none: see [`OrderUnknownReason`].
    Undetermined {
        reason: OrderUnknownReason,
        detail: String,
    },
}

impl OrderVerdict {
    pub fn is_determined(&self) -> bool {
        matches!(self, Self::Determined)
    }
}

/// Why the declared loader graph does not determine a search order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderUnknownReason {
    /// A delegation policy is `custom`/`unknown`: how the parent is consulted is not declared.
    DelegationPolicyUnsupported,
    /// A module mode is not `class_path`; module readability is not a class-path search order.
    ModuleModeUnsupported,
    /// A parent loader is named but was not supplied, so its roots are unknown.
    ParentDomainNotSupplied,
    /// The domain says external overriding may happen; the runtime origin may be outside this scan.
    ExternalOverrideUnknown,
    /// The domain says runtime transformation may happen; the running bytes may not be these.
    RuntimeTransformationUnknown,
}

/// One profile's view of the shared physical evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMatrixProfile {
    /// Index in [`RuntimeMatrixRequest::profiles`].
    pub index: usize,
    pub view: RuntimeView,
    pub policies: RuntimeProfilePolicies,
    /// Which containers and prefixes this profile's layout puts on the runtime path.
    pub layout: LayoutSelection,
    /// Whether the declared loader graph determines this profile's search order, and why not when
    /// it does not.
    ///
    /// It is [`RuntimeMatrix::domain_graph`]'s verdict as it applies here: every profile in one
    /// matrix is asked from the same requesting loader, so the two can never disagree.
    pub loader: OrderVerdict,
    /// One definition per runtime name, aggregated across the containers that publish it.
    pub definitions: Vec<RuntimeMatrixDefinition>,
    /// The reused per-container selection evidence: manifest, every entry with its decision, the
    /// selections, and that container's own coverage and execution.
    pub containers: Vec<MultiReleaseContainerReport>,
    pub coverage: Coverage,
    pub execution: ExecutionReport,
    pub diagnostics: Vec<MultiReleaseReportDiagnostic>,
    /// What this profile's selection cost **after** the shared scan.
    pub usage: PhaseUsage,
}

impl RuntimeMatrixProfile {
    /// The definition for one logical path, if this view has one.
    pub fn definition(&self, binary_name: &[u8]) -> Option<&RuntimeMatrixDefinition> {
        self.definitions
            .iter()
            .find(|definition| definition.binary_name.0 == binary_name)
    }
}

/// The two profile-level policy verdicts this view depends on.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfilePolicies {
    pub multi_release: PolicyVerdict,
    pub layout: PolicyVerdict,
}

/// One selectable logical path in one view, aggregated over the containers that publish it.
///
/// The aggregation is deliberate: the same binary name in two containers is exactly the case A07
/// asks about, and keeping one definition per path with several origins keeps that visible
/// instead of collapsing it to whichever container happened to be read last.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeMatrixDefinition {
    /// The name this definition is resolved under at runtime.
    ///
    /// For a generic layout that is the multi-release logical path itself. For a layout that
    /// publishes a class-path layer it is that path with the layer's prefix removed, which is what
    /// makes the same class shipped in a class directory and inside a library one definition with
    /// two origins instead of two unrelated paths. Only the layer the tree walker published is
    /// stripped: no other part of a name is reinterpreted here.
    pub binary_name: ArchiveNameBytes,
    /// One origin per container that publishes this name, in physical order.
    pub origins: Vec<RuntimeDefinitionOrigin>,
    /// How the declared loader graph orders those origins, or why it does not.
    pub loader: LoaderOrder,
}

impl RuntimeMatrixDefinition {
    /// Every physical entry that claims this path, selected or not, in origin then ordinal order.
    pub fn candidates(&self) -> Vec<&RuntimeCandidate> {
        self.origins
            .iter()
            .flat_map(|origin| origin.candidates.iter())
            .collect()
    }

    /// The physical entries this view did **not** select for the path.
    ///
    /// Deriving it from [`RuntimeCandidate::decision`] keeps one source of truth: the list cannot
    /// disagree with the decisions it is drawn from.
    pub fn unselected(&self) -> Vec<&RuntimeCandidate> {
        self.candidates()
            .into_iter()
            .filter(|candidate| {
                !matches!(candidate.decision, MultiReleaseSelectionDecision::Selected)
            })
            .collect()
    }

    /// The entry this view's selection chose for the path, when it chose one.
    pub fn selected(&self) -> Option<&RuntimeCandidate> {
        self.candidates()
            .into_iter()
            .find(|candidate| matches!(candidate.decision, MultiReleaseSelectionDecision::Selected))
    }
}

/// One container's contribution to a definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDefinitionOrigin {
    pub container: ContainerOrigin,
    /// The physical logical path this container publishes, before the layout projection.
    pub logical_path: ArchiveNameBytes,
    /// The rule this container's selection applied to the path.
    pub rule: RuntimeSelectionRule,
    /// **Every** physical entry of this container that claims the path, with the decision the
    /// reused selection reached for it.
    pub candidates: Vec<RuntimeCandidate>,
}

/// One physical entry that claims a logical path, and what the selection decided about it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeCandidate {
    pub entry: PhysicalEntryId,
    pub variant: MultiReleaseEntryVariant,
    /// Why this entry was not the one selected (`Selected` marks the winner).
    pub decision: MultiReleaseSelectionDecision,
    /// The compliance evidence the reused selection produced for this entry.
    pub compliance: MultiReleaseCompliance,
    /// Which declared root covers this entry's container, and at which search position.
    pub root: RootAttribution,
    /// Whether this profile's layout puts the entry on the runtime path.
    ///
    /// `None` means the profile's layout policy is unsupported, so the matrix has no basis for
    /// either answer; it is not the same as "not on the path".
    pub on_path: Option<bool>,
}

/// Which declared root covers one physical entry's container.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RootAttribution {
    /// The narrowest declared root that covers the container, at its search position.
    ///
    /// "Narrowest" is the lowest position, which is also the first one searched: the declared root
    /// list is what decides, and a container covered by several roots is reached at the earliest.
    Covered { loader: LoaderId, position: u64 },
    /// No supplied domain's roots cover this container: the entry is physical evidence only.
    Uncovered,
    /// The declared graph does not determine order, so no position is claimed.
    Undetermined { detail: String },
}

/// How the declared loader graph orders the origins of one definition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LoaderOrder {
    /// One origin, or several with one of them earliest: the declared order picks it first.
    Ordered { loader: LoaderId, position: u64 },
    /// Several origins share the winning position: which one the loader finds first is not
    /// determined by the declaration, so nothing is claimed.
    Ambiguous {
        position: u64,
        containers: Vec<ContainerOrigin>,
    },
    /// Every origin is outside the supplied roots.
    Uncovered,
    /// The declared graph does not determine order (see [`OrderUnknownReason`]).
    Unknown {
        reason: OrderUnknownReason,
        detail: String,
    },
}

/// Which rule selected what, stated for one container's answer about one path.
///
/// Every variant is a **projection** of the reused selection: the chosen entry is the one the
/// outcome names, and the rule only says which of the selection's own rules that outcome came
/// from. It never re-decides, so it cannot disagree with [`RuntimeCandidate::decision`].
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeSelectionRule {
    /// Multi-release selection was active and the highest versioned entry at or below the target
    /// release won.
    HighestReleaseAtOrBelowTarget { release: u64, target: u16 },
    /// The base entry won, for one of the reasons the selection states below.
    BaseSelected { reason: BaseSelectionReason },
    /// Several physical entries qualify equally: the selection refuses to pick one.
    Ambiguous { entries: Vec<PhysicalEntryId> },
    /// No entry could serve the path.
    NoSelection {
        reason: MultiReleaseNoSelectionReason,
    },
    /// The policy or the evidence does not decide; the matrix states the existing reason and does
    /// not present it as a choice.
    Unknown { reason: MultiReleaseUnknownReason },
}

/// Why the base entry is the selection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BaseSelectionReason {
    /// The profile disables multi-release selection.
    PolicyDisabled,
    /// The target release is below 9, so no versioned directory can apply.
    TargetBelowNine { target: u16 },
    /// The container's manifest does not activate multi-release selection.
    ///
    /// Only `missing`/`inactive` can appear: an `ambiguous`, `malformed` or unreadable manifest
    /// makes the selection unknown instead (see [`RuntimeSelectionRule::Unknown`]), so no base
    /// entry is ever presented as *the* answer there.
    ManifestNotActive { state: ManifestState },
    /// Versioned entries exist, but only above the target release.
    ReleaseAboveTarget { nearest: u64, target: u16 },
    /// The path has no versioned entry at all.
    NoVersionedCandidate,
}

/// The layout layers one profile's mode puts on the runtime path.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutSelection {
    pub mode: LayoutMode,
    pub policy: PolicyVerdict,
    /// The layout kinds the physical tree published, whatever this profile's mode selects.
    pub present: Vec<LayoutNodeKind>,
    /// The layers this profile's mode selects.
    pub layers: Vec<LayoutLayer>,
    /// The containers that are on this profile's runtime path at all.
    pub containers: Vec<ContainerOrigin>,
}

/// One layout layer, projected from a `LayoutNode` the tree walker published.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutLayer {
    pub kind: LayoutNodeKind,
    /// The container the layer belongs to.
    pub container: ContainerOrigin,
    /// The class-path prefix the layer publishes, for the class layers.
    pub prefix: Option<ArchiveNameBytes>,
    /// The nested container the layer publishes, for the library layers.
    pub child_container: Option<ContainerOrigin>,
    /// The entry the tree walker read the layer from.
    pub evidence_entry: PhysicalEntryId,
}

/// One group of profiles whose selection is the same, and *why* that is a proof.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionGroup {
    /// Profile indexes, ascending; every index appears in exactly one group.
    pub members: Vec<usize>,
    /// The interval of target releases on which every member's own selection is the same as this
    /// group's, or `None` when no such interval was proven.
    pub stable_interval: Option<StableInterval>,
    pub reason: GroupReason,
}

/// A proven interval of target releases: the selection is a step function of the target, and the
/// breakpoints are exactly the versioned releases the same physical scan enumerated.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StableInterval {
    pub lowest_release: u64,
    /// `None` means "no versioned entry above this point", so the interval is open upwards.
    pub highest_release: Option<u64>,
}

impl StableInterval {
    pub fn contains(&self, release: u64) -> bool {
        release >= self.lowest_release
            && self
                .highest_release
                .is_none_or(|highest| release <= highest)
    }

    fn intersect(self, other: Self) -> Option<Self> {
        let lowest_release = self.lowest_release.max(other.lowest_release);
        let highest_release = match (self.highest_release, other.highest_release) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) | (None, Some(a)) => Some(a),
            (None, None) => None,
        };
        let empty = highest_release.is_some_and(|highest| highest < lowest_release);
        (!empty).then_some(Self {
            lowest_release,
            highest_release,
        })
    }
}

/// Why a group has the members it has.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GroupReason {
    /// Several members, and every member's own intervals contain every member's target release:
    /// the agreement is proven, not observed.
    SharedStableInterval,
    /// One member, with the interval its own selection is proven constant on.
    SingleStableInterval,
    /// One member and no proven interval. Profiles in this state may still agree by coincidence;
    /// the matrix refuses to merge them, because the merge would present that agreement as a
    /// version-independent fact.
    NotProvable { detail: String },
}

/// One matrix diagnostic: a fact the matrix itself established, or one profile's own diagnostic.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeMatrixDiagnostic {
    /// Established by the matrix, for one profile or across all of them.
    Matrix {
        /// The profile this condition was found under, when it is profile-specific.
        profile: Option<usize>,
        code: RuntimeMatrixDiagnosticCode,
        message: String,
        provenance: Option<Provenance>,
    },
    /// Forwarded from a profile's reused multi-release report, with the profile it belongs to.
    Profile {
        profile: usize,
        diagnostic: MultiReleaseReportDiagnostic,
    },
}

impl RuntimeMatrixDiagnostic {
    pub fn code(&self) -> Option<RuntimeMatrixDiagnosticCode> {
        match self {
            Self::Matrix { code, .. } => Some(*code),
            Self::Profile { .. } => None,
        }
    }

    pub fn severity(&self) -> DiagnosticSeverity {
        match self {
            Self::Matrix { code, .. } => code.severity(),
            Self::Profile { diagnostic, .. } => match diagnostic {
                MultiReleaseReportDiagnostic::Domain { diagnostic } => diagnostic.severity,
                MultiReleaseReportDiagnostic::Terminal { diagnostic } => diagnostic.severity,
            },
        }
    }

    fn matrix(
        profile: Option<usize>,
        code: RuntimeMatrixDiagnosticCode,
        message: impl Into<String>,
        provenance: Option<Provenance>,
    ) -> Self {
        Self::Matrix {
            profile,
            code,
            message: message.into(),
            provenance,
        }
    }
}

/// A condition the matrix established itself, across the profiles it compared.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMatrixDiagnosticCode {
    /// Versioned entries are shipped for a path that no compared profile can select, because the
    /// container's manifest condition does not activate multi-release selection (A06).
    RuntimeMatrixVersionedEntriesInactive,
    /// The selected entry's own compliance check proved it nonconformant (A06).
    RuntimeMatrixSelectedEntryNonConformant,
    /// The profile declares a layout the physical tree does not exhibit.
    RuntimeMatrixLayoutLayerMissing,
    /// The tree exhibits layout layers the profile's layout mode selects none of.
    RuntimeMatrixLayoutFactsIgnored,
}

impl RuntimeMatrixDiagnosticCode {
    /// Severity of one matrix-established condition.
    ///
    /// A selected entry the compliance probe proved nonconformant is an error: something that
    /// would run is known to violate the `Multi-Release` rules. The other three are packaging
    /// conditions: the archive is well formed, and the pairing of its content with the declared
    /// manifest/layout condition is what a reader should look at.
    pub fn severity(self) -> DiagnosticSeverity {
        match self {
            Self::RuntimeMatrixSelectedEntryNonConformant => DiagnosticSeverity::Error,
            Self::RuntimeMatrixVersionedEntriesInactive
            | Self::RuntimeMatrixLayoutLayerMissing
            | Self::RuntimeMatrixLayoutFactsIgnored => DiagnosticSeverity::Warning,
        }
    }
}

/// What the shared scan cost, and what each profile added to it.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScanAccounting {
    /// The one enumeration every view is computed over.
    pub shared_scan: PhaseUsage,
    /// Per profile, in request order: what its selection cost after that scan.
    pub per_profile: Vec<PhaseUsage>,
    /// How many enumerations this matrix performed. Always `1`: the batch shares the scan.
    pub physical_scans: u64,
}

impl ScanAccounting {
    /// What the same profiles would have cost if each performed its own physical scan.
    ///
    /// The scan is the profile-independent part, so the projection is the shared scan repeated
    /// once per profile; it is arithmetic over the measured scan, not a second measurement, and
    /// the tests compare it against separately measured per-profile runs.
    pub fn rescan_projection(&self) -> PhaseUsage {
        let profiles = self.per_profile.len() as u64;
        self.shared_scan.scaled(profiles)
    }
}

/// Counted usage of one phase: the difference of two [`UsageSnapshot`]s.
///
/// High-water dimensions (`nested_depth`, `dependency_depth`) and the wall clock are deliberately
/// absent: a difference of maxima is not a per-phase cost.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseUsage {
    pub input_bytes: u64,
    pub archive_entries: u64,
    pub entry_bytes: u64,
    pub read_bytes: u64,
    pub class_bytes: u64,
    pub attribute_bytes: u64,
    pub code_bytes: u64,
    pub result_items: u64,
    pub output_bytes: u64,
    pub class_headers: u64,
    pub method_bodies: u64,
}

impl PhaseUsage {
    /// The counted difference between two points on one budget.
    pub fn between(before: &UsageSnapshot, after: &UsageSnapshot) -> Self {
        let delta = |a: u64, b: u64| b.saturating_sub(a);
        Self {
            input_bytes: delta(before.input_bytes, after.input_bytes),
            archive_entries: delta(before.archive_entries, after.archive_entries),
            entry_bytes: delta(before.entry_bytes, after.entry_bytes),
            read_bytes: delta(before.read_bytes, after.read_bytes),
            class_bytes: delta(before.class_bytes, after.class_bytes),
            attribute_bytes: delta(before.attribute_bytes, after.attribute_bytes),
            code_bytes: delta(before.code_bytes, after.code_bytes),
            result_items: delta(before.result_items, after.result_items),
            output_bytes: delta(before.output_bytes, after.output_bytes),
            class_headers: delta(before.class_headers, after.class_headers),
            method_bodies: delta(before.method_bodies, after.method_bodies),
        }
    }

    fn scaled(&self, times: u64) -> Self {
        Self {
            input_bytes: self.input_bytes * times,
            archive_entries: self.archive_entries * times,
            entry_bytes: self.entry_bytes * times,
            read_bytes: self.read_bytes * times,
            class_bytes: self.class_bytes * times,
            attribute_bytes: self.attribute_bytes * times,
            code_bytes: self.code_bytes * times,
            result_items: self.result_items * times,
            output_bytes: self.output_bytes * times,
            class_headers: self.class_headers * times,
            method_bodies: self.method_bodies * times,
        }
    }
}

/// Builds the matrix: one physical scan, one selection per profile over that scan.
pub fn build(
    snapshot: &ArtifactSnapshot,
    request: &RuntimeMatrixRequest,
    budget: &mut Budget,
) -> Result<RuntimeMatrix> {
    let requester = requester_domain(request)?.clone();
    let domain_graph = build_domain_graph(request, &requester)?;

    // The scan needs a view only for its physical half; every profile below shares its result.
    let probe = RuntimeView {
        physical: request.physical.clone(),
        profile: request.profiles[0].clone(),
        load_domain: requester.clone(),
    };
    let scan_before = budget.usage();
    let physical = multi_release::physical_evidence(snapshot, &probe, budget)?;
    let shared_scan = PhaseUsage::between(&scan_before, &budget.usage());

    let mut issues = Issues::new(multi_release::physical_execution(&physical));
    let mut profiles = Vec::with_capacity(request.profiles.len());
    let mut diagnostics = Vec::new();
    let mut keys = Vec::with_capacity(request.profiles.len());
    let mut intervals: Vec<Vec<Option<StableInterval>>> =
        Vec::with_capacity(request.profiles.len());

    for (index, profile) in request.profiles.iter().enumerate() {
        let before = budget.usage();
        let view = RuntimeView {
            physical: request.physical.clone(),
            profile: profile.clone(),
            load_domain: requester.clone(),
        };
        // `physical` is cloned into the report the caller receives, but the *scan* is not
        // repeated: `select_over_physical` reads no archive bytes.
        let report =
            multi_release::select_over_physical(snapshot, &view, physical.clone(), budget)?;
        let usage = PhaseUsage::between(&before, &budget.usage());
        let MultiReleaseViewReport {
            containers,
            coverage,
            execution,
            diagnostics: profile_diagnostics,
            ..
        } = report;
        issues.merge(&execution);
        let policies = RuntimeProfilePolicies {
            multi_release: multi_release_policy(&profile.multi_release),
            layout: layout_policy(&profile.layout),
        };
        let layout = layout_selection(&physical, &profile.layout, &containers);
        let definitions = definitions_for(&containers, profile, &layout, &domain_graph);
        let loader = domain_graph.ordering.clone();
        let manifests: HashMap<&ContainerOrigin, ManifestState> = containers
            .iter()
            .map(|container| (&container.origin, container.manifest.state))
            .collect();
        for definition in &definitions {
            for origin in &definition.origins {
                establish_conditions(&mut diagnostics, index, definition, origin, &manifests);
            }
        }
        diagnostics.extend(profile_diagnostics.iter().cloned().map(|diagnostic| {
            RuntimeMatrixDiagnostic::Profile {
                profile: index,
                diagnostic,
            }
        }));
        keys.push(comparison_key(&definitions));
        intervals.push(
            definitions
                .iter()
                .map(|definition| definition_interval(definition, profile))
                .collect(),
        );
        profiles.push(RuntimeMatrixProfile {
            index,
            view,
            policies,
            layout,
            loader,
            definitions,
            containers,
            coverage,
            execution,
            diagnostics: profile_diagnostics,
            usage,
        });
    }

    let groups = compress(&profiles, &keys, &intervals);
    let execution = issues.finish(budget);
    let coverage = matrix_coverage(&physical, &profiles);
    let per_profile: Vec<PhaseUsage> = profiles.iter().map(|profile| profile.usage).collect();
    Ok(RuntimeMatrix {
        physical,
        domain_graph,
        profiles,
        groups,
        coverage,
        execution,
        diagnostics,
        scan: ScanAccounting {
            shared_scan,
            per_profile,
            physical_scans: 1,
        },
        usage: budget.usage(),
        verification: crate::classfile::VerificationStatus::NotPerformed,
    })
}

/// The requested domain, validated against the profile ceiling and the loader graph.
///
/// Every check here rejects a request that would otherwise make the report read as something it
/// is not: a repeated profile would look like several views while showing one, and a loader graph
/// with a repeated loader id cannot be walked at all.
fn requester_domain(request: &RuntimeMatrixRequest) -> Result<&LoadDomain> {
    if request.profiles.is_empty() {
        return Err(Error::invalid_input(
            "runtime_matrix_empty_profiles",
            "a runtime matrix needs at least one profile",
        ));
    }
    if request.profiles.len() > MAX_PROFILES {
        return Err(Error::invalid_input(
            "runtime_matrix_profile_limit",
            format!("a runtime matrix compares at most {MAX_PROFILES} profiles"),
        ));
    }
    for (index, profile) in request.profiles.iter().enumerate() {
        if request.profiles[..index].contains(profile) {
            return Err(Error::invalid_input(
                "runtime_matrix_duplicate_profile",
                "a runtime matrix compares distinct profiles; one profile was requested twice",
            ));
        }
    }
    for (index, domain) in request.domains.iter().enumerate() {
        if request.domains[..index]
            .iter()
            .any(|earlier| earlier.loader == domain.loader)
        {
            return Err(Error::invalid_input(
                "runtime_matrix_duplicate_loader",
                "each loader id may appear once in a runtime matrix request",
            ));
        }
    }
    request
        .domains
        .iter()
        .find(|domain| domain.loader == request.requester)
        .ok_or_else(|| {
            Error::invalid_input(
                "runtime_matrix_requester_missing",
                "the requesting loader is not among the supplied load domains",
            )
        })
}

/// Walks the declared loader graph outwards from the requester and states the search order it
/// produces.
fn build_domain_graph(
    request: &RuntimeMatrixRequest,
    requester: &LoadDomain,
) -> Result<DomainGraph> {
    let mut chain: Vec<&LoadDomain> = vec![requester];
    let mut visited: Vec<LoaderId> = vec![requester.loader.clone()];
    let mut missing_parent: Option<LoaderId> = None;
    while let Some(parent) = chain[chain.len() - 1].parent_loader.clone() {
        let Some(domain) = request
            .domains
            .iter()
            .find(|domain| domain.loader == parent)
        else {
            // A parent outside the request is not an error: the query simply cannot see its
            // roots, which is a fact the report states instead of guessing around.
            missing_parent = Some(parent);
            break;
        };
        if visited.contains(&domain.loader) {
            return Err(Error::invalid_input(
                "runtime_matrix_loader_cycle",
                "the declared parent loaders form a cycle",
            ));
        }
        visited.push(domain.loader.clone());
        chain.push(domain);
    }

    let mut undetermined: Option<(OrderUnknownReason, String)> = None;
    for domain in &chain {
        match module_verdict(&domain.module_mode) {
            PolicyVerdict::Supported => {}
            PolicyVerdict::Unsupported { detail } => {
                undetermined.get_or_insert((
                    OrderUnknownReason::ModuleModeUnsupported,
                    format!("loader `{}`: {detail}", domain.loader.0),
                ));
            }
        }
        match delegation_verdict(&domain.delegation) {
            PolicyVerdict::Supported => {}
            PolicyVerdict::Unsupported { detail } => {
                undetermined.get_or_insert((
                    OrderUnknownReason::DelegationPolicyUnsupported,
                    format!("loader `{}`: {detail}", domain.loader.0),
                ));
            }
        }
        if domain.external_override == RuntimeUncertainty::Unknown {
            undetermined.get_or_insert((
                OrderUnknownReason::ExternalOverrideUnknown,
                format!(
                    "loader `{}`: external overriding is declared unknown",
                    domain.loader.0
                ),
            ));
        }
        if domain.runtime_transformation == RuntimeUncertainty::Unknown {
            undetermined.get_or_insert((
                OrderUnknownReason::RuntimeTransformationUnknown,
                format!(
                    "loader `{}`: runtime transformation is declared unknown",
                    domain.loader.0
                ),
            ));
        }
    }
    if let Some(parent) = &missing_parent {
        undetermined.get_or_insert((
            OrderUnknownReason::ParentDomainNotSupplied,
            format!("parent loader `{}` was not supplied", parent.0),
        ));
    }

    let ordering = match undetermined {
        Some((reason, detail)) => OrderVerdict::Undetermined { reason, detail },
        None => OrderVerdict::Determined,
    };
    let search = match &ordering {
        OrderVerdict::Undetermined { .. } => Vec::new(),
        OrderVerdict::Determined => {
            let mut search = Vec::new();
            let mut next_position = 0_u64;
            collect_search(&chain, 0, &mut search, &mut next_position);
            search
        }
    };

    let facts = chain
        .iter()
        .map(|domain| DomainFacts {
            loader: domain.loader.clone(),
            parent_loader: domain.parent_loader.clone(),
            delegation: delegation_verdict(&domain.delegation),
            module_mode: domain.module_mode.clone(),
            module: module_verdict(&domain.module_mode),
            roots: domain.roots.clone(),
            external_override: domain.external_override,
            runtime_transformation: domain.runtime_transformation,
        })
        .collect();
    Ok(DomainGraph {
        requester: request.requester.clone(),
        chain: facts,
        ordering,
        search,
    })
}

/// Appends one loader's roots in the order its own delegation policy searches them.
///
/// A `parent_first` loader consults its parent before its own roots, a `child_first` one the other
/// way round: that is what the declaration means, and the resulting sequence is what a class-path
/// search really walks through. The chain may end before the outermost loader (a parent outside
/// the request), in which case the walk simply stops — the verdict already says so.
fn collect_search(
    chain: &[&LoadDomain],
    index: usize,
    out: &mut Vec<SearchPosition>,
    next_position: &mut u64,
) {
    let domain = chain[index];
    let parent = chain.get(index + 1);
    let push_own = |out: &mut Vec<SearchPosition>, next_position: &mut u64| {
        for root in &domain.roots {
            out.push(SearchPosition {
                loader: domain.loader.clone(),
                root: root.clone(),
                position: *next_position,
            });
            *next_position += 1;
        }
    };
    match domain.delegation {
        DelegationPolicy::ChildFirst => {
            push_own(out, next_position);
            if parent.is_some() {
                collect_search(chain, index + 1, out, next_position);
            }
        }
        _ => {
            if parent.is_some() {
                collect_search(chain, index + 1, out, next_position);
            }
            push_own(out, next_position);
        }
    }
}

/// Whether a declared delegation policy is one the search order can be built from.
fn delegation_verdict(policy: &DelegationPolicy) -> PolicyVerdict {
    match policy {
        DelegationPolicy::ParentFirst | DelegationPolicy::ChildFirst => PolicyVerdict::Supported,
        DelegationPolicy::Custom { id } => PolicyVerdict::Unsupported {
            detail: format!("delegation policy `custom` ({id}) declares no search order"),
        },
        DelegationPolicy::Unknown => PolicyVerdict::Unsupported {
            detail: "unknown delegation policy declares no search order".to_string(),
        },
    }
}

/// Whether a declared module mode is one this engine can reason about at all.
///
/// Only `class_path` is: named-module readability is decided from `module-info` descriptors, and
/// this engine measures those attributes to reach the end of the header but never reads their
/// contents, so it cannot claim to resolve or order anything through a module path. This mirrors
/// `jarde_jvm::providers`, which refuses to build a provider for the other modes; the matrix
/// states the same verdict as data instead of claiming a selection for them.
fn module_verdict(mode: &ModuleMode) -> PolicyVerdict {
    match mode {
        ModuleMode::ClassPath => PolicyVerdict::Supported,
        ModuleMode::ModulePath => PolicyVerdict::Unsupported {
            detail: "module mode `module_path` needs named-module readability, which is not read"
                .to_string(),
        },
        ModuleMode::Hybrid => PolicyVerdict::Unsupported {
            detail: "module mode `hybrid` needs named-module readability, which is not read"
                .to_string(),
        },
        ModuleMode::Custom { id } => PolicyVerdict::Unsupported {
            detail: format!("module mode `custom` ({id}) declares no semantics this engine reads"),
        },
        ModuleMode::Unknown => PolicyVerdict::Unsupported {
            detail: "unknown module mode declares no semantics this engine reads".to_string(),
        },
    }
}

/// Whether a multi-release policy is one the selection acts on.
fn multi_release_policy(policy: &MultiReleasePolicy) -> PolicyVerdict {
    match policy {
        MultiReleasePolicy::Enabled | MultiReleasePolicy::Disabled => PolicyVerdict::Supported,
        MultiReleasePolicy::Custom { id } => PolicyVerdict::Unsupported {
            detail: format!("multi release policy `custom` ({id}) is not one this engine applies"),
        },
        MultiReleasePolicy::Unknown => PolicyVerdict::Unsupported {
            detail: "unknown multi release policy is not one this engine applies".to_string(),
        },
    }
}

/// Whether a layout mode is one the selection acts on.
fn layout_policy(mode: &LayoutMode) -> PolicyVerdict {
    match mode {
        LayoutMode::Generic | LayoutMode::War | LayoutMode::SpringBoot => PolicyVerdict::Supported,
        LayoutMode::Custom { id } => PolicyVerdict::Unsupported {
            detail: format!("layout mode `custom` ({id}) declares no layers this engine reads"),
        },
        LayoutMode::Unknown => PolicyVerdict::Unsupported {
            detail: "unknown layout mode declares no layers this engine reads".to_string(),
        },
    }
}

/// Every container the one physical evidence enumerated, with its entries in provider order.
fn evidence_containers(
    physical: &MultiReleasePhysicalEvidence,
) -> Vec<(ContainerOrigin, &[PhysicalEntry])> {
    match physical {
        MultiReleasePhysicalEvidence::Snapshot { report } => vec![(
            multi_release::root_origin_of(&report.snapshot),
            report.entries.as_slice(),
        )],
        MultiReleasePhysicalEvidence::ArtifactTree { report } => report
            .containers
            .iter()
            .map(|container| (container.origin.clone(), container.entries.as_slice()))
            .collect(),
    }
}

/// The layout nodes one physical evidence published, in the tree walker's order.
fn layout_nodes(physical: &MultiReleasePhysicalEvidence) -> &[crate::artifact::LayoutNode] {
    match physical {
        MultiReleasePhysicalEvidence::Snapshot { .. } => &[],
        MultiReleasePhysicalEvidence::ArtifactTree { report } => &report.layout_nodes,
    }
}

/// Projects the tree's layout facts onto one profile's layout mode.
///
/// The *facts* are read by the tree walker (`BOOT-INF/classes/`, `WEB-INF/lib/*.jar`, …); the mode
/// only decides which of them this profile puts on the runtime path. A mode that selects no layer
/// of an evidenced kind is a condition worth stating, not something to paper over.
fn layout_selection(
    physical: &MultiReleasePhysicalEvidence,
    mode: &LayoutMode,
    containers: &[MultiReleaseContainerReport],
) -> LayoutSelection {
    let nodes = layout_nodes(physical);
    let mut present: Vec<LayoutNodeKind> = Vec::new();
    for node in nodes {
        if !present.contains(&node.kind) {
            present.push(node.kind);
        }
    }
    let policy = layout_policy(mode);
    let selected_kinds: Option<[LayoutNodeKind; 2]> = match mode {
        LayoutMode::War => Some([LayoutNodeKind::WarClasses, LayoutNodeKind::WarLibrary]),
        LayoutMode::SpringBoot => Some([LayoutNodeKind::BootClasses, LayoutNodeKind::BootLibrary]),
        LayoutMode::Generic => None,
        LayoutMode::Custom { .. } | LayoutMode::Unknown => {
            return LayoutSelection {
                mode: mode.clone(),
                policy,
                present,
                layers: Vec::new(),
                containers: Vec::new(),
            };
        }
    };
    let layers: Vec<LayoutLayer> = match selected_kinds {
        None => Vec::new(),
        Some(kinds) => nodes
            .iter()
            .filter(|node| kinds.contains(&node.kind))
            .map(layout_layer)
            .collect(),
    };
    let mut on_path: Vec<ContainerOrigin> = Vec::new();
    match selected_kinds {
        None => {
            // The generic mode is the tree as the walker read it: every enumerated container is on
            // the path, and no prefix is reinterpreted.
            on_path.extend(containers.iter().map(|container| container.origin.clone()));
        }
        Some(_) => {
            for layer in &layers {
                for origin in [Some(&layer.container), layer.child_container.as_ref()]
                    .into_iter()
                    .flatten()
                {
                    if !on_path.contains(origin) {
                        on_path.push(origin.clone());
                    }
                }
            }
        }
    }
    LayoutSelection {
        mode: mode.clone(),
        policy,
        present,
        layers,
        containers: on_path,
    }
}

fn layout_layer(node: &crate::artifact::LayoutNode) -> LayoutLayer {
    match &node.source {
        LayoutNodeSource::Prefix {
            container,
            prefix,
            evidence_entry,
        } => LayoutLayer {
            kind: node.kind,
            container: container.clone(),
            prefix: Some(prefix.clone()),
            child_container: None,
            evidence_entry: evidence_entry.clone(),
        },
        LayoutNodeSource::Archive {
            entry,
            child_container,
        } => LayoutLayer {
            kind: node.kind,
            container: entry.origin.clone(),
            prefix: None,
            child_container: child_container.clone(),
            evidence_entry: entry.clone(),
        },
    }
}

/// Whether one profile's layout puts a physical entry on the runtime path.
fn entry_on_path(entry: &PhysicalEntryId, layout: &LayoutSelection) -> Option<bool> {
    if !layout.policy.is_supported() {
        return None;
    }
    if layout.mode == LayoutMode::Generic {
        return Some(true);
    }
    if layout.layers.is_empty() {
        // The mode selects layers the physical evidence does not publish. Whether such an entry
        // would be on the path is then not a question this evidence can answer, and the matrix
        // says so instead of answering `false`.
        return None;
    }
    if !layout.containers.contains(&entry.origin) {
        return Some(false);
    }
    match layout
        .layers
        .iter()
        .find(|layer| layer.kind.is_class_layer() && layer.container == entry.origin)
    {
        Some(layer) => {
            let prefix = layer.prefix.as_ref().map(|prefix| prefix.0.as_slice());
            Some(prefix.is_some_and(|prefix| entry.raw_name.0.starts_with(prefix)))
        }
        // A library layer's container: everything the library ships is on the path.
        None => Some(true),
    }
}

/// One profile's definitions: every selectable logical path, with every entry that claims it.
fn definitions_for(
    containers: &[MultiReleaseContainerReport],
    profile: &RuntimeProfile,
    layout: &LayoutSelection,
    graph: &DomainGraph,
) -> Vec<RuntimeMatrixDefinition> {
    let mut definitions: Vec<RuntimeMatrixDefinition> = Vec::new();
    for container in containers {
        for selection in &container.selections {
            let name = projected_name(layout, &container.origin, &selection.logical_path)
                .unwrap_or_else(|| selection.logical_path.clone());
            let origin = definition_origin(selection, container, profile, layout, graph);
            match definitions
                .iter_mut()
                .find(|definition| definition.binary_name == name)
            {
                Some(definition) => definition.origins.push(origin),
                None => definitions.push(RuntimeMatrixDefinition {
                    binary_name: name,
                    origins: vec![origin],
                    loader: LoaderOrder::Uncovered,
                }),
            }
        }
    }
    definitions.sort_by(|left, right| left.binary_name.0.cmp(&right.binary_name.0));
    for definition in &mut definitions {
        definition.loader = loader_order(definition, graph);
    }
    definitions
}

fn definition_origin(
    selection: &MultiReleaseSelection,
    container: &MultiReleaseContainerReport,
    profile: &RuntimeProfile,
    layout: &LayoutSelection,
    graph: &DomainGraph,
) -> RuntimeDefinitionOrigin {
    let mut candidates: Vec<RuntimeCandidate> = container
        .entries
        .iter()
        .filter(|entry| {
            entry
                .logical_path
                .as_ref()
                .is_some_and(|path| *path == selection.logical_path)
        })
        .map(|entry| RuntimeCandidate {
            entry: entry.entry.clone(),
            variant: entry.variant.clone(),
            decision: entry.decision.clone(),
            compliance: entry.compliance.clone(),
            root: root_attribution(&entry.entry.origin, graph),
            on_path: entry_on_path(&entry.entry, layout),
        })
        .collect();
    candidates.sort_by_key(|candidate| candidate.entry.ordinal);
    let rule = selection_rule(selection, &candidates, profile, container.manifest.state);
    RuntimeDefinitionOrigin {
        container: container.origin.clone(),
        logical_path: selection.logical_path.clone(),
        rule,
        candidates,
    }
}

/// The runtime name one container's physical logical path projects onto under this layout.
///
/// `None` means the profile's layout does not determine a name for that path — an unsupported mode,
/// a class layer the physical scope does not publish, or an entry outside the layer's prefix.
fn projected_name(
    layout: &LayoutSelection,
    container: &ContainerOrigin,
    logical_path: &ArchiveNameBytes,
) -> Option<ArchiveNameBytes> {
    if !layout.policy.is_supported() {
        return None;
    }
    if layout.mode == LayoutMode::Generic {
        return Some(logical_path.clone());
    }
    if !layout.containers.contains(container) {
        return None;
    }
    match layout
        .layers
        .iter()
        .find(|layer| layer.kind.is_class_layer() && layer.container == *container)
    {
        Some(layer) => {
            let prefix = layer.prefix.as_ref()?;
            logical_path
                .0
                .strip_prefix(prefix.0.as_slice())
                .map(|rest| ArchiveNameBytes(rest.to_vec()))
        }
        None => Some(logical_path.clone()),
    }
}

/// How many versioned entries of this origin a multi-release search really considers.
///
/// `NotApplicable` entries are exactly the ones the reused selection excluded from every group —
/// directories, invalid version paths and versioned `META-INF` resources — so they are never
/// levels a target release could select.
fn selectable(candidate: &RuntimeCandidate) -> bool {
    !matches!(
        candidate.decision,
        MultiReleaseSelectionDecision::NotApplicable { .. }
    )
}

fn versioned_releases(candidates: &[RuntimeCandidate]) -> Vec<u64> {
    let mut releases: Vec<u64> = candidates
        .iter()
        .filter(|candidate| selectable(candidate))
        .filter_map(|candidate| match candidate.variant {
            MultiReleaseEntryVariant::Versioned { release } => Some(release),
            _ => None,
        })
        .collect();
    releases.sort_unstable();
    releases.dedup();
    releases
}

/// The rule one container's selection applied to one path.
///
/// This is a total projection of the reused outcome. The chosen entry is the one the outcome names;
/// the variant of that entry and the facts the selection itself exposes (the manifest state, the
/// profile's target release, the versioned releases present) decide only *which* of the selection's
/// rules the outcome came from. The matrix therefore cannot report a rule its own evidence
/// contradicts — a test pins each label of the divergent fixture against the entry the outcome
/// names.
fn selection_rule(
    selection: &MultiReleaseSelection,
    candidates: &[RuntimeCandidate],
    profile: &RuntimeProfile,
    manifest: ManifestState,
) -> RuntimeSelectionRule {
    match &selection.outcome {
        crate::multi_release::MultiReleaseSelectionOutcome::Unknown { reason } => {
            RuntimeSelectionRule::Unknown { reason: *reason }
        }
        crate::multi_release::MultiReleaseSelectionOutcome::Ambiguous { entries } => {
            RuntimeSelectionRule::Ambiguous {
                entries: entries.clone(),
            }
        }
        crate::multi_release::MultiReleaseSelectionOutcome::NoSelection { reason } => {
            RuntimeSelectionRule::NoSelection { reason: *reason }
        }
        crate::multi_release::MultiReleaseSelectionOutcome::Selected { entry } => {
            let chosen = candidates
                .iter()
                .find(|candidate| &candidate.entry == entry);
            match chosen.map(|candidate| &candidate.variant) {
                Some(MultiReleaseEntryVariant::Versioned { release }) => {
                    RuntimeSelectionRule::HighestReleaseAtOrBelowTarget {
                        release: *release,
                        target: profile.java_release,
                    }
                }
                Some(_) => RuntimeSelectionRule::BaseSelected {
                    reason: base_reason(profile, manifest, candidates),
                },
                // The outcome names an entry the container does not list. Consistent evidence
                // cannot produce this, and saying so is the only honest answer available: the
                // alternative is inventing a rule for an entry that is not there.
                None => RuntimeSelectionRule::Unknown {
                    reason: MultiReleaseUnknownReason::PhysicalEvidenceIncomplete,
                },
            }
        }
    }
}

/// Why the base entry is the selection, in the same precedence the reused selection uses for its
/// own `inactive` reasons.
fn base_reason(
    profile: &RuntimeProfile,
    manifest: ManifestState,
    candidates: &[RuntimeCandidate],
) -> BaseSelectionReason {
    let releases = versioned_releases(candidates);
    if matches!(profile.multi_release, MultiReleasePolicy::Disabled) {
        BaseSelectionReason::PolicyDisabled
    } else if manifest == ManifestState::Missing {
        BaseSelectionReason::ManifestNotActive {
            state: ManifestState::Missing,
        }
    } else if manifest == ManifestState::Inactive {
        BaseSelectionReason::ManifestNotActive {
            state: ManifestState::Inactive,
        }
    } else if profile.java_release < 9 {
        BaseSelectionReason::TargetBelowNine {
            target: profile.java_release,
        }
    } else if let Some(nearest) = releases.first() {
        BaseSelectionReason::ReleaseAboveTarget {
            nearest: *nearest,
            target: profile.java_release,
        }
    } else {
        BaseSelectionReason::NoVersionedCandidate
    }
}

/// Whether the declared loader graph covers a container, and where.
fn root_attribution(container: &ContainerOrigin, graph: &DomainGraph) -> RootAttribution {
    match &graph.ordering {
        OrderVerdict::Undetermined { detail, .. } => RootAttribution::Undetermined {
            detail: detail.clone(),
        },
        OrderVerdict::Determined => graph
            .search
            .iter()
            .find(|position| root_covers(&position.root, container))
            .map_or(RootAttribution::Uncovered, |position| {
                RootAttribution::Covered {
                    loader: position.loader.clone(),
                    position: position.position,
                }
            }),
    }
}

/// Whether one declared root covers an enumerated container, and where.
///
/// A standalone CLASS root names a whole snapshot and no container of its own; a container root
/// covers the container it addresses and everything derived from it — the prefix decides which
/// entries of that container are on the class path, never which container it is. The prefix
/// therefore plays no part here: it is a lookup key inside the container, not a second container.
fn root_covers(root: &LoadRoot, container: &ContainerOrigin) -> bool {
    match root {
        LoadRoot::StandaloneClass { snapshot } => container.snapshot == *snapshot,
        LoadRoot::Container { origin, .. } => container_is_under(container, origin),
        // An external root names bytes this scan never read: it covers no enumerated container.
        LoadRoot::External { .. } => false,
    }
}

/// Whether one container's origin chain starts with another's.
fn container_is_under(container: &ContainerOrigin, root: &ContainerOrigin) -> bool {
    container.snapshot == root.snapshot
        && container.root_container == root.root_container
        && container.steps.len() >= root.steps.len()
        && container.steps[..root.steps.len()] == root.steps[..]
}

/// How the declared order reaches the origins of one definition.
fn loader_order(definition: &RuntimeMatrixDefinition, graph: &DomainGraph) -> LoaderOrder {
    if let OrderVerdict::Undetermined { reason, detail } = &graph.ordering {
        return LoaderOrder::Unknown {
            reason: *reason,
            detail: detail.clone(),
        };
    }
    let reached: Vec<(&ContainerOrigin, &LoaderId, u64)> = definition
        .origins
        .iter()
        .filter_map(|origin| {
            let candidate =
                origin
                    .candidates
                    .iter()
                    .find_map(|candidate| match &candidate.root {
                        RootAttribution::Covered { loader, position } => Some((loader, *position)),
                        _ => None,
                    })?;
            Some((&origin.container, candidate.0, candidate.1))
        })
        .collect();
    let Some(first) = reached.iter().map(|reached| reached.2).min() else {
        return LoaderOrder::Uncovered;
    };
    let tied: Vec<&ContainerOrigin> = reached
        .iter()
        .filter(|reached| reached.2 == first)
        .map(|reached| reached.0)
        .collect();
    if tied.len() == 1 {
        let loader = reached
            .iter()
            .find(|reached| reached.2 == first)
            .map(|reached| reached.1.clone())
            .expect("the earliest origin has a loader");
        LoaderOrder::Ordered {
            loader,
            position: first,
        }
    } else {
        LoaderOrder::Ambiguous {
            position: first,
            containers: tied.into_iter().cloned().collect(),
        }
    }
}

/// The conditions the matrix states across its own evidence, per profile and path.
fn establish_conditions(
    diagnostics: &mut Vec<RuntimeMatrixDiagnostic>,
    profile: usize,
    definition: &RuntimeMatrixDefinition,
    origin: &RuntimeDefinitionOrigin,
    manifests: &HashMap<&ContainerOrigin, ManifestState>,
) {
    let releases = versioned_releases(&origin.candidates);
    let inactive = match &origin.rule {
        RuntimeSelectionRule::BaseSelected { reason } => match reason {
            BaseSelectionReason::PolicyDisabled => Some("the profile disables multi release"),
            BaseSelectionReason::ManifestNotActive { state } => match state {
                ManifestState::Missing => Some("the container has no manifest"),
                ManifestState::Inactive => Some("the manifest does not activate multi release"),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    };
    if let (Some(condition), false) = (inactive, releases.is_empty()) {
        diagnostics.push(RuntimeMatrixDiagnostic::matrix(
            Some(profile),
            RuntimeMatrixDiagnosticCode::RuntimeMatrixVersionedEntriesInactive,
            format!(
                "`{}` ships versioned entries at releases {:?} that no selected entry can reach: {condition}",
                path_text(&definition.binary_name.0),
                releases
            ),
            None,
        ));
    }
    let nonconformant = origin.candidates.iter().any(|candidate| {
        matches!(candidate.decision, MultiReleaseSelectionDecision::Selected)
            && candidate.compliance == MultiReleaseCompliance::NonConformant
    });
    if nonconformant {
        let state = manifests
            .get(&origin.container)
            .map_or("unknown", |state| manifest_text(*state));
        diagnostics.push(RuntimeMatrixDiagnostic::matrix(
            Some(profile),
            RuntimeMatrixDiagnosticCode::RuntimeMatrixSelectedEntryNonConformant,
            format!(
                "the entry selected for `{}` is nonconformant under the `Multi-Release` rules (manifest condition: {state})",
                path_text(&definition.binary_name.0)
            ),
            None,
        ));
    }
}

fn manifest_text(state: ManifestState) -> &'static str {
    match state {
        ManifestState::Missing => "missing",
        ManifestState::Active => "active",
        ManifestState::Inactive => "inactive",
        ManifestState::Ambiguous => "ambiguous",
        ManifestState::Malformed => "malformed",
        ManifestState::Unknown => "unknown",
    }
}

/// What one origin's selection *chose*, as a value two profiles can be compared by.
///
/// The comparison is deliberately about the answer, not about the rule text that explains it: two
/// profiles that reach the same entry by different reasons do select the same thing, and every
/// interval below is proven from each profile's own facts. Including the rule's own wording would
/// also make the answer depend on the target release, which no stable interval could ever cover.
#[derive(Clone, Debug, Eq, PartialEq)]
enum SignatureChoice {
    /// Exactly one entry was chosen.
    Selected(PhysicalEntryId),
    /// Several entries qualify equally, or none does: no single answer.
    Undecided(Vec<PhysicalEntryId>),
    NoSelection(MultiReleaseNoSelectionReason),
    Unknown(MultiReleaseUnknownReason),
}

/// What two profiles must agree on for one origin to be called the same answer.
#[derive(Clone, Debug, Eq, PartialEq)]
struct OriginSignature {
    container: ContainerOrigin,
    choice: SignatureChoice,
}

/// One definition's choices, per origin, as a value two profiles can be compared by.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ComparisonKey(Vec<DefinitionSignature>);

/// One definition's name and its per-origin answers.
#[derive(Clone, Debug, Eq, PartialEq)]
struct DefinitionSignature {
    binary_name: Vec<u8>,
    origins: Vec<OriginSignature>,
}

fn comparison_key(definitions: &[RuntimeMatrixDefinition]) -> ComparisonKey {
    ComparisonKey(
        definitions
            .iter()
            .map(|definition| DefinitionSignature {
                binary_name: definition.binary_name.0.clone(),
                origins: definition
                    .origins
                    .iter()
                    .map(|origin| OriginSignature {
                        container: origin.container.clone(),
                        choice: signature_choice(origin),
                    })
                    .collect(),
            })
            .collect(),
    )
}

fn signature_choice(origin: &RuntimeDefinitionOrigin) -> SignatureChoice {
    let chosen: Vec<PhysicalEntryId> = origin
        .candidates
        .iter()
        .filter(|candidate| matches!(candidate.decision, MultiReleaseSelectionDecision::Selected))
        .map(|candidate| candidate.entry.clone())
        .collect();
    if let [entry] = chosen.as_slice() {
        return SignatureChoice::Selected(entry.clone());
    }
    match &origin.rule {
        RuntimeSelectionRule::NoSelection { reason } => SignatureChoice::NoSelection(*reason),
        RuntimeSelectionRule::Unknown { reason } => SignatureChoice::Unknown(*reason),
        RuntimeSelectionRule::Ambiguous { entries } => SignatureChoice::Undecided(entries.clone()),
        // The single-chosen case returned above. Zero or several entries marked selected here is
        // evidence that disagrees with itself: not a choice, and not something to panic over
        // either, so it stays a value the comparison can carry.
        _ => SignatureChoice::Undecided(chosen),
    }
}

/// The interval of target releases on which this profile's selection of one definition is proven
/// to stay the same.
fn definition_interval(
    definition: &RuntimeMatrixDefinition,
    profile: &RuntimeProfile,
) -> Option<StableInterval> {
    let mut interval: Option<StableInterval> = None;
    for origin in &definition.origins {
        let candidate = rule_interval(&origin.rule, &origin.candidates, profile)?;
        interval = Some(match interval {
            None => candidate,
            Some(current) => current.intersect(candidate)?,
        });
    }
    interval
}

/// The interval one rule is proven constant on, from the same facts the rule was projected from.
///
/// The breakpoints of a multi-release selection as a function of the target release are exactly
/// the versioned releases the scan enumerated: between two of them the levels a target can reach
/// do not change. A rule with no such argument — an unknown or ambiguous outcome — has no interval,
/// and the compression below refuses to merge on it.
fn rule_interval(
    rule: &RuntimeSelectionRule,
    candidates: &[RuntimeCandidate],
    profile: &RuntimeProfile,
) -> Option<StableInterval> {
    let releases = versioned_releases(candidates);
    match rule {
        RuntimeSelectionRule::HighestReleaseAtOrBelowTarget { release, .. } => {
            let next = releases
                .iter()
                .copied()
                .find(|candidate| candidate > release);
            Some(StableInterval {
                lowest_release: *release,
                highest_release: next.map(|next| next - 1),
            })
        }
        RuntimeSelectionRule::BaseSelected { reason } => match reason {
            // The condition is a property of the policy or of the manifest, neither of which
            // depends on the target release.
            BaseSelectionReason::PolicyDisabled
            | BaseSelectionReason::ManifestNotActive { .. }
            | BaseSelectionReason::NoVersionedCandidate => Some(StableInterval {
                lowest_release: 0,
                highest_release: None,
            }),
            // Below nine no versioned directory applies at all.
            BaseSelectionReason::TargetBelowNine { .. } => Some(StableInterval {
                lowest_release: 0,
                highest_release: Some(8),
            }),
            // The base entry is reachable only while the target stays below the nearest release.
            BaseSelectionReason::ReleaseAboveTarget { nearest, .. } => {
                let highest = nearest.checked_sub(1)?;
                (highest >= 9).then_some(StableInterval {
                    lowest_release: 9,
                    highest_release: Some(highest),
                })
            }
        },
        RuntimeSelectionRule::Ambiguous { .. }
        | RuntimeSelectionRule::NoSelection { .. }
        | RuntimeSelectionRule::Unknown { .. } => None,
    }
    .filter(|interval| interval.contains(u64::from(profile.java_release)))
}

/// Merges profiles whose selection is the same **and** proven to stay the same.
fn compress(
    profiles: &[RuntimeMatrixProfile],
    keys: &[ComparisonKey],
    intervals: &[Vec<Option<StableInterval>>],
) -> Vec<SelectionGroup> {
    struct Draft {
        members: Vec<usize>,
        per_definition: Vec<Option<StableInterval>>,
        overall: Option<StableInterval>,
    }
    let mut drafts: Vec<Draft> = Vec::new();
    for index in 0..profiles.len() {
        let mut placed = false;
        for draft in &mut drafts {
            if keys[draft.members[0]] != keys[index] {
                continue;
            }
            let merged = merge_intervals(&draft.per_definition, &intervals[index]);
            let Some((merged, overall)) = merged else {
                continue;
            };
            let mut releases: Vec<u64> = draft
                .members
                .iter()
                .map(|member| u64::from(profiles[*member].view.profile.java_release))
                .collect();
            releases.push(u64::from(profiles[index].view.profile.java_release));
            if !releases
                .iter()
                .all(|release| overall.is_some_and(|overall| overall.contains(*release)))
            {
                continue;
            }
            draft.members.push(index);
            draft.per_definition = merged;
            draft.overall = overall;
            placed = true;
            break;
        }
        if !placed {
            let overall = overall_interval(&intervals[index]);
            drafts.push(Draft {
                members: vec![index],
                per_definition: intervals[index].clone(),
                overall,
            });
        }
    }
    drafts
        .into_iter()
        .map(|draft| {
            let reason = if draft.members.len() > 1 {
                GroupReason::SharedStableInterval
            } else {
                match draft.overall {
                    Some(_) => GroupReason::SingleStableInterval,
                    None => GroupReason::NotProvable {
                        detail: not_provable_detail(
                            &profiles[draft.members[0]],
                            &draft.per_definition,
                        ),
                    },
                }
            };
            SelectionGroup {
                members: draft.members,
                stable_interval: draft.overall,
                reason,
            }
        })
        .collect()
}

/// The intersection of two per-definition intervals, when every definition has one.
fn merge_intervals(
    left: &[Option<StableInterval>],
    right: &[Option<StableInterval>],
) -> Option<(Vec<Option<StableInterval>>, Option<StableInterval>)> {
    if left.len() != right.len() {
        return None;
    }
    let mut merged = Vec::with_capacity(left.len());
    for (left, right) in left.iter().zip(right) {
        let interval = left.as_ref()?.intersect(*right.as_ref()?)?;
        merged.push(Some(interval));
    }
    let overall = overall_interval(&merged);
    Some((merged, overall))
}

fn overall_interval(per_definition: &[Option<StableInterval>]) -> Option<StableInterval> {
    let mut overall: Option<StableInterval> = None;
    for interval in per_definition {
        let interval = interval.as_ref()?;
        overall = Some(match overall {
            None => *interval,
            Some(current) => current.intersect(*interval)?,
        });
    }
    overall
}

/// Why one profile's selection has no proven interval, named by the definition that lacks one.
fn not_provable_detail(
    profile: &RuntimeMatrixProfile,
    per_definition: &[Option<StableInterval>],
) -> String {
    for (definition, interval) in profile.definitions.iter().zip(per_definition) {
        if interval.is_some() {
            continue;
        }
        let rule = definition
            .origins
            .first()
            .map_or("no origin".to_string(), |origin| rule_text(&origin.rule));
        return format!(
            "`{}` has no proven interval: {rule}",
            path_text(&definition.binary_name.0)
        );
    }
    "no proven interval".to_string()
}

fn rule_text(rule: &RuntimeSelectionRule) -> String {
    match rule {
        RuntimeSelectionRule::HighestReleaseAtOrBelowTarget { release, target } => {
            format!("release {release} is selected at target {target}")
        }
        RuntimeSelectionRule::BaseSelected { reason } => match reason {
            BaseSelectionReason::PolicyDisabled => "the profile disables multi release".to_string(),
            BaseSelectionReason::TargetBelowNine { target } => {
                format!("target {target} is below nine")
            }
            BaseSelectionReason::ManifestNotActive { state } => {
                format!("the manifest condition is {}", manifest_text(*state))
            }
            BaseSelectionReason::ReleaseAboveTarget { nearest, target } => {
                format!("the nearest release {nearest} is above target {target}")
            }
            BaseSelectionReason::NoVersionedCandidate => {
                "the path has no versioned entry".to_string()
            }
        },
        RuntimeSelectionRule::Ambiguous { .. } => "the selection is ambiguous".to_string(),
        RuntimeSelectionRule::NoSelection { .. } => "the selection found no entry".to_string(),
        RuntimeSelectionRule::Unknown { reason } => format!("the rule is unknown ({reason:?})"),
    }
}

/// What every view established together: the structural dimension of the one scan, and the ranges
/// **every** view scanned.
///
/// The runtime dimension is intersected on purpose. A profile that stopped early did not scan what
/// it skipped, so a matrix-level "scanned" claim may only carry the ranges all of them reached;
/// the ranges any one of them skipped are skipped for the matrix.
fn matrix_coverage(
    physical: &MultiReleasePhysicalEvidence,
    profiles: &[RuntimeMatrixProfile],
) -> Coverage {
    let mut scanned: BTreeMap<(String, u64, u64), (CoverageRange, usize)> = BTreeMap::new();
    let mut skipped: BTreeMap<(String, u64, u64), CoverageRange> = BTreeMap::new();
    let mut extensions: BTreeSet<String> = BTreeSet::new();
    let mut states: Vec<CoverageState> = Vec::new();
    for profile in profiles {
        let dimension = &profile.coverage.runtime_resolution;
        states.push(dimension.state);
        for range in &dimension.scanned {
            let key = (range.label.clone(), range.start, range.end);
            let entry = scanned.entry(key).or_insert((range.clone(), 0));
            entry.1 += 1;
        }
        for range in &dimension.skipped {
            skipped.insert((range.label.clone(), range.start, range.end), range.clone());
        }
        extensions.extend(dimension.uninterpreted_extensions.iter().cloned());
    }
    let every_view = profiles.len();
    let scanned: Vec<CoverageRange> = scanned
        .into_values()
        .filter(|(_, seen)| *seen == every_view)
        .map(|(range, _)| range)
        .collect();
    let state = match states.as_slice() {
        // No view: nothing was claimed, so the matrix claims nothing.
        [] => CoverageState::NotRequested,
        // The views agree on what they established; the matrix says exactly that.
        [first, rest @ ..] if rest.iter().all(|state| state == first) => *first,
        // They do not agree, so the joint claim is partial whatever the members claimed alone.
        _ => CoverageState::Partial,
    };
    Coverage {
        artifact_structural: multi_release::physical_coverage(physical)
            .artifact_structural
            .clone(),
        runtime_resolution: CoverageDimension {
            state,
            scanned,
            skipped: skipped.into_values().collect(),
            uninterpreted_extensions: extensions.into_iter().collect(),
        },
        dynamic_analysis: CoverageDimension::not_requested(),
    }
}

/// A path as diagnostic text: the name when it is text, its octets when it is not.
fn path_text(path: &[u8]) -> String {
    match std::str::from_utf8(path) {
        Ok(text) => text.to_string(),
        Err(_) => format!("{path:?}"),
    }
}
